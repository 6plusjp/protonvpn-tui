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

---

## 未完了 (優先度順)

### 🔴 高優先度

| # | 項目 | 作業量 | 詳細 | ステータス |
|---|------|--------|------|------------|
| 1 | `filtered_servers()` の重複計算 | 小 | 1フレーム内で3回呼び出し (connect_view.rs:20, app_state.rs:329, app_state.rs:490等)。ソート処理 O(n log n) x 3 が重複 | 未着手 |
| 2 | AsyncTaskManager の不要Runtime | 中 | 全て `spawn_blocking` のみ使用 → `std::thread` で十分。main.rs は非同期不使用 | 未着手 |
| 3 | Graceful Shutdown の欠如 | 中 | Ctrl+C で即座に終了、TUI画面が復元されない。panic hook のみ存在 | 未着手 |
| 4 | ConnectionStats がダミーデータ | 小 | `vpn/client.rs:422-430` で bytes_sent=0, bytes_received=0 を返す。削除または procfs から取得 | 未着手 |

### 🟡 中優先度

| # | 項目 | 作業量 | 詳細 | ステータス |
|---|------|--------|------|------------|
| 5 | Constants の散在 | 小 | PAGE_SIZE (app_state.rs:28), NOTIFICATION_TIMER_* (app.rs:19-20), POPUP_WIDTH_* (app.rs:22-23) が散在 | 未着手 |
| 6 | 設定ファイルの未使用 | 小 | Settings 構造体は未使用 (config/settings.rs) | 未着手 |
| 7 | エラー処理重複 | 小 | `connect()` と `connect_random()` で同一のエラー検査ロジック重複 (client.rs:78-90, 116-127) | 未着手 |
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

### #1: `filtered_servers()` の重複計算

**呼び出し箇所（1フレーム内）:**

| ファイル | 行 | メソッド | 用途 |
|---------|-----|---------|------|
| `src/ui/views/connect_view.rs` | 20 | `render_connect_view()` | リスト描画 |
| `src/state/app_state.rs` | 329 | `connect()` | サーバー取得 |
| `src/state/app_state.rs` | 490 | `select_next()` | `.len()` |
| `src/state/app_state.rs` | 495 | `select_prev()` | `.len()` |
| `src/state/app_state.rs` | 500 | `select_first()` | `.len()` |
| `src/state/app_state.rs` | 504 | `select_last()` | `.len()` |
| `src/state/app_state.rs` | 509 | `select_page_down()` | `.len()` |
| `src/state/app_state.rs` | 514 | `select_page_up()` | `.len()` |

**改善案:**

```rust
// src/state/app_state.rs
use std::cell::{Cell, RefCell};

pub struct AppState {
    // ... 既存フィールド
    filtered_servers_cache: RefCell<Option<(Vec<Server>, u64)>>,
    filtered_servers_version: Cell<u64>,
}

impl AppState {
    fn invalidate_filtered_cache(&self) {
        self.filtered_servers_version.set(self.filtered_servers_version.get() + 1);
    }

    pub fn filtered_servers(&self) -> Vec<Server> {
        let version = self.filtered_servers_version.get();
        if let Some((ref cached, cached_version)) = *self.filtered_servers_cache.borrow() {
            if cached_version == version {
                return cached.clone();
            }
        }

        let result = self.compute_filtered_servers();
        *self.filtered_servers_cache.borrow_mut() = Some((result.clone(), version));
        result
    }

    fn compute_filtered_servers(&self) -> Vec<Server> {
        // 既存の filtered_servers() ロジックを移動
    }
}
```

**無効化タイミング:** `search_query`, `filter`, `sort`, `sort_direction`, `servers`, `connection` が変更されたとき

---

### #2: AsyncTaskManager の不要Runtime

**現状 (`src/state/async_tasks.rs:24-29`):**
```rust
let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .thread_name("protonvpn-async")
    .build()
    .expect("Failed to create tokio runtime");
```

**問題点:**
- 全て `spawn_blocking` のみ使用（CLI コマンドの同期実行）
- `main.rs` は非同期不使用
- Runtime 生成コスト、スレッドプール維持コスト

**改善案 A) std::thread に置換（推奨）:**
```rust
// src/state/async_tasks.rs
pub struct AsyncTaskManager;

impl AsyncTaskManager {
    pub fn spawn_connect(
        vpn_state: VpnState,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.connect(&server_id);
            let _ = sender.send(result);
        });
    }
}
```

**改善案 B) main.rs を async 化:**
```rust
// src/main.rs
#[tokio::main]
async fn main() -> io::Result<()> {
    // runtime はここで1回だけ作成
    let mut app = TuiApp::new()?;
    app.run().await?;
}
```

---

### #3: Graceful Shutdown

**現状:**
- `src/ui/app.rs:34-37` に panic hook のみ
- Ctrl+C で即座に終了、画面が復元されない

**改善案:**
```rust
// src/ui/app.rs
use std::signal::{Signal, SignalKind};

impl TuiApp {
    pub fn run(&mut self) -> io::Result<()> {
        // ... existing setup ...

        // SIGINT/SIGTERM ハンドリング用チャンネルの準備
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            use std::signal;
            signal::signal(SignalKind::terminate(), |_| {}).ok();
            signal::signal(SignalKind::interrupt(), |_| {
                shutdown_tx.send(()).ok();
            }).ok();
        });

        loop {
            terminal.draw(|f| self.render(f))?;
            // ...

            // シグナルチェック
            if shutdown_rx.try_recv().is_ok() {
                break;
            }
        }

        // クリーンアップ
        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
```

