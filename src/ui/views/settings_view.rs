use crate::config::SettingKey;
use crate::ui::components::{centered_block, styled_list_item};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::AppState;

pub fn render_settings_view(
    state: &mut AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let theme = state.get_theme();
    let block = centered_block("Settings", &theme);

    let proton_settings = state.get_proton_settings();
    let selected = state.settings_selected.unwrap_or(0);
    let expanded = state.settings_expanded;
    let option_selected = state.settings_option_selected;

    let mut all_items: Vec<ListItem> = Vec::new();

    for (idx, key) in SettingKey::ALL.iter().enumerate() {
        let value = get_setting_value(key, proton_settings);
        let label = get_setting_label(key);

        let is_selected = selected == idx;
        let is_expanded = expanded && is_selected;

        let main_line = if is_expanded {
            format!("> {} {}", label.trim(), value)
        } else {
            format!("  {} {}", label.trim(), value)
        };

        all_items.push(styled_list_item(&main_line, is_selected, true, &theme));

        if is_expanded {
            let options = key.selectable_options();
            let opt_len = options.len();
            if opt_len > 0 {
                for (opt_idx, option) in options.iter().enumerate() {
                    let prefix = if opt_idx == option_selected {
                        if opt_idx == opt_len - 1 {
                            "└►"
                        } else {
                            "├►"
                        }
                    } else if opt_idx == opt_len - 1 {
                        "└─"
                    } else {
                        "│ "
                    };
                    let opt_line = format!("  {} {}", prefix, option);
                    all_items.push(
                        ListItem::from(Line::from(opt_line))
                            .style(Style::default().fg(theme.foreground).dim()),
                    );
                }
            }
        }
    }

    let items_len = all_items.len();

    let list = List::new(all_items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    let selected = selected.min(items_len.saturating_sub(1));
    list_state.select(Some(selected));

    f.render_stateful_widget(list, area, list_state);
}

fn get_setting_value(
    key: &SettingKey,
    proton_settings: Option<&crate::config::ProtonSettings>,
) -> String {
    let ps = match proton_settings {
        Some(p) => p,
        None => return "unknown".to_string(),
    };

    match key {
        SettingKey::Killswitch => match ps.killswitch {
            Some(0) => "off".to_string(),
            Some(1) => "standard".to_string(),
            _ => "unknown".to_string(),
        },
        SettingKey::Ipv6 => {
            if ps.ipv6 == Some(true) {
                "on".to_string()
            } else if ps.ipv6 == Some(false) {
                "off".to_string()
            } else {
                "unknown".to_string()
            }
        }
        SettingKey::Dns => {
            if ps.custom_dns.enabled {
                let ips: Vec<String> = ps.custom_dns.ip_list.iter().map(|d| d.ip.clone()).collect();
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
            .unwrap_or_else(|| "unknown".to_string()),
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
            .unwrap_or_else(|| "unknown".to_string()),
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
            .unwrap_or_else(|| "unknown".to_string()),
        SettingKey::Theme => "Dark".to_string(),
    }
}

fn get_setting_label(key: &SettingKey) -> &'static str {
    match key {
        SettingKey::Killswitch => "Kill Switch:      ",
        SettingKey::Ipv6 => "IPv6:             ",
        SettingKey::Dns => "DNS:              ",
        SettingKey::NetShield => "NetShield:        ",
        SettingKey::ModerateNat => "Moderate NAT:     ",
        SettingKey::VpnAccelerator => "VPN Accelerator:  ",
        SettingKey::PortForwarding => "Port Forwarding:  ",
        SettingKey::Theme => "Theme:            ",
    }
}
