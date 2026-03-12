//! Connection-related async types

use crate::vpn::Server;
use std::sync::Condvar;
use std::sync::Mutex;

/// Async event types for event-driven notification
#[derive(Debug, Clone)]
pub enum AsyncEvent {
    ServersRefreshed(Vec<Server>),
    ServersRefreshFailed(String),
    Connected(String, Option<String>),
    ConnectFailed(String),
    Disconnected,
    DisconnectFailed(String),
    CitiesLoaded(String, Vec<crate::vpn::City>),
    ConnectCityResult(String, Option<String>),
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
