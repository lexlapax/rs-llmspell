// ABOUTME: Overflow strategies for handling backpressure and buffer limits
// ABOUTME: Provides 4 strategies: DropOldest, DropNewest, Block, and Reject

use crate::universal_event::UniversalEvent;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{debug, warn};

/// Strategy for handling buffer overflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStrategy {
    /// Drop the oldest events when buffer is full
    DropOldest,
    /// Drop the newest events when buffer is full
    DropNewest,
    /// Block until space is available
    Block,
    /// Reject new events with an error
    Reject,
}

impl Default for OverflowStrategy {
    fn default() -> Self {
        Self::DropOldest
    }
}

/// Result of overflow handling
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowResult {
    /// Event was accepted
    Accepted,
    /// Event was dropped
    Dropped { reason: String },
    /// Event was blocked (should retry)
    Blocked,
    /// Event was rejected (error)
    Rejected { reason: String },
}

impl OverflowResult {
    /// Check if the operation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, OverflowResult::Accepted)
    }

    /// Check if the operation should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self, OverflowResult::Blocked)
    }
}

/// Trait for handling overflow situations
#[async_trait]
pub trait OverflowHandler: Send + Sync + std::fmt::Debug {
    /// Handle an overflow situation
    async fn handle_overflow(
        &self,
        event: UniversalEvent,
        buffer_size: usize,
        max_size: usize,
    ) -> OverflowResult;

    /// Get the strategy name
    fn strategy_name(&self) -> &'static str;
}

/// Drop oldest events overflow handler
#[derive(Debug)]
pub struct DropOldestHandler;

#[async_trait]
impl OverflowHandler for DropOldestHandler {
    async fn handle_overflow(
        &self,
        _event: UniversalEvent,
        buffer_size: usize,
        max_size: usize,
    ) -> OverflowResult {
        debug!(
            "DropOldest: Buffer full ({}/{}), will drop oldest event",
            buffer_size, max_size
        );
        OverflowResult::Accepted
    }

    fn strategy_name(&self) -> &'static str {
        "drop_oldest"
    }
}

/// Drop newest events overflow handler
#[derive(Debug)]
pub struct DropNewestHandler;

#[async_trait]
impl OverflowHandler for DropNewestHandler {
    async fn handle_overflow(
        &self,
        event: UniversalEvent,
        buffer_size: usize,
        max_size: usize,
    ) -> OverflowResult {
        warn!(
            "DropNewest: Buffer full ({}/{}), dropping event: {}",
            buffer_size, max_size, event.event_type
        );
        OverflowResult::Dropped {
            reason: "Buffer full, dropping newest event".to_string(),
        }
    }

    fn strategy_name(&self) -> &'static str {
        "drop_newest"
    }
}

/// Block until space available overflow handler
#[derive(Debug)]
pub struct BlockHandler {
    notify: Arc<Notify>,
}

impl BlockHandler {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    /// Notify waiting producers that space is available
    pub fn notify_space_available(&self) {
        self.notify.notify_waiters();
    }
}

impl Default for BlockHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OverflowHandler for BlockHandler {
    async fn handle_overflow(
        &self,
        _event: UniversalEvent,
        buffer_size: usize,
        max_size: usize,
    ) -> OverflowResult {
        debug!(
            "Block: Buffer full ({}/{}), waiting for space",
            buffer_size, max_size
        );

        // Wait for notification that space is available
        self.notify.notified().await;

        OverflowResult::Blocked
    }

    fn strategy_name(&self) -> &'static str {
        "block"
    }
}

/// Reject overflow handler
#[derive(Debug, Default)]
pub struct RejectHandler;

#[async_trait]
impl OverflowHandler for RejectHandler {
    async fn handle_overflow(
        &self,
        event: UniversalEvent,
        buffer_size: usize,
        max_size: usize,
    ) -> OverflowResult {
        warn!(
            "Reject: Buffer full ({}/{}), rejecting event: {}",
            buffer_size, max_size, event.event_type
        );
        OverflowResult::Rejected {
            reason: format!("Buffer full ({}/{}), event rejected", buffer_size, max_size),
        }
    }

    fn strategy_name(&self) -> &'static str {
        "reject"
    }
}

/// Factory for creating overflow handlers
pub struct OverflowHandlerFactory;

