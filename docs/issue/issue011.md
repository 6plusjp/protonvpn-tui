# issue011: Notifications Sometimes Don't Disappear

## Summary

Notifications sometimes stay visible indefinitely or disappear too quickly after async operations complete.

## Symptom

- User performs an action (connect, disconnect, refresh, etc.)
- Initial notification appears correctly (e.g., "Connecting...")
- After the async operation completes, a new notification appears (e.g., "Connected to X")
- This new notification either:
  - Disappears almost immediately (within a few hundred milliseconds)
  - Or stays visible indefinitely

## Root Cause

**Problem 1: Timer not reset when `sync_connection_state()` shows notifications**

In `src/ui/app.rs`, the main loop:
```rust
loop {
    terminal.draw(|f| self.render(f))?;
    self.state.sync_connection_state();  // Can call show_notification()
    
    if self.notification_timer > 0 {
        self.notification_timer -= 1;
        if self.notification_timer == 0 {
            self.state.clear_notification();
        }
    }
    // ...
}
```

`sync_connection_state()` is called BEFORE the timer check. When it shows a new notification (e.g., when async connection completes), it does NOT reset `notification_timer`.

If the timer was already close to 0 when the new notification appears, it will be cleared almost immediately.

**Problem 2: `notification_timer` only reset in key handlers**

The timer is only reset in key event handlers (lines 118, 124, 133, 148, 173, 196, 201, 206, 211, 292, 333, 338 in `src/ui/app.rs`). Notifications triggered by `sync_connection_state()` don't reset it.

**Problem 3: No mechanism to reset timer from AppState**

`AppState::show_notification()` has no access to `TuiApp::notification_timer`. There's no way for async completion handlers to reset the display timer.

## Affected Code Paths

All async operations that call `show_notification()` via `sync_connection_state()`:
- Connection completion (`src/state/app_state.rs:274`)
- Connection failure (`src/state/app_state.rs:292`)
- Disconnect completion (`src/state/app_state.rs:310`)
- Disconnect failure (`src/state/app_state.rs:323`)
- Server refresh completion (`src/state/app_state.rs:251`)
- Server refresh failure (`src/state/app_state.rs:257`)
- Cities load completion (`src/state/app_state.rs:342`)
- Cities load failure (`src/state/app_state.rs:348`)
- City connection completion (`src/state/app_state.rs:364`)
- City connection failure (`src/state/app_state.rs:381`)

## Proposed Solution

**Option A: Add timer reset parameter to show_notification**

Modify `AppState::show_notification()` to return whether a notification was shown, then reset timer in `TuiApp`:
```rust
// In AppState
pub fn show_notification(&mut self, ...) -> bool {
    self.notification = Some(...);
    // ...
    return true;  // Notification was shown
}

// In TuiApp main loop
let notification_shown = self.state.sync_connection_state();
if notification_shown {
    self.notification_timer = NOTIFICATION_TIMER_DEFAULT;
}
```

**Option B: Move timer management entirely to AppState**

Store `notification_timer` in `AppState` instead of `TuiApp`, and manage it alongside notification state.

**Option C: Reset timer in sync_connection_state return value**

Have `sync_connection_state()` return a boolean indicating whether it showed a notification, and reset timer accordingly.

## Related Code

- `src/ui/app.rs:26` - `notification_timer: u8` field
- `src/ui/app.rs:67-71` - Timer decrement and clear logic
- `src/state/app_state.rs:226-238` - `show_notification()` method
- `src/state/app_state.rs:244-404` - `sync_connection_state()` method
- `src/constants.rs:2-3` - Timer constants (30 = 3 seconds at 100ms tick)

## Tags

- bug
- notification
- async
- timing
- ui
