use crate::config::Settings;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::AppState;

pub fn render_settings_view(state: &mut AppState, f: &mut Frame<'_>, area: Rect) {
    let block = Block::default().title(" Settings ").borders(Borders::ALL);

    let proton_settings = Settings::load_proton_settings();

    let settings = match proton_settings {
        Some(ps) => {
            let killswitch = match ps.killswitch {
                Some(0) => "off",
                Some(1) => "on",
                _ => "unknown",
            };
            let ipv6 = if ps.ipv6 == Some(true) {
                "enabled"
            } else {
                "disabled"
            };
            let dns = if ps.custom_dns.enabled {
                format!("custom ({})", ps.custom_dns.ip_list.join(", "))
            } else {
                "default".to_string()
            };
            let netshield = match ps.features.as_ref().and_then(|f| f.netshield) {
                Some(0) => "off",
                Some(1) => "standard",
                Some(2) => "plus",
                _ => "unknown",
            };
            let moderate_nat = ps
                .features
                .as_ref()
                .and_then(|f| f.moderate_nat)
                .map(|v| if v { "on" } else { "off" })
                .unwrap_or("off");
            let vpn_accelerator = ps
                .features
                .as_ref()
                .and_then(|f| f.vpn_accelerator)
                .map(|v| if v { "on" } else { "off" })
                .unwrap_or("off");
            let port_forwarding = ps
                .features
                .as_ref()
                .and_then(|f| f.port_forwarding)
                .map(|v| if v { "on" } else { "off" })
                .unwrap_or("off");

            vec![
                format!("Kill Switch: {}", killswitch),
                format!("IPv6: {}", ipv6),
                format!("DNS: {}", dns),
                format!("NetShield: {}", netshield),
                format!("Moderate NAT: {}", moderate_nat),
                format!("VPN Accelerator: {}", vpn_accelerator),
                format!("Port Forwarding: {}", port_forwarding),
            ]
        }
        None => vec!["No Proton settings found".to_string()],
    };

    let items: Vec<ListItem> = settings
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let is_selected = state.settings_selected == Some(idx);
            let line = if is_selected {
                Line::from(vec![
                    Span::raw("> "),
                    Span::styled(
                        s.as_str(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![Span::raw("  "), Span::raw(s.as_str())])
            };
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}
