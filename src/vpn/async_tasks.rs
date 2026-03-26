//! Async task management using a custom thread pool for blocking I/O operations.
//!
//! Uses a fixed-size thread pool to avoid spawning a new thread for every operation.
//! This significantly reduces overhead for frequent VPN operations.

use crate::error::AppResult;
use crate::state::AsyncEvent;
use crate::state::AsyncNotifier;
use crate::vpn::VpnClient;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

fn execute_and_notify<T>(
    vpn_state: &VpnClient,
    notifier: &AsyncNotifier,
    operation: impl FnOnce(&VpnClient) -> AppResult<T>,
    success: impl Fn(T) -> AsyncEvent,
    failure: impl Fn(String) -> AsyncEvent,
) {
    match operation(vpn_state) {
        Ok(result) => notifier.notify(success(result)),
        Err(e) => notifier.notify(failure(e.to_string())),
    }
}

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
                let mut queue = job_queue.lock().expect("job_queue mutex poisoned");
                while queue.is_empty() {
                    if *shutdown.lock().expect("shutdown mutex poisoned") {
                        return;
                    }
                    queue = not_empty.wait(queue).expect("condvar wait failed");
                }
                if *shutdown.lock().expect("shutdown mutex poisoned") {
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
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.refresh_servers(),
                    AsyncEvent::ServersRefreshed,
                    AsyncEvent::ServersRefreshFailed,
                );
            }
            Job::Connect {
                vpn_state,
                server_id,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_country(&server_id),
                    AsyncEvent::Connected,
                    AsyncEvent::ConnectFailed,
                );
            }
            Job::Disconnect {
                vpn_state,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.disconnect(),
                    |_| AsyncEvent::Disconnected,
                    AsyncEvent::DisconnectFailed,
                );
            }
            Job::ConnectRandom {
                vpn_state,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_random(),
                    AsyncEvent::Connected,
                    AsyncEvent::ConnectFailed,
                );
            }
            Job::Cities {
                vpn_state,
                country_code,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.list_cities_with_features(&country_code),
                    |cities| AsyncEvent::CitiesLoaded(country_code.clone(), cities),
                    |e| AsyncEvent::CitiesLoadFailed(country_code.clone(), e),
                );
            }
            Job::ConnectCity {
                vpn_state,
                city,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_city(&city),
                    AsyncEvent::ConnectCityResult,
                    AsyncEvent::ConnectCityFailed,
                );
            }
            Job::ConfigSet {
                vpn_state,
                key,
                value,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.set_config(&key, &value),
                    AsyncEvent::ConfigSetResult,
                    AsyncEvent::ConfigSetFailed,
                );
            }
            Job::ConnectFastest {
                vpn_state,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_fastest(),
                    AsyncEvent::Connected,
                    AsyncEvent::ConnectFailed,
                );
            }
            Job::ConnectP2P {
                vpn_state,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_p2p(),
                    AsyncEvent::Connected,
                    AsyncEvent::ConnectFailed,
                );
            }
            Job::ConnectTor {
                vpn_state,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_tor(),
                    AsyncEvent::Connected,
                    AsyncEvent::ConnectFailed,
                );
            }
            Job::ConnectSecureCore {
                vpn_state,
                notifier,
            } => {
                execute_and_notify(
                    &vpn_state,
                    &notifier,
                    |v| v.connect_securecore(),
                    AsyncEvent::Connected,
                    AsyncEvent::ConnectFailed,
                );
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
