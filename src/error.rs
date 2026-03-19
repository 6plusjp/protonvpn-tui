//! ProtonVPN TUI - Error types

use anyhow::Result;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("VPN connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication error: {0}")]
    AuthFailed(String),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("{0}")]
    CommandFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

/// VPN-specific errors for reusable components
#[derive(Debug, thiserror::Error)]
pub enum VpnError {
    #[error("Not connected")]
    NotConnected,

    #[error("Invalid server: {0}")]
    InvalidServer(String),
}

// Allow converting VpnError to AppError
impl From<VpnError> for AppError {
    fn from(e: VpnError) -> Self {
        match e {
            VpnError::NotConnected => AppError::ConnectionFailed("Not connected".into()),
            VpnError::InvalidServer(s) => AppError::ServerNotFound(s),
        }
    }
}

/// Categorize error into user-friendly type
pub fn categorize_error(e: &AppError) -> &'static str {
    match e {
        AppError::Timeout(_) => "Timeout",
        AppError::ConnectionFailed(msg) => {
            let lower = msg.to_lowercase();
            if lower.contains("not logged in") || lower.contains("not loggedin") {
                "NotLoggedIn"
            } else if lower.contains("not connected") {
                "NotConnected"
            } else if lower.contains("already connected") {
                "AlreadyConnected"
            } else if lower.contains("no servers available") {
                "NoServers"
            } else {
                "ConnectionFailed"
            }
        }
        AppError::CommandFailed(msg) => {
            let lower = msg.to_lowercase();
            if lower.contains("no such file")
                || lower.contains("not found")
                || lower.contains("not exist")
            {
                "NotFound"
            } else if lower.contains("permission denied") || lower.contains("access denied") {
                "AccessDenied"
            } else if lower.contains("timed out") {
                "Timeout"
            } else {
                "CommandFailed"
            }
        }
        AppError::ServerNotFound(_) => "ServerNotFound",
        AppError::AuthFailed(_) => "AuthFailed",
        AppError::ParseError(_) => "ParseError",
        AppError::ConfigError(_) => "ConfigError",
    }
}
