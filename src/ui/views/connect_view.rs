use crate::state::ConnectionState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::AppState;

pub fn render_connect_view(
    state: &mut AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let block = Block::default().title(" Servers ").borders(Borders::ALL);

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
                server.cities.join(", ")
            };
            let row = format!(
                "{:<4} {:<width$} {}",
                server.id,
                server.country,
                cities_str,
                width = country_width
            );
            let line = if is_selected {
                Line::from(vec![
                    Span::raw("> "),
                    Span::styled(
                        row.to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else if is_connected {
                Line::from(vec![
                    Span::raw("* "),
                    Span::styled(
                        row.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![Span::raw("  "), Span::raw(row)])
            };

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_stateful_widget(list, area, list_state);
}
