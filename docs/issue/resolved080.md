# issue080: Refactor - ThemeMode index conversion duplication

## Summary

**RESOLVED** - Added `ThemeMode::from_index()` method to eliminate duplicated match blocks in tools.rs.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`input/tools.rs` had 6 identical match blocks converting option index to ThemeMode.

## Solution

Added `ThemeMode::from_index(index: usize) -> Option<Self>` method to `ThemeMode` enum. All 6 match blocks now use this method.

### Changes

| File | Change |
|------|--------|
| `src/ui/styles.rs` | Added `ThemeMode::from_index()` method |
| `src/ui/input/tools.rs` | Replaced 6 match blocks with `ThemeMode::from_index()` |

### Code Reduction

- **tools.rs**: ~60 lines removed

## Acceptance Criteria

- [x] `ThemeMode::from_index()` implemented
- [x] All 6 match blocks replaced
- [x] Theme switching behavior unchanged
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
