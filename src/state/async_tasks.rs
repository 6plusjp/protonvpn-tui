//! Async task management using tokio for blocking I/O operations.

use crate::error::AppError;
use crate::vpn::VpnState;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

pub type AsyncResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AsyncOperation {
    RefreshComplete(Result<Vec<crate::vpn::Server>, AppError>),
    ConnectComplete(Result<(String, Option<String>), AppError>),
    DisconnectComplete(Result<(), AppError>),
}

pub struct AsyncTaskManager {
    runtime: Arc<tokio::runtime::Runtime>,
    handle: Arc<Handle>,
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("protonvpn-async")
            .build()
            .expect("Failed to create tokio runtime");

        let runtime = Arc::new(runtime);
        let handle = Arc::new(runtime.handle().clone());

        Self { runtime, handle }
    }

    pub fn handle(&self) -> Arc<Handle> {
        self.handle.clone()
    }
}

impl AsyncTaskManager {
    pub fn spawn_refresh_servers(
        &self,
        vpn_state: VpnState,
        sender: mpsc::Sender<AsyncResult<Vec<crate::vpn::Server>>>,
    ) {
        let handle = self.handle.clone();
        handle.spawn_blocking(move || {
            let mut state = vpn_state;
            let result = state.refresh_servers();
            let _ = sender.blocking_send(result);
        });
    }

    pub fn spawn_connect(
        &self,
        vpn_state: VpnState,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        let handle = self.handle.clone();
        handle.spawn_blocking(move || {
            let mut state = vpn_state;
            let result = state.connect(&server_id);
            let _ = sender.blocking_send(result);
        });
    }

    pub fn spawn_disconnect(&self, vpn_state: VpnState, sender: mpsc::Sender<AsyncResult<()>>) {
        let handle = self.handle.clone();
        handle.spawn_blocking(move || {
            let mut state = vpn_state;
            let result = state.disconnect();
            let _ = sender.blocking_send(result);
        });
    }

    pub fn spawn_connect_random(
        &self,
        vpn_state: VpnState,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        let handle = self.handle.clone();
        handle.spawn_blocking(move || {
            let mut state = vpn_state;
            let result = state.connect_random();
            let _ = sender.blocking_send(result);
        });
    }
}

impl Clone for AsyncTaskManager {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_channel<T>(buffer: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel(buffer)
}
