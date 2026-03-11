# issue037 - Centralize Render Logic

## Summary

Unify render function signatures across UI views using a trait-based approach.

## Background

From issue027 (S4):

> Centralize Render Logic
> 
> **Status**: Skipped - UI changes required
> **Reason**: Requires architectural changes to UI layer.

## Current State

Render functions are scattered across 7+ files with inconsistent signatures:

```
src/ui/
├── app.rs                    # render_main(), render_header(), render_footer()
├── views/
│   ├── servers_view.rs       # render_servers_view(), render_countries_pane(), render_cities_pane()
│   ├── settings_view.rs      # render_settings_view()
│   ├── logs_view.rs          # render_logs_view()
│   ├── stats_view.rs         # render_stats_view()
│   └── help_view.rs          # render_help_view()
```

Each function has different signature:
```rust
// servers_view.rs
pub fn render_servers_view(state: &mut AppState, ...)

// settings_view.rs  
pub fn render_settings_view(state: &mut AppState, ...)

// logs_view.rs
pub fn render_logs_view(state: &AppState, ...)  // Note: &AppState, not &mut
```

## Problem

- 7+ files with render functions
- Inconsistent function signatures
- Adding new view requires editing multiple files

## Proposed Solution

Create a unified `Renderable` trait:

```rust
// src/ui/render.rs

pub trait Renderable {
    fn render(&mut self, state: &mut AppState, f: &mut Frame<'_>, area: Rect);
}

impl Renderable for ServersView { ... }
impl Renderable for SettingsView { ... }
impl Renderable for LogsView { ... }
```

Then simplify `render_main()`:
```rust
fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
    self.current_view.render(&mut self.state, f, area)
}
```

## Benefits

- Consistent API for all views
- Easy to add new views (just impl trait)
- Better encapsulation

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| Low | Unify render function signatures | Medium | Pending |

## Files to Change

- Create `src/ui/render.rs` (trait definition)
- Refactor `src/ui/views/*.rs` (implement trait)
- Update `src/ui/app.rs` (use trait)
