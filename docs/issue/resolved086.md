# issue086: Clippy warnings cleanup

## Summary

**RESOLVED** - Fixed all clippy warnings in the codebase.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

Running `cargo clippy` produced 11 warnings:
- Needless borrow in common.rs
- Identical if/else blocks in footer.rs
- Dead code (unused imports, functions, types, variants)

## Solution

### 1. Fixed needless borrow
- `src/ui/input/common.rs`: `connect_fn(&mut input.state)` → `connect_fn(input.state)`

### 2. Fixed identical if/else
- `src/ui/renderers/footer.rs`: Removed redundant if/else block

### 3. Removed dead code
- Deleted `src/ui/components/list.rs` (unused functions)
- Removed unused imports from `src/ui/components/mod.rs`
- Removed unused types/methods from `src/ui/components/pane_table.rs`:
  - `ColumnAlign` enum
  - `Column::align` field
  - `Column::dynamic()` method
  - `PaneTable::header()`, `header_with_widths()`, `format_row_with_widths()`, `format_row()` methods
- Removed `AppAction::None` variant from `src/ui/input/app_action.rs`
- Removed corresponding match arm from `src/ui/app.rs`

### Changes

| File | Change |
|------|--------|
| `src/ui/input/common.rs` | Fixed needless borrow |
| `src/ui/renderers/footer.rs` | Removed identical if/else |
| `src/ui/components/list.rs` | Deleted |
| `src/ui/components/mod.rs` | Removed unused imports |
| `src/ui/components/pane_table.rs` | Removed ~95 lines of dead code |
| `src/ui/input/app_action.rs` | Removed unused variant |
| `src/ui/app.rs` | Removed unused match arm |

### Code Reduction

- **Total**: ~150 lines of dead code removed

## Acceptance Criteria

- [x] All clippy warnings resolved
- [x] No compilation errors
- [x] Tests pass

## Verification

```bash
cargo clippy
# warning: `protonvpn-tui` (lib) generated 0 warnings

cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
