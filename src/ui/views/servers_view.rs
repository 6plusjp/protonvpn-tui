use crate::state::ConnectionState;
use crate::ui::components::{centered_block, connected_list_item, styled_list_item};
use crate::ui::styles::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::AppState;

pub fn render_servers_view(
    state: &mut AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let theme = Theme::default();
    let block = centered_block("Servers", &theme);

    let servers = state.filtered_servers();

    if let Some(idx) = state.selected_server {
        list_state.select(Some(idx));
    }

    let connected_server_id = match &state.connection {
        ConnectionState::Connected { server, .. } => Some(server.clone()),
        _ => None,
    };

    let max_country_len = servers
        .iter()
        .map(|s| s.country.len())
        .max()
        .unwrap_or(0)
        .max(8);
    let country_width = max_country_len + 2;

    let items: Vec<ListItem> = servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let is_selected = state.selected_server == Some(idx);
            let is_connected = connected_server_id
                .as_ref()
                .map(|cid| cid.starts_with(&server.id) || server.id.starts_with(cid))
                .unwrap_or(false);

            let cities_str = if server.cities.is_empty() {
                "-".to_string()
            } else {
                server
                    .cities
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let row = format!(
                "{:<4} {:<width$} {}",
                server.id,
                server.country,
                cities_str,
                width = country_width
            );

            if is_connected {
                connected_list_item(&row, &theme)
            } else {
                styled_list_item(&row, is_selected, &theme)
            }
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_stateful_widget(list, area, list_state);
}
