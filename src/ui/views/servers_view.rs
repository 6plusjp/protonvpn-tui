use crate::state::{ConnectionState, Pane};
use crate::ui::components::{
    centered_block, connected_list_item, styled_list_item, CitiesTable, CountriesTable,
};
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
    let countries_table = CountriesTable::table();

    let servers = state.filtered_servers();

    list_state.select(state.selected_server);

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

    let dynamic_widths = [4, max_country_len + 2, max_cities_len + 2];

    let header = countries_table.header_with_widths(&dynamic_widths);
    let title = format!(
        "{}\n{}",
        countries_table.title_with_indicator(is_focused),
        header
    );
    let block = centered_block(&title, theme);

    let items: Vec<ListItem> = servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let is_selected = state.selected_server == Some(idx);
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
            let row = countries_table.format_row_with_widths(
                &[&server.id, &server.country, &cities_str],
                &dynamic_widths,
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
    let cities_table = CitiesTable::table();
    let is_focused = state.pane_focus == Pane::Cities;

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
        .unwrap_or(0);
    let dynamic_widths = [max_city_len + 2, max_features_len + 2];

    let header_row = cities_table.header_with_widths(&dynamic_widths);
    let title_with_header = format!("{}\n{}", title, header_row);
    let block = centered_block(&title_with_header, theme);

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
        let items = vec![ListItem::new(Line::from("No cities available"))];
        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(theme.secondary));
        f.render_stateful_widget(list, area, list_state);
        return;
    }

    let is_focused = state.pane_focus == Pane::Cities;

    let selected = state.selected_city;
    list_state.select(selected);

    let city_items: Vec<ListItem> = cities
        .iter()
        .enumerate()
        .map(|(idx, (city_name, features))| {
            let is_selected = state.selected_city == Some(idx);

            let row = if features.is_empty() {
                cities_table.format_row_with_widths(&[city_name], &[dynamic_widths[0], 0])
            } else {
                cities_table.format_row_with_widths(&[city_name, features], &dynamic_widths)
            };

            styled_list_item(&row, is_selected, is_focused, theme)
        })
        .collect();

    let list = List::new(city_items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_stateful_widget(list, area, list_state);
}
