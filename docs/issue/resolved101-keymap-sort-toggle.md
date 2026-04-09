# issue026: Keymap Improvement - Separate Arrow Keys and Sort Toggle

## Summary

Separate the dual-purpose arrow keys (←/→) and move sort direction toggle to a dedicated key.

## Current Problem

Arrow keys currently have dual purposes:

| Key | Function |
|------|----------|
| `←` / `h` | PanePrev (move to previous pane) |
| `→` / `l` | PaneNext (move to next pane) |
| `←` / `→` | **sort direction toggle** |

**Issue**: 
- `←`/`→` handle both pane navigation AND sort toggle
- Users may find this confusing
- Separating these would be more intuitive

## Solution

### 1. Make Arrow Keys Pane Navigation Only (Fixed, Non-Customizable)

Fix `←`/`→` to work exactly like `h`/`l` (pane navigation). **This behavior should be non-customizable** - always map to pane navigation.

### 2. Assign Dedicated Key for Sort Direction Toggle

Based on currently unassigned keys, propose the following candidates:

| Candidate Key | Recommendation | Reason |
|---------------|----------------|--------|
| **`a`** | ★★★★★ | "ascending/descending" toggle, intuitive, currently unassigned |
| **`Space`** | ★★★★☆ | Easy to press, currently unassigned |
| `s` | ★★★☆☆ | "sort" direct meaning but conflicts with connection's "s" |
| `<` / `>` | ★★☆☆☆ | Directional but requires Shift |
| `Tab` | ★☆☆☆☆ | Conflicts with other functions |

**Recommended**: `a` key ("ascending" toggle)

## Files to Modify

| File | Changes |
|----------|---------|
| `src/ui/input/servers.rs` | Remove ←/→ sort toggle, use keymap for pane nav, add 'a' key sort toggle |
| `src/ui/input/tools.rs` | Use keymap for pane navigation (includes arrow keys) |
| `src/ui/keymap.rs` | Add `sort_direction` KeyAction and KeyMatcher |
| `src/config/settings.rs` | Add `sort_direction` to KeyBindings |
| `src/config/user_config.rs` | Add `sort_direction` to KeyBindingsConfig |
| `src/ui/renderers/footer.rs` | Update footer hints with sort key |
| `src/ui/views/help_view.rs` | Update Sorting section |
| `README.md` | Update keybindings table |

## Implementation Tasks

- [x] Remove `KeyCode::Left/Right` sort toggle from `servers.rs` (arrows always pane nav)
- [x] Use `KeyMap` for pane navigation in `servers.rs` (includes arrow keys)
- [x] Use `KeyMap` for pane navigation in `tools.rs` (includes arrow keys)
- [x] Add `KeyCode::Char('a')` sort toggle to `servers.rs`
- [x] Add `sort_direction` KeyAction and KeyMatcher to `keymap.rs`
- [x] Add `sort_direction` to KeyBindings in `settings.rs` and `user_config.rs`
- [x] Keep `h`/`l` customizable (unchanged)
- [x] Update footer hint to show sort direction key
- [x] Update README keybindings table
- [x] Update Help view

## Expected Behavior After Change

| Key | Action |
|------|------|
| `h` / `←` | Move to previous pane |
| `l` / `→` | Move to next pane |
| `a` | Toggle sort direction (Asc ↔ Desc) |

---

## Acceptance Criteria

- [x] `←`/`→` only performs pane navigation (no sort toggle) - **non-customizable**
- [x] `h`/`l` remain customizable for pane navigation
- [x] `a` key toggles sort direction
- [x] Footer shows hint for sort direction key
- [x] README keybindings table reflects changes
