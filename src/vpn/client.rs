//! VPN client - wraps protonvpn CLI
//!
//! Uses the `protonvpn` CLI commands (per PROTONVPN_CLI_REFERENCE.md v1.0.0):
//! - `protonvpn countries list` → list countries
//! - `protonvpn cities list <CC>` → list cities for a country (CC = country code)
//! - `protonvpn connect` → connect to fastest server
//! - `protonvpn connect --country <CC>` → connect to a country
//! - `protonvpn connect --city <city>` → connect to a city
//! - `protonvpn connect --random` → connect to random server
//! - `protonvpn connect --p2p` → connect to fastest P2P server
//! - `protonvpn connect --tor` → connect to fastest Tor server
//! - `protonvpn connect --securecore` → connect to fastest Secure Core server
//! - `protonvpn disconnect` → disconnect
//! - `protonvpn status` → display connection status
//! - `protonvpn config set <setting> <value>` → change settings

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
use crate::constants::settings::{FEATURE_OFF, FEATURE_ON};

use crate::constants::vpn::*;
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

        let client = Self {
            cli_path: "protonvpn".to_string(),
            cache: Mutex::new(cache),
            cache_path,
        };

        // Adjust connected_at timestamp if it survived a reboot
        client.adjust_connected_at_on_startup();

        client
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
    pub fn cached_servers(&self) -> Vec<Server> {
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
    pub fn connect_country(&self, target: &str) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("--country", target, &[target])
    }

    pub fn connect_random(&self) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("--random", "Random Server", &[])
    }

    /// Connect to a server by city name
    pub fn connect_city(&self, city_arg: &str) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("--city", city_arg, &[city_arg])
    }

    pub fn connect_fastest(&self) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("", "Fastest Server", &[])
    }

    pub fn connect_p2p(&self) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("--p2p", "P2P Server", &[])
    }

    pub fn connect_tor(&self) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("--tor", "Tor Server", &[])
    }

    pub fn connect_securecore(&self) -> AppResult<(ConnectResult, bool)> {
        self.connect_with_args("--securecore", "SecureCore Server", &[])
    }

    fn connect_with_args(
        &self,
        flag: &str,
        fallback_name: &str,
        args: &[&str],
    ) -> AppResult<(ConnectResult, bool)> {
        let mut cmd_args = vec!["connect"];
        if !flag.is_empty() {
            cmd_args.push(flag);
        }
        cmd_args.extend(args.iter().copied());
        let output = self
            .run_command_with_timeout(&cmd_args, CONNECT_TIMEOUT)
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        let result = parse_connect_output(&stdout);

        let server_id = self.get_connected_server_info().map(|(sid, _, _)| sid);

        let final_server = if !result.server.is_empty() {
            result.server.clone()
        } else {
            fallback_name.to_string()
        };

        let needs_time_update = self
            .with_cache(|c| {
                c.connected_server.as_deref() != Some(&final_server)
                    || c.connected_server_id.as_deref() != server_id.as_deref()
            })
            .unwrap_or(true);

        self.with_cache(|c| {
            if needs_time_update {
                c.set_connected(
                    final_server.clone(),
                    server_id.clone(),
                    result.ip.clone(),
                    result.via.clone(),
                )
            } else {
                c.connected_server = Some(final_server.clone());
                c.connected_server_id = server_id.clone();
                c.connected_ip = result.ip.clone();
                c.connected_via = result.via.clone();
            }
        })?;
        self.save_cache()?;

        let needs_refresh = self.check_server_list_outdated(&stdout, &stderr);

        Ok((
            ConnectResult {
                server: final_server,
                server_id: None,
                ip: result.ip,
                city: result.city,
                country: result.country,
                via: result.via,
            },
            needs_refresh,
        ))
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

    fn check_server_list_outdated(&self, stdout: &str, stderr: &str) -> bool {
        let combined = format!("{} {}", stdout, stderr).to_lowercase();
        combined.contains("server list is outdated")
    }

    /// Disconnect from VPN
    pub fn disconnect(&self) -> AppResult<()> {
        let output = self
            .run_command_with_timeout(&["disconnect"], DISCONNECT_TIMEOUT)
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

    pub fn get_connected_server_info(&self) -> Option<(String, String, String)> {
        let persistence_path = self.persistence_file_path();
        let content = std::fs::read_to_string(persistence_path).ok()?;

        #[derive(serde::Deserialize)]
        struct Persistence {
            server: ServerInfo,
        }

        #[derive(serde::Deserialize)]
        struct ServerInfo {
            #[serde(rename = "server_id")]
            server_id: String,
            #[serde(rename = "server_name")]
            server_name: String,
            #[serde(rename = "server_ip")]
            server_ip: String,
        }

        let persistence: Persistence = serde_json::from_str(&content).ok()?;
        Some((
            persistence.server.server_id,
            persistence.server.server_name,
            persistence.server.server_ip,
        ))
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

    pub fn get_connected_server(&self) -> Option<String> {
        self.with_cache(|c| c.connected_server.clone())
            .ok()
            .flatten()
    }

    pub fn get_connected_server_id(&self) -> Option<String> {
        self.with_cache(|c| c.connected_server_id.clone())
            .ok()
            .flatten()
    }

    pub fn get_connected_ip(&self) -> Option<String> {
        self.with_cache(|c| c.connected_ip.clone()).ok().flatten()
    }

    pub fn clear_connected_at(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.connected_at = None;
            if let Err(e) = self.save_cache() {
                tracing::warn!("Failed to save cache after clearing connected_at: {}", e);
            }
        }
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

    /// Get uptime from protonvpn status command
    ///
    /// Returns None if status command fails or uptime is not available
    pub fn get_status_uptime(&self) -> Option<chrono::Duration> {
        let output = self
            .run_command_with_timeout(&["status"], std::time::Duration::from_secs(5))
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        super::types::parse_status_uptime(&stdout)
    }

    /// Get full status information from protonvpn status command
    ///
    /// Returns None if status command fails or VPN is not connected.
    /// This is used to restore location information (city/country) on startup.
    pub fn get_status_info(&self) -> Option<super::types::StatusInfo> {
        let output = self
            .run_command_with_timeout(&["status"], std::time::Duration::from_secs(5))
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        super::types::parse_status_output(&stdout)
    }

    pub fn sync_cache_with_connection(&self, server: &str, server_id: &str, ip: &str) {
        tracing::debug!(
            "sync_cache_with_connection called: server={}, server_id={}, ip={}",
            server,
            server_id,
            ip
        );
        if let Ok(mut cache) = self.cache.lock() {
            let server_changed = cache.connected_server.as_deref() != Some(server);
            let server_id_changed = cache.connected_server_id.as_deref() != Some(server_id);
            let ip_changed = cache.connected_ip.as_deref() != Some(ip);

            tracing::debug!(
                "sync_cache_with_connection: server_changed={}, server_id_changed={}, ip_changed={}",
                server_changed,
                server_id_changed,
                ip_changed
            );

            if server_changed || server_id_changed || ip_changed {
                tracing::debug!(
                    "Syncing cache: old={:?}/{:?}/{:?}, new={}/{:?}/{:?}",
                    cache.connected_server,
                    cache.connected_server_id,
                    cache.connected_ip,
                    server,
                    server_id,
                    ip
                );
                cache.connected_server = Some(server.to_string());
                cache.connected_server_id = Some(server_id.to_string());
                cache.connected_ip = Some(ip.to_string());
                cache.connected_via = None;

                if let Some(boot_time) = super::cache::system_boot_time() {
                    if cache.connected_at.map_or(true, |t| t < boot_time) {
                        cache.connected_at = Some(boot_time);
                    }
                }

                tracing::debug!("sync_cache_with_connection: cache updated (skipping disk save)");
            } else {
                tracing::debug!("sync_cache_with_connection: no changes needed");
            }
        }
    }
    pub fn update_connected_at(&self, server: &str, ip: &str) {
        tracing::debug!("update_connected_at called: server={}, ip={}", server, ip);
        if let Ok(mut cache) = self.cache.lock() {
            let server_changed = cache.connected_server.as_deref() != Some(server);
            let ip_changed = cache.connected_ip.as_deref() != Some(ip);

            tracing::debug!(
                "update_connected_at: server_changed={}, ip_changed={}, cached_server={:?}, cached_ip={:?}, new_server={}, new_ip={}",
                server_changed,
                ip_changed,
                cache.connected_server,
                cache.connected_ip,
                server,
                ip
            );

            if server_changed || ip_changed {
                cache.connected_server = Some(server.to_string());
                cache.connected_ip = Some(ip.to_string());
                cache.connected_at = Some(chrono::Utc::now());
                tracing::debug!("update_connected_at: server/ip changed, reset to current time");
                drop(cache);
                if let Err(e) = self.save_cache() {
                    tracing::warn!("Failed to save cache in update_connected_at: {}", e);
                }
            } else {
                tracing::debug!("update_connected_at: same server/ip, no update needed");
            }
        }
    }

    /// Adjust connected_at based on uptime duration
    ///
    /// Sets connected_at to (now - uptime). Used when restoring connection state
    /// from protonvpn status output which includes uptime information.
    pub fn adjust_connected_at_from_uptime(&self, uptime: chrono::Duration) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.adjust_connected_at_from_uptime(uptime);
            if let Err(e) = self.save_cache() {
                tracing::warn!("Failed to save cache after uptime adjustment: {}", e);
            }
        }
    }

    fn adjust_connected_at_on_startup(&self) {
        let is_connected = self.with_cache(|c| c.is_connected()).unwrap_or(false);
        if !is_connected {
            return;
        }

        tracing::debug!("adjust_connected_at_on_startup: is_connected=true");
        if let Err(e) = self.with_cache(|c| c.validate_after_boot()) {
            tracing::warn!("Failed to validate connected_at after boot: {}", e);
        }
        // Save after validation
        if let Err(e) = self.save_cache() {
            tracing::warn!("Failed to save cache after boot validation: {}", e);
        }
    }

    pub fn refresh_countries(&self) -> AppResult<HashMap<String, String>> {
        let output = self
            .run_command_with_timeout(&["countries", "list"], COUNTRIES_LIST_TIMEOUT)
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

    pub fn list_cities_with_features(&self, country_code: &str) -> AppResult<(Vec<City>, bool)> {
        let is_stale = self.with_cache(|c| c.is_stale())?;
        let has_cached = self
            .with_cache(|c| c.cities.contains_key(country_code))
            .is_ok_and(|r| r);

        if !is_stale && has_cached {
            if let Ok(Some(cities)) = self.with_cache(|c| c.cities.get(country_code).cloned()) {
                return Ok((cities, false));
            }
        }

        let output = self
            .run_command_with_timeout(&["cities", "list", country_code], CITIES_LIST_TIMEOUT)
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

        let needs_refresh = self.check_server_list_outdated(&stdout, "");

        Ok((cities, needs_refresh))
    }

    pub fn clear_cities_cache(&self, country_code: &str) -> AppResult<()> {
        self.with_cache(|c| {
            c.cities.remove(country_code);
        })?;
        self.save_cache()?;
        Ok(())
    }

    /// Load servers on startup - use cache if available, fallback to CLI if empty
    pub fn servers_from_cache(&self) -> AppResult<Vec<Server>> {
        let countries = self.with_cache(|c| c.countries.clone()).unwrap_or_default();
        let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();

        if countries.is_empty() {
            // Cache is empty - execute CLI to get fresh data
            return self.refresh_servers();
        }

        Ok(countries_to_servers(&countries, &cities))
    }

    pub fn refresh_servers(&self) -> AppResult<Vec<Server>> {
        let countries = self.refresh_countries()?;
        let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();
        Ok(countries_to_servers(&countries, &cities))
    }

    /// Set a configuration option via `protonvpn config set <setting> <value>`
    pub fn set_config(&self, setting: &str, value: &str) -> AppResult<String> {
        let output = self
            .run_command_with_timeout(&["config", "set", setting, value], CONFIG_SET_TIMEOUT)
            .map_err(|e| AppError::CommandFailed(categorize_error(&e).to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.check_cli_error(&output, &stdout, &stderr)?;

        Ok(format!("{} {}", stdout, stderr).trim().to_string())
    }

    /// Toggle a boolean setting (off <-> on)
    fn toggle_bool_setting(&self, setting: &str, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) {
            FEATURE_OFF
        } else {
            FEATURE_ON
        };
        self.set_config(setting, new_value)
    }

    /// Toggle setting by key (for boolean settings)
    pub fn toggle_setting(
        &self,
        key: crate::config::SettingKey,
        current: Option<bool>,
    ) -> AppResult<String> {
        self.toggle_bool_setting(key.config_key(), current)
    }

    /// Toggle kill switch (off <-> standard)
    pub fn toggle_killswitch(&self, current: Option<i32>) -> AppResult<String> {
        let new_value = if current == Some(1) {
            FEATURE_OFF
        } else {
            "standard"
        };
        self.set_config("kill-switch", new_value)
    }

    /// Set NetShield mode (off -> malware-only -> malware-ads-trackers -> off)
    pub fn set_netshield(&self, current: Option<i32>, _next: i32) -> AppResult<String> {
        let new_value = match current.unwrap_or(0) {
            0 => "malware-only",
            1 => "malware-ads-trackers",
            _ => FEATURE_OFF,
        };
        self.set_config("netshield", new_value)
    }

    /// Set custom DNS servers
    pub fn set_custom_dns(&self, dns_list: &str) -> AppResult<String> {
        let output = self
            .run_command_with_timeout(
                &["config", "set", "custom-dns", FEATURE_ON, "--dns", dns_list],
                CONFIG_SET_TIMEOUT,
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
                &["config", "set", "custom-dns", FEATURE_OFF],
                CONFIG_SET_TIMEOUT,
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
        if let Ok(mut cache) = client.cache.lock() {
            cache.countries = countries;
            cache.cities = cities_map;
        }

        client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_servers() -> Vec<Server> {
        vec![
            Server {
                code: "JP".into(),
                code_lower: "jp".into(),
                country: "Japan".into(),
                country_lower: "japan".into(),
                cities: vec![City::new("Tokyo".into())],
            },
            Server {
                code: "US".into(),
                code_lower: "us".into(),
                country: "United States".into(),
                country_lower: "united states".into(),
                cities: vec![
                    City::new("New York".into()),
                    City::new("Los Angeles".into()),
                ],
            },
        ]
    }

    #[test]
    fn test_vpn_client_new() {
        let client = VpnClient::new();
        assert_eq!(client.cli_path, "protonvpn");
    }

    #[test]
    fn test_vpn_client_with_path() {
        let client = VpnClient::with_path("/custom/path/protonvpn");
        assert_eq!(client.cli_path, "/custom/path/protonvpn");
    }

    #[test]
    fn test_vpn_client_with_test_servers() {
        let servers = make_test_servers();
        let client = VpnClient::with_test_servers(servers.clone());

        let cached = client.cached_servers();
        assert_eq!(cached.len(), 2);

        let jp = cached.iter().find(|s| s.code == "JP").unwrap();
        assert_eq!(jp.country, "Japan");
        assert_eq!(jp.cities.len(), 1);
    }

    #[test]
    fn test_servers_empty_when_no_cache() {
        let client = VpnClient::with_test_servers(vec![]);
        let servers = client.cached_servers();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_matches_ip_false_when_not_connected() {
        let servers = make_test_servers();
        let client = VpnClient::with_test_servers(servers);

        assert!(!client.matches_ip("1.2.3.4"));
    }

    #[test]
    fn test_cached_cities_returns_none_when_not_cached() {
        let servers = make_test_servers();
        let client = VpnClient::with_test_servers(servers);

        let cities = client.cached_cities("DE");
        assert!(cities.is_none());
    }

    #[test]
    fn test_cached_cities_returns_cities_when_cached() {
        let servers = make_test_servers();
        let client = VpnClient::with_test_servers(servers);

        let cities = client.cached_cities("JP");
        assert!(cities.is_some());
        let cities = cities.unwrap();
        assert_eq!(cities.len(), 1);
        assert_eq!(cities[0].name, "Tokyo");
    }
}
