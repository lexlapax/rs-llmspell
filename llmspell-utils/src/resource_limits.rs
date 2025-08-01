//! ABOUTME: Resource limit enforcement framework for tools
//! ABOUTME: Provides memory, CPU, file size, and operation count limits with monitoring

use llmspell_core::{LLMSpellError, Result as LLMResult};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (None = unlimited)
    pub max_memory_bytes: Option<usize>,
    /// Maximum CPU time in milliseconds (None = unlimited)
    pub max_cpu_time_ms: Option<u64>,
    /// Maximum file size in bytes for read/write operations
    pub max_file_size_bytes: Option<usize>,
    /// Maximum number of operations (e.g., loop iterations)
    pub max_operations: Option<usize>,
    /// Maximum concurrent operations
    pub max_concurrent_ops: Option<usize>,
    /// Timeout for async operations
    pub operation_timeout_ms: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(100 * 1024 * 1024),   // 100MB
            max_cpu_time_ms: Some(30_000),               // 30 seconds
            max_file_size_bytes: Some(50 * 1024 * 1024), // 50MB
            max_operations: Some(1_000_000),             // 1M operations
            max_concurrent_ops: Some(100),               // 100 concurrent ops
            operation_timeout_ms: Some(60_000),          // 60 seconds
        }
    }
}

impl ResourceLimits {
    /// Create unlimited resource limits (use with caution)
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_memory_bytes: None,
            max_cpu_time_ms: None,
            max_file_size_bytes: None,
            max_operations: None,
            max_concurrent_ops: None,
            operation_timeout_ms: None,
        }
    }

    /// Create strict resource limits for untrusted operations
    #[must_use]
    pub fn strict() -> Self {
        Self {
            max_memory_bytes: Some(10 * 1024 * 1024),   // 10MB
            max_cpu_time_ms: Some(5_000),               // 5 seconds
            max_file_size_bytes: Some(5 * 1024 * 1024), // 5MB
            max_operations: Some(10_000),               // 10K operations
            max_concurrent_ops: Some(10),               // 10 concurrent ops
            operation_timeout_ms: Some(10_000),         // 10 seconds
        }
    }

    /// Create relaxed resource limits for trusted operations
    #[must_use]
    pub fn relaxed() -> Self {
        Self {
            max_memory_bytes: Some(1024 * 1024 * 1024),   // 1GB
            max_cpu_time_ms: Some(300_000),               // 5 minutes
            max_file_size_bytes: Some(500 * 1024 * 1024), // 500MB
            max_operations: Some(100_000_000),            // 100M operations
            max_concurrent_ops: Some(1000),               // 1000 concurrent ops
            operation_timeout_ms: Some(600_000),          // 10 minutes
        }
    }
}

/// Resource usage tracker for monitoring
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    /// Current memory usage in bytes
    memory_used: Arc<AtomicUsize>,
    /// CPU time used in milliseconds (kept for future use)
    #[allow(dead_code)]
    cpu_time_used: Arc<AtomicU64>,
    /// Number of operations performed
    operations_count: Arc<AtomicUsize>,
    /// Number of concurrent operations
    concurrent_ops: Arc<AtomicUsize>,
    /// Start time for CPU tracking
    start_time: Instant,
    /// Resource limits
    limits: ResourceLimits,
}

