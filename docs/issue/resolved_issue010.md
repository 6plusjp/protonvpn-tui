# issue010: Refactor Settings Navigation to Use Internal Count

## Summary

Remove the need for `get_settings_count()` by following the same pattern as Servers/Cities views - get the count internally within the `settings_select_*` methods.

## Status

**Resolved** - Completed

## Problem Context

Currently, settings navigation requires passing a count from outside:

```rust
// src/ui/app.rs - repeated 6 times
let count = self.state.get_settings_count();
self.state.settings_select_next(count);
```

This is inconsistent with Servers/Cities which handle the count internally:

```rust
// src/ui/app.rs - Servers/Cities pattern
self.state.select_next();  // count handled internally
```

Additionally, `get_settings_count()` is hardcoded:

```rust
// src/state/app_state.rs
pub fn get_settings_count(&self) -> usize {
    8  // Hardcoded - not dynamic
}
```

## Requirements

1. **Remove `get_settings_count()`** from `AppState`
2. **Modify `settings_select_*` methods** to internally obtain the settings count:
   - Access the settings list/vector
   - Get its length directly
3. **Update callers** in `ui/app.rs` to call methods without passing count:

```rust
// Before
let count = self.state.get_settings_count();
self.state.settings_select_next(count);

// After
self.state.settings_select_next();
```

## Implementation Notes

The settings are stored in `AppState.proton_settings_cache: Option<ProtonSettings>`. The actual count is computed via `ProtonSettings::settings_count(&self)`.

## Completed Changes

### 1. Removed `get_settings_count()` from `AppState`

Deleted the hardcoded function from `src/state/app_state.rs`.

### 2. Modified `settings_select_*` methods to get count internally

All 6 methods (`next`, `prev`, `first`, `last`, `page_down`, `page_up`) now:
- Call `get_proton_settings()` internally
- Use `settings_count()` to get actual count
- Apply `.max(7)` to guarantee minimum 7 Proton settings (UI always shows 7)
- Add +1 for Theme setting

```rust
pub fn settings_select_next(&mut self) {
    let count = self
        .get_proton_settings()
        .map(|ps| ps.settings_count())
        .unwrap_or(0)
        .max(7)
        + 1;
    self.settings_selected.move_next(count);
}
```

### 3. Updated callers in `ui/app.rs`

Changed 6 call sites from:
```rust
let count = self.state.get_settings_count();
self.state.settings_select_next(count);
```

To:
```rust
self.state.settings_select_next();
```

### 4. Fixed clippy warning

Changed `match` to `if` for single-arm equality check in `src/ui/app.rs`.

## Notes

- This is a follow-up to issue007
- The hardcoded value of 8 was a workaround to ensure theme navigation worked
- Bug fix: `.max(7)` ensures Theme is always accessible regardless of loaded settings
