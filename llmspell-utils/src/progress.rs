// ABOUTME: Progress reporting framework for long-running operations
// ABOUTME: Provides a unified way to track and report progress with callbacks and streaming updates

use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

/// Progress error types
#[derive(Debug, Error)]
pub enum ProgressError {
    #[error("Progress reporter is closed")]
    /// Progress reporter channel is closed
    Closed,

    #[error("Invalid progress value: {message}")]
    /// Invalid progress value provided
    InvalidValue {
        /// Error message describing the issue
        message: String,
    },

    #[error("Progress operation failed: {message}")]
    /// Progress operation failed
    OperationFailed {
        /// Error message describing the failure
        message: String,
    },
}

/// Progress update event
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Started a new operation
    Started {
        /// Name of the operation
        operation: String,
        /// Total number of items to process (if known)
        total: Option<u64>,
    },

    /// Progress update
    Update {
        /// Current progress value
        current: u64,
        /// Total number of items (if known)
        total: Option<u64>,
        /// Optional progress message
        message: Option<String>,
    },

    /// Sub-task progress
    SubTask {
        /// Name of the sub-task
        name: String,
        /// Current progress value
        current: u64,
        /// Total for this sub-task
        total: u64,
    },

    /// Operation completed
    Completed {
        /// Optional completion message
        message: Option<String>,
        /// Total operation duration
        duration: Duration,
    },

    /// Operation failed
    /// Operation failed
    Failed {
        /// Error message
        error: String,
        /// Duration before failure
        duration: Duration,
    },

    /// Custom event
    Custom {
        /// Type of the custom event
        event_type: String,
        /// Event data payload
        data: serde_json::Value,
    },
}

/// Progress callback function
pub type ProgressCallback = Box<dyn Fn(ProgressEvent) + Send + Sync>;

/// Progress reporter for tracking operation progress
pub struct ProgressReporter {
    operation: String,
    total: Option<u64>,
    current: Arc<RwLock<u64>>,
    started_at: Instant,
    sender: mpsc::UnboundedSender<ProgressEvent>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(
        operation: impl Into<String>,
        total: Option<u64>,
    ) -> (Self, mpsc::UnboundedReceiver<ProgressEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let operation = operation.into();

        // Send started event
        let _ = sender.send(ProgressEvent::Started {
            operation: operation.clone(),
            total,
        });

        let reporter = Self {
            operation,
            total,
            current: Arc::new(RwLock::new(0)),
            started_at: Instant::now(),
            sender,
        };

        (reporter, receiver)
    }

    /// Update progress by incrementing
    ///
    /// # Errors
    ///
    /// Returns `ProgressError::InvalidValue` if the resulting value exceeds the total.
    /// Returns `ProgressError::Closed` if the progress channel is closed.
    pub async fn increment(&self, delta: u64) -> Result<(), ProgressError> {
        let mut current = self.current.write().await;
        *current += delta;
        let current_val = *current;
        drop(current);

        self.update(current_val, None).await
    }

    /// Update progress to a specific value
    ///
    /// # Errors
    ///
    /// Returns `ProgressError::InvalidValue` if current exceeds total.
    /// Returns `ProgressError::Closed` if the progress channel is closed.
    pub async fn update(&self, current: u64, message: Option<String>) -> Result<(), ProgressError> {
        // Validate progress
        if let Some(total) = self.total {
            if current > total {
                return Err(ProgressError::InvalidValue {
                    message: format!("Progress {current} exceeds total {total}"),
                });
            }
        }

        // Update current value
        *self.current.write().await = current;

        // Send update event
        self.sender
            .send(ProgressEvent::Update {
                current,
                total: self.total,
                message,
            })
            .map_err(|_| ProgressError::Closed)?;

        // Log progress
        if let Some(total) = self.total {
            #[allow(clippy::cast_precision_loss)]
            let percentage = (current as f64 / total as f64) * 100.0;
            debug!(
                "{}: {:.1}% ({}/{})",
                self.operation, percentage, current, total
            );
        } else {
            debug!("{}: {} items processed", self.operation, current);
        }

        Ok(())
    }