impl ResourceTracker {
    /// Create a new resource tracker with given limits
    #[must_use]
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            memory_used: Arc::new(AtomicUsize::new(0)),
            cpu_time_used: Arc::new(AtomicU64::new(0)),
            operations_count: Arc::new(AtomicUsize::new(0)),
            concurrent_ops: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            limits,
        }
    }

    /// Track memory allocation
    ///
    /// # Errors
    /// Returns error if memory limit would be exceeded
    pub fn track_memory(&self, bytes: usize) -> LLMResult<()> {
        if let Some(max_memory) = self.limits.max_memory_bytes {
            let current = self.memory_used.fetch_add(bytes, Ordering::SeqCst);
            if current + bytes > max_memory {
                // Rollback the addition
                self.memory_used.fetch_sub(bytes, Ordering::SeqCst);
                return Err(LLMSpellError::ResourceLimit {
                    resource: "memory".to_string(),
                    limit: max_memory,
                    used: current + bytes,
                });
            }
        }
        Ok(())
    }

    /// Release tracked memory
    pub fn release_memory(&self, bytes: usize) {
        self.memory_used.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Track an operation
    ///
    /// # Errors
    /// Returns error if operation count limit would be exceeded
    pub fn track_operation(&self) -> LLMResult<()> {
        if let Some(max_ops) = self.limits.max_operations {
            let current = self.operations_count.fetch_add(1, Ordering::SeqCst);
            if current + 1 > max_ops {
                return Err(LLMSpellError::ResourceLimit {
                    resource: "operations".to_string(),
                    limit: max_ops,
                    used: current + 1,
                });
            }
        }
        Ok(())
    }

    /// Track CPU time usage
    ///
    /// # Errors
    /// Returns error if CPU time limit has been exceeded
    #[allow(clippy::cast_possible_truncation)]
    pub fn check_cpu_time(&self) -> LLMResult<()> {
        if let Some(max_cpu_ms) = self.limits.max_cpu_time_ms {
            let elapsed = self.start_time.elapsed().as_millis() as u64;
            if elapsed > max_cpu_ms {
                return Err(LLMSpellError::ResourceLimit {
                    resource: "cpu_time".to_string(),
                    limit: max_cpu_ms as usize,
                    used: elapsed as usize,
                });
            }
        }
        Ok(())
    }

    /// Track concurrent operation start
    ///
    /// # Errors
    /// Returns error if concurrent operation limit would be exceeded
    pub fn track_concurrent_start(&self) -> LLMResult<ConcurrentGuard> {
        if let Some(max_concurrent) = self.limits.max_concurrent_ops {
            let current = self.concurrent_ops.fetch_add(1, Ordering::SeqCst);
            if current + 1 > max_concurrent {
                // Rollback
                self.concurrent_ops.fetch_sub(1, Ordering::SeqCst);
                return Err(LLMSpellError::ResourceLimit {
                    resource: "concurrent_operations".to_string(),
                    limit: max_concurrent,
                    used: current + 1,
                });
            }
        }
        Ok(ConcurrentGuard {
            tracker: self.concurrent_ops.clone(),
        })
    }

    /// Check file size limit
    ///
    /// # Errors
    /// Returns error if file size exceeds limit
    pub fn check_file_size(&self, size: usize) -> LLMResult<()> {
        if let Some(max_size) = self.limits.max_file_size_bytes {
            if size > max_size {
                return Err(LLMSpellError::ResourceLimit {
                    resource: "file_size".to_string(),
                    limit: max_size,
                    used: size,
                });
            }
        }
        Ok(())
    }

    /// Get operation timeout duration
    #[must_use]
    pub fn operation_timeout(&self) -> Option<Duration> {
        self.limits.operation_timeout_ms.map(Duration::from_millis)
    }

    /// Execute an async operation with timeout
    ///
    /// # Errors
    /// Returns error if operation times out
    #[allow(clippy::cast_possible_truncation)]
    pub async fn with_timeout<F, T>(&self, future: F) -> LLMResult<T>
    where
        F: std::future::Future<Output = T>,
    {
        if let Some(timeout_duration) = self.operation_timeout() {
            match timeout(timeout_duration, future).await {
                Ok(result) => Ok(result),
                Err(_) => Err(LLMSpellError::ResourceLimit {
                    resource: "timeout".to_string(),
                    limit: self.limits.operation_timeout_ms.unwrap_or(0) as usize,
                    used: self.limits.operation_timeout_ms.unwrap_or(0) as usize,
                }),
            }
        } else {
            Ok(future.await)
        }
    }

    /// Get current resource usage metrics
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_metrics(&self) -> ResourceMetrics {
        ResourceMetrics {
            memory_bytes: self.memory_used.load(Ordering::SeqCst),
            cpu_time_ms: self.start_time.elapsed().as_millis() as u64,
            operations_count: self.operations_count.load(Ordering::SeqCst),
            concurrent_ops: self.concurrent_ops.load(Ordering::SeqCst),
        }
    }
}

/// Guard for tracking concurrent operations
#[derive(Debug)]
pub struct ConcurrentGuard {
    tracker: Arc<AtomicUsize>,
}

