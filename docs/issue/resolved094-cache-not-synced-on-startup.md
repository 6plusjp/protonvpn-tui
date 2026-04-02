# Resolved 094: Cache Not Synced with Actual Connection on Startup

**Status**: ✅ Resolved  
**Resolved**: 2026-04-02  
**Related**: Issue 093 (Session time persistence), Issue 095 (city/country restoration)

---

## Summary

After reboot with VPN auto-connect, the cache contained stale connection info (server, IP) while the UI displayed the actual connection from CLI's persistence file. This caused a mismatch between cached and displayed data.

## Problem Description

When the app started with an existing VPN connection, `sync_connection_state()` updated the UI with actual connection info from `connection_persistence.json`, but the cache remained unchanged with old data from before the reboot.

### Data Flow Analysis

#### Connection Flow (via app) — Works Correctly

```
connect()
  → spawn_connect() [async]
  → CLI executes protonvpn connect
  → parse_connect_output() extracts server_id, ip
  → cache.set_connected(server, ip, via)
      connected_at = Utc::now()
      connected_server = server_id
      connected_ip = ip
  → cache.save() to disk
  → Connected event → ConnectionState::Connected { server, ip, ... }
```

Result: Cache and UI are in sync ✓

#### Startup Flow (Before Fix) — Problem Identified

```
App startup:
  VpnClient::new()
    → cache.load() from disk  ← loads old data (from before reboot)

  sync_connection_state()
    → is_connected() = true  (proton0 interface + persistence file exist)
    → get_connected_server_info()
        reads: connection_persistence.json  ← actual connection info
    → ConnectionState::Connected { server, ip, ... }  ← UI updated
    → cache is NOT updated  ← problem!
```

Result: Cache has stale data, UI has actual data — **MISMATCH**

#### Startup Flow (After Fix) — Resolved

```
App startup:
  VpnClient::new()
    → cache.load() from disk
    → adjust_connected_at_on_startup()  ← connected_at検証

  sync_connection_state()
    → is_connected() = true
    → get_connected_server_info()
        reads: connection_persistence.json
    → sync_cache_with_connection(server, ip)
        → メモリ更新 (connected_server, connected_ip, connected_at)
        → save_cache()  ← ディスク永続化
    → ConnectionState::Connected { server, ip, ... }  ← UI更新
```

Result: Cache and UI are in sync ✓

## Solution

### 1. Added `sync_cache_with_connection()` to `VpnClient` (`src/vpn/client.rs`)

```rust
/// Sync cache with actual connection info (e.g., after reboot with auto-connect)
pub fn sync_cache_with_connection(&self, server: &str, ip: &str) {
    if let Ok(mut cache) = self.cache.lock() {
        let server_changed = cache.connected_server.as_deref() != Some(server);
        let ip_changed = cache.connected_ip.as_deref() != Some(ip);

        if server_changed || ip_changed {
            tracing::debug!(
                "Syncing cache: old={:?}/{:?}, new={:?}/{:?}",
                cache.connected_server,
                cache.connected_ip,
                server,
                ip
            );
            cache.connected_server = Some(server.to_string());
            cache.connected_ip = Some(ip.to_string());
            cache.connected_via = None;

            // Update connected_at to boot time if it's before boot
            if let Some(boot_time) = super::cache::system_boot_time() {
                if cache.connected_at.map_or(true, |t| t < boot_time) {
                    cache.connected_at = Some(boot_time);
                }
            }

            // Persist sync to disk
            if let Err(e) = self.save_cache() {
                tracing::warn!("Failed to persist cache sync: {}", e);
            }
        }
    }
}
```

### 2. Updated `sync_connection_state()` in `src/state/event_handler.rs`

```rust
fn sync_connection_state(&mut self) -> bool {
    if self.connection_manager.connection.is_connected() {
        return false;
    }
    if self.connection_manager.connection == ConnectionState::Disconnected
        && self.vpn_state.is_connected()
    {
        let (server, ip) = self
            .vpn_state
            .get_connected_server_info()
            .unwrap_or_else(|| ("Unknown".to_string(), String::new()));

        // Sync cache with actual connection info (e.g., after reboot with auto-connect)
        self.vpn_state.sync_cache_with_connection(&server, &ip);

        self.connection_manager.connection = ConnectionState::Connected {
            server,
            ip,
            city: None,
            country: None,
            via: None,
        };
        return true;
    }
    false
}
```

### 3. Made `system_boot_time()` public in `src/vpn/cache.rs`

```rust
/// Get system boot time from /proc/stat
///
/// Returns None if /proc/stat is unavailable or malformed (non-Linux systems)
pub fn system_boot_time() -> Option<DateTime<Utc>> {
    // ...
}
```

## Changed Files

| File | Change |
|------|--------|
| `src/vpn/cache.rs` | `system_boot_time()` を `pub` に変更 |
| `src/vpn/client.rs` | `sync_cache_with_connection()` メソッド追加 |
| `src/state/event_handler.rs` | `sync_connection_state()` でキャッシュ同期呼び出し |

## Verification

- `cargo check`: ✅ 成功
- `cargo test`: ✅ 111テスト全て成功

## Dependencies

- Issue 093: `system_boot_time()` helper — ✅ 完了済み

## Related

- Issue 095: city/country not restored on startup (未実装)
- `src/vpn/client.rs` - VpnClient initialization
- `src/state/event_handler.rs` - `sync_connection_state()`
