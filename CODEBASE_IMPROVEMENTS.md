# Codebase Improvements

## 未完了 (優先度順)

### 🟢 低優先度

| #   | 項目              | 作業量 | 詳細                                                  | ステータス |
| --- | ----------------- | ------ | ----------------------------------------------------- | ---------- |
| 13  | AppState の巨大化 | 大     | 12+ フィールドを DomainState/UiState/DataState に分割 | ⏸️ 保留    |

---

## #13 AppState 分割 詳細分析

### 現在の AppState フィールド (20 フィールド)

```
AppState {
  // === Connection/Network (3) ===
  connection: ConnectionState          // VPN 接続状態 (Disconnected/Connecting/Connected/Disconnecting/Error)
  vpn_state: VpnState                   // VPN クライアントラッパー (CLI 操作担う)
  previous_connection: Option<ConnectionState>  // 失敗時のロールバック用

  // === Server Data (3) ===
  servers: Vec<Server>                  // 全サーバーリスト
  filtered_servers_cache: Mutex<...>    // フィルタ済みキャッシュ (2026-03-05: RefCell→Mutex)
  filtered_servers_version: u64          // キャッシュバージョン (2026-03-05: Cell→u64)

  // === UI Selection State (2) ===
  selected_server: Option<usize>         // 選択中サーバー
  settings_selected: Option<usize>       // 設定選択位置

  // === UI View State (5) ===
  current_view: AppView                  // 現在ビュー (Connect/Stats/Settings/Help)
  search_query: String                   // 検索クエリ
  filter: ServerFilter                  // フィルタタイプ (Id/Country/City)
  sort: ServerSort                      // ソートフィールド (Id/Country)
  sort_direction: SortDirection         // ソート方向 (Asc/Desc)

  // === Notification (2) ===
  notification: Option<Notification>     // 現在表示中の通知
  notification_log: Vec<Notification>    // 通知履歴

  // === Async Operations (4) ===
  async_manager: AsyncTaskManager        // 非同期タスク管理
  pending_refresh: Option<ServerReceiver> // サーバー更新保留中
  pending_connect: Option<ConnectReceiver> // 接続保留中
  pending_disconnect: Option<DisconnectReceiver> // 切断保留中

  // === Configuration (1) ===
  proton_settings_cache: OnceLock<...>   // Proton設定キャッシュ
}
```

### 使用パターン分析

#### UI Layer (`app.rs`, `views/*.rs`)

| フィールド          | Read         | Write                  |
| ------------------- | ------------ | ---------------------- |
| `connection`        | ✓            | -                      |
| `vpn_state`         | -            | -                      |
| `current_view`      | ✓            | ✓ (switch_view)        |
| `selected_server`   | ✓            | ✓ (select\_\*)         |
| `settings_selected` | ✓            | ✓ (settings*select*\*) |
| `servers`           | ✓ (filtered) | -                      |
| `search_query`      | ✓            | -                      |
| `filter`            | ✓            | -                      |
| `sort`              | ✓            | -                      |
| `sort_direction`    | ✓            | -                      |
| `notification`      | ✓            | ✓                      |
| `proton_settings`   | ✓            | -                      |

**Key Insights:**

- Views は読み取り専用で扱うフィールドが多い
- 書き込みは selection 関連のみ
- `filtered_servers()` は `search_query`, `filter`, `sort`, `sort_direction` から計算

#### Connection Layer (sync_connection_state, connect, disconnect)

| フィールド            | Read | Write |
| --------------------- | ---- | ----- |
| `connection`          | ✓    | ✓     |
| `vpn_state`           | ✓    | -     |
| `previous_connection` | ✓    | ✓     |
| `pending_*`           | ✓    | ✓     |
| `notification`        | ✓    | ✓     |
| `servers`             | -    | ✓     |

### 推奨分割案

```
┌─────────────────────────────────────────────────────────────────┐
│                        AppState                                 │
├─────────────────────────────────────────────────────────────────┤
│  domain_state: DomainState    # VPN/Server/Connection 管理    │
│  ui_state: UiState            # UI表示/選択状態               │
│  data_state: DataState        # 設定/キャッシュ               │
└─────────────────────────────────────────────────────────────────┘
```

#### DomainState (Connection + Server 管理)

```rust
pub struct DomainState {
    connection: ConnectionState,           // VPN 接続状態
    vpn_state: VpnState,                   // VPN クライアント
    previous_connection: Option<ConnectionState>,

    // Server データ
    servers: Vec<Server>,
    filtered_servers_cache: Mutex<Option<(Vec<Server>, u64)>>,  // 2026-03-05: RefCell→Mutex
    filtered_servers_version: u64,

    // 非同期操作
    async_manager: AsyncTaskManager,
    pending_refresh: Option<ServerReceiver>,
    pending_connect: Option<ConnectReceiver>,
    pending_disconnect: Option<DisconnectReceiver>,
}
```

#### UiState (UI 表示状態)

```rust
pub struct UiState {
    // 選択状態
    selected_server: Option<usize>,
    settings_selected: Option<usize>,

    // ビュー状態
    current_view: AppView,

    // フィルター/ソート
    search_query: String,
    filter: ServerFilter,
    sort: ServerSort,
    sort_direction: SortDirection,

    // 通知
    notification: Option<Notification>,
    notification_log: Vec<Notification>,
}
```

#### DataState (設定/キャッシュ)

```rust
pub struct DataState {
    proton_settings_cache: OnceLock<Option<ProtonSettings>>,
}
```

### 依存関係マトリクス

```
              │ Domain │   Ui   │  Data
──────────────┼────────┼────────┼───────
DomainState   │   -    │ Shared │  -
UiState       │Shared │   -    │  Read
DataState     │  -    │  Read  │   -
```

