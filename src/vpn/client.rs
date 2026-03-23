//! VPN client - wraps protonvpn CLI
//!
//! Uses the new `protonvpn` CLI commands:
//! - protonvpn countries list     -> list countries
//! - protonvpn cities list <CC>  -> list cities for a country
//! - protonvpn connect           -> connect to fastest server
//! - protonvpn connect --country <CC>  -> connect to a country
//! - protonvpn connect --city <city>   -> connect to a city
//! - protonvpn connect --fastest      -> connect to fastest server
//! - protonvpn connect --p2p            -> connect to fastest P2P server
//! - protonvpn connect --tor            -> connect to fastest Tor server
//! - protonvpn connect --securecore     -> connect to fastest Secure Core server
//! - protonvpn disconnect              -> disconnect

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use chrono::Utc;

use super::cache::{countries_to_servers, ServerCache};
use super::types::{
    parse_cities_with_features, parse_connect_output, parse_countries, City, ConnectResult, Server,
};

use crate::error::{categorize_error, AppError, AppResult};
use crate::paths;

/// VPN client for interacting with protonvpn CLI
#[derive(Debug)]
pub struct VpnClient {
    /// Path to protonvpn CLI (default: protonvpn)
    cli_path: String,
    cache: Mutex<ServerCache>,
    cache_path: PathBuf,
}

impl VpnClient {
    fn persistence_file_path(&self) -> PathBuf {
        paths::proton_connection_persistence_fallback()
    }
}

impl Default for VpnClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VpnClient {
    pub fn new() -> Self {
        let cache_path = paths::cache_path().unwrap_or_else(|| PathBuf::from("."));

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
            .map_err(|e| AppError::CommandFailed(format!("Failed to lock cache: {}", e)))?;
        Ok(f(&mut cache))
    }

    fn save_cache(&self) -> AppResult<()> {
        let (cache_data, cache_path) = {
            let cache = self
                .cache
                .lock()
                .map_err(|e| AppError::CommandFailed(format!("Failed to lock cache: {}", e)))?;
            tracing::debug!("Saving server cache to disk");
            (cache.clone(), self.cache_path.clone())
        };
        cache_data.save(cache_path)
    }

