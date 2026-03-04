# Codebase Improvements

## 完了済み ✅

| 項目 | 作業量 | 備考 |
| ---- | ------ | ------ |
| 設定カウント計算の重複除去 | 小 | `get_settings_count()` キャッシュ化 |
| ConnectionState 重複の解決 | 中 | 分割済み |
| 選択メソッドの統一的改善 | 小 | Navigatable trait 実装 |
| 非同期処理の tokio 化 | 大 | AsyncTaskManager 実装済み |
| connect_random の非同期化 | 小 | async_tasks.rs |
| ファイルの分割 | 中 | 模块分割済み |
| AsyncTaskManager のメモリリーク | 中 | Arc 化で解決 |
| Clippy warning の修正 | 小 | 対応済み |
| Settings 呼び出しのキャッシュ | 小 | OnceLock 採用 |
| notification_log の上限設定 | 小 | MAX_NOTIFICATION_LOG 追加 |
| EnterAlternateScreen 重複 | 極小 | 対応済み |
| マジック Numbers の定数化 | 小 | コード内定数化 |
| テストの追加 | 中 | 継続中 |
| VpnState API の対称性修正 | 小 | 対応済み |
| generate_fuzzy_variants 最適化 | 小 | キャッシュ追加 |
| help_view のキー更新 | 小 | 対応済み |
| #1 filtered_servers キャッシュ化 | 小 | RefCell+versioning でO(3n log n) → O(n log n) |
| #2 AsyncTaskManager → std::thread | 中 | tokio 依存削除、std::thread + mpsc に置換 |
| #3 Graceful Shutdown | 中 | Ctrl+C 検出後、TUI画面を正しく復元 |
| #4 ConnectionStats 削除 | 小 | 使われていないダミーメソッドを削除 |
| #5 constants.rs 作成と集約 | 小 | 新規 src/constants.rs に全定数を集約 |
| #7 エラー処理 DRY 化 | 小 | check_cli_error() ヘルパーに抽出 |

---

## 未完了 (優先度順)

### 🟡 中優先度

| # | 項目 | 作業量 | 詳細 | ステータス |
|---|------|--------|------|------------|
| 6 | 設定ファイルの未使用 | 小 | Settings 構造体は未使用 (config/settings.rs) | 未着手 |
| 8 | テストの不足 | 中 | フィルタリング、パース、UI のテスト不足 | 未着手 |

### 🟢 低優先度

| # | 項目 | 作業量 | 詳細 | ステータス |
|---|------|--------|------|------------|
| 9 | ServerFeatures がデフォルト | 小 | `protonvpn-cli` が機能を提供していないため取得不能。ドキュメント化即可 | 未着手 |
| 10 | マジック Numbers | 小 | client.rs にリトライ回数 (10), 遅延 (500ms) などがハードコード | 未着手 |
| 11 | VpnState API の非対称性 | 小 | connect=mut, disconnect=mut だが get_servers=immutable | 未着手 |
| 12 | Error Response の統一 | 小 | AppError/String/Result が混在 | 未着手 |
| 13 | AppState の巨大化 | 大 | 12+ フィールドを DomainState/UiState/DataState に分割 | 未着手 |
| 14 | fuzzy_match の to_lowercase 重複 | 小 | generate_fuzzy_variants + fuzzy_match で同一文字列を2回 lower ケース変換 | 未着手 |
| 15 | proton_settings_cache の同期 | 小 | `get_or_init` が毎フレーム呼ばれる可能性 | 未着手 |

---

## 詳細

### #2: AsyncTaskManager の不要Runtime (完了)

**実装内容:**
- tokio 依存を Cargo.toml から削除
- `AsyncTaskManager` を `std::thread::spawn` + `std::sync::mpsc` で再実装
- チャンネルバッファサイズ指定を削除（mpsc::channel のデフォルトを使用）

**効果:**
- 依存crate 1つ削除
- コンパイル時間短縮
- バイナリサイズ若干減少

---

### #3: Graceful Shutdown (完了)

**実装内容:**
- `app.rs` のイベントループで Ctrl+C (KeyModifiers::CONTROL) を検出
- 検出時は通常通りループを抜け出し、`LeaveAlternateScreen` + `disable_raw_mode()` を実行

**効果:**
- Ctrl+C 時にもTUI画面が正しく復元される

---

### #4: ConnectionStats ダミーデータ (完了)

**実装内容:**
- `client.rs` の `stats()` メソッドを削除
- `ConnectionStats` types 定義は残しておく（将来の実装のため）

---

### #5: Constants の散在 (完了)

**実装内容:**
- 新規 `src/constants.rs` を作成
- 3つのモジュールに分類:
  - `ui`: NOTIFICATION_TIMER_*, POPUP_WIDTH_*, NOTIFICATION_MSG_MAX_LEN
  - `state`: PAGE_SIZE, MAX_NOTIFICATION_LOG
  - `vpn`: DISCONNECT_RETRY_COUNT, DISCONNECT_RETRY_DELAY_MS

**効果:**
- 定数の散在を解決
- 保守性向上

---

### #7: エラー処理重複 (完了)

**実装内容:**
- `client.rs` に `check_cli_error()` ヘルパーメソッドを追加
- `connect()` と `connect_random()` で重複していたエラーチェロジックを統合

---

## タスク化管理

```todo
- [x] #1 filtered_servers キャッシュ化
- [x] #2 AsyncTaskManager → std::thread 置換
- [x] #3 Graceful Shutdown 実装
- [x] #4 ConnectionStats 削除
- [x] #5 constants.rs 作成と集約
- [x] #7 エラー処理 DRY 化
```

---

## メモ

- `#2` と `#3` は組み合わせて実装可能（thread 使用時に signal ハンドリング追加）
- `#9` (ServerFeatures) は `protonvpn-cli` の制限により実現不能。CODEBASE_IMPROVEMENTS.md から削除して README 等で完結
- `#13` (AppState 巨大化) はリファクタリングコストが高く、優先度を下げるべき
- tokio 削除により `Cargo.toml` から `tokio = { version = "1", features = ["full"] }` を削除済み
