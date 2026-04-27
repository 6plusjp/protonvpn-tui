use std::path::PathBuf;

use crate::constants::paths::{
    APP_NAME, CACHE_FILE_NAME, CONFIG_FILE_NAME, LOGS_FILE_NAME,
    PROTON_CONNECTION_PERSISTENCE_FILE, PROTON_DIR, PROTON_SETTINGS_FILE,
};

fn base_dir() -> PathBuf {
    PathBuf::from(".")
}

fn dirs_config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

fn dirs_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir()
}

fn dirs_data_local_dir() -> Option<PathBuf> {
    dirs::data_local_dir()
}

pub fn config_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join(APP_NAME));
    }
    dirs_config_dir().map(|p| p.join(APP_NAME))
}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join(CONFIG_FILE_NAME))
}

pub fn config_display_path() -> String {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg)
            .join(APP_NAME)
            .join(CONFIG_FILE_NAME)
            .to_string_lossy()
            .to_string();
    }
    if let Some(dir) = dirs_config_dir() {
        return dir
            .join(APP_NAME)
            .join(CONFIG_FILE_NAME)
            .to_string_lossy()
            .to_string();
    }
    crate::constants::paths::CONFIG_DISPLAY_PATH.to_string()
}

pub fn cache_dir() -> Option<PathBuf> {
    dirs_cache_dir().map(|p| p.join(APP_NAME))
}

pub fn cache_path() -> Option<PathBuf> {
    cache_dir().map(|p| p.join(CACHE_FILE_NAME))
}

pub fn data_dir() -> Option<PathBuf> {
    dirs_data_local_dir().map(|p| p.join(APP_NAME))
}

pub fn logs_path() -> Option<PathBuf> {
    data_dir().map(|p| p.join(LOGS_FILE_NAME))
}

pub fn proton_config_dir() -> Option<PathBuf> {
    dirs_config_dir().map(|p| p.join(PROTON_DIR).join("VPN"))
}

pub fn proton_settings_path() -> Option<PathBuf> {
    proton_config_dir().map(|p| p.join(PROTON_SETTINGS_FILE))
}

pub fn proton_connection_dir() -> Option<PathBuf> {
    dirs_cache_dir().map(|p| p.join(PROTON_DIR).join("VPN").join("connection"))
}

pub fn proton_connection_persistence_path() -> Option<PathBuf> {
    proton_connection_dir().map(|p| p.join(PROTON_CONNECTION_PERSISTENCE_FILE))
}

pub fn proton_connection_persistence_fallback() -> PathBuf {
    dirs_cache_dir()
        .unwrap_or_else(base_dir)
        .join(PROTON_DIR)
        .join("VPN")
        .join("connection")
        .join(PROTON_CONNECTION_PERSISTENCE_FILE)
}

/// Get ProtonVPN runtime directory (XDG_RUNTIME_DIR/Proton/VPN)
pub fn proton_runtime_dir() -> Option<PathBuf> {
    std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(|p| PathBuf::from(p).join(PROTON_DIR).join("VPN"))
}

pub fn proton_forwarded_port_path() -> Option<PathBuf> {
    proton_runtime_dir().map(|p| p.join("forwarded_port"))
}
