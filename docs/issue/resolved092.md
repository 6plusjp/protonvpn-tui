# issue092: Config file handling issues - double load, no external edit detection, unsafe writes

**Status: RESOLVED** ✅

## Summary

Multiple issues found in config.toml handling: double load on startup, no external edit detection, and unsafe write patterns in other file operations.

## Problems

### 1. Double Load on Startup

`UserConfig::load()` is called in `main.rs`, then called again inside `AppState::from_config()`.

### 2. No External Edit Detection

`ProtonSettings` has `invalidate_cache()`/`clear_cache()`, but `UserConfig` has **no reload mechanism**.

### 3. Same Unsafe Pattern in Other File Writes

| File | Method | Issue |
|---|---|---|
| `vpn/cache.rs` | `ServerCache::save()` | Direct overwrite |
| `state/log_persistence.rs` | `save_notification_log()` | Direct overwrite |

## Resolution

### 1. Fixed Double Load

Modified `AppState::from_config()` to accept `UserConfig` as parameter instead of loading again.

### 2. Added External Edit Detection

Added `reload_user_config()` method to `AppState` that reloads config from disk and updates UI state.

### 3. Fixed Unsafe File Writes

Applied atomic writes (temp file + rename) to:
- `ServerCache::save()` in `vpn/cache.rs`
- `save_notification_log()` in `state/log_persistence.rs`

## Files Modified

- `src/ui/app.rs` - pass UserConfig to AppState
- `src/state/app_state.rs` - accept UserConfig param, add reload_user_config()
- `src/vpn/cache.rs` - atomic writes
- `src/state/log_persistence.rs` - atomic writes

## Related Issues

- issue048: config.toml content deletion (Read-Modify-Write fix applied)
