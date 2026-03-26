pub mod paths {
    /// Application name (kebab-case for app-specific files)
    pub const APP_NAME: &str = "protonvpn-tui";

    /// Proton VPN official directory name (PascalCase - matches Proton's actual structure)
    pub const PROTON_DIR: &str = "Proton";

    // File names
    pub const CONFIG_FILE_NAME: &str = "config.toml";
    pub const CACHE_FILE_NAME: &str = "server_cache.toml";
    pub const LOGS_FILE_NAME: &str = "logs.json";
    pub const APP_LOG_FILE: &str = "protonvpn-tui.log";

    // Proton VPN official file names
    pub const PROTON_SETTINGS_FILE: &str = "settings.json";
    pub const PROTON_CONNECTION_PERSISTENCE_FILE: &str = "connection_persistence.json";

    // Display paths (for user-facing messages)
    pub const CONFIG_DISPLAY_PATH: &str = "~/.config/protonvpn-tui/config.toml";
}

pub mod ui {
    pub const NOTIFICATION_TIMER_DEFAULT: u16 = 300; // 30 × 10 (loop interval 10ms)
    pub const NOTIFICATION_TIMER_SHORT: u16 = 150; // 15 × 10 (loop interval 10ms)
    pub const NOTIFICATION_MSG_MAX_LEN: usize = 35;
    pub const POPUP_WIDTH_MIN: usize = 30;
    pub const POPUP_WIDTH_MAX: usize = 54;
    pub const MAX_VISIBLE_NOTIFICATIONS: usize = 3;
}

pub mod state {
    pub const PAGE_SIZE: usize = 10;
    pub const MAX_NOTIFICATION_LOG: usize = 100;
}

pub mod vpn {
    use std::time::Duration;

    pub const DISCONNECT_RETRY_COUNT: usize = 10;
    pub const DISCONNECT_RETRY_DELAY_MS: u64 = 500;

    pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(15);
    pub const COUNTRIES_LIST_TIMEOUT: Duration = Duration::from_secs(60);
    pub const CITIES_LIST_TIMEOUT: Duration = Duration::from_secs(20);
    pub const CONFIG_SET_TIMEOUT: Duration = Duration::from_secs(20);
}
