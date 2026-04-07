# Resolved 096: Session Time Incorrect - Same Server Reconnection

**Status**: ✅ Resolved  
**Resolved**: 2026-04-04

---

## Summary

> **Important**: `protonvpn status` does NOT include uptime field in current CLI versions (v0.1.8+). The uptime parsing code exists but will always return `None`.

When connecting to the same server twice (already connected), the session time showed incorrect value (reset to boot time). Also, session was reset on every app startup.

## Problem 1: Same Server Reconnection

### Root Cause

**Location**: `src/vpn/client.rs` - `update_connected_at()` method

```rust
// BEFORE (buggy)
if server_changed || ip_changed {
    cache.connected_at = Some(boot_time);  // ← BUG: used boot_time instead of Utc::now()
}
```

## Problem 2: Session Reset on Startup

### Root Cause

`sync_connection_from_vpn()` was called on every startup without checking if the server/IP actually changed:

```rust
// BEFORE (buggy)
if self.vpn_state.is_connected() {
    if let Some((server, ip)) = ... {
        self.vpn_state.update_connected_at(&server, &ip);  // ← always called
    }
}
```

This caused `connected_at` to be reset to `Utc::now()` even when the same server was already connected.

## Solution

### Fix 1: Update `update_connected_at()` to use `Utc::now()`

**File**: `src/vpn/client.rs`

```rust
// AFTER (fixed)
if server_changed || ip_changed {
    cache.connected_at = Some(chrono::Utc::now());  // Use current time
}
```

### Fix 2: Check server/IP before updating in `sync_connection_from_vpn()`

**File**: `src/state/app_state_impl.rs`

```rust
// AFTER (fixed)
pub fn sync_connection_from_vpn(&mut self) {
    if is_connected {
        if let Some((server, ip)) = get_connected_server_info() {
            let current_server = self.vpn_state.get_connected_server();
            let current_ip = self.vpn_state.get_connected_ip();

            let server_changed = current_server.as_deref() != Some(&server);
            let ip_changed = current_ip.as_deref() != Some(&ip);

            if server_changed || ip_changed {
                self.vpn_state.update_connected_at(&server, &ip);
            } else {
                // Same server/ip → preserve session
            }
        }
    } else {
        self.vpn_state.clear_connected_at();
    }
}
```

### Fix 3: Added helper methods

**File**: `src/vpn/client.rs`

- Added `get_connected_server()` - returns cached server name
- Added `get_connected_ip()` - returns cached IP
- Added `clear_connected_at()` - clears connected_at when disconnected

## Session Update Logic

| Condition | connected_at |
|-----------|--------------|
| `is_connected() = false` | `None` (cleared) |
| `is_connected() = true` + same server/IP as cache | **Preserved** |
| `is_connected() = true` + different server/IP | `Utc::now()` (new session) |

## Files Changed

| File | Changes |
|------|---------|
| `src/vpn/client.rs` | `update_connected_at()` uses `Utc::now()`, added `get_connected_server()`, `get_connected_ip()`, `clear_connected_at()` |
| `src/state/app_state_impl.rs` | `sync_connection_from_vpn()` now checks server/IP before updating |

## Related Issues

- Issue 093: Session Time Persists Across Computer Reboot (resolved)
- Issue 094: Cache Not Synced with Actual Connection on Startup (resolved)
- Issue 095: city/country not restored on startup (resolved)

## Note

`protonvpn status` does NOT include uptime field in current CLI versions (v0.1.8+). The code to parse uptime exists (`parse_status_uptime()`) but will always return `None`. Session time is calculated from `connected_at` stored in the app's cache, or falls back to `boot_time` for pre-reboot timestamps.