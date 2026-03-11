//! VPN client - wraps protonvpn CLI
//!
//! Uses the new `protonvpn` CLI commands:
//! - protonvpn countries     -> list countries
//! - protonvpn cities --country &lt;CC&gt;  -> list cities for a country
//! - protonvpn connect       -> connect to fastest server
//! - protonvpn connect --country &lt;CC&gt;  -> connect to a country
//! - protonvpn connect --city &lt;city&gt;  -> connect to a city
//! - protonvpn disconnect    -> disconnect

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use chrono::Utc;

use super::cache::{countries_to_servers, ServerCache};
use super::types::{
    parse_cities_with_features, parse_connect_output, parse_countries, City, Server,
};

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

    /// Get currently connected server name
    pub fn get_connected_server(&self) -> Option<String> {
        self.with_cache(|c| c.connected_server.clone())
            .ok()
            .flatten()
    }

    /// Get VPN IP address if connected
    pub fn get_vpn_ip(&self) -> Option<String> {
        self.with_cache(|c| c.connected_ip.clone()).ok().flatten()
    }

    pub fn matches_ip(&self, ip: &str) -> bool {
        self.with_cache(|c| c.matches_ip(ip)).is_ok_and(|r| r)
    }

    pub fn is_cli_unavailable(&self) -> bool {
        self.with_cache(|c| c.is_cli_unavailable()).unwrap_or(false)
    }

    /// Connect to a server by country code
    pub fn connect_country(&self, target: &str) -> AppResult<(String, Option<String>)> {
        let output = Command::new(&self.cli_path)
            .args(["connect", "--country", target])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let (server_id, ip, _city, _country) = parse_connect_output(&stdout);

        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            target.to_string()
        };
        self.with_cache(|c| c.set_connected(final_server.clone(), ip.clone()))?;
        self.save_cache()?;

        Ok((final_server, ip))
    }

    /// Connect to a random server
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

        let (server_id, ip, _city, _country) = parse_connect_output(&stdout);

        let final_server = if !server_id.is_empty() {
            server_id
        } else {
            "Random Server".to_string()
        };
        self.with_cache(|c| c.set_connected(final_server.clone(), ip.clone()))?;
        self.save_cache()?;

        Ok((final_server, ip))
    }

    /// Connect to a server by city name
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

        let (server_id, ip, _city, _country) = parse_connect_output(&stdout);

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

    /// Disconnect from VPN
    pub fn disconnect(&self) -> AppResult<()> {
        let output = Command::new(&self.cli_path)
            .args(["disconnect"])
            .output()
            .map_err(|e| {
                tracing::warn!("Failed to execute disconnect command: {}", e);
                AppError::ConnectionFailed(format!("Failed to execute disconnect: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("{}{}", stdout, stderr);

        // Exit code 0 = successfully disconnected
        // Exit code 1 = already disconnected or error
        if output.status.success() {
            let server_info = self
                .with_cache(|c| c.connected_server.clone())
                .unwrap_or_default()
                .unwrap_or_else(|| "VPN".to_string());
            self.with_cache(|c| c.set_disconnected())?;
            self.save_cache()?;
            tracing::info!("Disconnected from {}", server_info);
            return Ok(());
        }

        // Handle "already disconnected" gracefully - not an error
        if combined_output
            .to_lowercase()
            .contains("already disconnected")
            || combined_output.to_lowercase().contains("not connected")
        {
            self.with_cache(|c| c.set_disconnected())?;
            self.save_cache()?;
            tracing::info!("Already disconnected from VPN");
            return Ok(());
        }

        Err(AppError::ConnectionFailed(format!(
            "Failed to disconnect: {}",
            combined_output.trim()
        )))
    }

    /// Check if VPN is connected (by checking proton0 interface)
    pub fn is_connected(&self) -> bool {
        // Check proton0 interface for active connection
        match Command::new("ip")
            .args(["addr", "show", "proton0"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("inet ")
            }
            Err(e) => {
                tracing::debug!("Failed to check proton0 interface: {}", e);
                false
            }
        }
    }

    pub fn refresh_countries(&self) -> AppResult<HashMap<String, String>> {
        let output = Command::new(&self.cli_path)
            .args(["countries"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("protonvpn countries failed: {}", stderr);
            return self.use_fallback_countries();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let countries = parse_countries(&stdout);

        self.with_cache(|c| {
            c.countries = countries.clone();
            c.last_updated = Some(Utc::now());
            c.cli_unavailable = false;
        })?;
        self.save_cache()?;

        Ok(countries)
    }

    fn use_fallback_countries(&self) -> AppResult<HashMap<String, String>> {
        use super::cache::FALLBACK_COUNTRIES;

        let fallback: HashMap<String, String> = FALLBACK_COUNTRIES
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        self.with_cache(|c| {
            c.countries = fallback.clone();
            c.cli_unavailable = true;
        })?;
        self.save_cache()?;

        tracing::info!("Using fallback countries (CLI unavailable)");
        Ok(fallback)
    }

    pub fn get_cached_cities(&self, country_code: &str) -> Option<Vec<City>> {
        let has_cached = self
            .with_cache(|c| c.cities.contains_key(country_code))
            .ok()?;
        if !has_cached {
            return None;
        }
        self.with_cache(|c| c.cities.get(country_code).cloned())
            .ok()
            .flatten()
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
        let cities = parse_cities_with_features(&stdout);

        self.with_cache(|c| {
            c.cities.insert(country_code.to_string(), cities.clone());
            c.last_updated = Some(Utc::now());
        })?;
        self.save_cache()?;

        Ok(cities)
    }

    pub fn clear_cities_cache(&self, country_code: &str) -> AppResult<()> {
        self.with_cache(|c| {
            c.cities.remove(country_code);
        })?;
        self.save_cache()?;
        Ok(())
    }

    /// List all available servers
    pub fn list_servers(&self) -> AppResult<Vec<Server>> {
        let is_stale = self.with_cache(|c| c.is_stale())?;
        let is_empty = self.with_cache(|c| c.countries.is_empty())?;
        if !is_stale && !is_empty {
            let countries = self.with_cache(|c| c.countries.clone())?;
            let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();
            return Ok(countries_to_servers(&countries, &cities));
        }

        self.refresh_servers()
    }

    /// Get cached servers (non-refreshing)
    pub fn get_servers(&self) -> Vec<Server> {
        let countries = match self.with_cache(|c| c.countries.clone()) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("Failed to get countries from cache: {}", e);
                HashMap::new()
            }
        };
        let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();
        countries_to_servers(&countries, &cities)
    }

    pub fn refresh_servers(&self) -> AppResult<Vec<Server>> {
        let countries = self.refresh_countries()?;
        let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();
        Ok(countries_to_servers(&countries, &cities))
    }

    /// Set a configuration option via `protonvpn config set <setting> <value>`
    pub fn set_config(&self, setting: &str, value: &str) -> AppResult<String> {
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
        self.set_config("kill-switch", new_value)
    }

    /// Toggle IPv6 (off <-> on)
    pub fn toggle_ipv6(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.set_config("ipv6", new_value)
    }

    /// Toggle moderate NAT (off <-> on)
    pub fn toggle_moderate_nat(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.set_config("moderate-nat", new_value)
    }

    /// Toggle VPN accelerator (off <-> on)
    pub fn toggle_vpn_accelerator(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.set_config("vpn-accelerator", new_value)
    }

    /// Toggle port forwarding (off <-> on)
    pub fn toggle_port_forwarding(&self, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.set_config("port-forwarding", new_value)
    }

    /// Set NetShield mode (off -> malware-only -> malware-ads-trackers -> off)
    pub fn set_netshield(&self, current: Option<i32>, _next: i32) -> AppResult<String> {
        let new_value = match current.unwrap_or(0) {
            0 => "malware-only",
            1 => "malware-ads-trackers",
            _ => "off",
        };
        self.set_config("netshield", new_value)
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
        // Use match for cleaner error handling in test code
        if let Ok(mut cache) = client.cache.lock() {
            cache.countries = countries;
            cache.cities = cities_map;
        }

        client
    }
}
