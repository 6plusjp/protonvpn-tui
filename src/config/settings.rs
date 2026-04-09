//! Proton VPN settings

use crate::paths;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Modifier key for keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyModifier {
    None,
    Control,
    Alt,
    Shift,
}

impl From<KeyModifiers> for KeyModifier {
    fn from(modifiers: KeyModifiers) -> Self {
        if modifiers.contains(KeyModifiers::CONTROL) {
            KeyModifier::Control
        } else if modifiers.contains(KeyModifiers::ALT) {
            KeyModifier::Alt
        } else if modifiers.contains(KeyModifiers::SHIFT) {
            KeyModifier::Shift
        } else {
            KeyModifier::None
        }
    }
}

impl From<KeyModifier> for KeyModifiers {
    fn from(m: KeyModifier) -> Self {
        match m {
            KeyModifier::Control => KeyModifiers::CONTROL,
            KeyModifier::Alt => KeyModifiers::ALT,
            KeyModifier::Shift => KeyModifiers::SHIFT,
            KeyModifier::None => KeyModifiers::NONE,
        }
    }
}

/// Settings key for UI rendering and toggle operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingKey {
    Killswitch,
    Ipv6,
    Dns,
    NetShield,
    ModerateNat,
    VpnAccelerator,
    PortForwarding,
    AnonymousCrashReports,
    Theme,
    Footer,
}

impl SettingKey {
    /// All settings in display order (indices match UI)
    pub const ALL: [SettingKey; 10] = [
        SettingKey::Killswitch,
        SettingKey::Ipv6,
        SettingKey::Dns,
        SettingKey::NetShield,
        SettingKey::ModerateNat,
        SettingKey::VpnAccelerator,
        SettingKey::PortForwarding,
        SettingKey::AnonymousCrashReports,
        SettingKey::Theme,
        SettingKey::Footer,
    ];

    pub fn from_index(index: usize) -> Option<SettingKey> {
        Self::ALL.get(index).copied()
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|k| k == self).unwrap_or(0)
    }

    /// Return selectable options only (excludes "unknown").
    /// "unknown" is display-only - users cannot set a setting to "unknown".
    pub fn label(&self) -> &'static str {
        match self {
            SettingKey::Killswitch => "Kill Switch:          ",
            SettingKey::Ipv6 => "IPv6:                 ",
            SettingKey::Dns => "DNS:                  ",
            SettingKey::NetShield => "NetShield:            ",
            SettingKey::ModerateNat => "Moderate NAT:         ",
            SettingKey::VpnAccelerator => "VPN Accelerator:      ",
            SettingKey::PortForwarding => "Port Forwarding:      ",
            SettingKey::AnonymousCrashReports => "Crash Reports:        ",
            SettingKey::Theme => "Theme:                ",
            SettingKey::Footer => "Footer:               ",
        }
    }

    pub fn selectable_options(&self) -> Vec<&'static str> {
        match self {
            SettingKey::Killswitch => vec!["off", "standard"],
            SettingKey::Ipv6 => vec!["off", "on"],
            SettingKey::Dns => vec!["off", "on"],
            SettingKey::NetShield => vec!["off", "malware-only", "malware-ads-trackers"],
            SettingKey::ModerateNat => vec!["off", "on"],
            SettingKey::VpnAccelerator => vec!["off", "on"],
            SettingKey::PortForwarding => vec!["off", "on"],
            SettingKey::AnonymousCrashReports => vec!["off", "on"],
            SettingKey::Theme => vec![
                "System",
                "Catppuccin Mocha",
                "Catppuccin Latte",
                "Dracula",
                "Nord",
                "Gruvbox",
                "Tokyo Night",
            ],
            SettingKey::Footer => vec!["on", "off"],
        }
    }

    /// Convenience: number of selectable options
    pub fn selectable_option_count(&self) -> usize {
        self.selectable_options().len()
    }

    /// Given a selectable option index, return the (config_key, value) pair.
    pub fn get_selectable_option_command(&self, option_index: usize) -> Option<(String, String)> {
        let opts = self.selectable_options();
        if option_index >= opts.len() {
            return None;
        }
        let value = opts[option_index];
        Some((self.config_key().to_string(), value.to_string()))
    }

    // The CLI config key name corresponding to this setting.
    pub fn config_key(&self) -> &'static str {
        match self {
            SettingKey::Killswitch => "kill-switch",
            SettingKey::Ipv6 => "ipv6",
            SettingKey::Dns => "custom-dns",
            SettingKey::NetShield => "netshield",
            SettingKey::ModerateNat => "moderate-nat",
            SettingKey::VpnAccelerator => "vpn-accelerator",
            SettingKey::PortForwarding => "port-forwarding",
            SettingKey::AnonymousCrashReports => "anonymous-crash-reports",
            SettingKey::Theme => "theme",
            SettingKey::Footer => "footer",
        }
    }
}

