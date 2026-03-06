# issue005: Server view - Cities not loaded from cache after navigation

## Summary

Server view should always display cities from the VPN state cache, including cities fetched during Cities view navigation.

## Problem

Current behavior:

1. **Servers view (initial)**: Shows `cities` as empty (`-`)
2. **Enter key → Cities view**: Fetches cities for selected country, stores in `VpnClient.cache.cities`
3. **Esc key → Servers view**: Shows `cities` as empty (`-`)
4. **'r' key (refresh)**: Reloads servers from cache, now shows cities correctly

The issue is that `AppState.servers` is not updated when cities are fetched. The cache (`VpnClient.cache.cities`) has the data, but `AppState.servers` still holds stale data without cities.

## Root Cause

- `fetch_cities()` in `app_state.rs:468` calls `vpn_state.list_cities_with_features()` which stores cities in `VpnClient.cache.cities`
- However, `AppState.servers` is not updated with this cached data
- `render_servers_view()` uses `state.filtered_servers()` which returns `AppState.servers` (stale)
- Only after pressing 'r' (refresh) does `vpn_state.get_servers()` return the merged data

## Solution

### Option B (Recommended): Use `vpn_state.get_servers()` in `filtered_servers()`

Modify `AppState.filtered_servers()` to always call `vpn_state.get_servers()` instead of using the stale `self.servers`.

**Before:**
```rust
pub fn filtered_servers(&self) -> Vec<Server> {
    // Returns self.servers (stale, no cities from cache)
    // ...
}
```

**After:**
```rust
pub fn filtered_servers(&self) -> Vec<Server> {
    // Always get fresh data from cache
    let servers = self.vpn_state.get_servers();
    // Then apply filters/sorting...
}
```

### Benefits

1. Single source of truth: `VpnClient.cache` always has the latest data
2. No need to sync `AppState.servers` separately
3. Works correctly for all navigation patterns
4. Minimal code change

### Drawback

- Slight performance overhead (cache read + filter each render)
  - But cache read is fast (in-memory `HashMap`)
  - Filter results are still cached via `filtered_servers_cache`

## Technical Notes

- `VpnClient.get_servers()` (line 428) already merges cities from cache: `cities_map.get(code).cloned().unwrap_or_default()`
- `VpnClient.list_cities_with_features()` (line 339) stores fetched cities in cache
- Added `invalidate_filtered_cache()` calls in `fetch_cities()` and after pending cities fetch completes
- Added `VpnClient::with_test_servers()` and `VpnState::with_test_servers()` for testing

## Changes Made

1. `src/state/app_state.rs:536` - `compute_filtered_servers()` uses `vpn_state.get_servers()` instead of `self.servers`
2. `src/state/app_state.rs:477` - Added `invalidate_filtered_cache()` in `fetch_cities()`
3. `src/state/app_state.rs:343` - Added `invalidate_filtered_cache()` after pending cities fetch completes
4. `src/vpn/client.rs:522-541` - Added `VpnClient::with_test_servers()` test helper
5. `src/vpn/state.rs` - Added `VpnState::with_test_servers()` test helper
6. Updated all tests in `app_state.rs` to use `VpnState::with_test_servers()`

## Acceptance Criteria

- [x] Servers view shows cities immediately after returning from Cities view
- [x] No need to press 'r' (refresh) to see cities
- [x] Behavior consistent with other cache data
