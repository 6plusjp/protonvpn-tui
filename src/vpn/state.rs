//! VPN state machine

use super::client::VpnClient;
use crate::error::AppResult;
use crate::state::ConnectionState;

/// VPN state manager
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

    pub fn list_servers(&self) -> AppResult<Vec<super::types::Server>> {
        self.client.list_servers()
    }
}
