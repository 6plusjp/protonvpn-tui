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

    #[error("Command execution failed: {0}")]
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
