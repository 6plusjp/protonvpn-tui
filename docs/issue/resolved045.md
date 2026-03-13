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

---

## Current Implementation Status

### Files Involved

| File | Role |
|------|------|
| `src/state/notifications.rs` | Core notification state management |
| `src/state/app_state.rs` | Main call sites (60+ occurrences) |
| `src/ui/app.rs` | UI layer "in progress" notifications |
| `src/constants.rs` | Timer and limit constants |

### Current Structure

```rust
// src/state/notifications.rs
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u16,
    // ← operation_key NOT IMPLEMENTED
}

pub fn show(&mut self, message: String, notification_type: NotificationType) {
    // Simply appends to notifications vector
    self.notifications.push(ToastNotification { ... });
}
```

### Constants

```rust
// src/constants.rs
pub mod ui {
    pub const NOTIFICATION_TIMER_DEFAULT: u16 = 300; // 30秒 (10ms tick)
    pub const NOTIFICATION_TIMER_SHORT: u16 = 150;  // 15秒
    pub const MAX_VISIBLE_NOTIFICATIONS: usize = 3;
}
```

---

## Detailed Flow Analysis

### A. Server Refresh Flow

| Phase | Function | Notification Message | Line | Key |
|-------|----------|---------------------|------|-----|
| Start | `refresh_servers()` | `"Refreshing servers..."` | 607 | `"servers"` |
| In Progress (tick) | `app.rs` | `"Refresh in progress..."` | 281, 291 | `"servers"` |
| Success | event handler | `"Refreshed X servers"` | 240 | `"servers"` |
| Failure | event handler | `"Refresh failed: X"` | 252 | `"servers"` |

### B. City Loading Flow

| Phase | Function | Notification Message | Line | Key |
|-------|----------|---------------------|------|-----|
| Start | `fetch_cities()` | `"Loading cities for XX..."` | 1014 | `"cities:XX"` |
| Reload | `reload_cities()` | `"Loading cities for XX..."` | 1038 | `"cities:XX"` |
| Success | event handler | `"Loaded X cities for XX"` | 489 | `"cities:XX"` |
| Failure | event handler | `"Failed to load cities: X"` | 496 | `"cities:XX"` |

### C. Connection Flow (6 types)

| Connection Type | Start Notification | Line | Key |
|----------------|-------------------|------|-----|
| `connect()` | `"Connecting to {country}..."` | 645 | `"connect:{country}"` |
| `connect_random()` | `"Connecting to random server..."` | 665 | `"connect:random"` |
| `connect_fastest()` | `"Connecting to fastest server..."` | 685 | `"connect:fastest"` |
| `connect_p2p()` | `"Connecting to P2P server..."` | 705 | `"connect:p2p"` |
| `connect_tor()` | `"Connecting to Tor server..."` | 725 | `"connect:tor"` |
| `connect_securecore()` | `"Connecting to SecureCore server..."` | 745 | `"connect:securecore"` |
| `connect_city()` | `"Connecting to {city}..."` | 798 | `"connect:city:{city}"` |

### D. Disconnect Flow

| Phase | Function | Notification Message | Line | Key |
|-------|----------|---------------------|------|-----|
| Start | `disconnect()` | `"Disconnecting from {server}..."` | 774 | `"disconnect"` |
| Complete | event handler | `"Disconnected"` | 289 | `"disconnect"` |
| Failure | event handler | `"Disconnect failed: X"` | 299 | `"disconnect"` |

### E. UI Layer ("In Progress" Notifications)

| Location | Notification Message | Line | Issue |
|----------|----------------------|------|-------|
| `app.rs:210` | `"Connection in progress..."` | 210 | Shows on every keypress while connecting |
| `app.rs:281, 291` | `"Refresh in progress..."` | 281, 291 | Same issue |
| `app.rs:321, 341, 361, 381, 401` | `"Connection in progress..."` | various | Multiple connect shortcuts |

### F. All show_notification Call Sites (60+ occurrences)

| Category | Lines | Example |
|----------|-------|---------|
| Errors | 236, 252, 283, 300, 335, 364, 423, 462, etc. | `"No server selected"`, `"Connection failed: X"` |
| Success | 241, 260, 314, 369, 397, 490, etc. | `"Refreshed X servers"`, `"Connected to X"` |
| Info | 289, 448, 607, 646, 666, 686, etc. | `"Refreshing servers..."`, `"Disconnected"` |
| Loading | 798, 1015, 1039 | `"Connecting to..."`, `"Loading cities..."` |

---

## Solution

### Option A: Add operation_key (Recommended)

Add `operation_key: Option<String>` to `ToastNotification` / `Notification`:

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

### Replacement Logic

