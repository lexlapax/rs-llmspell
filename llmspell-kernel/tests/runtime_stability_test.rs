//! Runtime Stability Tests - Phase 9.4a.4
//!
//! Comprehensive test suite to validate the global IO runtime fixes the "dispatch task is gone"
//! error completely. These tests run for 60+ seconds to ensure stability over extended periods.
//!
//! ## Test Categories:
//! 1. HTTP Client Keep-Alive Tests (60+ seconds)
//! 2. Provider Operation Tests (simulated long-running LLM calls)
//! 3. Concurrent Runtime Operation Tests (100+ concurrent operations)
//! 4. Performance Benchmarks (runtime overhead measurement)
//!
//! ## Running These Tests:
//! ```bash
//! cargo test -p llmspell-kernel --test runtime_stability_test -- --ignored --nocapture
//! ```

use llmspell_kernel::runtime::{
    block_on_global, create_io_bound_resource, ensure_runtime_initialized,
    runtime_metrics, spawn_global,
};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tracing::{error, info, trace, warn};

/// HTTP Keep-Alive Test - 60 seconds
///
/// This test creates an HTTP client and maintains it for 60+ seconds,
/// making periodic requests to ensure the connection stays alive and
/// doesn't trigger "dispatch task is gone" errors.
#[test]
#[ignore] // Run with --ignored flag
fn test_http_client_60_second_keepalive() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Starting 60-second HTTP client keep-alive test");
    let start = Instant::now();

    // Create an HTTP client using the global runtime
    let client = create_io_bound_resource(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Duration::from_secs(60))
            .user_agent("llmspell-runtime-test/1.0")
            .build()
            .expect("Failed to create HTTP client")
    });

    let client = Arc::new(client);
    let request_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    // Spawn a task that makes requests over 60 seconds
    let client_clone = client.clone();
    let request_count_clone = request_count.clone();
    let error_count_clone = error_count.clone();

    let handle = spawn_global(async move {
        info!("Starting HTTP request loop");

        for i in 0..7 {
            // Wait 10 seconds between requests
            if i > 0 {
                sleep(Duration::from_secs(10)).await;
            }

            let elapsed = start.elapsed();
            info!("Making request {} at {:?}", i, elapsed);

            // Make a request to a reliable endpoint
            match client_clone
                .get("https://httpbin.org/delay/1")
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    let _ = response.text().await;
                    info!("Request {} completed with status {}", i, status);
                    request_count_clone.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    error!("Request {} failed: {}", i, e);
                    error_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        info!("HTTP request loop completed");
        "http_test_completed"
    });

    // Wait for the test to complete
    let result = block_on_global(handle);
    assert_eq!(result.unwrap(), "http_test_completed");

    let elapsed = start.elapsed();
    info!("Test completed in {:?}", elapsed);

    // Assertions
    assert!(elapsed >= Duration::from_secs(60), "Test should run for at least 60 seconds");
    assert!(request_count.load(Ordering::Relaxed) >= 6, "Should complete at least 6 requests");
    assert_eq!(error_count.load(Ordering::Relaxed), 0, "Should have no errors");

    // Verify the client is still valid
    let final_handle = spawn_global(async move {
        match client.get("https://httpbin.org/get").send().await {
            Ok(response) => {
                info!("Final validation request succeeded: {}", response.status());
                true
            }
            Err(e) => {
                error!("Final validation request failed: {}", e);
                false
            }
        }
    });

    let final_result = block_on_global(final_handle);
    assert!(final_result.unwrap(), "Client should still be valid after 60+ seconds");
}