impl OverflowHandlerFactory {
    /// Create an overflow handler for the given strategy
    pub fn create(strategy: OverflowStrategy) -> Box<dyn OverflowHandler> {
        match strategy {
            OverflowStrategy::DropOldest => Box::new(DropOldestHandler),
            OverflowStrategy::DropNewest => Box::new(DropNewestHandler),
            OverflowStrategy::Block => Box::new(BlockHandler::new()),
            OverflowStrategy::Reject => Box::new(RejectHandler),
        }
    }
}

/// Configuration for overflow handling
#[derive(Debug, Clone)]
pub struct OverflowConfig {
    /// The overflow strategy to use
    pub strategy: OverflowStrategy,
    /// Maximum buffer size before overflow
    pub max_buffer_size: usize,
    /// High water mark for warnings
    pub high_water_mark: usize,
    /// Low water mark for backpressure relief
    pub low_water_mark: usize,
}

impl Default for OverflowConfig {
    fn default() -> Self {
        Self {
            strategy: OverflowStrategy::DropOldest,
            max_buffer_size: 10000,
            high_water_mark: 8000,
            low_water_mark: 2000,
        }
    }
}

impl OverflowConfig {
    /// Create a new overflow configuration
    pub fn new(strategy: OverflowStrategy, max_buffer_size: usize) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let max_size_f64 = max_buffer_size as f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let high_water_mark = (max_size_f64 * 0.8) as usize;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let low_water_mark = (max_size_f64 * 0.2) as usize;

        Self {
            strategy,
            max_buffer_size,
            high_water_mark,
            low_water_mark,
        }
    }

    /// Check if buffer size is at high water mark
    pub fn is_high_water(&self, size: usize) -> bool {
        size >= self.high_water_mark
    }

    /// Check if buffer size is at low water mark
    pub fn is_low_water(&self, size: usize) -> bool {
        size <= self.low_water_mark
    }

    /// Check if buffer is full
    pub fn is_full(&self, size: usize) -> bool {
        size >= self.max_buffer_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_event::{Language, UniversalEvent};
    use serde_json::Value;

    fn create_test_event() -> UniversalEvent {
        UniversalEvent::new("test.event", Value::Null, Language::Rust)
    }
    #[tokio::test]
    async fn test_drop_oldest_handler() {
        let handler = DropOldestHandler;
        let event = create_test_event();

        let result = handler.handle_overflow(event, 100, 100).await;
        assert_eq!(result, OverflowResult::Accepted);
        assert_eq!(handler.strategy_name(), "drop_oldest");
    }
    #[tokio::test]
    async fn test_drop_newest_handler() {
        let handler = DropNewestHandler;
        let event = create_test_event();

        let result = handler.handle_overflow(event, 100, 100).await;
        assert!(matches!(result, OverflowResult::Dropped { .. }));
        assert_eq!(handler.strategy_name(), "drop_newest");
    }
    #[tokio::test]
    async fn test_reject_handler() {
        let handler = RejectHandler;
        let event = create_test_event();

        let result = handler.handle_overflow(event, 100, 100).await;
        assert!(matches!(result, OverflowResult::Rejected { .. }));
        assert_eq!(handler.strategy_name(), "reject");
    }
    #[test]
    fn test_overflow_config() {
        let config = OverflowConfig::new(OverflowStrategy::Block, 1000);

        assert_eq!(config.max_buffer_size, 1000);
        assert_eq!(config.high_water_mark, 800);
        assert_eq!(config.low_water_mark, 200);

        assert!(!config.is_high_water(500));
        assert!(config.is_high_water(900));
        assert!(config.is_low_water(100));
        assert!(!config.is_low_water(500));
        assert!(config.is_full(1000));
        assert!(!config.is_full(999));
    }
    #[test]
    fn test_overflow_result() {
        assert!(OverflowResult::Accepted.is_success());
        assert!(!OverflowResult::Dropped {
            reason: "test".to_string()
        }
        .is_success());

        assert!(OverflowResult::Blocked.should_retry());
        assert!(!OverflowResult::Accepted.should_retry());
    }
    #[test]
    fn test_overflow_handler_factory() {
        let handler = OverflowHandlerFactory::create(OverflowStrategy::DropOldest);
        assert_eq!(handler.strategy_name(), "drop_oldest");

        let handler = OverflowHandlerFactory::create(OverflowStrategy::DropNewest);
        assert_eq!(handler.strategy_name(), "drop_newest");

        let handler = OverflowHandlerFactory::create(OverflowStrategy::Block);
        assert_eq!(handler.strategy_name(), "block");

        let handler = OverflowHandlerFactory::create(OverflowStrategy::Reject);
        assert_eq!(handler.strategy_name(), "reject");
    }
}
