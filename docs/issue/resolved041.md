# Issue 041 - Filter Performance Optimization & Sort State Display

## Summary

Two UX improvements for the servers view table:
1. **Filter performance**: Optimize filter operations that cause lag on every keystroke (IMPLEMENTED)
2. **Sort indicator**: Display current sort direction (↑/↓) in table headers (IMPLEMENTED)

---

## Issue 1: Filter Performance (IMPLEMENTED)

### Bottlenecks Identified (BEFORE)

| Issue | Location | Complexity | Impact |
|-------|----------|------------|--------|
| Cache invalidation on every keystroke | `app_state.rs:184` | O(1) | Triggers full recomputation every time |
| O(n²) fuzzy matching | `app_state.rs:845, 897-902` | O(n²) | `generate_fuzzy_variants()` iterates ALL servers for EACH server being filtered |
| Repeated `to_lowercase()` in sort | `app_state.rs:859-868` | O(n log n) | ~10,000 calls for 1000 servers |
| Repeated `to_lowercase()` in filter | `app_state.rs:839-840` | O(n) | Per-server, per-keystroke |

### Implemented Optimizations

| Optimization | File | Effect |
|--------------|------|--------|
| **Pre-computed lowercase fields** | `vpn/types.rs`, `vpn/cache.rs` | `code_lower`, `country_lower` - eliminates all runtime to_lowercase() during sort/filter |
| **Fuzzy variants pre-computation** | `app_state.rs:832-836, 891-913` | O(n²) → O(n): compute variants once, pass to filter |
| **Query length optimization** | `app_state.rs:841-842` | Skip City search for 1-2 char queries |

### Final Architecture

```
User types key
    ↓
handle_filter_input() [app.rs:798-815]
    ↓
set_search_query() [app_state.rs:182-194]
    ↓
invalidate_filtered_cache() → version++
    ↓
filtered_servers() called during render
    ↓
compute_filtered_servers() [app_state.rs:827]
    ↓
1. Get pre-computed fuzzy variants (O(n))
2. Filter using code_lower/country_lower (no to_lowercase!)
3. Sort using code_lower/country_lower (no to_lowercase!)
```

### Performance Improvements

| Metric | Before | After |
|--------|--------|-------|
| Fuzzy matching complexity | O(n²) per keystroke | O(n) per keystroke |
| to_lowercase() calls (1000 servers, sort) | ~10,000 | 0 |
| Filter latency | Visible lag | Instant |

### Changes Made

| File | Change |
|------|--------|
| `src/vpn/types.rs` | Added `code_lower`, `country_lower` fields to `Server` struct |
| `src/vpn/cache.rs` | Populate lowercase fields in `countries_to_servers()` |
| `src/state/app_state.rs` | Pre-compute fuzzy variants, use lowercase fields, skip City for short queries |
| `src/ui/app.rs` | Removed debouncing - instant filter application |

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

| File | Purpose | Key Lines |
|------|---------|-----------|
| `src/state/app_state.rs` | Filter logic, caching | 182-194, 816-873, 891-913 |
| `src/state/ui_state.rs` | SearchQuery, ServerCache | 88-119, 121-165 |
| `src/state/server_sort.rs` | Sort state types | - |
| `src/ui/components/pane_table.rs` | Table header rendering | - |
| `src/ui/views/servers_view.rs` | Servers view rendering | - |
| `src/ui/app.rs` | Filter input handling | 798-815 |
| `src/vpn/types.rs` | Server struct definition | 51-55 |
| `src/vpn/cache.rs` | Server creation | 102-114 |

---

## Priority

- **Filter performance**: High (IMPLEMENTED)
- **Sort indicator**: Medium (IMPLEMENTED)

---

## Testing

```
cargo test: 131 tests passed
cargo clippy: No warnings
```

---

## Notes

- Initial debouncing approach was replaced with direct application for better UX (filter applies immediately on each keystroke)
- Pre-computed lowercase fields add minimal memory overhead (~2x string storage) but eliminate all runtime conversion costs
- City search is skipped for queries ≤2 characters for faster initial filtering
