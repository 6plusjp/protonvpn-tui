# issue022: Feature - Settings and Logs UI consolidation + Enhanced log viewer

## Summary

Enhance Logs view with scroll support, copy-to-clipboard, and more detailed messages.

## Updated Requirements (2026-03-09)

### 1. Make logs scrollable
- Currently logs list doesn't scroll when content exceeds viewport
- Need to enable vertical scrolling with arrow keys

### 2. Copy to clipboard on Enter
- When cursor is on a log entry and Enter is pressed, copy full message to clipboard

### 3. Sort by newest first
- Logs should display with newest entries at the top (reverse chronological order)

### 4. More detailed log messages
- Replace generic messages with detailed ones:
  - `"Connecting..."` → `"Connecting to Japan..."` (use actual server/city name)
  - `"Disconnecting..."` → `"Disconnecting from Japan..."`

---

## Part 1: Settings + Logs Integration (Future)

### Problem

Settings and Logs are separate views, but they're both configuration/info views that could share UI patterns.

### Solution Options

| Option | Description | Complexity |
|--------|-------------|------------|
| **A) Tabbed view** | Single view with tabs: [Settings] [Logs] | Medium |
| **B) Side panel** | Settings on left, Logs on right | High |
| **C) Unified list** | Settings and Logs as list items in one view | Medium |
| **D) Settings with log toggle** | Settings view has "show logs" toggle | Low |

### Recommendation: Option A (Tabbed view)

Use existing tab/keybinding pattern:
- `Tab` switches between Settings and Logs in the combined view
- Maintain current keyboard shortcuts where possible
- Share footer hints appropriately

---

## Part 2: Full log content display

### Problem

Logs with long messages are truncated in the list view. Users cannot see the full content.

### Solution: Expand on selection

When user selects a log entry (via Enter or click):

1. **Popup modal**: Show full message in overlay
2. **Expand inline**: Expand the list item in place
3. **Copy support**: Allow copying full message to clipboard

### Implementation

```rust
// In logs_view.rs
pub fn render_logs_view(...) {
    // Show truncated list (current)
    // When selected + Enter:
    //   render_popup(full_message)
}
```

### UI Design

```
┌─────────────────────────────────────┐
│  Logs                        [Esc]  │
├─────────────────────────────────────┤
│  [INFO]  Connected to US-NY        │
│  [ERR]   Connection failed: ...      │
│  [OK]    Disconnected successfully │
│  ...                                │
├─────────────────────────────────────┤
│ Selected:                           │
│ ┌─────────────────────────────────┐ │
│ │ Connection failed: timeout      │ │
│ │ after 30s. Server not respondi.. │ │
│ │ [Copy] [Close]                  │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Keyboard Controls

- `Enter` / `o`: Open selected log details
- `y`: Copy full message to clipboard
- `Esc`: Close popup / return to list

---

## Related: Also fix issue018 (log persistence)

The log viewer is useless if logs aren't persisted. Ensure logs are saved to file so they're available on restart.

---

## Acceptance Criteria

- [x] Logs are scrollable with arrow keys
- [x] Enter key copies selected log to clipboard
- [x] Newest logs appear at top
- [x] Log messages are detailed (include server/city name)
- [ ] Settings and Logs accessible via tabs in unified view (postponed)
- [ ] Tab switching works with existing keys (postponed)
- [ ] Long log messages show full content on selection (postponed)
- [ ] Popup/expand is keyboard accessible (postponed)
- [ ] Logs persist across restarts (issue018) (postponed)

---

## Completed (2026-03-09)

| Feature | Commit |
|---------|--------|
| Detailed log messages | `afcff59` |
| Logs scrolling | `c002c2c` |
| Copy to clipboard | `8c91cf4` |
| Newest first | `0e670e3` |
