//! System notification via D-Bus (freedesktop notifications)

use notify_rust::Notification;

/// Send desktop notification for VPN connected event
pub fn notify_connected(server_id: &str, city: Option<&str>, via: Option<&str>) {
    let body = match (city, via) {
        (Some(c), Some(v)) => format!("{} ({} via {})", server_id, c, v),
        (Some(c), None) => format!("{} ({})", server_id, c),
        (None, Some(v)) => format!("{} via {}", server_id, v),
        (None, None) => format!("Connected to {}", server_id),
    };

    let result = Notification::new()
        .appname("ProtonVPN TUI")
        .summary("VPN Connected")
        .body(&body)
        .icon("network-vpn")
        .show();

    if let Err(e) = result {
        tracing::warn!("Failed to show system notification: {}", e);
    }
}

/// Send desktop notification for VPN disconnected event
pub fn notify_disconnected() {
    let result = Notification::new()
        .appname("ProtonVPN TUI")
        .summary("VPN Disconnected")
        .body("Disconnected from VPN")
        .icon("network-vpn")
        .show();

    if let Err(e) = result {
        tracing::warn!("Failed to show system notification: {}", e);
    }
}

/// Send desktop notification for VPN connection failure
pub fn notify_connect_failed(error: &str) {
    let result = Notification::new()
        .appname("ProtonVPN TUI")
        .summary("Connection Failed")
        .body(error)
        .icon("network-vpn")
        .show();

    if let Err(e) = result {
        tracing::warn!("Failed to show system notification: {}", e);
    }
}
