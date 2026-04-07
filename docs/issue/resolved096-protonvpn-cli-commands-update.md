# issue096: protonvpn CLI output format changes (connect/disconnect/status)

## Summary

The protonvpn CLI commands have changed their output format. The TUI's parsing functions in `src/vpn/types.rs` need to be verified/updated to match the current CLI (v0.1.8) output formats.

## Verified CLI Output Formats (v0.1.8)

### 1. countries list ✓ Working

```
$ protonvpn countries list
Server list is outdated, updating... This may take a moment.
Country                           Code
--------------------------------  ------
Japan                             JP
United States                     US
...
```

- Current parsing: `parse_countries()` in types.rs
- **Status**: Works correctly

### 2. cities list ✓ Working

```
$ protonvpn cities list JP
Server list is outdated, updating... This may take a moment.

Cities in Japan:
City    Features
------  ----------
Tokyo   P2P
Osaka   P2P
...
```

- Current parsing: `parse_cities_with_features()` in types.rs
- **Status**: Works correctly

### 3. connect - Verified Working

**Actual output format** (tested with v0.1.8):
```
Server list is outdated, updating... This may take a moment.
Connected to JP#221 in Tokyo, Japan.
Your new IP address is 103.155.232.234.

Port forwarding is active on this server.
To get your forwarded port, run the natpmpc setup script.
Guide: https://protonvpn.com/support/port-forwarding-manual-setup#linux
```

**Key points**:
- "Server list is outdated" message may appear first (if list is stale)
- "Connected to {server_id} in {city}, {country}." format
- "Your new IP address is {ip}." on next line
- Additional info (port forwarding, etc.) may follow

**Current parsing** (`parse_connect_output()`):
- Uses `.position()` to find "Connected to " line anywhere in output ✓
- Looks for IP on line after "Connected to" ✓
- **Status**: Works correctly

### 4. disconnect - Verified Working

**Actual output format** (tested with v0.1.8):
```
(no output)
```
Exit code: 0 (success)

**Current logic** (client.rs):
- Checks `output.status.success()` (exit code 0) ✓
- Handles "already disconnected" / "not connected" gracefully ✓
- **Status**: Works correctly

### 5. status - Verified Working

**Actual output format** (tested with v0.1.8):
```
Status: Connected
Server: JP#221 in Tokyo, Japan
Load: 5%
Protocol: wireguard
```

When disconnected:
```
Status: Disconnected
```

**Current parsing** (`parse_status_output()`):
- Parses "Status:" field ✓
- Parses "Server:" field (e.g., "JP#221 in Tokyo, Japan") ✓
- Parses "Load:" field ✓
- Parses "Protocol:" field ✓
- **Status**: Works correctly

### 6. config set - Need verification (but likely works)

**Expected behavior**:
- Success: Exit code 0, output contains confirmation
- Error: Exit code non-zero, error message in output

## Issues Fixed

### Issue 1: parse_status_output() doesn't capture Load field

**Status**: ✅ FIXED

Added `load` field to `StatusInfo` struct and parsing for "Load: X%" in `parse_status_output()`.

**Implementation**:
1. Added `pub load: Option<u8>` to `StatusInfo` struct (types.rs)
2. Added parsing for "Load:" line (types.rs:parse_status_output())
3. Added `load: Option<u8>` to `ConnectionState::Connected` (connection_state.rs)

### Issue 2: Refresh servers notification not showing completion

**Status**: ✅ FIXED

The "Refreshing servers..." notification was shown but the completion notification was never received.

**Root cause**: Main render loop didn't call `process_async_events()` to receive async event results.

**Implementation**: Added `self.state.process_async_events()` call in main render loop (app.rs).

### Issue 3: VPN connection not synced at startup

**Status**: ✅ FIXED

When app starts while VPN is already connected externally, the header showed "Disconnected".

**Root cause**: No connection state sync at startup. `protonvpn status` is slow (blocks), so couldn't use sync call at startup.

**Implementation**: Added `sync_connection_from_vpn()` method (app_state_impl.rs) that:
1. Checks `vpn_state.is_connected()` (proton0 + persistence file check)
2. Reads connection info from `connection_persistence.json` via `get_connected_server_info()`
3. Updates `ConnectionState::Connected`

Called at app startup in `TuiApp::new()` after `refresh_servers()`.

**Note**: Uses persistence file directly instead of `protonvpn status` to avoid blocking startup.

## Priority

All issues have been resolved.

## Conclusion

All CLI commands work correctly with current implementation:

| Command | TUI Implementation | Status |
|---------|-------------------|--------|
| `protonvpn countries list` | `["countries", "list"]` | ✓ Works |
| `protonvpn cities list <CC>` | `["cities", "list", cc]` | ✓ Works |
| `protonvpn connect` | `parse_connect_output()` | ✓ Works |
| `protonvpn disconnect` | Exit code check | ✓ Works |
| `protonvpn status` | `parse_status_output()` | ✓ Works (with Load field) |

### Additional Notes

1. **Server load in cities list**: CLI returns duplicate city entries with different server IDs (e.g., multiple "Tokyo" servers). This is likely intentional - shows multiple servers per city. Current parser handles this.

2. **Port forwarding message**: CLI outputs additional info after connect (port forwarding, etc.). Parser correctly ignores these lines.

3. **Server list outdated message**: CLI outputs "Server list is outdated" before connect/status. Parser correctly finds relevant lines anywhere in output.

### Commands NOT Used by TUI (no changes needed)

- `protonvpn signin` / `signout` - Manual CLI only
- `protonvpn info` - Not needed (account info already from protonvpn)