---

### #4: ConnectionStats ダミーデータ

**現状 (`src/vpn/client.rs:422-430`):**
```rust
pub fn stats(&self) -> AppResult<ConnectionStats> {
    Ok(ConnectionStats {
        bytes_sent: 0,           // 常に0
        bytes_received: 0,      // 常に0
        connected_at: self.cache.connected_at.unwrap_or(Utc::now()),
        server_ip: String::new(),
        protocol: String::from("WireGuard"),
    })
}
```

**改善案:**

**案A: 削除（推奨）**
```rust
pub fn stats(&self) -> AppResult<Option<ConnectionStats>> {
    Ok(None)  // プロトンは未対応
}
```

**案B: /proc/net/dev から実測（Linux のみ）**
```rust
pub fn stats(&self) -> AppResult<ConnectionStats> {
    let content = std::fs::read_to_string("/proc/net/dev")?;
    // proton0 の送受信バイト数を解析
    // ...
}
```

---

### #5: Constants の散在

**現在の散在箇所:**

| 定数 | ファイル | 行 |
|------|---------|-----|
| `PAGE_SIZE` | `src/state/app_state.rs` | 28 |
| `NOTIFICATION_TIMER_DEFAULT` | `src/ui/app.rs` | 19 |
| `NOTIFICATION_TIMER_SHORT` | `src/ui/app.rs` | 20 |
| `NOTIFICATION_MSG_MAX_LEN` | `src/ui/app.rs` | 21 |
| `POPUP_WIDTH_MIN` | `src/ui/app.rs` | 22 |
| `POPUP_WIDTH_MAX` | `src/ui/app.rs` | 23 |
| `MAX_NOTIFICATION_LOG` | `src/state/app_state.rs` | 177 |

**改善案:**

```rust
// src/constants.rs (新規作成)
pub mod ui {
    pub const NOTIFICATION_TIMER_DEFAULT: u8 = 30;
    pub const NOTIFICATION_TIMER_SHORT: u8 = 15;
    pub const NOTIFICATION_MSG_MAX_LEN: usize = 35;
    pub const POPUP_WIDTH_MIN: usize = 30;
    pub const POPUP_WIDTH_MAX: usize = 54;
}

pub mod state {
    pub const PAGE_SIZE: usize = 10;
    pub const MAX_NOTIFICATION_LOG: usize = 100;
}

pub mod vpn {
    pub const DISCONNECT_RETRY_COUNT: usize = 10;
    pub const DISCONNECT_RETRY_DELAY_MS: u64 = 500;
}
```

---

### #7: エラー処理重複

**重複パターン (`src/vpn/client.rs`):**

```rust
// connect() - 行 78-90
let combined_lower = combined.to_lowercase();
if combined_lower.contains("error:") {
    return Err(AppError::ConnectionFailed(...));
}
if !output.status.success() {
    return Err(AppError::ConnectionFailed(...));
}

// connect_random() - 行 116-127 (同一パターン)
```

**改善案:**
```rust
// src/vpn/client.rs
fn check_cli_error(&self, output: &std::process::Output) -> AppResult<()> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("error:") || !output.status.success() {
        return Err(AppError::ConnectionFailed(
            format!("{}\n{}", stdout.trim(), stderr.trim())
        ));
    }
    Ok(())
}
```

---

### #14: fuzzy_match の to_lowercase 重複

**現状:**

```rust
// src/state/app_state.rs
fn fuzzy_match(&self, text: &str, query: &str) -> bool {
    let text_lower = text.to_lowercase();   // 1回目
    let query_lower = query.to_lowercase();  // 2回目
    // ...
}

fn generate_fuzzy_variants(&self, query: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();  // 3回目
    // ...
}
```

**改善案:** キャッシュまたは引数で渡す

---

## 優先度付けの建議

| 優先度 | 項目 | 作業量 | 期待効果 |
|--------|------|--------|----------|
| 1 | #1 filtered_servers キャッシュ | 小 | パフォーマンス向上 (O(3n log n) → O(n log n)) |
| 2 | #2 AsyncTaskManager 簡素化 | 中 | 依存関係削除、コンパイル時間短縮 |
| 3 | #3 Graceful Shutdown | 中 | UX 向上 |
| 4 | #5 Constants 集約 | 小 | 保守性向上 |
| 5 | #7 エラー処理 DRY | 小 | 保守性向上 |

**削除推奨:** #4 (価値がないコードを維持するコスト > 削除コスト)

---

## タスク化管理

```todo
- [ ] #1 filtered_servers キャッシュ化
- [ ] #2 AsyncTaskManager → std::thread 置換
- [ ] #3 Graceful Shutdown 実装
- [ ] #5 constants.rs 作成と集約
- [ ] #7 エラー処理 DRY 化
- [ ] #4 ConnectionStats 削除 (または procfs 実装)
```

---

## メモ

- `#2` と `#3` は組み合わせて実装可能（thread 使用時に signal ハンドリング追加）
- `#9` (ServerFeatures) は `protonvpn-cli` の制限により実現不能。CODEBASE_IMPROVEMENTS.md から削除して README 等で完結
- `#13` (AppState 巨大化) はリファクタリングコストが高く、優先度を下げるべき
