use crate::state::{ConnectionState, Pane};
use crate::ui::components::{centered_block, connected_list_item, styled_list_item};
use crate::ui::styles::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::AppState;

pub fn render_servers_view(
    state: &mut AppState,
    countries_list_state: &mut ListState,
    cities_list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let theme = state.get_theme();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_countries_pane(state, countries_list_state, f, chunks[0], &theme);
    render_cities_pane(state, cities_list_state, f, chunks[1], &theme);
}

fn render_countries_pane(
    state: &mut AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
    theme: &Theme,
) {
    let title = if state.pane_focus == Pane::Countries {
        "> Countries"
    } else {
        "  Countries"
    };
    let block = centered_block(title, theme);

    let servers = state.filtered_servers();

    if let Some(idx) = state.selected_server {
        list_state.select(Some(idx));
    }

    let connected_server_id = match &state.connection {
        ConnectionState::Connected { server, .. } => Some(server.clone()),
        _ => None,
    };

    let is_focused = state.pane_focus == Pane::Countries;

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

            if is_selected {
                styled_list_item(&row, true, is_focused, theme)
            } else if is_connected {
                connected_list_item(&row, theme)
            } else {
                styled_list_item(&row, false, is_focused, theme)
            }
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_stateful_widget(list, area, list_state);
}

fn render_cities_pane(
    state: &mut AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
    theme: &Theme,
) {
    let title = match (&state.current_country_code, state.current_cities.is_empty()) {
        (None, _) => {
            if state.pane_focus == Pane::Cities {
                "> Select country".to_string()
            } else {
                "  Select country".to_string()
            }
        }
        (Some(code), true) => {
            if state.pane_focus == Pane::Cities {
                format!("> {} - Loading...", code)
            } else {
                format!("  {} - Loading...", code)
            }
        }
        (Some(code), false) => {
            if state.pane_focus == Pane::Cities {
                format!("> {} - Cities", code)
            } else {
                format!("  {} - Cities", code)
            }
        }
    };
    let block = centered_block(&title, theme);

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
            .style(Style::default().fg(theme.secondary));
        f.render_stateful_widget(list, area, list_state);
        return;
    }

    let selected_idx = state.selected_city.unwrap_or(0);
    list_state.select(Some(selected_idx.min(cities.len() - 1)));

    let is_focused = state.pane_focus == Pane::Cities;

    let items: Vec<ListItem> = cities
        .iter()
        .enumerate()
        .map(|(idx, (city_name, features))| {
            let is_selected = state.selected_city == Some(idx);

            let row = if features.is_empty() {
                city_name.clone()
            } else {
                format!("{:<15} {}", city_name, features)
            };

            styled_list_item(&row, is_selected, is_focused, theme)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_stateful_widget(list, area, list_state);
}
