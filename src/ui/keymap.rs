//! Keybinding handling with unified structures
//!
//! This module centralizes keybinding definitions to improve extensibility and maintainability.
//! Instead of scattering navigation keys across multiple handlers, all keybindings are defined
//! in a single [`KeyMap`] structure.
//!
//! **Single Source of Truth**: Default keybindings are defined here and imported by
//! `config/settings.rs` and `config/user_config.rs` to avoid duplication.

use crate::config::{KeyBinding, KeyBindings, KeyModifier};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Arrow key direction
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyArrow {
    Up,
    Down,
    Left,
    Right,
}

/// Single key matcher - represents a single key or key+modifier combination
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyMatcher {
    /// Plain character without modifiers (e.g., 'j', 'k')
    Char(char),
    /// Character with modifiers (e.g., Ctrl+'d', Alt+'j')
    CharWithMod(char, KeyModifiers),
    /// Arrow key
    Arrow(KeyArrow),
    /// Double character sequence (e.g., 'gg' for go first)
    DoubleChar(char),
}

impl KeyMatcher {
    /// Check if this matcher matches the given key event
    /// `pending` is the previous key pressed for double-char sequences
    #[must_use]
    pub fn matches(&self, event: &KeyEvent, pending: Option<char>) -> bool {
        match self {
            KeyMatcher::Char(c) => event.code == KeyCode::Char(*c) && event.modifiers.is_empty(),
            KeyMatcher::CharWithMod(c, mods) => {
                event.code == KeyCode::Char(*c) && event.modifiers.contains(*mods)
            }
            KeyMatcher::Arrow(dir) => {
                let code = match dir {
                    KeyArrow::Up => KeyCode::Up,
                    KeyArrow::Down => KeyCode::Down,
                    KeyArrow::Left => KeyCode::Left,
                    KeyArrow::Right => KeyCode::Right,
                };
                event.code == code && event.modifiers.is_empty()
            }
            KeyMatcher::DoubleChar(c) => {
                if let Some(p) = pending {
                    event.code == KeyCode::Char(*c) && event.modifiers.is_empty() && p == *c
                } else {
                    false
                }
            }
        }
    }
}

/// All possible key actions in the application
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyAction {
    Down,
    Up,
    PageDown,
    PageUp,
    GoFirst,
    GoLast,
    Connect,
    Disconnect,
    Refresh,
    RandomConnect,
    PaneNext,
    PanePrev,
    SortByCode,
    SortByCountry,
    ConnectFastest,
    ConnectP2p,
    ConnectTor,
    Securecore,
    Search,
    Cancel,
    Help,
    Quit,
    NextSetting,
    PrevSetting,
    ToggleSetting,
    SelectCity,
    RefreshCities,
    ToggleFavorite,
}

