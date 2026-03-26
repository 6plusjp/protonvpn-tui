# issue083: Refactor - Consolidate toggle methods

## Summary

**RESOLVED** - Added `toggle_setting()` method to VpnClient and updated settings_ops.rs to use it.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`vpn/client.rs` had 6 nearly identical toggle methods for VPN settings.

## Solution

Added a generic `toggle_setting(&self, key: SettingKey, current: Option<bool>)` method that uses `SettingKey::config_key()`. Updated `settings_ops.rs` to use this method for all boolean settings (kept `toggle_killswitch` as special case).

### Changes

| File | Change |
|------|--------|
| `src/vpn/client.rs` | Added `toggle_setting()` method; removed 5 individual toggle methods |
| `src/state/settings_ops.rs` | Updated to use `toggle_setting()` |

### Code Reduction

- **client.rs**: ~25 lines removed (5 methods removed)

## Acceptance Criteria

- [x] `toggle_setting()` method implemented
- [x] Boolean settings use `toggle_setting()`
- [x] Killswitch special case preserved
- [x] Settings toggle behavior unchanged
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
