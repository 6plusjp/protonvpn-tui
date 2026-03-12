# issue038 - Logs View UI Improvements

## Summary

Enhance the Logs view with table layout, header, and improved column ordering.

## Background

Current implementation uses a simple `List` widget without header. After issue022, logs are scrollable and selectable, but the UI can be improved for better readability and consistency with `servers_view`.

## Current State

```
> 2m   [ERR]  Connection failed to Japan...
   5m   [OK]   Connected to Japan #3...
   10m   [INFO] Refreshing servers...
```

- Widget: `List` (not `Table`)
- Time column: Positioned after selection indicator (2nd)
- No header
- Selection shows `>` prefix only

## Proposed Changes

### 1. Convert to Table with Header

Replace `List` widget with `Table` widget (consistent with `servers_view`).

```
+----------+--------------------------------+----------+
| Type     | Message                        | Time     |
+----------+--------------------------------+----------+
| [ERR]    | Connection failed to Japan...  | 2m       |
| [OK]     | Connected to Japan #3...       | 5m       |
| [INFO]   | Refreshing servers...          | 10m      |
+----------+--------------------------------+----------+
```

### 2. Reorder Columns

Move time to the last column (least important for user).

- Current: `> time [TYPE] message`
- Proposed: `[TYPE] message time`

### 3. Add Header Row

| Column | Width Strategy | Alignment |
|--------|----------------|-----------|
| Type | Fixed 6 (`[INFO]`) | Left |
| Message | Flexible (remaining space) | Left |
| Time | Dynamic based on max content + 1 padding | Left |

> Note: Time column uses left alignment with consistent width for visual consistency.

## Acceptance Criteria

- [x] Logs view uses `Table` widget instead of `List`
- [x] Header row displays: `| Type | Message | Time |`
- [x] Time column is last (rightmost)
- [x] Time column is right-aligned in header
- [x] Time values are left-aligned (consistent width)
- [x] Selection highlight works with table layout (using `highlight_symbol`)
- [x] Column spacing is 2 (consistent with servers_view)
- [x] Consistent with `servers_view` table patterns
- [x] Build passes

## Related

- issue022: Enhanced Logs View (scroll, selection, detailed messages)
- servers_view.rs: Reference implementation for Table pattern

---

## Notes (2026-03-12)

### 未実装（別のissueで検討）

- Long log messages show full content on selection (issue022 未完了)
- Delete selected log / Clear all logs
- Filter by type (Info/Success/Warning/Error)
