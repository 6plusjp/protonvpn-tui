# issue045 - Notification Set: Treat Loading/Loaded as a Set

## Summary

Extend the notification system to treat `Loading...` → `Loaded` as a set. For the same operation key, only one notification should be displayed at a time, replacing or auto-dismissing the previous one.

## Current Problem

Multiple notifications stack up when keys are pressed rapidly during each operation series:

### Refresh Series
- `Refreshing servers...` (Info) - start
- `Refresh in progress...` (Warning) - every tick/keypress while pending
- `Refreshed X servers` (Success) - complete
- `Server list refresh failed: X` (Error) - failure

### Loading Cities Series
- `Loading cities for XX...` (Info) - start (line 1008, 1032)
- `Loaded X cities for XX` (Success) - complete (line 485)
- `Failed to load cities: X` (Error) - failure

### Connect Series
- `Connecting to XX...` (Info) - start (line 793)
- `Connection in progress...` (Warning) - every tick/keypress while connecting (line 209 etc.)
- `Connected to XX` (Success) - complete (line 311)
- `Connection failed: X` (Error) - failure

### Disconnect Series
- `Disconnecting...` (Warning) - start (line 215)
- `Disconnected` (Info) - complete (line 288)
- `Disconnect failed: X` (Error) - failure

### Problems

- Key mashing: `Refresh in progress...` displays multiple times stacked
- Both `Loading cities for XX...` and `Loaded X cities for XX` visible simultaneously
- `notification_state.show()` only appends, never replaces or removes

## Solution

### Option A: Add operation_key (Recommended)

Add `operation_key: Option<String>` to `Notification` / `ToastNotification`:

```rust
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u16,
    pub operation_key: Option<String>,  // e.g., "servers", "cities:FR", "connect"
}
```

When showing a notification, replace/remove existing ones with the same key:

```rust
pub fn show(&mut self, message: String, notification_type: NotificationType, operation_key: Option<String>) {
    // For Loading notifications: replace existing Loading with same key
    // For Loaded/Error notifications: auto-dismiss Loading with same key
    // Otherwise: replace existing with same key
}
```

### Option B: Manage as NotificationSet

Manage a set of notifications with `NotificationSet`:

```rust
pub struct NotificationSet {
    pub loading: Option<ToastNotification>,
    pub loaded: Option<ToastNotification>,
    pub operation_key: String,
}
```

## Implementation Tasks

1. Add `operation_key` field to `ToastNotification`
2. Extend `NotificationState::show()` to implement key-based replacement/dismissal
3. Update all call sites to pass operation_key:
   - `refresh_servers()` → `"servers"`
   - `fetch_cities(country_code)` → `format!("cities:{}", country_code)`
   - `connect_city(city)` → `format!("connect:{}", city)`
   - `disconnect()` → `"disconnect"`
4. Add auto-dismiss logic: Loading → Loaded/Error dismisses the Loading

## Related

- src/state/notifications.rs - NotificationState implementation
- src/state/app_state.rs - show_notification() call sites
- src/ui/app.rs - "in progress" notifications (line 209, 281, etc.)
