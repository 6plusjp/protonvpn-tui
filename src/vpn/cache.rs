//! VPN server cache - stores countries and cities locally
//!
//! Since protonvpn CLI doesn't have a server list command,
//! we cache the results of `protonvpn countries` and `protonvpn cities`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Cached server data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCache {
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
    /// Country code -> Country name
    pub countries: HashMap<String, String>,
    /// Country code -> Cities
    pub cities: HashMap<String, Vec<String>>,
    /// Connection status (tracked locally since no `protonvpn status` exists)
    pub connected_server: Option<String>,
    pub connected_ip: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
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
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
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

    /// Mark as connected
    pub fn set_connected(&mut self, server: String, ip: Option<String>) {
        self.connected_server = Some(server);
        self.connected_ip = ip;
        self.connected_at = Some(Utc::now());
    }

    /// Mark as disconnected
    pub fn set_disconnected(&mut self) {
        self.connected_server = None;
        self.connected_ip = None;
        self.connected_at = None;
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
}
