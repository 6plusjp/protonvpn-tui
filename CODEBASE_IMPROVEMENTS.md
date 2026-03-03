# Codebase Improvements

## 優先順位表

| 優先度 | 項目                            | 状態   | 作業量 |
| ------ | ------------------------------- | ------ | ------ |
| ✅     | 設定カウント計算の重複除去      | 完了   | 小     |
| ✅     | ConnectionState 重複の解決      | 完了   | 中     |
| ✅     | 選択メソッドの統一的改善        | 完了   | 小     |
| ✅     | 非同期処理の tokio 化           | 完了   | 大     |
| ✅     | connect_random の非同期化       | 完了   | 小     |
| ✅     | ファイルの分割                  | 完了   | 中     |
| ✅     | AsyncTaskManager のメモリリーク | 完了   | 中     |
| ✅     | Clippy warning の修正           | 完了   | 小     |
| **中** | Settings 呼び出しのキャッシュ   | 未着手 | 小     |
| **低** | notification_log の上限設定     | 未着手 | 小     |
| **低** | EnterAlternateScreen 重複       | 未着手 | 極小   |
| **低** | マジック Numbers の定数化       | 未着手 | 小     |
| **低** | テストの追加                    | 未着手 | 中     |
| **低** | VpnState API の対称性修正       | 未着手 | 小     |
| **低** | generate_fuzzy_variants 最適化  | 未着手 | 小     |
| **低** | help_view のキー更新            | 未着手 | 小     |
| **低** | AppState の分割 (God Object)    | 未着手 | 大     |

---

## 未完了の問題点

### 中優先度

#### #2: Clippy warning

- `src/state/app_state.rs:112`: 複雑な型 → type エイリアス
- `src/state/app_state.rs:274`: 不要な return
- `src/ui/app.rs:272`: `format!()` → `to_string()`
- `src/ui/app.rs:278`: `.max().min()` → `.clamp()`
- `src/ui/app.rs:432`: 不要な `Line::from()`
- `src/vpn/client.rs:154`: ネストした if の統合

#### #3: Settings 呼び出しのキャッシュ

`Settings::load_proton_settings()` が app.rs (8箇所) と views (毎フレーム) で重複呼び出し。
ファイル I/O が毎フレーム発生しているため、AppState にキャッシュを持たせる。

```rust
// AppState に追加
proton_settings_cache: OnceCell<ProtonSettings>,
```

---

### 低優先度

#### #4: notification_log の無制限成長

`Vec<Notification>` が永遠に成長する。上限 (100件) を設定。

#### #5: 重複した EnterAlternateScreen

`src/ui/app.rs:46-48` で2回呼び出し。

#### #6: マジック Numbers

- `page_size = 10`
- `notification_timer = 30`
- `max_len = 35`
- `popup_width .min(54)`

定数ファイルに分離。

#### #7: テストの不在

`tests/` ディレクトリなし。

#### #8: VpnState API の非対称性

`get_servers(&mut self)`、`list_servers(&mut self)` は `&self` で十分。

#### #9: generate_fuzzy_variants の HashMap

毎回 `HashMap::from([...])` を作成している。`lazy_static` を使用。

#### #10: help_view のキー不一致

- 不足: `x` (random connect), `g/G`, `Ctrl+d/u`
- 不要な説明が含まれている可能性

#### #11: AppState の巨大化 (God Object)

単一責任原則の違反。DomainState / UiState / OperationState に分割。
