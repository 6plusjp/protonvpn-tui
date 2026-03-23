# issue070: Connection statistics not tracked

## Summary

`ConnectionStats` type was defined but never stored or displayed in the UI.

## Problem

In `src/vpn/types.rs`, a `ConnectionStats` struct existed but was unused:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub server_ip: String,
    pub protocol: String,
}
```

However:
- No method populated this struct
- No UI displayed bandwidth/traffic data
- The struct was unused
- `connected_at`, `server_ip`, `protocol` were already tracked elsewhere

### Current State (Before Implementation)

| Feature | Status |
|---------|--------|
| Connected server name | ✅ Tracked |
| Connected IP | ✅ Tracked |
| Connected city/country | ✅ Tracked |
| Connection timestamp | ✅ Tracked |
| Bytes received/sent | ❌ Not tracked |
| Session duration | ✅ Displayed (issue064) |
| Connection protocol | ✅ Tracked |

## Solution

### Step 1: Data source

`protonvpn status` does not exist in the new CLI. Only sysfs is available:

```bash
cat /sys/class/net/proton0/statistics/rx_bytes
cat /sys/class/net/proton0/statistics/tx_bytes
```

### Step 2: Design decisions

1. **Delete `ConnectionStats`** — Unused struct with fields already tracked elsewhere
2. **Return tuple `(u64, u64)`** — Minimal, no wrapper struct needed
3. **5-second update interval** — Reduced from 1s to avoid unnecessary UI updates
4. **Session time: minute-only** — Changed from showing seconds to minutes only

### Step 3: Implementation

Add `get_connection_stats()` to `VpnClient`:

```rust
pub fn get_connection_stats(&self) -> Option<(u64, u64)> {
    let base = std::path::Path::new("/sys/class/net/proton0/statistics");
    if !base.exists() {
        return None;
    }
    let rx = std::fs::read_to_string(base.join("rx_bytes")).ok()?;
    let tx = std::fs::read_to_string(base.join("tx_bytes")).ok()?;
    let bytes_received = rx.trim().parse().ok()?;
    let bytes_sent = tx.trim().parse().ok()?;
    Some((bytes_received, bytes_sent))
}
```

### Step 4: Display in UI

Add to header after session time:

```
● JP#374  ip:159.26.119.143  loc:Tokyo,Japan  protocol:UDP  session:2h 15m  ↓1.2GB  ↑345MB
```

Format: human-readable with SI prefixes (B/KB/MB/GB)

---

## Implementation Results

### Status: ✅ COMPLETED

### Changes Made

#### `src/vpn/types.rs`

- Deleted `ConnectionStats` struct and its `Default` impl (unused)

#### `src/vpn/client.rs`

Added `get_connection_stats()` method:

```rust
pub fn get_connection_stats(&self) -> Option<(u64, u64)> {
    let base = std::path::Path::new("/sys/class/net/proton0/statistics");
    if !base.exists() {
        return None;
    }
    let rx = std::fs::read_to_string(base.join("rx_bytes")).ok()?;
    let tx = std::fs::read_to_string(base.join("tx_bytes")).ok()?;
    let bytes_received = rx.trim().parse().ok()?;
    let bytes_sent = tx.trim().parse().ok()?;
    Some((bytes_received, bytes_sent))
}
```

#### `src/ui/renderers/header.rs`

Added bytes display after session time:

```rust
if let Some((bytes_received, bytes_sent)) = state.vpn_state.get_connection_stats() {
    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        if bytes >= GB {
            format!("{:.1}GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.0}MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.0}KB", bytes as f64 / KB as f64)
        } else {
            format!("{}B", bytes)
        }
    }
    status_spans.push(Span::styled("↓", Style::default().fg(theme.dim)));
    status_spans.push(Span::styled(format_bytes(bytes_received), ...));
    status_spans.push(Span::styled("↑", Style::default().fg(theme.dim)));
    status_spans.push(Span::styled(format_bytes(bytes_sent), ...));
}
```

#### `src/ui/app.rs`

Changed render interval from 1s to 5s:

```rust
let should_update_session =
    is_connected && elapsed >= std::time::Duration::from_secs(5);
```

#### Session time format change (related to issue064)

Changed to minute-only display:

```rust
let mins = elapsed.num_minutes();
let session_str = if mins < 60 {
    format!("{}m", mins.max(1))
} else {
    format!("{}h {}m", mins / 60, mins % 60)
};
```

### Display Example

```
● JP#374  ip:159.26.119.143  loc:Tokyo,Japan  protocol:UDP  session:2m  ↓1.2GB  ↑345MB
```

### Verification

- ✅ `cargo check` — Compilation successful
- ✅ `cargo test` — 168 tests passed
- ✅ `cargo clippy` — No warnings

## Files Affected

- `src/vpn/types.rs` — Deleted `ConnectionStats`
- `src/vpn/client.rs` — Added `get_connection_stats()`
- `src/ui/renderers/header.rs` — Added bytes display, changed session time format
- `src/ui/app.rs` — Changed render interval to 5s

## Dependencies

- Requires sysfs access (`/sys/class/net/proton0/statistics/`)
- Session time: depends on issue064 (already resolved)

## Severity

🟢 **LOW** — Nice-to-have feature, not critical for VPN functionality.

## Labels

`feature` `ui` `connection` `statistics` `completed`
