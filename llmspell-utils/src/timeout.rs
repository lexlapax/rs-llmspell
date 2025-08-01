// ABOUTME: Timeout management utilities for async operations
// ABOUTME: Provides consistent timeout handling with cancellation support

use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use thiserror::Error;
use tokio::time::{error::Elapsed, timeout, Instant};
use tracing::{debug, warn};

/// Timeout error types
#[derive(Debug, Error)]
pub enum TimeoutError {
    #[error("Operation timed out after {duration:?}")]
    /// Operation timed out
    Timeout {
        /// Duration after which the timeout occurred
        duration: Duration,
    },

    #[error("Operation was cancelled")]
    /// Operation was cancelled
    Cancelled,

    #[error("Invalid timeout configuration: {message}")]
    /// Invalid timeout configuration
    InvalidConfiguration {
        /// Error message describing the issue
        message: String,
    },
}

impl From<Elapsed> for TimeoutError {
    fn from(_: Elapsed) -> Self {
        TimeoutError::Timeout {
            duration: Duration::ZERO, // Will be overridden with actual duration
        }
    }
}

/// Timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Default timeout duration
    pub default_timeout: Duration,
    /// Maximum allowed timeout
    pub max_timeout: Option<Duration>,
    /// Whether to warn on long operations
    pub warn_threshold: Option<Duration>,
    /// Grace period before hard timeout
    pub grace_period: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_timeout: Some(Duration::from_secs(300)), // 5 minutes
            warn_threshold: Some(Duration::from_secs(10)),
            grace_period: Some(Duration::from_secs(5)),
        }
    }
}

impl TimeoutConfig {
    /// Create a timeout config with a specific duration
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            default_timeout: timeout,
            ..Default::default()
        }
    }

    /// Set the maximum timeout
    #[must_use]
    pub fn with_max_timeout(mut self, max: Duration) -> Self {
        self.max_timeout = Some(max);
        self
    }

    /// Set the warning threshold
    #[must_use]
    pub fn with_warn_threshold(mut self, threshold: Duration) -> Self {
        self.warn_threshold = Some(threshold);
        self
    }

    /// Set the grace period
    #[must_use]
    pub fn with_grace_period(mut self, grace: Duration) -> Self {
        self.grace_period = Some(grace);
        self
    }

    /// Validate and clamp a requested timeout
    ///
    /// # Errors
    ///
    /// Returns `TimeoutError::InvalidConfiguration` if the duration is zero.
    pub fn validate_timeout(&self, requested: Duration) -> Result<Duration, TimeoutError> {
        if requested.is_zero() {
            return Err(TimeoutError::InvalidConfiguration {
                message: "Timeout duration must be greater than zero".to_string(),
            });
        }

        let effective = if let Some(max) = self.max_timeout {
            if requested > max {
                warn!(
                    "Requested timeout {:?} exceeds maximum {:?}, using maximum",
                    requested, max
                );
                max
            } else {
                requested
            }
        } else {
            requested
        };

        Ok(effective)
    }
}

/// Execute an operation with timeout
///
/// # Errors
///
/// Returns `TimeoutError::Timeout` if the operation exceeds the specified duration.
pub async fn with_timeout<F, T>(duration: Duration, operation: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    match timeout(duration, operation).await {
        Ok(result) => Ok(result),
        Err(_) => Err(TimeoutError::Timeout { duration }),
    }
}

/// Execute an operation with timeout and configuration
///
/// # Errors
///
/// Returns `TimeoutError::Timeout` if the operation exceeds the timeout duration.
/// Returns `TimeoutError::InvalidConfiguration` if the timeout configuration is invalid.
pub async fn with_timeout_config<F, T>(
    config: &TimeoutConfig,
    requested_timeout: Option<Duration>,
    operation: F,
) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    let duration = config.validate_timeout(requested_timeout.unwrap_or(config.default_timeout))?;

    let start = Instant::now();

    // Create a warning task if configured
    let warn_task = config.warn_threshold.map(|warn_threshold| {
        tokio::spawn(async move {
            tokio::time::sleep(warn_threshold).await;
            warn!("Operation is taking longer than {:?}", warn_threshold);
        })
    });

    let result = if let Ok(result) = timeout(duration, operation).await {
        let elapsed = start.elapsed();
        debug!("Operation completed in {:?}", elapsed);
        Ok(result)
    } else {
        warn!("Operation timed out after {:?}", duration);
        Err(TimeoutError::Timeout { duration })
    };

    // Cancel warning task if still running
    if let Some(task) = warn_task {
        task.abort();
    }

    result
}

/// A future that can be cancelled
pub struct CancellableTimeout<F> {
    inner: Pin<Box<F>>,
    deadline: Instant,
    cancelled: bool,
}

impl<F> CancellableTimeout<F>
where
    F: Future,
{
    /// Create a new cancellable timeout
    #[must_use]
    pub fn new(future: F, duration: Duration) -> Self {
        Self {
            inner: Box::pin(future),
            deadline: Instant::now() + duration,
            cancelled: false,
        }
    }

    /// Cancel the operation
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Check if cancelled
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Get remaining time
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
}

