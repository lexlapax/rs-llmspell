// ABOUTME: FlowController for rate limiting and backpressure handling in event bus
// ABOUTME: Implements token bucket algorithm with configurable rates and burst limits

use crate::overflow::{OverflowConfig, OverflowHandler, OverflowHandlerFactory, OverflowResult};
use crate::universal_event::UniversalEvent;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Notification when backpressure conditions change
#[derive(Debug, Clone, PartialEq)]
pub enum BackpressureNotification {
    /// Backpressure has started
    Started {
        buffer_size: usize,
        max_size: usize,
        reason: String,
    },
    /// Backpressure has been relieved
    Relieved { buffer_size: usize, max_size: usize },
    /// Rate limit exceeded
    RateLimitExceeded { current_rate: f64, limit_rate: f64 },
    /// Flow control warning
    Warning { message: String, buffer_size: usize },
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,
    /// Maximum tokens (burst capacity)
    capacity: f64,
    /// Token refill rate per second
    refill_rate: f64,
    /// Last refill timestamp
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume tokens
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        if elapsed > 0.0 {
            let new_tokens = elapsed * self.refill_rate;
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }

    /// Get current token count
    fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Flow control statistics
#[derive(Debug, Clone, Default)]
pub struct FlowStats {
    /// Total events processed
    pub events_processed: u64,
    /// Events dropped due to overflow
    pub events_dropped: u64,
    /// Events rejected
    pub events_rejected: u64,
    /// Events blocked
    pub events_blocked: u64,
    /// Rate limit violations
    pub rate_limit_violations: u64,
    /// Current buffer size
    pub current_buffer_size: usize,
    /// Maximum buffer size seen
    pub max_buffer_size_seen: usize,
    /// Average processing rate (events/sec)
    pub avg_processing_rate: f64,
}

/// Flow controller configuration
#[derive(Debug, Clone)]
pub struct FlowControllerConfig {
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimit>,
    /// Buffer overflow configuration
    pub overflow_config: OverflowConfig,
    /// Enable backpressure notifications
    pub enable_notifications: bool,
    /// Statistics collection interval
    pub stats_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum events per second
    pub max_rate: f64,
    /// Burst capacity
    pub burst_capacity: f64,
}

impl Default for FlowControllerConfig {
    fn default() -> Self {
        Self {
            rate_limit: Some(RateLimit {
                max_rate: 1000.0,       // 1000 events per second
                burst_capacity: 2000.0, // Allow bursts up to 2000
            }),
            overflow_config: OverflowConfig::default(),
            enable_notifications: true,
            stats_interval: Duration::from_secs(30),
        }
    }
}

/// Flow controller for rate limiting and backpressure handling
pub struct FlowController {
    /// Configuration
    config: FlowControllerConfig,
    /// Token bucket for rate limiting
    token_bucket: Arc<RwLock<Option<TokenBucket>>>,
    /// Overflow handler
    overflow_handler: Box<dyn OverflowHandler>,
    /// Event buffer for overflow management
    buffer: Arc<RwLock<VecDeque<UniversalEvent>>>,
    /// Flow statistics
    stats: Arc<RwLock<FlowStats>>,
    /// Notification sender
    notification_tx: Option<mpsc::UnboundedSender<BackpressureNotification>>,
    /// Start time for rate calculations
    start_time: Instant,
}

impl FlowController {
    /// Create a new flow controller
    pub fn new(config: FlowControllerConfig) -> Self {
        let token_bucket = if let Some(rate_limit) = &config.rate_limit {
            Arc::new(RwLock::new(Some(TokenBucket::new(
                rate_limit.burst_capacity,
                rate_limit.max_rate,
            ))))
        } else {
            Arc::new(RwLock::new(None))
        };

        let overflow_handler = OverflowHandlerFactory::create(config.overflow_config.strategy);

        Self {
            config,
            token_bucket,
            overflow_handler,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(FlowStats::default())),
            notification_tx: None,
            start_time: Instant::now(),
        }
    }

    /// Create a flow controller with notification channel
    pub fn with_notifications(
        config: FlowControllerConfig,
    ) -> (Self, mpsc::UnboundedReceiver<BackpressureNotification>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut controller = Self::new(config);
        controller.notification_tx = Some(tx);
        (controller, rx)
    }

