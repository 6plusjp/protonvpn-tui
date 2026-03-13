//! Proton VPN settings

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

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
    Theme,
}

impl SettingKey {
    /// All settings in display order (indices match UI)
    pub const ALL: [SettingKey; 8] = [
        SettingKey::Killswitch,
        SettingKey::Ipv6,
        SettingKey::Dns,
        SettingKey::NetShield,
        SettingKey::ModerateNat,
        SettingKey::VpnAccelerator,
        SettingKey::PortForwarding,
        SettingKey::Theme,
    ];

    pub fn from_index(index: usize) -> Option<SettingKey> {
        Self::ALL.get(index).copied()
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|k| k == self).unwrap_or(0)
    }

    /// Return selectable options only (excludes "unknown").
    /// "unknown" is display-only - users cannot set a setting to "unknown".
    pub fn selectable_options(&self) -> Vec<&'static str> {
        match self {
            SettingKey::Killswitch => vec!["off", "standard"],
            SettingKey::Ipv6 => vec!["off", "on"],
            SettingKey::Dns => vec!["off", "on"],
            SettingKey::NetShield => vec!["off", "malware-only", "malware-ads-trackers"],
            SettingKey::ModerateNat => vec!["off", "on"],
            SettingKey::VpnAccelerator => vec!["off", "on"],
            SettingKey::PortForwarding => vec!["off", "on"],
            SettingKey::Theme => vec!["Dark", "Light"],
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
            SettingKey::Theme => "theme",
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
    /// Load Proton VPN settings from ~/.config/Proton/VPN/settings.json
    pub fn load() -> Option<Self> {
        let config_path = dirs::config_dir()?
            .join("Proton")
            .join("VPN")
            .join("settings.json");
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
        c
    }
}

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
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            navigation_down: KeyBinding::new('j', KeyModifier::None),
            navigation_up: KeyBinding::new('k', KeyModifier::None),
            page_down: KeyBinding::new('d', KeyModifier::Control),
            page_up: KeyBinding::new('u', KeyModifier::Control),
            go_first: KeyBinding::new('g', KeyModifier::None),
            go_last: KeyBinding::new('G', KeyModifier::None),
            connect: KeyBinding::new('c', KeyModifier::None),
            disconnect: KeyBinding::new('d', KeyModifier::None),
            refresh: KeyBinding::new('r', KeyModifier::None),
            random_connect: KeyBinding::new('x', KeyModifier::None),
        }
    }
}

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
