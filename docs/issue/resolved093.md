# Issue 093: Session Time Persists Across Computer Reboot

## Summary

VPN connection session time continues to display after a computer reboot, showing incorrect elapsed time.

## Problem Description

After rebooting the computer and launching ProtonVPN TUI, the session time is calculated from the pre-reboot connection timestamp, resulting in an incorrectly long session duration.

## Root Cause

The `connected_at` timestamp is stored in a persisted cache file (TOML format), so the old timestamp survives reboots.

### Data Flow

1. **Session start time stored** (`src/vpn/cache.rs` line 26):
   ```rust
   pub struct ServerCache {
       pub connected_at: Option<DateTime<Utc>>,  // ← persisted to disk
   }
   ```

2. **Cache written to disk** (`src/vpn/cache.rs` lines 44-57):
   - `save()` method writes TOML to `~/.cache/protonvpn-tui/server_cache.toml`

3. **Cache loaded on startup** (`src/vpn/client.rs` lines 57-63):
   ```rust
   let cache = match ServerCache::load(cache_path.clone()) {
       Ok(c) => c,  // ← old connected_at is loaded
       // ...
   };
   ```

4. **Session time calculation** (`src/ui/renderers/header.rs` lines 111-118):
   ```rust
   if let Some(connected_at) = state.vpn_state.get_connected_at() {
       let elapsed = Utc::now().signed_duration_since(connected_at);
       // ↑ calculated from pre-reboot timestamp → incorrect
   }
   ```

## Affected Files

| File | Role |
|------|------|
| `src/vpn/cache.rs` | `connected_at` definition and persistence |
| `src/vpn/client.rs` | Cache loading, `get_connected_at()` |
| `src/ui/renderers/header.rs` | Session time display |

## Expected Behavior

After reboot, the session time should either reset or display the correct connection time if VPN is actually connected.

## Proposed Solutions

### Option A: Use `protonvpn status` uptime (Recommended)

`protonvpn status` (v0.1.8+) outputs `Uptime: 00:15:32` which is the actual session time.

**⚠️ IMPORTANT NOTE (2026-04-02):** The current ProtonVPN CLI (proton-vpn-cli) does NOT include uptime/time field in `protonvpn status` output. Confirmed output format:
```
Status: Connected
Server: JP#443 in Tokyo, Japan
Load: 15%
Protocol: wireguard
```
Option A is implemented but will always return `None` with current CLI. Option B (boot time validation) is the actual fallback used.

**Implementation:**
```rust
// Parse uptime from protonvpn status
pub fn parse_status_uptime(output: &str) -> Option<Duration> {
    for line in output.lines() {
        if let Some(uptime_str) = line.strip_prefix("Uptime:") {
            // Parse "00:15:32" format
            let parts: Vec<&str> = uptime_str.trim().split(':').collect();
            if parts.len() == 3 {
                let hours: u64 = parts[0].parse().ok()?;
                let mins: u64 = parts[1].parse().ok()?;
                let secs: u64 = parts[2].parse().ok()?;
                return Some(Duration::from_secs(hours * 3600 + mins * 60 + secs));
            }
        }
    }
    None
}

// Calculate connected_at from uptime
fn connected_at_from_uptime(uptime: Duration) -> DateTime<Utc> {
    Utc::now() - chrono::Duration::from_std(uptime).unwrap_or_default()
}
```

**Benefits:**
- Uses actual CLI data (authoritative source)
- No boot time calculation needed
- Works with auto-connect
- Accurate session time

**Drawbacks:**
- Requires CLI call on startup
- CLI may be unavailable

### Option B: Validate `connected_at` against system boot time

Fallback if CLI is unavailable. Check if `connected_at` is before system boot time.

**Implementation:**

```rust
fn system_boot_time() -> Option<DateTime<Utc>> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    for line in content.lines() {
        if let Some(btime_str) = line.strip_prefix("btime ") {
            let secs: i64 = btime_str.trim().parse().ok()?;
            return DateTime::from_timestamp(secs, 0);
        }
    }
    None
}

impl ServerCache {
    pub fn validate_after_boot(&mut self) {
        if let Some(connected_at) = self.connected_at {
            if let Some(boot_time) = system_boot_time() {
                if connected_at < boot_time {
                    self.connected_at = Some(boot_time);
                }
            }
        }
    }
}
```

### Recommended Flow

```
Startup:
  if is_connected() {
    run "protonvpn status"
    if status outputs uptime {
      connected_at = now - uptime  // Option A
    } else {
      cache.validate_after_boot()  // Option B
    }
  }
```

## Verification Steps

1. Connect to VPN
2. Verify session time is displayed
3. Reboot computer
4. Launch ProtonVPN TUI
5. Verify session time is calculated from boot time (not from pre-reboot)

## Priority

High - incorrect information displayed to user

## Related

- Issue 094: Cache not synced with actual connection on startup
- Issue 095: city/country not restored on startup
- `src/vpn/cache.rs` - ServerCache struct
- `src/vpn/client.rs` - VpnClient initialization
- `src/ui/renderers/header.rs` - Header rendering
