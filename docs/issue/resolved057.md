# issue057: Architecture - ConfigState is a thin wrapper with no value

## Summary

`src/state/config_state.rs` (27 lines) was analyzed and found to have value as a wrapper around `ProtonSettings::load()`. However, `AppState` had duplicate state storage.

## Problem (Initial Analysis)

`AppState` had both:
1. `proton_settings_cache: Option<ProtonSettings>` - direct field (unused directly)
2. `config_state: ConfigState` - wrapper (used everywhere)

The code consistently accessed via `config_state.proton_settings_cache`, not the direct field.

## Resolution (2026-03-19)

**Removed** the direct field `proton_settings_cache` from `AppState`:
- Deleted from `AppState` struct definition
- Deleted from `AppState::from_config()` initialization
- Removed unused import `ProtonSettings`

**Kept** `ConfigState` wrapper as it provides:
- Semantic grouping of settings-related state
- `clear_cache()` and `invalidate_cache()` methods
- Consistent access pattern: `self.config_state.proton_settings_cache`

## Files Changed

- `src/state/app_state.rs` - removed duplicate `proton_settings_cache` field

## Severity

🟢 RESOLVED - Removed redundant state storage
