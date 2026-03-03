# Codebase Analysis & Improvement Proposals

## 概要

本ドキュメントは、protonvpn-tui コードベースの構造的・状態管理に関する問題を分析し、改善案を提示する。

---

## 完了した改善

### ✅ 問題点 3: 設定カウント計算の重複 (app.rs)

**状態**: 完了

`src/config/settings.rs` に `ProtonSettings::settings_count()` メソッドを追加し、app.rs から7箇所の重複コードを削除した。

```rust
// src/config/settings.rs
impl ProtonSettings {
    pub fn settings_count(&self) -> usize {
        let mut c = 0;
        if self.killswitch.is_some() { c += 1; }
        if self.ipv6.is_some() { c += 1; }
        if self.custom_dns.enabled || self.custom_dns.ip_list.is_empty() { c += 1; }
        if self.features.as_ref().and_then(|f| f.netshield).is_some() { c += 1; }
        if self.features.as_ref().and_then(|f| f.moderate_nat).is_some() { c += 1; }
        if self.features.as_ref().and_then(|f| f.vpn_accelerator).is_some() { c += 1; }
        if self.features.as_ref().and_then(|f| f.port_forwarding).is_some() { c += 1; }
        c
    }
}
```

**変更ファイル**:
- `src/config/settings.rs` - メソッド追加
- `src/ui/app.rs` - 7箇所を `ps.settings_count()` に置換

---

### ✅ 問題点 2: ConnectionState の重複定義

**状態**: 完了

`VpnState` から `ConnectionState` フィールドを削除し、重複を排除した。

```rust
// src/vpn/state.rs (変更後)
pub struct VpnState {
    client: VpnClient,
}
```

**変更ファイル**:
- `src/vpn/state.rs` - `ConnectionState` フィールドと関連メソッドを削除

---

### ✅ 問題点 5: 選択メソッドの重複

**状態**: 完了

`Navigatable` trait を追加し、選択メソッドを約70行から30行に短縮した。

```rust
// src/state/app_state.rs
pub trait Navigatable {
    fn move_next(&mut self, bounds: usize);
    fn move_prev(&mut self, bounds: usize);
    fn move_first(&mut self, bounds: usize);
    fn move_last(&mut self, bounds: usize);
    fn move_page_down(&mut self, bounds: usize);
    fn move_page_up(&mut self, bounds: usize);
}

impl Navigatable for Option<usize> {
    fn move_next(&mut self, bounds: usize) {
        if bounds == 0 { return; }
        *self = Some(match *self {
            Some(i) => (i + 1).min(bounds - 1),
            None => 0,
        });
    }
    // ... 他のメソッド
}
```

**変更ファイル**:
- `src/state/app_state.rs` - Navigatable trait 追加、メソッド簡略化

---

## 未完了の問題点

---

## 問題点 1: AppState の巨大化 (God Object)

### 現状

`src/state/app_state.rs` が以下の責任を全て担っている:

- **VPN接続状態**: `connection: ConnectionState`, `vpn_state: VpnState`
- **サーバーデータ**: `servers: Vec<Server>`
- **UI状態**: `selected_server`, `settings_selected`, `current_view`
- **検索/フィルター**: `search_query`, `filter`, `sort`, `sort_direction`
- **設定**: `config: Settings`
- **通知**: `notification`, `notification_log`
- **非同期操作**: `pending_refresh`, `pending_connect`, `pending_disconnect` (生channel)

### 問題

1. **単一責任原則の違反**: 複数の関心事が混在
2. **テスト困難**: 全ての依存関係を持つためモック化が困難
3. **保守性の低下**: コード変更の影響範囲が不明確

### 提案

```rust
// 提案: 状態を分割

// 1. ドメイン状態 (参照型)
struct DomainState {
    connection: ConnectionState,
    servers: Vec<Server>,
    vpn_state: VpnState,
}

// 2. UI状態 (値型)
struct UiState {
    view: AppView,
    selected_server: Option<usize>,
    search_query: String,
    filter: ServerFilter,
    sort: ServerSort,
}

// 3. 操作状態 (別モジュール)
struct OperationState {
    pending: HashMap<OperationId, Operation>,
}
```

---

## 問題点 4: 非同期操作の実装が原始的

### 現状

`AppState` で生 channel を使用:

```rust
pub fn refresh_servers(&mut self) {
    let (tx, rx) = std::sync::mpsc::channel();
    self.pending_refresh = Some(rx);
    std::thread::spawn(move || {
        let mut vpn_state = VpnState::new();
        let result = vpn_state.refresh_servers();
        let _ = tx.send(result);
    });
}
```

### 問題

