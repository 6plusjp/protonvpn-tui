# issue088: Refactor handle_settings_pane_key - extract duplicated theme preview logic

**Status: RESOLVED** ✅

## Summary

`src/ui/input/tools.rs` の `handle_settings_pane_key` 関数 (241行) に重複したロジックが3回登場するため、リファクタリングが必要。

## Problems

### 1. 関数が長大 (241行)

```
src/ui/input/tools.rs:35-276
```

### 2. テーマプレビューロジックの重複

以下のコードブロックがほぼ同一の内容で3回登場 (lines 195-272):

```rust
// Theme preview logic - appears 3 times
if input.state.ui_state.settings_selected == Some(index) {
    if !is_theme_setting {
        // Non-theme: apply directly
    } else {
        // Theme setting: update preview
    }
}
```

### 3. 保守性の問題

- 重複変更時に3箇所の同步が必要
- 認知負荷が高い
- テストが書きにくい

## Solution

### Step 1: Extract `update_theme_preview` helper

```rust
fn update_theme_preview(
    state: &mut AppState,
    setting_key: &SettingKey,
    option_index: usize,
) -> bool {
    // Theme preview logic extracted here
}
```

### Step 2: Replace duplicated calls

```rust
// Before (3 duplicated blocks)
if input.state.ui_state.settings_selected == Some(8) {
    // Theme preview logic repeated
}
if input.state.ui_state.settings_selected == Some(9) {
    // Theme preview logic repeated  
}

// After
if let Some(idx) = input.state.ui_state.settings_selected {
    if let Some(key) = SettingKey::from_index(idx) {
        update_theme_preview(&mut input.state, &key, option_index);
    }
}
```

## Files to Modify

- `src/ui/input/tools.rs`

## Resolution

**Implemented**: Extracted `update_theme_preview()` helper function.

### Changes:
- Added `fn update_theme_preview(input: &mut InputState, key: &SettingKey)` in `src/ui/input/tools.rs`
- Replaced 4 duplicated code blocks with calls to the helper function

### Result:
- Lines: 338 → 324 (-14 lines)
- No functional changes - behavior preserved

---

## References

- Related: issue087 (cargo doc warnings) - already resolved
