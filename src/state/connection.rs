//! Connection-related async types

use crate::vpn::ConnectResult;
use crate::vpn::Server;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

use crate::state::async_tasks::AsyncTaskManager;
use crate::state::ConnectionState;

/// Async event types for event-driven notification
#[derive(Debug, Clone)]
pub enum AsyncEvent {
    ServersRefreshed(Vec<Server>),
    ServersRefreshFailed(String),
    Connected(String, Option<String>, Option<String>, Option<String>),
    ConnectFailed(String),
    Disconnected,
    DisconnectFailed(String),
    CitiesLoaded(String, Vec<crate::vpn::City>),
    ConnectCityResult(String, Option<String>, Option<String>, Option<String>),
    ConnectCityFailed(String),
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

/// Receiver types for async operations
pub type ConnectReceiver = std::sync::mpsc::Receiver<crate::state::AsyncResult<ConnectResult>>;
pub type ServerReceiver = std::sync::mpsc::Receiver<crate::state::AsyncResult<Vec<Server>>>;
pub type DisconnectReceiver = std::sync::mpsc::Receiver<crate::state::AsyncResult<()>>;
pub type CitiesReceiver =
    std::sync::mpsc::Receiver<crate::state::AsyncResult<Vec<crate::vpn::City>>>;
pub type ConfigReceiver = std::sync::mpsc::Receiver<crate::state::AsyncResult<String>>;

/// Connection manager - handles VPN connection state and async operations
pub struct ConnectionManager {
    pub connection: ConnectionState,
    pub previous_connection: Option<ConnectionState>,
    pub async_manager: AsyncTaskManager,
    pub async_notifier: Arc<AsyncNotifier>,
    pub pending_refresh: HashMap<(), ServerReceiver>,
    pub pending_connect: HashMap<(), ConnectReceiver>,
    pub pending_disconnect: HashMap<(), DisconnectReceiver>,
    pub pending_cities: HashMap<String, CitiesReceiver>,
    pub pending_connect_city: HashMap<(), ConnectReceiver>,
    pub pending_config_set: HashMap<(), ConfigReceiver>,
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
            pending_refresh: HashMap::new(),
            pending_connect: HashMap::new(),
            pending_disconnect: HashMap::new(),
            pending_cities: HashMap::new(),
            pending_connect_city: HashMap::new(),
            pending_config_set: HashMap::new(),
        }
    }

    pub fn set_connection(&mut self, state: ConnectionState) {
        self.previous_connection = Some(std::mem::replace(&mut self.connection, state));
    }

    pub fn is_idle(&self) -> bool {
        self.pending_refresh.is_empty()
            && self.pending_connect.is_empty()
            && self.pending_disconnect.is_empty()
            && self.pending_cities.is_empty()
            && self.pending_connect_city.is_empty()
            && self.pending_config_set.is_empty()
    }
}
