# issue070: Connection statistics not tracked

## Summary

`ConnectionStats` type is defined but never stored or displayed in the UI.

## Problem

In `src/vpn/types.rs`, a `ConnectionStats` struct exists:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub connected_since: Option<DateTime<Utc>>,
}
```

However:
- No method populates this struct
- No UI displays bandwidth/traffic data
- The struct is unused

### Current State

| Feature | Status |
|---------|--------|
| Connected server name | ✅ Tracked |
| Connected IP | ✅ Tracked |
| Connected city/country | ✅ Tracked |
| Connection timestamp | ✅ Tracked (but not displayed) |
| Bytes received/sent | ❌ Not tracked |
| Session duration | ❌ Not displayed (see issue064) |
| Connection protocol | ✅ Tracked |

## Solution

### Step 1: Determine data source

Check if `protonvpn` CLI provides statistics:
```bash
protonvpn status  # Does this output bytes transferred?
```

Or read from system:
```bash
cat /sys/class/net/proton0/statistics/rx_bytes
cat /sys/class/net/proton0/statistics/tx_bytes
```

### Step 2: Populate `ConnectionStats`

```rust
impl VpnClient {
    pub fn get_connection_stats(&self) -> Option<ConnectionStats> {
        let rx_bytes = std::fs::read_to_string("/sys/class/net/proton0/statistics/rx_bytes")
            .ok()?
            .trim()
            .parse()
            .ok()?;
        let tx_bytes = std::fs::read_to_string("/sys/class/net/proton0/statistics/tx_bytes")
            .ok()?
            .trim()
            .parse()
            .ok()?;
        
        Some(ConnectionStats {
            bytes_received: rx_bytes,
            bytes_sent: tx_bytes,
            connected_since: self.with_cache(|c| c.connected_at).ok()?,
        })
    }
}
```

### Step 3: Display in UI

Add to header or connection status area:
```
JP#374  1.2 GB↓  345 MB↑  session: 2h 15m
```

## Files Affected

- `src/vpn/client.rs` — Add `get_connection_stats()` method
- `src/ui/app.rs` — Render stats in header
- `src/ui/render.rs` — Update header layout if needed

## Dependencies

- Requires `protonvpn status` to work OR sysfs access
- May depend on issue064 (session time display)

## Severity

🟢 **LOW** — Nice-to-have feature, not critical for VPN functionality.

## Labels

`feature` `ui` `connection` `statistics`