impl<F> Future for CancellableTimeout<F>
where
    F: Future,
{
    type Output = Result<F::Output, TimeoutError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.cancelled {
            return Poll::Ready(Err(TimeoutError::Cancelled));
        }

        if Instant::now() >= self.deadline {
            return Poll::Ready(Err(TimeoutError::Timeout {
                duration: Duration::ZERO, // Could store original duration
            }));
        }

        match self.inner.as_mut().poll(cx) {
            Poll::Ready(output) => Poll::Ready(Ok(output)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Timeout manager for coordinating multiple timeouts
#[derive(Clone)]
pub struct TimeoutManager {
    config: TimeoutConfig,
    active_timeouts: Arc<tokio::sync::Mutex<Vec<(String, Instant)>>>,
}

impl TimeoutManager {
    /// Create a new timeout manager
    #[must_use]
    pub fn new(config: TimeoutConfig) -> Self {
        Self {
            config,
            active_timeouts: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Execute an operation with managed timeout
    ///
    /// # Errors
    ///
    /// Returns `TimeoutError::Timeout` if the operation exceeds the timeout duration.
    /// Returns `TimeoutError::InvalidConfiguration` if the timeout configuration is invalid.
    pub async fn execute<F, T>(
        &self,
        name: impl Display,
        duration: Option<Duration>,
        operation: F,
    ) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        let name_str = name.to_string();
        let timeout_duration = self
            .config
            .validate_timeout(duration.unwrap_or(self.config.default_timeout))?;

        // Register the operation
        {
            let mut active = self.active_timeouts.lock().await;
            active.push((name_str.clone(), Instant::now() + timeout_duration));
            debug!(
                "Started operation '{}' with timeout {:?}",
                name_str, timeout_duration
            );
        }

        // Execute with timeout
        let result = with_timeout_config(&self.config, Some(timeout_duration), operation).await;

        // Unregister the operation
        {
            let mut active = self.active_timeouts.lock().await;
            active.retain(|(n, _)| n != &name_str);
        }

        result
    }

    /// Get currently active operations
    pub async fn active_operations(&self) -> Vec<(String, Duration)> {
        let active = self.active_timeouts.lock().await;
        let now = Instant::now();

        active
            .iter()
            .map(|(name, deadline)| {
                let remaining = deadline.saturating_duration_since(now);
                (name.clone(), remaining)
            })
            .collect()
    }

    /// Cancel all active operations (placeholder - would need actual cancellation tokens)
    pub async fn cancel_all(&self) {
        let mut active = self.active_timeouts.lock().await;
        warn!("Cancelling {} active operations", active.len());
        active.clear();
    }
}

/// Builder pattern for timeout operations
#[derive(Default)]
pub struct TimeoutBuilder {
    config: TimeoutConfig,
    name: Option<String>,
}

impl TimeoutBuilder {
    /// Set the timeout duration
    #[must_use]
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.default_timeout = duration;
        self
    }

    /// Set the operation name
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the warning threshold
    #[must_use]
    pub fn warn_after(mut self, duration: Duration) -> Self {
        self.config.warn_threshold = Some(duration);
        self
    }

    /// Set the grace period
    #[must_use]
    pub fn grace_period(mut self, duration: Duration) -> Self {
        self.config.grace_period = Some(duration);
        self
    }

    /// Execute the operation
    ///
    /// # Errors
    ///
    /// Returns `TimeoutError::Timeout` if the operation exceeds the timeout duration.
    pub async fn execute<F, T>(self, operation: F) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        let name = self.name.unwrap_or_else(|| "unnamed operation".to_string());
        debug!(
            "Executing '{}' with timeout {:?}",
            name, self.config.default_timeout
        );

        with_timeout_config(&self.config, None, operation).await
    }
}

/// Extension trait for futures to add timeout methods
pub trait TimeoutExt: Future + Sized {
    /// Add a timeout to this future
    fn with_timeout(
        self,
        duration: Duration,
    ) -> impl Future<Output = Result<Self::Output, TimeoutError>> {
        with_timeout(duration, self)
    }

    /// Add a cancellable timeout to this future
    fn with_cancellable_timeout(self, duration: Duration) -> CancellableTimeout<Self> {
        CancellableTimeout::new(self, duration)
    }
}

impl<F: Future> TimeoutExt for F {}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_successful_operation() {
        let result = with_timeout(Duration::from_secs(1), async {
            sleep(Duration::from_millis(100)).await;
            42
        })
        .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout() {
        let result = with_timeout(Duration::from_millis(100), async {
            sleep(Duration::from_secs(1)).await;
            42
        })
        .await;

        assert!(matches!(result, Err(TimeoutError::Timeout { .. })));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_config() {
        let config =
            TimeoutConfig::new(Duration::from_secs(1)).with_max_timeout(Duration::from_millis(500));

        // Should clamp to max timeout
        let validated = config.validate_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(validated, Duration::from_millis(500));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_builder() {
        let result = TimeoutBuilder::default()
            .duration(Duration::from_secs(1))
            .name("test operation")
            .execute(async {
                sleep(Duration::from_millis(100)).await;
                "success"
            })
            .await;

        assert_eq!(result.unwrap(), "success");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_manager() {
        let manager = TimeoutManager::new(TimeoutConfig::default());

        // Start an operation
        let handle = tokio::spawn({
            let manager = manager.clone();
            async move {
                manager
                    .execute("long operation", Some(Duration::from_secs(10)), async {
                        sleep(Duration::from_secs(1)).await;
                        "done"
                    })
                    .await
            }
        });

        // Check active operations
        sleep(Duration::from_millis(100)).await;
        let active = manager.active_operations().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].0, "long operation");

        // Wait for completion
        let result = handle.await.unwrap();
        assert_eq!(result.unwrap(), "done");

        // Should be no active operations
        let active = manager.active_operations().await;
        assert_eq!(active.len(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_extension_trait() {
        use crate::timeout::TimeoutExt;

        let future = async {
            sleep(Duration::from_millis(100)).await;
            42
        };

        let result = future.with_timeout(Duration::from_secs(1)).await;
        assert_eq!(result.unwrap(), 42);
    }
}
