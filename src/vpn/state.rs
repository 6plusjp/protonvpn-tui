use super::client::VpnClient;
use super::types::Server;
use crate::error::AppResult;

/// VPN state manager
#[derive(Debug, Clone)]
pub struct VpnState {
    client: VpnClient,
}

impl Default for VpnState {
    fn default() -> Self {
        Self::new()
    }
}

impl VpnState {
    pub fn new() -> Self {
        Self {
            client: VpnClient::new(),
        }
    }

    /// Check if currently connected (from local cache)
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Get connected server (from local cache)
    pub fn get_connected_server(&self) -> Option<String> {
        self.client.get_connected_server()
    }

    /// Get VPN IP from cache (not proton0)
    pub fn get_vpn_ip(&self) -> Option<String> {
        self.client.get_vpn_ip()
    }

    /// Check if IP matches cached connection
    pub fn matches_ip(&self, ip: &str) -> bool {
        self.client.matches_ip(ip)
    }

    pub fn connect(&mut self, server: &str) -> AppResult<(String, Option<String>)> {
        let (server_id, ip) = self.client.connect(server)?;
        Ok((server_id, ip))
    }

    pub fn connect_random(&mut self) -> AppResult<(String, Option<String>)> {
        let (server_id, ip) = self.client.connect_random()?;
        Ok((server_id, ip))
    }

    pub fn disconnect(&mut self) -> AppResult<()> {
        self.client.disconnect()?;
        Ok(())
    }

    pub fn list_servers(&mut self) -> AppResult<Vec<Server>> {
        self.client.list_servers()
    }

    /// Get cached servers, refresh from CLI if empty
    pub fn get_servers(&mut self) -> AppResult<Vec<Server>> {
        let cached = self.client.get_servers();
        if cached.is_empty() {
            return self.refresh_servers();
        }
        Ok(cached)
    }

    /// Refresh servers from CLI
    pub fn refresh_servers(&mut self) -> AppResult<Vec<Server>> {
        self.client.refresh_servers()
    }
}
