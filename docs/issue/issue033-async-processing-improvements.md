# issue033 - Async Processing Improvements

## Summary

Async processing uses a custom thread pool with mpsc channels. While functional, there are design issues around polling delays, error handling patterns, and state management.

## Related Issues

- issue027: Code Review + Optimization (P1 implemented: Condvar)
- issue030: Startup Performance Improvements
- issue032: State Management Improvements

---

## Problems Identified

### 1. Polling Delay (Medium Priority) - CHOSEN APPROACH: mpsc + Condvar

**Location**: `src/ui/app.rs:88`

```rust
if event::poll(Duration::from_millis(100))? {
    // ... check async results via sync_connection_state()
}
```

**Issue**: Async results are checked only every 100ms during event polling. This means:
- Maximum 100ms delay before async results update the UI
- Connection state changes may not reflect immediately
- Wasteful when no input events occur

**Selected Solution**: Use `mpsc` + `Condvar` for event-driven wakeup

**Rationale**:
- No external dependencies (std only) - consistent with issue027 P1
- Extends existing ThreadPool pattern (already uses Condvar)
- Easier to understand than crossbeam

**Implementation Design**:

```rust
use std::sync::{Arc, Condvar, Mutex};

// Shared state between main thread and async receivers
struct AsyncNotifier {
    pending: Mutex<Vec<AsyncEvent>>,
    condvar: Condvar,
}

enum AsyncEvent {
    ServersRefreshed(Vec<Server>),
    Connected(String, Option<String>),
    Disconnected,
    CitiesLoaded(String, Vec<City>),
    Error(String),
}

// In AppState
struct AppState {
    // ... existing fields
    async_notifier: Arc<AsyncNotifier>,
}

// Spawn async task with notification
fn refresh_servers(&mut self) {
    let (tx, rx) = mpsc::channel();
    let notifier = Arc::clone(&self.async_notifier);

    self.async_manager.spawn_refresh_servers(self.vpn_state.clone(), tx);

    // Background thread notifies on completion
    std::thread::spawn(move || {
        let result = rx.recv().unwrap();
        let mut pending = notifier.pending.lock().unwrap();
        pending.push(match result {
            Ok(servers) => AsyncEvent::ServersRefreshed(servers),
            Err(e) => AsyncEvent::Error(e.to_string()),
        });
        notifier.condvar.notify_one();
    });
}

// In event loop - replace polling with wait
loop {
    // Wait for async event (with timeout for input handling)
    let (mut pending, timeout_result) = notifier
        .condvar
        .wait_timeout(notifier.pending.lock().unwrap(), Duration::from_millis(100))
        .unwrap();

    // Process pending events
    if !pending.is_empty() {
        let events = pending.drain(..).collect::<Vec<_>>();
        drop(pending);

        for event in events {
            self.handle_async_event(event);
        }
    }

    // Handle input events (always check, don't block only on async)
    if event::poll(Duration::from_millis(0))? {
        // ... handle input
    }
}
```

**Benefits**:
- Zero delay on async result (vs 100ms max)
- CPU efficient (no busy polling)
- Works with existing mpsc architecture

**Trade-offs**:
- Additional thread per async operation for notification
- Slightly more complex than current polling

---

### 2. Duplicate Error Handling in Job Execution

**Location**: `src/state/async_tasks.rs:108-158`

```rust
fn execute_job(job: Job) {
    match job {
        Job::RefreshServers { vpn_state, sender } => {
            let result = vpn_state.refresh_servers();
            if sender.send(result).is_err() {  // Repeated pattern
                tracing::warn!("Failed to send refresh result...");
            }
        }
        Job::Connect { vpn_state, server_id, sender } => {
            let result = vpn_state.connect(&server_id);
            if sender.send(result).is_err() {  // Same pattern
                tracing::warn!("Failed to send connect result...");
            }
        }
        // ... 4 more similar patterns
    }
}
```

**Issue**: Same `sender.send().is_err()` check repeated 6 times with identical warning message pattern.

**Proposed Solution**: Extract helper function:

```rust
fn execute_job(job: Job) {
    let result = match job {
        Job::RefreshServers { vpn_state, .. } => vpn_state.refresh_servers(),
        Job::Connect { vpn_state, server_id, .. } => vpn_state.connect(&server_id),
        // ...
    };
    Self::send_result(result, &sender);
}

fn send_result<T>(result: AsyncResult<T>, sender: &mpsc::Sender<AsyncResult<T>>) {
    if sender.send(result).is_err() {
        tracing::warn!("Failed to send async result - receiver dropped");
    }
}
```

---

### 3. Too Many Pending State Fields

**Location**: `src/state/app_state.rs:143-149`

```rust
pending_refresh: Option<ServerReceiver>,
pending_connect: Option<ConnectReceiver>,
pending_disconnect: Option<DisconnectReceiver>,
pub(crate) pending_cities: HashMap<String, CitiesReceiver>,
pending_connect_city: Option<ConnectReceiver>,
```

