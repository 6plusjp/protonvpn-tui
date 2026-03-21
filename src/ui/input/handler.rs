use crossterm::event::KeyEvent;

use super::app_action::AppAction;
use super::common::handle_common_keys;
use super::filter::handle_filter_input;
use super::help::handle_help_key;
use super::servers::handle_servers_key;
use super::tools::handle_tools_key;
use super::InputState;
use crate::state::{AppView, InputMode};

pub fn handle_key(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    if *input.filter_mode || input.state.ui_state.input_mode == InputMode::DnsInput {
        return handle_filter_input(input, key_event);
    }

    if let Some(action) = handle_common_keys(input, key_event) {
        return Some(action);
    }

    match input.state.ui_state.current_view {
        AppView::Servers => handle_servers_key(input, key_event),
        AppView::Tools => handle_tools_key(input, key_event),
        AppView::Help => handle_help_key(input, key_event),
    }
}