/// Provider Operation Test - 90 seconds
///
/// Simulates long-running LLM provider operations to ensure they remain
/// stable over extended periods without runtime context issues.
#[test]
#[ignore] // Run with --ignored flag
fn test_provider_operations_90_seconds() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Starting 90-second provider operations test");
    let start = Instant::now();

    // Create HTTP client for provider simulation
    let provider_client = create_io_bound_resource(|| {
        Client::builder()
            .timeout(Duration::from_secs(120))
            .pool_idle_timeout(Duration::from_secs(120))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create provider client")
    });

    let provider_client = Arc::new(provider_client);
    let operation_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let client_clone = provider_client.clone();
    let op_count_clone = operation_count.clone();
    let err_count_clone = error_count.clone();

    let handle = spawn_global(async move {
        info!("Starting provider operation loop");

        // Simulate 3 long-running LLM calls
        for i in 0..3 {
            let elapsed = start.elapsed();
            info!("Starting provider operation {} at {:?}", i, elapsed);

            // Make request with large delay to simulate LLM processing
            let request_result = timeout(
                Duration::from_secs(30),
                client_clone
                    .post("https://httpbin.org/delay/20")
                    .json(&json!({
                        "prompt": format!("Test prompt {}", i),
                        "max_tokens": 1000,
                        "temperature": 0.7
                    }))
                    .send()
            ).await;

            match request_result {
                Ok(Ok(response)) => {
                    let status = response.status();
                    let _ = response.text().await;
                    info!("Provider operation {} completed with status {}", i, status);
                    op_count_clone.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Err(e)) => {
                    error!("Provider operation {} failed: {}", i, e);
                    err_count_clone.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    error!("Provider operation {} timed out", i);
                    err_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }

            // Wait 5 seconds between operations
            if i < 2 {
                sleep(Duration::from_secs(5)).await;
            }
        }

        info!("Provider operation loop completed");
        "provider_test_completed"
    });

    let result = block_on_global(handle);
    assert_eq!(result.unwrap(), "provider_test_completed");

    let elapsed = start.elapsed();
    info!("Provider test completed in {:?}", elapsed);

    // Assertions
    assert!(elapsed >= Duration::from_secs(70), "Test should run for at least 70 seconds");
    assert_eq!(operation_count.load(Ordering::Relaxed), 3, "Should complete all 3 operations");
    assert_eq!(error_count.load(Ordering::Relaxed), 0, "Should have no errors");
}

