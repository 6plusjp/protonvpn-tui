# issue031 - Key Handling Improvements

## Summary

Key handling implementation has several issues: potential input loss, lack of state validation, code duplication, and missing configurability.

## Problems Identified

### 1. Potential Key Input Loss (Medium Priority)

**Location**: `src/ui/app.rs:88-108`

```rust
if event::poll(Duration::from_millis(100))? {
    if let Event::Key(key_event) = event::read()? {
        // ...
    }
}
```

**Issue**: `event::read()` can block after `event::poll()` returns true. In rare cases with concurrent input sources, key events could be missed.

**Note**: This is a theoretical issue with crossterm's design. In practice, it's unlikely to cause problems.

### 2. No State Validation During Connection (High Priority)

**Location**: `src/ui/app.rs:187-264`

```rust
fn handle_servers_key(&mut self, key_event: ...) -> Option<AppAction> {
    match key_event.code {
        KeyCode::Char('c') => {
            self.handle_connect();  // No check for current state!
            None
        }
        // ...
    }
}
```

**Issue**: Users can trigger connection actions even when:
- Already connecting
- Already connected
- Disconnecting in progress

This can cause race conditions or confusing UI states.

**Example**:
```
User presses 'c' → Connection starts
User presses 'c' again quickly → Another connection attempt
User presses 'd' → Disconnect
```

### 3. Code Duplication (Low Priority)

**Location**: `src/ui/app.rs:187-264`, `266-393`, `395-423`

Same key patterns repeated across handlers:

```rust
KeyCode::Char('j') | KeyCode::Down => self.handle_navigation_down(),
KeyCode::Char('k') | KeyCode::Up   => self.handle_navigation_up(),
KeyCode::Char('d') if Ctrl         => self.handle_page_down(),
KeyCode::Char('g')                 => self.handle_go_to_first(),
KeyCode::Char('G')                 => self.handle_go_to_last(),
KeyCode::Char('u') if Ctrl         => self.handle_page_up(),
```

Could be extracted into a common handler or trait.

### 4. Hardcoded Key Bindings (Low Priority)

**Issue**: Key bindings are hardcoded in match statements. Users cannot:
- Customize key bindings
- See all available keys in settings
- Add new shortcuts

## Proposed Solutions

### Solution 1: State-Aware Key Handling

Add connection state checks before actions:

```rust
KeyCode::Char('c') => {
    match self.state.get_connection() {
        ConnectionState::Connected => {
            self.state.show_notification("Already connected", NotificationType::Info);
        }
        ConnectionState::Connecting => {
            self.state.show_notification("Connection in progress...", NotificationType::Warning);
        }
        ConnectionState::Disconnecting => {
            self.state.show_notification("Disconnecting...", NotificationType::Warning);
        }
        ConnectionState::Disconnected | ConnectionState::Error(_) => {
            self.handle_connect();
        }
    }
    None
}
 checks needed for:
- `d` (disconnect) - check if already disconnected```

Similar
- `r` (refresh) - prevent concurrent refreshes
- `x` (random connect) - check connection state

### Solution 2: Extract Common Key Handler

Create a trait or helper:

```rust
trait NavigationHandler {
    fn handle_navigation(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Option<AppAction> {
        match (key, modifiers) {
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => self.handle_navigation_down(),
            (KeyCode::Char('k'), _) | (KeyCode::Up, _)   => self.handle_navigation_up(),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.handle_page_down(),
            (KeyCode::Char('g'), _) => self.handle_go_to_first(),
            (KeyCode::Char('G'), _) => self.handle_go_to_last(),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.handle_page_up(),
            _ => None,
        }
    }
}
```

### Solution 3: Configurable Key Bindings

Load key bindings from config:

```rust
// config/settings.rs
struct KeyBindings {
    connect: KeyEvent,
    disconnect: KeyEvent,
    next: KeyEvent,
    previous: KeyEvent,
    // ...
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            connect: KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            disconnect: KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            // ...
        }
    }
}
```

## Priority

| Priority | Item | Effort |
|----------|------|--------|
| High | State-aware key handling | Low |
| Low | Extract common navigation handler | Low |
| Low | Configurable key bindings | Medium |

## References

- Related issue: issue030 (startup performance)
- crossterm key event documentation
