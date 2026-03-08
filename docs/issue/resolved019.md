# issue019: Bug - Footer inconsistency across views

## Summary

The footer displays inconsistently across different views, with some views missing action hints or having formatting issues.

## Decision (Resolved)

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Filter input box at **top** (header area) | Bottom会被footer挡住，视觉上在header更自然 |
| 2 | `[c] connect` 常时表示 | 常に表示 |
| 3 | `[d] disconnect` 非接続時のみ表示 | 接続中は意味がない |
| 4 | **全viewでtrailing separatorなし** | 見た目统一のため |

## Implementation Details

### 1. Filter Input Position

Currently: Bottom (lines 462-474 in `render()`)
```
┌─────────────────────────────┐
│ Header                      │
├─────────────────────────────┤
│ Main Content                │
├─────────────────────────────┤
│ [Filter input here]    ← NOW│
├─────────────────────────────┤
│ [?] help [Tab] switch ...   │
└─────────────────────────────┘
```

Changed: Between header and main content
```
┌─────────────────────────────┐
│ Header                      │
├─────────────────────────────┤
│ [Filter input here]    ← NEW│
├─────────────────────────────┤
│ Main Content                │
├─────────────────────────────┤
│ [?] help [Tab] switch ...   │
└─────────────────────────────┘
```

### 2. Connection State in Footer

| Connection State | `c` (connect) | `d` (disconnect) |
|------------------|---------------|------------------|
| Disconnected     | ✓ Show        | ✗ Hide           |
| Connecting       | ✗ Hide        | ✓ Show           |
| Connected        | ✗ Hide        | ✓ Show           |
| Disconnecting    | ✓ Show        | ✗ Hide           |
| Error            | ✓ Show        | ✗ Hide           |

### 3. Separator Removal

All views will have **no trailing separator**:
- Servers/Countries: `[?] help [Tab] switch [q] quit [j/k] navigate [l/Enter] cities [c] connect [r] refresh [s] sort [f] field [/] filter`
- Servers/Cities: `[?] help [Tab] switch [q] quit [j/k] navigate [c/Enter] connect [h/Backspace] countries`
- Settings: `[?] help [Tab] switch [q] quit [j/k] move [Enter] toggle/input [Space] off`
- Logs: `[?] help [Tab] switch [q] quit [j/k] scroll`
- Help: `Press Tab or q to return`
- Filter Input: `[Enter] apply [Esc] cancel`

---

## Current Coverage Analysis

| View | Pane | Has Footer? |
|------|------|------------|
| Servers | Countries | ✓ Complete |
| Servers | Cities | ✓ Complete |
| Settings | - | ✓ Complete |
| Logs | - | ✓ (minimal: scroll only) |
| Help | - | ✓ (static text) |
| FilterInput | - | ✗ Missing (need to add) |

---

## Solution

1. Move filter input to top (header area)
2. Add filter input mode footer hints
3. Remove all trailing separators
4. Show/hide connect/disconnect based on connection state

---

## Acceptance Criteria

- [x] Filter input box moved to top (header area)
- [x] Filter mode shows proper input hints: `[Enter]` apply, `[Esc]` cancel
- [x] Footer formatting is consistent (no trailing separators)
- [x] `[c] connect` always shown
- [x] `[d] disconnect` shown only when not connected
