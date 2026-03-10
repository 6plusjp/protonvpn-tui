use crate::config::SettingKey;
use crate::ui::components::{centered_block, styled_list_item};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{List, ListItem},
    Frame,
};

use crate::AppState;

pub fn render_settings_view(state: &mut AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.get_theme();
    let block = centered_block("Settings", &theme);

    let proton_settings = state.get_proton_settings();

    let settings: Vec<String> = match proton_settings {
        Some(ps) => SettingKey::ALL
            .iter()
            .map(|key| {
                let value = match key {
                    SettingKey::Killswitch => match ps.killswitch {
                        Some(0) => "off".to_string(),
                        Some(1) => "on".to_string(),
                        _ => "unknown".to_string(),
                    },
                    SettingKey::Ipv6 => {
                        if ps.ipv6 == Some(true) {
                            "enabled".to_string()
                        } else {
                            "disabled".to_string()
                        }
                    }
                    SettingKey::Dns => {
                        if ps.custom_dns.enabled {
                            let ips: Vec<String> =
                                ps.custom_dns.ip_list.iter().map(|d| d.ip.clone()).collect();
                            format!("custom ({})", ips.join(", "))
                        } else {
                            "default".to_string()
                        }
                    }
                    SettingKey::NetShield => match ps.features.as_ref().and_then(|f| f.netshield) {
                        Some(0) => "off".to_string(),
                        Some(1) => "malware-only".to_string(),
                        Some(2) => "malware-ads-trackers".to_string(),
                        _ => "unknown".to_string(),
                    },
                    SettingKey::ModerateNat => ps
                        .features
                        .as_ref()
                        .and_then(|f| f.moderate_nat)
                        .map(|v| {
                            if v {
                                "on".to_string()
                            } else {
                                "off".to_string()
                            }
                        })
                        .unwrap_or_else(|| "off".to_string()),
                    SettingKey::VpnAccelerator => ps
                        .features
                        .as_ref()
                        .and_then(|f| f.vpn_accelerator)
                        .map(|v| {
                            if v {
                                "on".to_string()
                            } else {
                                "off".to_string()
                            }
                        })
                        .unwrap_or_else(|| "off".to_string()),
                    SettingKey::PortForwarding => ps
                        .features
                        .as_ref()
                        .and_then(|f| f.port_forwarding)
                        .map(|v| {
                            if v {
                                "on".to_string()
                            } else {
                                "off".to_string()
                            }
                        })
                        .unwrap_or_else(|| "off".to_string()),
                    SettingKey::Theme => {
                        if state.is_dark_theme {
                            "Dark".to_string()
                        } else {
                            "Light".to_string()
                        }
                    }
                };

                let label = match key {
                    SettingKey::Killswitch => "Kill Switch:      ",
                    SettingKey::Ipv6 => "IPv6:             ",
                    SettingKey::Dns => "DNS:              ",
                    SettingKey::NetShield => "NetShield:        ",
                    SettingKey::ModerateNat => "Moderate NAT:     ",
                    SettingKey::VpnAccelerator => "VPN Accelerator:  ",
                    SettingKey::PortForwarding => "Port Forwarding:  ",
                    SettingKey::Theme => "Theme:            ",
                };

                format!("{}{}", label, value)
            })
            .collect(),
        None => {
            vec!["Loading settings...".to_string()]
        }
    };

    let items: Vec<ListItem> = settings
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let is_selected = state.settings_selected == Some(idx);
            styled_list_item(s, is_selected, true, &theme)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_widget(list, area);
}
