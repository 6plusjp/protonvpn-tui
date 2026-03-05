//! VPN client - wraps protonvpn CLI
//!
//! Uses the new `protonvpn` CLI commands:
//! - protonvpn countries     -> list countries
//! - protonvpn cities <CC>  -> list cities for a country
//! - protonvpn connect       -> connect to fastest server
//! - protonvpn connect <CC>  -> connect to a country
//! - protonvpn disconnect    -> disconnect

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;

use super::cache::ServerCache;
use super::types::{Server, ServerFeatures};
use crate::constants::vpn::{DISCONNECT_RETRY_COUNT, DISCONNECT_RETRY_DELAY_MS};
use crate::error::{AppError, AppResult};

/// VPN client for interacting with protonvpn CLI
#[derive(Debug, Clone)]
pub struct VpnClient {
    /// Path to protonvpn CLI (default: protonvpn)
    cli_path: String,
    /// Server data cache
    cache: ServerCache,
    /// Cache file path
    cache_path: PathBuf,
}

impl Default for VpnClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VpnClient {
    pub fn new() -> Self {
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("protonvpn-tui")
            .join("server_cache.toml");

        let cache = ServerCache::load(cache_path.clone()).unwrap_or_default();

        Self {
            cli_path: "protonvpn".to_string(),
            cache,
            cache_path,
        }
    }

    pub fn with_path(path: impl Into<String>) -> Self {
        let mut client = Self::new();
        client.cli_path = path.into();
        client
    }

    /// Save cache to disk
    fn save_cache(&self) -> AppResult<()> {
        self.cache.save(self.cache_path.clone())?;
        Ok(())
    }

    /// Connect to a server by country code
    pub fn connect(&mut self, target: &str) -> AppResult<(String, Option<String>)> {
        // Use new CLI syntax: protonvpn connect --country <country_code>
        let output = Command::new(&self.cli_path)
            .args(["connect", "--country", target])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let combined = format!("{} {}", stdout, stderr);
        let (server_id, ip) = self.parse_connect_output(&combined);

        // Update local connection status with actual server info
        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            target.to_string()
        };
        self.cache.set_connected(final_server.clone(), ip.clone());
        self.save_cache()?;