    /// Report a sub-task
    ///
    /// # Errors
    ///
    /// Returns `ProgressError::Closed` if the progress channel is closed.
    pub fn subtask(
        &self,
        name: impl Display,
        current: u64,
        total: u64,
    ) -> Result<(), ProgressError> {
        self.sender
            .send(ProgressEvent::SubTask {
                name: name.to_string(),
                current,
                total,
            })
            .map_err(|_| ProgressError::Closed)?;

        Ok(())
    }

    /// Send a custom event
    ///
    /// # Errors
    ///
    /// Returns `ProgressError::Closed` if the progress channel is closed.
    pub fn custom_event(
        &self,
        event_type: impl Into<String>,
        data: serde_json::Value,
    ) -> Result<(), ProgressError> {
        self.sender
            .send(ProgressEvent::Custom {
                event_type: event_type.into(),
                data,
            })
            .map_err(|_| ProgressError::Closed)?;

        Ok(())
    }

    /// Mark the operation as completed
    ///
    /// # Errors
    ///
    /// Returns `ProgressError::Closed` if the progress channel is closed.
    pub fn complete(self, message: Option<String>) -> Result<(), ProgressError> {
        let duration = self.started_at.elapsed();

        self.sender
            .send(ProgressEvent::Completed { message, duration })
            .map_err(|_| ProgressError::Closed)?;

        info!("{} completed in {:?}", self.operation, duration);
        Ok(())
    }

    /// Mark the operation as failed
    ///
    /// # Errors
    ///
    /// Returns `ProgressError::Closed` if the progress channel is closed.
    pub fn fail(self, error: impl Display) -> Result<(), ProgressError> {
        let duration = self.started_at.elapsed();

        self.sender
            .send(ProgressEvent::Failed {
                error: error.to_string(),
                duration,
            })
            .map_err(|_| ProgressError::Closed)?;

        Ok(())
    }

    /// Get elapsed time
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Get current progress value
    #[must_use]
    pub async fn current(&self) -> u64 {
        *self.current.read().await
    }

    /// Get progress percentage (if total is known)
    #[must_use]
    pub async fn percentage(&self) -> Option<f64> {
        if let Some(total) = self.total {
            let current = *self.current.read().await;
            #[allow(clippy::cast_precision_loss)]
            {
                Some((current as f64 / total as f64) * 100.0)
            }
        } else {
            None
        }
    }
}

/// Progress tracker that manages multiple progress reporters
pub struct ProgressTracker {
    callbacks: Arc<RwLock<Vec<ProgressCallback>>>,
    active_operations: Arc<RwLock<Vec<String>>>,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressTracker {
    /// Create a new progress tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(RwLock::new(Vec::new())),
            active_operations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a progress callback
    pub async fn add_callback<F>(&self, callback: F)
    where
        F: Fn(ProgressEvent) + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(Box::new(callback));
    }

    /// Create a new progress reporter
    pub async fn create_reporter(
        &self,
        operation: impl Into<String>,
        total: Option<u64>,
    ) -> ProgressReporter {
        let operation = operation.into();

        // Track active operation
        {
            let mut active = self.active_operations.write().await;
            active.push(operation.clone());
        }

        let (reporter, mut receiver) = ProgressReporter::new(operation.clone(), total);

        // Forward events to callbacks
        let callbacks = self.callbacks.clone();
        let active_operations = self.active_operations.clone();
        let op_name = operation.clone();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                // Handle completion/failure
                match &event {
                    ProgressEvent::Completed { .. } | ProgressEvent::Failed { .. } => {
                        let mut active = active_operations.write().await;
                        active.retain(|op| op != &op_name);
                    }
                    _ => {}
                }

                // Forward to callbacks
                let callbacks = callbacks.read().await;
                for callback in callbacks.iter() {
                    callback(event.clone());
                }
            }
        });

        reporter
    }

    /// Get active operations
    pub async fn active_operations(&self) -> Vec<String> {
        self.active_operations.read().await.clone()
    }
}

/// Progress reporter builder for fluent API
pub struct ProgressBuilder {
    operation: String,
    total: Option<u64>,
    tracker: Option<ProgressTracker>,
}

