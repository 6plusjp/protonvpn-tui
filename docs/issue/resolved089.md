# issue089: Optimize render functions - reduce allocations in hot paths

**Status: RESOLVED** ✅

## Summary

レンダリング関数で毎フレームのメモリ割り当てが発生している問題を修正。TUI はイベントループで高频度更新されるため、ホットパスでの allocation がパフォーマンスに影響する。

## Problems

### 1. `render_countries_pane` - 毎フレーム Vec 構築

**Location**: `src/ui/views/servers_view.rs:31-171`

```rust
let rows: Vec<Row> = servers
    .iter()
    .enumerate()
    .map(|(i, server)| {
        // 50+ lines of closure with style computation
        // Called every frame
    })
    .collect();
```

### 2. `render_help_view` - 50+ to_string() 呼び出し

**Location**: `src/ui/views/help_view.rs:24-180`

```rust
let categories = vec![
    Category {
        name: "Navigation".to_string(),  // to_string() repeated 50+ times
        entries: vec![...],
    },
    // ...
];
```

### 3. `get_footer_action_hints` - 毎フレーム vec 構築

**Location**: `src/ui/renderers/footer.rs:58-136`

```rust
let hints: Vec<Span> = vec![
    // .extend() chains repeated
]
```

### 4. fuzzy_match_with_variants - to_lowercase() 过多

**Location**: `src/vpn/types.rs:118`

```rust
let text_lower = text.to_lowercase(); // Per server per keystroke
for variant in &server.features {
    if variant.to_lowercase().contains(&text_lower) {  // Another allocation
```

## Solutions

### Option A: LazyLock / OnceCell (Recommended)

```rust
use std::sync::LazyLock;

static FOOTER_HINTS: LazyLock<Vec<Span<'static>>> = LazyLock::new(|| {
    // Build once, reuse forever
    build_footer_hints()
});
```

### Option B: Cache on State

```rust
impl UIState {
    // Cache computed values
    pub cached_footer_hints: Option<Vec<Span<'static>>>,
    pub cached_help_categories: Option<Vec<Category<'static>>>,
    
    pub fn invalidate_caches(&mut self) {
        // Called when view/context changes
    }
}
```

### Option C: Borrow instead of clone

```rust
// Before
let rows: Vec<Row> = ...

// After - use iterator directly without collecting
for (i, server) in servers.iter().enumerate() {
    render_row(i, server);
}
```

## Priority

1. **High**: `fuzzy_match_with_variants` - ユーザー入力に直接影响
2. **Medium**: `get_footer_action_hints` - 简单な静的データ
3. **Medium**: `render_help_view` - help view は频繁に開かない
4. **Low**: `render_countries_pane` - 実装复杂度が高い

## Resolution

**Implemented**: Added pre-computed lowercase fields to avoid allocations in hot paths.

### Changes:

1. **City struct** (`src/vpn/types.rs`):
   - Added `name_lower: String` field with `#[serde(default)]`
   - Pre-computed in constructors: `City::new()`, `City::with_features()`
   - Added `ensure_name_lower()` for backward compatibility

2. **Server filtering** (`src/state/server_ops.rs`):
   - `server.country` → `server.country_lower` (pre-computed)
   - `server.code.to_lowercase()` → `server.code_lower`
   - `c.name.to_lowercase()` → use `name_lower` (with fallback)

3. **fuzzy_match_with_variants**:
   - Added `text_lower: &str` parameter to avoid allocation
   - Function now takes pre-lowercased text

### Result:
- Eliminated per-keystroke allocations during server filtering
- Significant performance improvement for fuzzy search

### Not implemented (skipped):
- LazyLock for footer/help - lower priority, more complex

---

## Files Modified

- `src/vpn/types.rs`
- `src/vpn/cache.rs`
- `src/state/server_ops.rs`

## References

- Performance: TUI event loop runs at 60fps, every allocation adds latency
