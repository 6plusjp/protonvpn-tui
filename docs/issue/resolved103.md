# Resolved: Navigation Methods Consolidation

## Summary

Consolidate 24 nearly identical navigation methods in `navigation.rs` into 6 generic methods using a selection target enum.

## Problem

`navigation.rs` contains 4 groups of nearly identical navigation methods with only field name differences:

### Original Structure (24 methods)
- Group 1: Server selection (6 methods)
- Group 2: City selection (6 methods)
- Group 3: Settings selection (6 methods)
- Group 4: Logs selection (6 methods)

Each method only differs in which field it reads/modifies.

## Solution

Created `SelectionTarget` enum and 6 generic navigation methods:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionTarget {
    Server,
    City,
    Settings,
    Logs,
}

impl AppState {
    pub fn navigate_next(&mut self, target: SelectionTarget) { ... }
    pub fn navigate_prev(&mut self, target: SelectionTarget) { ... }
    pub fn navigate_first(&mut self, target: SelectionTarget) { ... }
    pub fn navigate_last(&mut self, target: SelectionTarget) { ... }
    pub fn navigate_page_down(&mut self, target: SelectionTarget) { ... }
    pub fn navigate_page_up(&mut self, target: SelectionTarget) { ... }
}
```

### Files Modified

| File | Changes |
|------|---------|
| `src/state/navigation.rs` | Consolidated 24 → 6 methods, added SelectionTarget enum |
| `src/state/mod.rs` | Export SelectionTarget |
| `src/ui/input/common.rs` | Updated to use generic methods |
| `src/ui/input/tools.rs` | Updated to use generic methods |

### Acceptance Criteria

- [x] 24 methods consolidated to 6
- [x] All navigation behavior preserved
- [x] `cargo test` passes (125 tests)
- [x] 111 lines reduced (398 → 287)
- [x] Easier to add new selection targets in future

## Completion Notes

- No wrapper methods - callers use generic `navigate_*` methods directly with `SelectionTarget` enum
- Added `get_selection_target()` in `common.rs` to determine target from view/pane
- `SelectionTarget` is publicly exported from `crate::state`
