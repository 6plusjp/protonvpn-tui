# issue111: Session comparison using server_id from persistence.json

## Summary

Session recording (`connected_at` timestamp) incorrectly resets during sync. This fix compares `server_id` (base64 format) from cache with `server_id` from `connection_persistence.json` instead of comparing stale `server_ip`.

## Root Cause

### Previous (Problematic) Flow

```
persistence.json              cache
┌─────────────────────┐    ┌─────────────────────┐
│ server.server_name  │    │ connected_server   │ = "JP#374"
│ server.server_ip    │    │ connected_ip      │ = "1.2.3.4"
└─────────────────────┘    └─────────────────────┘
         ↓                            ↓
    server_name comparison    ← works
    server_ip comparison    ← WRONG (stale IP never updates!)
         ↓
    IP different → session reset ← BUG
```

### Problem

`connection_persistence.json` `server_ip` is NEVER updated by ProtonVPN CLI (confirmed by research).

### Solution

Compare `server_id` (base64 format) and `server_name` from cache with persistence.json:

```
On connection:
  cache.connected_server = "JP#374"       ← server_name format
  cache.connected_server_id = "base64"      ← NEW: server_id from CLI
  cache.connected_ip = "1.2.3.4"

On sync:
  ├─ get_connected_server_info() → (server_id, server_name, server_ip)
  ├─ get_connected_server() → cache: "JP#374" (server_name)
  ├─ get_connected_server_id() → cache: "base64" (server_id)
  └─ Compare both server_name and server_id
      → same → preserve session
```

## connection_persistence.json Schema

From `python-proton-vpn-api-core` test files:

```json
{
  "connection_id": "connection_id",
  "backend": "backend",
  "protocol": "protocol",
  "server": {
    "server_ip": "1.2.3.4",
    "openvpn_ports": { "udp": [12345], "tcp": [80] },
    "wireguard_ports": { "udp": [54321], "tcp": [81] },
    "domain": "server.domain",
    "x25519pk": "public_key",
    "server_id": "oiHccQGEZFmtz...==",  ← base64 format (UNIQUE)
    "server_name": "JP#374",           ← server name format
    "has_ipv6_support": "0",
    "label": "label"
  }
}
```

Both `server_id` (base64) and `server_name` ("JP#374") exist in the JSON.

## Data Flow

### Before Fix (Buggy)

```
【connect command】
protonvpn connect → connect output (server="JP#374", ip)
                              ↓
cache.connected_server = "JP#374"
cache.connected_ip = "1.2.3.4"

【app startup】
sync_connection_from_vpn()
  ├─ get_connected_server_info() → (server="JP#374", server_ip=stale)
  ├─ get_connected_server() → cache: "JP#374"
  ├─ get_connected_ip() → cache: ip="1.2.3.4"
  └─ Compare: server same, IP different → reset ← BUG
```

### After Fix

```
【connect command】
protonvpn connect → connect output (server="JP#374", ip)
                              ↓
cache.connected_server = "JP#374"       ← server_name
cache.connected_server_id = "base64"   ← NEW: server_id
cache.connected_ip = "1.2.3.4"

【app startup】
sync_connection_from_vpn()
  ├─ get_connected_server_info() → (server_id, server_name, server_ip)
  ���─ get_connected_server() → cache: "JP#374"
  ├─ get_connected_server_id() → cache: "base64"
  └─ Compare: server_name same AND server_id same → preserve session
```

## Implementation

### Files Changed

| File | Change |
|------|-------|
| `src/vpn/cache.rs` | Added `connected_server_id` field |
| `src/vpn/types.rs` | Added `server_id` field to `ConnectResult` |
| `src/vpn/client.rs` | Updated `get_connected_server_info()`, `get_connected_server_id()`, `set_connected()`, `sync_cache_with_connection()` |
| `src/state/app_state_impl.rs` | Updated to compare both `server_name` and `server_id` |
| `src/state/event_handler.rs` | Updated tuple handling |

### Key Changes

1. **ServerCache** - Added `connected_server_id` field:
   ```rust
   pub connected_server: Option<String>,   // "JP#374"
   pub connected_server_id: Option<String>, // base64 server_id
   ```

2. **get_connected_server_info()** - Returns 3-tuple:
   ```rust
   pub fn get_connected_server_info(&self) -> Option<(String, String, String)> {
       // Returns (server_id, server_name, server_ip)
   }
   ```

3. **sync_connection_from_vpn()** - Compares both:
   ```rust
   let server_changed = current_server.as_deref() != Some(&server_name)
       || current_server_id.as_deref() != Some(&server_id);
   ```

## Related Files

- `src/vpn/client.rs` - VPN client, cache management
- `src/vpn/cache.rs` - ServerCache struct
- `src/vpn/types.rs` - ConnectResult
- `src/state/app_state_impl.rs` - sync_connection_from_vpn()
- `src/state/event_handler.rs` - Event handling

## Notes

- `server_id` (base64) is stable - never changes for same server instance
- `server_name` ("JP#374") may change if server IP is reassigned
- Comparing both provides robust session preservation
- Issue only affects session tracking display, not actual VPN connectivity