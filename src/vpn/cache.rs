//! VPN server cache - stores countries and cities locally
//!
//! Since protonvpn CLI doesn't have a server list command,
//! we cache the results of `protonvpn countries` and `protonvpn cities`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::types::{City, Server};

/// Get system boot time from /proc/stat
///
/// Returns None if /proc/stat is unavailable or malformed (non-Linux systems)
pub fn system_boot_time() -> Option<DateTime<Utc>> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    for line in content.lines() {
        if let Some(btime_str) = line.strip_prefix("btime ") {
            let secs: i64 = btime_str.trim().parse().ok()?;
            return DateTime::from_timestamp(secs, 0);
        }
    }
    None
}

/// Cached server data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCache {
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
    /// Country code -> Country name
    pub countries: HashMap<String, String>,
    /// Country code -> Cities with features
    pub cities: HashMap<String, Vec<City>>,
    /// Connection status (tracked locally since no `protonvpn status` exists)
    pub connected_server: Option<String>,
    #[serde(rename = "server_id")]
    pub connected_server_id: Option<String>,
    pub connected_ip: Option<String>,
    pub connected_via: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub forwarded_port: Option<u16>,
    /// Flag indicating if CLI was unavailable during last refresh
    #[serde(default)]
    pub cli_unavailable: bool,
}

impl ServerCache {
    /// Load cache from file
    pub fn load(path: PathBuf) -> crate::error::AppResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let cache: ServerCache = toml::from_str(&content)?;
        Ok(cache)
    }

    /// Save cache to file
    pub fn save(&self, path: PathBuf) -> crate::error::AppResult<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let path_for_log = path.clone();
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content)?;
        if let Err(e) = std::fs::rename(&temp_path, &path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e.into());
        }
        tracing::debug!("Server cache saved to {:?}", path_for_log);
        Ok(())
    }

    /// Check if cache is stale (older than 24 hours)
    pub fn is_stale(&self) -> bool {
        match self.last_updated {
            None => true,
            Some(last) => {
                let duration = Utc::now().signed_duration_since(last);
                duration.num_hours() > 24
            }
        }
    }

    pub fn set_connected(
        &mut self,
        server: String,
        server_id: Option<String>,
        ip: Option<String>,
        via: Option<String>,
    ) {
        self.connected_server = Some(server);
        self.connected_server_id = server_id;
        self.connected_ip = ip;
        self.connected_via = via;
        self.connected_at = Some(Utc::now());
    }

    pub fn set_forwarded_port(&mut self, port: Option<u16>) {
        self.forwarded_port = port;
    }

    pub fn set_disconnected(&mut self) {
        self.connected_server = None;
        self.connected_server_id = None;
        self.connected_ip = None;
        self.connected_via = None;
        self.connected_at = None;
        self.forwarded_port = None;
    }

    /// Check if IP matches cached connection
    pub fn matches_ip(&self, ip: &str) -> bool {
        match &self.connected_ip {
            Some(cached_ip) => cached_ip == ip,
            None => false,
        }
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.connected_server.is_some()
    }

    /// Check if CLI was unavailable during last refresh
    pub fn is_cli_unavailable(&self) -> bool {
        self.cli_unavailable
    }

    /// Validate connected_at against system boot time
    ///
    /// If connected_at is before system boot time, adjust it to boot time.
    /// This handles the case where the cache survived a reboot.
    pub fn validate_after_boot(&mut self) {
        if let Some(connected_at) = self.connected_at {
            if let Some(boot_time) = system_boot_time() {
                if connected_at < boot_time {
                    tracing::debug!(
                        "Adjusting connected_at from {} to boot time {}",
                        connected_at,
                        boot_time
                    );
                    self.connected_at = Some(boot_time);
                }
            }
        }
    }

    /// Adjust connected_at based on uptime duration
    ///
    /// Sets connected_at to (now - uptime).
    pub fn adjust_connected_at_from_uptime(&mut self, uptime: chrono::Duration) {
        let new_connected_at = Utc::now() - uptime;
        tracing::debug!(
            "Adjusting connected_at to {} based on uptime {}",
            new_connected_at,
            uptime
        );
        self.connected_at = Some(new_connected_at);
    }
}

pub fn countries_to_servers(
    countries: &HashMap<String, String>,
    cities: &HashMap<String, Vec<City>>,
) -> Vec<Server> {
    countries
        .iter()
        .map(|(id, country)| Server {
            code: id.clone(),
            code_lower: id.to_lowercase(),
            country: country.clone(),
            country_lower: country.to_lowercase(),
            cities: cities.get(id).cloned().unwrap_or_default(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_system_boot_time_returns_some_on_linux() {
        // This test only runs on Linux where /proc/stat exists
        if cfg!(target_os = "linux") {
            let boot_time = system_boot_time();
            assert!(boot_time.is_some(), "boot_time should be Some on Linux");
        }
    }

    #[test]
    fn test_validate_after_boot_adjusts_old_timestamp() {
        let mut cache = ServerCache::default();
        // Set connected_at to a very old time (before any boot)
        cache.connected_at = DateTime::from_timestamp(0, 0); // Unix epoch

        cache.validate_after_boot();

        // After validation, connected_at should be adjusted (if boot_time is available)
        if cfg!(target_os = "linux") {
            assert!(
                cache.connected_at.is_some(),
                "connected_at should still be Some"
            );
            // It should be adjusted to boot_time (which is after epoch)
            assert!(
                cache.connected_at.unwrap().timestamp() > 0,
                "connected_at should be adjusted to boot_time"
            );
        }
    }

    #[test]
    fn test_validate_after_boot_no_adjustment_for_recent_timestamp() {
        let mut cache = ServerCache::default();
        // Set connected_at to a recent time (after boot)
        cache.connected_at = Some(Utc::now());

        let original = cache.connected_at;
        cache.validate_after_boot();

        // Should not adjust since it's already after boot
        assert_eq!(cache.connected_at, original);
    }

    #[test]
    fn test_validate_after_boot_none_connected_at() {
        let mut cache = ServerCache::default();
        cache.connected_at = None;

        cache.validate_after_boot();

        assert!(cache.connected_at.is_none());
    }

    #[test]
    fn test_adjust_connected_at_from_uptime() {
        let mut cache = ServerCache::default();
        let uptime = Duration::minutes(30);

        cache.adjust_connected_at_from_uptime(uptime.clone());

        assert!(cache.connected_at.is_some());
        let connected_at = cache.connected_at.unwrap();
        let expected = Utc::now() - uptime;
        // Allow 1 second tolerance for test execution time
        let diff = (expected - connected_at).num_seconds().abs();
        assert!(
            diff <= 1,
            "connected_at should be approximately now - uptime"
        );
    }
}
