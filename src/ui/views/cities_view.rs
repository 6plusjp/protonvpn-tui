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
    let title = match &state.current_country_code {
        Some(code) => format!(" {} - Cities ", code),
        None => " Cities ".to_string(),
    };

    let block = Block::default().title(title).borders(Borders::ALL);

    let cities: Vec<(String, String)> = state
        .current_cities
        .iter()
        .map(|city| {
            let features = if city.features.is_empty() {
                String::new()
            } else {
                city.features.join(", ")
            };
            (city.name.clone(), features)
        })
        .collect();

    if cities.is_empty() {
        let items = vec![ListItem::new(Line::from("No cities available"))];
        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::Gray));
        f.render_stateful_widget(list, area, list_state);
        return;
    }

    let selected_idx = state.selected_server.unwrap_or(0);
    list_state.select(Some(selected_idx.min(cities.len() - 1)));

    let items: Vec<ListItem> = cities
        .iter()
        .enumerate()
        .map(|(idx, (city_name, features))| {
            let is_selected = state.selected_server == Some(idx);

            let row = if features.is_empty() {
                city_name.clone()
            } else {
                format!("{:<15} {}", city_name, features)
            };
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
