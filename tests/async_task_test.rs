//! Tests for async task management module.
//!
//! Tests AsyncTaskManager, ThreadPool, and channel creation.

use protonvpn_tui::vpn::{create_channel, AsyncTaskManager, ThreadPool};

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
fn test_create_channel_sender_receiver() {
    let (sender, receiver) = create_channel::<String>();
    // Verify channel components exist
    drop(sender);
    drop(receiver);
}

#[test]
fn test_create_channel_send_receive() {
    let (sender, receiver) = create_channel::<i32>();
    sender.send(42).unwrap();
    let received = receiver.recv().unwrap();
    assert_eq!(received, 42);
}

#[test]
fn test_create_channel_multiple_values() {
    let (sender, receiver) = create_channel::<String>();

    sender.send("first".to_string()).unwrap();
    sender.send("second".to_string()).unwrap();
    sender.send("third".to_string()).unwrap();
    drop(sender); // Close sender

    let mut results = Vec::new();
    while let Ok(msg) = receiver.recv() {
        results.push(msg);
    }

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], "first");
    assert_eq!(results[1], "second");
    assert_eq!(results[2], "third");
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
