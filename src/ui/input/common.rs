use crossterm::event::{KeyCode, KeyEvent};

use super::app_action::AppAction;
use super::InputState;
use crate::state::{AppView, ConnectionState, Pane};
use crate::AppState;

fn handle_connection_command<F>(
    input: &mut InputState,
    is_match: bool,
    connect_fn: F,
) -> Option<AppAction>
where
    F: FnOnce(&mut AppState),
{
    if !is_match {
        return None;
    }

    match input.state.connection_manager.connection {
        ConnectionState::Connecting => {
            input.state.show_notification(
                "Connection in progress...".to_string(),
                crate::state::NotificationType::Warning,
                Some("connect".to_string()),
            );
        }
        ConnectionState::Disconnecting => {
            input.state.show_notification(
                "Disconnecting...".to_string(),
                crate::state::NotificationType::Warning,
                Some("disconnect".to_string()),
            );
        }
        _ => {
            connect_fn(input.state);
        }
    }

    None
}

pub fn handle_common_keys(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    let is_help_view = input.state.ui_state.current_view == AppView::Help;
    let bindings = &input.state.key_bindings;

    let is_disconnect = bindings
        .disconnect
        .matches(key_event.code, key_event.modifiers);
    let is_connect_fastest = bindings
        .connect_fastest
        .matches(key_event.code, key_event.modifiers);
    let is_connect_p2p = bindings
        .connect_p2p
        .matches(key_event.code, key_event.modifiers);
    let is_connect_tor = bindings
        .connect_tor
        .matches(key_event.code, key_event.modifiers);
    let is_securecore = bindings
        .securecore
        .matches(key_event.code, key_event.modifiers);
    let is_toggle_favorite = bindings
        .toggle_favorite
        .matches(key_event.code, key_event.modifiers);
    let is_random_connect = bindings
        .random_connect
        .matches(key_event.code, key_event.modifiers);

    if is_disconnect {
        match input.state.connection_manager.connection {
            ConnectionState::Disconnected => {
                input.state.show_notification(
                    "Not connected".to_string(),
                    crate::state::NotificationType::Info,
                    None,
                );
            }
            ConnectionState::Disconnecting => {
                input.state.show_notification(
                    "Already disconnecting...".to_string(),
                    crate::state::NotificationType::Warning,
                    None,
                );
            }
            _ => {
                input.state.disconnect();
            }
        }
        return None;
    }

    handle_connection_command(input, is_connect_fastest, |s| s.connect_fastest());
    handle_connection_command(input, is_connect_p2p, |s| s.connect_p2p());
    handle_connection_command(input, is_connect_tor, |s| s.connect_tor());
    handle_connection_command(input, is_securecore, |s| s.connect_securecore());
    handle_connection_command(input, is_random_connect, |s| s.connect_random());

    match key_event.code {
        KeyCode::Char('q') => Some(AppAction::Quit),
        KeyCode::Tab => Some(AppAction::SwitchView),
        KeyCode::Char('?') => {
            if is_help_view {
                input.state.ui_state.current_view = input.state.ui_state.previous_view;
            } else {
                input.state.ui_state.set_view(AppView::Help);
            }
            sync_view(input);
            None
        }
        KeyCode::Char('/') => {
            *input.filter_mode = true;
            *input.filter_input = input.state.ui_state.search_query.query.clone();
            None
        }
        KeyCode::Esc => {
            if is_help_view {
                input.state.ui_state.current_view = input.state.ui_state.previous_view;
                sync_view(input);
                None
            } else if !input.state.ui_state.search_query.query.is_empty() {
                input.state.set_search_query(String::new());
                input.filter_input.clear();
                None
            } else {
                None
            }
        }
        _ if is_toggle_favorite => {
            let idx = input.state.ui_state.selected_server?;
            let code = input.state.filtered_servers().get(idx)?.code.clone();
            input.state.toggle_favorite(&code);
            if let Some(new_idx) = input
                .state
                .filtered_servers()
                .iter()
                .position(|s| s.code == code)
            {
                input.state.ui_state.selected_server = Some(new_idx);
            }
            None
        }
        _ => None,
    }
}

pub fn sync_view(input: &mut InputState) {
    match input.state.ui_state.current_view {
        AppView::Servers => {
            if !matches!(input.current_view, crate::ui::render::View::Servers(_)) {
                *input.current_view = crate::ui::render::View::Servers(
                    crate::ui::render::ServersViewState::default(),
                );
            }
        }
        AppView::Tools => {
            if !matches!(input.current_view, crate::ui::render::View::Tools(_)) {
                *input.current_view =
                    crate::ui::render::View::Tools(crate::ui::render::ToolsViewState::default());
            }
        }
        AppView::Help => {
            if !matches!(input.current_view, crate::ui::render::View::Help) {
                *input.current_view = crate::ui::render::View::Help;
            }
        }
    }
}

