//! Notification types

use crate::constants::state::MAX_NOTIFICATION_LOG;
use crate::constants::ui::{MAX_VISIBLE_NOTIFICATIONS, NOTIFICATION_TIMER_DEFAULT};
use crate::state::log_persistence;
use chrono::{DateTime, Utc};

/// Notification type for UI feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// Notification popup (legacy - used for notification_log)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: DateTime<Utc>,
}

/// Toast notification with individual timer for stacked display
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u16,
}

/// Notification state - contains all notification-related fields
/// This struct can be extracted to notifications.rs in Phase 2.2
#[derive(Clone)]
pub struct NotificationState {
    pub(crate) notifications: Vec<ToastNotification>,
    pub(crate) notification_log: Vec<Notification>,
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

    // === Getters ===

    /// Get active toast notifications
    pub fn get_notifications(&self) -> &Vec<ToastNotification> {
        &self.notifications
    }

    /// Get notification log (history)
    pub fn get_notification_log(&self) -> &Vec<Notification> {
        &self.notification_log
    }

    /// Get notification log length
    pub fn len(&self) -> usize {
        self.notification_log.len()
    }

    /// Check if notifications is empty
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    /// Check if notification log is empty
    pub fn is_log_empty(&self) -> bool {
        self.notification_log.is_empty()
    }

    // === Mutation methods (delegated from AppState) ===

    /// Show a notification
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

    /// Clear all notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// Tick down notification timers and remove expired ones
    pub fn tick(&mut self) {
        for notification in &mut self.notifications {
            if notification.timer > 0 {
                notification.timer -= 1;
            }
        }
        self.notifications.retain(|n| n.timer > 0);
    }
}
