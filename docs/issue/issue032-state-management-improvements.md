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

### Goal

Split `app_state.rs` (1817 lines) by responsibility.

### Proposed Structure

```
src/state/
├── mod.rs
├── app_state.rs        # AppState struct, new(), vpn_state, Navigatable trait
├── ui_state.rs         # InputMode, SearchQuery + UI fields
├── notifications.rs   # NotificationType, Notification, ToastNotification + NotificationManager
├── server_data.rs     # Server fields + filtered_servers + filter/sort methods
├── connection.rs      # AsyncEvent, AsyncNotifier + connection methods
└── config.rs         # proton_settings_cache
```

### What to Move

| Module | Types to Move | Fields to Move | Methods to Move |
|--------|---------------|----------------|-----------------|
| `ui_state.rs` | `InputMode` | current_view, selected_*, pane_focus, settings_*, logs_*, filter, sort, search_query, is_dark_theme, input_mode, dns_input | navigation methods |
| `notifications.rs` | `NotificationType`, `Notification`, `ToastNotification` | notifications, notification_log | show/clear/tick methods |
| `server_data.rs` | - | servers, filtered_servers_cache, current_cities, current_country_code | filtered_servers(), filter/sort methods |
| `connection.rs` | `AsyncEvent`, `AsyncNotifier` | connection, previous_connection, async_manager, async_notifier, pending_* | connect/disconnect, async methods |
| `config.rs` | - | proton_settings_cache | settings methods |
| `app_state.rs` | - | vpn_state (keep here) | new(), default(), Navigatable impl |

### Incremental Approach

#### Phase 1: Move type definitions (DONE) ✅

| Step | File | Types to Move | Status |
|------|------|---------------|--------|
| 1.1 | `ui_state.rs` | `InputMode` | ✅ Done |
| 1.2 | `notifications.rs` | `NotificationType`, `Notification`, `ToastNotification` | ✅ Done |
| 1.3 | `connection.rs` | `AsyncEvent`, `AsyncNotifier` | ✅ Done |

### Refactoring for Future Extraction (DONE) ✅

To make future module extraction easier, added wrapper types:

| Type | File | Purpose |
|------|------|---------|
| `SearchQuery` | `ui_state.rs` | Encapsulates search_query + search_query_lower |
| `NotificationManager` | `notifications.rs` | Handles notification state mutations |

**Benefits**:
- Ensures related fields stay in sync (e.g., query + query_lower)
- Provides clean API for future field extraction
- Reduces coupling in AppState

#### Phase 2: Move fields (POSTPONED)

**Reason**: High complexity due to:
- Many fields are tightly coupled (e.g., search_query + search_query_lower)
- filter/sort fields tied to server_data
- Moving fields requires moving all methods that use them
- Risk of breaking existing functionality

**Current state**: 82 lines reduced (1817 → 1735), types organized into modules

#### Phase 3: Move methods (POSTPONED)

#### Phase 4: Update imports & tests

### Notes

- `vpn_state: Arc<VpnClient>` stays in `app_state.rs` (used for VPN access)
- Keep all methods that need access to multiple field groups in `app_state.rs`

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| High | `previous_connection` implementation | Low | ✅ Done |
| Medium | Add AppState Unit Tests | Medium | ✅ Done (58 tests) |
| Medium | Encapsulate UI state fields (selected_*, pane_focus, current_view) | Medium | ✅ Done |
| Low | Encapsulate settings/logs fields | Medium | ✅ Done |
| Low | Module split - Type definitions | Low | ✅ Done (Phase 1) |
| Low | Refactoring for extraction (SearchQuery, NotificationManager) | Low | ✅ Done |
| Low | Module split - Fields & Methods | High | ⏸️ Postponed (complexity) |

### Future Work Required

Phase 2 (fields) and Phase 3 (methods) are still required but postponed due to complexity.

**Completed prep work**:
- ✅ SearchQuery wrapper (reduces search_query + search_query_lower coupling)
- ✅ NotificationManager (prepares notification method extraction)

**When to revisit**:
- After other refactoring simplifies dependencies
- When more time available for careful migration

**Approach**:
1. Move UI state fields first (lowest coupling)
2. Then move notification fields
3. Then server_data fields (most complex due to cache)
4. Finally connection fields

---

## References

- issue027 Part 3: Structural Improvements (skipped S1-S4)
- Rust best practices: Builder pattern, Newtype pattern for state