/// Action-centric keymap - maps actions to their possible keybindings
#[derive(Clone, Debug)]
pub struct KeyMap {
    pub down: Vec<KeyMatcher>,
    pub up: Vec<KeyMatcher>,
    pub page_down: Vec<KeyMatcher>,
    pub page_up: Vec<KeyMatcher>,
    pub go_first: Vec<KeyMatcher>,
    pub go_last: Vec<KeyMatcher>,
    pub connect: Vec<KeyMatcher>,
    pub disconnect: Vec<KeyMatcher>,
    pub refresh: Vec<KeyMatcher>,
    pub random_connect: Vec<KeyMatcher>,
    pub pane_next: Vec<KeyMatcher>,
    pub pane_prev: Vec<KeyMatcher>,
    pub sort_by_code: Vec<KeyMatcher>,
    pub sort_by_country: Vec<KeyMatcher>,
    pub connect_fastest: Vec<KeyMatcher>,
    pub connect_p2p: Vec<KeyMatcher>,
    pub connect_tor: Vec<KeyMatcher>,
    pub securecore: Vec<KeyMatcher>,
    pub search: Vec<KeyMatcher>,
    pub cancel: Vec<KeyMatcher>,
    pub help: Vec<KeyMatcher>,
    pub quit: Vec<KeyMatcher>,
    pub next_setting: Vec<KeyMatcher>,
    pub prev_setting: Vec<KeyMatcher>,
    pub toggle_setting: Vec<KeyMatcher>,
    pub select_city: Vec<KeyMatcher>,
    pub refresh_cities: Vec<KeyMatcher>,
    pub toggle_favorite: Vec<KeyMatcher>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            // Navigation: Vim-style + Arrow keys + Ctrl+n/Ctrl+p
            down: vec![
                KeyMatcher::Char('j'),
                KeyMatcher::Arrow(KeyArrow::Down),
                KeyMatcher::CharWithMod('n', KeyModifiers::CONTROL),
            ],
            up: vec![
                KeyMatcher::Char('k'),
                KeyMatcher::Arrow(KeyArrow::Up),
                KeyMatcher::CharWithMod('p', KeyModifiers::CONTROL),
            ],
            page_down: vec![KeyMatcher::CharWithMod('d', KeyModifiers::CONTROL)],
            page_up: vec![KeyMatcher::CharWithMod('u', KeyModifiers::CONTROL)],
            go_first: vec![KeyMatcher::DoubleChar('g')],
            go_last: vec![KeyMatcher::CharWithMod('G', KeyModifiers::SHIFT)],
            // Connection actions
            connect: vec![
                KeyMatcher::Char('c'),
                KeyMatcher::CharWithMod('c', KeyModifiers::CONTROL),
            ],
            disconnect: vec![KeyMatcher::Char('d')],
            refresh: vec![KeyMatcher::Char('r')],
            random_connect: vec![KeyMatcher::Char('x')],
            connect_fastest: vec![KeyMatcher::Char('f')],
            connect_p2p: vec![KeyMatcher::Char('p')],
            connect_tor: vec![KeyMatcher::Char('t')],
            securecore: vec![KeyMatcher::Char('s')],
            // Pane navigation
            pane_next: vec![
                KeyMatcher::Char('l'),
                KeyMatcher::Arrow(KeyArrow::Right),
                KeyMatcher::CharWithMod('l', KeyModifiers::CONTROL),
            ],
            pane_prev: vec![
                KeyMatcher::Char('h'),
                KeyMatcher::Arrow(KeyArrow::Left),
                KeyMatcher::CharWithMod('h', KeyModifiers::CONTROL),
            ],
            // Sorting
            sort_by_code: vec![KeyMatcher::Char('1')],
            sort_by_country: vec![KeyMatcher::Char('2')],
            // UI actions
            search: vec![KeyMatcher::Char('/')],
            cancel: vec![
                KeyMatcher::CharWithMod('[', KeyModifiers::CONTROL),
                KeyMatcher::CharWithMod('c', KeyModifiers::CONTROL),
                KeyMatcher::Char('q'),
            ],
            help: vec![KeyMatcher::Char('?')],
            quit: vec![KeyMatcher::Char('q')],
            // Settings pane navigation
            next_setting: vec![KeyMatcher::Char('j'), KeyMatcher::Arrow(KeyArrow::Down)],
            prev_setting: vec![KeyMatcher::Char('k'), KeyMatcher::Arrow(KeyArrow::Up)],
            toggle_setting: vec![
                KeyMatcher::Char(' '),
                KeyMatcher::Char('\n'),
                KeyMatcher::CharWithMod('m', KeyModifiers::CONTROL),
            ],
            // Cities pane
            select_city: vec![
                KeyMatcher::Char('\n'),
                KeyMatcher::CharWithMod('o', KeyModifiers::CONTROL),
            ],
            refresh_cities: vec![KeyMatcher::CharWithMod('r', KeyModifiers::CONTROL)],
            toggle_favorite: vec![KeyMatcher::CharWithMod('f', KeyModifiers::CONTROL)],
        }
    }
}

impl KeyMap {
    pub fn matches(&self, action: KeyAction, event: &KeyEvent, pending: Option<char>) -> bool {
        let matchers = match action {
            KeyAction::Down => &self.down,
            KeyAction::Up => &self.up,
            KeyAction::PageDown => &self.page_down,
            KeyAction::PageUp => &self.page_up,
            KeyAction::GoFirst => &self.go_first,
            KeyAction::GoLast => &self.go_last,
            KeyAction::Connect => &self.connect,
            KeyAction::Disconnect => &self.disconnect,
            KeyAction::Refresh => &self.refresh,
            KeyAction::RandomConnect => &self.random_connect,
            KeyAction::PaneNext => &self.pane_next,
            KeyAction::PanePrev => &self.pane_prev,
            KeyAction::SortByCode => &self.sort_by_code,
            KeyAction::SortByCountry => &self.sort_by_country,
            KeyAction::ConnectFastest => &self.connect_fastest,
            KeyAction::ConnectP2p => &self.connect_p2p,
            KeyAction::ConnectTor => &self.connect_tor,
            KeyAction::Securecore => &self.securecore,
            KeyAction::Search => &self.search,
            KeyAction::Cancel => &self.cancel,
            KeyAction::Help => &self.help,
            KeyAction::Quit => &self.quit,
            KeyAction::NextSetting => &self.next_setting,
            KeyAction::PrevSetting => &self.prev_setting,
            KeyAction::ToggleSetting => &self.toggle_setting,
            KeyAction::SelectCity => &self.select_city,
            KeyAction::RefreshCities => &self.refresh_cities,
            KeyAction::ToggleFavorite => &self.toggle_favorite,
        };
        matchers.iter().any(|m| m.matches(event, pending))
    }
}

