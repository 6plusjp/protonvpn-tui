# Issue 053: Systematize Keybinding Handling

## Status: Open

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
// src/ui/keymap.rs (new file)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Single key matcher
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyMatcher {
    /// Plain character without modifiers (e.g., 'j', 'k')
    Char(char),
    /// Character with modifiers (e.g., Ctrl+'d', Alt+'j')
    CharWithMod(char, KeyModifiers),
    /// Arrow key
    Arrow(KeyArrow),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyArrow {
    Up,
    Down,
    Left,
    Right,
}

impl KeyMatcher {
    pub fn matches(&self, event: &KeyEvent) -> bool {
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
                event.code == code
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
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            // Vim-style + Arrow keys
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
            // ... other actions
        }
    }
}

impl KeyMap {
    pub fn matches(&self, action: KeyAction, event: &KeyEvent) -> bool {
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
        };
        matchers.iter().any(|m| m.matches(event))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}
```

### Simplified app.rs

```rust
fn handle_common_navigation(&mut self, key_event: KeyEvent) -> bool {
    let keymap = &self.state.keymap;
    
    if keymap.matches(KeyAction::Down, &key_event) {
        self.handle_navigation_down();
        return true;
    }
    if keymap.matches(KeyAction::Up, &key_event) {
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

### Phase 1: Create KeyMap
- Create `src/ui/keymap.rs`
- Define `KeyMatcher`, `KeyMap`, `KeyAction`
- Implement `matches()` method

### Phase 2: Integration
- Add `keymap: KeyMap` field to `AppState`
- Refactor `handle_common_navigation()` to use KeyMap
- Refactor settings pane handlers to use KeyMap

### Phase 3: Config File Support (optional)
- Support generating KeyMap from TOML config

---

## Related Issues

- #053: Systematize Keybinding Handling (this issue)
- #007: Ctrl+n/Ctrl+p Navigation Support (resolved)

---

## Labels

enhancement, refactoring, usability
