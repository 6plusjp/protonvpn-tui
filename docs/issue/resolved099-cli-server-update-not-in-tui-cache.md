# issue099: CLI automatic server list update not reflected in TUI cache

## Status: Implemented ✓

## Summary

When `protonvpn` CLI internally calls `get_updated_server_list()` (triggered by commands like `connect`, `status`, `cities list` when server list is stale), the CLI outputs "Server list is outdated, updating..." and refreshes its internal server data from Proton API. However, this updated data is NOT saved to the TUI's cache, causing the TUI to display stale server information.

## Problem Description

### Upstream CLI Behavior

The `protonvpn` CLI internally uses a Python function called `get_updated_server_list()` in `proton/vpn/cli/core/controller.py`. This function is called when:

- Running `protonvpn connect` (with or without arguments)
- Running `protonvpn status`
- Running `protonvpn cities list <CC>`
- Running `protonvpn countries list`

When the CLI's cached server list is stale (typically 24+ hours), the CLI:
1. Outputs "Server list is outdated, updating... This may take a moment."
2. Calls Proton API to get fresh server data
3. Updates its internal cache
4. Proceeds with the requested command

### Current TUI Behavior

The TUI maintains its own server cache (`ServerCache` in `src/vpn/cache.rs`) that is:
- Loaded at startup from `~/.cache/protonvpn-tui/server_cache.toml`
- Updated only when `refresh_servers()` is explicitly called

The TUI calls `refresh_servers()` in two scenarios:
1. **App startup** (only if cache is empty) - `src/ui/app.rs:67`
2. **User presses 'r'** - `src/ui/input/servers.rs:80`

### The Problem

When the user runs `protonvpn` CLI commands outside the TUI (e.g., in another terminal), or even when TUI executes commands that trigger internal server list updates, the TUI's cache is NOT updated.

**Example scenario:**
1. User uses TUI to connect to a server
2. CLI internally updates server list (detected by "Server list is outdated" message)
3. User quits TUI
4. Next time TUI starts, it loads stale cached server data
5. Server list in TUI may be missing new servers or show offline servers as available

## Solution Implemented

### Option 1: Detect "outdated" message and auto-refresh (Implemented)

After any CLI command execution, check if the output contains "Server list is outdated":
- If found, automatically trigger `refresh_servers()` to sync the updated data
- This is implemented in `src/vpn/client.rs` where CLI commands are executed

### Implementation Details

**1. `src/vpn/client.rs`:**
- Added `check_server_list_outdated()` helper function to detect "server list is outdated" message
- Modified connect methods to return `(ConnectResult, bool)` tuple with `needs_refresh` flag:
  - `connect_country()`
  - `connect_random()`
  - `connect_city()`
  - `connect_fastest()`
  - `connect_p2p()`
  - `connect_tor()`
  - `connect_securecore()`
- Modified `list_cities_with_features()` to return `(Vec<City>, bool)` tuple

**2. `src/vpn/async_tasks.rs`:**
- Updated async job handlers to pass `needs_refresh` flag to `AsyncEvent` variants

**3. `src/state/connection_manager.rs`:**
- Extended `AsyncEvent` enum with `bool` flag:
  - `Connected(ConnectResult, bool)`
  - `CitiesLoaded(String, Vec<City>, bool)`
  - `ConnectCityResult(ConnectResult, bool)`

**4. `src/state/event_handler.rs`:**
- Added logic to trigger `refresh_servers()` when `needs_refresh` is true after:
  - Connection success (`AsyncEvent::Connected`)
  - Cities loaded (`AsyncEvent::CitiesLoaded`)
  - City connection success (`AsyncEvent::ConnectCityResult`)

### Commands Covered

| Command | Status | Notes |
|---------|--------|-------|
| `connect_country` | ✓ Implemented | Returns ConnectResult + bool |
| `connect_random` | ✓ Implemented | Same pattern |
| `connect_fastest` | ✓ Implemented | Same pattern |
| `connect_p2p` | ✓ Implemented | Same pattern |
| `connect_tor` | ✓ Implemented | Same pattern |
| `connect_securecore` | ✓ Implemented | Same pattern |
| `connect_city` | ✓ Implemented | Returns ConnectResult + bool |
| `list_cities_with_features` | ✓ Implemented | Returns Vec<City> + bool |
| `refresh_countries` | Not needed | Already triggers refresh |
| `disconnect` | Not needed | Doesn't trigger server list update |
| `get_status_info` | Not implemented | Status command could trigger update but not in scope |

## Priority

**Medium** - Not critical but affects user experience when server list changes frequently (new servers added, servers taken offline, etc.)

## Affected Files

- `src/vpn/client.rs` - CLI command execution layer (modified)
- `src/vpn/cache.rs` - Server cache implementation (no changes needed)
- `src/state/app_state_impl.rs` - refresh_servers() method (no changes needed)
- `src/vpn/async_tasks.rs` - Async task management (modified)
- `src/state/connection_manager.rs` - Async event definitions (modified)
- `src/state/event_handler.rs` - Event handling with auto-refresh (modified)

## References

- Upstream CLI: https://github.com/ProtonVPN/proton-vpn-cli
- Related issue: `issue096-protonvpn-cli-commands-update.md` (documents "Server list is outdated" message handling)
- `docs/reference/PROTONVPN_CONCURRENT.md` - Documents upstream CLI's get_updated_server_list() function

## Implementation Verification

- ✓ `cargo check` passes
- ✓ `cargo test` passes (125 unit tests + integration tests)
- ✓ `cargo build --release` builds successfully