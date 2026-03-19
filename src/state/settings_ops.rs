//! Settings management operations

use crate::config::SettingKey;
use crate::state::InputMode;
use crate::ui::styles::ThemeMode;

impl crate::state::AppState {
    pub fn toggle_settings(&mut self, index: usize) {
        let key = match SettingKey::from_index(index) {
            Some(k) => k,
            None => {
                self.show_notification(
                    "Invalid setting selection".to_string(),
                    crate::state::NotificationType::Error,
                    None,
                );
                return;
            }
        };

        if key == SettingKey::Dns {
            self.ui_state.input_mode = InputMode::DnsInput;
            self.ui_state.dns_input = String::new();
            self.show_notification(
                "Enter DNS IPs (e.g., 1.1.1.1,9.9.9.9)".to_string(),
                crate::state::NotificationType::Info,
                None,
            );
            return;
        }

        if key == SettingKey::Theme {
            let old_mode = self.ui_state.theme_mode;
            self.ui_state.toggle_theme();
            let new_mode = self.ui_state.theme_mode;
            let theme_name = match new_mode {
                ThemeMode::System => "System",
                ThemeMode::CatppuccinMocha => "Catppuccin Mocha",
                ThemeMode::CatppuccinLatte => "Catppuccin Latte",
                ThemeMode::Dracula => "Dracula",
                ThemeMode::Nord => "Nord",
                ThemeMode::Gruvbox => "Gruvbox",
                ThemeMode::TokyoNight => "Tokyo Night",
            };
            tracing::info!("Theme changed from {:?} to {:?}", old_mode, new_mode);
            self.show_notification(
                format!("Theme changed to {}", theme_name),
                crate::state::NotificationType::Info,
                None,
            );
            return;
        }

        let ps = self.config_state.proton_settings_cache.as_ref();
        let result = match key {
            SettingKey::Killswitch => {
                let current = ps.and_then(|p| p.killswitch);
                self.vpn_state.toggle_killswitch(current)
            }
            SettingKey::Ipv6 => {
                let current = ps.and_then(|p| p.ipv6);
                self.vpn_state.toggle_ipv6(current)
            }
            SettingKey::Dns => unreachable!(),
            SettingKey::NetShield => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.netshield);
                let next = 0;
                self.vpn_state.set_netshield(current, next)
            }
            SettingKey::ModerateNat => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.moderate_nat);
                self.vpn_state.toggle_moderate_nat(current)
            }
            SettingKey::VpnAccelerator => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.vpn_accelerator);
                self.vpn_state.toggle_vpn_accelerator(current)
            }
            SettingKey::PortForwarding => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.port_forwarding);
                self.vpn_state.toggle_port_forwarding(current)
            }
            SettingKey::AnonymousCrashReports => {
                let current = self
                    .config_state
                    .proton_settings_cache
                    .as_ref()
                    .and_then(|p| p.anonymous_crash_reports);
                self.vpn_state.toggle_anonymous_crash_reports(current)
            }
            SettingKey::Theme => unreachable!(),
            SettingKey::Footer => unreachable!(),
        };

        match result {
            Ok(msg) => {
                tracing::info!("Setting updated: {}", msg);
                self.show_notification(
                    format!("Setting updated: {}", msg),
                    crate::state::NotificationType::Success,
                    None,
                );
                self.config_state.proton_settings_cache = None;
            }
            Err(e) => {
                tracing::warn!("Failed to update setting: {}", e);
                self.show_notification(
                    format!("Failed to update setting: {}", e),
                    crate::state::NotificationType::Error,
                    None,
                );
            }
        }
    }

    pub fn toggle_settings_off(&mut self, index: usize) {
        let key = match SettingKey::from_index(index) {
            Some(k) => k,
            None => return,
        };

        if key == SettingKey::Dns {
            let ps = self.config_state.proton_settings_cache.as_ref();
            let dns_enabled = ps.map(|p| p.custom_dns.enabled).unwrap_or(false);
            if dns_enabled {
                match self.vpn_state.disable_custom_dns() {
                    Ok(msg) => {
                        self.show_notification(
                            format!("DNS disabled: {}", msg),
                            crate::state::NotificationType::Success,
                            None,
                        );
                        self.config_state.proton_settings_cache = None;
                    }
                    Err(e) => {
                        self.show_notification(
                            format!("Failed to disable DNS: {}", e),
                            crate::state::NotificationType::Error,
                            None,
                        );
                    }
                }
            } else {
                self.show_notification(
                    "DNS is already off".to_string(),
                    crate::state::NotificationType::Info,
                    None,
                );
            }
            return;
        }

        self.toggle_settings(index);
    }

    pub fn apply_dns_setting(&mut self, dns_ips: &str) {
        let ips: Vec<&str> = dns_ips
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if ips.is_empty() {
            self.show_notification(
                "No DNS IPs provided".to_string(),
                crate::state::NotificationType::Error,
                None,
            );
            return;
        }

        for ip in &ips {
            if !is_valid_ip(ip) {
                self.show_notification(
                    format!("Invalid IP address: {}", ip),
                    crate::state::NotificationType::Error,
                    None,
                );
                return;
            }
        }

        let dns_list = ips.join(",");
        let result = self.vpn_state.set_custom_dns(&dns_list);

        match result {
            Ok(msg) => {
                self.show_notification(
                    format!("DNS updated: {}", msg),
                    crate::state::NotificationType::Success,
                    None,
                );
                self.config_state.proton_settings_cache = None;
            }
            Err(e) => {
                self.show_notification(
                    format!("Failed to update DNS: {}", e),
                    crate::state::NotificationType::Error,
                    None,
                );
            }
        }
    }

    pub fn apply_setting(&mut self, key: &str, value: &str) -> Result<String, String> {
        self.vpn_state
            .set_config(key, value)
            .map_err(|e| e.to_string())
    }

    pub fn clear_settings_cache(&mut self) {
        self.config_state.clear_cache();
    }
}

fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts
        .iter()
        .all(|p| p.chars().all(|c| c.is_ascii_digit()) && p.parse::<u8>().is_ok())
}
