# State Module Architecture

## Overview

The `state/` module manages all application state including UI state, VPN connection state, notifications, server data, and user preferences.

## Module Structure

```
src/state/
├── mod.rs              # Module root - re-exports public APIs
├── app_state.rs        # AppState container + Navigatable trait + tests
├── app_state_impl.rs  # VPN connect/disconnect operations
├── connection_manager.rs       # Async operations + event handling
├── connection_state.rs # VPN connection state enum
├── navigation.rs       # Selection & pane navigation methods
├── server_ops.rs       # Server filtering, sorting, favorites
├── settings_ops.rs     # Settings management operations
├── event_handler.rs    # Async event processing logic
├── notifications.rs    # Toast notifications + notification log
├── ui_state.rs         # UI state (selection, scroll, filters)
├── app_view.rs        # Current view (servers, connect, settings, logs)
├── server_filter.rs   # Server filtering logic
├── server_sort.rs     # Server sorting logic
├── config_state.rs    # User configuration state
└── log_persistence.rs # Notification log persistence
```

## Core Concepts

### AppState (app_state.rs)

The central state container that holds all sub-states. **Methods are distributed across modules:**

| Module | Responsibility |
|--------|----------------|
| `app_state.rs` | Struct definition, Navigatable trait, constructors |
| `app_state_impl.rs` | VPN connect/disconnect operations |
| `navigation.rs` | Selection & pane navigation |
| `server_ops.rs` | Server filtering, sorting, favorites |
| `settings_ops.rs` | Settings management |
| `event_handler.rs` | Async event processing |

```rust
pub struct AppState {
    pub vpn_state: Arc<VpnClient>,
    pub connection_manager: ConnectionManager,
    pub servers: Vec<Server>,
    pub current_cities: Vec<City>,
    pub ui_state: UiState,
    pub notification_state: NotificationState,
    pub config_state: ConfigState,
    pub is_initialized: bool,
    // ...
}
```

### ConnectionManager (connection_manager.rs)

Manages async VPN operations and event-driven notifications:

```rust
pub struct ConnectionManager {
    pub async_manager: AsyncTaskManager,
    pub async_notifier: AsyncNotifier,
    pub pending_connect: HashMap<(), ConnectReceiver>,
    pub pending_cities: HashMap<String, CitiesReceiver>,
    // ...
}
```

### Notification System (notifications.rs)

Two types of notifications:
1. **Toast Notifications** - Temporary popups (visible for limited time)
2. **Notification Log** - Persistent history

#### Toast Notifications

Toast notifications use an `operation_key` for deduplication:

```rust
// Show notification with key - replaces any existing notification with same key
self.show_notification(
    "Loading cities...".to_string(),
    NotificationType::Info,
    Some("cities:US".to_string()),  // key for deduplication
);

// Show notification without key - multiple can coexist
self.show_notification(
    "Some info".to_string(),
    NotificationType::Info,
    None,
);
```

**Key Rules:**
- Loading notifications MUST have a key to be replaced by success/error
- Success/Error notifications for the same operation MUST use the same key
- Key format: `cities:{country_code}`, `connect`, `disconnect`, etc.

## Design Principles

### 1. Single Responsibility

Each file has one clear purpose:

| File | Responsibility |
|------|----------------|
| `app_state.rs` | Container + Navigatable trait + constructors |
| `app_state_impl.rs` | VPN connect/disconnect operations |
| `navigation.rs` | Selection & pane navigation methods |
| `server_ops.rs` | Server filtering, sorting, favorites |
| `settings_ops.rs` | Settings management operations |
| `event_handler.rs` | Async event processing |
| `connection_manager.rs` | Async task management, event channels |
| `notifications.rs` | Toast + log notification management |
| `ui_state.rs` | Selection, scroll, filter state |
| `server_filter.rs` | Filter logic (by country, city, features) |
| `server_sort.rs` | Sort logic (by name, load, country) |

### 2. State Mutation

State should be mutated through methods on `AppState`:

```rust
// DO THIS
self.current_cities.clear();
self.show_notification(...);

// NOT THIS (direct field mutation when method exists)
app_state.current_cities = Vec::new();
```

### 3. Async Operations

Long-running operations (VPN connect, server refresh) use async tasks:

```rust
// Spawn async operation
self.connection_manager.async_manager.spawn_connect(
    self.vpn_state.clone(),
    server_id,
    tx,
);

// Handle result in check_pending_async_events()
if let Ok(result) = rx.try_recv() {
    // Show notification, update state
}
```

### 4. Notifications Always Have Keys

When showing loading → success/error flow, always use the same key:

```rust
// Loading
self.show_notification(
    format!("Loading cities for {}...", country_code),
    NotificationType::Info,
    Some(format!("cities:{}", country_code)),  // KEY REQUIRED
);

// Success
self.show_notification(
    format!("Loaded {} cities", count),
    NotificationType::Success,
    Some(format!("cities:{}", country_code)),  // SAME KEY
);

// Error
self.show_notification(
    format!("Failed to load cities: {}", e),
    NotificationType::Error,
    Some(format!("cities:{}", country_code)),  // SAME KEY
);
```

## When Adding New State

### New UI State? → Add to `ui_state.rs`

```rust
// src/state/ui_state.rs
pub struct UIState {
    pub selected_server: usize,
    pub scroll_offset: usize,
    // Add new field here
    pub new_field: String,
}
```

### New Notification Type? → Add to `notifications.rs`

```rust
// src/state/notifications.rs
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Error,
    // Add new type here
}
```

### New Async Operation? → Add to `connection_manager.rs`

```rust
// src/state/connection_manager.rs
pub enum AsyncEvent {
    ServersRefreshed(Vec<Server>),
    Connected(ConnectResult),
    // Add new event here
}
```

## Testing

- **State tests**: In `app_state.rs` (search for `#[cfg(test)]` module)
- **Notification tests**: In `app_state.rs` (notification_tests module)

Run tests:
```bash
cargo test --lib
```

## Adding New Methods

When adding methods to AppState, consider which module it belongs to:

| Method Type | Module |
|-------------|--------|
| Selection/navigation | `navigation.rs` |
| Server filtering/sorting/favorites | `server_ops.rs` |
| Settings toggle/apply | `settings_ops.rs` |
| Async event handling | `event_handler.rs` |
| VPN connect/disconnect | `app_state_impl.rs` |
| State accessors/mutations | `app_state.rs` |

## Anti-Patterns

### Don't add notification without key for operations

```rust
// DON'T DO THIS - loading has no key, success has key
self.show_notification("Loading...", NotificationType::Info, None);
self.show_notification("Success!", NotificationType::Success, Some("op".to_string()));

// DO THIS - both have same key
self.show_notification("Loading...", NotificationType::Info, Some("op".to_string()));
self.show_notification("Success!", NotificationType::Success, Some("op".to_string()));
```

### Don't skip error handling

Always handle errors from async operations:

```rust
// DON'T DO THIS - ignore error
let _ = rx.try_recv();

// DO THIS - handle both Ok and Err
if let Ok(result) = rx.try_recv() {
    match result {
        Ok(data) => { /* handle success */ }
        Err(e) => { /* handle error */ }
    }
}
```

### Don't mutate state outside AppState methods

Keep state mutation centralized in `AppState`:

```rust
// DON'T DO THIS - external mutation
app_state.current_cities.clear();
app_state.some_field = value;

// DO THIS - use AppState methods
app_state.clear_cities();
app_state.set_something(value);
```
