//! Async task management using a custom thread pool for blocking I/O operations.
//!
//! Uses a fixed-size thread pool to avoid spawning a new thread for every operation.
//! This significantly reduces overhead for frequent VPN operations.

use crate::error::AppError;
use crate::vpn::ConnectResult;
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
    ConnectComplete(Result<ConnectResult, AppError>),
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
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    },
    Disconnect {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<()>>,
    },
    ConnectRandom {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    },
    Cities {
        vpn_state: Arc<VpnClient>,
        country_code: String,
        sender: mpsc::Sender<AsyncResult<Vec<City>>>,
    },
    ConnectCity {
        vpn_state: Arc<VpnClient>,
        city: String,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    },
    ConfigSet {
        vpn_state: Arc<VpnClient>,
        key: String,
        value: String,
        sender: mpsc::Sender<AsyncResult<String>>,
    },
    ConnectFastest {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    },
    ConnectP2P {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    },
    ConnectTor {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    },
    ConnectSecureCore {
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
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
                Self::send_result(vpn_state.refresh_servers(), &sender, "refresh");
            }
            Job::Connect {
                vpn_state,
                server_id,
                sender,
            } => {
                Self::send_result(vpn_state.connect_country(&server_id), &sender, "connect");
            }
            Job::Disconnect { vpn_state, sender } => {
                Self::send_result(vpn_state.disconnect(), &sender, "disconnect");
            }
            Job::ConnectRandom { vpn_state, sender } => {
                Self::send_result(vpn_state.connect_random(), &sender, "connect_random");
            }
            Job::Cities {
                vpn_state,
                country_code,
                sender,
            } => {
                Self::send_result(
                    vpn_state.list_cities_with_features(&country_code),
                    &sender,
                    "cities",
                );
            }
            Job::ConnectCity {
                vpn_state,
                city,
                sender,
            } => {
                Self::send_result(vpn_state.connect_city(&city), &sender, "connect_city");
            }
            Job::ConfigSet {
                vpn_state,
                key,
                value,
                sender,
            } => {
                Self::send_result(vpn_state.set_config(&key, &value), &sender, "config_set");
            }
            Job::ConnectFastest { vpn_state, sender } => {
                Self::send_result(vpn_state.connect_fastest(), &sender, "connect_fastest");
            }
            Job::ConnectP2P { vpn_state, sender } => {
                Self::send_result(vpn_state.connect_p2p(), &sender, "connect_p2p");
            }
            Job::ConnectTor { vpn_state, sender } => {
                Self::send_result(vpn_state.connect_tor(), &sender, "connect_tor");
            }
            Job::ConnectSecureCore { vpn_state, sender } => {
                Self::send_result(
                    vpn_state.connect_securecore(),
                    &sender,
                    "connect_securecore",
                );
            }
        }
    }

    fn send_result<T>(result: T, sender: &mpsc::Sender<T>, operation: &str) {
        if sender.send(result).is_err() {
            tracing::warn!("Failed to send {} result - receiver dropped", operation);
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
    /// Create a new AsyncTaskManager with default number of workers (10).
    pub fn new() -> Self {
        Self::new_with_workers(10)
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
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
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
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
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
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
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

    pub fn spawn_connect_fastest(
        &self,
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    ) {
        self.pool.submit(Job::ConnectFastest { vpn_state, sender });
    }

    pub fn spawn_connect_p2p(
        &self,
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    ) {
        self.pool.submit(Job::ConnectP2P { vpn_state, sender });
    }

    pub fn spawn_connect_tor(
        &self,
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    ) {
        self.pool.submit(Job::ConnectTor { vpn_state, sender });
    }

    pub fn spawn_connect_securecore(
        &self,
        vpn_state: Arc<VpnClient>,
        sender: mpsc::Sender<AsyncResult<ConnectResult>>,
    ) {
        self.pool
            .submit(Job::ConnectSecureCore { vpn_state, sender });
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
