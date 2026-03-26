# issue078: Refactor - Connection command handler duplication

## Summary

**RESOLVED** - Refactored common.rs to eliminate duplicated connection command handlers using a generic helper.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`input/common.rs` had 5 connection command handlers (fastest, p2p, tor, securecore, random) with identical patterns.

## Solution

Extracted a `handle_connection_command` helper function that handles the common pattern of checking connection state and either showing a notification or executing the connect function.

### Changes

| File | Change |
|------|--------|
| `src/ui/input/common.rs` | Added `handle_connection_command` helper; refactored 5 handlers |

### Code Reduction

- **Before**: ~376 lines
- **After**: ~300 lines
- **Reduction**: ~76 lines

## Acceptance Criteria

- [x] `handle_connection_command` helper implemented
- [x] All 5 connection handlers use the helper
- [x] Existing behavior preserved
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
