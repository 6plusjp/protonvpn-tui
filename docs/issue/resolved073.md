# issue073: Module visibility inconsistency (state vs ui mod.rs)

## Summary

**RESOLVED** - Module visibility patterns were inconsistent between `src/state/mod.rs` and `src/ui/mod.rs`. Applied Option A to align patterns.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

Two different patterns existed for module declarations:

### src/ui/mod.rs (all public)
```rust
pub mod app;
pub mod components;
pub mod input;
pub mod keymap;
pub mod render;
pub mod renderers;
pub mod styles;
pub mod views;
```

### src/state/mod.rs (implementation modules private)
```rust
mod app_state;
pub use app_state::*;

// Extracted modules from app_state.rs
mod app_state_impl;      // Private
mod event_handler;       // Private
mod navigation;          // Private
mod server_ops;          // Private
mod settings_ops;        // Private
```

## Solution

Applied **Option A**: Make ui implementation modules private, re-export public API types.

### Changes Made

**src/ui/mod.rs**:
```rust
// Public entry points
pub mod app;
pub mod views;

// Private implementation
mod components;
mod input;
mod keymap;
mod render;
mod renderers;
mod styles;

// Re-export public API types
pub use keymap::{default_keybindings, KeyArrow, KeyMap, KeyMatcher};
pub use styles::{Theme, ThemeMode};
```

**External imports updated** (5 files):
- `src/config/user_config.rs`: `use crate::ui::keymap::*` → `use crate::ui::*`
- `src/config/settings.rs`: `crate::ui::keymap::default_keybindings()` → `crate::ui::default_keybindings()`
- `src/state/app_state.rs`: `use crate::ui::keymap::KeyMap` + `use crate::ui::styles::*` → `use crate::ui::*`
- `src/state/ui_state.rs`: `use crate::ui::styles::ThemeMode` → `use crate::ui::ThemeMode`
- `src/state/settings_ops.rs`: `use crate::ui::styles::ThemeMode` → `use crate::ui::ThemeMode`

**AGENTS.md updated**:
- Root AGENTS.md: Added "Module visibility conventions" section
- src/ui/AGENTS.md: Added "Module Visibility" section

## Acceptance Criteria

- [x] Decision made on visibility pattern (Option A selected)
- [x] `src/ui/mod.rs` updated with consistent pattern
- [x] AGENTS.md updated to document visibility conventions
- [x] `cargo check` passes (no errors)
- [x] `cargo clippy` passes (no new warnings)

## Verification

```bash
cargo check   # Passed
cargo clippy  # Passed (pre-existing warnings only)
```
