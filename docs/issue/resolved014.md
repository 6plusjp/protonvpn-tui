# issue014: Stacked Toast Notifications

## Status

**Resolved** - Implementation complete

Replace the single notification popup with a stacked toast notification system that displays multiple notifications simultaneously without overwriting.

## Problem Statement

Currently, when a new notification arrives while one is already displayed, the old notification is completely replaced. This causes users to miss important messages when multiple events occur in quick succession (e.g., refresh completes while connection is in progress).

## Proposed Solution: Toast Notification System

### Behavior

1. **New notifications appear at the bottom** with a slide-up animation
2. **Existing notifications shift upward** to make room
3. **Each notification has its own timer** - disappears after ~3 seconds
4. **Maximum 3 visible notifications** - oldest is removed when limit exceeded
5. **Notification log preserved** - all notifications still stored in `notification_log`

### Visual Layout

```
┌─────────────────────────────────────────┐
│  ● ProtonVPN TUI              Disconnected │
├─────────────────────────────────────────┤
│                                         │
│            [Main Content]               │
│                                         │
├─────────────────────────────────────────┤
│ [?] help [Tab] switch view [q] quit     │
└─────────────────────────────────────────┘

              ┌──────────────────────┐
              │   Connected to JP   │  ← Newest (bottom)
              └──────────────────────┘
        ┌──────────────────────┐
        │ Refreshed 50 servers│  ← Older (above)
        └──────────────────────┘
  ┌──────────────────────┐
  │   Loading cities...  │  ← Oldest (top)
  └──────────────────────┘
```

## Current Implementation vs Required Changes

| Component | Current | Required Change |
|-----------|---------|-----------------|
| `AppState.notification` | `Option<Notification>` | `Vec<ToastNotification>` |
| Timer management | `TuiApp.notification_timer` (single) | Individual timer per `ToastNotification` |
| Rendering | Single popup at fixed position | Stacked display (newest at bottom) |
| Constants | None | `MAX_VISIBLE_NOTIFICATIONS = 3` |

## Implementation Plan

### Phase 1: Data Structure Changes

#### 1.1 Add `ToastNotification` struct (`src/state/app_state.rs`)

```rust
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u8,           // Individual countdown (decrements each tick)
    pub32,      // created_at: u For ordering (optional, can use index)
}
```

#### 1.2 Modify `AppState` fields

- **Replace**: `notification: Option<Notification>`
- **With**: `notifications: Vec<ToastNotification>`
- **Keep**: `notification_log: Vec<Notification>` (unchanged, stores all notifications)

#### 1.3 Update `show_notification()` method

```rust
pub fn show_notification(&mut self, message: String, notification_type: NotificationType) {
    // Add to notifications vector (visible toasts)
    self.notifications.push(ToastNotification {
        message: message.clone(),
        notification_type,
        timer: NOTIFICATION_TIMER_DEFAULT,
    });

    // Enforce max visible notifications
    if self.notifications.len() > MAX_VISIBLE_NOTIFICATIONS {
        self.notifications.remove(0); // Remove oldest
    }

    // Add to log (all notifications preserved)
    self.notification_log.push(Notification {
        message,
        notification_type,
    });
    if self.notification_log.len() > MAX_NOTIFICATION_LOG {
        self.notification_log.remove(0);
    }
}
```

---

### Phase 2: Timer Management

#### 2.1 Add timer management methods to `AppState`

```rust
/// Decrement timers for all notifications, remove expired ones
pub fn tick_notifications(&mut self) {
    // Decrement all timers
    for notification in &mut self.notifications {
        if notification.timer > 0 {
            notification.timer -= 1;
        }
    }
    // Remove expired notifications (timer == 0)
    self.notifications.retain(|n| n.timer > 0);
}

/// Clear all visible notifications (e.g., on view change)
pub fn clear_notifications(&mut self) {
    self.notifications.clear();
}
```

#### 2.2 Remove from `TuiApp` (`src/ui/app.rs`)

- Remove field: `notification_timer: u8`
- Remove timer update logic in main loop

#### 2.3 Update main loop (`src/ui/app.rs`)

**Before:**
```rust
if self.notification_timer > 0 {
    self.notification_timer -= 1;
    if self.notification_timer == 0 {
        self.state.clear_notification();
    }
}
if notification_shown {
    self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
}
```

**After:**
```rust
self.state.tick_notifications();
if notification_shown {
    // No need to reset single timer - new notification has its own
}
```

---

### Phase 3: UI Rendering

#### 3.1 Add constant (`src/constants.rs`)

