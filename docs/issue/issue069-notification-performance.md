# issue069: Notification log synchronous disk write

## Summary

Every notification triggers a synchronous disk write, causing potential UI lag during rapid events.

## Problem

In `src/state/notifications.rs:95`:

```rust
pub fn show(&mut self, message: String, notification_type: NotificationType, operation_key: Option<String>) {
    // ... notification logic ...
    
    log_persistence::save_notification_log(&self.notification_log);  // ← SYNC DISK WRITE
}
```

### Impact

1. **UI Blocking**: During rapid events (e.g., multiple cities loading simultaneously), each notification blocks the render loop
2. **Disk I/O**: File writes happen on the main thread (no async)
3. **No Batching**: If 5 notifications arrive in 10ms, 5 separate writes occur

### Example Scenario

```
User selects Japan → 
  1. "Refreshing servers..." → write ✓
  2. "Loading cities for JP..." → write ✓
  3. "Refreshing cities..." → write ✓
  4. "Loaded 15 cities" → write ✓
  5. "Connected to JP#374" → write ✓

5 disk writes in < 100ms
```

## Solution

### Option A: Batch writes with dirty flag

```rust
pub struct NotificationState {
    notifications: Vec<ToastNotification>,
    notification_log: Vec<Notification>,
    log_dirty: bool,  // ← New field
}

pub fn show(&mut self, ...) {
    // ... existing logic ...
    self.log_dirty = true;
}

pub fn flush_if_dirty(&mut self) {
    if self.log_dirty {
        log_persistence::save_notification_log(&self.notification_log);
        self.log_dirty = false;
    }
}
```

Then call `flush_if_dirty()` once per main loop iteration.

### Option B: Write on app shutdown only

For crash resilience, keep log in memory and write:
- On graceful shutdown
- On explicit user request (view logs)

### Option C: Async write with mpsc

Spawn a background thread that receives log entries via channel.

## Recommendation

**Option A** — Simple, effective, minimal code changes. Batching to once-per-loop is sufficient.

## Files Affected

- `src/state/notifications.rs` — Add `log_dirty` flag, remove sync write from `show()`
- `src/ui/app.rs` — Call `flush_if_dirty()` in main loop

## Severity

🟢 **LOW** — UX improvement, not functionally critical. Only noticeable during rapid operations.

## Labels

`performance` `ui` `optimization`
