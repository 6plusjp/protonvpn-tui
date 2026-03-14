use crate::config::SettingKey;
use crate::ui::styles::ThemeMode;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
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
    is_focused: bool,
) {
    let theme = state.get_theme();

    let proton_settings = state.config_state.proton_settings_cache.as_ref();
    let selected = state.ui_state.settings_selected.unwrap_or(0);
    let expanded = state.ui_state.settings_expanded;
    let option_selected = state.ui_state.settings_option_selected;

    let mut all_items: Vec<ListItem> = Vec::new();

    for (idx, key) in SettingKey::ALL.iter().enumerate() {
        let value = get_setting_value(key, proton_settings, state.ui_state.theme_mode);
        let label = get_setting_label(key);

        let is_selected = selected == idx;
        let is_expanded = expanded && is_selected;

        let main_line = if is_expanded {
            format!("> {} {}", label.trim(), value)
        } else {
            format!("  {} {}", label.trim(), value)
        };

        all_items.push(ListItem::new(main_line));

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
                            .style(Style::default().fg(theme.inactive)),
                    );
                }
            }
        }
    }

    let items_len = all_items.len();

    let list = List::new(all_items)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(if is_focused {
            Style::default()
                .fg(theme.foreground)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        })
        .highlight_symbol("> ");

    let selected = selected.min(items_len.saturating_sub(1));
    list_state.select(Some(selected));

    f.render_stateful_widget(list, area, list_state);
}

fn get_setting_value(
    key: &SettingKey,
    proton_settings: Option<&crate::config::ProtonSettings>,
    theme_mode: ThemeMode,
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
        SettingKey::Theme => match theme_mode {
            ThemeMode::System => "System".to_string(),
            ThemeMode::Terminal => "Terminal".to_string(),
            ThemeMode::CatppuccinMocha => "Catppuccin Mocha".to_string(),
            ThemeMode::CatppuccinLatte => "Catppuccin Latte".to_string(),
            ThemeMode::Dracula => "Dracula".to_string(),
            ThemeMode::Nord => "Nord".to_string(),
            ThemeMode::Gruvbox => "Gruvbox".to_string(),
            ThemeMode::TokyoNight => "Tokyo Night".to_string(),
        },
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