/// Proton VPN settings (from ~/.config/Proton/VPN/settings.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonSettings {
    pub protocol: Option<String>,
    pub killswitch: Option<i32>,
    pub ipv6: Option<bool>,
    #[serde(rename = "custom_dns")]
    pub custom_dns: ProtonCustomDns,
    #[serde(rename = "anonymous_crash_reports")]
    pub anonymous_crash_reports: Option<bool>,
    pub features: Option<ProtonFeatures>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonCustomDns {
    pub enabled: bool,
    #[serde(rename = "ip_list")]
    pub ip_list: Vec<ProtonDnsIp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtonDnsIp {
    pub ip: String,
    #[serde(default)]
    pub enabled: bool,
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

impl ProtonSettings {
    pub fn load() -> Option<Self> {
        let config_path = paths::proton_settings_path()?;
        let content = std::fs::read_to_string(config_path).ok()?;
        let settings: Self = serde_json::from_str(&content).ok()?;
        tracing::debug!(
            "Loaded ProtonSettings: killswitch={:?}, ipv6={:?}, dns_enabled={}",
            settings.killswitch,
            settings.ipv6,
            settings.custom_dns.enabled
        );
        Some(settings)
    }

    pub fn settings_count(&self) -> usize {
        let mut c = 0;
        if self.killswitch.is_some() {
            c += 1;
        }
        if self.ipv6.is_some() {
            c += 1;
        }
        if self.custom_dns.enabled {
            c += 1;
        }
        if self.features.as_ref().and_then(|f| f.netshield).is_some() {
            c += 1;
        }
        if self
            .features
            .as_ref()
            .and_then(|f| f.moderate_nat)
            .is_some()
        {
            c += 1;
        }
        if self
            .features
            .as_ref()
            .and_then(|f| f.vpn_accelerator)
            .is_some()
        {
            c += 1;
        }
        if self
            .features
            .as_ref()
            .and_then(|f| f.port_forwarding)
            .is_some()
        {
            c += 1;
        }
        if self.anonymous_crash_reports.is_some() {
            c += 1;
        }
        c
    }
}

/// Collection of key bindings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub navigation_down: KeyBinding,
    pub navigation_up: KeyBinding,
    pub page_down: KeyBinding,
    pub page_up: KeyBinding,
    pub go_first: KeyBinding,
    pub go_last: KeyBinding,
    pub connect: KeyBinding,
    pub disconnect: KeyBinding,
    pub refresh: KeyBinding,
    pub random_connect: KeyBinding,
    pub pane_next: KeyBinding,
    pub pane_prev: KeyBinding,
    // Sorting keys (number keys)
    pub sort_by_code: KeyBinding,
    pub sort_by_country: KeyBinding,
    pub sort_direction: KeyBinding,
    // Connection type shortcuts
    pub connect_fastest: KeyBinding,
    pub connect_p2p: KeyBinding,
    pub connect_tor: KeyBinding,
    pub securecore: KeyBinding,
    pub toggle_favorite: KeyBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        crate::ui::default_keybindings()
    }
}

/// Key binding configuration for a single action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBinding {
    pub code: char,
    pub modifiers: KeyModifier,
}

impl KeyBinding {
    pub fn new(code: char, modifiers: KeyModifier) -> Self {
        Self { code, modifiers }
    }

    pub fn matches(&self, key_code: KeyCode, key_modifiers: KeyModifiers) -> bool {
        match key_code {
            KeyCode::Char(c) => {
                c == self.code && KeyModifier::from(key_modifiers) == self.modifiers
            }
            _ => false,
        }
    }
}
