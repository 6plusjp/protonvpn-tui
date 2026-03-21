//! Async task management using a custom thread pool for blocking I/O operations.
//!
//! Uses a fixed-size thread pool to avoid spawning a new thread for every operation.
//! This significantly reduces overhead for frequent VPN operations.

use crate::state::AsyncEvent;
use crate::state::AsyncNotifier;
use crate::vpn::VpnClient;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

enum Job {
    RefreshServers {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    },
    Connect {
        vpn_state: Arc<VpnClient>,
        server_id: String,
        notifier: Arc<AsyncNotifier>,
    },
    Disconnect {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    },
    ConnectRandom {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    },
    Cities {
        vpn_state: Arc<VpnClient>,
        country_code: String,
        notifier: Arc<AsyncNotifier>,
    },
    ConnectCity {
        vpn_state: Arc<VpnClient>,
        city: String,
        notifier: Arc<AsyncNotifier>,
    },
    ConfigSet {
        vpn_state: Arc<VpnClient>,
        key: String,
        value: String,
        notifier: Arc<AsyncNotifier>,
    },
    ConnectFastest {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    },
    ConnectP2P {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    },
    ConnectTor {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    },
    ConnectSecureCore {
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
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
            Job::RefreshServers {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.refresh_servers();
                match result {
                    Ok(servers) => notifier.notify(AsyncEvent::ServersRefreshed(servers)),
                    Err(e) => notifier.notify(AsyncEvent::ServersRefreshFailed(e.to_string())),
                }
            }
            Job::Connect {
                vpn_state,
                server_id,
                notifier,
            } => {
                let result = vpn_state.connect_country(&server_id);
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::Connected(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectFailed(e.to_string())),
                }
            }
            Job::Disconnect {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.disconnect();
                match result {
                    Ok(()) => notifier.notify(AsyncEvent::Disconnected),
                    Err(e) => notifier.notify(AsyncEvent::DisconnectFailed(e.to_string())),
                }
            }
            Job::ConnectRandom {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.connect_random();
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::Connected(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectFailed(e.to_string())),
                }
            }
            Job::Cities {
                vpn_state,
                country_code,
                notifier,
            } => {
                let result = vpn_state.list_cities_with_features(&country_code);
                match result {
                    Ok(cities) => notifier.notify(AsyncEvent::CitiesLoaded(country_code, cities)),
                    Err(e) => {
                        notifier.notify(AsyncEvent::CitiesLoadFailed(country_code, e.to_string()))
                    }
                }
            }
            Job::ConnectCity {
                vpn_state,
                city,
                notifier,
            } => {
                let result = vpn_state.connect_city(&city);
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::ConnectCityResult(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectCityFailed(e.to_string())),
                }
            }
            Job::ConfigSet {
                vpn_state,
                key,
                value,
                notifier,
            } => {
                let result = vpn_state.set_config(&key, &value);
                match result {
                    Ok(msg) => notifier.notify(AsyncEvent::ConfigSetResult(msg)),
                    Err(e) => notifier.notify(AsyncEvent::ConfigSetFailed(e.to_string())),
                }
            }
            Job::ConnectFastest {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.connect_fastest();
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::Connected(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectFailed(e.to_string())),
                }
            }
            Job::ConnectP2P {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.connect_p2p();
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::Connected(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectFailed(e.to_string())),
                }
            }
            Job::ConnectTor {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.connect_tor();
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::Connected(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectFailed(e.to_string())),
                }
            }
            Job::ConnectSecureCore {
                vpn_state,
                notifier,
            } => {
                let result = vpn_state.connect_securecore();
                match result {
                    Ok(conn_result) => notifier.notify(AsyncEvent::Connected(conn_result)),
                    Err(e) => notifier.notify(AsyncEvent::ConnectFailed(e.to_string())),
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

    pub fn spawn_refresh_servers(&self, vpn_state: Arc<VpnClient>, notifier: Arc<AsyncNotifier>) {
        self.pool.submit(Job::RefreshServers {
            vpn_state,
            notifier,
        });
    }

    pub fn spawn_connect(
        &self,
        vpn_state: Arc<VpnClient>,
        server_id: String,
        notifier: Arc<AsyncNotifier>,
    ) {
        self.pool.submit(Job::Connect {
            vpn_state,
            server_id,
            notifier,
        });
    }

    pub fn spawn_disconnect(&self, vpn_state: Arc<VpnClient>, notifier: Arc<AsyncNotifier>) {
        self.pool.submit(Job::Disconnect {
            vpn_state,
            notifier,
        });
    }

    pub fn spawn_connect_random(&self, vpn_state: Arc<VpnClient>, notifier: Arc<AsyncNotifier>) {
        self.pool.submit(Job::ConnectRandom {
            vpn_state,
            notifier,
        });
    }

    pub fn spawn_cities(
        &self,
        vpn_state: Arc<VpnClient>,
        country_code: String,
        notifier: Arc<AsyncNotifier>,
    ) {
        self.pool.submit(Job::Cities {
            vpn_state,
            country_code,
            notifier,
        });
    }

    pub fn spawn_connect_city(
        &self,
        vpn_state: Arc<VpnClient>,
        city: String,
        notifier: Arc<AsyncNotifier>,
    ) {
        self.pool.submit(Job::ConnectCity {
            vpn_state,
            city,
            notifier,
        });
    }

    pub fn spawn_config_set(
        &self,
        vpn_state: Arc<VpnClient>,
        key: String,
        value: String,
        notifier: Arc<AsyncNotifier>,
    ) {
        self.pool.submit(Job::ConfigSet {
            vpn_state,
            key,
            value,
            notifier,
        });
    }

    pub fn spawn_connect_fastest(&self, vpn_state: Arc<VpnClient>, notifier: Arc<AsyncNotifier>) {
        self.pool.submit(Job::ConnectFastest {
            vpn_state,
            notifier,
        });
    }

    pub fn spawn_connect_p2p(&self, vpn_state: Arc<VpnClient>, notifier: Arc<AsyncNotifier>) {
        self.pool.submit(Job::ConnectP2P {
            vpn_state,
            notifier,
        });
    }

    pub fn spawn_connect_tor(&self, vpn_state: Arc<VpnClient>, notifier: Arc<AsyncNotifier>) {
        self.pool.submit(Job::ConnectTor {
            vpn_state,
            notifier,
        });
    }

    pub fn spawn_connect_securecore(
        &self,
        vpn_state: Arc<VpnClient>,
        notifier: Arc<AsyncNotifier>,
    ) {
        self.pool.submit(Job::ConnectSecureCore {
            vpn_state,
            notifier,
        });
    }
}

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new_with_workers(4)
    }
}
