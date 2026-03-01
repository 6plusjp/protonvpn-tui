//! Application settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralSettings {
    /// Log level (debug, info, warn, error)
    pub log_level: String,
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiSettings {
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Theme (dark, light)
    pub theme: String,
}

/// Connection settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionSettings {
    /// Auto-connect on startup
    pub auto_connect: bool,
    /// Default server
    pub default_server: Option<String>,
    /// Kill switch enabled
    pub kill_switch: bool,
    /// Secure core enabled
    pub secure_core: bool,
    /// Always on
    pub always_on: bool,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub general: GeneralSettings,

    #[serde(default)]
    pub ui: UiSettings,

    #[serde(default)]
    pub connection: ConnectionSettings,
}

impl Settings {
    /// Load settings from file
    pub fn load(path: PathBuf) -> crate::error::AppResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    }

    /// Save settings to file
    pub fn save(&self, path: PathBuf) -> crate::error::AppResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
