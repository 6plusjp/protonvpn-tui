# issue037 - Centralize Render Logic

## Summary

Unify render function signatures across UI views using a trait-based approach.

## Background

From issue027 (S4):

> Centralize Render Logic
> 
> **Status**: Skipped - UI changes required
> **Reason**: Requires architectural changes to UI layer.

## Completed

Render functions have been unified using a `Renderable` trait and `View` enum.

### Changes Made

**Created:**
- `src/ui/render.rs` - Contains `Renderable` trait and `View` enum

**Deleted:**
- `src/ui/views/stats_view.rs` - Unused file removed

**Modified:**
- `src/ui/mod.rs` - Added `render` module
- `src/ui/app.rs` - Simplified to use `Renderable` trait

### Architecture

```rust
// src/ui/render.rs
pub trait Renderable {
    fn render(&mut self, state: &mut AppState, f: &mut Frame<'_>, area: Rect);
}

pub enum View {
    Servers(ServersViewState),
    Tools(ToolsViewState),
    Help,
}
```

### Before

```rust
fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
    match self.state.ui_state.current_view {
        AppView::Servers => views::servers_view::render_servers_view(...),
        AppView::Tools => views::tools_view::render_tools_view(...),
        AppView::Help => views::help_view::render_help_view(...),
    }
}
```

### After

```rust
fn render_main(&mut self, f: &mut Frame<'_>, area: Rect) {
    self.current_view.render(&mut self.state, f, area);
}
```

## Benefits

- Consistent API for all views (all use `&mut AppState`)
- Easy to add new views (just add variant to `View` enum)
- View state encapsulated in dedicated structs
- Simplified `app.rs`

## Status

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| Low | Unify render function signatures | Medium | ✅ Completed |
