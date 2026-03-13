# Issue 041 - Filter Performance Optimization & Sort State Display

## Summary

Two UX improvements for the servers view table:
1. **Filter performance**: Optimize filter operations that cause lag on every keystroke
2. **Sort indicator**: Display current sort direction (↑/↓) in table headers (IMPLEMENTED)

---

## Issue 1: Filter Performance (NOT IMPLEMENTED)

### Current Behavior

Every keystroke in filter mode triggers full recomputation:

```
src/state/app_state.rs:compute_filtered_servers()
```

**Bottlenecks identified:**

| Issue | Location | Impact |
|-------|----------|--------|
| Repeated `.to_lowercase()` | Lines 761-766, 781-790 | O(n) string allocations per keystroke |
| Fuzzy matching on every filter | Lines 767, 808-817 | O(n) variants generated per server |
| No debouncing | `handle_filter_input()` in app.rs | Recomputes on every single keystroke |
| Cloning on every filter | Lines 755, 775, 740 | Memory allocation overhead |

### Expected Behavior

Filter should feel responsive even with 1000+ servers. No visible lag on typing.

### Suggested Fix

1. **Query length-based filter optimization**:
   - 1-2 chars → search by `Code` only (fast, exact match)
   - 3+ chars → search by `Country` + fuzzy matching
   - Skip: City search (expensive)

2. **Add debouncing** (100-200ms) to filter input

3. **Replace fuzzy matching with pre-computed N-gram HashSet**

4. **Pre-compute lowercase versions** on server load

---

## Issue 2: Sort State in Table Header (IMPLEMENTED)

### Implemented

Table headers now display current sort direction:

```
Code ↑    Country    Cities     (ascending)
Code ↓    Country    Cities     (descending)
```

### Changes Made

| File | Change |
|------|--------|
| `src/ui/components/pane_table.rs` | Added `header_row_with_sort()` method |
| `src/ui/views/servers_view.rs` | Pass sort state to header, Code column width = 6 |
| `src/state/server_sort.rs` | Uses existing `label()` method returning `↑`/`↓` |

---

## Related Files

- `src/state/app_state.rs` - Filter logic (lines 744-806)
- `src/state/server_sort.rs` - Sort state types
- `src/state/ui_state.rs` - SearchQuery, ServerCache
- `src/ui/components/pane_table.rs` - Table header rendering
- `src/ui/views/servers_view.rs` - Servers view rendering
- `src/ui/app.rs` - Filter input handling (lines 656-704)

---

## Priority

- **Filter performance**: High (NOT IMPLEMENTED)
- **Sort indicator**: Medium (IMPLEMENTED)
