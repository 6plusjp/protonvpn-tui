# resolved028: Settings - First run + UI redesign

## Summary

| Part | Status |
|------|--------|
| Part A: settings.json not found on first run | ✅ Implemented |
| Part B: Settings UI (toggle → expandable) | ✅ Implemented |
| Part C: Sync config_set blocks UI | ✅ Implemented |
| Part D: "unknown" in selectable options | ✅ Implemented |

---

## Part A: settings.json not found on first run ✅

**Problem**: `settings.json` doesn't exist on first run → shows "off" incorrectly.

**Solution**: Return `None` when file not found, display "unknown".

**Files**: 
- `src/config/settings.rs` - `ProtonSettings::load()` returns `None` if file missing
- `src/ui/views/settings_view.rs` - shows "unknown" when `proton_settings` is `None`

---

## Part B: Settings UI (toggle → expandable) ✅

**Solution**: Expandable selection menu with Enter to expand, j/k to navigate options.

**Files**:
- `src/ui/views/settings_view.rs` - renders expandable list with `├►`, `└─`
- `src/ui/app.rs` - `handle_settings_key()` handles expand/select/collapse
- `src/config/settings.rs` - `SettingKey::selectable_options()` defines selectable options

---

## Part C: Sync config_set blocks UI ✅

**Problem**: `protonvpn config set` was called synchronously, blocking the UI.

**Solution**: 
- Added `Job::ConfigSet` to async_tasks.rs
- Added `pending_config_set` field to AppState
- Added `spawn_config_set()` method
- Results handled in `sync_connection_state()`

**Files**:
- `src/state/async_tasks.rs` - ConfigSet job
- `src/state/app_state.rs` - pending_config_set, spawn_config_set()
- `src/ui/app.rs` - calls spawn_config_set()

---

## Part D: "unknown" not selectable ✅

**Problem**: "unknown" appeared as selectable option, but users cannot set a setting to "unknown".

**Solution**: 
- Added `selectable_options()` method that excludes "unknown"
- Added `selectable_option_count()` and `get_selectable_option_command()`
- UI now uses selectable options only

**Files**:
- `src/config/settings.rs` - selectable_options(), etc.
- `src/ui/views/settings_view.rs` - uses selectable_options()
- `src/ui/app.rs` - uses get_selectable_option_command()

---

## Changes

| Commit | Description |
|--------|-------------|
| - | Add selectable_options() to settings.rs |
| - | Add ConfigSet async job to async_tasks.rs |
| - | Add pending_config_set to AppState |
| - | Update settings_view.rs to use selectable_options |
| - | Update app.rs to use async spawn_config_set |

---

## Acceptance Criteria

- [x] Part A: App works without settings.json
- [x] Part A: Shows "unknown" (not "off")
- [x] Part B: Expandable settings UI works
- [x] Part C: config_set runs async
- [x] Part D: "unknown" not selectable

---

## Related

- `src/config/settings.rs`
- `src/ui/views/settings_view.rs`
- `src/ui/app.rs`
- `src/state/async_tasks.rs`
- `src/state/app_state.rs`