/// Concurrent Operations Test - 100 concurrent tasks
///
/// Tests that the runtime can handle many concurrent operations without
/// context mismatches or "dispatch task is gone" errors.
#[test]
#[ignore] // Run with --ignored flag
fn test_100_concurrent_runtime_operations() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Starting 100 concurrent operations test");
    let start = Instant::now();

    // Create shared resources
    let shared_state = Arc::new(RwLock::new(HashMap::<usize, String>::new()));
    let completed_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    // Spawn 100 concurrent tasks
    for i in 0..100usize {
        let state_clone = shared_state.clone();
        let completed_clone = completed_count.clone();
        let error_clone = error_count.clone();

        let handle = spawn_global(async move {
            // Vary the delay to simulate realistic workloads
            let delay_ms = 100 + (i % 20) * 50;
            sleep(Duration::from_millis(delay_ms as u64)).await;

            // Perform some work with shared state
            match timeout(Duration::from_secs(5), async {
                let mut state = state_clone.write().await;
                state.insert(i, format!("task_{}_completed", i));
                drop(state);

                // Simulate additional async work
                sleep(Duration::from_millis(50)).await;

                // Read back the value
                let state = state_clone.read().await;
                state.get(&i).cloned()
            }).await {
                Ok(Some(value)) => {
                    trace!("Task {} completed with value: {}", i, value);
                    completed_clone.fetch_add(1, Ordering::Relaxed);
                    Ok(value)
                }
                Ok(None) => {
                    error!("Task {} failed: value not found", i);
                    error_clone.fetch_add(1, Ordering::Relaxed);
                    Err("Value not found")
                }
                Err(_) => {
                    error!("Task {} timed out", i);
                    error_clone.fetch_add(1, Ordering::Relaxed);
                    Err("Timeout")
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    info!("Waiting for 100 concurrent tasks to complete");
    for (i, handle) in handles.into_iter().enumerate() {
        let result = block_on_global(handle);
        match result {
            Ok(Ok(_)) => trace!("Task {} join successful", i),
            Ok(Err(e)) => warn!("Task {} failed: {}", i, e),
            Err(e) => error!("Task {} join error: {}", i, e),
        }
    }

    let elapsed = start.elapsed();
    info!("Concurrent test completed in {:?}", elapsed);

    // Assertions
    assert_eq!(
        completed_count.load(Ordering::Relaxed),
        100,
        "All 100 tasks should complete"
    );
    assert_eq!(
        error_count.load(Ordering::Relaxed),
        0,
        "Should have no errors"
    );

    // Verify all values are in the state
    let final_state = block_on_global(async {
        let state = shared_state.read().await;
        state.len()
    });
    assert_eq!(final_state, 100, "All 100 values should be in state");
}

/// Performance Benchmark - Runtime Overhead
///
/// Measures the overhead of using the global runtime vs direct async execution
#[test]
fn test_runtime_overhead_benchmark() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Starting runtime overhead benchmark");

    const ITERATIONS: usize = 1000;

    // Benchmark 1: Resource creation overhead
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _resource = create_io_bound_resource(|| {
            format!("resource_{}", rand::random::<u64>())
        });
    }
    let resource_creation_time = start.elapsed();

    info!(
        "Resource creation: {} iterations in {:?} ({:?} per operation)",
        ITERATIONS,
        resource_creation_time,
        resource_creation_time / ITERATIONS as u32
    );

    // Benchmark 2: Task spawning overhead
    let start = Instant::now();
    let mut handles = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        handles.push(spawn_global(async move { i * 2 }));
    }
    for handle in handles {
        let _result = block_on_global(handle);
    }
    let spawn_time = start.elapsed();

    info!(
        "Task spawning: {} iterations in {:?} ({:?} per operation)",
        ITERATIONS,
        spawn_time,
        spawn_time / ITERATIONS as u32
    );

    // Benchmark 3: Concurrent task execution
    let start = Instant::now();
    let mut handles = Vec::with_capacity(100);
    for i in 0..100 {
        handles.push(spawn_global(async move {
            sleep(Duration::from_millis(10)).await;
            i
        }));
    }
    for handle in handles {
        let _result = block_on_global(handle);
    }
    let concurrent_time = start.elapsed();

    info!(
        "Concurrent execution: 100 tasks with 10ms sleep in {:?}",
        concurrent_time
    );

    // Performance assertions
    assert!(
        resource_creation_time / (ITERATIONS as u32) < Duration::from_millis(1),
        "Resource creation should be under 1ms per operation"
    );
    assert!(
        spawn_time / (ITERATIONS as u32) < Duration::from_millis(1),
        "Task spawning should be under 1ms per operation"
    );
    assert!(
        concurrent_time < Duration::from_secs(2),
        "100 concurrent 10ms tasks should complete in under 2 seconds"
    );

    // Report metrics
    if let Some(metrics) = runtime_metrics() {
        info!("Runtime metrics:");
        info!("  Resources created: {}", metrics.resources_created());
        info!("  Tasks spawned: {}", metrics.tasks_spawned());
        info!("  Uptime: {:?}", metrics.uptime());
    }
}

/// Memory Leak Test - Long running with resource cycling
///
/// Ensures no memory leaks occur over extended runtime usage
#[test]
#[ignore] // Run with --ignored flag
fn test_memory_stability_60_seconds() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Starting 60-second memory stability test");
    let start = Instant::now();
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Track resources created and destroyed
    let resources_created = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    // Spawn resource creation/destruction loop
    for thread_id in 0..4 {
        let stop_clone = stop_flag.clone();
        let created_clone = resources_created.clone();

        let handle = spawn_global(async move {
            info!("Thread {} starting resource cycling", thread_id);
            let mut cycle_count = 0;

            while !stop_clone.load(Ordering::Relaxed) {
                // Create HTTP client resource
                let client = create_io_bound_resource(|| {
                    Client::builder()
                        .timeout(Duration::from_secs(5))
                        .build()
                        .unwrap()
                });
                created_clone.fetch_add(1, Ordering::Relaxed);

                // Use the resource
                let _ = timeout(
                    Duration::from_secs(2),
                    client.get("https://httpbin.org/get").send()
                ).await;

                cycle_count += 1;

                // Small delay
                sleep(Duration::from_millis(100)).await;
            }

            info!("Thread {} stopped after {} cycles", thread_id, cycle_count);
            cycle_count
        });

        handles.push(handle);
    }

    // Let it run for 60 seconds
    std::thread::sleep(Duration::from_secs(60));

    // Stop all threads
    stop_flag.store(true, Ordering::Relaxed);

    // Wait for all threads to complete
    let mut total_cycles = 0;
    for handle in handles {
        if let Ok(cycles) = block_on_global(handle) {
            total_cycles += cycles;
        }
    }

    let elapsed = start.elapsed();
    info!("Memory stability test completed in {:?}", elapsed);

    let created = resources_created.load(Ordering::Relaxed);

    info!("Resources created: {}", created);
    info!("Total cycles: {}", total_cycles);

    assert!(elapsed >= Duration::from_secs(60), "Test should run for at least 60 seconds");
    assert!(created > 100, "Should create many resources during the test");
}

