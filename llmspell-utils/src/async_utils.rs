// ABOUTME: Async operation utilities including timeouts, cancellation, and concurrency helpers
// ABOUTME: Provides common patterns for managing async tasks in the LLMSpell framework

//! Async operation utilities and helpers
//!
//! This module provides utilities for working with asynchronous operations,
//! including timeout management, cancellation tokens, and concurrency helpers.

use futures::stream::{FuturesUnordered, StreamExt};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use thiserror::Error;
use tokio::time;

/// Represents a cancellable async operation
pub struct Cancellable;

/// Errors that can occur during async operations
#[derive(Debug, Error)]
pub enum AsyncError {
    /// Operation timed out
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),

    /// Retry limit exceeded
    #[error("Retry limit exceeded after {attempts} attempts")]
    RetryLimitExceeded {
        /// Number of attempts made before giving up
        attempts: usize,
    },

    /// Operation was cancelled
    #[error("Operation was cancelled")]
    Cancelled,
}

/// Result type for async operations
pub type AsyncResult<T> = Result<T, AsyncError>;

/// Configuration for retry operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts
    pub max_attempts: usize,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Factor to multiply delay by after each attempt
    pub backoff_factor: f64,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Add jitter to retry delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            max_delay: Duration::from_secs(10),
            jitter: true,
        }
    }
}

/// Execute an async operation with a timeout
///
/// # Errors
///
/// Returns an error if the operation times out
///
/// # Examples
///
/// ```
/// use llmspell_utils::async_utils::timeout;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = timeout(
///     Duration::from_secs(1),
///     async { 42 }
/// ).await?;
/// assert_eq!(result, 42);
/// # Ok(())
/// # }
/// ```
pub async fn timeout<T, F>(duration: Duration, future: F) -> Result<T, time::error::Elapsed>
where
    F: Future<Output = T>,
{
    time::timeout(duration, future).await
}

/// Execute an async operation with a timeout, returning a default value on timeout
///
/// # Examples
///
/// ```
/// use llmspell_utils::async_utils::timeout_with_default;
/// use std::time::Duration;
///
/// # async fn example() {
/// let result = timeout_with_default(
///     Duration::from_secs(1),
///     async { 42 },
///     0
/// ).await;
/// assert_eq!(result, 42);
/// # }
/// ```
pub async fn timeout_with_default<T, F>(duration: Duration, future: F, default: T) -> T
where
    F: Future<Output = T>,
{
    match time::timeout(duration, future).await {
        Ok(value) => value,
        Err(_) => default,
    }
}

/// Retry an async operation with exponential backoff
///
/// # Errors
///
/// Returns `AsyncError::RetryLimitExceeded` if all attempts fail
///
/// # Examples
///
/// ```
/// use llmspell_utils::async_utils::{retry_async, RetryConfig};
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let counter = Arc::new(AtomicUsize::new(0));
/// let counter_clone = counter.clone();
///
/// let result = retry_async(RetryConfig::default(), || {
///     let count = counter_clone.fetch_add(1, Ordering::SeqCst);
///     async move {
///         if count < 2 {
///             Err("Not ready yet")
///         } else {
///             Ok("Success!")
///         }
///     }
/// }).await?;
///
/// assert_eq!(result, "Success!");
/// assert_eq!(counter.load(Ordering::SeqCst), 3);
/// # Ok(())
/// # }
/// ```
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub async fn retry_async<T, E, F, Fut>(config: RetryConfig, mut f: F) -> Result<T, AsyncError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Debug,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;

        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt >= config.max_attempts {
                    tracing::error!(
                        "Retry limit exceeded after {} attempts. Last error: {:?}",
                        attempt,
                        e
                    );
                    return Err(AsyncError::RetryLimitExceeded { attempts: attempt });
                }

                tracing::debug!(
                    "Attempt {} failed with error: {:?}. Retrying in {:?}",
                    attempt,
                    e,
                    delay
                );

                // Add jitter if configured
                let actual_delay = if config.jitter {
                    let jitter_range = delay.as_millis() as f64 * 0.1;
                    let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
                    Duration::from_millis((delay.as_millis() as f64 + jitter).max(0.0) as u64)
                } else {
                    delay
                };

                time::sleep(actual_delay).await;

                // Calculate next delay with exponential backoff
                delay = Duration::from_millis(
                    ((delay.as_millis() as f64 * config.backoff_factor) as u64)
                        .min(config.max_delay.as_millis() as u64),
                );
            }
        }
    }
}

