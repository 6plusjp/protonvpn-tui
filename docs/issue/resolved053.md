# Issue 053: Systematize Keybinding Handling

## Status: Phase 3 Complete

## Created: 2026-03-18

## Summary

Centralize keybinding handling with unified structures to improve extensibility and maintainability.

---

## Background

Currently (`v0.1.0`), keybinding handling has the following issues:

### 1. Duplicate Definitions

Navigation keys (`j/k`, `Up/Down`, `Ctrl+n/Ctrl+p`) are scattered across multiple locations:

```
src/ui/app.rs:
├── handle_common_navigation()  # j/k via key_bindings
├── handle_settings_pane_key()  # j/k/Up/Down/n/p individually defined
│   ├── (false, ...) mode       # j/k/Down/n only (guard: !Ctrl)
│   └── (true, ...) mode        # j/k/Down/n only (guard: !Ctrl) + separate Ctrl+n/p
```

### 2. Poor Extensibility

| New Keybinding | Edit Locations Required |
|---------------|----------------------|
| Add `Ctrl+n` | 3 locations |
| Add `Alt+j` | 6+ locations |
| New action | Complete rewrite |

### 3. KeyBinding Structure Limitations

```rust
// Current: 1 action = 1 key
pub struct KeyBinding {
    pub code: char,
    pub modifiers: KeyModifier,
}
```

Example: Cannot assign `j`, `Down`, and `Ctrl+n` all to the "Down" action.

---

## Proposed Solution

### Introduce KeyMap Structure

```rust
// src/ui/keymap.rs (implemented)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Arrow key direction
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyArrow {
    Up,
    Down,
    Left,
    Right,
}

/// Single key matcher
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
    pub fn matches(&self, event: &KeyEvent, pending: Option<char>) -> bool {
        match self {
            KeyMatcher::Char(c) => {
                event.code == KeyCode::Char(*c) && event.modifiers.is_empty()
            }
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

/// Action-centric keymap
#[derive(Clone)]
pub struct KeyMap {
    pub down: Vec<KeyMatcher>,
    pub up: Vec<KeyMatcher>,
    pub page_down: Vec<KeyMatcher>,
    pub page_up: Vec<KeyMatcher>,
    pub go_first: Vec<KeyMatcher>,    // DoubleChar('g') = 'gg'
    pub go_last: Vec<KeyMatcher>,     // Char('G')
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
            page_down: vec![
                KeyMatcher::CharWithMod('d', KeyModifiers::CONTROL),
                KeyMatcher::CharWithMod('f', KeyModifiers::CONTROL),
            ],
            page_up: vec![
                KeyMatcher::CharWithMod('u', KeyModifiers::CONTROL),
                KeyMatcher::CharWithMod('b', KeyModifiers::CONTROL),
            ],
            // 'gg' = go first, 'G' = go last
            go_first: vec![KeyMatcher::DoubleChar('g')],
            go_last: vec![KeyMatcher::CharWithMod('G', KeyModifiers::SHIFT)],
            // ... other actions
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
        };
        matchers.iter().any(|m| m.matches(event, pending))
    }
}

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
}
```

### Config-Level KeyMatcherConfig

For TOML configuration, `KeyMatcherConfig` serializes as:

```rust
// src/config/user_config.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum KeyMatcherConfig {
    Char(char),
    CharWithMod { code: char, modifiers: Vec<String> },
    Arrow(String),
    DoubleChar(char),
}
```

Example TOML config:
```toml
[keybindings]
navigation_down = [
    { type = "Char", value = "j" },
    { type = "Arrow", value = "Down" },
    { type = "CharWithMod", value = { code = "n", modifiers = ["Control"] } }
]
navigation_up = [
    { type = "Char", value = "k" },
    { type = "Arrow", value = "Up" },
    { type = "CharWithMod", value = { code = "p", modifiers = ["Control"] } }
]
go_first = [{ type = "DoubleChar", value = "g" }]
go_last = [{ type = "CharWithMod", value = { code = "G", modifiers = ["Shift"] } }]
```

### Simplified app.rs

```rust
fn handle_common_navigation(&mut self, key_event: KeyEvent) -> bool {
    let keymap = &self.state.keymap;
    let pending = self.pending_g.then_some('g');

    // Handle 'gg' double-key sequence
    if let KeyCode::Char('g') = key_event.code {
        if key_event.modifiers.is_empty() {
            if self.pending_g {
                self.pending_g = false;
                self.handle_go_to_first();
                return true;
            } else {
                self.pending_g = true;
                return true;
            }
        }
    }

    // Handle other navigation actions
    if keymap.matches(KeyAction::Down, &key_event, pending) {
        self.pending_g = false;
        self.handle_navigation_down();
        return true;
    }
    if keymap.matches(KeyAction::Up, &key_event, pending) {
        self.pending_g = false;
        self.handle_navigation_up();
        return true;
    }
    // ... other actions

    false
}
```

---

## Benefits

| Item | Before | After |
|------|--------|-------|
| Navigation definition locations | 6 match blocks | 1 KeyMap |
| Adding keybindings | Edit multiple locations | Add 1 line to KeyMap |
| Lines of code (est.) | ~100 lines | ~80 lines |
| Testability | Difficult | Unit-testable per KeyMatcher |
| New key overhead | O(n) | O(1) |

---

## Implementation Notes

### Phase 1: Create KeyMap ✅ COMPLETE
- Created `src/ui/keymap.rs`
- Defined `KeyMatcher`, `KeyMap`, `KeyAction`
- Implemented `matches()` methods
- Added `DoubleChar` variant for multi-key sequences (e.g., `gg`)
- Added comprehensive unit tests (7 tests passing)

### Phase 2: Integration ✅ COMPLETE
- Added `keymap: KeyMap` field to `AppState`
- Added `KeyAction` import to `app.rs`
- Refactored `handle_common_navigation()` to use KeyMap
- Implemented `gg` double-key sequence handling via `pending_g` state
- Fixed `go_last` to use `CharWithMod('G', SHIFT)` for proper Shift key matching
- All tests passing (53 tests)

### Phase 3: Config File Support ✅ COMPLETE
- Added `KeyMatcherConfig` enum with variants: `Char`, `CharWithMod`, `Arrow`, `DoubleChar`
- Updated `KeyBindingsConfig` to use `Vec<KeyMatcherConfig>` per action
- Implemented `From<KeyBindingsConfig> for KeyMap` for config-to-KeyMap conversion
- Added `to_keymatcher()` method to convert config to runtime `KeyMatcher`
- Config format now supports multiple keys per action
- Config file schema updated to support new format (see example above)

---

## Related Issues

- #053: Systematize Keybinding Handling (this issue)
- #007: Ctrl+n/Ctrl+p Navigation Support (resolved)

---

## Labels

enhancement, refactoring, usability
