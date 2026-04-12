# issue028: Refactor - Navigation Methods Consolidation

## Summary

Consolidate 24 nearly identical navigation methods in `navigation.rs` into 4 generic methods using a selection target enum.

## Problem

`navigation.rs` contains 4 groups of nearly identical navigation methods with only field name differences:

### Current Structure (24 methods)
```rust
// Group 1: Server selection (lines 37-85)
pub fn select_next(&mut self) { ... }
pub fn select_prev(&mut self) { ... }
pub fn select_first(&mut self) { ... }
pub fn select_last(&mut self) { ... }
pub fn select_page_down(&mut self) { ... }
pub fn select_page_up(&mut self) { ... }

// Group 2: City selection (lines 101-133)
pub fn city_select_next(&mut self) { ... }
pub fn city_select_prev(&mut self) { ... }
pub fn city_select_first(&mut self) { ... }
pub fn city_select_last(&mut self) { ... }
pub fn city_select_page_down(&mut self) { ... }
pub fn city_select_page_up(&mut self) { ... }

// Group 3: Settings selection (lines 135-187)
pub fn settings_select_next(&mut self) { ... }
pub fn settings_select_prev(&mut self) { ... }
// ... 4 more

// Group 4: Logs selection (lines 189-221)
pub fn logs_select_next(&mut self) { ... }
pub fn logs_select_prev(&mut self) { ... }
// ... 4 more
```

Each method only differs in which field it reads/modifies (e.g., `selected_server` vs `selected_city`).

## Solution

Create a target enum and generic selection methods:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionTarget {
    Server,
    City,
    Settings,
    Logs,
}

impl Navigation {
    pub fn select_next(&mut self, target: SelectionTarget) { ... }
    pub fn select_prev(&mut self, target: SelectionTarget) { ... }
    pub fn select_first(&mut self, target: SelectionTarget) { ... }
    pub fn select_last(&mut self, target: SelectionTarget) { ... }
    pub fn select_page_down(&mut self, target: SelectionTarget) { ... }
    pub fn select_page_up(&mut self, target: SelectionTarget) { ... }
}
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/state/navigation.rs` | Consolidate 24 methods → 4 generic methods |
| `src/ui/input/common.rs` | Update callers to use new signatures |

### Acceptance Criteria

- [ ] 24 methods consolidated to 4
- [ ] All navigation behavior preserved
- [ ] `cargo test` passes
- [ ] Estimated ~150 lines reduced
- [ ] Easier to add new selection targets in future