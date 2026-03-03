//! Application settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Proton VPN settings (from ~/.config/Proton/VPN/settings.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonSettings {
    pub protocol: Option<String>,
    pub killswitch: Option<i32>,
    pub ipv6: Option<bool>,
    #[serde(rename = "custom_dns")]
    pub custom_dns: ProtonCustomDns,
    pub features: Option<ProtonFeatures>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonCustomDns {
    pub enabled: bool,
    #[serde(rename = "ip_list")]
    pub ip_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonFeatures {
    pub netshield: Option<i32>,
    #[serde(rename = "moderate_nat")]
    pub moderate_nat: Option<bool>,
    #[serde(rename = "vpn_accelerator")]
    pub vpn_accelerator: Option<bool>,
    #[serde(rename = "port_forwarding")]
    pub port_forwarding: Option<bool>,
    #[serde(rename = "split_tunneling")]
    pub split_tunneling: Option<ProtonSplitTunneling>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonSplitTunneling {
    pub enabled: bool,
    pub mode: Option<String>,
}

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

    /// Load Proton VPN settings from ~/.config/Proton/VPN/settings.json
    pub fn load_proton_settings() -> Option<ProtonSettings> {
        let config_path = dirs::config_dir()?
            .join("Proton")
            .join("VPN")
            .join("settings.json");
        let content = std::fs::read_to_string(config_path).ok()?;
        serde_json::from_str(&content).ok()
    }
}
