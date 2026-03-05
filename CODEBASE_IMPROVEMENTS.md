# Codebase Improvements

## 完了済み ✅

| 項目                              | 作業量 | 備考                                          |
| --------------------------------- | ------ | --------------------------------------------- |
| 設定カウント計算の重複除去        | 小     | `get_settings_count()` キャッシュ化           |
| ConnectionState 重複の解決        | 中     | 分割済み                                      |
| 選択メソッドの統一的改善          | 小     | Navigatable trait 実装                        |
| 非同期処理の tokio 化             | 大     | AsyncTaskManager 実装済み                     |
| connect_random の非同期化         | 小     | async_tasks.rs                                |
| ファイルの分割                    | 中     | 模块分割済み                                  |
| AsyncTaskManager のメモリリーク   | 中     | Arc 化で解決                                  |
| Clippy warning の修正             | 小     | 対応済み                                      |
| Settings 呼び出しのキャッシュ     | 小     | OnceLock 採用                                 |
| notification_log の上限設定       | 小     | MAX_NOTIFICATION_LOG 追加                     |
| EnterAlternateScreen 重複         | 極小   | 対応済み                                      |
| マジック Numbers の定数化         | 小     | コード内定数化                                |
| テストの追加                      | 中     | 継続中                                        |
| VpnState API の対称性修正         | 小     | 対応済み                                      |
| generate_fuzzy_variants 最適化    | 小     | キャッシュ追加                                |
| help_view のキー更新              | 小     | 対応済み                                      |
| #1 filtered_servers キャッシュ化  | 小     | RefCell+versioning でO(3n log n) → O(n log n) |
| #2 AsyncTaskManager → std::thread | 中     | tokio 依存削除、std::thread + mpsc に置換     |
| #3 Graceful Shutdown              | 中     | Ctrl+C 検出後、TUI画面を正しく復元            |
| #4 ConnectionStats 削除           | 小     | 使われていないダミーメソッドを削除            |
| #5 constants.rs 作成と集約        | 小     | 新規 src/constants.rs に全定数を集約          |
| #7 エラー処理 DRY 化              | 小     | check_cli_error() ヘルパーに抽出              |
| #9 ServerFeatures ドキュメント    | 小     | types.rs にCLI非対応事項を記載                |
| #11 VpnState API 非対称性         | 小     | 設計上の理由（キャッシュ更新のため）          |
| #12 Error Response の統一         | 小     | status() の AppResult → String 変更           |
| #15 proton_settings_cache 最適化  | 小     | get_or_init 事前チェック追加                  |

---

## 未完了 (優先度順)

### 🟢 低優先度

| #   | 項目              | 作業量 | 詳細                                                  | ステータス |
| --- | ----------------- | ------ | ----------------------------------------------------- | ---------- |
| 13  | AppState の巨大化 | 大     | 12+ フィールドを DomainState/UiState/DataState に分割 | 未着手     |

---

## メモ

- `#2` と `#3` は組み合わせて実装可能（thread 使用時に signal ハンドリング追加）
- `#13` (AppState 巨大化) はリファクタリングコストが高く、優先度を下げるべき
- tokio 削除により `Cargo.toml` から `tokio = { version = "1", features = ["full"] }` を削除済み
