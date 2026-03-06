# issue006: Add Logs view with notification logs

## Summary

Add a new Logs view to display session logs and notifications, accessible via tab navigation.

## Problem Context (Codebase Analysis)

### Existing Architecture

**AppView navigation cycle** (`src/state/app_view.rs`):
```
Servers → Stats → Settings → (back to Servers)
```

Current implementation in `AppView::next()`:
```rust
pub fn next(&self) -> Self {
    match self {
        Self::Servers => Self::Stats,
        Self::Stats => Self::Settings,
        Self::Settings => Self::Servers,  // <-- Logs goes here
        Self::Help => Self::Servers,
        Self::Cities => Self::Servers,
    }
}
```

**Notification system already exists** (`src/state/app_state.rs:89-102, 214-230`):
```rust
pub enum NotificationType {
    Info,
    Success,
    Error,
}

pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
}
```

- `AppState.notification_log: Vec<Notification>` - already exists (line 143)
- Max 100 entries (`MAX_NOTIFICATION_LOG = 100` in constants)
- Automatically populated via `show_notification()` in `sync_connection_state()`

**View rendering pattern** (`src/ui/app.rs`):
- Views are pure functions (NOT structs)
- Signature: `pub fn render_xxx_view(state: &mut AppState, f: &mut Frame<'_>, area: Rect)`
- Dispatched in `render_main()` via `match state.current_view`

### Files to Modify

| File | Changes |
|------|---------|
| `src/state/app_view.rs` | Add `Logs` variant to `AppView` enum, update `next()`/`prev()` |
| `src/ui/views/mod.rs` | Export new `logs_view` module |
| `src/ui/views/logs_view.rs` | **Create new** - render function |
| `src/ui/app.rs` | Add case in `render_main()`, add keyboard handling if needed |

## Requirements

### 1. Logs View

- Display `state.notification_log` (already exists)
- Each entry shows: level (Info/Success/Error), message
- Scrollable list (newest at bottom)
- Read-only view (no selection needed)

### 2. Tab Navigation

Update the cycle:
```
Servers → Stats → Settings → Logs → Servers
```

Changes in `AppView`:
```rust
pub enum AppView {
    Servers,
    Stats,
    Settings,
    Logs,    // NEW
    Help,
    Cities,
}

// next(): Settings → Logs → Servers
// prev(): Logs → Settings → Stats
```

### 3. Notification Events (Already Logged)

The following events already generate notifications that are stored in `notification_log`:

| Event | NotificationType | When |
|-------|-----------------|------|
| Server refresh | Info | Success/failure |
| Connection | Success/Error | Connected/disconnected |
| Disconnect | Info | Success |
| Cities loaded | Info | After fetch |
| Settings updated | Success | After toggle |

No additional logging code needed - just display existing `notification_log`.

## Technical Implementation

### New file: `src/ui/views/logs_view.rs`

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::state::{AppState, NotificationType};

pub fn render_logs_view(
    state: &AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let block = Block::default().title(" Logs ").borders(Borders::ALL);

    let items: Vec<ListItem> = state
        .notification_log
        .iter()
        .map(|n| {
            let (prefix, color) = match n.notification_type {
                NotificationType::Info => ("[INFO] ", Color::Cyan),
                NotificationType::Success => ("[OK]   ", Color::Green),
                NotificationType::Error => ("[ERR]  ", Color::Red),
            };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::raw(&n.message),
            ]);
            ListItem::new(line)
        })
        .collect();

    if items.is_empty() {
        let empty_list = List::new(vec![ListItem::new("No logs yet")])
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_list, area);
        return;
    }

    let selected = list_state.selected().unwrap_or(0).min(items.len() - 1);
    list_state.select(Some(selected));

    let list = List::new(items).block(block);

    f.render_stateful_widget(list, area, list_state);
}
```

### Modifications

**`src/state/app_view.rs`:**
```rust
pub enum AppView {
    Servers,
    Stats,
    Settings,
    Logs,    // Add
    Help,
    Cities,
}

impl AppView {
    pub fn next(&self) -> Self {
        match self {
            Self::Servers => Self::Stats,
            Self::Stats => Self::Settings,
            Self::Settings => Self::Logs,    // Change
            Self::Logs => Self::Servers,       // Add
            // ...
        }
    }
    // Same for prev()
}
```

**`src/ui/app.rs`:**
- Add `AppView::Logs` case in `render_main()`
- Keyboard handling: Tab already cycles via `next()`

## Acceptance Criteria

- [x] New Logs view exists and displays session logs
- [x] Tab navigation includes Logs: Servers → Stats → Settings → Logs → Servers
- [x] Logs persist during session (uses existing `notification_log`)
- [x] Each log entry shows level (Info/Success/Error) and message
- [x] Scrollable list works correctly
- [x] Level prefixes are aligned (6 chars: `[INFO] `, `[OK]   `, `[ERR]  `)
- [x] Each level has distinct color (Info=Cyan, Success=Green, Error=Red)

## Implementation Complete

All acceptance criteria have been implemented:

1. **Logs view created** (`src/ui/views/logs_view.rs`)
   - Displays `notification_log` with level prefix ([INFO], [OK], [ERR])
   - Shows "No logs yet" when empty
   - Uses ListState for scrollable selection
   - **Color coding**:
     - `[INFO] ` → Cyan
     - `[OK]   ` → Green
     - `[ERR]  ` → Red
   - **Alignment**: All prefixes padded to 6 characters

2. **Navigation updated** (`src/state/app_view.rs`)
   - Added `Logs` variant to `AppView` enum
   - Updated `next()`: Settings → Logs → Servers
   - Updated `prev()`: Servers → Logs → Settings

3. **Integration** (`src/ui/app.rs`)
   - Added `AppView::Logs` case in `render_main()`
   - Added footer keyboard hints for Logs view

## Notes

- No new logging infrastructure needed - reuse existing `notification_log`
- View is read-only (no selection state needed)
- Similar pattern to `stats_view.rs` which is also read-only
