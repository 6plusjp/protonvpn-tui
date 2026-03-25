# resolved065: Dual async event processing paths

## Summary

Async event handling had two parallel code paths (event-driven and polling), creating duplication and potential race conditions. Now resolved by unifying on event-driven approach.

## Problem

`ConnectionManager` had TWO mechanisms for processing async results:

### Before: Two Paths (Duplication)

```
非同期タスク完了
    │
    ├──→ Path 1: AsyncNotifier (未使用)
    │
    └──→ Path 2: pending_* HashMaps (チャネル経由)
```

### Issues

1. **Duplication**: `event_handler.rs` had both `process_async_events()` AND 7 `check_pending_*()` methods
2. **Inconsistent Key Usage**: Most pending maps used `()` as key, but `pending_cities` used `String`
3. **Silent Failures**: Second connect request silently replaced the receiver

## Solution

**Option A: Unify on Event-driven** を実装

### After: Single Path

```
非同期タスク完了
    ↓
AsyncNotifier.notify(event)
    ↓
wait_for_async_events()
    ↓
handle_async_events()
```

## Implementation Details

### 1. ConnectionManager

```rust
// Before
pub struct ConnectionManager {
    pub pending_refresh: HashMap<(), ServerReceiver>,
    pub pending_connect: HashMap<(), ConnectReceiver>,
    pub pending_disconnect: HashMap<(), DisconnectReceiver>,
    pub pending_cities: HashMap<String, CitiesReceiver>,
    pub pending_connect_city: HashMap<(), ConnectReceiver>,
    pub pending_config_set: HashMap<(), ConfigReceiver>,
}

// After
pub struct ConnectionManager {
    pub connection: ConnectionState,
    pub previous_connection: Option<ConnectionState>,
    pub async_manager: AsyncTaskManager,
    pub async_notifier: Arc<AsyncNotifier>,
    pub loading_cities: HashSet<String>,  // ローディング状態追跡用
}
```

### 2. AsyncTaskManager

```rust
// Before: チャネル使用
pub fn spawn_connect(&self, vpn_state: Arc<VpnClient>, server_id: String, sender: mpsc::Sender<...>)

// After: AsyncNotifier使用
pub fn spawn_connect(&self, vpn_state: Arc<VpnClient>, server_id: String, notifier: Arc<AsyncNotifier>)
```

### 3. Event Handler

```rust
// Before: 複数メソッド
process_async_events()
check_pending_refresh()
check_pending_connect()
check_pending_disconnect()
check_pending_cities()
check_pending_connect_city()
check_pending_config_set()

// After: 単一メソッド
wait_for_async_events(timeout)  // イベント待機 + 接続状態同期
```

### 4. Cities Loading

```rust
// Before: pending_cities HashMap
self.connection_manager.pending_cities.insert(country_code.clone(), rx);

// After: loading_cities HashSet
self.connection_manager.loading_cities.insert(country_code.clone());
```

## Additional Fixes

1. **Cities Update**: `server.cities`も更新するように修正（Countriesペインの表示反映）
2. **Connection State Sync**: `sync_connection_state()`が正しく`true`を返すように修正
3. **Redundant Processing**: `check_pending_async_events()`を削除し、`wait_for_async_events()`に統合

## Files Modified

| File | Changes |
|------|---------|
| `src/state/connection_manager.rs` | `pending_*`削除、`loading_cities`追加 |
| `src/vpn/async_tasks.rs` | `AsyncNotifier`使用に変更 |
| `src/state/app_state_impl.rs` | チャネル作成削除 |
| `src/state/event_handler.rs` | `check_pending_*`削除、イベント処理統合 |
| `src/state/server_ops.rs` | `loading_cities`使用 |
| `src/ui/app.rs` | 冗長呼び出し削除 |
| `src/ui/views/servers_view.rs` | `loading_cities`使用 |

## Verification

```bash
$ cargo check
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.54s

$ cargo test
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Severity

~~🟠 MEDIUM~~ → ✅ RESOLVED

## Labels

`architecture` `async` `refactor` `resolved`
