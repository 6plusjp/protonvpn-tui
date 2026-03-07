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
                let ips: Vec<String> = ps.custom_dns.ip_list.iter().map(|d| d.ip.clone()).collect();
                format!("custom ({})", ips.join(", "))
            } else {
                "default".to_string()
            };
            let netshield = match ps.features.as_ref().and_then(|f| f.netshield) {
                Some(0) => "off",
                Some(1) => "malware-only",
                Some(2) => "malware-ads-trackers",
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
                format!("Kill Switch:      {}", killswitch),
                format!("IPv6:             {}", ipv6),
                format!("DNS:              {}", dns),
                format!("NetShield:        {}", netshield),
                format!("Moderate NAT:     {}", moderate_nat),
                format!("VPN Accelerator:  {}", vpn_accelerator),
                format!("Port Forwarding:  {}", port_forwarding),
            ]
        }
        None => {
            vec!["Loading settings...".to_string()]
        }
    };

    let mut all_settings = settings;
    all_settings.push(format!(
        "Theme:            {}",
        if state.is_dark_theme { "Dark" } else { "Light" }
    ));

    let items: Vec<ListItem> = all_settings
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
