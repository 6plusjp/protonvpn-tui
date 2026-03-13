# issue039 - Key Bindings Runtime Migration

## Summary

Complete the migration to runtime-configurable key bindings by updating all key matching locations to use the `KeyBindings` infrastructure.

## Background

issue031 added the key bindings infrastructure:

- `KeyBindings` struct with all default bindings
- `KeyBinding` struct with `matches()` method
- `KeyModifier` enum (None, Control, Alt, Shift)
- `key_bindings` field in `AppState`

However, the runtime binding is not yet active - key matching still uses hardcoded patterns.

## Current State

All key handlers still use hardcoded `KeyCode::Char('x')` patterns:

```rust
// Current (hardcoded)
KeyCode::Char('c') => { ... }
KeyCode::Char('j') | KeyCode::Down => { ... }

// Desired (runtime-configurable)
if self.state.key_bindings.connect.matches(key_event.code, key_event.modifiers) { ... }
```

## Migration Plan

### Phase 1: Navigation Keys (Low Priority)

Update `handle_common_navigation()` in `src/ui/app.rs`:

```rust
fn handle_common_navigation(&mut self, key_event: crossterm::event::KeyEvent) -> bool {
    let bindings = &self.state.key_bindings;
    
    if bindings.navigation_down.matches(key_event.code, key_event.modifiers) {
        self.handle_navigation_down();
        return true;
    }
    // ... other bindings
    false
}
```

### Phase 2: Action Keys (Medium Priority)

Update connection-related keys in `handle_servers_key()`:

- `c` (connect)
- `d` (disconnect)
- `r` (refresh)
- `x` (random connect)

### Phase 3: View-Specific Keys (Low Priority)

Update remaining view handlers:

- Settings view: navigation, toggle, connect
- Logs view: navigation
- Servers view: pane switching (`h`, `l`), sort (`s`, `f`)

### Phase 4: Help View Display

Add key bindings display in help view (not settings):

```
Key Bindings:
  j/k       - Navigate
  c         - Connect
  d         - Disconnect
  ...
```

Note: Key bindings are configurable via config file, not UI.
Users can edit `~/.config/protonvpn-tui/keybindings.json` to customize.

## Files to Change

| Phase | File | Changes |
|-------|------|---------|
| 1 | `src/ui/app.rs` | Update `handle_common_navigation()` |
| 2 | `src/ui/app.rs` | Update action keys in `handle_servers_key()` |
| 3 | `src/ui/app.rs` | Update remaining handlers |
| 4 | `src/ui/views/help_view.rs` | Add bindings display + config note |

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| Low | Phase 1: Navigation keys | Low | ✅ Done |
| Medium | Phase 2: Action keys | Medium | ✅ Done |
| Low | Phase 3: View-specific keys | Medium | ✅ Done |
| Low | Phase 4: Settings UI | Low | ✅ Done |

## Dependencies

- issue031: Key bindings infrastructure (completed)

## Implementation Notes

### Completed (2026-03-13)

1. **Phase 1: Navigation keys** - Updated `handle_common_navigation()` to use `key_bindings`
2. **Phase 2: Action keys** - Updated connect, disconnect, refresh, random_connect in `handle_servers_key()`
3. **Phase 3: View-specific keys** - Added pane_next (l), pane_prev (h), cycle_sort (s), cycle_sort_field (f) bindings

### Key Bindings Added

- `navigation_down` (j), `navigation_up` (k)
- `page_down` (Ctrl+d), `page_up` (Ctrl+u)
- `go_first` (g), `go_last` (G)
- `connect` (c), `disconnect` (d)
- `refresh` (r), `random_connect` (x)
- `pane_next` (l), `pane_prev` (h)
- `cycle_sort` (s), `cycle_sort_field` (f)

## Notes

- The `KeyBinding::matches()` currently only supports `KeyCode::Char`. Need to extend for arrow keys, special keys if needed.
- Consider adding a "reset to defaults" option in settings.
- Key binding changes should persist across restarts (requires saving to config file).

## References

- issue031: Key Handling Improvements
