// ABOUTME: EventBus implementation with async pub/sub and pattern matching
// ABOUTME: Provides high-performance event routing with flow control integration

use crate::flow_controller::{FlowController, FlowControllerConfig};
use crate::handler::AsyncEventHandler;
use crate::pattern::{EventPattern, PatternMatcher};
use crate::storage_adapter::{EventPersistenceManager, EventStorage, PersistenceConfig};
use crate::universal_event::UniversalEvent;
use dashmap::DashMap;
use llmspell_storage::StorageBackend;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    /// Pattern-based subscriptions
    subscriptions: Arc<DashMap<String, Vec<Subscription>>>,
    /// Flow controller for backpressure
    flow_controller: Arc<FlowController>,
    /// Broadcast channel for all events
    broadcast_tx: broadcast::Sender<UniversalEvent>,
    /// Pattern matcher
    pattern_matcher: PatternMatcher,
    /// Optional event persistence
    persistence_manager: Option<Arc<tokio::sync::Mutex<Box<dyn EventPersistenceManagerTrait>>>>,
}

/// Trait for type-erased persistence manager
#[async_trait::async_trait]
trait EventPersistenceManagerTrait: Send + Sync {
    async fn maybe_store_event(&self, event: &UniversalEvent) -> anyhow::Result<bool>;
    fn storage(&self) -> &dyn EventStorage;
}

#[async_trait::async_trait]
impl<B: StorageBackend + 'static> EventPersistenceManagerTrait for EventPersistenceManager<B> {
    async fn maybe_store_event(&self, event: &UniversalEvent) -> anyhow::Result<bool> {
        self.maybe_store_event(event).await
    }

    fn storage(&self) -> &dyn EventStorage {
        self.storage()
    }
}

/// Individual subscription
#[derive(Debug)]
struct Subscription {
    #[allow(dead_code)] // Used for future unsubscribe functionality
    id: Uuid,
    #[allow(dead_code)] // Used for debugging and future pattern optimization
    pattern: EventPattern,
    sender: mpsc::UnboundedSender<UniversalEvent>,
}

impl EventBus {
    /// Create a new event bus with default configuration
    pub fn new() -> Self {
        Self::with_config(FlowControllerConfig::default())
    }

    /// Create an event bus with custom flow control configuration
    pub fn with_config(config: FlowControllerConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(10000);

        Self {
            subscriptions: Arc::new(DashMap::new()),
            flow_controller: Arc::new(FlowController::new(config)),
            broadcast_tx,
            pattern_matcher: PatternMatcher::new(),
            persistence_manager: None,
        }
    }

