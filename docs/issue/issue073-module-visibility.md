# issue073: Module visibility inconsistency (state vs ui mod.rs)

## Summary

Module visibility patterns are inconsistent between `src/state/mod.rs` and `src/ui/mod.rs`, creating unclear API surface for the library.

## Problem

Two different patterns exist for module declarations:

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

mod connection_state;
pub use connection_state::*;

// ... (public modules with re-exports)

// Extracted modules from app_state.rs
mod app_state_impl;      // Private
mod event_handler;       // Private
mod navigation;          // Private
mod server_ops;          // Private
mod settings_ops;        // Private
```

### Issue

- `state/` correctly hides implementation details (`app_state_impl`, `event_handler`, etc.)
- `ui/` exposes everything publicly, including implementation details like `renderers::header`, `renderers::footer`
- This creates an inconsistent API surface — users can access internal rendering functions but not internal state operations

## Analysis

Current state module structure is **well-designed**:
- Public API: `AppState`, `ConnectionState`, `AppView`, etc. (via re-exports)
- Implementation: `app_state_impl`, `navigation`, etc. (private, used by `AppState` methods)

Current ui module structure **lacks encapsulation**:
- All submodules public → users can access `ui::renderers::footer::render_footer()` directly
- Should users call these, or should they use higher-level rendering functions?

## Solution Options

### Option A: Make ui implementation modules private (Recommended)

```rust
// src/ui/mod.rs
pub mod app;        // Main entry point — public
pub mod views;      // Public views

// Implementation details — private
mod components;
mod input;
mod keymap;
mod render;
mod renderers;
mod styles;

// Re-export public types only
pub use styles::Theme;
pub use keymap::KeyAction;
```

**Pros**: Consistent with state module, clear API boundary
**Cons**: More re-exports needed in mod.rs

### Option B: Keep all public but document conventions

Add documentation to AGENTS.md explaining:
- `ui/*` is public for flexibility (renderers can be composed)
- `state/*` implementation is private (AppState methods are the API)

**Pros**: No code changes
**Cons**: Inconsistent patterns remain

### Option C: Mirror state pattern — public types, private implementation

```rust
// src/ui/mod.rs
pub use app::TuiApp;
pub use styles::Theme;
pub use keymap::{KeyAction, KeyMap};

mod app;
mod components;
mod input;
mod keymap;
mod render;
mod renderers;
mod styles;
mod views;
```

**Pros**: Cleanest API, consistent with state
**Cons**: More re-exports, could limit composability

## Recommendation

**Option A** — Make renderers, components, input, render, styles, keymap private. Re-export only public API types.

This matches the state module's proven pattern and creates a clear public API boundary.

## Acceptance Criteria

- [ ] Decision made on visibility pattern (A, B, or C)
- [ ] If Option A or C: `src/ui/mod.rs` updated with consistent pattern
- [ ] If Option A or C: AGENTS.md updated to document visibility conventions
- [ ] `cargo check` passes with no warnings
- [ ] `cargo clippy` passes clean

## Notes

- No code behavior changes expected — this is purely about API surface
- If renderers need to be composable externally, keep them public but document the intent
