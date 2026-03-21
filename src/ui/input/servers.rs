use crossterm::event::{KeyCode, KeyEvent};

use super::app_action::AppAction;
use super::common::handle_common_navigation;
use super::InputState;
use crate::state::Pane;

pub fn handle_servers_key(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    let bindings = &input.state.key_bindings;

    let is_refresh = bindings
        .refresh
        .matches(key_event.code, key_event.modifiers);
    let is_pane_next = bindings
        .pane_next
        .matches(key_event.code, key_event.modifiers);
    let is_pane_prev = bindings
        .pane_prev
        .matches(key_event.code, key_event.modifiers);
    let is_sort_by_code = bindings
        .sort_by_code
        .matches(key_event.code, key_event.modifiers);
    let is_sort_by_country = bindings
        .sort_by_country
        .matches(key_event.code, key_event.modifiers);

    match key_event.code {
        _ if is_pane_next => {
            input.state.move_to_cities();
            None
        }
        _ if is_pane_prev => {
            if input.state.ui_state.pane_focus == Pane::Cities {
                input.state.move_to_countries();
            }
            None
        }
        KeyCode::Backspace => {
            if input.state.ui_state.pane_focus == Pane::Cities {
                input.state.move_to_countries();
            }
            None
        }
        KeyCode::Enter => {
            if input.state.ui_state.pane_focus == Pane::Cities {
                if let Some(idx) = input.state.ui_state.selected_city {
                    let city_name = input.state.current_cities.get(idx).map(|c| c.name.clone());
                    if let Some(name) = city_name {
                        input.state.connect_city(&name);
                    }
                }
            } else {
                input.state.move_to_cities();
            }
            None
        }
        _ if handle_common_navigation(input, key_event) => None,
        _ if is_refresh => {
            match input.state.ui_state.pane_focus {
                Pane::Cities => {
                    input.state.reload_cities();
                }
                Pane::Countries | Pane::Settings | Pane::Logs => {
                    input.state.refresh_servers();
                }
            }
            None
        }
        _ if is_sort_by_code => {
            input.state.set_sort_by_code();
            None
        }
        _ if is_sort_by_country => {
            input.state.set_sort_by_country();
            None
        }
        KeyCode::Left => {
            input.state.toggle_sort_direction();
            None
        }
        KeyCode::Right => {
            input.state.toggle_sort_direction();
            None
        }
        _ => None,
    }
}
