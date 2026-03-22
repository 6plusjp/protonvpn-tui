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
    pub operation_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u16,
    pub operation_key: Option<String>,
}

pub struct NotificationState {
    pub notifications: Vec<ToastNotification>,
    pub notification_log: Vec<Notification>,
    log_dirty: bool,
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
            log_dirty: false,
        }
    }

    pub fn show(
        &mut self,
        message: String,
        notification_type: NotificationType,
        operation_key: Option<String>,
    ) {
        let is_completion = matches!(
            notification_type,
            NotificationType::Success | NotificationType::Error
        );

        if is_completion {
            if let Some(ref key) = operation_key {
                self.notifications.retain(|n| {
                    n.operation_key.as_ref() != Some(key)
                        || n.notification_type != NotificationType::Info
                });
            }
        }

        if let Some(ref key) = operation_key {
            self.notifications
                .retain(|n| n.operation_key.as_ref() != Some(key));
        }

        self.notifications.push(ToastNotification {
            message: message.clone(),
            notification_type,
            timer: NOTIFICATION_TIMER_DEFAULT,
            operation_key: operation_key.clone(),
        });

        if self.notifications.len() > MAX_VISIBLE_NOTIFICATIONS {
            self.notifications.remove(0);
        }

        self.notification_log.push(Notification {
            message,
            notification_type,
            timestamp: Utc::now(),
            operation_key,
        });
        if self.notification_log.len() > MAX_NOTIFICATION_LOG {
            self.notification_log.remove(0);
        }

        self.log_dirty = true;
    }

    pub fn flush_if_dirty(&mut self) {
        if self.log_dirty {
            log_persistence::save_notification_log(&self.notification_log);
            self.log_dirty = false;
        }
    }

    /// Decrements timers and returns true if any notifications were removed
    pub fn tick(&mut self) -> bool {
        let before = self.notifications.len();
        for notification in &mut self.notifications {
            if notification.timer > 0 {
                notification.timer -= 1;
            }
        }
        self.notifications.retain(|n| n.timer > 0);
        self.notifications.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        crate::state::log_persistence::set_test_mode(true);
    }

    #[test]
    fn test_same_key_loading_replaces() {
        setup();
        let mut state = NotificationState::new();
        state.show(
            "Loading...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );
        assert_eq!(state.notifications.len(), 1);

        state.show(
            "Loading...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(state.notifications[0].message, "Loading...");
    }

    #[test]
    fn test_loaded_dismisses_loading() {
        setup();
        let mut state = NotificationState::new();
        state.show(
            "Loading...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );
        assert_eq!(state.notifications.len(), 1);

        state.show(
            "Loaded!".to_string(),
            NotificationType::Success,
            Some("servers".to_string()),
        );
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(state.notifications[0].message, "Loaded!");
    }

    #[test]
    fn test_error_dismisses_loading() {
        setup();
        let mut state = NotificationState::new();
        state.show(
            "Loading...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );
        assert_eq!(state.notifications.len(), 1);

        state.show(
            "Failed!".to_string(),
            NotificationType::Error,
            Some("servers".to_string()),
        );
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(state.notifications[0].message, "Failed!");
    }

    #[test]
    fn test_different_keys_stack() {
        setup();
        let mut state = NotificationState::new();
        state.show(
            "Loading servers...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );
        state.show(
            "Loading cities...".to_string(),
            NotificationType::Info,
            Some("cities".to_string()),
        );
        assert_eq!(state.notifications.len(), 2);
    }

    #[test]
    fn test_notification_log_preserves_all_with_keys() {
        setup();
        let mut state = NotificationState::new();
        state.show(
            "Loading...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );
        state.show(
            "Loaded!".to_string(),
            NotificationType::Success,
            Some("servers".to_string()),
        );
        assert_eq!(state.notification_log.len(), 2);
    }

    #[test]
    fn test_none_key_does_not_replace() {
        setup();
        let mut state = NotificationState::new();
        state.show("Message 1".to_string(), NotificationType::Info, None);
        state.show("Message 2".to_string(), NotificationType::Info, None);
        assert_eq!(state.notifications.len(), 2);
    }
}
