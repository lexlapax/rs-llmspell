//! ABOUTME: Integration tests for resource limit enforcement
//! ABOUTME: Tests memory, CPU, file size, and operation count limits

use llmspell_utils::resource_limits::{MemoryGuard, ResourceLimits, ResourceTracker};
use std::time::Duration;
use tokio::time::sleep;
#[test]
fn test_memory_limit_enforcement() {
    let limits = ResourceLimits {
        max_memory_bytes: Some(1_000_000), // 1MB
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Allocate within limit
    assert!(tracker.track_memory(500_000).is_ok());
    assert!(tracker.track_memory(400_000).is_ok());

    // Try to exceed limit
    let result = tracker.track_memory(200_000);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("memory"));
        assert!(e.to_string().contains("1000000"));
    }

    // Release some memory and try again
    tracker.release_memory(400_000);
    assert!(tracker.track_memory(200_000).is_ok());
}
#[test]
fn test_operation_count_limits() {
    let limits = ResourceLimits {
        max_operations: Some(100),
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Track operations up to limit
    for i in 0..100 {
        assert!(
            tracker.track_operation().is_ok(),
            "Operation {} should succeed",
            i
        );
    }

    // 101st operation should fail
    let result = tracker.track_operation();
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("operations"));
        assert!(e.to_string().contains("100"));
    }
}
#[test]
fn test_file_size_limits() {
    let limits = ResourceLimits {
        max_file_size_bytes: Some(5_000_000), // 5MB
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Check files within limit
    assert!(tracker.check_file_size(1_000_000).is_ok());
    assert!(tracker.check_file_size(4_999_999).is_ok());
    assert!(tracker.check_file_size(5_000_000).is_ok());

    // Check files exceeding limit
    let result = tracker.check_file_size(5_000_001);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("file_size"));
        assert!(e.to_string().contains("5000000"));
    }
}
#[test]
fn test_cpu_time_tracking() {
    let limits = ResourceLimits {
        max_cpu_time_ms: Some(100), // 100ms
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Initial check should pass
    assert!(tracker.check_cpu_time().is_ok());

    // Sleep for a bit to consume CPU time
    std::thread::sleep(Duration::from_millis(150));

    // Now check should fail
    let result = tracker.check_cpu_time();
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("cpu_time"));
    }
}
#[test]
fn test_concurrent_operations_limit() {
    let limits = ResourceLimits {
        max_concurrent_ops: Some(3),
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Start concurrent operations
    let guard1 = tracker.track_concurrent_start().unwrap();
    let guard2 = tracker.track_concurrent_start().unwrap();
    let guard3 = tracker.track_concurrent_start().unwrap();

    // 4th should fail
    let result = tracker.track_concurrent_start();
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("concurrent_operations"));
        assert!(e.to_string().contains("3"));
    }

    // Drop one guard and try again
    drop(guard2);
    let _guard4 = tracker.track_concurrent_start().unwrap();

    // Clean up
    drop(guard1);
    drop(guard3);
}
#[tokio::test]
async fn test_operation_timeout() {
    let limits = ResourceLimits {
        operation_timeout_ms: Some(200), // 200ms
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Fast operation should succeed
    let result = tracker
        .with_timeout(async {
            sleep(Duration::from_millis(100)).await;
            "success"
        })
        .await;
    assert_eq!(result.unwrap(), "success");

    // Slow operation should timeout
    let result = tracker
        .with_timeout(async {
            sleep(Duration::from_millis(300)).await;
            "should not reach"
        })
        .await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("timeout"));
        assert!(e.to_string().contains("200"));
    }
}
#[test]
fn test_memory_guard_lifecycle() {
    let limits = ResourceLimits {
        max_memory_bytes: Some(1_000_000), // 1MB
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Check initial metrics
    assert_eq!(tracker.get_metrics().memory_bytes, 0);

    {
        // Create memory guard
        let _guard = MemoryGuard::new(&tracker, 500_000).unwrap();

        // Memory should be tracked
        assert_eq!(tracker.get_metrics().memory_bytes, 500_000);

        // Try to allocate more than remaining
        let result = MemoryGuard::new(&tracker, 600_000);
        assert!(result.is_err());
    }

    // Memory should be released after guard is dropped
    assert_eq!(tracker.get_metrics().memory_bytes, 0);

    // Should be able to allocate again
    let _guard = MemoryGuard::new(&tracker, 900_000).unwrap();
}
#[test]
fn test_resource_metrics() {
    let limits = ResourceLimits::default();
    let tracker = ResourceTracker::new(limits);

    // Track various operations
    tracker.track_memory(100_000).unwrap();
    tracker.track_operation().unwrap();
    tracker.track_operation().unwrap();
    tracker.track_operation().unwrap();

    let metrics = tracker.get_metrics();
    assert_eq!(metrics.memory_bytes, 100_000);
    assert_eq!(metrics.operations_count, 3);
    // CPU time is always non-negative (u64)
    assert_eq!(metrics.concurrent_ops, 0);

    // Start concurrent operation
    let _guard = tracker.track_concurrent_start().unwrap();
    assert_eq!(tracker.get_metrics().concurrent_ops, 1);
}
#[test]
fn test_limit_configurations() {
    // Test default limits
    let default_limits = ResourceLimits::default();
    assert_eq!(default_limits.max_memory_bytes, Some(100 * 1024 * 1024));
    assert_eq!(default_limits.max_cpu_time_ms, Some(30_000));
    assert_eq!(default_limits.max_file_size_bytes, Some(50 * 1024 * 1024));
    assert_eq!(default_limits.max_operations, Some(1_000_000));

    // Test strict limits
    let strict_limits = ResourceLimits::strict();
    assert_eq!(strict_limits.max_memory_bytes, Some(10 * 1024 * 1024));
    assert_eq!(strict_limits.max_cpu_time_ms, Some(5_000));
    assert_eq!(strict_limits.max_file_size_bytes, Some(5 * 1024 * 1024));
    assert_eq!(strict_limits.max_operations, Some(10_000));

    // Test relaxed limits
    let relaxed_limits = ResourceLimits::relaxed();
    assert_eq!(relaxed_limits.max_memory_bytes, Some(1024 * 1024 * 1024));
    assert_eq!(relaxed_limits.max_cpu_time_ms, Some(300_000));
    assert_eq!(relaxed_limits.max_file_size_bytes, Some(500 * 1024 * 1024));
    assert_eq!(relaxed_limits.max_operations, Some(100_000_000));

    // Test unlimited
    let unlimited = ResourceLimits::unlimited();
    assert_eq!(unlimited.max_memory_bytes, None);
    assert_eq!(unlimited.max_cpu_time_ms, None);
    assert_eq!(unlimited.max_file_size_bytes, None);
    assert_eq!(unlimited.max_operations, None);
}
#[tokio::test]
async fn test_complex_resource_scenario() {
    let limits = ResourceLimits {
        max_memory_bytes: Some(2_000_000), // 2MB
        max_operations: Some(10),          // 10 operations
        max_concurrent_ops: Some(2),       // 2 concurrent
        operation_timeout_ms: Some(500),   // 500ms
        ..Default::default()
    };
    let tracker = ResourceTracker::new(limits);

    // Start concurrent operations
    let guard1 = tracker.track_concurrent_start().unwrap();
    let _guard2 = tracker.track_concurrent_start().unwrap();

    // Allocate memory
    let _mem_guard = MemoryGuard::new(&tracker, 1_000_000).unwrap();

    // Track operations
    for _ in 0..5 {
        tracker.track_operation().unwrap();
    }

    // Execute with timeout
    let result = tracker
        .with_timeout(async {
            sleep(Duration::from_millis(100)).await;
            tracker.track_operation().unwrap();
            "completed"
        })
        .await;
    assert_eq!(result.unwrap(), "completed");

    // Check final metrics
    let metrics = tracker.get_metrics();
    assert_eq!(metrics.memory_bytes, 1_000_000);
    assert_eq!(metrics.operations_count, 6);
    assert_eq!(metrics.concurrent_ops, 2);

    // Clean up
    drop(guard1);
}
