# issue029 - Server List Not Displayed After Cache Deletion

## Summary

When the cache file (`~/.cache/protonvpn-tui/server_cache.toml`) is deleted and the app is launched, nothing is displayed.

## Expected Behavior

- When cache is missing, the hardcoded fallback country list (`FALLBACK_COUNTRIES`) should be used to display the server list
- A prompt message "Refreshing servers..." should be displayed
- After server fetch completes, the country list should be shown on screen

## Actual Behavior

- On startup, the screen shows nothing (or the country list is blank)
- Servers may not be displayed even after async processing completes

## Root Cause Analysis

### 1. Timing Issue (app_state.rs:541-554)

Initialization flow in `TuiApp::new()`:
```rust
let mut state = AppState::new();  // servers = Vec::new()
state.refresh_servers();           // spawn async task
```

- First render executes before `refresh_servers()` async task completes
- At this point, `state.servers` is empty, so nothing is displayed
- This is expected behavior but provides poor UX

### 2. Bug in use_fallback_countries() (client.rs:377-391)

```rust
fn use_fallback_countries(&self) -> AppResult<HashMap<String, String>> {
    use super::cache::FALLBACK_COUNTRIES;

    let fallback: HashMap<String, String> = FALLBACK_COUNTRIES
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    self.with_cache(|c| {
        c.cli_unavailable = true;  // only sets this
        // BUG: Does NOT save fallback countries to cache!
    })?;

    Ok(fallback)
}
```

**Problem**:
- Fallback country data is returned but NOT saved to cache
- As a result, subsequent calls to `get_servers()` return empty
- Servers won't be displayed until next async refresh

### 3. Cache Save Failure

If `ServerCache::save()` fails (permission issues, etc.):
- Country data exists in memory but is not persisted to disk
- On next launch, it will be empty again

## Impact

- When cache is deleted
- When ProtonVPN CLI is unavailable
- When cache file fails to load

## Fix Applied

### Fix 1: Save Fallback Countries to Cache (client.rs)

```rust
fn use_fallback_countries(&self) -> AppResult<HashMap<String, String>> {
    use super::cache::FALLBACK_COUNTRIES;

    let fallback: HashMap<String, String> = FALLBACK_COUNTRIES
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    self.with_cache(|c| {
        c.countries = fallback.clone();  // save to cache
        c.cli_unavailable = true;
    })?;
    self.save_cache()?;  // persist to disk

    tracing::info!("Using fallback countries (CLI unavailable)");
    Ok(fallback)
}
```

### Fix 2: Load Servers Synchronously on Init (app_state.rs)

```rust
pub fn new() -> Self {
    let vpn_state = Arc::new(VpnState::new());
    let servers = vpn_state.get_servers_or_refresh().unwrap_or_default();
    // ... rest of initialization
}
```

Now loads servers synchronously during AppState creation, ensuring fallback countries are available immediately on first render.

## Related Files

- `src/vpn/client.rs` - `use_fallback_countries()`, `refresh_countries()`
- `src/vpn/cache.rs` - `ServerCache::load()`, `ServerCache::save()`
- `src/state/app_state.rs` - `refresh_servers()`, `set_servers()`
- `src/ui/app.rs` - `TuiApp::new()`

## Status

- [x] Investigation complete
- [x] Fix implemented (2026-03-11)

## Commits

| Fix | Commit |
|-----|--------|
| Save fallback countries to cache | `4dcb507` |
| Load servers synchronously on init | `34aec3c` |
