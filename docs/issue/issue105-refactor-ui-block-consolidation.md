# issue030: Refactor - UI Block Consolidation

## Summary

Consolidate nearly identical `centered_block` and `block_with_title` functions in `block.rs`.

## Problem

Two functions with nearly identical implementations in `src/ui/components/block.rs`:

```rust
pub fn centered_block(title: &str, theme: &Theme, focused: bool) -> Block<'static> {
    let border_color = if focused { theme.primary } else { theme.dim };
    Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color))
}

pub fn block_with_title(title: String, theme: &Theme, focused: bool) -> Block<'static> {
    let border_color = if focused { theme.primary } else { theme.dim };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color))
}
```

Only differences:
1. `centered_block` wraps title in spaces: `format!(" {} ", title)`
2. `centered_block` takes `&str`, `block_with_title` takes `String`

## Solution

Consolidate into single function with optional formatting:

```rust
pub fn block(title: impl Into<String>, theme: &Theme, focused: bool, centered: bool) -> Block<'static> {
    let border_color = if focused { theme.primary } else { theme.dim };
    let title = if centered {
        format!(" {} ", title.into())
    } else {
        title.into()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color))
}
```

Keep backwards compatibility via type aliases:

```rust
/// Deprecated: Use block() instead
pub type centered_block = block;
// Or keep both for now and mark deprecated
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/ui/components/block.rs` | Consolidate to single function |
| Callers | Update to use new function if needed |

### Acceptance Criteria

- [ ] Both functions work as before
- [ ] Single implementation
- [ ] `cargo test` passes
- [ ] Estimated ~10 lines reduced