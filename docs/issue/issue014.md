# issue014: Stacked Toast Notifications

## Summary

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

## Implementation Plan

### Phase 1: Data Structure Changes

1. **Replace `notification: Option<Notification>` with `notifications: Vec<Notification>`**
   - Add individual timers per notification

2. **Create `ToastNotification` struct:**
   ```rust
   pub struct ToastNotification {
       pub message: String,
       pub notification_type: NotificationType,
       pub timer: u8,           // Individual countdown
       pub created_at: u32,      // For ordering
   }
   ```

### Phase 2: Timer Management

1. **Move timer logic to AppState**
   - Each notification has independent timer
   - Timers decrement in main loop

2. **Remove notification_timer from TuiApp**
   - Fully managed in AppState

### Phase 3: UI Rendering

1. **Modify `render_notification_popup` to render stacked toasts**
   - Calculate position for each notification
   - Apply slide-up offset based on age/index

2. **Update constants:**
   - `MAX_VISIBLE_NOTIFICATIONS: usize = 3`
   - Keep existing timer values

### Phase 4: Cleanup

1. **Ensure notification_log still captures all notifications**
2. **Handle edge case: notification cleared externally**

## Affected Files

- `src/state/app_state.rs` - Data structure, timer logic
- `src/ui/app.rs` - Main loop, render logic
- `src/ui/views/logs_view.rs` - If notification log display needs update
- `src/constants.rs` - Add `MAX_VISIBLE_NOTIFICATIONS`

## Related Issues

- issue012: Notifications Sometimes Don't Disappear (fixed timer reset)

## Tags

- enhancement
- notification
- ui
- ux
