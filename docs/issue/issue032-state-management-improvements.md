# issue032 - State Management Improvements

## Summary

This issue covers structural problems with `AppState` and related state management code. Many of these are follow-ups to issue027 (structural improvements that were skipped).

## Related Issues

- issue027: Code Review + Optimization (S1-S4 skipped)
- issue030: Startup Performance Improvements
- issue031: Key Handling Improvements

---

## Status: Partially Implemented (2026-03-12)

### Implemented

- [x] `previous_connection` field - Already implemented and used for connection rollback on failure
- [x] Add AppState Unit Tests (58 tests for Navigatable, ConnectionState)

### Not Implemented

- [ ] Encapsulate AppState fields (getter/setter) - incremental approach

---

## Encapsulation Plan: Incremental Approach

Many getters/setters already exist. Progress incrementally:

### Already Implemented

| Field | Getter | Setter | Action |
|-------|--------|--------|--------|
| `search_query` | - | `set_search_query()` | ✅ |
| `servers` | - | `set_servers()` | ✅ |
| `filter` | - | `set_filter()`, `cycle_filter()` | ✅ |
| `theme` | `get_theme()` | - | ✅ |
| `connection` | `get_connection()` | - | ✅ |
| `current_view` | `get_current_view()` | `set_current_view()` | ✅ (2026-03-12) |
| `pane_focus` | `get_pane_focus()` | - | ✅ |
| `selected_server` | `get_selected_server()` | - | ✅ |
| `selected_city` | `get_selected_city()` | - | ✅ |
| `settings_selected` | `get_settings_selected()` | - | ✅ |
| `settings_expanded` | `is_settings_expanded()` | `set_settings_expanded()` | ✅ |
| `settings_option_selected` | `get_settings_option_selected()` | `set_settings_option_selected()` | ✅ |
| `logs_selected` | `get_logs_selected()` | - | ✅ |

### Completed Encapsulation (2026-03-12)

- [x] Add `set_current_view()` setter
- [x] Update `servers_view.rs` to use getters (`get_pane_focus()`, `get_selected_server()`, `get_selected_city()`, `get_connection()`)
- [x] Update `app.rs` to use `set_current_view()`

#### Completed Settings & Logs (2026-03-12)

- [x] Add getters: `get_settings_selected()`, `is_settings_expanded()`, `get_settings_option_selected()`, `get_logs_selected()`
- [x] Add setters: `set_settings_expanded()`, `set_settings_option_selected()`, `reset_settings_selection()`
- [x] Update `settings_view.rs` to use getters
- [x] Update `logs_view.rs` to use getters
- [x] Update `app.rs` to use getters/setters for settings fields

---

## Module Split Plan

### Implemented
- [x] `previous_connection` field - connection rollback on failure
- [x] Add AppState Unit Tests (58 tests: Navigatable, ConnectionState)
- [x] Encapsulate UI state fields (selected_*, pane_focus, current_view)
- [x] Encapsulate settings/logs fields

### Not Implemented
- [ ] Module split (incremental approach)

---

## Module Split Plan

### Proposed Structure

```
src/state/
├── mod.rs              # Re-exports
├── app_state.rs        # Main struct (~200 lines)
├── ui_state.rs         # UI state fields + navigation
├── notifications.rs    # Notification management
├── server_data.rs     # Server list + filtering
├── connection.rs     # Connection + async
└── config.rs         # Settings cache
```

### Incremental Approach

#### Phase 1: Create module files (code move only)

| Step | File | Contents | Risk |
|------|------|----------|------|
| 1.1 | `ui_state.rs` | UI fields + navigation methods | Low |
| 1.2 | `notifications.rs` | Notification types + methods | Low |
| 1.3 | `server_data.rs` | Server fields + filter methods | Medium |
| 1.4 | `connection.rs` | ConnectionState + async | Medium |

#### Phase 2: Update imports & tests

#### Phase 3 (Optional): Make fields truly private

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| High | `previous_connection` implementation | Low | ✅ Done |
| Medium | Add AppState Unit Tests | Medium | ✅ Done (58 tests) |
| Medium | Encapsulate UI state fields (selected_*, pane_focus, current_view) | Medium | ✅ Done |
| Low | Encapsulate settings/logs fields | Medium | ✅ Done |
| Low | Split AppState into modules | High | Pending |

---

## References

- issue027 Part 3: Structural Improvements (skipped S1-S4)
- Rust best practices: Builder pattern, Newtype pattern for state