**Issue**: 
- 5 different pending receiver fields
- Each requires separate handling in `sync_connection_state()`
- `pending_cities` uses HashMap while others use Option - inconsistent

**Proposed Solution**: Consolidate into enum or struct:

```rust
struct PendingAsync {
    refresh: Option<ServerReceiver>,
    connect: Option<ConnectReceiver>,
    disconnect: Option<DisconnectReceiver>,
    cities: HashMap<String, CitiesReceiver>,
    connect_city: Option<ConnectReceiver>,
}

// Or use an enum for unified handling
enum AsyncTask {
    Refresh,
    Connect,
    Disconnect,
    Cities(String),
    ConnectCity,
}
```

---

### 4. Blocking Call in sync_connection_state

**Location**: `src/state/app_state.rs:340`

```rust
pub fn sync_connection_state(&mut self) -> bool {
    // Called every 100ms from event loop
    // ... checks all pending_* fields via try_recv()
}
```

**Issue**: Function name suggests "sync" but it's actually polling for async results. Name is misleading.

**Proposed Solution**: Rename to reflect actual behavior:
- `check_pending_async_results()`
- `poll_async_results()`

---

### 5. No Timeout on Async Operations

**Issue**: VPN operations have no timeout. If `protonvpn` CLI hangs:
- Connect could hang forever
- No way to cancel long-running operations

**Proposed Solution**: Add timeout handling:

```rust
use std::time::Duration;

pub fn spawn_connect_with_timeout(
    &self,
    vpn_state: Arc<VpnState>,
    server_id: String,
    timeout: Duration,
    sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
) {
    self.pool.submit(Job::ConnectWithTimeout {
        vpn_state,
        server_id,
        timeout,
        sender,
    });
}
```

---

## Status: Partially Implemented (2026-03-11)

### Implemented

- [x] Reduce polling delay: 100ms → 10ms using Condvar wait
- [x] Add AsyncNotifier struct (Mutex + Condvar)
- [x] Add AsyncEvent enum for event types
- [x] Add wait_for_async_events() method with Condvar
- [x] Update event loop to use event-driven waiting

### Not Implemented

- [ ] Full event-driven notification (mpsc::Receiver not Clone)
- [ ] Extract duplicate error handling
- [ ] Consolidate pending state fields
- [ ] Rename sync_connection_state
- [ ] Add timeout support

---

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| **High** | Reduce polling delay (mpsc + Condvar) | Medium | ✅ Done (10ms polling) |
| Low | Extract duplicate error handling | Low | Pending |
| Medium | Consolidate pending state fields | Medium | Pending |
| Low | Rename sync_connection_state | Low | Pending |
| Low | Add timeout support | Medium | Pending |

---

## Architecture Suggestion: Event-Driven Async (IMPLEMENTED)

**Selected Approach**: mpsc + Condvar (std only)

**Before** (100ms polling):
```
[Event Loop] --100ms--> [check try_recv()]
                              |
                              v
                        [update state]
```

**After Implementation** (10ms polling with Condvar):
```
[Event Loop] --10ms--> [wait_for_async_events]
                              |
                              v
                        [update state]
```

**Note**: Full event-driven notification not implemented due to mpsc::Receiver not being Clone.
To implement full event-driven, use crossbeam channel or Arc-based solution.

### Implementation Steps

1. **Add AsyncNotifier to AppState**:
   ```rust
   struct AsyncNotifier {
       pending: Mutex<Vec<AsyncEvent>>,
       condvar: Condvar,
   }
   ```

2. **Modify async task spawns** to notify on completion:
   - Wrap existing mpsc channel receive in background thread
   - Signal Condvar when result arrives

3. **Replace polling in event loop**:
   - Use `condvar.wait_timeout()` with 100ms max
   - Process all pending events immediately when notified

### Comparison with Alternatives

| Approach | Pros | Cons |
|----------|------|------|
| **mpsc + Condvar** (chosen) | No deps, extends existing pattern | Extra thread per task |
| crossbeam select! | Native multi-channel support | External dep, steeper learning |
| Atomic flag | Simple | Busy-wait risk, imprecise |

### Notes

- This approach is consistent with issue027 P1 (ThreadPool already uses Condvar)
- Maintains compatibility with existing mpsc architecture
- Minimal external dependencies
- 10ms polling provides near-immediate response (10x improvement from 100ms)

### Files Changed

- `src/state/app_state.rs`: Added AsyncNotifier, AsyncEvent, wait_for_async_events()
- `src/ui/app.rs`: Updated event loop to use wait_for_async_events(10ms)

---

## References

- issue027 P1: ThreadPool busy-wait fix (implemented with Condvar)
- Rust mpsc documentation
- crossterm event handling
