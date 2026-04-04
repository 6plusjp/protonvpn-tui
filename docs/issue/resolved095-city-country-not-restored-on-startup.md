# Resolved 095: city/country Not Restored on Startup

**Status**: ✅ Resolved
**Resolved**: 2026-04-02
**Related**: Issue 093 (Session time persistence), Issue 094 (Cache sync fix)

---

## Summary

After reboot with VPN connected, the header displayed server name and IP but not city/country/via (for Secure Core) because the CLI's persistence file doesn't contain this information.

## Problem Description

When the app starts with an existing VPN connection, `sync_connection_state()` reads from `connection_persistence.json` which only contains:
- `server_name` (e.g., "JP#374")
- `server_ip` (e.g., "159.26.119.144")
- `protocol` (e.g., "wireguard")

Missing:
- `city` (e.g., "Tokyo")
- `country` (e.g., "Japan")
- `via` (e.g., "Switzerland" for Secure Core)

### Header Display After Startup

| Phase | Server | IP | Location (city/country/via) |
|-------|--------|-----|----------------------------|
| After connect | CH-JP#2 | 37.19.x.x | Tokyo, via Switzerland |
| After reboot (Before Fix) | CH-JP#2 | 37.19.x.x | **Not displayed** |
| After reboot (After Fix) | CH-JP#2 | 37.19.x.x | Tokyo, CH |

## Root Cause

The CLI's `connection_persistence.json` is managed by Proton's Python backend (`python-proton-vpn-api-core`) and only stores minimal connection info. The `city`, `country`, `via` fields are not included.

## Solution

Use `protonvpn status` command to restore location information on startup.

### Implementation

1. **`StatusInfo` struct** in `types.rs`:
   ```rust
   pub struct StatusInfo {
       pub server: Option<String>,
       pub city: Option<String>,
       pub country: Option<String>,
       pub protocol: Option<String>,
       pub uptime: Option<Duration>,
   }
   ```

2. **`parse_status_output()` function** in `types.rs`:
   - Parses multiple CLI output formats:
     - Format 1: `Location: Tokyo, Japan`
     - Format 2: `City: Tokyo` + `Country: Japan`
   - Falls back to extracting country code from server ID

3. **`extract_country_code()` function** in `types.rs`:
   - Extracts country code from server ID (e.g., "JP#374" → "JP", "CH-JP#2" → "CH")

4. **`get_status_info()` method** in `client.rs`:
   - Calls `protonvpn status` and parses output

5. **`adjust_connected_at_from_uptime()` method** in `client.rs`:
   - Exists for future CLI versions that may include uptime
   - Currently returns `None` since CLI doesn't output uptime
   - Falls back to boot_time validation for pre-reboot timestamps

6. **`sync_connection_state()` update** in `event_handler.rs`:
   - Gets status info from `protonvpn status`
   - Adjusts connected_at if uptime is available
   - Falls back to persistence file if status unavailable

### Data Flow (After Fix)

```
App startup:
  VpnClient::new()
    → cache.load() from disk
    → adjust_connected_at_on_startup() ← Validates/adjusts connected_at

  sync_connection_state()
    → get_status_info() ← Gets city, country from protonvpn status
    → adjust_connected_at_from_uptime() ← Updates if uptime available
    → sync_cache_with_connection() ← Syncs server/IP, preserves connected_at
    → ConnectionState::Connected { server, ip, city, country, via: None }
```

### Limitations

- `via` (entry country for Secure Core) is not in status output - this is a CLI limitation
- Country code (e.g., "JP") is used instead of full name (e.g., "Japan") when location is not available

## Files Changed

| File | Changes |
|------|---------|
| `src/vpn/types.rs` | Added `StatusInfo`, `parse_status_output()`, `extract_country_code()` |
| `src/vpn/client.rs` | Added `get_status_info()`, `adjust_connected_at_from_uptime()` |
| `src/state/event_handler.rs` | Updated `sync_connection_state()` to use status info |

## Test Coverage

- `parse_status_output()`: 7 tests covering various formats
- `parse_status_uptime()`: 7 tests covering uptime parsing
- `extract_country_code()`: 4 tests covering country code extraction
- All 122 tests pass, no clippy warnings

## Verification Steps

1. Connect to VPN (city/country shown in header)
2. Reboot computer
3. Launch ProtonVPN TUI
4. Verify city/country is displayed from `protonvpn status`

## Dependencies

- Issue 093: Session time fix ✅
- Issue 094: Cache sync fix ✅
- ProtonVPN CLI v0.1.8+ (for `protonvpn status` command)
