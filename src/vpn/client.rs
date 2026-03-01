//! VPN client - wraps protonvpn CLI
//!
//! Uses the new `protonvpn` CLI commands:
//! - protonvpn countries     -> list countries
//! - protonvpn cities <CC>  -> list cities for a country
//! - protonvpn connect       -> connect (interactive)
//! - protonvpn connect <CC>  -> connect to a country
//! - protonvpn disconnect    -> disconnect

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;

use super::cache::ServerCache;
use super::types::{ConnectionStats, Server, ServerFeatures};
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
        self.cache.save(self.cache_path.clone())
    }

    /// Connect to a server by country code or city
    pub fn connect(&mut self, target: &str) -> AppResult<()> {
        // Use new CLI syntax: protonvpn connect <country_code>
        let output = Command::new(&self.cli_path)
            .args(["connect", target])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if already connected
            if stdout.contains("already connected") || stderr.contains("already connected") {
                return Ok(());
            }
            return Err(AppError::ConnectionFailed(format!(
                "Connection failed: {} {}",
                stderr.trim(),
                stdout.trim()
            )));
        }

        // Update local connection status with IP
        let ip = self.get_vpn_ip();
        self.cache.set_connected(target.to_string(), ip);
        self.save_cache()?;

        Ok(())
    }

    /// Disconnect from VPN
    pub fn disconnect(&mut self) -> AppResult<()> {
        let output = Command::new(&self.cli_path)
            .args(["disconnect"])
            .output()
            .map_err(|e| {
                AppError::ConfigError(format!("Failed to execute {}: {}", self.cli_path, e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check if already disconnected
            if stderr.contains("No active connection") {
                return Ok(());
            }
            return Err(AppError::ConnectionFailed(format!(
                "Disconnect failed: {}",
                stderr.trim()
            )));
        }

        // Update local connection status
        self.cache.set_disconnected();
        self.save_cache()?;

        Ok(())
    }

    /// Check if connected using system-level check (proton0 interface)
    pub fn is_connected(&self) -> bool {
        // Check proton0 interface for active connection
        match Command::new("ip").args(["addr", "show", "proton0"]).output() {
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

    /// Get VPN IP address from proton0 interface
    pub fn get_vpn_ip(&self) -> Option<String> {
        match Command::new("ip").args(["addr", "show", "proton0"]).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse "inet 10.2.0.2" from output
                for line in stdout.lines() {
                    if let Some(ip) = line.trim().strip_prefix("inet ") {
                        return Some(ip.split('/').next()?.to_string());
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    /// Check if IP matches cached connection
    pub fn matches_ip(&self, ip: &str) -> bool {
        self.cache.matches_ip(ip)
    }

    /// List available countries
    pub fn list_countries(&mut self) -> AppResult<HashMap<String, String>> {
        // Return cached if valid
        if !self.cache.is_stale() && !self.cache.countries.is_empty() {
            return Ok(self.cache.countries.clone());
        }

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
    fn parse_countries(&self, output: &str) -> HashMap<String, String> {
        let mut countries = HashMap::new();

        // Output format: "JP - Japan\nUS - United States\n..."
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse "XX - Country Name" format
            if let Some((code, name)) = line.split_once(" - ") {
                let code = code.trim().to_string();
                let name = name.trim().to_string();
                if !code.is_empty() && !name.is_empty() {
                    countries.insert(code, name);
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
    fn parse_cities(&self, output: &str) -> Vec<String> {
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

    /// List available servers (generates from countries/cities)
    pub fn list_servers(&mut self) -> AppResult<Vec<Server>> {
        // Return cached countries
        let countries = self.list_countries()?;

        let mut servers = Vec::new();

        // For each country, get cities and create server entries
        for (code, name) in &countries {
            if let Ok(cities) = self.list_cities(code) {
                for (i, city) in cities.iter().enumerate() {
                    servers.push(Server {
                        id: format!("{}-{}", code.to_lowercase(), i + 1),
                        name: format!("{} #{}", city, i + 1),
                        country: name.clone(),
                        city: city.clone(),
                        load: 50, // Default load - real load not available
                        ping: None,
                        features: ServerFeatures::default(),
                    });
                }
            }
        }

        // If no real data, use mock servers
        if servers.is_empty() {
            servers = self.mock_servers();
        }

        Ok(servers)
    }

    /// Mock servers for testing
    fn mock_servers(&self) -> Vec<Server> {
        vec![
            Server {
                id: "jp-1".to_string(),
                name: "Tokyo #1".to_string(),
                country: "Japan".to_string(),
                city: "Tokyo".to_string(),
                load: 45,
                ping: Some(120),
                features: ServerFeatures::default(),
            },
            Server {
                id: "us-1".to_string(),
                name: "New York #1".to_string(),
                country: "United States".to_string(),
                city: "New York".to_string(),
                load: 72,
                ping: Some(180),
                features: ServerFeatures::default(),
            },
            Server {
                id: "ch-1".to_string(),
                name: "Zurich #1".to_string(),
                country: "Switzerland".to_string(),
                city: "Zurich".to_string(),
                load: 28,
                ping: Some(90),
                features: ServerFeatures::default(),
            },
            Server {
                id: "de-1".to_string(),
                name: "Frankfurt #1".to_string(),
                country: "Germany".to_string(),
                city: "Frankfurt".to_string(),
                load: 55,
                ping: Some(95),
                features: ServerFeatures::default(),
            },
            Server {
                id: "nl-1".to_string(),
                name: "Amsterdam #1".to_string(),
                country: "Netherlands".to_string(),
                city: "Amsterdam".to_string(),
                load: 60,
                ping: Some(100),
                features: ServerFeatures::default(),
            },
        ]
    }

    /// Get connection status (from local cache)
    pub fn status(&self) -> AppResult<String> {
        if let Some(server) = &self.cache.connected_server {
            Ok(format!("Connected to: {}", server))
        } else {
            Ok("Disconnected".to_string())
        }
    }

    /// Get connection statistics
    pub fn stats(&self) -> AppResult<ConnectionStats> {
        // Connection stats from local cache
        Ok(ConnectionStats {
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: self.cache.connected_at.unwrap_or(Utc::now()),
            server_ip: String::new(),
            protocol: String::from("WireGuard"),
        })
    }
}
