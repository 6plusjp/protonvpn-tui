# issue032 - State Management Improvements

## Summary

This issue covers structural problems with `AppState` and related state management code. Many of these are follow-ups to issue027 (structural improvements that were skipped).

## Related Issues

- issue027: Code Review + Optimization (S1-S4 skipped)
- issue030: Startup Performance Improvements
- issue031: Key Handling Improvements

---

## Problems Identified

### 1. Unused Field: `previous_connection`

**Location**: `src/state/app_state.rs:142`

```rust
previous_connection: Option<ConnectionState>,
```

**Issue**: This field exists in the struct but is never used in the codebase.

**Action**: Either implement its intended use (e.g., detecting connection state changes for notifications) or remove it.

---

### 2. Too Many Public Fields in AppState

**Location**: `src/state/app_state.rs:138-182`

```rust
pub struct AppState {
    // Many fields marked as public
    pub current_view: crate::state::AppView,
    pub selected_server: Option<usize>,
    pub selected_city: Option<usize>,
    pub pane_focus: Pane,
    pub settings_selected: Option<usize>,
    pub settings_expanded: bool,
    pub settings_option_selected: usize,
    pub logs_selected: Option<usize>,
    pub search_query: String,
    pub filter: ServerFilter,
    pub sort: ServerSort,
    pub sort_direction: SortDirection,
    pub is_dark_theme: bool,
    pub input_mode: InputMode,
    pub dns_input: String,
    pub notifications: Vec<ToastNotification>,
    pub notification_log: Vec<Notification>,
    // ... and more
}
```

**Issue**: 
- External code can mutate fields directly without going through methods
- No encapsulation - difficult to add validation or trigger side effects
- Makes testing harder (can't easily mock or verify state changes)

**Proposed Solution**: Use getter/setter methods with private fields:

```rust
struct AppState {
    current_view: AppView,
    selected_server: Option<usize>,
    // ... private fields
}

impl AppState {
    pub fn set_selected_server(&mut self, index: Option<usize>) {
        // Add validation, logging, or trigger side effects
        self.selected_server = index;
    }

    pub fn selected_server(&self) -> Option<usize> {
        self.selected_server
    }
}
```

---

### 3. No Unit Tests for AppState Core Logic

**Location**: `tests/state_test.rs`

**Current State**: Tests exist only for enum behavior:
- `ServerFilter` - cycle, label
- `ServerSort` - cycle, label
- `SortDirection` - toggle

**Missing Tests**:
- Connection state transitions
- Server selection navigation (`move_next`, `move_prev`, etc.)
- Filter application
- Notification handling
- Async task state management

**Proposed Test Coverage**:

```rust
mod app_state {
    #[test]
    fn test_navigation_move_next() {
        let mut selected: Option<usize> = None;
        selected.move_next(10);
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn test_navigation_move_next_wraps() {
        let mut selected = Some(9);
        selected.move_next(10);
        assert_eq!(selected, Some(9)); // Stays at max
    }

    #[test]
    fn test_connection_state_transitions() {
        // Test connection -> disconnect -> connection flow
    }

    #[test]
    fn test_filter_cache_invalidation() {
        // Test that changing search query invalidates cache
    }
}
```

---

## Structural Refactoring (Follow-up to issue027 S1)

### S1-1: Split AppState by Responsibility

**Current**: Single 1500+ line file with mixed concerns

**Proposed**: Split into modules:

```
src/state/
├── mod.rs              # Re-exports
├── app_state.rs        # Main struct (reduced)
├── connection.rs       # Connection state management
├── ui_state.rs         # UI-related state (views, selection)
├── server_data.rs     # Server list and filtering
├── notifications.rs    # Notification management
└── config.rs          # Settings caching
```

### S1-2: Extract Async Task Management

**Current**: `async_tasks.rs` mixed with state

**Proposed**: Move to dedicated module with clearer boundaries:

```rust
// src/state/async_manager.rs
pub struct AsyncManager {
    pool: ThreadPool,
    pending: HashMap<TaskId, TaskReceiver>,
}

impl AsyncManager {
    pub fn spawn_connect(&mut self, server: String) -> ConnectReceiver;
    pub fn spawn_refresh(&mut self) -> ServerReceiver;
    pub fn check_completed(&mut self) -> Vec<AsyncEvent>;
}
```

---

## Priority

| Priority | Item | Effort |
|----------|------|--------|
| High | Remove or implement `previous_connection` | Low |
| Medium | Add AppState Unit Tests | Medium |
| Medium | Encapsulate AppState fields (getter/setter) | Medium |
| Low | Split AppState into modules | High |

---

## References

- issue027 Part 3: Structural Improvements (skipped S1-S4)
- Rust best practices: Builder pattern, Newtype pattern for state
