# issue017: Split Large Functions - handle_key() and render()

## Status: RESOLVED

## Priority: Low

## Description

Split large functions in `src/ui/app.rs` to improve code readability and maintainability.

## Background

During issue016 code review, it was noted that `handle_key()` and `render()` functions in `src/ui/app.rs` are too large:
- `handle_key()` - 270+ lines
- `render()` - 60+ lines

This was marked as "Low Priority" / "backlog" in issue016 due to the large refactoring effort required.

## Current State Analysis

### AppView Structure
```rust
enum AppView {
    Servers,   // Default - split pane (Countries + Cities)
    Settings,
    Logs,
    Help,      // Only accessible via ?
}

enum Pane {
    Countries, // Left pane - country list
    Cities,    // Right pane - city list
}
```

### Code Metrics (Before)

| Function | Lines | Issues |
|----------|-------|--------|
| `handle_key()` | ~275 (117-392) | Giant match statement, repeated view/pane branching |
| `render()` | ~54 (446-500) | Layout logic embedded |
| `render_footer()` | ~85 (713-797) | Long action hints generation |

---

## Detailed Refactoring Plan

### Phase 1: handle_key() Refactoring

#### Step 1: Add View-based Dispatch (New)
Create a dispatcher that routes to view-specific handlers.

```rust
fn handle_key(&mut self, key_event: KeyEvent) -> Option<AppAction> {
    // First handle common keys
    if let Some(action) = self.handle_common_keys(key_event) {
        return action;
    }
    
    // Dispatch to view-specific handler
    match self.state.get_current_view() {
        AppView::Servers => self.handle_servers_key(key_event),
        AppView::Settings => self.handle_settings_key(key_event),
        AppView::Logs => self.handle_logs_key(key_event),
        AppView::Help => self.handle_help_key(key_event),
    }
}
```

#### Step 2: Extract Common Keys Handler (New)
```rust
fn handle_common_keys(&mut self, key_event: KeyEvent) -> Option<AppAction>
```
**Handles:**
- `q` → Quit
- `Tab` → SwitchView
- `/` → Enter filter mode
- `?` → Show help
- `Esc` → Clear filter
- `Ctrl+C` → Quit (handled in run() loop)

#### Step 3: Extract View-specific Handlers (New)

| Method | Responsibility |
|--------|----------------|
| `handle_servers_key()` | c(connect), l(to cities), h/Backspace(to countries), Enter |
| `handle_settings_key()` | Space(toggle off), Enter(toggle/input) |
| `handle_logs_key()` | j/k (scroll) |
| `handle_help_key()` | (no special keys, just displays help) |

#### Step 4: Extract Navigation Helpers (New)
```rust
fn handle_navigation_down(&mut self)   // j, ↓
fn handle_navigation_up(&mut self)     // k, ↑
fn handle_page_down(&mut self)         // Ctrl+d, G
fn handle_page_up(&mut self)           // Ctrl+u
fn handle_go_to_first(&mut self)       // gg
fn handle_go_to_last(&mut self)         // G
```

**Note:** Each navigation method should check current view/pane internally.

#### Step 5: Extract Connection Actions (New)
```rust
fn handle_connect(&mut self)           // c
fn handle_disconnect(&mut self)        // d
fn handle_connect_random(&mut self)    // x
fn handle_refresh(&mut self)           // r
fn handle_cycle_sort(&mut self)        // s
fn handle_cycle_sort_field(&mut self) // f
```

#### Step 6: Keep Existing filter_input Handler
`handle_filter_input()` (lines 394-444) handles filter/DNS input mode - no changes needed.

---

### Phase 2: render() Refactoring

#### Current Structure
```
render()
├── render_header()        # Already exists (good)
├── render_main()          # Already exists (good)
├── render_footer()        # Action hints too long (~85 lines)
├── render_filter_input()  # Already exists (good)
├── render_dns_input()     # Already exists (good)
└── render_notification_popup() # Already exists (good)
```

#### Step: Extract Footer Action Hints (New)
```rust
fn get_footer_action_hints(&self) -> Vec<Span<'_>>
```
Move the long `Vec<Span>` generation to this helper method.

**Current hints by view:**
- `AppView::Servers` (Countries pane): j/k, l/Enter, c, d, r, s, f, /
- `AppView::Servers` (Cities pane): j/k, c/Enter, h/Backspace
- `AppView::Settings`: j/k, Enter, Space
- `AppView::Logs`: j/k
- `AppView::Help`: "Tab or q to return"

---

## Expected Improvements

