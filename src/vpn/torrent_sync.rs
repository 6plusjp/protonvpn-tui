use crate::error::{AppError, AppResult};

pub fn update_qbittorrent_port(port: u16) -> AppResult<()> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| AppError::ConfigNotFound("config directory not found".into()))?
        .join("qBittorrent")
        .join("qBittorrent.conf");

    if !config_path.exists() {
        tracing::debug!("qBittorrent config not found, skipping port update");
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let mut in_session = false;
    let mut port_updated = false;
    for line in lines.iter_mut() {
        if line.trim() == "[Session]" {
            in_session = true;
        } else if line.starts_with('[') && line.ends_with(']') {
            in_session = false;
        } else if in_session && (line.starts_with("Session\\Port=") || line.starts_with("Port=")) {
            *line = format!("Session\\Port={}", port);
            tracing::info!("Updated qBittorrent Port to {}", port);
            port_updated = true;
            break;
        }
    }

    if !port_updated {
        for line in lines.iter_mut() {
            if line.starts_with("Session\\Port=") {
                *line = format!("Session\\Port={}", port);
                tracing::info!("Updated qBittorrent Session\\Port to {}", port);
                break;
            }
        }
    }

    std::fs::write(&config_path, lines.join("\n"))?;
    Ok(())
}

pub fn sync_forwarded_port(port: u16) {
    if let Err(e) = update_qbittorrent_port(port) {
        tracing::warn!("Failed to update qBittorrent port: {}", e);
    }
}