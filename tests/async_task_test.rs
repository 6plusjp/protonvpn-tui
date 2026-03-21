//! Tests for async task management module.
//!
//! Tests AsyncTaskManager and ThreadPool.

use protonvpn_tui::vpn::{AsyncTaskManager, ThreadPool};

#[test]
fn test_async_task_manager_default_workers() {
    let manager = AsyncTaskManager::new();
    // Verify it can be created without panic
    drop(manager);
}

#[test]
fn test_async_task_manager_custom_workers() {
    let manager = AsyncTaskManager::new_with_workers(2);
    drop(manager);
}

#[test]
fn test_async_task_manager_zero_workers() {
    let manager = AsyncTaskManager::new_with_workers(0);
    drop(manager);
}

#[test]
fn test_async_task_manager_many_workers() {
    let manager = AsyncTaskManager::new_with_workers(16);
    drop(manager);
}

#[test]
fn test_async_task_manager_default() {
    let manager = AsyncTaskManager::default();
    drop(manager);
}

#[test]
fn test_async_task_manager_clone() {
    let manager1 = AsyncTaskManager::new();
    let manager2 = manager1.clone();
    // Both should be usable after clone
    drop(manager1);
    drop(manager2);
}

#[test]
fn test_thread_pool_single_worker() {
    let pool = ThreadPool::new(1);
    drop(pool);
}

#[test]
fn test_thread_pool_multiple_workers() {
    let pool = ThreadPool::new(4);
    drop(pool);
}

#[test]
fn test_thread_pool_shutdown() {
    let pool = ThreadPool::new(2);
    drop(pool);
    // Pool should shut down gracefully
}
