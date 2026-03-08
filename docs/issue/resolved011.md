# resolved_issue011: Cursor Selection Issues

## Summary

Two cursor/selection state issues affected user experience:
1. Cursor resets to top when navigating Servers -> Cities -> Servers
2. No item selected on application startup

## Status: RESOLVED ✅

Fixed in commit `a15d01b`

## Issue 1: Cursor Resets on Cities -> Servers Navigation

### Original Symptom

1. User is on Servers view, selects a server in the middle of the list (e.g., index 10)
2. User presses Enter to view Cities for that server
3. User presses Esc to go back to Servers view
4. Cursor is now at the top (index 0) instead of staying at the original position

### Root Cause

In `src/ui/app.rs:300-306`, when handling Esc key to return from Cities to Servers, the code clears `current_cities` and `current_country_code` but does NOT reset `selected_server`.

### Fix Applied

Added `self.state.selected_server = Some(0);` in the Esc key handler:

```rust
KeyCode::Esc => {
    if self.state.current_view == AppView::Cities {
        self.state.current_view = AppView::Servers;
        self.state.current_cities.clear();
        self.state.current_country_code = None;
        self.state.selected_server = Some(0);  // ← Added
    }
    None
}
```

---

## Issue 2: No Initial Selection on Startup

### Original Symptom

When the application starts, no server is selected. User must press j/k to select the first server before they can connect.

### Root Cause

`selected_server` and `settings_selected` initialized to `None`.

### Fix Applied

Changed initialization in `src/state/app_state.rs`:

```rust
// Before
selected_server: None,
settings_selected: None,

// After
selected_server: Some(0),
settings_selected: Some(0),
```

Now both Servers and Settings views select the first item on startup.

---

## Related Commits

- `a15d01b` - fix(ui): select first item on startup for servers and settings

## Tags

- bug
- cursor
- selection
- navigation
- ui
- resolved