```rust
pub mod ui {
    pub const NOTIFICATION_TIMER_DEFAULT: u8 = 30;
    pub const NOTIFICATION_TIMER_SHORT: u8 = 15;
    pub const NOTIFICATION_MSG_MAX_LEN: usize = 35;
    pub const POPUP_WIDTH_MIN: usize = 30;
    pub const POPUP_WIDTH_MAX: usize = 54;
    pub const MAX_VISIBLE_NOTIFICATIONS: usize = 3;  // NEW
}
```

#### 3.2 Modify `render_notification_popup` → `render_toast_notifications`

**Rendering logic:**

```rust
fn render_toast_notifications(&self, f: &mut Frame<'_>) {
    let theme = self.get_theme();
    let terminal = f.size();
    let visible_notifications: Vec<_> = self.state.notifications.iter().rev().collect();

    for (i, toast) in visible_notifications.iter().enumerate() {
        let position_from_bottom = i;
        let popup_height = 3;

        let y = terminal.height.saturating_sub(3 + (position_from_bottom * popup_height) as u16);
        if y < 1 {
            break; // Don't render if off screen
        }

        // Render notification at calculated position
        // ... (same rendering logic as before, but with dynamic y)
    }
}
```

**Position calculation:**
```
y = terminal_height - 3  (bottom area)
for (i, notification) in visible_notifications.iter().enumerate() {
    y_offset = i * height
    render at (y - y_offset)
}
```

#### 3.3 Update render condition

**Before:**
```rust
if self.state.notification.is_some() {
    self.render_notification_popup(f);
}
```

**After:**
```rust
if !self.state.notifications.is_empty() {
    self.render_toast_notifications(f);
}
```

---

### Phase 4: Testing

#### 4.1 Unit Tests (add to `src/state/app_state.rs`)

Add test module at the end of `app_state.rs`:

```rust
#[cfg(test)]
mod notification_tests {
    use super::*;

    #[test]
    fn test_show_notification_adds_to_vector() {
        let mut state = AppState::new();
        state.show_notification("Test 1".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Success);

        assert_eq!(state.notifications.len(), 2);
    }

    #[test]
    fn test_tick_notifications_removes_expired() {
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info);

        // Simulate timer expiration (default timer is 30)
        for _ in 0..30 {
            state.tick_notifications();
        }

        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_tick_notifications_preserves_non_expired() {
        let mut state = AppState::new();
        state.show_notification("Test 1".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Info);

        // Tick once - timers should decrement but not expire
        state.tick_notifications();

        assert_eq!(state.notifications.len(), 2);
        assert!(state.notifications.iter().all(|n| n.timer < 30));
    }

    #[test]
    fn test_max_notifications_enforced() {
        let mut state = AppState::new();
        for i in 0..5 {
            state.show_notification(format!("Msg {}", i), NotificationType::Info);
        }

        // MAX_VISIBLE_NOTIFICATIONS = 3, oldest should be removed
        assert_eq!(state.notifications.len(), 3);
        // Verify oldest messages were removed
        assert!(state.notifications.iter().any(|n| n.message == "Msg 2"));
        assert!(state.notifications.iter().any(|n| n.message == "Msg 3"));
        assert!(state.notifications.iter().any(|n| n.message == "Msg 4"));
    }

    #[test]
    fn test_notification_log_preserves_all() {
        let mut state = AppState::new();
        state.show_notification("Msg 1".to_string(), NotificationType::Info);
        state.show_notification("Msg 2".to_string(), NotificationType::Error);

        // notification_log should have ALL notifications (not limited to MAX_VISIBLE_NOTIFICATIONS)
        assert_eq!(state.notification_log.len(), 2);
    }

    #[test]
    fn test_clear_notifications() {
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Error);

        state.clear_notifications();

        assert!(state.notifications.is_empty());
        // notification_log should still have entries
        assert_eq!(state.notification_log.len(), 2);
    }
}
```

#### 4.2 Test Execution

```bash
# Run notification tests only
cargo test notification

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

---

## Affected Files

| File | Changes |
|------|---------|
| `src/state/app_state.rs` | Add `ToastNotification` struct, change `notification` to `notifications`, add `tick_notifications()`, `clear_notifications()`, add unit tests |
| `src/ui/app.rs` | Remove `notification_timer` field and logic, update render method |
| `src/constants.rs` | Add `MAX_VISIBLE_NOTIFICATIONS` constant |

---

## Verification Checklist

- [ ] Multiple notifications display simultaneously
- [ ] Each notification expires independently
- [ ] Maximum 3 notifications visible
- [ ] `notification_log` preserves all notifications
- [ ] Build succeeds: `cargo build`
- [ ] Check succeeds: `cargo check`
- [ ] Clippy passes: `cargo clippy`
- [ ] Tests pass: `cargo test`

---

## Related Issues

- issue012: Notifications Sometimes Don't Disappear (fixed timer reset)

---

## Tags

- enhancement
- notification
- ui
- ux
