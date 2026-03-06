//! Async task management using std::thread for blocking I/O operations.
//!
//! Uses std::thread since all operations are blocking CLI commands.
//! This avoids the overhead of tokio runtime while maintaining the same functionality.

use crate::error::AppError;
use crate::vpn::{City, VpnState};
use std::sync::mpsc;

pub type AsyncResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AsyncOperation {
    RefreshComplete(Result<Vec<crate::vpn::Server>, AppError>),
    ConnectComplete(Result<(String, Option<String>), AppError>),
    DisconnectComplete(Result<(), AppError>),
    CitiesComplete(Result<Vec<City>, AppError>),
}

#[derive(Clone)]
pub struct AsyncTaskManager;

impl AsyncTaskManager {
    pub fn new() -> Self {
        Self
    }
}

impl AsyncTaskManager {
    pub fn spawn_refresh_servers(
        &self,
        vpn_state: VpnState,
        sender: mpsc::Sender<AsyncResult<Vec<crate::vpn::Server>>>,
    ) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.refresh_servers();
            let _ = sender.send(result);
        });
    }

    pub fn spawn_connect(
        &self,
        vpn_state: VpnState,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.connect(&server_id);
            let _ = sender.send(result);
        });
    }

    pub fn spawn_disconnect(&self, vpn_state: VpnState, sender: mpsc::Sender<AsyncResult<()>>) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.disconnect();
            let _ = sender.send(result);
        });
    }

    pub fn spawn_connect_random(
        &self,
        vpn_state: VpnState,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.connect_random();
            let _ = sender.send(result);
        });
    }

    pub fn spawn_cities(
        &self,
        vpn_state: VpnState,
        country_code: String,
        sender: mpsc::Sender<AsyncResult<Vec<City>>>,
    ) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.list_cities_with_features(&country_code);
            let _ = sender.send(result);
        });
    }

    pub fn spawn_connect_city(
        &self,
        vpn_state: VpnState,
        city: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        std::thread::spawn(move || {
            let mut state = vpn_state;
            let result = state.connect_city(&city);
            let _ = sender.send(result);
        });
    }
}

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_channel<T>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel()
}
