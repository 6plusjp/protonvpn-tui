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

### Notification Events

| Event | Title | Body | Urgency |
|-------|-------|------|---------|
| Connected | "VPN Connected" | "Connected to {server} ({city})" | Normal |
| Disconnected | "VPN Disconnected" | "Disconnected from VPN" | Normal |
| Connect Failed | "Connection Failed" | Error message | Critical |

### Implementation Plan

1. **Add dependency**: `notify-rust = "4"` to Cargo.toml

2. **Create notification module** (`src/ui/system_notification.rs`):
   ```rust
   pub struct SystemNotification;
   
   impl SystemNotification {
       pub fn notify_connected(server: &str, city: Option<&str>);
       pub fn notify_disconnected();
       pub fn notify_connect_failed(error: &str);
   }
   ```

3. **Integrate with AppState**:
   - Add `enable_system_notifications: bool` to config
   - Call system notifications in `app_state_impl.rs` alongside toast notifications

3. **Settings toggle** (required):
   - Add "System Notifications" toggle in Settings view
   - Default: enabled (true)

### Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `notify-rust` dependency |
| `src/ui/mod.rs` | Add system_notification module |
| `src/ui/system_notification.rs` | New - D-Bus notification wrapper |
| `src/config/settings.rs` | Add `system_notifications` SettingKey |
| `src/config/user_config.rs` | Add to UIConfig |
| `src/state/app_state_impl.rs` | Call system notifications on events |
| `src/ui/views/settings_view.rs` | Add system notifications toggle |
| `docs/policy/policy.md` | Document notification feature |

### Dependencies Note

The crate works without extra dependencies on most Linux systems with a notification daemon. For build:
- `dbus` (default): requires libdbus
- `zbus` (optional): pure Rust, no system dependency

Using default features is recommended for most systems.

---

## Acceptance Criteria

- [ ] Desktop notifications sent on VPN connect
- [ ] Desktop notifications sent on VPN disconnect
- [ ] Desktop notifications sent on connection failure
- [ ] Notification includes server/location info
- [ ] Notifications can be toggled via config
- [ ] No crash if notification daemon unavailable
- [ ] Works on major desktop environments (GNOME, KDE, XFCE)