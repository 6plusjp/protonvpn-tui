# issue081: Refactor - Extract timeout constants

## Summary

**RESOLVED** - Extracted hardcoded timeout values in client.rs to constants in constants.rs.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`vpn/client.rs` had hardcoded timeout values scattered across 7+ locations.

## Solution

Created timeout constants in `constants.rs::vpn` module and replaced all hardcoded values.

### Changes

| File | Change |
|------|--------|
| `src/constants.rs` | Added `vpn` module with timeout constants |
| `src/vpn/client.rs` | Replaced 7 hardcoded timeouts with constants |

### Constants Added

```rust
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(15);
pub const COUNTRIES_LIST_TIMEOUT: Duration = Duration::from_secs(60);
pub const CITIES_LIST_TIMEOUT: Duration = Duration::from_secs(20);
pub const CONFIG_SET_TIMEOUT: Duration = Duration::from_secs(20);
```

## Acceptance Criteria

- [x] Timeout constants defined in `constants.rs`
- [x] All hardcoded timeouts replaced
- [x] Behavior unchanged
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
