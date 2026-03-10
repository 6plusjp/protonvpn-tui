//! Async task management using a custom thread pool for blocking I/O operations.
//!
//! Uses a fixed-size thread pool to avoid spawning a new thread for every operation.
//! This significantly reduces overhead for frequent VPN operations.

use crate::error::AppError;
use crate::vpn::{City, VpnState};
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

pub type AsyncResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AsyncOperation {
    RefreshComplete(Result<Vec<crate::vpn::Server>, AppError>),
    ConnectComplete(Result<(String, Option<String>), AppError>),
    DisconnectComplete(Result<(), AppError>),
    CitiesComplete(Result<Vec<City>, AppError>),
}

enum Job {
    RefreshServers {
        vpn_state: Arc<VpnState>,
        sender: mpsc::Sender<AsyncResult<Vec<crate::vpn::Server>>>,
    },
    Connect {
        vpn_state: Arc<VpnState>,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    },
    Disconnect {
        vpn_state: Arc<VpnState>,
        sender: mpsc::Sender<AsyncResult<()>>,
    },
    ConnectRandom {
        vpn_state: Arc<VpnState>,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    },
    Cities {
        vpn_state: Arc<VpnState>,
        country_code: String,
        sender: mpsc::Sender<AsyncResult<Vec<City>>>,
    },
    ConnectCity {
        vpn_state: Arc<VpnState>,
        city: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    },
}

/// Custom thread pool for executing VPN operations.
/// Uses a fixed number of worker threads to reduce thread spawning overhead.
pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    job_queue: Arc<Mutex<VecDeque<Job>>>,
    shutdown: Arc<Mutex<bool>>,
}

impl ThreadPool {
    /// Create a new thread pool with the specified number of workers.
    pub fn new(num_workers: usize) -> Self {
        let job_queue = Arc::new(Mutex::new(VecDeque::new()));
        let shutdown = Arc::new(Mutex::new(false));

        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let job_queue = Arc::clone(&job_queue);
            let shutdown = Arc::clone(&shutdown);

            let worker = thread::spawn(move || loop {
                // Check shutdown flag with proper error handling
                let should_shutdown = shutdown.lock().map(|guard| *guard).unwrap_or_else(|e| {
                    tracing::warn!("ThreadPool shutdown lock poisoned: {}", e);
                    true // Treat poison as shutdown signal
                });
                if should_shutdown {
                    break;
                }

                let job = {
                    let queue_guard = job_queue.lock().map_err(|e| {
                        tracing::warn!("ThreadPool queue lock poisoned: {}", e);
                    });
                    match queue_guard {
                        Ok(mut queue) => queue.pop_front(),
                        Err(_) => None,
                    }
                };

                match job {
                    Some(job) => {
                        Self::execute_job(job);
                    }
                    None => {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            });

            workers.push(worker);
        }

        Self {
            workers,
            job_queue,
            shutdown,
        }
    }

    /// Execute a job
    fn execute_job(job: Job) {
        match job {
            Job::RefreshServers { vpn_state, sender } => {
                let result = vpn_state.refresh_servers();
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send refresh result - receiver dropped");
                }
            }
            Job::Connect {
                vpn_state,
                server_id,
                sender,
            } => {
                let result = vpn_state.connect(&server_id);
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send connect result - receiver dropped");
                }
            }
            Job::Disconnect { vpn_state, sender } => {
                let result = vpn_state.disconnect();
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send disconnect result - receiver dropped");
                }
            }
            Job::ConnectRandom { vpn_state, sender } => {
                let result = vpn_state.connect_random();
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send connect_random result - receiver dropped");
                }
            }
            Job::Cities {
                vpn_state,
                country_code,
                sender,
            } => {
                let result = vpn_state.list_cities_with_features(&country_code);
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send cities result - receiver dropped");
                }
            }
            Job::ConnectCity {
                vpn_state,
                city,
                sender,
            } => {
                let result = vpn_state.connect_city(&city);
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send connect_city result - receiver dropped");
                }
            }
        }
    }

    /// Submit a job to the thread pool
    fn submit(&self, job: Job) {
        if let Ok(mut queue) = self.job_queue.lock() {
            queue.push_back(job);
        } else {
            tracing::error!("Failed to lock job queue for submission");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.shutdown.lock() {
            *guard = true;
        }

        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// Async task manager using thread pool for VPN operations.
#[derive(Clone)]
pub struct AsyncTaskManager {
    pool: Arc<ThreadPool>,
}

impl AsyncTaskManager {
    /// Create a new AsyncTaskManager with default number of workers (4).
    pub fn new() -> Self {
        Self::new_with_workers(4)
    }

    /// Create a new AsyncTaskManager with a custom number of workers.
    pub fn new_with_workers(num_workers: usize) -> Self {
        Self {
            pool: Arc::new(ThreadPool::new(num_workers)),
        }
    }

    pub fn spawn_refresh_servers(
        &self,
        vpn_state: Arc<VpnState>,
        sender: mpsc::Sender<AsyncResult<Vec<crate::vpn::Server>>>,
    ) {
        self.pool.submit(Job::RefreshServers { vpn_state, sender });
    }

    pub fn spawn_connect(
        &self,
        vpn_state: Arc<VpnState>,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        self.pool.submit(Job::Connect {
            vpn_state,
            server_id,
            sender,
        });
    }

    pub fn spawn_disconnect(
        &self,
        vpn_state: Arc<VpnState>,
        sender: mpsc::Sender<AsyncResult<()>>,
    ) {
        self.pool.submit(Job::Disconnect { vpn_state, sender });
    }

    pub fn spawn_connect_random(
        &self,
        vpn_state: Arc<VpnState>,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        self.pool.submit(Job::ConnectRandom { vpn_state, sender });
    }

    pub fn spawn_cities(
        &self,
        vpn_state: Arc<VpnState>,
        country_code: String,
        sender: mpsc::Sender<AsyncResult<Vec<City>>>,
    ) {
        self.pool.submit(Job::Cities {
            vpn_state,
            country_code,
            sender,
        });
    }

    pub fn spawn_connect_city(
        &self,
        vpn_state: Arc<VpnState>,
        city: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
    ) {
        self.pool.submit(Job::ConnectCity {
            vpn_state,
            city,
            sender,
        });
    }
}

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new_with_workers(4)
    }
}

pub fn create_channel<T>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel()
}
