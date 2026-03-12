use crate::constants::state::MAX_NOTIFICATION_LOG;
use crate::constants::ui::{MAX_VISIBLE_NOTIFICATIONS, NOTIFICATION_TIMER_DEFAULT};
use crate::state::log_persistence;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u16,
}

pub struct NotificationState {
    pub notifications: Vec<ToastNotification>,
    pub notification_log: Vec<Notification>,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            notification_log: log_persistence::load_notification_log(),
        }
    }

    pub fn show(&mut self, message: String, notification_type: NotificationType) {
        self.notifications.push(ToastNotification {
            message: message.clone(),
            notification_type,
            timer: NOTIFICATION_TIMER_DEFAULT,
        });

        if self.notifications.len() > MAX_VISIBLE_NOTIFICATIONS {
            self.notifications.remove(0);
        }

        self.notification_log.push(Notification {
            message,
            notification_type,
            timestamp: Utc::now(),
        });
        if self.notification_log.len() > MAX_NOTIFICATION_LOG {
            self.notification_log.remove(0);
        }

        log_persistence::save_notification_log(&self.notification_log);
    }

    pub fn tick(&mut self) {
        for notification in &mut self.notifications {
            if notification.timer > 0 {
                notification.timer -= 1;
            }
        }
        self.notifications.retain(|n| n.timer > 0);
    }
}
