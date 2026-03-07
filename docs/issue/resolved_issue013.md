# resolved_issue013: Yazi-style Split-pane Navigation

## Summary

Implement Yazi-inspired split-pane navigation where Servers list and Cities panel are always side-by-side.

## Status: RESOLVED ✅

## Overview

The original behavior required users to navigate between full-screen views (Servers → Cities), which caused cursor position to reset and was less efficient. The new implementation provides a persistent split-pane view.

## Changes Made

### 1. State Management (`src/state/app_state.rs`)

- Added `Pane` enum (Countries, Cities) to track focused pane
- Added `selected_city` for independent city selection
- Added `pane_focus: Pane` field
- Added `move_to_cities()` and `move_to_countries()` methods

### 2. View Rendering (`src/ui/views/servers_view.rs`)

- Always render split-pane layout (60% Countries / 40% Cities)
- Countries pane: shows `>` indicator only for selection
- Cities pane: shows normal selection highlight

### 3. Key Handling (`src/ui/app.rs`)

| Key | Action |
|-----|--------|
| `l` or `Enter` | Move focus from Countries to Cities |
| `h` or `Backspace` | Move focus from Cities to Countries |
| `j/k` | Navigate within focused pane |
| `c` | Connect (server in Countries, city in Cities) |
| `Tab` | Cycle views (Servers → Settings → Logs → Servers, position preserved) |
| `Esc` | Clear filter |

### 4. Footer

Dynamic key hints based on focus:
- **Countries focus**: `[j/k] navigate [l/Enter] cities [c] connect [d] disconnect [r] refresh [s] sort [f] field [h] countries`
- **Cities focus**: `[j/k] navigate [c/Enter] connect [h/Backspace] countries`

## Benefits

1. **Efficient navigation**: Stay in Servers view while browsing cities
2. **Position preservation**: Cursor position maintained when switching views
3. **Visual clarity**: Clear indication of focused pane
4. **Familiar UX**: Vim-style keybindings (h/j/k/l)

## Related Commits

- `5f8a8cb` - feat(state): add Pane enum and split-pane state management
- `4517326` - feat(ui): implement split-pane rendering for Servers view
- `19a7f69` - feat(ui): update key handling for split-pane navigation
- `ac0ee27` - docs(issue013): finalize specification after implementation

## Tags

- feature
- ui
- navigation
- split-pane
- ux
- yazi-style
- resolved
