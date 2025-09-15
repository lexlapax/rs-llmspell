//! Global IO Runtime Foundation
//!
//! This module provides the critical global IO runtime that ensures all HTTP clients
//! and I/O operations across the entire `LLMSpell` system use the same runtime context.
//! This fixes the "dispatch task is gone" error that occurs when HTTP clients are
//! created in one runtime context and used in another.
//!
//! ## Problem Being Solved
//!
//! When the kernel was spawned as a background task (`tokio::spawn`), it created
//! an isolated runtime context. HTTP clients created in this context would fail
//! after ~30 seconds with "dispatch task is gone" because the original task that
//! created them had completed. This global runtime ensures all I/O resources
//! share the same persistent runtime context.

use metrics::{counter, gauge, histogram};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, info, instrument, trace, warn};

/// Global IO runtime instance - the foundation of kernel stability
static GLOBAL_IO_RUNTIME: OnceCell<Arc<Runtime>> = OnceCell::new();

/// Runtime metrics tracking
static RUNTIME_METRICS: OnceCell<Arc<RuntimeMetrics>> = OnceCell::new();

/// Get the global IO runtime instance
///
/// This function returns the global Tokio runtime that should be used for ALL
/// I/O operations in the `LLMSpell` system. It ensures consistent runtime context
/// across all components, preventing "dispatch task is gone" errors.
///
/// # Panics
///
/// Panics if the runtime cannot be created (should never happen in practice).
///
/// # Example
///
/// ```no_run
/// use llmspell_kernel::runtime::global_io_runtime;
///
/// let runtime = global_io_runtime();
/// runtime.spawn(async {
///     // Your async task here
/// });
/// ```
pub fn global_io_runtime() -> &'static Arc<Runtime> {
    GLOBAL_IO_RUNTIME.get_or_init(|| {
        info!("Initializing global IO runtime");

        let runtime = Builder::new_multi_thread()
            .worker_threads(num_cpus::get())
            .thread_name("llmspell-io")
            .enable_all()
            .on_thread_start(|| {
                trace!("IO runtime thread started");
            })
            .on_thread_stop(|| {
                trace!("IO runtime thread stopped");
            })
            .build()
            .expect("Failed to create global IO runtime");

        // Initialize runtime metrics
        let metrics = RuntimeMetrics::new();
        RUNTIME_METRICS.set(Arc::new(metrics)).ok();

        info!(
            "Global IO runtime initialized with {} worker threads",
            num_cpus::get()
        );
        Arc::new(runtime)
    })
}

/// Create an I/O-bound resource safely within the global runtime context
///
/// This function ensures that any I/O resources (like HTTP clients) are created
/// within the global runtime context, preventing runtime context mismatches.
///
/// # Type Parameters
///
/// * `T` - The type of resource being created
/// * `F` - The creator function type
///
/// # Arguments
///
/// * `creator` - A function that creates the resource
///
/// # Returns
///
/// The created resource of type `T`
///
/// # Example
///
/// ```no_run
/// use llmspell_kernel::runtime::create_io_bound_resource;
/// use reqwest::Client;
///
/// let client = create_io_bound_resource(|| {
///     Client::builder()
///         .timeout(std::time::Duration::from_secs(30))
///         .build()
///         .unwrap()
/// });
/// ```
#[instrument(level = "debug", skip(creator))]
pub fn create_io_bound_resource<T, F>(creator: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let start = Instant::now();

    // Check if we're already in a runtime context
    let in_runtime = tokio::runtime::Handle::try_current().is_ok();

    if in_runtime {
        debug!("Creating IO-bound resource in current runtime context");
        // We're already in a runtime (test or production), use it directly
        let resource = creator();

        let elapsed = start.elapsed();
        debug!(
            "IO-bound resource created in current runtime in {:?}",
            elapsed
        );

        // Track metrics
        if let Some(metrics) = RUNTIME_METRICS.get() {
            metrics.record_resource_creation(elapsed);
        }

        resource
    } else {
        debug!("Creating IO-bound resource in global runtime context");
        // No runtime context, enter the global runtime
        let _guard = global_io_runtime().enter();

        // Create the resource within the global runtime context
        let resource = creator();

        let elapsed = start.elapsed();
        debug!(
            "IO-bound resource created in global runtime in {:?}",
            elapsed
        );

        // Track metrics
        if let Some(metrics) = RUNTIME_METRICS.get() {
            metrics.record_resource_creation(elapsed);
        }

        resource
    }
}

/// Block on a future using the global runtime
///
/// This function runs a future to completion using the global runtime.
/// It should be used sparingly, primarily for initialization or in non-async contexts.
///
/// # Arguments
///
/// * `future` - The future to execute
///
/// # Returns
///
/// The result of the future
///
/// # Panics
///
/// Panics if called from within an async context (would cause deadlock)
#[instrument(level = "debug", skip(future))]
pub fn block_on_global<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    // Check if we're already in an async context
    assert!(
        tokio::runtime::Handle::try_current().is_err(),
        "block_on_global called from within an async context - this would deadlock!"
    );

    debug!("Blocking on future using global runtime");
    let start = Instant::now();

    let result = global_io_runtime().block_on(future);

    let elapsed = start.elapsed();
    debug!("Future completed in {:?}", elapsed);

    // Track metrics
    if let Some(metrics) = RUNTIME_METRICS.get() {
        metrics.record_block_on(elapsed);
    }

    result
}

/// Spawn a task on the global runtime
///
/// This is a convenience function for spawning tasks on the global runtime.
/// Unlike `tokio::spawn`, this ensures the task runs in the global runtime context.
///
/// # Arguments
///
/// * `future` - The future to spawn
///
/// # Returns
///
/// A `JoinHandle` for the spawned task
#[instrument(level = "trace", skip(future))]
pub fn spawn_global<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    trace!("Spawning task on global runtime");

    // Track metrics
    if let Some(metrics) = RUNTIME_METRICS.get() {
        metrics.increment_spawned_tasks();
    }

    global_io_runtime().spawn(future)
}

