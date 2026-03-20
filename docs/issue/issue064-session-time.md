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
