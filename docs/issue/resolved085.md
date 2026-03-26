# issue085: Bug - DNS settings not updating after disable

## Summary

**RESOLVED** - Fixed DNS settings cache not reloading after disable operation.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

When disabling DNS settings via the TUI, the displayed value remained "custom(1.1.1.1)" instead of updating to "default" or "off".

## Root Cause

After calling `disable_custom_dns()`, the code set `proton_settings_cache = None` but never reloaded the cache from the settings file. The display code reads from this cache, so it showed stale data.

## Solution

Changed `proton_settings_cache = None` to `clear_cache()` in three locations in `settings_ops.rs`:

1. `toggle_settings_off()` - DNS disable
2. `apply_dns_setting()` - DNS update
3. `toggle_settings()` - General setting toggle

### Changes

| File | Change |
|------|--------|
| `src/state/settings_ops.rs` | Changed `proton_settings_cache = None` to `clear_cache()` in 3 locations |

## Acceptance Criteria

- [x] DNS disable updates display immediately
- [x] DNS set updates display immediately
- [x] Other settings toggle updates display immediately
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
