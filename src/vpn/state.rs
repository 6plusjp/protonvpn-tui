use super::client::VpnClient;
use super::types::{City, Server};
use crate::error::AppResult;

/// VPN state manager
#[derive(Debug)]
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

    pub fn connect(&self, server: &str) -> AppResult<(String, Option<String>)> {
        let (server_id, ip) = self.client.connect(server)?;
        Ok((server_id, ip))
    }

    pub fn connect_random(&self) -> AppResult<(String, Option<String>)> {
        let (server_id, ip) = self.client.connect_random()?;
        Ok((server_id, ip))
    }

    pub fn connect_city(&self, city: &str) -> AppResult<(String, Option<String>)> {
        let (server_id, ip) = self.client.connect_city(city)?;
        Ok((server_id, ip))
    }

    pub fn list_cities_with_features(&self, country_code: &str) -> AppResult<Vec<City>> {
        self.client.list_cities_with_features(country_code)
    }

    pub fn disconnect(&self) -> AppResult<()> {
        self.client.disconnect()?;
        Ok(())
    }

    pub fn list_servers(&self) -> AppResult<Vec<Server>> {
        self.client.list_servers()
    }

    pub fn get_servers(&self) -> Vec<Server> {
        self.client.get_servers()
    }

    pub fn get_servers_or_refresh(&self) -> AppResult<Vec<Server>> {
        let cached = self.client.get_servers();
        if cached.is_empty() {
            return self.refresh_servers();
        }
        Ok(cached)
    }

    pub fn refresh_servers(&self) -> AppResult<Vec<Server>> {
        self.client.refresh_servers()
    }

    pub fn toggle_killswitch(&self, current: Option<i32>) -> AppResult<String> {
        self.client.toggle_killswitch(current)
    }

    pub fn toggle_ipv6(&self, current: Option<bool>) -> AppResult<String> {
        self.client.toggle_ipv6(current)
    }

    pub fn toggle_moderate_nat(&self, current: Option<bool>) -> AppResult<String> {
        self.client.toggle_moderate_nat(current)
    }

    pub fn toggle_vpn_accelerator(&self, current: Option<bool>) -> AppResult<String> {
        self.client.toggle_vpn_accelerator(current)
    }

    pub fn toggle_port_forwarding(&self, current: Option<bool>) -> AppResult<String> {
        self.client.toggle_port_forwarding(current)
    }

    pub fn set_netshield(&self, current: Option<i32>, next: i32) -> AppResult<String> {
        self.client.set_netshield(current, next)
    }
}
