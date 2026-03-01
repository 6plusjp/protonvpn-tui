//! VPN state machine
//!

use super::client::VpnClient;
use super::types::Server;
use crate::error::AppResult;
use crate::state::ConnectionState;

/// VPN state manager
#[derive(Debug, Clone)]
pub struct VpnState {
    client: VpnClient,
    state: ConnectionState,
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
            state: ConnectionState::Disconnected,
        }
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Check if currently connected (from local cache)
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Get connected server (from local cache)
    pub fn get_connected_server(&self) -> Option<String> {
        self.client.get_connected_server()
    }

    /// Get VPN IP from proton0 interface
    pub fn get_vpn_ip(&self) -> Option<String> {
        self.client.get_vpn_ip()
    }

    /// Check if IP matches cached connection
    pub fn matches_ip(&self, ip: &str) -> bool {
        self.client.matches_ip(ip)
    }

    pub fn connect(&mut self, server: &str) -> AppResult<()> {
        if !self.state.can_connect() {
            return Err(crate::error::VpnError::AlreadyConnected.into());
        }

        self.state = ConnectionState::Connecting;
        self.client.connect(server)?;
        self.state = ConnectionState::Connected {
            server: server.to_string(),
            ip: String::new(), // TODO: fetch actual IP
        };
        Ok(())
    }

    pub fn disconnect(&mut self) -> AppResult<()> {
        if !self.state.can_disconnect() {
            return Err(crate::error::VpnError::NotConnected.into());
        }

        self.state = ConnectionState::Disconnecting;
        self.client.disconnect()?;
        self.state = ConnectionState::Disconnected;
        Ok(())
    }

    pub fn list_servers(&mut self) -> AppResult<Vec<Server>> {
        self.client.list_servers()
    }
}
