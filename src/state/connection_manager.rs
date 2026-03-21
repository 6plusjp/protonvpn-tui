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
        let mut pending = self.pending.lock().unwrap();
        pending.push(event);
        self.condvar.notify_one();
    }

    pub fn try_recv_all(&self) -> Vec<AsyncEvent> {
        let mut pending = self.pending.lock().unwrap();
        pending.drain(..).collect()
    }

    pub fn wait_timeout(&self, duration: std::time::Duration) -> Vec<AsyncEvent> {
        let guard = self.pending.lock().unwrap();
        let (mut remaining, _timeout_result) = self.condvar.wait_timeout(guard, duration).unwrap();
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