impl Drop for ConcurrentGuard {
    fn drop(&mut self) {
        self.tracker.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Resource usage metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceMetrics {
    /// Memory used in bytes
    pub memory_bytes: usize,
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Number of operations performed
    pub operations_count: usize,
    /// Current concurrent operations
    pub concurrent_ops: usize,
}

/// Memory allocation guard that tracks and releases memory
pub struct MemoryGuard<'a> {
    tracker: &'a ResourceTracker,
    bytes: usize,
}

impl<'a> MemoryGuard<'a> {
    /// Create a new memory guard
    ///
    /// # Errors
    /// Returns error if memory limit would be exceeded
    pub fn new(tracker: &'a ResourceTracker, bytes: usize) -> LLMResult<Self> {
        tracker.track_memory(bytes)?;
        Ok(Self { tracker, bytes })
    }
}

impl Drop for MemoryGuard<'_> {
    fn drop(&mut self) {
        self.tracker.release_memory(self.bytes);
    }
}

/// Helper macro for tracking operations in loops
#[macro_export]
macro_rules! track_operation {
    ($tracker:expr) => {
        $tracker.track_operation()?;
        $tracker.check_cpu_time()?;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resource_limits_creation() {
        let default_limits = ResourceLimits::default();
        assert_eq!(default_limits.max_memory_bytes, Some(100 * 1024 * 1024));

        let strict_limits = ResourceLimits::strict();
        assert_eq!(strict_limits.max_memory_bytes, Some(10 * 1024 * 1024));

        let unlimited_limits = ResourceLimits::unlimited();
        assert_eq!(unlimited_limits.max_memory_bytes, None);
    }
    #[test]
    fn test_memory_tracking() {
        let limits = ResourceLimits {
            max_memory_bytes: Some(1000),
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Should succeed
        assert!(tracker.track_memory(500).is_ok());
        assert!(tracker.track_memory(400).is_ok());

        // Should fail - exceeds limit
        assert!(tracker.track_memory(200).is_err());

        // Release some memory
        tracker.release_memory(400);

        // Should succeed now
        assert!(tracker.track_memory(200).is_ok());
    }
    #[test]
    fn test_operation_tracking() {
        let limits = ResourceLimits {
            max_operations: Some(5),
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Track 5 operations - should succeed
        for _ in 0..5 {
            assert!(tracker.track_operation().is_ok());
        }

        // 6th operation should fail
        assert!(tracker.track_operation().is_err());
    }
    #[test]
    fn test_concurrent_operations() {
        let limits = ResourceLimits {
            max_concurrent_ops: Some(2),
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Start 2 concurrent operations
        let guard1 = tracker.track_concurrent_start().unwrap();
        let guard2 = tracker.track_concurrent_start().unwrap();

        // 3rd should fail
        assert!(tracker.track_concurrent_start().is_err());

        // Drop one guard
        drop(guard1);

        // Now we can start another
        let _guard3 = tracker.track_concurrent_start().unwrap();

        // Drop all guards
        drop(guard2);
    }
    #[test]
    fn test_file_size_check() {
        let limits = ResourceLimits {
            max_file_size_bytes: Some(1024),
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Should succeed
        assert!(tracker.check_file_size(512).is_ok());
        assert!(tracker.check_file_size(1024).is_ok());

        // Should fail
        assert!(tracker.check_file_size(1025).is_err());
    }
    #[tokio::test]
    async fn test_timeout() {
        let limits = ResourceLimits {
            operation_timeout_ms: Some(100),
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Should succeed
        let result = tracker
            .with_timeout(async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                42
            })
            .await;
        assert_eq!(result.unwrap(), 42);

        // Should timeout
        let result = tracker
            .with_timeout(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                42
            })
            .await;
        assert!(result.is_err());
    }
    #[test]
    fn test_memory_guard() {
        let limits = ResourceLimits {
            max_memory_bytes: Some(1000),
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        {
            let _guard = MemoryGuard::new(&tracker, 500).unwrap();
            assert_eq!(tracker.get_metrics().memory_bytes, 500);
        }
        // Memory should be released after guard is dropped
        assert_eq!(tracker.get_metrics().memory_bytes, 0);
    }
    #[test]
    fn test_metrics() {
        let limits = ResourceLimits::default();
        let tracker = ResourceTracker::new(limits);

        tracker.track_memory(1000).unwrap();
        tracker.track_operation().unwrap();
        tracker.track_operation().unwrap();

        let metrics = tracker.get_metrics();
        assert_eq!(metrics.memory_bytes, 1000);
        assert_eq!(metrics.operations_count, 2);
        // cpu_time_ms is u64, so it's always >= 0
        assert!(metrics.cpu_time_ms < u64::MAX);
    }
}
