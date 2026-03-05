# Codebase Improvements

## 解決済み

### ✅ 問題4: 手動キャッシュInvalidation (2026-03-05 解決)

**場所:** `src/state/app_state.rs`

**修正内容:**

1. フィールドを `pub` から `pub(crate)` に変更
2. setter メソッドを追加:

```rust
pub fn set_search_query(&mut self, query: String) {
    self.search_query = query;
    self.invalidate_filtered_cache();
}

pub fn set_servers(&mut self, servers: Vec<Server>) {
    self.servers = servers;
    self.invalidate_filtered_cache();
}
```

3. 既存の代入箇所を setter 呼叫に置換:
   - `sync_connection_state()` の `self.servers = servers`
   - `refresh_servers()` の `self.servers = cached`

**追加で発見・修正したバグ:**
- `sync_connection_state()` で servers を更新時にキャッシュを無効化してなかった

---

## 将来保留

- AppState 巨大化: フィールドグルーピングで整理 (2026-03-05完了)
