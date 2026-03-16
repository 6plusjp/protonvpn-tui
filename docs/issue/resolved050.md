# issue050-is_connected-false-positive-when-persistence-file-exists

## Summary

`VpnClient::is_connected()` returns `true` based solely on the existence of the persistence file (`~/.cache/Proton/VPN/connection/connection_persistence.json`), without verifying that the VPN is actually connected. This causes false positives when the file exists from a previous connection but the VPN is currently disconnected.

## Status

**[Implemented]**

## Implementation

### Changes Made

| File | Change |
|------|--------|
| `src/vpn/client.rs:253-275` | Rewrote `is_connected()` to check both proton0 interface and persistence file |
| `src/vpn/client.rs:39-49` | Added `persistence_file_path()` helper to centralize file path |

### Logic

```rust
pub fn is_connected(&self) -> bool {
    let proton0_exists = std::path::Path::new("/sys/class/net/proton0").exists();
    if !proton0_exists {
        return false;
    }

    let persistence_path = self.persistence_file_path();
    persistence_path.exists()
}
```

- First checks if `proton0` interface exists (VPN is active)
- Then checks if persistence file exists (connection is maintained)
- Both must be true for `is_connected()` to return `true`

### Helper Function

```rust
fn persistence_file_path(&self) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Proton")
        .join("VPN")
        .join("connection")
        .join("connection_persistence.json")
}
```

Centralized persistence file path makes future path changes easier.

### Why This Works

- `proton0` alone is insufficient: lingers after `protonvpn disconnect`
- Persistence alone is insufficient: remains after crash/network interruption
- **Combined approach**: Both conditions must be true for accurate detection

## Problem Description

### Current Implementation

In `src/vpn/client.rs:253-264`:

```rust
pub fn is_connected(&self) -> bool {
    let persistence_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Proton")
        .join("VPN")
        .join("connection")
        .join("connection_persistence.json");

    let connected = persistence_path.exists();
    tracing::debug!("Connected (persistence file): {}", connected);
    connected
}
```

The method only checks if the file **exists**, not if the VPN is actually connected.

### Why Persistence File Exists When Disconnected

