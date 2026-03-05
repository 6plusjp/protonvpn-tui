use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::AppState;

pub fn render_cities_view(
    state: &mut AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let selected_country = state.selected_server.and_then(|idx| {
        let servers = state.filtered_servers();
        servers.get(idx).map(|s| (s.id.clone(), s.country.clone()))
    });

    let title = match &selected_country {
        Some((_id, country)) => format!(" {} - Cities ", country),
        None => " Cities ".to_string(),
    };

    let block = Block::default().title(title).borders(Borders::ALL);

    let cities: Vec<(String, String)> = vec![
        ("Tokyo".to_string(), "P2P, Secure".to_string()),
        ("Osaka".to_string(), "P2P".to_string()),
    ];

    if let Some(idx) = state.selected_server {
        list_state.select(Some(idx));
    }

    let items: Vec<ListItem> = cities
        .iter()
        .enumerate()
        .map(|(idx, (city, features))| {
            let is_selected = state.selected_server == Some(idx);

            let row = format!("{:<15} {}", city, features);
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
