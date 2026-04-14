# Resolved 106: Refactor - Highlight Style Helper Extraction

## Summary

Extract duplicated highlight style logic across 4 view files into a shared helper in `components/mod.rs`.

## Problem

Identical (or similar) style pattern in multiple locations:

### servers_view.rs (lines 160-167, 286-293)
```rust
.row_highlight_style(if is_focused {
    Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
} else {
    Style::default()
})
```
Note: line 286-293 had a bug where else branch incorrectly had `.add_modifier(Modifier::BOLD)`

### logs_view.rs (lines 102-109)
```rust
.row_highlight_style(if is_focused {
    Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
} else {
    Style::default()
})
```

### settings_view.rs (lines 76-83)
```rust
.highlight_style(if is_focused {
    Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
} else {
    Style::default().add_modifier(Modifier::BOLD)  // Bug: BOLD in else
})
```

## Solution

Add unified helper to `src/ui/components/mod.rs`:

```rust
pub fn highlight_style(theme: &Theme, is_focused: bool) -> Style {
    if is_focused {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(theme.background)
            .bg(theme.accent)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}
```

### Files Modified

| File | Changes |
|------|---------|
| `src/ui/components/mod.rs` | Add `highlight_style()` helper |
| `src/ui/views/servers_view.rs` | Use helper (2 locations, fixed bug) |
| `src/ui/views/logs_view.rs` | Use helper |
| `src/ui/views/settings_view.rs` | Use helper (fixed bug) |

Note: `tools_view.rs` did not have this pattern, so no changes needed there.

### Acceptance Criteria

- [x] Helper function added
- [x] 4 view files refactored (servers, logs, settings + bug fixes)
- [x] `cargo test` passes
- [x] `cargo clippy` passes
- [x] Bug fixed: removed incorrect BOLD in else branches
- [x] Reduced ~30 lines of duplication