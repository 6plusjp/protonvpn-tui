# issue110: Feature - OS System Notifications

## Summary

Add Linux desktop notifications (D-Bus notifications) to notify users of connection status changes even when the TUI is not in focus.

## Problem

Currently, notifications only appear within the TUI interface. Users working in other applications won't see:
- VPN connected
- VPN disconnected
- Connection failed
- Server connected (with server info)

## Solution

Use the `notify-rust` crate to send desktop notifications via D-Bus (freedesktop notification specification).

### Library: notify-rust

| Aspect | Details |
|--------|---------|
| **Crate** | `notify-rust` (1.4k stars) |
| **Protocol** | D-Bus (freedesktop.Notifications) |
| **Desktop Support** | GNOME, KDE, XFCE, LXDE, MATE, Sway, etc. |
| **Dependencies** | dbus or zbus (optional, via feature flags) |
| **License** | Apache-2.0 / MIT |

### Usage Example

```rust
use notify_rust::Notification;

Notification::new()
    .appname("ProtonVPN TUI")
    .summary("VPN Connected")
    .body("Connected to Japan (#5)")
    .icon("network-vpn")
    .show()?;
```

### Notification Body Format

| Event | Title | Body | Urgency |
|-------|-------|------|---------|
| Connected | "VPN Connected" | "{server_id} ({city} via {via})" or "{server_id} ({city})" | Normal |
| Disconnected | "VPN Disconnected" | "Disconnected from VPN" | Normal |
| Connect Failed | "Connection Failed" | Error message | Critical |

| Connection Type | Body Format |
|-----------------|-------------|
| Standard | "JP#5 (Tokyo)" |
| Secure Core | "JP#12 (Tokyo via Sweden)" |
| City + via | "JP#5 (Tokyo via Sweden)" |
| No city | "Connected to JP#5" |

### Implementation Plan

1. **Add dependency**: `notify-rust = "4"` to Cargo.toml

2. **Create notification module** (`src/ui/system_notification.rs`):
   - `notify_connected(server_id: &str, city: Option<&str>, via: Option<&str>)`
   - `notify_disconnected()`
   - `notify_connect_failed(error: &str)`

3. **Integrate with AppState**:
   - Add `system_notifications: bool` to UIConfig (default: true)
   - Call system notifications in `event_handler.rs` alongside toast notifications
   - All connection types (connect, connect_random, connect_fastest, connect_p2p, connect_tor, connect_securecore, connect_city) use `AsyncEvent::Connected` → handled uniformly

4. **Settings toggle** (required):
   - Add "System Notifications" toggle in Settings view (SettingKey::SystemNotifications)
   - Default: enabled (true)

### Files Modified

| File | Changes |
|------|---------|
| `Cargo.toml` | Added `notify-rust = "4"` |
| `src/ui/mod.rs` | Added `pub mod system_notification` |
| `src/ui/system_notification.rs` | New - D-Bus notification wrapper |
| `src/config/settings.rs` | Added `SettingKey::SystemNotifications` to enum and ALL array |
| `src/config/user_config.rs` | Added `system_notifications: bool` to UiConfig, updated save/load logic |
| `src/state/app_state.rs` | Added `save_system_notifications()` method, updated `reload_user_config()` |
| `src/state/event_handler.rs` | Added system notification calls for Connected, ConnectCityResult, ConnectFailed, ConnectCityFailed, Disconnected |
| `src/state/settings_ops.rs` | Added `SettingKey::SystemNotifications` case (unreachable!) |
| `src/ui/views/settings_view.rs` | Added system notifications toggle display and value |
| `src/ui/input/tools.rs` | Added toggle handling for system notifications setting |

---

## Implementation Notes

- Uses `notify-rust` crate for D-Bus notifications
- Graceful fallback: logs debug message on failure (no crash if daemon unavailable)
- All connection types handled via `AsyncEvent::Connected` / `ConnectCityResult` events
- System notifications toggle persists to user config file
- Implemented: 2026-04-17

---

## Acceptance Criteria

- [x] Desktop notifications sent on VPN connect
- [x] Desktop notifications sent on VPN disconnect
- [x] Desktop notifications sent on connection failure
- [x] Notification includes server/location info
- [x] Notifications can be toggled via config
- [x] No crash if notification daemon unavailable
- [x] Works on major desktop environments (GNOME, KDE, XFCE)