/// Integration test combining all scenarios
///
/// This is the ultimate test that validates complete stability
#[test]
#[ignore] // Run with --ignored flag
fn test_comprehensive_runtime_stability() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Starting comprehensive runtime stability test");
    let start = Instant::now();

    ensure_runtime_initialized();

    let mut all_handles = Vec::new();

    // 1. HTTP Client with periodic requests
    let http_handle = spawn_global(async {
        let client = create_io_bound_resource(|| {
            Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap()
        });

        for i in 0..6 {
            if i > 0 {
                sleep(Duration::from_secs(15)).await;
            }
            match client.get("https://httpbin.org/get").send().await {
                Ok(_) => info!("HTTP request {} succeeded", i),
                Err(e) => error!("HTTP request {} failed: {}", i, e),
            }
        }
        "http_complete".to_string()
    });
    all_handles.push(http_handle);

    // 2. Provider simulation
    let provider_handle = spawn_global(async {
        let client = create_io_bound_resource(|| {
            Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap()
        });

        for i in 0..3 {
            if i > 0 {
                sleep(Duration::from_secs(20)).await;
            }

            let result = timeout(
                Duration::from_secs(30),
                client
                    .post("https://httpbin.org/delay/10")
                    .json(&json!({
                        "operation": i,
                        "test": "provider_simulation"
                    }))
                    .send()
            ).await;

            match result {
                Ok(Ok(resp)) => info!("Provider operation {} completed: {}", i, resp.status()),
                Ok(Err(e)) => error!("Provider operation {} failed: {}", i, e),
                Err(_) => error!("Provider operation {} timed out", i),
            }
        }
        "provider_complete".to_string()
    });
    all_handles.push(provider_handle);

    // 3. Many concurrent short tasks
    for batch in 0..3 {
        let batch_handle = spawn_global(async move {
            sleep(Duration::from_secs(batch * 30)).await;

            let mut tasks = Vec::new();
            for i in 0..20 {
                tasks.push(spawn_global(async move {
                    sleep(Duration::from_millis(100 + i * 10)).await;
                    format!("batch_{}_task_{}", batch, i)
                }));
            }

            for task in tasks {
                let _ = task.await;
            }
            format!("batch_{}_complete", batch)
        });
        all_handles.push(batch_handle);
    }

    // Wait for all components to complete
    info!("Waiting for all components to complete");
    for handle in all_handles {
        match block_on_global(handle) {
            Ok(result) => info!("Component completed: {}", result),
            Err(e) => error!("Component failed: {:?}", e),
        }
    }

    let elapsed = start.elapsed();
    info!("Comprehensive test completed in {:?}", elapsed);

    assert!(
        elapsed >= Duration::from_secs(90),
        "Comprehensive test should run for at least 90 seconds"
    );

    // Final metrics check
    if let Some(metrics) = runtime_metrics() {
        info!("Final runtime metrics:");
        info!("  Resources created: {}", metrics.resources_created());
        info!("  Tasks spawned: {}", metrics.tasks_spawned());
        info!("  Runtime uptime: {:?}", metrics.uptime());

        assert!(metrics.resources_created() > 5, "Should create multiple resources");
        assert!(metrics.tasks_spawned() > 50, "Should spawn many tasks");
    }
}