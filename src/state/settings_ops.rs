//! Settings management operations

use crate::config::SettingKey;
use crate::state::InputMode;
use crate::ui::ThemeMode;

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
                self.vpn_state.toggle_setting(key, current)
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
                self.vpn_state.toggle_setting(key, current)
            }
            SettingKey::VpnAccelerator => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.vpn_accelerator);
                self.vpn_state.toggle_setting(key, current)
            }
            SettingKey::PortForwarding => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.port_forwarding);
                self.vpn_state.toggle_setting(key, current)
            }
            SettingKey::AnonymousCrashReports => {
                let current = self
                    .config_state
                    .proton_settings_cache
                    .as_ref()
                    .and_then(|p| p.anonymous_crash_reports);
                self.vpn_state.toggle_setting(key, current)
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
                self.config_state.clear_cache();
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
                        self.config_state.clear_cache();
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
                self.config_state.clear_cache();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::state::NotificationType;
    use crate::test_helpers::test_helpers::{self, make_servers};
    use crate::vpn::VpnClient;

    use test_helpers::setup;

    #[test]
    fn test_is_valid_ip_valid() {
        assert!(is_valid_ip("1.1.1.1"));
        assert!(is_valid_ip("255.255.255.255"));
        assert!(is_valid_ip("192.168.0.1"));
    }

    #[test]
    fn test_is_valid_ip_invalid() {
        assert!(!is_valid_ip(""));
        assert!(!is_valid_ip("1.2.3"));
        assert!(!is_valid_ip("1.2.3.4.5"));
        assert!(!is_valid_ip("a.b.c.d"));
        assert!(!is_valid_ip("256.0.0.1"));
        assert!(!is_valid_ip("1.2.3.4."));
        assert!(!is_valid_ip("1.2.3.4.5"));
    }

    #[test]
    fn test_apply_dns_setting_empty() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.apply_dns_setting("");
        let last = state.notification_state.notifications.last().unwrap();
        assert_eq!(last.notification_type, NotificationType::Error);
        assert!(last.message.contains("No DNS IPs provided"));
    }

    #[test]
    fn test_apply_dns_setting_invalid_ip() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.apply_dns_setting("invalid");
        let last = state.notification_state.notifications.last().unwrap();
        assert_eq!(last.notification_type, NotificationType::Error);
        assert!(last.message.contains("Invalid IP address"));
    }

    #[test]
    fn test_apply_dns_setting_valid_ip_but_command_fails() {
        setup();
        let mut state = AppState::new();
        // Use a non-existent CLI path to prevent real protonvpn execution during tests.
        // This simulates command failure without modifying system DNS settings.
        state.vpn_state =
            std::sync::Arc::new(VpnClient::with_path("nonexistent_protonvpn_for_testing"));
        state.apply_dns_setting("1.1.1.1");
        let last = state.notification_state.notifications.last().unwrap();
        assert!(!last.message.contains("Invalid IP address"));
    }

    #[test]
    fn test_toggle_settings_invalid_index() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.toggle_settings(999);
        let last = state.notification_state.notifications.last().unwrap();
        assert_eq!(last.notification_type, NotificationType::Error);
        assert!(last.message.contains("Invalid setting selection"));
    }
}
