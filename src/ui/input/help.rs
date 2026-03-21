use crossterm::event::{KeyCode, KeyEvent};

use super::app_action::AppAction;
use super::InputState;

pub fn handle_help_key(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    match key_event.code {
        KeyCode::Esc => {
            input.state.ui_state.current_view = input.state.ui_state.previous_view;
            None
        }
        _ => None,
    }
}
