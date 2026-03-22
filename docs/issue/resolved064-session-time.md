# issue064: Feature - Session time display in header

## Summary

Display elapsed time since successful connection in the header.

## Problem

The connection session duration is not visible in the UI.
Users must review logs to determine "how long have I been connected?"

However, the `connected_at` timestamp is already recorded in `ServerCache` but is not rendered in the UI.

## Current Implementation

### Data Source

**`src/vpn/cache.rs:26`** — connection timestamp already recorded:
```rust
pub connected_at: Option<DateTime<Utc>>,
```

**Set timing** (`cache.rs:68-73`):
```rust
pub fn set_connected(&mut self, server: String, ip: Option<String>, via: Option<String>) {
    self.connected_server = Some(server);
    self.connected_ip = ip;
    self.connected_via = via;
    self.connected_at = Some(Utc::now());  // ← recorded here
}
```

### Display Location

**`src/ui/app.rs:1140-1184`** — `render_header` Connected state handling:
- server, ip, loc, protocol are displayed
- session time is NOT displayed

## Solution

Display session duration in `HH:MM:SS` format in the header.

### Display Format

Human-readable format with dynamic units:
- Under 1 minute: `45s`
- Under 1 hour: `23m 45s`
- 1 hour+: `1h 23m`

### Display Mockup

```
JP#374  ip: 159.26.119.143  loc: Tokyo,Japan  protocol: UDP  session: 1h 23m
```

### Implementation Approach

1. Retrieve `connected_at` inside `render_header`
2. Calculate elapsed time with `Utc::now() - connected_at`
3. Format dynamically based on duration
4. Add to header after protocol

```rust
// Elapsed time formatting example
if let Some(connected_at) = self.state.vpn_state.get_connected_at() {
    let elapsed = Utc::now().signed_duration_since(connected_at);
    let secs = elapsed.num_seconds();
    let session_str = if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    };
    // render...
}
```

## Files Affected

- `src/ui/app.rs:1140-1184` — add session time rendering
- `src/vpn/client.rs` — add `get_connected_at()` accessor (via existing cache)

## Recommendation

1. Add `get_connected_at()` method to `VpnClient` (via `with_cache` to return `connected_at`)
2. In `render_header` Connected branch, calculate and render elapsed time
3. Position: after protocol
4. Format: human-readable with dynamic units (s, m s, h m)

## Severity

🟢 LOW — UX improvement, not functionally critical

## Labels

`feature` `ui` `connection`

---

# Implementation Results

## Status: ✅ COMPLETED

### Changes Made

#### `src/vpn/client.rs`

Added `get_connected_at()` method:

```rust
pub fn get_connected_at(&self) -> Option<chrono::DateTime<Utc>> {
    self.with_cache(|c| c.connected_at).ok().flatten()
}
```

#### `src/ui/renderers/header.rs`

Added session time display after protocol:

```rust
if let ConnectionState::Connected { .. } = state.connection_manager.connection {
    if let Some(connected_at) = state.vpn_state.get_connected_at() {
        let elapsed = Utc::now().signed_duration_since(connected_at);
        let secs = elapsed.num_seconds();
        let session_str = if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        };
        // Display in header
    }
}
```

#### `src/ui/app.rs`

Added 1-second refresh for session time display:

- Added `last_render_time: Instant` field to `TuiApp`
- Modified render condition to force re-render every second when connected

```rust
let is_connected = matches!(self.state.connection_manager.connection, ConnectionState::Connected { .. });
let elapsed = self.last_render_time.elapsed();
let should_update_session = is_connected && elapsed >= std::time::Duration::from_secs(1);

if !is_first_render || async_processed || notifications_expired || should_update_session {
    terminal.draw(|f| self.render(f))?;
    self.last_render_time = Instant::now();
}
```

### Display Example

```
● JP#374  ip: 159.26.119.143  loc: Tokyo,Japan  protocol: UDP  session: 1h 23m
```

### Behavior

- **New connection**: Timer starts from connection time
- **App restart with existing connection**: Timer continues from original connection time
- **Server change**: Timer resets
- **Disconnected**: Session time not displayed

### Verification

- ✅ `cargo check` — Compilation successful
- ✅ `cargo test` — 56 unit tests + integration tests passed
- ✅ `cargo clippy` — No warnings