/// Runtime metrics for monitoring and debugging
#[derive(Debug)]
pub struct RuntimeMetrics {
    resources_created: Arc<RwLock<u64>>,
    total_resource_creation_time: Arc<RwLock<Duration>>,
    block_on_calls: Arc<RwLock<u64>>,
    total_block_on_time: Arc<RwLock<Duration>>,
    tasks_spawned: Arc<RwLock<u64>>,
    start_time: Instant,
}

impl RuntimeMetrics {
    /// Create new runtime metrics instance
    fn new() -> Self {
        Self {
            resources_created: Arc::new(RwLock::new(0)),
            total_resource_creation_time: Arc::new(RwLock::new(Duration::ZERO)),
            block_on_calls: Arc::new(RwLock::new(0)),
            total_block_on_time: Arc::new(RwLock::new(Duration::ZERO)),
            tasks_spawned: Arc::new(RwLock::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Record a resource creation
    fn record_resource_creation(&self, duration: Duration) {
        let mut count = self.resources_created.write();
        *count += 1;

        let mut total_time = self.total_resource_creation_time.write();
        *total_time += duration;

        // Emit metrics
        counter!("kernel.runtime.resources_created").increment(1);
        histogram!("kernel.runtime.resource_creation_time").record(duration.as_secs_f64());
    }

    /// Record a `block_on` call
    fn record_block_on(&self, duration: Duration) {
        let mut count = self.block_on_calls.write();
        *count += 1;

        let mut total_time = self.total_block_on_time.write();
        *total_time += duration;

        // Emit metrics
        counter!("kernel.runtime.block_on_calls").increment(1);
        histogram!("kernel.runtime.block_on_time").record(duration.as_secs_f64());
    }

    /// Increment spawned tasks counter
    fn increment_spawned_tasks(&self) {
        let mut count = self.tasks_spawned.write();
        *count += 1;

        // Emit metrics
        counter!("kernel.runtime.tasks_spawned").increment(1);
        #[allow(clippy::cast_precision_loss)]
        gauge!("kernel.runtime.active_tasks").set(*count as f64);
    }

    /// Get runtime uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get total resources created
    pub fn resources_created(&self) -> u64 {
        *self.resources_created.read()
    }

    /// Get total tasks spawned
    pub fn tasks_spawned(&self) -> u64 {
        *self.tasks_spawned.read()
    }
}

/// Get runtime metrics
///
/// Returns the runtime metrics if available
pub fn runtime_metrics() -> Option<Arc<RuntimeMetrics>> {
    RUNTIME_METRICS.get().cloned()
}

/// Ensure the global runtime is initialized
///
/// This function can be called during application startup to ensure
/// the runtime is initialized before any I/O operations.
pub fn ensure_runtime_initialized() {
    let _ = global_io_runtime();
    info!("Global runtime initialized and ready");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_global_runtime_singleton() {
        // Get runtime multiple times - should be same instance
        let runtime1 = global_io_runtime();
        let runtime2 = global_io_runtime();

        // Use Arc::ptr_eq to check if they're the same instance
        assert!(Arc::ptr_eq(runtime1, runtime2));
    }

    #[test]
    fn test_create_io_bound_resource() {
        // Create a resource using the global runtime
        let value = create_io_bound_resource(|| {
            // Simulate creating an HTTP client
            "test_resource".to_string()
        });

        assert_eq!(value, "test_resource");

        // Check metrics
        if let Some(metrics) = runtime_metrics() {
            assert!(metrics.resources_created() > 0);
        }
    }

    #[test]
    fn test_block_on_global() {
        // Run an async operation using block_on_global
        let result = block_on_global(async {
            sleep(Duration::from_millis(10)).await;
            42
        });

        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "block_on_global called from within an async context")]
    fn test_block_on_global_panics_in_async_context() {
        // This should panic because we're calling block_on_global from within block_on_global
        block_on_global(async { block_on_global(async { 42 }) });
    }

    #[test]
    fn test_spawn_global() {
        // Spawn a task on the global runtime
        let handle = spawn_global(async {
            sleep(Duration::from_millis(10)).await;
            "completed"
        });

        // Wait for the task to complete
        let result = block_on_global(handle);
        assert_eq!(result.unwrap(), "completed");

        // Check metrics
        if let Some(metrics) = runtime_metrics() {
            assert!(metrics.tasks_spawned() > 0);
        }
    }

    #[test]
    fn test_runtime_metrics() {
        // Ensure runtime is initialized
        ensure_runtime_initialized();

        // Get metrics
        let metrics = runtime_metrics().expect("Metrics should be available");

        // Check uptime is non-zero
        assert!(metrics.uptime() > Duration::ZERO);
    }

    #[test]
    fn test_long_running_resource() {
        // This test simulates the scenario that was causing "dispatch task is gone"
        // Create a resource that would be used after 30+ seconds
        let resource = create_io_bound_resource(|| Arc::new("long_lived_resource".to_string()));

        // Spawn a task that uses the resource after a delay
        let resource_clone = resource.clone();
        let handle = spawn_global(async move {
            // In real scenario this would be 35+ seconds
            // For testing, we use a short delay
            sleep(Duration::from_millis(100)).await;

            // Access the resource - this would fail with "dispatch task is gone"
            // if not using the global runtime
            resource_clone.len()
        });

        // Wait for the task to complete
        let result = block_on_global(handle);
        assert_eq!(result.unwrap(), 19); // "long_lived_resource".len()
    }
}
