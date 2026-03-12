//! Async task management using a custom thread pool for blocking I/O operations.
//!
//! Uses a fixed-size thread pool to avoid spawning a new thread for every operation.
//! This significantly reduces overhead for frequent VPN operations.

use crate::error::AppError;
use crate::vpn::{City, VpnClient};
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

pub type AsyncResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AsyncOperation {
    RefreshComplete(Result<Vec<crate::vpn::Server>, AppError>),
    ConnectComplete(Result<(String, Option<String>, Option<String>, Option<String>), AppError>),
    DisconnectComplete(Result<(), AppError>),
    CitiesComplete(Result<Vec<City>, AppError>),
}

enum Job {
    RefreshServers {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<Vec<crate::vpn::Server>>>,
    },
    Connect {
        vpn_state: Arc<VpnClient>,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>, Option<String>, Option<String>)>>,
    },
    Disconnect {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<()>>,
    },
    ConnectRandom {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>, Option<String>, Option<String>)>>,
    },
    Cities {
        vpn_state: Arc<VpnClient>,
        country_code: String,
        sender: mpsc::Sender<AsyncResult<Vec<City>>>,
    },
    ConnectCity {
        vpn_state: Arc<VpnClient>,
        city: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>, Option<String>, Option<String>)>>,
    },
    ConfigSet {
        vpn_state: Arc<VpnClient>,
        key: String,
        value: String,
        sender: mpsc::Sender<AsyncResult<String>>,
    },
}

/// Custom thread pool for executing VPN operations.
/// Uses a fixed number of worker threads to reduce thread spawning overhead.
pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    job_queue: Arc<Mutex<VecDeque<Job>>>,
    not_empty: Arc<Condvar>,
    shutdown: Arc<Mutex<bool>>,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let job_queue = Arc::new(Mutex::new(VecDeque::new()));
        let not_empty = Arc::new(Condvar::new());
        let shutdown = Arc::new(Mutex::new(false));

        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let job_queue = Arc::clone(&job_queue);
            let not_empty = Arc::clone(&not_empty);
            let shutdown = Arc::clone(&shutdown);

            let worker = thread::spawn(move || loop {
                let mut queue = job_queue.lock().unwrap();
                while queue.is_empty() {
                    if *shutdown.lock().unwrap() {
                        return;
                    }
                    queue = not_empty.wait(queue).unwrap();
                }
                if *shutdown.lock().unwrap() {
                    return;
                }
                let job = queue.pop_front();
                drop(queue);

                if let Some(job) = job {
                    Self::execute_job(job);
                }
            });

            workers.push(worker);
        }

        Self {
            workers,
            job_queue,
            not_empty,
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
                let result = vpn_state.connect_country(&server_id);
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
            Job::ConfigSet {
                vpn_state,
                key,
                value,
                sender,
            } => {
                let result = vpn_state.set_config(&key, &value);
                if sender.send(result).is_err() {
                    tracing::warn!("Failed to send config_set result - receiver dropped");
                }
            }
        }
    }

    fn submit(&self, job: Job) {
        if let Ok(mut queue) = self.job_queue.lock() {
            queue.push_back(job);
            self.not_empty.notify_one();
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
        self.not_empty.notify_all();

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
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<Vec<crate::vpn::Server>>>,
    ) {
        self.pool.submit(Job::RefreshServers { vpn_state, sender });
    }

    pub fn spawn_connect(
        &self,
        vpn_state: Arc<VpnClient>,
        server_id: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>, Option<String>, Option<String>)>>,
    ) {
        self.pool.submit(Job::Connect {
            vpn_state,
            server_id,
            sender,
        });
    }

    pub fn spawn_disconnect(
        &self,
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<()>>,
    ) {
        self.pool.submit(Job::Disconnect { vpn_state, sender });
    }

    pub fn spawn_connect_random(
        &self,
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>, Option<String>, Option<String>)>>,
    ) {
        self.pool.submit(Job::ConnectRandom { vpn_state, sender });
    }

    pub fn spawn_cities(
        &self,
        vpn_state: Arc<VpnClient>,
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
        vpn_state: Arc<VpnClient>,
        city: String,
        sender: mpsc::Sender<AsyncResult<(String, Option<String>, Option<String>, Option<String>)>>,
    ) {
        self.pool.submit(Job::ConnectCity {
            vpn_state,
            city,
            sender,
        });
    }

    pub fn spawn_config_set(
        &self,
        vpn_state: Arc<VpnClient>,
        key: String,
        value: String,
        sender: mpsc::Sender<AsyncResult<String>>,
    ) {
        self.pool.submit(Job::ConfigSet {
            vpn_state,
            key,
            value,
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