/// Map a function over async operations with a concurrency limit
///
/// # Examples
///
/// ```
/// use llmspell_utils::async_utils::concurrent_map;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let numbers = vec![1, 2, 3, 4, 5];
/// let results = concurrent_map(
///     numbers.into_iter(),
///     2, // Max 2 concurrent operations
///     |n| async move { n * 2 }
/// ).await;
///
/// assert_eq!(results, vec![2, 4, 6, 8, 10]);
/// # Ok(())
/// # }
/// ```
pub async fn concurrent_map<I, F, Fut, T, U>(items: I, concurrency_limit: usize, f: F) -> Vec<U>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Fut + Clone,
    Fut: Future<Output = U>,
    T: Send + 'static,
    U: Send + 'static,
{
    let mut futures = FuturesUnordered::new();
    let mut items = items.into_iter();
    let mut results = Vec::new();
    let mut active_count = 0;
    let mut item_index = 0;
    let mut result_map = std::collections::HashMap::new();

    loop {
        // Fill up to concurrency limit
        while active_count < concurrency_limit {
            if let Some(item) = items.next() {
                let fut = f.clone()(item);
                let index = item_index;
                item_index += 1;
                active_count += 1;

                futures.push(async move {
                    let result = fut.await;
                    (index, result)
                });
            } else {
                break;
            }
        }

        // If no futures are active and no more items, we're done
        if active_count == 0 {
            break;
        }

        // Wait for a future to complete
        if let Some((index, result)) = futures.next().await {
            active_count -= 1;
            result_map.insert(index, result);
        }
    }

    // Reconstruct results in original order
    for i in 0..item_index {
        if let Some(result) = result_map.remove(&i) {
            results.push(result);
        }
    }

    results
}

/// A future that can be cancelled
pub struct CancellableFuture<F> {
    future: F,
    cancelled: bool,
}

impl<F> CancellableFuture<F> {
    /// Create a new cancellable future
    pub fn new(future: F) -> Self {
        Self {
            future,
            cancelled: false,
        }
    }

