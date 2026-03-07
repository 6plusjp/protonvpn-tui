# ProtonVPN CLI Concurrent Execution

## Overview

Analysis of whether multiple `protonvpn` commands can run concurrently.

**Note**: This is based on code analysis, not official Proton documentation.

---

## Concurrent Execution Matrix

| Command A | Command B | Can Run Concurrently? |
|-----------|-----------|---------------------|
| `protonvpn countries` | `protonvpn cities --country XX` | ✅ Yes |
| `protonvpn countries` | `protonvpn status` | ✅ Yes |
| `protonvpn cities --country JP` | `protonvpn cities --country US` | ✅ Yes |
| `protonvpn connect` | `protonvpn connect` | ❌ No |
| `protonvpn connect --random` | `protonvpn connect` | ❌ No |
| `protonvpn connect` | `protonvpn disconnect` | ❌ No |
| `protonvpn connect` | `protonvpn countries` | ✅ Yes |
| `protonvpn connect` | `protonvpn cities --country XX` | ✅ Yes |

---

## Analysis

### Read-Only Commands (Can Run in Parallel)

- `protonvpn countries`
- `protonvpn cities --country XX`
- `protonvpn info`

These commands only:
1. Create a new `Controller` instance
2. Make HTTP API calls to fetch server information
3. Do NOT interact with NetworkManager

Since each invocation creates a fresh API session, multiple read-only commands can run concurrently.

### Connection Commands (Cannot Run in Parallel)

- `protonvpn connect`
- `protonvpn connect --random`
- `protonvpn disconnect`

These commands interact with **NetworkManager** to manage VPN connections. NetworkManager only allows one active VPN connection at a time, so these commands must be serialized.

### Cross-Compatibility

- Read-only commands (`countries`, `cities`, `status`) can run while a VPN is connected
- Connection commands can run while other connection commands are NOT running (obviously)

---

## Implementation Notes

### Current protonvpn-tui Implementation

The current implementation uses sequential async tasks with `pending_*` fields:
- `pending_refresh` - server list refresh
- `pending_cities` - cities fetch
- `pending_connect` - VPN connection
- `pending_disconnect` - VPN disconnection
- `pending_connect_city` - city connection

This serialization is safe but could be optimized for read-only commands.

### Potential Optimization

For better performance, read-only commands could be parallelized using `tokio::task::spawn` without waiting for previous tasks to complete, since they don't conflict with each other.

---

## Sources

- `proton-vpn-cli` repository: https://github.com/ProtonVPN/proton-vpn-cli
- `location_discovery.py` - countries/cities commands
- `controller.py` - core logic

---

## Revision History

- 2026-03-07: Initial analysis based on code review
