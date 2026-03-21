use crossterm::event::{KeyCode, KeyEvent};

use super::app_action::AppAction;
use super::InputState;
use crate::state::InputMode;

pub fn handle_filter_input(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    let is_dns_input = input.state.ui_state.input_mode == InputMode::DnsInput;

    match key_event.code {
        KeyCode::Esc => {
            *input.filter_mode = false;
            input.filter_input.clear();
            input.state.set_search_query(String::new());
            if is_dns_input {
                input.state.ui_state.input_mode = InputMode::Normal;
                input.state.ui_state.dns_input.clear();
            }
            None
        }
        KeyCode::Enter => {
            if is_dns_input {
                let dns_ips = input.state.ui_state.dns_input.to_string();
                input.state.ui_state.dns_input.clear();
                input.state.ui_state.input_mode = InputMode::Normal;
                if !dns_ips.is_empty() {
                    input.state.apply_dns_setting(&dns_ips);
                }
            } else {
                *input.filter_mode = false;
            }
            None
        }
        KeyCode::Backspace => {
            if is_dns_input {
                input.state.ui_state.dns_input.pop();
            } else {
                input.filter_input.pop();
                input.state.set_search_query(input.filter_input.clone());
            }
            None
        }
        KeyCode::Char(c) => {
            if is_dns_input {
                input.state.ui_state.dns_input.push(c);
            } else {
                input.filter_input.push(c);
                input.state.set_search_query(input.filter_input.clone());
            }
            None
        }
        _ => None,
    }
}