    /// Cancel this future
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Check if this future has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl<F> Future for CancellableFuture<F>
where
    F: Future,
{
    type Output = Result<F::Output, AsyncError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if this.cancelled {
            return Poll::Ready(Err(AsyncError::Cancelled));
        }

        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        match future.poll(cx) {
            Poll::Ready(value) => Poll::Ready(Ok(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A type alias for boxed futures returning results
pub type BoxedResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

/// Execute multiple futures concurrently and return the first successful result
///
/// # Errors
///
/// Returns `AsyncError::RetryLimitExceeded` if all futures fail
///
/// # Examples
///
/// ```
/// use llmspell_utils::async_utils::{race_to_success, BoxedResultFuture};
/// use std::time::Duration;
/// use tokio::time::sleep;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let futures: Vec<BoxedResultFuture<&str, &str>> = vec![
///     Box::pin(async {
///         sleep(Duration::from_millis(100)).await;
///         Ok("slow")
///     }),
///     Box::pin(async {
///         sleep(Duration::from_millis(10)).await;
///         Ok("fast")
///     }),
/// ];
///
/// let result = race_to_success(futures).await?;
/// assert_eq!(result, "fast");
/// # Ok(())
/// # }
/// ```
pub async fn race_to_success<T, E>(futures: Vec<BoxedResultFuture<T, E>>) -> Result<T, AsyncError>
where
    T: Send + 'static,
    E: Debug + Send + 'static,
{
    use futures::future::select_all;

    if futures.is_empty() {
        return Err(AsyncError::RetryLimitExceeded { attempts: 0 });
    }

    let (result, _index, _remaining) = select_all(futures).await;

    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            tracing::error!("All futures failed. Last error: {:?}", e);
            Err(AsyncError::RetryLimitExceeded { attempts: 1 })
        }
    }
}

/// Add jitter for backoff calculations
#[cfg(test)]
#[cfg_attr(test_category = "util")]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn add_jitter(duration: Duration, jitter_fraction: f64) -> Duration {
    let millis = duration.as_millis() as f64;
    let jitter = millis * jitter_fraction * (rand::random::<f64>() - 0.5) * 2.0;
    Duration::from_millis((millis + jitter).max(0.0) as u64)
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::time::sleep;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_success() {
        let result = timeout(Duration::from_secs(1), async { 42 }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_failure() {
        let result = timeout(Duration::from_millis(10), async {
            sleep(Duration::from_millis(100)).await;
            42
        })
        .await;
        assert!(result.is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_with_default_success() {
        let result = timeout_with_default(Duration::from_secs(1), async { 42 }, 0).await;
        assert_eq!(result, 42);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_timeout_with_default_timeout() {
        let result = timeout_with_default(
            Duration::from_millis(10),
            async {
                sleep(Duration::from_millis(100)).await;
                42
            },
            0,
        )
        .await;
        assert_eq!(result, 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_retry_async_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            backoff_factor: 1.0,
            max_delay: Duration::from_millis(10),
            jitter: false,
        };

        let result = retry_async(config, || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err("Not ready yet")
                } else {
                    Ok("Success!")
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(result, "Success!");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_retry_async_failure() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            backoff_factor: 1.0,
            max_delay: Duration::from_millis(10),
            jitter: false,
        };

        let result = retry_async(config, || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            async move { Err::<(), _>("Always fails") }
        })
        .await;

        assert!(matches!(
            result,
            Err(AsyncError::RetryLimitExceeded { attempts: 2 })
        ));
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_concurrent_map() {
        let numbers = vec![1, 2, 3, 4, 5];
        let results = concurrent_map(numbers.into_iter(), 2, |n| async move { n * 2 }).await;

        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_concurrent_map_with_delay() {
        let numbers = vec![1, 2, 3];
        let start = std::time::Instant::now();

        let results = concurrent_map(
            numbers.into_iter(),
            2, // Only 2 concurrent operations
            |n| async move {
                sleep(Duration::from_millis(50)).await;
                n * 2
            },
        )
        .await;

        let elapsed = start.elapsed();
        assert_eq!(results, vec![2, 4, 6]);
        // With concurrency of 2, should take ~100ms (2 batches)
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_cancellable_future() {
        let mut future = CancellableFuture::new(async {
            sleep(Duration::from_millis(100)).await;
            42
        });

        future.cancel();
        let result = future.await;

        assert!(matches!(result, Err(AsyncError::Cancelled)));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_race_to_success() {
        type TestFuture<'a> = Pin<Box<dyn Future<Output = Result<&'a str, &'a str>> + Send + 'a>>;
        let futures: Vec<TestFuture> = vec![
            Box::pin(async {
                sleep(Duration::from_millis(100)).await;
                Ok("slow")
            }),
            Box::pin(async {
                sleep(Duration::from_millis(10)).await;
                Ok("fast")
            }),
        ];

        let result = race_to_success(futures).await.unwrap();
        assert_eq!(result, "fast");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_add_jitter() {
        let base_duration = Duration::from_millis(100);
        let jittered = add_jitter(base_duration, 0.1);

        // Jitter should be within 10% of base duration
        assert!(jittered.as_millis() >= 90);
        assert!(jittered.as_millis() <= 110);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_retry_with_backoff() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let times = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let times_clone = times.clone();

        let config = RetryConfig {
            max_attempts: 4,
            initial_delay: Duration::from_millis(10),
            backoff_factor: 2.0,
            max_delay: Duration::from_millis(100),
            jitter: false,
        };

        let start = std::time::Instant::now();

        let _ = retry_async(config, || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            times_clone.lock().push(start.elapsed());
            async move { Err::<(), _>("Always fails") }
        })
        .await;

        let times = times.lock();
        assert_eq!(times.len(), 4);

        // Verify exponential backoff timing
        // First attempt should be immediate
        assert!(times[0] < Duration::from_millis(5));
        // Second attempt after ~10ms
        assert!(times[1] >= Duration::from_millis(8));
        assert!(times[1] < Duration::from_millis(20));
        // Third attempt after ~20ms more (total ~30ms)
        assert!(times[2] >= Duration::from_millis(25));
        assert!(times[2] < Duration::from_millis(40));
        // Fourth attempt after ~40ms more (total ~70ms)
        assert!(times[3] >= Duration::from_millis(60));
        assert!(times[3] < Duration::from_millis(90));
    }
}