| Metric | Before | After |
|--------|--------|-------|
| `handle_key()` | ~275 lines | ~80 lines + handlers |
| `render_footer()` | ~85 lines | ~40 lines + helper |
| Testability | Difficult | Each handler testable independently |
| Extensibility | Must search all branches | Add to specific view handler |

---

## Key Handling Verification

All keys handled in `handle_key()` are covered by the refactoring plan:

| Key | Action | Handler |
|-----|--------|---------|
| `q` | Quit | `handle_common_keys()` |
| `Tab` | SwitchView | `handle_common_keys()` |
| `/` | Filter mode | `handle_common_keys()` |
| `?` | Show help | `handle_common_keys()` |
| `c` | Connect | `handle_connect()` |
| `Space` | Toggle off (Settings) | `handle_settings_key()` |
| `l` | Move to cities | `handle_servers_key()` |
| `h` / `Backspace` | Move to countries | `handle_servers_key()` |
| `Enter` | Toggle/Input | `handle_servers_key()` / `handle_settings_key()` |
| `Ctrl+d` | Page down | `handle_page_down()` |
| `d` | Disconnect | `handle_disconnect()` |
| `r` | Refresh | `handle_refresh()` |
| `s` | Cycle sort | `handle_cycle_sort()` |
| `f` | Cycle sort field | `handle_cycle_sort_field()` |
| `j` / `↓` | Navigate down | `handle_navigation_down()` |
| `k` / `↑` | Navigate up | `handle_navigation_up()` |
| `g` | Go to top (gg) | `handle_go_to_first()` |
| `G` | Go to bottom | `handle_go_to_last()` |
| `Ctrl+u` | Page up | `handle_page_up()` |
| `x` | Connect random | `handle_connect_random()` |
| `Esc` | Clear filter | `handle_filter_input()` |

### Filter Mode Handling

The filter/DNS input mode is handled by `handle_filter_input()` (lines 394-444).
This function should be **kept as-is** during refactoring - no changes needed.

**Design Decision:** Maintain the current pattern where filter mode keys are handled separately:
```rust
fn handle_key(&mut self, key_event: KeyEvent) -> Option<AppAction> {
    // 1. First check if in filter/DNS input mode
    if self.filter_mode || self.state.input_mode == InputMode::DnsInput {
        return self.handle_filter_input(key_event);
    }
    
    // 2. Handle common keys (q, Tab, /, ?)
    if let Some(action) = self.handle_common_keys(key_event) {
        return action;
    }
    
    // 3. Dispatch to view-specific handler
    match self.state.get_current_view() { ... }
}
```

---

## Navigation Helper Design

Each navigation method must check `current_view` and `pane_focus` internally:

```rust
fn handle_navigation_down(&mut self) {
    match (self.state.get_current_view(), self.state.get_pane_focus()) {
        (AppView::Servers, Pane::Cities) => self.state.city_select_next(),
        (AppView::Servers, Pane::Countries) => self.state.select_next(),
        (AppView::Settings, _) => self.state.settings_select_next(),
        _ => {}
    }
}
```

Same pattern for: `handle_navigation_up()`, `handle_page_down()`, `handle_page_up()`, `handle_go_to_first()`, `handle_go_to_last()`.

---

## Footer Action Hints Detail

`get_footer_action_hints()` must handle **both view AND pane focus**:

```rust
fn get_footer_action_hints(&self) -> Vec<Span<'_>> {
    match (self.state.get_current_view(), self.state.get_pane_focus()) {
        (AppView::Servers, Pane::Countries) => { /* countries pane hints */ },
        (AppView::Servers, Pane::Cities) => { /* cities pane hints */ },
        (AppView::Settings, _) => { /* settings hints */ },
        (AppView::Logs, _) => { /* logs hints */ },
        (AppView::Help, _) => { /* help hints */ },
    }
}
```

---

## Implementation Order

1. [ ] Extract `handle_common_keys()` method
2. [ ] Extract view-specific handlers: `handle_servers_key()`, `handle_settings_key()`, `handle_logs_key()`, `handle_help_key()`
3. [ ] Extract navigation helpers: `handle_navigation_down()`, `handle_navigation_up()`, etc.
4. [ ] Extract connection action methods
5. [ ] Refactor `handle_key()` to use dispatch pattern
6. [ ] Extract `get_footer_action_hints()` helper
7. [ ] Run `cargo check` and fix any issues
8. [ ] Run `cargo test` to verify no regressions

---

## Files Affected

- `src/ui/app.rs`

## Related Issues

- resolved_issue016: Code Review - Potential Improvements (Category 6 - Code Quality)

## Tags

- refactoring
- code-quality
- low-priority