1. **新しい VpnState インスタンスを毎回作成**: 効率悪い
2. **スレッド終了のタイミング不明**: 資源リークの可能性
3. **エラー処理が不十分**: channel が drop された場合の考慮がない
4. **tokio を使っている却没有活用**: Cargo.toml に tokio があるのに std::thread を使用

### 提案

`tokio` を活用した非同期実装:

```rust
// tokio を使用したタスク管理
use tokio::sync::mpsc;

pub struct AsyncTaskManager {
    // タスクの追跡
}

impl AppState {
    pub async fn refresh_servers(&mut self) {
        let vpn_state = self.vpn_state.clone();
        let tx = self.task_sender.clone();
        
        tokio::spawn(async move {
            let result = vpn_state.refresh_servers().await;
            let _ = tx.send(Operation::RefreshComplete(result)).await;
        });
    }
}
```

または、短期的には executor パターン:

```rust
pub struct TaskExecutor {
    tasks: Vec<JoinHandle<()>>,
}

impl TaskExecutor {
    pub fn spawn<F>(&mut self, f: F) 
    where F: Future<Output = ()> + Send + 'static {
        self.tasks.push(tokio::spawn(f));
    }
}
```

---

### ✅ 完了: AsyncTaskManager の実装

**状態**: 完了

`AsyncTaskManager` を作成し、`tokio` ベースの非同期処理に切り替え:

```rust
// src/state/async_tasks.rs
pub struct AsyncTaskManager {
    handle: Arc<tokio::runtime::Handle>,
}

impl AsyncTaskManager {
    pub fn spawn_refresh_servers(&self, vpn_state: VpnState, sender: mpsc::Sender<...>) {
        let handle = self.handle.clone();
        handle.spawn_blocking(move || {
            let mut state = vpn_state;
            let result = state.refresh_servers();
            let _ = sender.blocking_send(result);
        });
    }
    // ... connect, disconnect も同様に実装
}
```

**変更ファイル**:
- `src/state/async_tasks.rs` - 新規作成 (AsyncTaskManager)
- `src/state/mod.rs` - モジュール追加
- `src/state/app_state.rs` - std::thread → tokio::spawn_blocking に置換

**改善点**:
1. ✅ VpnState を再利用 (以前は每次新規作成)
2. ✅ tokio のスレッドプールを使用 (以前は std::thread)
3. ✅ tokio::sync::mpsc を使用 (以前は std::sync::mpsc)
4. ✅ タスクのライフサイクルが明確

---

## 問題点 6: connect_random の非同期的実装

### 現状

`connect_random` は同期的に動作:

```rust
pub fn connect_random(&mut self) {
    // ...
    match self.vpn_state.connect(&server_id) {  // ブロッキング
        Ok((server_id, ip)) => { ... }
        // ...
    }
}
```

他の `connect`/`disconnect` は非同期 (channel 使用)。

### 提案

一貫性を保つため、`connect_random` も非同期にするか、または DESIGN_RULES.md にこのケースを文書化。

---

## 問題点 7: ファイルサイズ過大

### 現状

- `src/ui/app.rs`: 924 行 → 改善後: 約820行
- `src/state/app_state.rs`: 565 行 → 改善後: 約480行
- `src/vpn/client.rs`: 400 行

### 提案

| ファイル | 改善案 |
|---------|--------|
| `app.rs` | ビュー毎のファイルを配置 (connect_view.rs, stats_view.rs, etc.) |
| `app_state.rs` | 状態の種類별로分割 (connection_state.rs は既に分離済み) |
| `client.rs` | パースロジックを separate parser モジュールに |

---

## 優先順位付き改善計画

| 優先度 | 項目 | 状態 | 作業量 |
|--------|------|------|--------|
| ✅完了 | 設定カウント計算の重複除去 | 完了 | 小 |
| ✅完了 | ConnectionState 重複の解決 | 完了 | 中 |
| ✅完了 | 選択メソッドの統一的改善 | 完了 | 小 |
| ✅完了 | 非同期処理の tokio 化 | 完了 | 大 |
| **低** | ファイルの分割 | 未着手 | 中 |
| **低** | connect_random の一貫性 | 未着手 | 小 |
| **低** | AppState の分割 | 未着手 | 大 |

---

## 結論

最初の4つの改善が完了した:

1. ✅ **設定カウント計算のヘルパーメソッド追加** - 7箇所の重複を削除
2. ✅ **ConnectionState の統合** - VpnState から重複を削除
3. ✅ **選択メソッドのリファクタリング** - Navigatable trait で約40行削減
4. ✅ **非同期処理の tokio 化** - tokio を使用してスレッド管理を改善

残りのおすすめ改善:
- **ファイル分割**: app.rs がまだ800行以上