pub fn handle_common_navigation(input: &mut InputState, key_event: KeyEvent) -> bool {
    let keymap = &input.state.keymap;
    let pending = (*input.pending_g).then_some('g');

    if let KeyCode::Char('g') = key_event.code {
        if key_event.modifiers.is_empty() {
            if *input.pending_g {
                *input.pending_g = false;
                handle_go_to_first(input);
                return true;
            } else {
                *input.pending_g = true;
                return true;
            }
        }
    }

    if keymap.matches(crate::ui::keymap::KeyAction::GoLast, &key_event, pending) {
        *input.pending_g = false;
        handle_go_to_last(input);
        return true;
    }

    if keymap.matches(crate::ui::keymap::KeyAction::Down, &key_event, pending) {
        *input.pending_g = false;
        handle_navigation_down(input);
        return true;
    }
    if keymap.matches(crate::ui::keymap::KeyAction::Up, &key_event, pending) {
        *input.pending_g = false;
        handle_navigation_up(input);
        return true;
    }
    if keymap.matches(crate::ui::keymap::KeyAction::PageDown, &key_event, pending) {
        *input.pending_g = false;
        handle_page_down(input);
        return true;
    }
    if keymap.matches(crate::ui::keymap::KeyAction::PageUp, &key_event, pending) {
        *input.pending_g = false;
        handle_page_up(input);
        return true;
    }

    match key_event.code {
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc => {
            *input.pending_g = false;
        }
        _ => {}
    }

    false
}

fn handle_navigation_down(input: &mut InputState) {
    match (
        input.state.ui_state.current_view,
        input.state.ui_state.pane_focus,
    ) {
        (AppView::Servers, Pane::Cities) => input.state.city_select_next(),
        (AppView::Servers, Pane::Countries) => input.state.select_next(),
        (AppView::Tools, Pane::Settings) => input.state.settings_select_next(),
        (AppView::Tools, Pane::Logs) => input.state.logs_select_next(),
        _ => {}
    }
}

fn handle_navigation_up(input: &mut InputState) {
    *input.pending_g = false;
    match (
        input.state.ui_state.current_view,
        input.state.ui_state.pane_focus,
    ) {
        (AppView::Servers, Pane::Cities) => input.state.city_select_prev(),
        (AppView::Servers, Pane::Countries) => input.state.select_prev(),
        (AppView::Tools, Pane::Settings) => input.state.settings_select_prev(),
        (AppView::Tools, Pane::Logs) => input.state.logs_select_prev(),
        _ => {}
    }
}

fn handle_page_down(input: &mut InputState) {
    *input.pending_g = false;
    match (
        input.state.ui_state.current_view,
        input.state.ui_state.pane_focus,
    ) {
        (AppView::Servers, Pane::Cities) => input.state.city_select_page_down(),
        (AppView::Servers, Pane::Countries) => input.state.select_page_down(),
        (AppView::Tools, Pane::Settings) => input.state.settings_select_page_down(),
        (AppView::Tools, Pane::Logs) => input.state.logs_select_page_down(),
        _ => {}
    }
}

fn handle_page_up(input: &mut InputState) {
    *input.pending_g = false;
    match (
        input.state.ui_state.current_view,
        input.state.ui_state.pane_focus,
    ) {
        (AppView::Servers, Pane::Cities) => input.state.city_select_page_up(),
        (AppView::Servers, Pane::Countries) => input.state.select_page_up(),
        (AppView::Tools, Pane::Settings) => input.state.settings_select_page_up(),
        (AppView::Tools, Pane::Logs) => input.state.logs_select_page_up(),
        _ => {}
    }
}

fn handle_go_to_first(input: &mut InputState) {
    match (
        input.state.ui_state.current_view,
        input.state.ui_state.pane_focus,
    ) {
        (AppView::Servers, Pane::Cities) => input.state.city_select_first(),
        (AppView::Servers, Pane::Countries) => input.state.select_first(),
        (AppView::Tools, Pane::Settings) => input.state.settings_select_first(),
        (AppView::Tools, Pane::Logs) => input.state.logs_select_first(),
        _ => {}
    }
}

fn handle_go_to_last(input: &mut InputState) {
    *input.pending_g = false;
    match (
        input.state.ui_state.current_view,
        input.state.ui_state.pane_focus,
    ) {
        (AppView::Servers, Pane::Cities) => input.state.city_select_last(),
        (AppView::Servers, Pane::Countries) => input.state.select_last(),
        (AppView::Tools, Pane::Settings) => input.state.settings_select_last(),
        (AppView::Tools, Pane::Logs) => input.state.logs_select_last(),
        _ => {}
    }
}
