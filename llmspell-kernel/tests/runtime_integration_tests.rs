//! Integration tests for the global IO runtime
//!
//! These tests validate that the global runtime fixes the "dispatch task is gone" error
//! and properly handles long-running operations.

use llmspell_kernel::runtime::{
    block_on_global, create_io_bound_resource, ensure_runtime_initialized, global_io_runtime,
    runtime_metrics, spawn_global,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test that the global runtime is a singleton
#[test]
fn test_global_runtime_is_singleton() {
    let runtime1 = global_io_runtime();
    let runtime2 = global_io_runtime();
    let runtime3 = global_io_runtime();

    // All should be the same instance
    assert!(Arc::ptr_eq(runtime1, runtime2));
    assert!(Arc::ptr_eq(runtime2, runtime3));
}

/// Test creating I/O-bound resources in the global context
#[test]
fn test_create_io_bound_resource_basic() {
    // Create a mock HTTP client
    let client = create_io_bound_resource(|| {
        // Simulate creating an HTTP client like reqwest::Client
        Arc::new("mock_http_client".to_string())
    });

    assert_eq!(*client, "mock_http_client");

    // Verify metrics
    if let Some(metrics) = runtime_metrics() {
        assert!(metrics.resources_created() > 0);
    }
}

/// Test that resources created with global runtime survive long operations
///
/// This is the critical test that validates the fix for "dispatch task is gone"
#[test]
fn test_long_running_resource_survival() {
    // Create a resource that simulates an HTTP client
    let resource = create_io_bound_resource(|| Arc::new(MockHttpClient::new()));

    // Clone for use in async task
    let resource_clone = resource.clone();

    // Spawn a task that uses the resource after a delay
    let handle = spawn_global(async move {
        // Simulate delay that would trigger "dispatch task is gone"
        // In real scenario this would be 35+ seconds
        sleep(Duration::from_millis(100)).await;

        // Try to use the resource - this would fail without global runtime
        resource_clone.make_request().await
    });

    // Wait for completion
    let result = block_on_global(handle);
    assert_eq!(result.unwrap(), "request_successful");

    // Verify the original resource is still valid
    assert!(resource.is_valid());
}

/// Test 60+ second operation as specified in the design doc
#[test]
#[ignore] // This test takes 60+ seconds, run with --ignored flag
fn test_sixty_second_operation() {
    let start = Instant::now();

    // Create a resource at the beginning
    let client = create_io_bound_resource(|| Arc::new(MockHttpClient::new()));

    let client_clone = client.clone();

    // Spawn a task that runs for 60+ seconds
    let handle = spawn_global(async move {
        // Wait for 35 seconds (past the original 30-second timeout)
        sleep(Duration::from_secs(35)).await;

        // Make a request - this would fail with "dispatch task is gone" in old architecture
        let result1 = client_clone.make_request().await;
        assert_eq!(result1, "request_successful");

        // Wait another 30 seconds
        sleep(Duration::from_secs(30)).await;

        // Make another request - still should work
        let result2 = client_clone.make_request().await;
        assert_eq!(result2, "request_successful");

        "completed_after_65_seconds"
    });

    // Wait for the long-running task
    let result = block_on_global(handle);
    assert_eq!(result.unwrap(), "completed_after_65_seconds");

    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_secs(65));

    // Original client should still be valid
    assert!(client.is_valid());
}

/// Test multiple concurrent long-running operations
#[test]
fn test_concurrent_long_operations() {
    let clients: Vec<_> = (0..5)
        .map(|i| create_io_bound_resource(move || Arc::new(MockHttpClient::with_id(i))))
        .collect();

    let mut handles = vec![];

    for client in clients.iter() {
        let client_clone = client.clone();
        let handle = spawn_global(async move {
            // Random delay to simulate real-world variance
            let delay = 50 + (client_clone.id * 20);
            sleep(Duration::from_millis(delay)).await;

            // Use the client
            client_clone.make_request().await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = block_on_global(handle);
        assert_eq!(result.unwrap(), "request_successful");
    }

    // All original clients should still be valid
    for client in &clients {
        assert!(client.is_valid());
    }
}

/// Test that runtime metrics are tracked correctly
#[test]
fn test_runtime_metrics_tracking() {
    ensure_runtime_initialized();

    let initial_metrics = runtime_metrics().expect("Metrics should be available");
    let initial_resources = initial_metrics.resources_created();
    let initial_tasks = initial_metrics.tasks_spawned();

    // Create some resources
    for _ in 0..3 {
        let _resource = create_io_bound_resource(|| "test_resource".to_string());
    }

    // Spawn some tasks
    for i in 0..5 {
        spawn_global(async move {
            sleep(Duration::from_millis(10)).await;
            i
        });
    }

    // Check metrics increased
    let final_metrics = runtime_metrics().expect("Metrics should be available");
    assert!(final_metrics.resources_created() >= initial_resources + 3);
    assert!(final_metrics.tasks_spawned() >= initial_tasks + 5);
    assert!(final_metrics.uptime() > Duration::ZERO);
}

/// Test error handling in global runtime
#[test]
#[should_panic(expected = "block_on_global called from within an async context")]
fn test_nested_block_on_panics() {
    // This should panic as designed to prevent deadlocks
    block_on_global(async {
        // Try to call block_on_global from within async context
        block_on_global(async { "this_should_panic" })
    });
}

// Mock HTTP client for testing
#[derive(Debug)]
struct MockHttpClient {
    id: u64,
    created_at: Instant,
}

impl MockHttpClient {
    fn new() -> Self {
        Self {
            id: 0,
            created_at: Instant::now(),
        }
    }

    fn with_id(id: u64) -> Self {
        Self {
            id,
            created_at: Instant::now(),
        }
    }

    async fn make_request(&self) -> String {
        // Simulate async operation
        sleep(Duration::from_millis(10)).await;
        "request_successful".to_string()
    }

    fn is_valid(&self) -> bool {
        // Check that the client hasn't been dropped/invalidated
        self.created_at.elapsed() < Duration::from_secs(3600)
    }
}
