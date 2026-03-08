# issue024: Feature - Lazy country/city loading on cursor movement

## Summary

Load cities automatically when user navigates to a country via cursor movement (j/k), not just on Enter/l.

## Problem

Currently, cities are only loaded when user presses `Enter` or `l` on a country. This means:
1. User must explicitly select country to see cities
2. Cities can't be preloaded for faster navigation

## Solution

### Flow

```
[j/k] でカーソル移動 → 国が変わる
    ↓
current_country_code が変わったかチェック
    ↓
├─ 変わった → キャッシュチェック
│           ├─ キャッシュあり → current_cities = キャッシュ（spinnerなし）
│           └─ キャッシュなし → fetch開始 + spinner表示
└─ 変わらない → 何もしない
    ↓
[Enter/l] → 强制更新（常にfetch）
```

### Visual Example

```
┌─────────────────────────────────────────────────┐
│ Countries         │ Cities                     │
├────────────────────┼───────────────────────────┤
│ > Japan           │ Tokyo, Osaka              │  ← キャッシュあり、spinnerなし
│   United States   │ New York ★ ─── spinner    │  ← US選択、キャッシュなし→fetch開始
│   Germany         │ Berlin                    │
└────────────────────┴───────────────────────────┘
```

### Loading Rules

| Action | Condition | Result |
|--------|-----------|--------|
| Cursor moves (j/k) | Country changed + cache empty | Fetch cities + show spinner |
| Cursor moves (j/k) | Country changed + cache exists | Use cached (no fetch, no spinner) |
| Cursor moves (j/k) | Country unchanged | Do nothing |
| Enter/l pressed | Always | Force fetch (force refresh) |

### Implementation

1. **In `select_next()` / `select_prev()`**:
   - After cursor position changes, check if country changed
   - If changed, check cache via `vpn_state.list_cities_with_features()`
   - If cached exists, set `current_cities` from cache
   - If cache empty, trigger async fetch

2. **In `move_to_cities()`**:
   - Always trigger fetch (force refresh) - existing behavior

3. **In servers_view.rs**:
   - Show spinner when `pending_cities` is set for current country

### Files to Modify

- `src/state/app_state.rs`
  - Modify `select_next()` / `select_prev()` to switch cities pane and trigger fetch
  - Modify `move_to_cities()` to always force refresh

- `src/ui/views/servers_view.rs`
  - Add spinner in cities pane when loading

---

## Acceptance Criteria

- [x] Moving cursor to new country switches cities pane
- [x] Empty cities triggers automatic fetch with spinner
- [x] Cached cities used without spinner
- [x] Works with both j/k navigation
- [x] Enter/l forces city refresh

---

## Implementation Notes

### Files Changed

- `src/state/app_state.rs`
  - Added `switch_cities_to_selected()` method
  - Added `load_cities_async()` method
  - Modified `select_next()` / `select_prev()` to call `switch_cities_to_selected()`
  - Made `pending_cities` and `pending_cities_country` fields `pub(crate)`

- `src/ui/views/servers_view.rs`
  - Modified `render_cities_pane()` to show loading state based on `pending_cities`