    /// Get cached servers (non-refreshing)
    pub fn servers(&self) -> Vec<Server> {
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

    pub fn matches_ip(&self, ip: &str) -> bool {
        self.with_cache(|c| c.matches_ip(ip)).is_ok_and(|r| r)
    }

    pub fn is_cli_unavailable(&self) -> bool {
        self.with_cache(|c| c.is_cli_unavailable()).unwrap_or(false)
    }

    /// Connect to a server by country code
    pub fn connect_country(&self, target: &str) -> AppResult<ConnectResult> {
        self.connect_with_args("--country", target, &[target])
    }

    pub fn connect_random(&self) -> AppResult<ConnectResult> {
        self.connect_with_args("--random", "Random Server", &[])
    }

    /// Connect to a server by city name
    pub fn connect_city(&self, city_arg: &str) -> AppResult<ConnectResult> {
        self.connect_with_args("--city", city_arg, &[city_arg])
    }

    pub fn connect_fastest(&self) -> AppResult<ConnectResult> {
        self.connect_with_args("", "Fastest Server", &[])
    }

    pub fn connect_p2p(&self) -> AppResult<ConnectResult> {
        self.connect_with_args("--p2p", "P2P Server", &[])
    }

    pub fn connect_tor(&self) -> AppResult<ConnectResult> {
        self.connect_with_args("--tor", "Tor Server", &[])
    }

    pub fn connect_securecore(&self) -> AppResult<ConnectResult> {
        self.connect_with_args("--securecore", "SecureCore Server", &[])
    }

    fn connect_with_args(
        &self,
        flag: &str,
        fallback_name: &str,
        args: &[&str],
    ) -> AppResult<ConnectResult> {
        let mut cmd_args = vec!["connect"];
        if !flag.is_empty() {
            cmd_args.push(flag);
        }
        cmd_args.extend(args.iter().copied());
        let output = self
            .run_command_with_timeout(&cmd_args, Duration::from_secs(30))
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let result = parse_connect_output(&stdout);

        let final_server = if !result.server_id.is_empty() {
            result.server_id.clone()
        } else {
            fallback_name.to_string()
        };
        self.with_cache(|c| {
            c.set_connected(final_server.clone(), result.ip.clone(), result.via.clone())
        })?;
        self.save_cache()?;

        Ok(ConnectResult {
            server_id: final_server,
            ip: result.ip,
            city: result.city,
            country: result.country,
            via: result.via,
        })
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
        let output = self
            .run_command_with_timeout(&["disconnect"], Duration::from_secs(15))
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

    pub fn is_connected(&self) -> bool {
        let proton0_exists = std::path::Path::new("/sys/class/net/proton0").exists();
        if !proton0_exists {
            tracing::debug!("Connected (proton0): false");
            return false;
        }

        let persistence_path = self.persistence_file_path();
        let persistence_exists = persistence_path.exists();
        tracing::debug!(
            "Connected check: proton0={}, persistence={}",
            proton0_exists,
            persistence_exists
        );

        persistence_exists
    }

    /// Get connected server name and IP from connection_persistence.json
    pub fn get_connected_server_info(&self) -> Option<(String, String)> {
        let persistence_path = self.persistence_file_path();
        let content = std::fs::read_to_string(persistence_path).ok()?;

        #[derive(serde::Deserialize)]
        struct Persistence {
            server: ServerInfo,
        }

        #[derive(serde::Deserialize)]
        struct ServerInfo {
            server_name: String,
            #[serde(rename = "server_ip")]
            server_ip: String,
        }

        let persistence: Persistence = serde_json::from_str(&content).ok()?;
        Some((persistence.server.server_name, persistence.server.server_ip))
    }

    pub fn get_connection_protocol(&self) -> Option<String> {
        let persistence_path = self.persistence_file_path();
        let content = std::fs::read_to_string(persistence_path).ok()?;

        #[derive(serde::Deserialize)]
        struct Persistence {
            protocol: Option<String>,
        }

        let persistence: Persistence = serde_json::from_str(&content).ok()?;
        persistence.protocol
    }

    pub fn get_connected_at(&self) -> Option<chrono::DateTime<Utc>> {
        self.with_cache(|c| c.connected_at).ok().flatten()
    }

    /// Get connection statistics (bytes_received, bytes_sent) from sysfs
    ///
    /// Returns None if proton0 interface doesn't exist or sysfs is unreadable.
    pub fn get_connection_stats(&self) -> Option<(u64, u64)> {
        let base = std::path::Path::new("/sys/class/net/proton0/statistics");
        if !base.exists() {
            return None;
        }
        let rx = std::fs::read_to_string(base.join("rx_bytes")).ok()?;
        let tx = std::fs::read_to_string(base.join("tx_bytes")).ok()?;
        let bytes_received = rx.trim().parse().ok()?;
        let bytes_sent = tx.trim().parse().ok()?;
        Some((bytes_received, bytes_sent))
    }

    pub fn refresh_countries(&self) -> AppResult<HashMap<String, String>> {
        let output = self
            .run_command_with_timeout(&["countries", "list"], Duration::from_secs(60))
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("protonvpn countries list failed: {}", stderr);
            return Err(AppError::CommandFailed(format!(
                "protonvpn countries list command failed: {}",
                stderr
            )));
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

    pub fn cached_cities(&self, country_code: &str) -> Option<Vec<City>> {
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

        let output = self
            .run_command_with_timeout(&["cities", "list", country_code], Duration::from_secs(20))
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("protonvpn cities list {} failed: {}", country_code, stderr);
            return Err(AppError::CommandFailed(format!(
                "protonvpn cities list command failed: {}",
                stderr.trim()
            )));
        }

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

    pub fn refresh_servers(&self) -> AppResult<Vec<Server>> {
        let countries = self.refresh_countries()?;
        let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();
        Ok(countries_to_servers(&countries, &cities))
    }

    /// Set a configuration option via `protonvpn config set <setting> <value>`
    pub fn set_config(&self, setting: &str, value: &str) -> AppResult<String> {
        let output = self
            .run_command_with_timeout(&["config", "set", setting, value], Duration::from_secs(20))
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok(format!("{} {}", stdout, stderr).trim().to_string())
    }

    /// Toggle a boolean setting (off <-> on)
    fn toggle_bool_setting(&self, setting: &str, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.set_config(setting, new_value)
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
        self.toggle_bool_setting("ipv6", current)
    }

    /// Toggle moderate NAT (off <-> on)
    pub fn toggle_moderate_nat(&self, current: Option<bool>) -> AppResult<String> {
        self.toggle_bool_setting("moderate-nat", current)
    }

    /// Toggle VPN accelerator (off <-> on)
    pub fn toggle_vpn_accelerator(&self, current: Option<bool>) -> AppResult<String> {
        self.toggle_bool_setting("vpn-accelerator", current)
    }

    /// Toggle port forwarding (off <-> on)
    pub fn toggle_port_forwarding(&self, current: Option<bool>) -> AppResult<String> {
        self.toggle_bool_setting("port-forwarding", current)
    }

    /// Toggle anonymous crash reports (off <-> on)
    pub fn toggle_anonymous_crash_reports(&self, current: Option<bool>) -> AppResult<String> {
        self.toggle_bool_setting("anonymous-crash-reports", current)
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
        let output = self
            .run_command_with_timeout(
                &["config", "set", "custom-dns", "on", "--dns", dns_list],
                Duration::from_secs(20),
            )
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok(format!("DNS set to {}", dns_list))
    }

    /// Disable custom DNS
    pub fn disable_custom_dns(&self) -> AppResult<String> {
        let output = self
            .run_command_with_timeout(
                &["config", "set", "custom-dns", "off"],
                Duration::from_secs(20),
            )
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok("Custom DNS disabled".to_string())
    }

    fn run_command_with_timeout(&self, args: &[&str], timeout: Duration) -> AppResult<Output> {
        use std::sync::mpsc;
        use std::thread;

        let cli_path = self.cli_path.clone();
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let args_display = args.join(" ");
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = Command::new(&cli_path)
                .args(&args_owned)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .and_then(|child| child.wait_with_output());

            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(AppError::CommandFailed(e.to_string())),
            Err(_) => Err(AppError::Timeout(format!(
                "Command '{}' timed out after {:?}",
                args_display, timeout
            ))),
        }
    }

    #[cfg(test)]
    pub fn with_test_servers(servers: Vec<Server>) -> Self {
        use std::collections::HashMap;

        let mut countries: HashMap<String, String> = HashMap::new();
        let mut cities_map: HashMap<String, Vec<City>> = HashMap::new();

        for server in &servers {
            countries.insert(server.code.clone(), server.country.clone());
            if !server.cities.is_empty() {
                cities_map.insert(server.code.clone(), server.cities.clone());
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
