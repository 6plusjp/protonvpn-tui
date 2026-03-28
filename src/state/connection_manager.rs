//! Connection-related async types

use crate::vpn::ConnectResult;
use crate::vpn::Server;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

use crate::state::ConnectionState;
use crate::vpn::async_tasks::AsyncTaskManager;

/// Async event types for event-driven notification
#[derive(Debug, Clone)]
pub enum AsyncEvent {
    ServersRefreshed(Vec<Server>),
    ServersRefreshFailed(String),
    Connected(ConnectResult),
    ConnectFailed(String),
    Disconnected,
    DisconnectFailed(String),
    CitiesLoaded(String, Vec<crate::vpn::City>),
    CitiesLoadFailed(String, String),
    ConnectCityResult(ConnectResult),
    ConnectCityFailed(String),
    ConfigSetResult(String),
    ConfigSetFailed(String),
}

/// Notifier for async task completion (event-driven wakeup)
pub struct AsyncNotifier {
    pending: Mutex<Vec<AsyncEvent>>,
    condvar: Condvar,
}

impl AsyncNotifier {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            condvar: Condvar::new(),
        }
    }

    pub fn notify(&self, event: AsyncEvent) {
        let mut pending = self.pending.lock().expect("pending mutex poisoned");
        pending.push(event);
        self.condvar.notify_one();
    }

    pub fn try_recv_all(&self) -> Vec<AsyncEvent> {
        let mut pending = self.pending.lock().expect("pending mutex poisoned");
        pending.drain(..).collect()
    }

    pub fn wait_timeout(&self, duration: std::time::Duration) -> Vec<AsyncEvent> {
        let guard = self.pending.lock().expect("pending mutex poisoned");
        let (mut remaining, _timeout_result) = self
            .condvar
            .wait_timeout(guard, duration)
            .expect("condvar wait_timeout failed");
        remaining.drain(..).collect()
    }
}

impl Default for AsyncNotifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection manager - handles VPN connection state and async operations
pub struct ConnectionManager {
    pub connection: ConnectionState,
    pub previous_connection: Option<ConnectionState>,
    pub async_manager: AsyncTaskManager,
    pub async_notifier: Arc<AsyncNotifier>,
    pub loading_cities: HashSet<String>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            previous_connection: None,
            async_manager: AsyncTaskManager::new(),
            async_notifier: Arc::new(AsyncNotifier::new()),
            loading_cities: HashSet::new(),
        }
    }

    pub fn is_idle(&self) -> bool {
        self.connection.is_disconnected() && self.loading_cities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_notifier_new() {
        let notifier = AsyncNotifier::new();
        let events = notifier.try_recv_all();
        assert!(events.is_empty());
    }

    #[test]
    fn test_async_notifier_notify_and_recv() {
        let notifier = AsyncNotifier::new();
        notifier.notify(AsyncEvent::Disconnected);

        let events = notifier.try_recv_all();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], AsyncEvent::Disconnected));
    }

    #[test]
    fn test_async_notifier_multiple_events() {
        let notifier = AsyncNotifier::new();
        notifier.notify(AsyncEvent::Disconnected);
        notifier.notify(AsyncEvent::ServersRefreshed(vec![]));

        let events = notifier.try_recv_all();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_connection_manager_new() {
        let manager = ConnectionManager::new();
        assert!(manager.connection.is_disconnected());
        assert!(manager.previous_connection.is_none());
        assert!(manager.loading_cities.is_empty());
    }

    #[test]
    fn test_connection_manager_is_idle_when_disconnected() {
        let manager = ConnectionManager::new();
        assert!(manager.is_idle());
    }

    #[test]
    fn test_connection_manager_is_not_idle_when_connecting() {
        let mut manager = ConnectionManager::new();
        manager.connection = ConnectionState::Connecting;
        assert!(!manager.is_idle());
    }
}