pub fn default_keybindings() -> KeyBindings {
    KeyBindings {
        navigation_down: KeyBinding::new('j', KeyModifier::None),
        navigation_up: KeyBinding::new('k', KeyModifier::None),
        page_down: KeyBinding::new('d', KeyModifier::Control),
        page_up: KeyBinding::new('u', KeyModifier::Control),
        go_first: KeyBinding::new('g', KeyModifier::None),
        go_last: KeyBinding::new('G', KeyModifier::Shift),
        connect: KeyBinding::new('c', KeyModifier::None),
        disconnect: KeyBinding::new('d', KeyModifier::None),
        refresh: KeyBinding::new('r', KeyModifier::None),
        random_connect: KeyBinding::new('x', KeyModifier::None),
        pane_next: KeyBinding::new('l', KeyModifier::None),
        pane_prev: KeyBinding::new('h', KeyModifier::None),
        sort_by_code: KeyBinding::new('1', KeyModifier::None),
        sort_by_country: KeyBinding::new('2', KeyModifier::None),
        connect_fastest: KeyBinding::new('f', KeyModifier::None),
        connect_p2p: KeyBinding::new('p', KeyModifier::None),
        connect_tor: KeyBinding::new('t', KeyModifier::None),
        securecore: KeyBinding::new('s', KeyModifier::None),
        toggle_favorite: KeyBinding::new('f', KeyModifier::Control),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_key_matcher_char() {
        let matcher = KeyMatcher::Char('j');
        assert!(matcher.matches(&make_event(KeyCode::Char('j'), KeyModifiers::empty()), None));
        assert!(!matcher.matches(&make_event(KeyCode::Char('k'), KeyModifiers::empty()), None));
        assert!(!matcher.matches(&make_event(KeyCode::Char('j'), KeyModifiers::CONTROL), None));
    }

    #[test]
    fn test_key_matcher_char_with_mod() {
        let matcher = KeyMatcher::CharWithMod('n', KeyModifiers::CONTROL);
        assert!(matcher.matches(&make_event(KeyCode::Char('n'), KeyModifiers::CONTROL), None));
        assert!(!matcher.matches(&make_event(KeyCode::Char('n'), KeyModifiers::empty()), None));
        assert!(!matcher.matches(&make_event(KeyCode::Char('p'), KeyModifiers::CONTROL), None));
    }

    #[test]
    fn test_key_matcher_arrow() {
        let matcher = KeyMatcher::Arrow(KeyArrow::Down);
        assert!(matcher.matches(&make_event(KeyCode::Down, KeyModifiers::empty()), None));
        assert!(!matcher.matches(&make_event(KeyCode::Up, KeyModifiers::empty()), None));
        assert!(!matcher.matches(&make_event(KeyCode::Down, KeyModifiers::CONTROL), None));
    }

    #[test]
    fn test_keymap_default_navigation() {
        let keymap = KeyMap::default();

        // Down: 'j', Down arrow, Ctrl+n
        assert!(keymap.matches(
            KeyAction::Down,
            &make_event(KeyCode::Char('j'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::Down,
            &make_event(KeyCode::Down, KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::Down,
            &make_event(KeyCode::Char('n'), KeyModifiers::CONTROL),
            None
        ));

        // Up: 'k', Up arrow, Ctrl+p
        assert!(keymap.matches(
            KeyAction::Up,
            &make_event(KeyCode::Char('k'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::Up,
            &make_event(KeyCode::Up, KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::Up,
            &make_event(KeyCode::Char('p'), KeyModifiers::CONTROL),
            None
        ));

        // Go last: 'G' (Shift+g)
        assert!(keymap.matches(
            KeyAction::GoLast,
            &make_event(KeyCode::Char('G'), KeyModifiers::SHIFT),
            None
        ));
    }

    #[test]
    fn test_key_matcher_double_char() {
        let matcher = KeyMatcher::DoubleChar('g');

        // 'gg' matches when pending is 'g'
        assert!(matcher.matches(
            &make_event(KeyCode::Char('g'), KeyModifiers::empty()),
            Some('g')
        ));

        // 'g' alone does not match
        assert!(!matcher.matches(&make_event(KeyCode::Char('g'), KeyModifiers::empty()), None));

        // 'gh' does not match (pending is 'g' but key is 'h')
        assert!(!matcher.matches(
            &make_event(KeyCode::Char('h'), KeyModifiers::empty()),
            Some('g')
        ));
    }

    #[test]
    fn test_keymap_connection() {
        let keymap = KeyMap::default();

        assert!(keymap.matches(
            KeyAction::Connect,
            &make_event(KeyCode::Char('c'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::Disconnect,
            &make_event(KeyCode::Char('d'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::Refresh,
            &make_event(KeyCode::Char('r'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::RandomConnect,
            &make_event(KeyCode::Char('x'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::ConnectFastest,
            &make_event(KeyCode::Char('f'), KeyModifiers::empty()),
            None
        ));
    }

    #[test]
    fn test_keymap_pane_navigation() {
        let keymap = KeyMap::default();

        // Pane next: 'l', Right arrow
        assert!(keymap.matches(
            KeyAction::PaneNext,
            &make_event(KeyCode::Char('l'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::PaneNext,
            &make_event(KeyCode::Right, KeyModifiers::empty()),
            None
        ));

        // Pane prev: 'h', Left arrow
        assert!(keymap.matches(
            KeyAction::PanePrev,
            &make_event(KeyCode::Char('h'), KeyModifiers::empty()),
            None
        ));
        assert!(keymap.matches(
            KeyAction::PanePrev,
            &make_event(KeyCode::Left, KeyModifiers::empty()),
            None
        ));
    }
}