    /// Create an event bus with persistence using any storage backend
    pub fn with_persistence<B: StorageBackend + 'static>(
        flow_config: FlowControllerConfig,
        storage_adapter: crate::storage_adapter::EventStorageAdapter<B>,
        persistence_config: PersistenceConfig,
    ) -> Self {
        let (broadcast_tx, _) = broadcast::channel(10000);
        let persistence_manager = EventPersistenceManager::new(storage_adapter, persistence_config);

        Self {
            subscriptions: Arc::new(DashMap::new()),
            flow_controller: Arc::new(FlowController::new(flow_config)),
            broadcast_tx,
            pattern_matcher: PatternMatcher::new(),
            persistence_manager: Some(Arc::new(tokio::sync::Mutex::new(
                Box::new(persistence_manager) as Box<dyn EventPersistenceManagerTrait>,
            ))),
        }
    }

    /// Publish an event to the bus
    pub async fn publish(&self, event: UniversalEvent) -> Result<(), PublishError> {
        // Check rate limiting
        if !self.flow_controller.can_process(&event).await {
            return Err(PublishError::RateLimited);
        }

        // Handle potential overflow
        let overflow_result = self.flow_controller.handle_overflow(event.clone()).await;
        match overflow_result {
            crate::overflow::OverflowResult::Accepted => {
                // Continue with publish
            }
            crate::overflow::OverflowResult::Dropped { reason } => {
                return Err(PublishError::Dropped { reason });
            }
            crate::overflow::OverflowResult::Rejected { reason } => {
                return Err(PublishError::Rejected { reason });
            }
            crate::overflow::OverflowResult::Blocked => {
                return Err(PublishError::Blocked);
            }
        }

        // Send to broadcast channel
        if self.broadcast_tx.send(event.clone()).is_err() {
            debug!("No broadcast receivers for event: {}", event.event_type);
        }

        // Store event in persistence if configured
        if let Some(persistence_manager) = &self.persistence_manager {
            let manager = persistence_manager.lock().await;
            if let Err(e) = manager.maybe_store_event(&event).await {
                error!("Failed to persist event: {}", e);
                // Don't fail the publish if persistence fails
            }
        }

        // Route to pattern-matched subscriptions
        self.route_event(event).await;

        Ok(())
    }

    /// Subscribe to events matching a pattern
    pub async fn subscribe(
        &self,
        pattern: &str,
    ) -> Result<mpsc::UnboundedReceiver<UniversalEvent>, SubscribeError> {
        let event_pattern = EventPattern::new(pattern)?;
        let (tx, rx) = mpsc::unbounded_channel();

        let subscription = Subscription {
            id: Uuid::new_v4(),
            pattern: event_pattern.clone(),
            sender: tx,
        };

        // Add to subscriptions
        self.subscriptions
            .entry(pattern.to_string())
            .or_default()
            .push(subscription);

        info!("New subscription created for pattern: {}", pattern);
        Ok(rx)
    }

    /// Subscribe with a custom event handler
    pub async fn subscribe_with_handler<H>(
        &self,
        pattern: &str,
        handler: H,
    ) -> Result<Uuid, SubscribeError>
    where
        H: AsyncEventHandler + Send + 'static,
    {
        let mut receiver = self.subscribe(pattern).await?;
        let subscription_id = Uuid::new_v4();

        // Spawn task to handle events
        let handler = Arc::new(handler);
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Err(e) = handler.handle_event(event).await {
                    error!("Event handler error: {}", e);
                }
            }
        });

        Ok(subscription_id)
    }

    /// Get a broadcast receiver for all events
    pub fn subscribe_all(&self) -> broadcast::Receiver<UniversalEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Route an event to matching subscriptions
    async fn route_event(&self, event: UniversalEvent) {
        let mut matched_count = 0;

        for subscription_entry in self.subscriptions.iter() {
            let pattern = subscription_entry.key();
            let subscriptions = subscription_entry.value();

            if self.pattern_matcher.matches(&event.event_type, pattern) {
                for subscription in subscriptions {
                    if subscription.sender.send(event.clone()).is_err() {
                        debug!("Subscription receiver dropped for pattern: {}", pattern);
                    } else {
                        matched_count += 1;
                    }
                }
            }
        }

        if matched_count == 0 {
            debug!("No subscribers for event: {}", event.event_type);
        } else {
            debug!(
                "Event {} routed to {} subscribers",
                event.event_type, matched_count
            );
        }
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.flow_controller.buffer_size()
    }

    /// Get flow statistics
    pub fn get_stats(&self) -> crate::flow_controller::FlowStats {
        self.flow_controller.get_stats()
    }

    /// Get number of active subscriptions
    pub fn subscription_count(&self) -> usize {
        self.subscriptions
            .iter()
            .map(|entry| entry.value().len())
            .sum()
    }

    /// Get persisted events by pattern (if persistence is enabled)
    pub async fn get_persisted_events(
        &self,
        pattern: &str,
    ) -> Result<Vec<UniversalEvent>, anyhow::Error> {
        if let Some(persistence_manager) = &self.persistence_manager {
            let manager = persistence_manager.lock().await;
            manager.storage().get_events_by_pattern(pattern).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get persisted events by correlation ID (if persistence is enabled)
    pub async fn get_events_by_correlation_id(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<UniversalEvent>, anyhow::Error> {
        if let Some(persistence_manager) = &self.persistence_manager {
            let manager = persistence_manager.lock().await;
            manager
                .storage()
                .get_events_by_correlation_id(correlation_id)
                .await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get storage statistics (if persistence is enabled)
    pub async fn get_storage_stats(
        &self,
    ) -> Result<Option<crate::storage_adapter::StorageStats>, anyhow::Error> {
        if let Some(persistence_manager) = &self.persistence_manager {
            let manager = persistence_manager.lock().await;
            let stats = manager.storage().get_storage_stats().await?;
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for EventBus
pub struct EventBusBuilder {
    flow_config: FlowControllerConfig,
    broadcast_capacity: usize,
    persistence_config: Option<(Box<dyn EventPersistenceManagerTrait>, PersistenceConfig)>,
}

impl EventBusBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            flow_config: FlowControllerConfig::default(),
            broadcast_capacity: 10000,
            persistence_config: None,
        }
    }

    /// Set flow controller configuration
    pub fn with_flow_config(mut self, config: FlowControllerConfig) -> Self {
        self.flow_config = config;
        self
    }

    /// Set broadcast channel capacity
    pub fn with_broadcast_capacity(mut self, capacity: usize) -> Self {
        self.broadcast_capacity = capacity;
        self
    }

    /// Set persistence configuration with storage backend
    pub fn with_storage_persistence<B: StorageBackend + 'static>(
        mut self,
        storage_adapter: crate::storage_adapter::EventStorageAdapter<B>,
        config: PersistenceConfig,
    ) -> Self {
        let manager = EventPersistenceManager::new(storage_adapter, config.clone());
        self.persistence_config = Some((Box::new(manager), config));
        self
    }

    /// Build the event bus
    pub fn build(self) -> EventBus {
        if let Some((manager, _)) = self.persistence_config {
            let (broadcast_tx, _) = broadcast::channel(self.broadcast_capacity);
            EventBus {
                subscriptions: Arc::new(DashMap::new()),
                flow_controller: Arc::new(FlowController::new(self.flow_config)),
                broadcast_tx,
                pattern_matcher: PatternMatcher::new(),
                persistence_manager: Some(Arc::new(tokio::sync::Mutex::new(manager))),
            }
        } else {
            EventBus::with_config(self.flow_config)
        }
    }
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when publishing events
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Event was rate limited")]
    RateLimited,
    #[error("Event was dropped: {reason}")]
    Dropped { reason: String },
    #[error("Event was rejected: {reason}")]
    Rejected { reason: String },
    #[error("Publisher is blocked")]
    Blocked,
}

/// Errors that can occur when subscribing
#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    #[error("Subscription limit reached")]
    LimitReached,
}

impl From<String> for SubscribeError {
    fn from(error: String) -> Self {
        SubscribeError::InvalidPattern(error)
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "event")]
mod tests {
    use super::*;
    use crate::universal_event::{Language, UniversalEvent};
    use serde_json::Value;

    fn create_test_event(event_type: &str) -> UniversalEvent {
        UniversalEvent::new(event_type, Value::Null, Language::Rust)
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_basic_pub_sub() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe("test.*").await.unwrap();

        let event = create_test_event("test.event");
        bus.publish(event.clone()).await.unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.event_type, "test.event");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_pattern_matching() {
        let bus = EventBus::new();
        let mut system_receiver = bus.subscribe("system.*").await.unwrap();
        let mut agent_receiver = bus.subscribe("agent.*").await.unwrap();

        // Publish system event
        let system_event = create_test_event("system.startup");
        bus.publish(system_event).await.unwrap();

        // Publish agent event
        let agent_event = create_test_event("agent.created");
        bus.publish(agent_event).await.unwrap();

        // System receiver should get system event
        let received = system_receiver.recv().await.unwrap();
        assert_eq!(received.event_type, "system.startup");

        // Agent receiver should get agent event
        let received = agent_receiver.recv().await.unwrap();
        assert_eq!(received.event_type, "agent.created");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_broadcast_all() {
        let bus = EventBus::new();
        let mut broadcast_receiver = bus.subscribe_all();

        let event = create_test_event("any.event");
        bus.publish(event.clone()).await.unwrap();

        let received = broadcast_receiver.recv().await.unwrap();
        assert_eq!(received.event_type, "any.event");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut receiver1 = bus.subscribe("multi.*").await.unwrap();
        let mut receiver2 = bus.subscribe("multi.*").await.unwrap();

        let event = create_test_event("multi.test");
        bus.publish(event).await.unwrap();

        // Both receivers should get the event
        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        assert_eq!(received1.event_type, "multi.test");
        assert_eq!(received2.event_type, "multi.test");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_statistics() {
        let bus = EventBus::new();

        let event = create_test_event("stats.test");
        bus.publish(event).await.unwrap();

        let stats = bus.get_stats();
        assert!(stats.events_processed > 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_builder() {
        let bus = EventBusBuilder::new().with_broadcast_capacity(5000).build();

        assert_eq!(bus.subscription_count(), 0);
    }
}
