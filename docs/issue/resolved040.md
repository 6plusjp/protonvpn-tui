# Issue 040: Connection Type Shortcuts & Improved Sorting UI

## Summary

Add keyboard shortcuts for quick connection types (fastest, P2P, Tor, SecureCore) and implement improved sorting UI with arrow keys.

## Implemented

### Keyboard Shortcuts

| Key | Action | CLI Flag |
|-----|--------|----------|
| `c` | Connect to selected server | `--country` |
| `d` | Disconnect | - |
| `r` | Refresh server list | - |
| `x` | Random connect | `--random` |
| `f` | Fastest connection | `--fastest` |
| `p` | P2P/Torrent connection | `--p2p` |
| `t` | Tor connection | `--tor` |
| `s` | SecureCore connection | `--securecore` |

### Sorting UI

- `←` / `→` keys: Toggle sort direction (ascending ↔ descending)
- Sort indicator (↑/↓) displayed in table header next to sorted column
- Code column width increased from 4 to 6 to accommodate sort indicator

### VPN Client Methods Added

- `connect_fastest()` - `protonvpn connect --fastest`
- `connect_p2p()` - `protonvpn connect --p2p`
- `connect_tor()` - `protonvpn connect --tor`
- `connect_securecore()` - `protonvpn connect --securecore`

---

## Related Files

- `src/config/settings.rs` - Key binding config
- `src/ui/app.rs` - Key event handling, footer hints
- `src/ui/components/pane_table.rs` - Table header with sort indicator
- `src/ui/views/servers_view.rs` - Pass sort state to header
- `src/ui/views/help_view.rs` - Help display
- `src/vpn/client.rs` - VPN CLI wrapper, get_connected_server_name()
- `src/vpn/async_tasks.rs` - Async job definitions
- `src/state/app_state.rs` - Connection methods

---

## Notes

- SecureCore and Tor require Plus/Professional plan (may return 401 error)
- Sort direction indicator (↑/↓) appears in the sorted column header
- Code column width is 6 to fit sort indicator
- Connection persistence: reads `~/.cache/Proton/VPN/connection/connection_persistence.json`