    /// Check if an event can be processed (rate limiting)
    pub async fn can_process(&self, event: &UniversalEvent) -> bool {
        let mut bucket_guard = self.token_bucket.write();
        if let Some(bucket) = bucket_guard.as_mut() {
            let can_process = bucket.try_consume(1.0);

            if !can_process {
                self.update_stats(|stats| stats.rate_limit_violations += 1);

                if let Some(tx) = &self.notification_tx {
                    let _ = tx.send(BackpressureNotification::RateLimitExceeded {
                        current_rate: self.calculate_current_rate(),
                        limit_rate: self.config.rate_limit.as_ref().unwrap().max_rate,
                    });
                }

                debug!("Rate limit exceeded for event: {}", event.event_type);
            }

            can_process
        } else {
            true // No rate limiting
        }
    }

    /// Handle buffer overflow
    pub async fn handle_overflow(&self, event: UniversalEvent) -> OverflowResult {
        let buffer_size = self.buffer.read().len();
        let max_size = self.config.overflow_config.max_buffer_size;

        if !self.config.overflow_config.is_full(buffer_size) {
            // Buffer not full, add event
            self.buffer.write().push_back(event.clone());
            self.update_stats(|stats| {
                stats.events_processed += 1;
                stats.current_buffer_size = buffer_size + 1;
                stats.max_buffer_size_seen = stats.max_buffer_size_seen.max(buffer_size + 1);
            });

            // Check for high water mark warning
            if self.config.overflow_config.is_high_water(buffer_size + 1) {
                if let Some(tx) = &self.notification_tx {
                    let _ = tx.send(BackpressureNotification::Warning {
                        message: "Buffer approaching capacity".to_string(),
                        buffer_size: buffer_size + 1,
                    });
                }
            }

            return OverflowResult::Accepted;
        }

        // Handle overflow
        let result = self
            .overflow_handler
            .handle_overflow(event, buffer_size, max_size)
            .await;

        match &result {
            OverflowResult::Dropped { .. } => {
                self.update_stats(|stats| stats.events_dropped += 1);

                if let Some(tx) = &self.notification_tx {
                    let _ = tx.send(BackpressureNotification::Started {
                        buffer_size,
                        max_size,
                        reason: "Buffer overflow - dropping events".to_string(),
                    });
                }
            }
            OverflowResult::Rejected { .. } => {
                self.update_stats(|stats| stats.events_rejected += 1);
            }
            OverflowResult::Blocked => {
                self.update_stats(|stats| stats.events_blocked += 1);
            }
            OverflowResult::Accepted => {
                // Should not happen when buffer is full
                warn!("Overflow handler returned Accepted when buffer is full");
            }
        }

        result
    }

    /// Get next event from buffer
    pub fn pop_event(&self) -> Option<UniversalEvent> {
        let event = self.buffer.write().pop_front();

        if event.is_some() {
            let new_size = self.buffer.read().len();
            self.update_stats(|stats| stats.current_buffer_size = new_size);

            // Check if backpressure should be relieved
            if self.config.overflow_config.is_low_water(new_size) {
                if let Some(tx) = &self.notification_tx {
                    let _ = tx.send(BackpressureNotification::Relieved {
                        buffer_size: new_size,
                        max_size: self.config.overflow_config.max_buffer_size,
                    });
                }
            }
        }

        event
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.read().len()
    }

    /// Get flow statistics
    pub fn get_stats(&self) -> FlowStats {
        let mut stats = self.stats.read().clone();
        stats.avg_processing_rate = self.calculate_current_rate();
        stats
    }

    /// Clear all buffered events
    pub fn clear_buffer(&self) -> usize {
        let count = self.buffer.write().len();
        self.buffer.write().clear();
        self.update_stats(|stats| stats.current_buffer_size = 0);
        count
    }

    /// Check if buffer is empty
    pub fn is_buffer_empty(&self) -> bool {
        self.buffer.read().is_empty()
    }

    /// Get available tokens (for debugging)
    pub fn available_tokens(&self) -> Option<f64> {
        self.token_bucket
            .write()
            .as_mut()
            .map(|bucket| bucket.available_tokens())
    }

    /// Calculate current processing rate
    fn calculate_current_rate(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.stats.read().events_processed as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Update statistics
    fn update_stats<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut FlowStats),
    {
        update_fn(&mut self.stats.write());
    }
}

/// Builder for FlowController
pub struct FlowControllerBuilder {
    config: FlowControllerConfig,
}

