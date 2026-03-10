# issue030 - Startup Performance and UX Improvements

## Summary

The application has startup performance issues and missing error handling during initialization. These issues cause the app to freeze and provide poor user feedback.

## Problems Identified

### 1. Blocking Initialization (High Priority)

**Location**: `src/ui/app.rs:41-43`

```rust
let mut state = AppState::new();
state.sync_connection_state();  // Blocks - executes protonvpn command
state.refresh_servers();         // Blocks - waits for result
```

**Issue**: On startup, the app synchronously waits for:
1. `protonvpn` CLI to respond with connection status
2. Server list to be fetched from cache or network

This causes the app to freeze for several seconds, especially on slow systems or when VPN is not installed.

### 2. Error Handling Ignored (Medium Priority)

**Location**: `src/ui/app.rs:41-43`

**Issue**: Return values of `sync_connection_state()` and `refresh_servers()` are `io::Result`, but errors are silently ignored. Users don't know if initialization failed.

### 3. Hash Computation Overhead (Low Priority)

**Location**: `src/ui/app.rs:116-144`

```rust
fn compute_render_hash(&self) -> u64 {
    format!("{:?}", self.state.connection).hash(&mut hasher);
    format!("{:?}", self.state.input_mode).hash(&mut hasher);
    // ... more string formatting per loop iteration
}
```

**Issue**: Every event loop iteration computes a hash using `format!("{:?}", ...)` which allocates strings. This is wasteful when no state changes.

### 4. No Loading Indicator (Medium Priority)

**Issue**: Users see a blank screen during initialization with no feedback about what's happening.

## Proposed Solutions

### Solution 1: Async Initialization with Loading Screen

Instead of blocking initialization:

```rust
// Current (blocking)
let mut state = AppState::new();
state.sync_connection_state();
state.refresh_servers();

// Proposed (non-blocking + loading)
let mut state = AppState::new();
state.set_loading(true);  // Show loading indicator
// Background task fetches data
// UI updates when data arrives
```

**Benefits**:
- Immediate UI response
- Loading indicator shows progress
- Background fetch doesn't block

### Solution 2: Propagate Errors to User

```rust
// Show error notification if initialization fails
if let Err(e) = state.sync_connection_state() {
    state.show_notification(format!("Failed to check VPN status: {}", e), Error);
}
```

### Solution 3: Dirty Flag Optimization

Replace hash computation with a simple dirty flag:

```rust
// Instead of computing hash every frame
if self.is_dirty {
    terminal.draw(|f| self.render(f))?;
    self.is_dirty = false;
}

// Set dirty flag when state changes
fn update_servers(&mut self, servers: Vec<Server>) {
    self.servers = servers;
    self.is_dirty = true;
}
```

### Solution 4: Loading View

Create a minimal loading view shown during initialization:

```
┌─────────────────────────────────┐
│         ProtonVPN TUI           │
│                                 │
│      Loading servers...         │
│                                 │
│  [■───────────────] 40%         │
│                                 │
└─────────────────────────────────┘
```

## Priority

| Priority | Item | Effort |
|----------|------|--------|
| High | Async initialization with loading screen | Medium |
| Medium | Error propagation to UI | Low |
| Medium | Loading indicator on startup | Low |
| Low | Dirty flag optimization | Low |

## References

- Related issue: issue029 (similar timing issue)
- ratatui async patterns
