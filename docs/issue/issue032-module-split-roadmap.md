# Module Split Roadmap

## Overview

Split `AppState` (1763 lines) into focused modules by responsibility.

```
AppState (after split)
├── Connection & Async (12 fields)  → Phase 2.4 ✅ Done (Foundation)
├── Server Data (5 fields)          → Phase 2.3 ✅ Done
├── UI State (16 fields)            → Phase 2.1 ✅ Done
├── Notification (2 fields)          → Phase 2.2 ✅ Done
└── Config (1 field)                 → Phase 2.5 ✅ Done
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
- Updated `set_servers()` to sync with server_data

### Existing Types
- `ServerDataState` (new)

### Approach
1. Created `ServerDataState` struct in new file
2. Added `server_data: ServerDataState` field to AppState
3. Added delegation methods to AppState
4. Kept old fields in AppState for backward compatibility
5. Tests pass

### Complexity
**Low** - Foundation added, fields not yet migrated

### Approach
1. Create `ServerDataState` struct (includes cache)
2. Add `server_data: ServerDataState` to `AppState`
3. Add delegation methods to `AppState`
4. Tests pass

### Complexity
**Medium** - Cache and filter/sort coupling

---

## Phase 2.4: Connection State

### Target Module
`src/state/connection.rs` (expand existing)

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

### Methods to Move
- `connect()`
- `connect_random()`
- `connect_city()`
- `disconnect()`
- `refresh_servers()`
- `sync_connection_state()`
- `process_async_events()`
- `wait_for_async_events()`

### Existing Types
- `AsyncEvent`
- `AsyncNotifier`
- `ConnectionState`

### Approach
1. Create `ConnectionManager` struct (includes async_manager, notifier)
2. Add `connection_manager: ConnectionManager` to `AppState`
3. Add delegation methods to `AppState`
4. Tests pass

### Complexity
**High** - Deep coupling with VPN operations

---

## Phase 2.5: Config State

### Target Module
New file: `src/state/config_state.rs`

### Fields to Move
- `proton_settings_cache: Option<ProtonSettings>`

### Methods to Move
- `get_proton_settings()`
- `get_proton_protocol()`
- `clear_settings_cache()`

### Dependencies
- `vpn_state` - for settings operations

### Approach
1. Create `ConfigState` struct
2. Add `config_state: ConfigState` to `AppState`
3. Add delegation methods to `AppState`
4. Tests pass

### Complexity
**Low** - Can load independently

---

## Execution Order

```
Phase 2.2 (Low)  ─┐
Phase 2.5 (Low)  ─┼─> Parallel possible
                  │
Phase 2.3 (Med) ──┤
                  │
Phase 2.4 (High) ┘
```

---

## Principles

1. **Backward Compatibility**: Add new struct field to AppState, keep legacy fields
2. **Delegation Pattern**: AppState delegates to sub-state structs
3. **Incremental Migration**: Each phase maintains working state
4. **Tests First**: Run tests after each phase

---

## Progress Tracking

| Phase | Complexity | Status | Notes |
|-------|------------|--------|-------|
| 2.1 UI State | Medium | ✅ Done | 16 fields moved to UiState |
| 2.2 Notification | Low | ✅ Done | 2 fields moved to NotificationState |
| 2.3 Server Data | Low | ✅ Done | Foundation added, 4 fields in ServerDataState |
| 2.5 Config | Low | ✅ Done | 1 field + methods moved to ConfigState |
| 2.4 Connection | High | ✅ Done (Foundation) | ConnectionManager struct added, 10 fields |
