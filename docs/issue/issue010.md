# issue010: Fix Settings Count to be Dynamic

## Summary

Make `get_settings_count()` return the correct count based on actual Proton settings loaded, plus the theme setting.

## Problem Context

Currently, `get_settings_count()` returns a hardcoded value of 8:

```rust
// src/state/app_state.rs
pub fn get_settings_count(&self) -> usize {
    8  // Hardcoded - not dynamic
}
```

This works for now but is not ideal because:
- It doesn't reflect the actual number of Proton settings loaded
- If Proton settings are partially loaded, the navigation may not work correctly
- The hardcoded value masks the real issue

## Requirements

Modify `get_settings_count()` to:
1. Return the actual count of Proton settings loaded (via `ps.settings_count()`)
2. Add 1 for the Theme setting

The implementation should look like:

```rust
pub fn get_settings_count(&self) -> usize {
    self.get_proton_settings()
        .map(|ps| ps.settings_count())
        .unwrap_or(7) // Default: 7 Proton settings
        + 1 // +1 for Theme setting
}
```

## Notes

- This is a follow-up fix to issue007
- The hardcoded value of 8 was a workaround to ensure theme navigation worked
- After fixing, verify that settings view navigation works correctly with actual Proton settings loaded
