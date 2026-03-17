//! VPN server cache - stores countries and cities locally
//!
//! Since protonvpn CLI doesn't have a server list command,
//! we cache the results of `protonvpn countries` and `protonvpn cities`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::types::{City, Server};

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
    pub connected_ip: Option<String>,
    pub connected_via: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
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
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let path_for_log = path.clone();
        std::fs::write(path, content)?;
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

    /// Mark as connected
    pub fn set_connected(&mut self, server: String, ip: Option<String>, via: Option<String>) {
        self.connected_server = Some(server);
        self.connected_ip = ip;
        self.connected_via = via;
        self.connected_at = Some(Utc::now());
    }

    /// Mark as disconnected
    pub fn set_disconnected(&mut self) {
        self.connected_server = None;
        self.connected_ip = None;
        self.connected_via = None;
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

    /// Check if CLI was unavailable during last refresh
    pub fn is_cli_unavailable(&self) -> bool {
        self.cli_unavailable
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