```rust
pub fn show(&mut self, message: String, notification_type: NotificationType, operation_key: Option<String>) {
    // 1. Loading (Info) → Replace existing Loading with same key
    // 2. Success/Error → Auto-dismiss Loading with same key
    // 3. If same key exists → Replace
    // 4. Otherwise → Append
    
    // notification_log: Always append (unchanged)
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

---

## Implementation Plan

### Phase 1: Core Infrastructure (Low Risk)

1. Add `operation_key` field to `ToastNotification`
2. Add `operation_key` field to `Notification` (for log consistency)
3. Extend `NotificationState::show()` signature: `show(msg, type, key?)`
4. Implement basic key-based replacement logic

### Phase 2: Core Call Sites Update (Medium Risk)

5. Update all call sites in `app_state.rs` to pass operation_key:
   - `refresh_servers()` → `"servers"`
   - `fetch_cities(country_code)` → `format!("cities:{}", country_code)`
   - `reload_cities()` → `format!("cities:{}", country_code)`
   - `connect()` → `format!("connect:{}", server_country)`
   - `connect_random()` → `"connect:random"`
   - `connect_fastest()` → `"connect:fastest"`
   - `connect_p2p()` → `"connect:p2p"`
   - `connect_tor()` → `"connect:tor"`
   - `connect_securecore()` → `"connect:securecore"`
   - `connect_city(city)` → `format!("connect:city:{}", city)`
   - `disconnect()` → `"disconnect"`

### Phase 3: UI Layer Update (Medium Risk)

6. Update "in progress" notifications in `app.rs`:
   - Line 210: `"Connection in progress..."` → key: `"connect:in_progress"`
   - Line 281, 291: `"Refresh in progress..."` → key: `"servers:in_progress"`
   - Line 321, 341, 361, 381, 401: Similar updates

### Phase 4: Auto-Dismiss Logic (High Risk)

7. Implement auto-dismiss logic:
   - Loading → Success: Dismiss Loading notification
   - Loading → Error: Dismiss Loading notification
8. Add comprehensive tests

---

## Risk Assessment

| Phase | Risk Level | Reason |
|-------|-----------|--------|
| Phase 1 | Low | Struct additions only, backward compatible |
| Phase 2 | Medium | 60+ call site changes |
| Phase 3 | Medium | UI layer changes |
| Phase 4 | High | Auto-dismiss logic complexity |

---

## Test Cases (To Be Added)

| Test | Expected Behavior |
|------|-------------------|
| `test_same_key_loading_replaces` | Loading with same key replaces existing |
| `test_loaded_dismisses_loading` | Success/Error dismisses Loading |
| `test_different_keys_stack` | Different keys coexist |
| `test_ui_in_progress_replaces` | UI notifications also replace |
| `test_notification_log_preserves_all` | Log still captures all notifications |
| `test_backward_compatibility` | None key works as before |

---

## Related

- `src/state/notifications.rs` - NotificationState implementation
- `src/state/app_state.rs` - show_notification() call sites (60+ occurrences)
- `src/ui/app.rs` - "in progress" notifications (line 209, 281, etc.)
- `src/constants.rs` - Timer and limit constants
- `docs/issue/resolved032.md` - Related state management improvements

---

## Status

**IMPLEMENTED (2026-03-14)**

### Implemented Features

- [x] Phase 1: Core Infrastructure
  - [x] Added `operation_key` field to `ToastNotification`
  - [x] Added `operation_key` field to `Notification` (for log consistency)
  - [x] Extended `NotificationState::show()` with key-based replacement logic
  - [x] Implemented auto-dismiss: Loading (Info) → Success/Error removes Loading

- [x] Phase 2: Core Call Sites Update
  - [x] Updated all call sites in `app_state.rs` with operation keys
  - [x] Server refresh: `"servers"` key
  - [x] City loading: `"cities:{country_code}"` key
  - [x] Connect: `"connect"` / `"connect:city"` keys
  - [x] Disconnect: `"disconnect"` key

- [x] Phase 3: UI Layer Update
  - [x] Updated "in progress" notifications in `app.rs` with keys

- [x] Phase 4: Tests
  - [x] Added 6 new tests in `notifications.rs`:
    - `test_same_key_loading_replaces`
    - `test_loaded_dismisses_loading`
    - `test_error_dismisses_loading`
    - `test_different_keys_stack`
    - `test_notification_log_preserves_all_with_keys`
    - `test_none_key_does_not_replace`

### Key Implementation Details

The `show()` method implements:
1. **Auto-dismiss**: When Success/Error arrives with a key, any Info (Loading) notification with the same key is automatically removed
2. **Replacement**: When a notification with the same key arrives, the existing one is replaced
3. **Backward compatibility**: Calls without a key (None) work as before - they just don't participate in key-based replacement

### Files Changed

- `src/state/notifications.rs` - Core logic + 6 tests
- `src/state/app_state.rs` - All show_notification call sites
- `src/ui/app.rs` - UI layer notifications