impl ProgressBuilder {
    /// Create a new progress builder
    #[must_use]
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            total: None,
            tracker: None,
        }
    }

    /// Set the total number of items
    #[must_use]
    pub fn total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }

    /// Set the progress tracker
    #[must_use]
    pub fn tracker(mut self, tracker: ProgressTracker) -> Self {
        self.tracker = Some(tracker);
        self
    }

    /// Build the progress reporter
    pub async fn build(self) -> ProgressReporter {
        if let Some(tracker) = self.tracker {
            tracker.create_reporter(self.operation, self.total).await
        } else {
            let (reporter, _) = ProgressReporter::new(self.operation, self.total);
            reporter
        }
    }
}

/// Extension trait for iterators with progress reporting
pub trait ProgressIteratorExt: Iterator + Sized {
    /// Add progress reporting to an iterator
    fn with_progress(self, reporter: Arc<ProgressReporter>) -> ProgressIterator<Self> {
        ProgressIterator {
            iter: self,
            reporter,
            count: 0,
        }
    }
}

impl<I: Iterator> ProgressIteratorExt for I {}

/// Iterator wrapper that reports progress
pub struct ProgressIterator<I> {
    iter: I,
    reporter: Arc<ProgressReporter>,
    count: u64,
}

impl<I: Iterator> Iterator for ProgressIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();

        if item.is_some() {
            self.count += 1;
            let reporter = self.reporter.clone();
            let count = self.count;

            // Update progress asynchronously
            tokio::spawn(async move {
                let _ = reporter.update(count, None).await;
            });
        }

        item
    }
}

/// Macro for easy progress reporting
#[macro_export]
macro_rules! progress {
    ($reporter:expr, $current:expr) => {
        $reporter.update($current, None).await?
    };

    ($reporter:expr, $current:expr, $message:expr) => {
        $reporter
            .update($current, Some($message.to_string()))
            .await?
    };
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_progress_reporter() {
        let (reporter, mut receiver) = ProgressReporter::new("Test Operation", Some(100));

        // Update progress
        reporter
            .update(50, Some("Halfway".to_string()))
            .await
            .unwrap();

        // Check events
        let event = receiver.recv().await.unwrap();
        assert!(matches!(event, ProgressEvent::Started { .. }));

        let event = receiver.recv().await.unwrap();
        match event {
            ProgressEvent::Update {
                current,
                total,
                message,
            } => {
                assert_eq!(current, 50);
                assert_eq!(total, Some(100));
                assert_eq!(message, Some("Halfway".to_string()));
            }
            _ => panic!("Expected Update event"),
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_progress_increment() {
        let (reporter, mut receiver) = ProgressReporter::new("Increment Test", Some(10));

        // Skip started event
        receiver.recv().await.unwrap();

        // Increment
        reporter.increment(3).await.unwrap();
        reporter.increment(2).await.unwrap();

        // Check final value
        assert_eq!(reporter.current().await, 5);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_progress_tracker() {
        let tracker = ProgressTracker::new();
        let events = Arc::new(Mutex::new(Vec::new()));

        // Add callback
        {
            let events = events.clone();
            tracker
                .add_callback(move |event| {
                    let events = events.clone();
                    tokio::spawn(async move {
                        events.lock().await.push(event);
                    });
                })
                .await;
        }

        // Create reporter
        let reporter = tracker.create_reporter("Tracked Operation", Some(50)).await;

        // Update progress
        reporter.update(25, None).await.unwrap();

        // Wait for events
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check events
        let events = events.lock().await;
        assert!(events.len() >= 2); // Started + Update
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_progress_completion() {
        let (reporter, mut receiver) = ProgressReporter::new("Complete Test", None);

        // Skip started event
        receiver.recv().await.unwrap();

        // Complete
        reporter.complete(Some("Done!".to_string())).unwrap();

        // Check event
        let event = receiver.recv().await.unwrap();
        match event {
            ProgressEvent::Completed { message, duration } => {
                assert_eq!(message, Some("Done!".to_string()));
                assert!(duration.as_millis() < 1000); // Should complete quickly
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_progress_builder() {
        let reporter = ProgressBuilder::new("Built Operation")
            .total(200)
            .build()
            .await;

        assert_eq!(reporter.total, Some(200));
        assert_eq!(reporter.operation, "Built Operation");
    }
}
