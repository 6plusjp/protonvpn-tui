# issue032 - State Management Improvements

## Summary

This issue covers structural problems with `AppState` and related state management code. Many of these are follow-ups to issue027 (structural improvements that were skipped).

## Related Issues

- issue027: Code Review + Optimization (S1-S4 skipped)
- issue030: Startup Performance Improvements
- issue031: Key Handling Improvements

---

## Status: COMPLETED (2026-03-12)

### Implemented

- [x] `previous_connection` field - Already implemented and used for connection rollback on failure
- [x] Add AppState Unit Tests (58 tests for Navigatable, ConnectionState)
- [x] **Phase 2.1: UI State Delegation** (2026-03-12)
  - [x] UiState struct added to ui_state.rs with all UI fields (16 fields)
  - [x] AppState.ui_state field added as delegation target
  - [x] All getters/setters for UI fields
  - [x] Updated app.rs and views to use getters/setters
  - [x] Updated tests to use ui_state delegation
- [x] **Phase 2.2: Notification State Delegation** (2026-03-12)
  - [x] NotificationState struct added to notifications.rs
  - [x] AppState.notification_state field added as delegation target
  - [x] Getters: get_notifications(), get_notification_log(), len(), is_empty(), is_log_empty()
  - [x] Methods: show(), clear(), tick()
  - [x] Updated app.rs to use getters
  - [x] Updated logs_view.rs to use getter
  - [x] Updated tests to use notification_state delegation
- [x] **Public Field Cleanup** (2026-03-12)
  - [x] Made `connection` field private (use get_connection() getter)
  - [x] Made `is_initialized` field private (use is_initialized() getter)
  - [x] Updated app.rs to use getters
  - [x] Updated stats_view.rs to use get_connection()
- [x] **Phase 2.3: Server Data State Delegation** (2026-03-12)
  - [x] ServerDataState struct added to server_data.rs
  - [x] AppState.server_data field added as delegation target
  - [x] Getters: get_servers(), get_current_cities(), get_current_country_code(), is_initialized()
  - [x] Setters: set_initialized()
  - [x] Updated set_servers() to sync with server_data
  - [x] Updated is_initialized() to delegate to server_data
- [x] **Phase 2.4: Connection State Delegation** (2026-03-12)
  - [x] ConnectionManager struct added to connection.rs
  - [x] 10 fields moved: connection, previous_connection, async_manager, async_notifier, pending_*
  - [x] AppState.connection_manager field as delegation target
  - [x] Updated app.rs, servers_view.rs, stats_view.rs to use connection_manager
- [x] **Phase 2.5: Config State Delegation** (2026-03-12)
  - [x] ConfigState struct added to config_state.rs
  - [x] proton_settings_cache field moved
  - [x] AppState.config_state field as delegation target
  - [x] Updated settings_view.rs, stats_view.rs, app.rs to use config_state
- [x] **Phase 2.6: Getter/Setter Removal** (2026-03-12)
  - [x] Remove unnecessary getter/setter methods per AGENTS.md coding standards
  - [x] Use pub fields instead of getter/setter patterns
  - [x] Make ui_state, server_data, notification_state, connection_manager, config_state pub in AppState
  - [x] Update callers to use direct field access: `state.ui_state.field`

### Not Implemented

- (None - All phases completed)

---

## Module Split Roadmap

### Overview

Split `AppState` (1763 lines) into focused modules by responsibility. **All phases completed (2026-03-12)**.

```
AppState (after split)
├── VPN State (1 field)           → stays in AppState
├── Connection Manager (10 fields) → Phase 2.4 ✅ Done
├── Server Data (5 fields)        → Phase 2.3 ✅ Done
├── UI State (16 fields)          → Phase 2.1 ✅ Done
├── Notification (2 fields)       → Phase 2.2 ✅ Done
└── Config (1 field)              → Phase 2.5 ✅ Done
```

---

## Phase 2.1: UI State ✅ Done (2026-03-12)

### Target Module
`src/state/ui_state.rs`

### Fields to Move
- `current_view`, `selected_server`, `selected_city`, `pane_focus`
- `settings_selected`, `settings_expanded`, `settings_option_selected`
- `logs_selected`, `search_query`, `filter`, `sort`, `sort_direction`
- `is_dark_theme`, `input_mode`, `dns_input`

### Methods to Move
- Navigation methods (select_next, select_prev, etc.)
- Filter/sort methods

### Existing Types
- `InputMode`, `SearchQuery`, `ServerCache`, `UiState`

### Approach
1. `UiState` struct already exists
2. Added 30+ getter/setter methods
3. Added `ui_state: UiState` field to AppState
4. Tests pass

### Complexity
**Medium** - 16 fields moved

---

