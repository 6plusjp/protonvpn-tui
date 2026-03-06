//! Proton VPN settings

use serde::{Deserialize, Serialize};

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