According to the [ProtonVPN persistence.py](https://github.com/ProtonVPN/python-proton-vpn-api-core/blob/stable/proton/vpn/connection/persistence.py):

> "Connection parameters are persisted to disk **so that they can be loaded after a crash**."

The persistence file is a **crash recovery mechanism**, not a connection status indicator. It survives:
- System reboot
- Network interruption  
- VPN daemon crash
- Manual disconnect via other means (NetworkManager, other apps)
- Any unexpected termination

The file is only removed when `protonvpn disconnect` is explicitly called (which calls `ConnectionPersistence.remove()`).

### Impact

1. **False connected state**: App shows "Connected" even when VPN is disconnected
2. **Stale connection info**: `get_connected_server_info()` returns old server data
3. **UI confusion**: User sees connected state but cannot access VPN-protected resources

### Where It's Used

In `src/state/app_state.rs:640-654`:

```rust
// When disconnected, check if externally connected
if self.connection_manager.connection == ConnectionState::Disconnected
    && self.vpn_state.is_connected()  // <- Uses the flawed logic
{
    let (server, ip) = self
        .vpn_state
        .get_connected_server_info()  // <- Returns stale data
        .unwrap_or_else(|| ("Unknown".to_string(), String::new()));
    // ... sets ConnectionState::Connected
}
```

## Root Cause

The comment in `src/vpn/cache.rs:22` states:
```rust
/// Connection status (tracked locally since no `protonvpn status` exists)
```

**Correction**: The new ProtonVPN CLI (`protonvpn`) does **NOT** have a `protonvpn status` command. The old legacy CLI (`protonvpn-cli`) had it, but the new Python-based CLI does not.

The new CLI reference ([PROTONVPN_CLI_REFERENCE.md](docs/reference/PROTONVPN_CLI_REFERENCE.md)) shows these commands only:
- `protonvpn signin`, `protonvpn signout`, `protonvpn info`
- `protonvpn connect`, `protonvpn disconnect`
- `protonvpn countries`, `protonvpn cities`
- `protonvpn config`

**No `status` command exists.**

## ProtonVPN Network Interfaces

When ProtonVPN is connected, it creates various network interfaces:

### Main VPN Interface

| Interface | Protocol | Description |
|-----------|----------|-------------|
| `proton0` | WireGuard | Primary WireGuard interface (new CLI) |
| `wg0` | WireGuard | Alternative WireGuard interface name |
| `tun0` | OpenVPN | OpenVPN tunnel interface |

**Note**: The new ProtonVPN CLI uses WireGuard protocol and typically creates `proton0` interface.

### IPv6 Leak Protection Interface

| Interface | Description |
|-----------|-------------|
| `ipv6leakintrf0` | IPv6 leak protection virtual interface |
| `pvpn-ipv6leak-protection` | NetworkManager connection (nmcli) |

This interface is created by the **legacy** CLI (`protonvpn-cli`) when IPv6 leak protection is enabled. The new CLI may use different naming.

### Kill Switch Interface

| Interface | Description |
|-----------|-------------|
| `pvpn-killswitch` | NetworkManager connection for kill switch |
| `pvpn-killswitch-ipv6` | Combined kill switch + IPv6 protection |

These are **NetworkManager connections** (not kernel interfaces), managed by `nmcli con show`.

### Interface Naming Convention

- **Kernel interfaces**: `proton0`, `wg0`, `tun0`, `ipv6leakintrf0`
- **NetworkManager connections**: `pvpn-killswitch`, `pvpn-ipv6leak-protection`

## Proposed Solutions

### Correct Approach: Combined Check (proton0 + persistence file)

Based on practical experience:

1. Using `proton0` alone is insufficient because the interface doesn't disappear immediately after `protonvpn disconnect`
2. Using persistence file alone is insufficient because the file persists after crash/disconnect
3. **Solution**: Both conditions must be true

**Connection criteria**:
- `proton0` interface is active (CLI is functioning)
- **AND** `connection_persistence.json` file exists (connection is maintained)

This ensures:
- ✅ False positives avoided (persistence alone is not enough)
- ✅ False negatives avoided (interface alone is not enough - it's slow to clean up)
- ✅ Actual connection state accurately reflected

### Option A: Combined Network Interface + Persistence Check

```rust
pub fn is_connected(&self) -> bool {
    // Check if proton0 interface exists AND is UP
    let proton0_active = self.is_interface_active("proton0");
    
    // Check if persistence file exists
    let persistence_exists = self.persistence_file_exists();
    
    // Both must be true for actual connection
    let connected = proton0_active && persistence_exists;
    
    tracing::debug!(
        "Connected check: proton0={}, persistence={}, result={}",
        proton0_active, persistence_exists, connected
    );
    
    connected
}

fn is_interface_active(&self, iface: &str) -> bool {
    let path = format!("/sys/class/net/{}", iface);
    if !std::path::Path::new(&path).exists() {
        return false;
    }
    // Check operstate: "up" or "unknown" means active
    if let Ok(operstate) = std::fs::read_to_string(format!("{}/operstate", path)) {
        let state = operstate.trim();
        return state == "up" || state == "unknown";
    }
    false
}

fn persistence_file_exists(&self) -> bool {
    let persistence_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Proton")
        .join("VPN")
        .join("connection")
        .join("connection_persistence.json");
    persistence_path.exists()
}
```

### Option B: Simplified Combined Check

```rust
pub fn is_connected(&self) -> bool {
    // Primary: proton0 must exist
    if !std::path::Path::new("/sys/class/net/proton0").exists() {
        return false;
    }
    
    // Secondary: persistence file must exist
    // This prevents false positive when proton0 lingers after disconnect
    let persistence_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Proton")
        .join("VPN")
        .join("connection")
        .join("connection_persistence.json");
    
    persistence_path.exists()
}
```

## Implementation Notes

- **Both checks are necessary**:
  - `proton0` alone is not sufficient (lingers after disconnect)
  - Persistence file alone is not sufficient (remains after crash/disconnect)
- The combined approach ensures accurate connection state detection
- The new CLI uses WireGuard protocol → primarily `proton0` interface
- Consider adding `is_interface_active()` helper method to `VpnClient`
- Handle cases where CLI is not installed or unavailable

## Persistence File Format

From [ProtonVPN python-proton-vpn-api-core](https://github.com/ProtonVPN/python-proton-vpn-api-core/blob/stable/proton/vpn/connection/persistence.py):

```json
{
  "connection_id": "...",
  "backend": "wireguard|openvpn",
  "protocol": "udp|tcp",
  "server": {
    "server_name": "JP#374",
    "server_ip": "123.45.67.89",
    ...
  }
}
```

## Related Code

| File | Line | Description |
|------|------|-------------|
| `src/vpn/client.rs` | 253-264 | `is_connected()` - flawed implementation |
| `src/vpn/client.rs` | 267-290 | `get_connected_server_info()` - reads persistence |
| `src/state/app_state.rs` | 640-654 | Usage of `is_connected()` |
| `src/vpn/cache.rs` | 22 | Comment about missing `protonvpn status` |

## References

- [ProtonVPN persistence.py](https://github.com/ProtonVPN/python-proton-vpn-api-core/blob/stable/proton/vpn/connection/persistence.py) - Official persistence implementation
- [PROTONVPN_CLI_REFERENCE.md](docs/reference/PROTONVPN_CLI_REFERENCE.md) - CLI command reference
- [IPv6 Leak Protection Issue](https://github.com/ProtonVPN/linux-cli/issues/43) - Known issues with ipv6leak interfaces
- Practical observation: `protonvpn disconnect` doesn't immediately remove `proton0` interface

## Historical Note

A previous implementation (referenced in `docs/issue/resolved016.md`) used `ip addr show proton0` to check connection status alone. This was later changed to persistence-based approach.

However, neither approach alone is sufficient:
- `proton0` alone: Lingers after `protonvpn disconnect` (doesn't disappear immediately)
- Persistence alone: Remains after crash/network interruption

**The combined approach (both proton0 AND persistence) is required for accurate detection.**