impl FlowControllerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: FlowControllerConfig::default(),
        }
    }

    /// Set rate limiting
    pub fn with_rate_limit(mut self, max_rate: f64, burst_capacity: f64) -> Self {
        self.config.rate_limit = Some(RateLimit {
            max_rate,
            burst_capacity,
        });
        self
    }

    /// Disable rate limiting
    pub fn without_rate_limit(mut self) -> Self {
        self.config.rate_limit = None;
        self
    }

    /// Set overflow configuration
    pub fn with_overflow_config(mut self, overflow_config: OverflowConfig) -> Self {
        self.config.overflow_config = overflow_config;
        self
    }

    /// Enable/disable notifications
    pub fn with_notifications(mut self, enable: bool) -> Self {
        self.config.enable_notifications = enable;
        self
    }

    /// Set statistics interval
    pub fn with_stats_interval(mut self, interval: Duration) -> Self {
        self.config.stats_interval = interval;
        self
    }

    /// Build the flow controller
    pub fn build(self) -> FlowController {
        FlowController::new(self.config)
    }

    /// Build with notification channel
    pub fn build_with_notifications(
        self,
    ) -> (
        FlowController,
        mpsc::UnboundedReceiver<BackpressureNotification>,
    ) {
        FlowController::with_notifications(self.config)
    }
}

impl Default for FlowControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "event")]
mod tests {
    use super::*;
    use crate::universal_event::{Language, UniversalEvent};
    use serde_json::Value;
    use tokio::time::{sleep, Duration};

    fn create_test_event() -> UniversalEvent {
        UniversalEvent::new("test.event", Value::Null, Language::Rust)
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_flow_controller_basic() {
        let config = FlowControllerConfig::default();
        let controller = FlowController::new(config);

        let event = create_test_event();
        assert!(controller.can_process(&event).await);

        let result = controller.handle_overflow(event.clone()).await;
        assert_eq!(result, OverflowResult::Accepted);

        assert_eq!(controller.buffer_size(), 1);
        assert!(!controller.is_buffer_empty());

        let popped = controller.pop_event();
        assert!(popped.is_some());
        assert!(controller.is_buffer_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_rate_limiting() {
        let config = FlowControllerConfig {
            rate_limit: Some(RateLimit {
                max_rate: 2.0, // 2 events per second
                burst_capacity: 2.0,
            }),
            ..Default::default()
        };

        let controller = FlowController::new(config);
        let event = create_test_event();

        // First two events should pass (burst capacity)
        assert!(controller.can_process(&event).await);
        assert!(controller.can_process(&event).await);

        // Third event should be rate limited
        assert!(!controller.can_process(&event).await);

        // Wait for token refill
        sleep(Duration::from_millis(600)).await;
        assert!(controller.can_process(&event).await);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10.0, 5.0); // 10 capacity, 5/sec refill

        // Should be able to consume up to capacity
        assert!(bucket.try_consume(5.0));
        assert!(bucket.try_consume(5.0));
        assert!(!bucket.try_consume(1.0)); // No tokens left

        // Check available tokens (use approximate comparison due to floating point precision)
        assert!(bucket.available_tokens() < 0.001);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_overflow_handling() {
        let config = FlowControllerConfig {
            overflow_config: OverflowConfig::new(crate::overflow::OverflowStrategy::DropNewest, 2),
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Fill buffer to capacity
        let event1 = create_test_event();
        let event2 = create_test_event();
        let event3 = create_test_event();

        assert_eq!(
            controller.handle_overflow(event1).await,
            OverflowResult::Accepted
        );
        assert_eq!(
            controller.handle_overflow(event2).await,
            OverflowResult::Accepted
        );

        // Third event should trigger overflow
        let result = controller.handle_overflow(event3).await;
        assert!(matches!(result, OverflowResult::Dropped { .. }));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_statistics() {
        let controller = FlowController::new(FlowControllerConfig::default());

        let event = create_test_event();
        let _ = controller.handle_overflow(event).await;

        let stats = controller.get_stats();
        assert_eq!(stats.events_processed, 1);
        assert_eq!(stats.current_buffer_size, 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_builder() {
        let (controller, _rx) = FlowControllerBuilder::new()
            .with_rate_limit(100.0, 200.0)
            .with_notifications(true)
            .build_with_notifications();

        assert!(controller.config.rate_limit.is_some());
        assert!(controller.notification_tx.is_some());
    }
}