        Ok((final_server, ip))
    }

    pub fn connect_random(&mut self) -> AppResult<(String, Option<String>)> {
        let output = Command::new(&self.cli_path)
            .args(["connect", "--random"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let combined = format!("{} {}", stdout, stderr);
        let (server_id, ip) = self.parse_connect_output(&combined);

        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            "Random Server".to_string()
        };
        self.cache.set_connected(final_server.clone(), ip.clone());
        self.save_cache()?;

        Ok((final_server, ip))
    }

    fn check_cli_error(
        &self,
        output: &std::process::Output,
        stdout: &str,
        stderr: &str,
    ) -> AppResult<()> {
        let combined = format!("{} {}", stdout, stderr).to_lowercase();

        if combined.contains("error:") || !output.status.success() {
            let error_msg = format!("{}\n{}", stdout.trim(), stderr.trim());
            return Err(AppError::ConnectionFailed(error_msg));
        }
        Ok(())
    }

    /// Parse connect output to extract server ID and IP
    pub(crate) fn parse_connect_output(&self, output: &str) -> (String, Option<String>) {
        let mut server_id = String::new();
        let mut ip = None;

        for line in output.lines() {
            let line = line.trim();

            // Try various IP address patterns
            if line.contains("IP address") || line.contains("IP:") || line.contains("address is") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    let part_clean = part.trim_end_matches(':');
                    if (part_clean == "is" || part_clean == "IP" || part_clean == "address")
                        && i + 1 < parts.len()
                    {
                        let potential_ip = parts[i + 1].trim_end_matches('.');
                        if potential_ip.contains('.')
                            && potential_ip.chars().filter(|&c| c == '.').count() == 3
                            && !potential_ip.starts_with("10.")
                            && !potential_ip.starts_with("172.")
                            && !potential_ip.starts_with("192.168")
                        {
                            ip = Some(potential_ip.to_string());
                            break;
                        }
                    }
                }
            }

            if line.starts_with("Connected to ") {
                // Parse: "Connected to JP#379 in Tokyo, Japan."
                if let Some(rest) = line.strip_prefix("Connected to ") {
                    if let Some(end_idx) = rest.find(" in ") {
                        server_id = rest[..end_idx].to_string();
                    }
                }
            }
        }

        (server_id, ip)
    }

    /// Disconnect from VPN
    pub fn disconnect(&mut self) -> AppResult<()> {
        let _ = Command::new(&self.cli_path).args(["disconnect"]).output();

        for _ in 0..DISCONNECT_RETRY_COUNT {
            if !self.is_connected() {
                self.cache.set_disconnected();
                self.save_cache()?;
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(DISCONNECT_RETRY_DELAY_MS));
        }

        Err(AppError::ConnectionFailed(
            "Failed to disconnect".to_string(),
        ))
    }

    /// Check if connected using system-level check (proton0 interface)
    pub fn is_connected(&self) -> bool {
        // Check proton0 interface for active connection
        match Command::new("ip")
            .args(["addr", "show", "proton0"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // If proton0 has an IP address, we're connected
                stdout.contains("inet ")
            }
            Err(_) => false,
        }
    }

    /// Get connected server (from local cache)
    pub fn get_connected_server(&self) -> Option<String> {
        self.cache.connected_server.clone()
    }

    /// Get cached IP address from connection (not proton0)
    pub fn get_vpn_ip(&self) -> Option<String> {
        self.cache.connected_ip.clone()
    }

    /// Check if IP matches cached connection
    pub fn matches_ip(&self, ip: &str) -> bool {
        self.cache.matches_ip(ip)
    }

    /// Get cached countries, refresh from CLI if empty
    pub fn get_countries(&mut self) -> AppResult<HashMap<String, String>> {
        if self.cache.countries.is_empty() {
            return self.refresh_countries();
        }
        Ok(self.cache.countries.clone())
    }

    /// List countries - returns cached if available, otherwise fetches from CLI
    pub fn list_countries(&mut self) -> AppResult<HashMap<String, String>> {
        // Return cached if valid
        if !self.cache.is_stale() && !self.cache.countries.is_empty() {
            return Ok(self.cache.countries.clone());
        }

        self.refresh_countries()
    }

    /// Refresh countries from CLI and update cache
    pub fn refresh_countries(&mut self) -> AppResult<HashMap<String, String>> {
        let output = Command::new(&self.cli_path)
            .args(["countries"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let countries = self.parse_countries(&stdout);

        // Update cache
        self.cache.countries = countries.clone();
        self.cache.last_updated = Some(Utc::now());
        self.save_cache()?;

        Ok(countries)
    }

    /// Parse countries output
    pub(crate) fn parse_countries(&self, output: &str) -> HashMap<String, String> {
        let mut countries = HashMap::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // Skip header lines
            if line.starts_with("Country") || line.starts_with('-') {
                continue;
            }
            // Skip update messages
            if line.starts_with("Server list") {
                continue;
            }

            // Parse: "Country Name             XX" (code at the end)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Last part is the code
                let code = parts.last().unwrap();
                // Everything before is the country name
                let name = parts[..parts.len() - 1].join(" ");
                if !code.is_empty() && !name.is_empty() && code.len() <= 3 {
                    countries.insert(code.to_string(), name.to_string());
                }
            }
        }

        countries
    }

    /// List cities in a country
    pub fn list_cities(&mut self, country_code: &str) -> AppResult<Vec<String>> {
        // Return cached if valid
        if let Some(cities) = self.cache.cities.get(country_code) {
            return Ok(cities.clone());
        }

        let output = Command::new(&self.cli_path)
            .args(["cities", country_code])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cities = self.parse_cities(&stdout);

        // Update cache
        self.cache
            .cities
            .insert(country_code.to_string(), cities.clone());
        self.save_cache()?;

        Ok(cities)
    }

    /// Parse cities output
    pub(crate) fn parse_cities(&self, output: &str) -> Vec<String> {
        let mut cities = Vec::new();

        // Output format: "Tokyo\nOsaka\n..."
        for line in output.lines() {
            let city = line.trim().to_string();
            if !city.is_empty() {
                cities.push(city);
            }
        }

        cities
    }

    /// List available servers (from countries)
    pub fn list_servers(&mut self) -> AppResult<Vec<Server>> {
        // Return cached if valid
        if !self.cache.is_stale() && !self.cache.countries.is_empty() {
            return Ok(self.countries_to_servers(&self.cache.countries));
        }

        self.refresh_servers()
    }

    /// Get cached servers (fast, no CLI call)
    pub fn get_servers(&self) -> Vec<Server> {
        self.countries_to_servers(&self.cache.countries)
    }

    /// Refresh servers from CLI
    pub fn refresh_servers(&mut self) -> AppResult<Vec<Server>> {
        let countries = self.refresh_countries()?;
        Ok(self.countries_to_servers(&countries))
    }

    /// Convert countries hashmap to server list
    fn countries_to_servers(&self, countries: &HashMap<String, String>) -> Vec<Server> {
        let mut servers: Vec<Server> = countries
            .iter()
            .map(|(code, name)| Server {
                id: code.clone(),
                name: format!("{} - {}", name, code),
                country: name.clone(),
                city: String::new(),
                features: ServerFeatures::default(),
            })
            .collect();

        if servers.is_empty() {
            servers = self.mock_servers();
        }

        servers
    }

    /// Mock servers for testing
    fn mock_servers(&self) -> Vec<Server> {
        vec![
            Server {
                id: "JP".to_string(),
                name: "Japan - JP".to_string(),
                country: "Japan".to_string(),
                city: String::new(),
                features: ServerFeatures::default(),
            },
            Server {
                id: "US".to_string(),
                name: "United States - US".to_string(),
                country: "United States".to_string(),
                city: String::new(),
                features: ServerFeatures::default(),
            },
            Server {
                id: "DE".to_string(),
                name: "Germany - DE".to_string(),
                country: "Germany".to_string(),
                city: String::new(),
                features: ServerFeatures::default(),
            },
        ]
    }

    /// Get connection status (from local cache)
    pub fn status(&self) -> String {
        if let Some(server) = &self.cache.connected_server {
            format!("Connected to: {}", server)
        } else {
            "Disconnected".to_string()
        }
    }

    /// Set a configuration option via `protonvpn config set <setting> <value>`
    pub fn config_set(&self, setting: &str, value: &str) -> AppResult<String> {
        let output = Command::new(&self.cli_path)
            .args(["config", "set", setting, value])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok(format!("{} {}", stdout, stderr).trim().to_string())
    }

    /// Toggle kill switch (off <-> standard)
    pub fn toggle_killswitch(&self, current: Option<i32>) -> AppResult<String> {
        let new_value = if current == Some(1) {
            "off"
        } else {
            "standard"
        };
        self.config_set("kill-switch", new_value)
    }

    /// Toggle IPv6 (off <-> on)
    pub fn toggle_ipv6(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.config_set("ipv6", new_value)
    }

    /// Toggle moderate NAT (off <-> on)
    pub fn toggle_moderate_nat(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.config_set("moderate-nat", new_value)
    }

    /// Toggle VPN accelerator (off <-> on)
    pub fn toggle_vpn_accelerator(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.config_set("vpn-accelerator", new_value)
    }

    /// Toggle port forwarding (off <-> on)
    pub fn toggle_port_forwarding(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.config_set("port-forwarding", new_value)
    }

    /// Set NetShield mode (off -> malware-only -> malware-ads-trackers -> off)
    pub fn set_netshield(&self, current: Option<i32>, _next: i32) -> AppResult<String> {
        let new_value = match current.unwrap_or(0) {
            0 => "malware-only",
            1 => "malware-ads-trackers",
            _ => "off",
        };
        self.config_set("netshield", new_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_countries_single() {
        let client = VpnClient::new();
        let output = "Japan               JP";
        let countries = client.parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
    }

    #[test]
    fn test_parse_countries_multiple() {
        let client = VpnClient::new();
        let output = r#"Japan               JP
United States       US
Germany             DE
United Kingdom      GB"#;
        let countries = client.parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
        assert_eq!(countries.get("US"), Some(&"United States".to_string()));
        assert_eq!(countries.get("DE"), Some(&"Germany".to_string()));
        assert_eq!(countries.get("GB"), Some(&"United Kingdom".to_string()));
    }

    #[test]
    fn test_parse_countries_skips_header() {
        let client = VpnClient::new();
        let output = r#"Country             Code
------------------  ----
Japan               JP"#;
        let countries = client.parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
        assert!(!countries.contains_key("Country"));
        assert!(!countries.contains_key("Code"));
    }

    #[test]
    fn test_parse_countries_skips_dashes() {
        let client = VpnClient::new();
        let output = r#"------------------  ----
Japan               JP"#;
        let countries = client.parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
    }

    #[test]
    fn test_parse_countries_empty() {
        let client = VpnClient::new();
        let output = "";
        let countries = client.parse_countries(output);

        assert!(countries.is_empty());
    }

    #[test]
    fn test_parse_countries_with_server_list_message() {
        let client = VpnClient::new();
        let output = r#"Server list is currently being updated. Please try again later.
Japan               JP"#;
        let countries = client.parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
    }

    #[test]
    fn test_parse_cities_single() {
        let client = VpnClient::new();
        let output = "Tokyo";
        let cities = client.parse_cities(output);

        assert_eq!(cities, vec!["Tokyo"]);
    }

    #[test]
    fn test_parse_cities_multiple() {
        let client = VpnClient::new();
        let output = "Tokyo\nOsaka\nKyoto\nSapporo";
        let cities = client.parse_cities(output);

        assert_eq!(cities, vec!["Tokyo", "Osaka", "Kyoto", "Sapporo"]);
    }

    #[test]
    fn test_parse_cities_with_whitespace() {
        let client = VpnClient::new();
        let output = "  Tokyo  \n  Osaka  \n  ";
        let cities = client.parse_cities(output);

        assert_eq!(cities, vec!["Tokyo", "Osaka"]);
    }

    #[test]
    fn test_parse_cities_empty() {
        let client = VpnClient::new();
        let output = "";
        let cities = client.parse_cities(output);

        assert!(cities.is_empty());
    }

    #[test]
    fn test_parse_connect_output_with_ip() {
        let client = VpnClient::new();
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 123.45.67.89
Enjoy your privacy."#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#379");
        assert_eq!(ip, Some("123.45.67.89".to_string()));
    }

    #[test]
    fn test_parse_connect_output_ip_with_colon() {
        let client = VpnClient::new();
        let output = r#"Connected to US#100 in New York, United States.
IP: 98.76.54.321
Stay secure."#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert_eq!(server_id, "US#100");
        assert_eq!(ip, Some("98.76.54.321".to_string()));
    }

    #[test]
    fn test_parse_connect_output_no_ip() {
        let client = VpnClient::new();
        let output = r#"Connected to DE#200 in Berlin, Germany.
No IP address found."#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert_eq!(server_id, "DE#200");
        assert_eq!(ip, None);
    }

    #[test]
    fn test_parse_connect_output_private_ip_ignored() {
        let client = VpnClient::new();
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 10.0.0.1"#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#379");
        assert_eq!(ip, None);
    }

    #[test]
    fn test_parse_connect_output_172_16_ip_ignored() {
        let client = VpnClient::new();
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 172.16.0.1"#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#379");
        assert_eq!(ip, None);
    }

    #[test]
    fn test_parse_connect_output_192_168_ip_ignored() {
        let client = VpnClient::new();
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 192.168.1.1"#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#379");
        assert_eq!(ip, None);
    }

    #[test]
    fn test_parse_connect_output_no_server() {
        let client = VpnClient::new();
        let output = "Connection failed. Please try again.";
        let (server_id, ip) = client.parse_connect_output(output);

        assert!(server_id.is_empty());
        assert_eq!(ip, None);
    }

    #[test]
    fn test_parse_connect_output_ip_address_format() {
        let client = VpnClient::new();
        let output = r#"The IP address is 123.45.67.89."#;
        let (server_id, ip) = client.parse_connect_output(output);

        assert!(server_id.is_empty());
        assert_eq!(ip, Some("123.45.67.89".to_string()));
    }
}
