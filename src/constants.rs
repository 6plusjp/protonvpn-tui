pub mod ui {
    pub const NOTIFICATION_TIMER_DEFAULT: u8 = 30;
    pub const NOTIFICATION_TIMER_SHORT: u8 = 15;
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
    pub const DISCONNECT_RETRY_COUNT: usize = 10;
    pub const DISCONNECT_RETRY_DELAY_MS: u64 = 500;
}
