use crate::paths;
use crate::state::Notification;
use chrono::{DateTime, Utc};
use std::fs;

fn get_log_file_path() -> std::path::PathBuf {
    paths::logs_path().unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[cfg(test)]
static TEST_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
pub fn set_test_mode(enabled: bool) {
    TEST_MODE.store(enabled, std::sync::atomic::Ordering::SeqCst);
}

pub fn load_notification_log() -> Vec<Notification> {
    #[cfg(test)]
    if TEST_MODE.load(std::sync::atomic::Ordering::SeqCst) {
        return Vec::new();
    }

    let path = get_log_file_path();
    if !path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(e) => {
            tracing::warn!("Failed to read log file: {}", e);
            Vec::new()
        }
    }
}

pub fn save_notification_log(log: &[Notification]) {
    #[cfg(test)]
    if TEST_MODE.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    let path = get_log_file_path();

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match serde_json::to_string_pretty(log) {
        Ok(content) => {
            if let Err(e) = fs::write(&path, content) {
                tracing::warn!("Failed to write log file: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to serialize log: {}", e);
        }
    }
}

#[cfg(test)]
pub fn clear_log_file() {
    let path = get_log_file_path();
    let _ = fs::remove_file(path);
}

pub fn format_relative_time(timestamp: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(timestamp);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        let mins = diff.num_minutes();
        format!("{}m ago", mins)
    } else if diff.num_hours() < 24 {
        let hours = diff.num_hours();
        format!("{}h ago", hours)
    } else if diff.num_days() < 7 {
        let days = diff.num_days();
        format!("{}d ago", days)
    } else {
        timestamp.format("%Y-%m-%d").to_string()
    }
}
