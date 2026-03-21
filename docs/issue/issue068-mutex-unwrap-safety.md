# issue068: Mutex unwrap() calls without error context

## Summary

Several `lock().unwrap()` calls on Mutexes lack error context, violating the "avoid unwrap()" coding standard.

## Problem

In `src/vpn/async_tasks.rs` and `src/state/connection_manager.rs`:

```rust
// async_tasks.rs:101
let mut queue = job_queue.lock().unwrap();  // No context on failure

// async_tasks.rs:103
if *shutdown.lock().unwrap() {  // No context

// connection_manager.rs:42
let mut pending = self.pending.lock().unwrap();  // No context

// connection_manager.rs:54
let (mut remaining, _) = self.condvar.wait_timeout(guard, duration).unwrap();  // No context
```

### What Can Go Wrong

Mutex poisoning occurs if a thread panics while holding the lock. While rare, it can happen:
- Async task panics during VPN operation
- Thread panics in worker pool

Without context, debugging is difficult.

## Solution

Replace `unwrap()` with `expect()` providing context:

```rust
// Before
let mut queue = job_queue.lock().unwrap();

// After
let mut queue = job_queue.lock()
    .expect("job_queue mutex poisoned");
```

Or for production robustness, handle the error:

```rust
let mut queue = job_queue.lock().map_err(|e| {
    tracing::error!("Mutex poisoned: {}", e);
    AppError::CommandFailed("Internal error: lock poisoned".into())
})?;
```

## Files Affected

- `src/vpn/async_tasks.rs` — Lines 101, 103, 106, 108
- `src/state/connection_manager.rs` — Lines 42, 48, 53, 54

## Severity

🟢 **LOW** — Defensive programming improvement. Mutex poisoning is rare but possible.

## Labels

`code-quality` `error-handling` `safety`
