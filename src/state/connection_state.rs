//! Connection state management

use serde::{Deserialize, Serialize};

/// Represents the VPN connection state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected {
        server: String,
        ip: String,
        city: Option<String>,
        country: Option<String>,
    },
    Disconnecting,
    Error(String),
}

impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected { .. })
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionState::Connecting)
    }

    pub fn can_connect(&self) -> bool {
        matches!(
            self,
            ConnectionState::Disconnected | ConnectionState::Error(_)
        )
    }

    pub fn is_disconnecting(&self) -> bool {
        matches!(self, ConnectionState::Disconnecting)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(self, ConnectionState::Disconnected)
    }
}
