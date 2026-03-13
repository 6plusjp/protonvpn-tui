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
    let all_servers = state.vpn_state.servers();

    // Extract country code from connected server (e.g., "US#1" -> "US")
    let connected_country_code = match &state.connection_manager.connection {
        ConnectionState::Connected { server, .. } => {
            server.split('#').next().filter(|c| c.len() >= 2)
        }
        _ => None,
    };

    let is_focused = state.ui_state.pane_focus == Pane::Countries;

    let max_country_len = all_servers
        .iter()
        .map(|s| s.country.len())
        .max()
        .unwrap_or(22);
    let max_cities_len = all_servers
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

    let dynamic_widths = [6, max_country_len, max_cities_len];

    let sort_by = state.ui_state.sort;
    let sort_direction = state.ui_state.sort_direction;
    let header_row =
        countries_table.header_row_with_sort(&dynamic_widths, Some(sort_by), sort_direction);
    let widths = countries_table.column_widths(&dynamic_widths);

    let rows: Vec<Row> = servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let is_selected = state.ui_state.selected_server == Some(idx);
            let is_connected = connected_country_code
                .as_ref()
                .map(|code| server.code.starts_with(code) || code.starts_with(&server.code))
                .unwrap_or(false);

            let is_loading_this = state
                .connection_manager
                .pending_cities
                .contains_key(&server.code);

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
                Cell::from(server.code.clone()),
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
        .column_spacing(2)
        .highlight_style(if is_focused {
            Style::default()
                .fg(theme.foreground)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        })
        .highlight_symbol("> ");

    table_state.select(state.ui_state.selected_server);
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
    let is_focused = state.ui_state.pane_focus == Pane::Cities;

    let is_loading = state
        .connection_manager
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
        .unwrap_or(8);
    let dynamic_widths = [max_features_len];

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

    let selected = state.ui_state.selected_city;
    table_state.select(selected);

    let rows: Vec<Row> = cities
        .iter()
        .enumerate()
        .map(|(idx, (city_name, features))| {
            let is_selected = state.ui_state.selected_city == Some(idx);

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
        .column_spacing(2)
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
