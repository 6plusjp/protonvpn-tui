//! VPN client - wraps protonvpn CLI
//!
//! Uses the new `protonvpn` CLI commands:
//! - protonvpn countries     -> list countries
//! - protonvpn cities --country <CC>  -> list cities for a country
//! - protonvpn connect       -> connect to fastest server
//! - protonvpn connect --country <CC>  -> connect to a country
//! - protonvpn connect --city <city>  -> connect to a city
//! - protonvpn disconnect    -> disconnect

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use chrono::Utc;

use super::cache::ServerCache;
use super::types::{City, Server};
use crate::constants::vpn::{DISCONNECT_RETRY_COUNT, DISCONNECT_RETRY_DELAY_MS};
use crate::error::{AppError, AppResult};

/// VPN client for interacting with protonvpn CLI
#[derive(Debug)]
pub struct VpnClient {
    /// Path to protonvpn CLI (default: protonvpn)
    cli_path: String,
    cache: Mutex<ServerCache>,
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

        let cache = match ServerCache::load(cache_path.clone()) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to load server cache: {}, using empty cache", e);
                ServerCache::default()
            }
        };

        Self {
            cli_path: "protonvpn".to_string(),
            cache: Mutex::new(cache),
            cache_path,
        }
    }

    pub fn with_path(path: impl Into<String>) -> Self {
        let mut client = Self::new();
        client.cli_path = path.into();
        client
    }

    fn with_cache<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut ServerCache) -> T,
    {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| AppError::ConfigError(format!("Failed to lock cache: {}", e)))?;
        Ok(f(&mut cache))
    }

    fn save_cache(&self) -> AppResult<()> {
        let (cache_data, cache_path) = {
            let cache = self
                .cache
                .lock()
                .map_err(|e| AppError::ConfigError(format!("Failed to lock cache: {}", e)))?;
            tracing::debug!("Saving server cache to disk");
            (cache.clone(), self.cache_path.clone())
        };
        cache_data.save(cache_path)
    }

    pub fn connect(&self, target: &str) -> AppResult<(String, Option<String>)> {
        let output = Command::new(&self.cli_path)
            .args(["connect", "--country", target])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let (server_id, ip, _city, _country) = self.parse_connect_output(&stdout);

        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            target.to_string()
        };
        self.with_cache(|c| c.set_connected(final_server.clone(), ip.clone()))?;
        self.save_cache()?;

        Ok((final_server, ip))
    }

    pub fn connect_random(&self) -> AppResult<(String, Option<String>)> {
        let output = Command::new(&self.cli_path)
            .args(["connect", "--random"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let (server_id, ip, _city, _country) = self.parse_connect_output(&stdout);

        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            "Random Server".to_string()
        };
        self.with_cache(|c| c.set_connected(final_server.clone(), ip.clone()))?;
        self.save_cache()?;

        Ok((final_server, ip))
    }

    pub fn connect_city(&self, city: &str) -> AppResult<(String, Option<String>)> {
        let output = Command::new(&self.cli_path)
            .args(["connect", "--city", city])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let (server_id, ip, _city, _country) = self.parse_connect_output(&stdout);

        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            city.to_string()
        };
        self.with_cache(|c| c.set_connected(final_server.clone(), ip.clone()))?;
        self.save_cache()?;

        Ok((final_server, ip))
    }

    fn check_cli_error(
        &self,
        output: &std::process::Output,
        stdout: &str,
        stderr: &str,
    ) -> AppResult<()> {
        // If "Connected to" is in stdout, connection succeeded (ignore stderr errors)
        if stdout.to_lowercase().contains("connected to ") {
            return Ok(());
        }

        let combined = format!("{} {}", stdout, stderr).to_lowercase();

        if combined.contains("error:") || !output.status.success() {
            let error_msg = format!("{}\n{}", stdout.trim(), stderr.trim());
            return Err(AppError::ConnectionFailed(error_msg));
        }
        Ok(())
    }

    /// Parse connect output to extract server ID, IP, city and country
    pub(crate) fn parse_connect_output(
        &self,
        output: &str,
    ) -> (String, Option<String>, Option<String>, Option<String>) {
        let mut server_id = String::new();
        let mut ip = None;
        let mut city = None;
        let mut country = None;

        // Find "Connected to" line first - all info is in this single line
        let connected_line = output
            .lines()
            .find(|l| l.trim().starts_with("Connected to "));

        if let Some(line) = connected_line {
            let line = line.trim();

            // Extract server_id, city, country
            if let Some(rest) = line.strip_prefix("Connected to ") {
                if let Some(end_idx) = rest.find(" in ") {
                    server_id = rest[..end_idx].to_string();
                    let after_server = &rest[end_idx + 4..];

                    if let Some(period_idx) = after_server.find('.') {
                        let location_part = &after_server[..period_idx];
                        if let Some(last_comma_idx) = location_part.rfind(", ") {
                            city = Some(location_part[..last_comma_idx].to_string());
                            country = Some(location_part[last_comma_idx + 2..].to_string());
                        }
                    } else if let Some(last_comma_idx) = after_server.rfind(", ") {
                        city = Some(after_server[..last_comma_idx].to_string());
                        country = Some(after_server[last_comma_idx + 2..].to_string());
                    }
                }

                // Extract IP from same line
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
        }

        (server_id, ip, city, country)
    }

    pub fn disconnect(&self) -> AppResult<()> {
        let result = Command::new(&self.cli_path).args(["disconnect"]).output();
        if let Err(e) = result {
            tracing::warn!("Failed to execute disconnect command: {}", e);
        }

        for _ in 0..DISCONNECT_RETRY_COUNT {
            if !self.is_connected() {
                self.with_cache(|c| c.set_disconnected())?;
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

    pub fn get_connected_server(&self) -> Option<String> {
        self.with_cache(|c| c.connected_server.clone())
            .ok()
            .flatten()
    }

    pub fn get_vpn_ip(&self) -> Option<String> {
        self.with_cache(|c| c.connected_ip.clone()).ok().flatten()
    }

    pub fn matches_ip(&self, ip: &str) -> bool {
        self.with_cache(|c| c.matches_ip(ip)).is_ok_and(|r| r)
    }

    pub fn get_countries(&self) -> AppResult<HashMap<String, String>> {
        let is_empty = self.with_cache(|c| c.countries.is_empty())?;
        if is_empty {
            return self.refresh_countries();
        }
        self.with_cache(|c| c.countries.clone())
    }

    pub fn list_countries(&self) -> AppResult<HashMap<String, String>> {
        let is_stale = self.with_cache(|c| c.is_stale())?;
        let is_empty = self.with_cache(|c| c.countries.is_empty())?;
        if !is_stale && !is_empty {
            return self.with_cache(|c| c.countries.clone());
        }

        self.refresh_countries()
    }

    pub fn refresh_countries(&self) -> AppResult<HashMap<String, String>> {
        let output = Command::new(&self.cli_path)
            .args(["countries"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let countries = self.parse_countries(&stdout);

        self.with_cache(|c| {
            c.countries = countries.clone();
            c.last_updated = Some(Utc::now());
        })?;
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

    pub fn list_cities(&self, country_code: &str) -> AppResult<Vec<String>> {
        let cities = self.list_cities_with_features(country_code)?;
        Ok(cities.into_iter().map(|c| c.name).collect())
    }

    pub fn list_cities_with_features(&self, country_code: &str) -> AppResult<Vec<City>> {
        let is_stale = self.with_cache(|c| c.is_stale())?;
        let has_cached = self
            .with_cache(|c| c.cities.contains_key(country_code))
            .is_ok_and(|r| r);

        if !is_stale && has_cached {
            if let Ok(Some(cities)) = self.with_cache(|c| c.cities.get(country_code).cloned()) {
                return Ok(cities);
            }
        }

        let output = Command::new(&self.cli_path)
            .args(["cities", "--country", country_code])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cities = self.parse_cities_with_features(&stdout);

        self.with_cache(|c| {
            c.cities.insert(country_code.to_string(), cities.clone());
            c.last_updated = Some(Utc::now());
        })?;
        self.save_cache()?;

        Ok(cities)
    }

    /// Parse cities output with features
    pub(crate) fn parse_cities_with_features(&self, output: &str) -> Vec<City> {
        let mut cities = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip lines that don't start with a letter (e.g., "------")
            if !line.starts_with(|c: char| c.is_alphabetic()) {
                continue;
            }

            // Skip header lines:
            // - "Cities in United Arab Emirates:" (title line)
            // - "City     Features" (column header - features is just "Features")
            if line.starts_with("Cities") || line.starts_with("City") {
                continue;
            }

            // Skip update messages
            if line.starts_with("Server list") {
                continue;
            }

            let mut chars = line.char_indices().peekable();
            let mut name_end = None;

            while let Some((start, c)) = chars.next() {
                if c.is_whitespace() {
                    let mut consecutive = 1;
                    while let Some(&(_, next_c)) = chars.peek() {
                        if next_c.is_whitespace() {
                            consecutive += 1;
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if consecutive >= 2 {
                        name_end = Some(start);
                        break;
                    }
                }
            }

            let (name, features_str) = match name_end {
                Some(pos) => (line[..pos].to_string(), line[pos..].trim()),
                None => (line.to_string(), ""),
            };

            // Features are comma-separated: "P2P, Secure Core" → ["P2P", "Secure Core"]
            let features: Vec<String> = features_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            cities.push(City::with_features(name, features));
        }

        cities
    }

    pub fn list_servers(&self) -> AppResult<Vec<Server>> {
        let is_stale = self.with_cache(|c| c.is_stale())?;
        let is_empty = self.with_cache(|c| c.countries.is_empty())?;
        if !is_stale && !is_empty {
            let countries = self.with_cache(|c| c.countries.clone())?;
            return Ok(self.countries_to_servers(&countries));
        }

        self.refresh_servers()
    }

    pub fn get_servers(&self) -> Vec<Server> {
        let countries = match self.with_cache(|c| c.countries.clone()) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("Failed to get countries from cache: {}", e);
                HashMap::new()
            }
        };
        self.countries_to_servers(&countries)
    }

    pub fn refresh_servers(&self) -> AppResult<Vec<Server>> {
        let countries = self.refresh_countries()?;
        Ok(self.countries_to_servers(&countries))
    }

    fn countries_to_servers(&self, countries: &HashMap<String, String>) -> Vec<Server> {
        let cities_map = match self.with_cache(|c| c.cities.clone()) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("Failed to get cities from cache: {}", e);
                HashMap::new()
            }
        };
        let servers: Vec<Server> = countries
            .iter()
            .map(|(code, name)| Server {
                id: code.clone(),
                country: name.clone(),
                cities: cities_map.get(code).cloned().unwrap_or_default(),
            })
            .collect();

        servers
    }

    pub fn status(&self) -> String {
        if let Ok(Some(server)) = self.with_cache(|c| c.connected_server.clone()) {
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

    /// Set custom DNS servers
    pub fn set_custom_dns(&self, dns_list: &str) -> AppResult<String> {
        let output = Command::new(&self.cli_path)
            .args(["config", "set", "custom-dns", "on", "--dns", dns_list])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok(format!("DNS set to {}", dns_list))
    }

    /// Disable custom DNS
    pub fn disable_custom_dns(&self) -> AppResult<String> {
        let output = Command::new(&self.cli_path)
            .args(["config", "set", "custom-dns", "off"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok("Custom DNS disabled".to_string())
    }

    #[cfg(test)]
    pub fn with_test_servers(servers: Vec<Server>) -> Self {
        use std::collections::HashMap;

        let mut countries: HashMap<String, String> = HashMap::new();
        let mut cities_map: HashMap<String, Vec<City>> = HashMap::new();

        for server in &servers {
            countries.insert(server.id.clone(), server.country.clone());
            if !server.cities.is_empty() {
                cities_map.insert(server.id.clone(), server.cities.clone());
            }
        }

        let client = Self::new();
        client.cache.lock().unwrap().countries = countries;
        client.cache.lock().unwrap().cities = cities_map;

        client
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
        let cities = client.parse_cities_with_features(output);

        assert_eq!(cities.len(), 1);
        assert_eq!(cities[0].name, "Tokyo");
    }

    #[test]
    fn test_parse_cities_multiple() {
        let client = VpnClient::new();
        let output = "Tokyo\nOsaka\nKyoto\nSapporo";
        let cities = client.parse_cities_with_features(output);

        assert_eq!(cities.len(), 4);
        assert_eq!(cities[0].name, "Tokyo");
        assert_eq!(cities[1].name, "Osaka");
        assert_eq!(cities[2].name, "Kyoto");
        assert_eq!(cities[3].name, "Sapporo");
    }

    #[test]
    fn test_parse_cities_with_whitespace() {
        let client = VpnClient::new();
        let output = "  Tokyo  \n  Osaka  \n  ";
        let cities = client.parse_cities_with_features(output);

        assert_eq!(cities.len(), 2);
        assert_eq!(cities[0].name, "Tokyo");
        assert_eq!(cities[1].name, "Osaka");
    }

    #[test]
    fn test_parse_cities_multi_word_with_features() {
        let client = VpnClient::new();
        let output = "Tel Aviv  P2P, Secure Core\nOsaka  P2P";
        let cities = client.parse_cities_with_features(output);

        assert_eq!(cities.len(), 2);
        assert_eq!(cities[0].name, "Tel Aviv");
        assert_eq!(cities[0].features, vec!["P2P", "Secure Core"]);
        assert_eq!(cities[1].name, "Osaka");
        assert_eq!(cities[1].features, vec!["P2P"]);
    }

    #[test]
    fn test_parse_cities_empty() {
        let client = VpnClient::new();
        let output = "";
        let cities = client.parse_cities_with_features(output);

        assert!(cities.is_empty());
    }

    #[test]
    fn test_parse_connect_output_no_ip() {
        let client = VpnClient::new();
        let output = r#"Connected to DE#200 in Berlin, Germany.
No IP address found."#;
        let (server_id, ip, city, country) = client.parse_connect_output(output);

        assert_eq!(server_id, "DE#200");
        assert_eq!(ip, None);
        assert_eq!(city, Some("Berlin".to_string()));
        assert_eq!(country, Some("Germany".to_string()));
    }

    #[test]
    fn test_parse_connect_output_private_ip_ignored() {
        let client = VpnClient::new();
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 10.0.0.1"#;
        let (server_id, ip, city, country) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#379");
        assert_eq!(ip, None);
        assert_eq!(city, Some("Tokyo".to_string()));
        assert_eq!(country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_no_server() {
        let client = VpnClient::new();
        let output = "Connection failed. Please try again.";
        let (server_id, ip, city, country) = client.parse_connect_output(output);

        assert!(server_id.is_empty());
        assert_eq!(ip, None);
        assert_eq!(city, None);
        assert_eq!(country, None);
    }

    #[test]
    fn test_parse_connect_output_same_line_ip() {
        let client = VpnClient::new();
        let output =
            r#"Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144."#;
        let (server_id, ip, city, country) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#374");
        assert_eq!(ip, Some("159.26.119.144".to_string()));
        assert_eq!(city, Some("Tokyo".to_string()));
        assert_eq!(country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_with_error_traceback() {
        let client = VpnClient::new();
        let output = r#"2026-03-06T06:48:22.914403+00:00 | concurrent.futures:336 | ERROR | exception calling callback
Traceback (most recent call last):
  File "/usr/lib/python3.14/site-packages/proton/vpn/backend/networkmanager/core/local_agent/listener.py", line 51, in connect
    certificate)
    ^^^^^^^^^^^^
local_agent.LocalAgentError: Tokio(Custom { kind: InvalidData, error: InvalidCertificate(NotValidForName) })

Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144."#;
        let (server_id, ip, city, country) = client.parse_connect_output(output);

        assert_eq!(server_id, "JP#374");
        assert_eq!(ip, Some("159.26.119.144".to_string()));
        assert_eq!(city, Some("Tokyo".to_string()));
        assert_eq!(country, Some("Japan".to_string()));
    }
}
