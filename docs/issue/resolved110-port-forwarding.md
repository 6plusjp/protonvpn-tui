# issue110-port-forwarding-integration.md

## Summary

Display the forwarded port number in the header when connected via VPN, and automatically update torrent client (qBittorrent only) configuration when the port changes.

## Background

ProtonVPN supports port forwarding (via `protonvpn config set port-forwarding on`). When enabled, the VPN server assigns a forwarded port number, which is stored in:

```
$XDG_RUNTIME_DIR/Proton/VPN/forwarded_port
# or: /run/user/<uid>/Proton/VPN/forwarded_port
```

This port needs to be:
1. Displayed in the TUI header (when connected)
2. Synced to torrent clients to avoid manual reconfiguration

## Important Notes

### protonvpn status Issue

Running `protonvpn status` clears the forwarded_port file when the VPN connection is managed by the protonvpn daemon. This is a known limitation:

- The file is written by the VPN daemon when connected
- Running `protonvpn status` triggers a refresh that clears the file
- The file is repopulated after the refresh completes

**Solution**: Read the port file only once during connection, cache it, and avoid reading again until next connection. Do NOT read the file on every render cycle.

### Server ID Validation

The forwarded port is cached with the server_id as a pair. When restoring connection state:

- If cached server_id matches current server_id → restore cached port
- If server_id differs → clear cached port (prevents showing stale port for different server)

This prevents showing wrong port when:
1. App restarts after connecting to server A
2. User reconnects to server B (different port)
3. Port displayed should be server B's port, not server A's cached port

## Implementation

### Files Modified

| File | Changes |
|------|---------|
| `src/paths.rs` | Added `proton_runtime_dir()`, `proton_forwarded_port_path()` |
| `src/vpn/cache.rs` | Added `forwarded_port` field to `ServerCache` |
| `src/vpn/client.rs` | Added `get_forwarded_port()`, `set_forwarded_port()`, `get_cached_forwarded_port()` |
| `src/vpn/torrent_sync.rs` | Added `update_qbittorrent_port()`, `sync_forwarded_port()` |
| `src/state/event_handler.rs` | Read port and sync on connect in background thread |
| `src/ui/renderers/header.rs` | Display port in header (connected only), protocol display |

### Port Display

- Displayed in header when `ConnectionState::Connected`
- Read from cache (`get_cached_forwarded_port()`), NOT directly from file
- Server_id validation ensures port matches current connection

### Torrent Sync

On connection:
1. Read forwarded port from file in background thread
2. Cache port with server_id (`vpn_state.set_forwarded_port()`)
3. Update qBittorrent config: `~/.config/qBittorrent/qBittorrent.conf` - `Session\Port`

### Config Locations

- **ProtonVPN**: `$XDG_RUNTIME_DIR/Proton/VPN/forwarded_port` (read-only, managed by VPN daemon)
- **qBittorrent**: `~/.config/qBittorrent/qBittorrent.conf` - `Session\Port`

## Acceptance Criteria

- [x] Header displays `port: XXXXX` when connected and port forwarding is enabled
- [x] Port is read on connect and cached with server_id
- [x] Port is restored on reconnection only if server_id matches
- [x] Port is cleared when connecting to different server
- [x] qBittorrent `Session\Port` is updated on port change
- [x] No errors when port file doesn't exist (port forwarding disabled)
- [x] Follows existing code patterns (error handling, module structure)

## GitHub Issues

Searched ProtonVPN/proton-vpn-cli repository for forwarded_port related issues:
- No open issues found referencing forwarded_port file
- This appears to be a runtime behavior issue with the daemon, not a CLI bug

## Reference: qbPortWeaver Implementation

qbPortWeaver (GitHub: martsg666/qbPortWeaver) is a Windows application that syncs qBittorrent's port with ProtonVPN. It uses two methods:

### Method 1: Log File Parsing (Default)

Reads the forwarded port from ProtonVPN's log file:
- Monitors ProtonVPN connection logs
- Parses the port from log output
- Updates qBittorrent via Web API

### Method 2: NAT-PMP Protocol

Queries the VPN gateway directly:
- ProtonVPN supports NAT-PMP natively on P2P servers
- Uses RFC 6886 UDP port mapping
- More reliable than log parsing

### Our Implementation

We use the direct file approach (similar to log file parsing):
- Read from `$XDG_RUNTIME_DIR/Proton/VPN/forwarded_port`
- **Important**: Running `protonvpn status` clears this file
- Solution: Read once on connect, cache it, avoid re-reading until next connection

### Future Improvements

Consider implementing:
1. **NAT-PMP support**: Query gateway directly for port (like qbPortWeaver's NAT-PMP mode)
2. **Periodic refresh**: Poll port in background loop (qbPortWeaver uses 180s default interval)
3. **Web API for qBittorrent**: Use qBittorrent's Web API instead of config file (more reliable)