## Phase 2.2: Notification State ✅ Done (2026-03-12)

### Target Module
`src/state/notifications.rs`

### Fields to Move
- `notifications: Vec<ToastNotification>`
- `notification_log: Vec<Notification>`

### Methods to Move
- `show_notification()`
- `clear_notifications()`
- `tick_notifications()`

### Existing Types
- `NotificationType`
- `Notification`
- `ToastNotification`
- `NotificationManager` (already extracted)

### Approach
1. Created `NotificationState` struct
2. Added `notification_state: NotificationState` field to AppState
3. Added delegation methods to AppState
4. Updated views to use getters
5. Tests pass

### Complexity
**Low** - Types already in separate module

---

## Phase 2.3: Server Data State ✅ Done (2026-03-12)

### Target Module
`src/state/server_data.rs` (new file)

### Fields Added to ServerDataState
- `servers: Vec<Server>`
- `is_initialized: bool`
- `current_cities: Vec<City>`
- `current_country_code: Option<String>`

Note: `server_cache` remains in AppState (used by filtered_servers())

### Methods Added
- Getters: `get_servers()`, `get_current_cities()`, `get_current_country_code()`, `is_initialized()`
- Setters: `set_initialized()`, `set_servers()`, `clear_cities()`, `set_country_code()`

### AppState Changes
- Added `server_data: ServerDataState` field
- Added delegation getters: `get_servers()`, `get_current_cities()`, `get_current_country_code()`, `is_initialized()`
- Added delegation setters: `set_initialized()`

### Complexity
**Low** - Simple struct with basic fields

---

## Phase 2.4: Connection State ✅ Done (2026-03-12)

### Target Module
`src/state/connection.rs`

### Fields to Move
- `connection: ConnectionState`
- `previous_connection: Option<ConnectionState>`
- `async_manager: AsyncTaskManager`
- `async_notifier: Arc<AsyncNotifier>`
- `pending_refresh: Option<ServerReceiver>`
- `pending_connect: Option<ConnectReceiver>`
- `pending_disconnect: Option<DisconnectReceiver>`
- `pending_cities: HashMap<String, CitiesReceiver>`
- `pending_connect_city: Option<ConnectReceiver>`
- `pending_config_set: Option<ConfigReceiver>`

### Methods to Keep in AppState
- All connection-related methods (connect, disconnect, refresh, etc.)
- These methods need access to vpn_state and other AppState fields
- Can delegate to ConnectionManager internally

### Approach
1. Created `ConnectionManager` struct in connection.rs
2. Added `connection_manager: ConnectionManager` field to AppState
3. All internal method references updated to use `self.connection_manager.*`
4. External callers updated to use `state.connection_manager.*`
5. Tests pass

### Complexity
**High** - 10 fields + async operations

---

## Phase 2.5: Config State ✅ Done (2026-03-12)

### Target Module
`src/state/config_state.rs` (new file)

### Fields to Move
- `proton_settings_cache: Option<ProtonSettings>`

### Methods to Move
- `get_proton_settings()` - returns cached settings
- `clear_settings_cache()` - reloads from disk

### Approach
1. Created `ConfigState` struct in config_state.rs
2. Added `config_state: ConfigState` field to AppState
3. Added delegation methods to AppState
4. Updated callers to use config_state
5. Tests pass

### Complexity
**Low** - 1 field + simple methods

---

## Phase Status

| Phase | Complexity | Status | Notes |
|-------|------------|--------|-------|
| 2.1 UI State | Medium | ✅ Done | 16 fields moved to UiState |
| 2.2 Notification | Low | ✅ Done | 2 fields moved to NotificationState |
| 2.3 Server Data | Low | ✅ Done | Foundation added, 4 fields in ServerDataState |
| 2.4 Connection | High | ✅ Done | 10 fields moved to ConnectionManager |
| 2.5 Config | Low | ✅ Done | 1 field + methods moved to ConfigState |

---

## Technical Notes

### Current Architecture (After Getter/Setter Removal)
- AppState has public fields for sub-state structs: `ui_state`, `server_data`, `notification_state`, `connection_manager`, `config_state`
- Callers access fields directly: `state.ui_state.current_view`
- No wrapper getters/setters - follows AGENTS.md coding standards

### Encapsulation Access Patterns

| Field | Access Pattern |
|-------|---------------|
| `ui_state` | `state.ui_state.field` |
| `server_data` | `state.server_data.field` |
| `notification_state` | `state.notification_state.field` |
| `connection` | `state.connection_manager.connection` |
| `connection_manager` | `state.connection_manager.field` |
| `config_state` | `state.config_state.proton_settings_cache` |
| `current_cities` | `state.current_cities` |
| `current_country_code` | `state.current_country_code` |
