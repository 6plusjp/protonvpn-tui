# issue007: Create UI Components Layer and Unify Design

## Summary

Create a UI components layer to unify design and improve code reusability across all views.

## Current Implementation Status

### ✅ Completed

1. **Theme struct enhanced** - New fields added: `block_border`, `selection`, `connected`, `key_hint`
2. **Theme switching implemented** - Both `dark()` and `light()` methods exist
3. **Components created**:
   - `components/block.rs` - `centered_block()` function
   - `components/list.rs` - `styled_list_item()`, `connected_list_item()` functions

### ⚠️ BUG: Theme Switching Broken

**Root Cause**: Only `settings_view.rs` checks `state.is_dark_theme`. All other views use `Theme::default()` which always returns dark theme.

| File | Current Theme Usage | Issue |
|------|---------------------|-------|
| `settings_view.rs` | `if state.is_dark_theme { Theme::dark() } else { Theme::light() }` | ✅ Works correctly |
| `servers_view.rs` | `Theme::default()` | ❌ Always dark |
| `cities_view.rs` | `Theme::default()` | ❌ Always dark |
| `logs_view.rs` | `Theme::default()` | ❌ Always dark |
| `stats_view.rs` | `Theme::default()` | ❌ Always dark |
| `help_view.rs` | `Theme::default()` | ❌ Always dark |

**Fix Required**: All views must use the same pattern as `settings_view.rs`:
```rust
let theme = if state.is_dark_theme {
    Theme::dark()
} else {
    Theme::light()
};
```

## Problem Context (Original Analysis)

### Current Issues

**1. Theme Not Used**
- `Theme` struct is defined in `src/ui/styles.rs` but all views directly use `Color::Cyan`, `Color::White`, etc.
- No theme switching capability (only dark/light definitions exist)

**2. Inconsistent Styles**

| File | Block style | Text color | Selection |
|------|-------------|------------|-----------|
| `servers_view.rs` | none | White | Cyan (selected), Green (connected) |
| `stats_view.rs` | none | White | none |
| `settings_view.rs` | none | White | Cyan (selected) |
| `cities_view.rs` | none | White | Cyan (selected) |
| `logs_view.rs` | none | Info=Cyan, OK=Green, ERR=Red | none |
| `help_view.rs` | White | Yellow/Cyan | none |
| `app.rs` header | Cyan | Green (always) | - |
| `app.rs` footer | - | Yellow | - |

**3. Inconsistent Selection Indicators**
- `servers_view.rs`: `> ` (selected), `* ` (connected)
- `settings_view.rs`: `> ` (selected)
- `cities_view.rs`: `> ` (selected)
- Not unified

**4. Empty Components Module**
- `src/ui/components/mod.rs` is empty (only 3 lines)

### Target Files

```
src/ui/
├── styles.rs          # Modify to use Theme throughout
├── components/
│   └── mod.rs         # Create common components here
└── views/
    ├── servers_view.rs
    ├── stats_view.rs
    ├── settings_view.rs
    ├── cities_view.rs
    ├── logs_view.rs
    └── help_view.rs
```

## Requirements

### 1. Apply Theme

Modify `src/ui/styles.rs` to apply Theme throughout:

```rust
// After modification
use crate::ui::styles::Theme;

pub fn render_servers_view(...) {
    let theme = Theme::default();
    let block = Block::default()
        .title(" Servers ")
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.primary));
}
```

### 2. Create Common UI Components

Create under `src/ui/components/mod.rs`:

| Component | Description |
|-----------|-------------|
| `block.rs` | Unified Block creation functions |
| `list.rs` | Selectable list widget helpers |
| `styles.rs` | Component-specific style functions |
| `mod.rs` | Public interface exports |

### 3. Unification Rules

| Element | Rule |
|---------|------|
| Block border | All use `Borders::ALL` |
| Block title | `" <ViewName> "` (single space) |
| Block color | `theme.primary` |
| List text | `theme.foreground` (White/Black) |
| Selection | `theme.primary` + BOLD |
| Connected | `theme.success` + BOLD |
| Prefix | Unify to `> ` only |

## Technical Implementation

### Phase 1: Enhance Theme

```rust
// src/ui/styles.rs

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    // Add new fields
    pub block_border: Color,
    pub selection: Color,
    pub connected: Color,
    pub key_hint: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            // Add new
            block_border: Color::Cyan,
            selection: Color::Cyan,
            connected: Color::Green,
            key_hint: Color::Yellow,
        }
    }
}
```

### Phase 2: Create Components

```rust
// src/ui/components/block.rs

use ratatui::{
    widgets::{Block, Borders},
    style::Style,
};
use crate::ui::styles::Theme;

pub fn centered_block(title: &str, theme: &Theme) -> Block<'static> {
    Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.block_border))
}
```

```rust
// src/ui/components/list.rs

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
};
use crate::ui::styles::Theme;

pub fn styled_list_item(
    text: &str,
    is_selected: bool,
    theme: &Theme,
) -> ListItem<'static> {
    let prefix = if is_selected { "> " } else { "  " };
    let style = if is_selected {
        Style::default()
            .fg(theme.selection)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.foreground)
    };
    
    ListItem::new(Line::from(vec![
        Span::raw(prefix),
        Span::styled(text, style),
    ]))
}
```

### Phase 3: Update Views

Refactor each view to use new components:

1. `servers_view.rs` - Get selection/connected colors from theme
2. `stats_view.rs` - Get block color from theme
3. `settings_view.rs` - Get selection color from theme
4. `cities_view.rs` - Get selection color from theme
5. `logs_view.rs` - Get info/success/error colors from theme
6. `help_view.rs` - Get key/action colors from theme
7. `app.rs` - header/footer/notification popup

**IMPORTANT**: All views must check `state.is_dark_theme` to switch between themes:
```rust
let theme = if state.is_dark_theme {
    Theme::dark()
} else {
    Theme::light()
};
```

## Acceptance Criteria

- [x] Add new fields to `Theme` struct: `block_border`, `selection`, `connected`, `key_hint`
- [x] Create `components/block.rs` - unified block creation function
- [x] Create `components/list.rs` - unified list item creation function
- [ ] Use Theme/Component in all views
- [x] Selection uses `> ` prefix + primary color + BOLD
- [x] Connected uses `* ` prefix + success color + BOLD
- [x] Footer key hints use `key_hint` color
- [x] Each view's block title follows unified format `" <ViewName> "`
- [ ] **FIX**: All views must check `state.is_dark_theme` for theme switching

## Notes

- Maintain existing color scheme while unifying access through Theme struct
- Dark theme only implementation is acceptable at this stage (light theme not required)
- Only styles should change, existing view logic remains unchanged
