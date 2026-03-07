# issue013: Split-pane Navigation (Servers + Cities Side-by-Side)

## Summary

Implement split-pane view where Servers list remains visible on the left, and Cities panel opens on the right when a server is selected.

## Current Behavior

1. User selects a server in Servers view
2. User presses Enter
3. Full view switches to Cities view
4. User presses Esc to return to Servers
5. Cursor resets to top (see issue011)

## Desired Behavior

1. User selects a server in Servers view
2. User presses Enter
3. Cities panel appears on the RIGHT side of the screen (split-pane)
4. Servers list remains visible on the LEFT side
5. User can navigate between Servers (left) and Cities (right)
6. User presses Esc to close Cities panel
7. Servers view resumes with original selection preserved

## UI Mockup

```
┌─────────────────────────────────────────────────────────┐
│  Servers          │  Cities for JP                      │
├───────────────────┼─────────────────────────────────────┤
│  > JP #1          │  > Tokyo #1                        │
│    JP #2          │    Osaka #2                         │
│    US #1          │    Yokohama #3                      │
│    DE #1          │                                     │
│                   │                                     │
│  [j/k] navigate   │  [j/k] navigate  [Esc] close       │
└───────────────────┴─────────────────────────────────────┘
```

## Implementation Approach

### Option A: Single View with Conditional Rendering

Modify the Servers view to conditionally render a Cities panel on the right:
- Add field `show_cities_panel: bool` to `AppState`
- In `servers_view.rs`, render both lists using a split layout
- When server selected + Enter → set `show_cities_panel = true`
- When Esc pressed in cities panel → set `show_cities_panel = false`

**Pros**: Simpler, less state management
**Cons**: May need to adjust layout constraints dynamically

### Option B: New Combined View

Create a new `AppView::ServersWithCities` variant that handles the split-pane logic:
- View manages both lists simultaneously
- Independent selection state for each pane
- Tab or similar to switch focus between panes

**Pros**: Cleaner separation, follows existing pattern
**Cons**: More code changes

## Required Changes

1. **State changes** (`src/state/app_state.rs`):
   - Add `show_cities_panel: bool` field
   - Modify navigation logic for split-pane behavior

2. **UI changes** (`src/ui/views/servers_view.rs` or new file):
   - Modify layout to show split-pane
   - Render both servers and cities lists side-by-side

3. **Input handling** (`src/ui/app.rs`):
   - Enter key: toggle cities panel (if server selected)
   - Esc key: close cities panel (if open), otherwise normal behavior
   - Tab key: switch focus between Servers and Cities panes

## Related Issues

- resolved_issue011: Cursor resets when returning from Cities (RESOLVED)
- issue012: Notification timing bug (separate)

## Tags

- feature
- ui
- navigation
- split-pane
- ux