**問題点:**

- `filtered_servers()` が `UiState` のフィールド (`search_query`, `filter`, `sort`) に依存
- 計算結果は `DomainState` のキャッシュに格納 -循環参照のリスク

### 代替案: 2分割 (現実的)

```
AppState {
    // === Connection & Async (深い結合) ===
    connection: ConnectionState
    vpn_state: VpnState
    previous_connection: Option<ConnectionState>
    async_manager: AsyncTaskManager
    pending_refresh: Option<ServerReceiver>
    pending_connect: Option<ConnectReceiver>
    pending_disconnect: Option<DisconnectReceiver>

    // === Server Data ===
    servers: Vec<Server>
    filtered_servers_cache: Mutex<...>  // 2026-03-05: RefCell→Mutex
    filtered_servers_version: u64       // 2026-03-05: Cell→u64

    // === UI State ( views から直接アクセス ) ===
    current_view: AppView
    selected_server: Option<usize>
    settings_selected: Option<usize>
    search_query: String
    filter: ServerFilter
    sort: ServerSort
    sort_direction: SortDirection

    // === Notification ===
    notification: Option<Notification>
    notification_log: Vec<Notification>

    // === Config (独立してロード可能) ===
    proton_settings_cache: OnceLock<...>
}
```

**単純な Field Grouping のみ:**

- フィールドの移動は最小限
- メソッドの移動も対応するグループへ
- 依存関係は維持

### リスク評価

| リスク       | レベル | 理由                                    |
| ------------ | ------ | --------------------------------------- |
| breaking API | 🔴 高  | `AppState` を参照する全コードを更新必要 |
| テスト壊れる | 🟡 中  | テストが AppState 依存                  |
| 循環参照     | 🟡 中  | 分割後の参照設計次第                    |
| 作業量       | 🔴 高  | 20 フィールド + メソッド総移動          |

### 推奨アプローチ

1. **まずやらない** (CODEBASE_IMPROVEMENTS.md に記載通り)
   - 現在のところ問題は発生していない
   - 分割による-benefits がコストを下回る

2. **もしやるなら:** 2分割案のみ推奨
   - 3分割は循環参照リスクが高く、非推奨
   - Field Grouping 程度で妥協

---

## 真の設計問題 (AppState分割とは別)

### 🔴 問題2: Interior Mutability (RefCell + Cell)

**場所:** `src/state/app_state.rs:123-124`

```rust
filtered_servers_cache: RefCell<Option<(Vec<Server>, u64)>>,
filtered_servers_version: Cell<u64>,
```

**問題:**

- Rust のアンチパターン（許可しない限り interior mutability 避免）
- `RefCell` は runtime borrow checker、オーバーヘッドあり
- キャッシュ用途なら `OnceLock` や `Mutex` が適切

**修正案:**

```rust
// Option 1: OnceLock (简单)
// 缺点: キャッシュクリア时再初期化必要

// Option 2: キャッシュ辞書を别途管理
// 缓存失效时重新计算（简单）
filtered_servers_version: u64,  // Cell なし
```

---

## 🔴 問題2: Interior Mutability (RefCell + Cell) ✅ 解決済み

**場所:** `src/state/app_state.rs:123-124`

```rust
// 変更後 (2026-03-05)
filtered_servers_cache: Mutex<Option<(Vec<Server>, u64)>>,
filtered_servers_version: u64,
```

**修正内容:**

- `RefCell` → `Mutex` (標準的なスレッドセーフな同期)
- `Cell<u64>` → `u64` (通常フィールド)

**ステータス:** ✅ 完了

**問題:**

- 大きなベクトルのコピーはコスト高い
- 呼び出し元が必要なければ、无意味なアロケーション

**修正案:**

```rust
// Clone なし - callerが必要なら自有で clone
cached_result.clone()  // まだあるが...

// 更好的方案: &Vec<Server> を返す
pub fn filtered_servers(&self) -> &Vec<Server>  // できない、cache管理が複雑に
```

---

### 🟡 問題4: 手動キャッシュInvalidation

**場所:** 以下のメソッド全て

```rust
pub fn cycle_filter(&mut self) {
    self.filter = self.filter.next();
    self.invalidate_filtered_cache();  // 忘れるやすい
}

pub fn cycle_sort(&mut self) {
    self.sort_direction = self.sort_direction.toggle();
    self.invalidate_filtered_cache();
}

pub fn set_filter(&mut self, filter: ServerFilter) {
    self.filter = filter;
    self.invalidate_filtered_cache();
}
```

**問題:**

- キャッシュを手动で无效化해야 する
- 忘れると古いデータが返るバグ

**修正案:**

```rust
// 或者: 每次 recompute (缓存は复杂的场合)
pub fn filtered_servers(&self) -> Vec<Server> {
    self.compute_filtered_servers()  // 简单、缓存しない
}

// 或者: State Change Listener パターン
```

---

## 設計改善提案

### 案A: 简单化 (推奨)

**やること:**

1. `generate_fuzzy_variants()` から隠れた依存を移除
2. キャッシュを削除、毎帧 recompute

**Pros:**

- コードが简单になる
- 隐藏された依存がなくなる
- テストが容易

**Cons:**

- パフォーマンス若干低下（しかしサーバーリストは较小）

### 案B: キャッシュ改善

**やること:**

1. `RefCell` → `OnceLock` or `Mutex` に変更
2. キャッシュクリアメソッドを统一化

**Pros:**

- Rust らしかった设计
- パフォーマンス維持

**Cons:**

- 実装コスト中程度

---

## メモ

- `#13` (AppState 巨大化) はリファクタリングコストが高く、優先度を下げるべき
