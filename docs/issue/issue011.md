# issue011: Cursor Selection Issues

## Summary

Two cursor/selection state issues affect user experience:
1. Cursor resets to top when navigating Servers -> Cities -> Servers
2. No item selected on application startup

## Issue 1: Cursor Resets on Cities -> Servers Navigation

### Symptom

1. User is on Servers view, selects a server in the middle of the list (e.g., index 10)
2. User presses Enter to view Cities for that server
3. User presses Esc to go back to Servers view
4. Cursor is now at the top (index 0) instead of staying at the original position

### Root Cause

In `src/ui/app.rs:300-306`, when handling Esc key to return from Cities to Servers:

```rust
KeyCode::Esc => {
    if self.state.current_view == AppView::Cities {
        self.state.current_view = AppView::Servers;
        self.state.current_cities.clear();
        self.state.current_country_code = None;
    }
    None
}
```

The code clears `current_cities` and `current_country_code` but does NOT reset `selected_server`. Since Cities list is shorter than Servers list, the stored index may be:
- Out of bounds for the Servers list
- Or point to a different server than originally selected

### Fix

Add `self.state.selected_server = Some(0);` when returning to Servers view, OR store and restore the original selection.

### Related Code

- `src/ui/app.rs:300-306` - Esc key handler
- `src/state/app_state.rs:145` - `selected_server` field

---

## Issue 2: No Initial Selection on Startup

### Symptom

When the application starts, no server is selected. User must press j/k to select the first server before they can connect.

### Root Cause

In `src/state/app_state.rs:182`:

```rust
selected_server: None,
```

`selected_server` initializes to `None`, meaning no item is selected on startup.

### Fix

Change to:

```rust
selected_server: Some(0),
```

This will automatically select the first server in the list on startup.

### Related Code

- `src/state/app_state.rs:182` - Initialization in `AppState::new()`

---

## Tags

- bug
- cursor
- selection
- navigation
- ui
