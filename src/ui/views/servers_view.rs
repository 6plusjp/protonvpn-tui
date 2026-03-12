use crate::state::{ConnectionState, Pane};
use crate::ui::components::{centered_block, CitiesTable, CountriesTable};
use crate::ui::styles::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};

use crate::AppState;

pub fn render_servers_view(
    state: &mut AppState,
    countries_list_state: &mut TableState,
    cities_list_state: &mut TableState,
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
    table_state: &mut TableState,
    f: &mut Frame<'_>,
    area: Rect,
    theme: &Theme,
) {
    let countries_table = CountriesTable::table();

    let servers = state.filtered_servers();

    let connected_server_id = match state.get_connection() {
        ConnectionState::Connected { server, .. } => Some(server.clone()),
        _ => None,
    };

    let is_focused = state.get_pane_focus() == Pane::Countries;

    let max_country_len = servers
        .iter()
        .map(|s| s.country.len())
        .max()
        .unwrap_or(0)
        .max(8);
    let max_cities_len = servers
        .iter()
        .map(|s| {
            if s.cities.is_empty() {
                1
            } else {
                s.cities.iter().map(|c| c.name.len()).sum::<usize>() + s.cities.len() * 2
            }
        })
        .max()
        .unwrap_or(1);

    let dynamic_widths = [
        4,
        max_country_len.clamp(8, 15) + 2,
        max_cities_len.clamp(8, 30) + 2,
    ];

    let header_row = countries_table.header_row(&dynamic_widths);
    let widths = countries_table.column_widths(&dynamic_widths);

    let rows: Vec<Row> = servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let is_selected = state.get_selected_server() == Some(idx);
            let is_connected = connected_server_id
                .as_ref()
                .map(|cid| cid.starts_with(&server.id) || server.id.starts_with(cid))
                .unwrap_or(false);

            let is_loading_this = state.pending_cities.contains_key(&server.id);

            let cities_str = if is_loading_this {
                "◐".to_string()
            } else if server.cities.is_empty() {
                "-".to_string()
            } else {
                server
                    .cities
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            let style = if is_selected {
                if is_focused {
                    Style::default()
                        .fg(theme.foreground)
                        .bg(theme.selection)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground)
                }
            } else if is_connected {
                Style::default()
                    .fg(theme.connected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };

            Row::new(vec![
                Cell::from(server.id.clone()),
                Cell::from(server.country.clone()),
                Cell::from(cities_str),
            ])
            .style(style)
        })
        .collect();

    let title = countries_table.title_with_indicator(is_focused);
    let block = centered_block(&title, theme);

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block)
        .highlight_style(if is_focused {
            Style::default()
                .fg(theme.foreground)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        })
        .highlight_symbol("> ");

    table_state.select(state.get_selected_server());
    f.render_stateful_widget(table, area, table_state);
}

fn render_cities_pane(
    state: &mut AppState,
    table_state: &mut TableState,
    f: &mut Frame<'_>,
    area: Rect,
    theme: &Theme,
) {
    let cities_table = CitiesTable::table();
    let is_focused = state.get_pane_focus() == Pane::Cities;

    let is_loading = state
        .pending_cities
        .contains_key(state.current_country_code.as_deref().unwrap_or(""));

    let title = match (
        &state.current_country_code,
        state.current_cities.is_empty(),
        is_loading,
    ) {
        (None, _, _) => cities_table.title_with_indicator(is_focused),
        (Some(_), _, true) => {
            format!(
                "{} - Loading...",
                cities_table.title_with_indicator(is_focused)
            )
        }
        (Some(_), true, false) => {
            format!(
                "{} - No cities",
                cities_table.title_with_indicator(is_focused)
            )
        }
        (Some(_), false, false) => cities_table.title_with_indicator(is_focused),
    };

    let max_city_len = state
        .current_cities
        .iter()
        .map(|c| c.name.len())
        .max()
        .unwrap_or(0)
        .max(8);
    let max_features_len = state
        .current_cities
        .iter()
        .map(|c| {
            if c.features.is_empty() {
                0
            } else {
                c.features.join(", ").len()
            }
        })
        .max()
        .unwrap_or(0)
        .max(10);
    let dynamic_widths = [max_city_len + 2, max_features_len.min(30) + 2];

    let header_row = cities_table.header_row(&dynamic_widths);
    let widths = cities_table.column_widths(&dynamic_widths);

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

    if cities.is_empty() && !is_loading {
        let block = centered_block(&title, theme);
        let paragraph = ratatui::widgets::Paragraph::new("No cities available")
            .block(block)
            .style(Style::default().fg(theme.secondary));
        f.render_widget(paragraph, area);
        return;
    }

    let selected = state.get_selected_city();
    table_state.select(selected);

    let rows: Vec<Row> = cities
        .iter()
        .enumerate()
        .map(|(idx, (city_name, features))| {
            let is_selected = state.selected_city == Some(idx);

            let style = if is_selected && is_focused {
                Style::default()
                    .fg(theme.foreground)
                    .bg(theme.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };

            if features.is_empty() {
                Row::new(vec![Cell::from(city_name.clone())]).style(style)
            } else {
                Row::new(vec![
                    Cell::from(city_name.clone()),
                    Cell::from(features.clone()),
                ])
                .style(style)
            }
        })
        .collect();

    let block = centered_block(&title, theme);

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block)
        .highlight_style(if is_focused {
            Style::default()
                .fg(theme.foreground)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        })
        .highlight_symbol("> ");

    f.render_stateful_widget(table, area, table_state);
}
