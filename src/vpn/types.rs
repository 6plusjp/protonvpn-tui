//! VPN data types

use serde::{Deserialize, Serialize};

/// Server features
///
/// Note: `protonvpn-cli` does not provide server feature information
/// (secure_core, p2p, tor, streaming). These fields will always be `false`
/// unless the CLI adds support for a server info command.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerFeatures {
    pub secure_core: bool,
    pub p2p: bool,
    pub tor: bool,
    pub streaming: bool,
}

/// VPN server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,          // Country code (e.g., "JP", "US")
    pub country: String,     // Full country name
    pub cities: Vec<String>, // City names (e.g., ["Tokyo", "Osaka"])
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub server_ip: String,
    pub protocol: String,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: chrono::Utc::now(),
            server_ip: String::new(),
            protocol: String::from("Unknown"),
        }
    }
}
