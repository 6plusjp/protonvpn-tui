use crate::ui::components::centered_block;
use crate::ui::styles::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{List, ListItem},
    Frame,
};

use crate::state::ConnectionState;

pub fn render_stats_view(state: &mut crate::AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = if state.is_dark_theme {
        Theme::dark()
    } else {
        Theme::light()
    };
    let block = centered_block("Statistics", &theme);

    let proton_settings = state.get_proton_settings();

    let mut stats = Vec::new();

    if let Some(ps) = proton_settings {
        if let Some(protocol) = &ps.protocol {
            stats.push(format!("Protocol: {}", protocol));
        }
        if let Some(features) = &ps.features {
            if let Some(st) = &features.split_tunneling {
                let mode = st.mode.as_deref().unwrap_or("unknown");
                let status = if st.enabled { "on" } else { "off" };
                stats.push(format!("Split Tunneling: {} ({})", status, mode));
            }
        }
    }

    stats.extend([
        "Download Speed: 0.00 KB/s".to_string(),
        "Upload Speed: 0.00 KB/s".to_string(),
        "Total Received: 0 B".to_string(),
        "Total Sent: 0 B".to_string(),
        "Session Time: 00:00:00".to_string(),
    ]);

    if let ConnectionState::Connected { ref server, ref ip } = state.connection {
        stats.push(format!("Server: {}", server));
        if !ip.is_empty() {
            stats.push(format!("IP: {}", ip));
        }
    }

    let items: Vec<ListItem> = stats.iter().map(|s| ListItem::new(s.as_str())).collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_widget(list, area);
}
