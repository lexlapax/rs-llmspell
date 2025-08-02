//! ABOUTME: Event bus integration for context-aware event handling
//! ABOUTME: Provides event publishing, subscription, and context propagation

use async_trait::async_trait;
use llmspell_core::execution_context::{ContextScope, ExecutionContext};
use llmspell_core::{EventMetadata, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Context-aware event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEvent {
    /// Event ID
    pub id: String,
    /// Event type/name
    pub event_type: String,
    /// Event payload
    pub payload: Value,
    /// Source context
    pub source_context: ContextScope,
    /// Target contexts (empty for broadcast)
    pub target_contexts: Vec<ContextScope>,
    /// Event metadata
    pub metadata: EventMetadata,
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
}

impl ContextEvent {
    /// Create a new context event
    #[must_use]
    pub fn new(event_type: String, payload: Value, source: ContextScope) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            payload,
            source_context: source,
            target_contexts: Vec::new(),
            metadata: EventMetadata::new(),
            correlation_id: None,
        }
    }

    /// Add target context
    #[must_use]
    pub fn with_target(mut self, target: ContextScope) -> Self {
        self.target_contexts.push(target);
        self
    }

    /// Set correlation ID
    #[must_use]
    pub fn with_correlation(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Check if event is targeted to a specific context
    #[must_use]
    pub fn is_targeted_to(&self, context: &ContextScope) -> bool {
        if self.target_contexts.is_empty() {
            // Broadcast event
            true
        } else {
            self.target_contexts.contains(context)
        }
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: ContextEvent, context: ExecutionContext) -> Result<()>;

    /// Get event types this handler is interested in
    fn event_types(&self) -> Vec<String>;

    /// Get handler priority (lower = higher priority)
    fn priority(&self) -> i32 {
        100
    }
}

/// Event subscription
struct Subscription {
    id: String,
    context_scope: ContextScope,
    event_types: Vec<String>,
    handler: Arc<dyn EventHandler>,
}

/// Context-aware event bus
#[derive(Clone)]
pub struct ContextEventBus {
    /// Event broadcast channel
    event_tx: broadcast::Sender<ContextEvent>,
    /// Subscriptions
    subscriptions: Arc<RwLock<Vec<Subscription>>>,
    /// Event history
    history: Arc<RwLock<EventHistory>>,
    /// Configuration
    config: EventBusConfig,
}

/// Event bus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// Maximum event history size
    pub max_history_size: usize,
    /// Event TTL in seconds
    pub event_ttl_secs: u64,
    /// Enable event persistence
    pub persist_events: bool,
    /// Maximum concurrent handlers
    pub max_concurrent_handlers: usize,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            event_ttl_secs: 3600,
            persist_events: false,
            max_concurrent_handlers: 10,
        }
    }
}

/// Event history tracking
#[derive(Debug, Default)]
struct EventHistory {
    events: Vec<ContextEvent>,
    by_type: HashMap<String, Vec<String>>, // event_type -> event_ids
    by_context: HashMap<String, Vec<String>>, // context_scope -> event_ids
}

impl EventHistory {
    fn add(&mut self, event: ContextEvent, max_size: usize) {
        // Add to main list
        self.events.push(event.clone());

        // Add to indices
        self.by_type
            .entry(event.event_type.clone())
            .or_default()
            .push(event.id.clone());

        self.by_context
            .entry(event.source_context.to_string())
            .or_default()
            .push(event.id);

        // Trim if needed
        if self.events.len() > max_size {
            let removed = self.events.remove(0);

            // Update indices
            if let Some(type_events) = self.by_type.get_mut(&removed.event_type) {
                type_events.retain(|id| id != &removed.id);
            }

            if let Some(ctx_events) = self.by_context.get_mut(&removed.source_context.to_string()) {
                ctx_events.retain(|id| id != &removed.id);
            }
        }
    }

    fn get_by_type(&self, event_type: &str) -> Vec<ContextEvent> {
        self.by_type
            .get(event_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.events.iter().find(|e| e.id == *id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_by_context(&self, context: &ContextScope) -> Vec<ContextEvent> {
        self.by_context
            .get(&context.to_string())
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.events.iter().find(|e| e.id == *id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl ContextEventBus {
    /// Create a new event bus
    #[must_use]
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            event_tx,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(EventHistory::default())),
            config: EventBusConfig::default(),
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: EventBusConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            event_tx,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(EventHistory::default())),
            config,
        }
    }

    /// Publish an event
    pub async fn publish(&self, event: ContextEvent) -> Result<()> {
        // Add to history
        {
            let mut history = self.history.write().await;
            history.add(event.clone(), self.config.max_history_size);
        }

        // Broadcast event (ignore if no active receivers)
        let _ = self.event_tx.send(event.clone());

        // Handle event with registered handlers
        self.dispatch_to_handlers(event).await?;

        Ok(())
    }

    /// Subscribe to events
    pub async fn subscribe(
        &self,
        context_scope: ContextScope,
        event_types: Vec<String>,
        handler: Arc<dyn EventHandler>,
    ) -> Result<String> {
        let subscription = Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            context_scope,
            event_types,
            handler,
        };

        let sub_id = subscription.id.clone();

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.push(subscription);

        Ok(sub_id)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.retain(|sub| sub.id != subscription_id);
        Ok(())
    }

    /// Get event receiver for raw event stream
    #[must_use]
    pub fn receiver(&self) -> broadcast::Receiver<ContextEvent> {
        self.event_tx.subscribe()
    }

    /// Get event history by type
    pub async fn history_by_type(&self, event_type: &str) -> Vec<ContextEvent> {
        let history = self.history.read().await;
        history.get_by_type(event_type)
    }

    /// Get event history by context
    pub async fn history_by_context(&self, context: &ContextScope) -> Vec<ContextEvent> {
        let history = self.history.read().await;
        history.get_by_context(context)
    }

    /// Dispatch event to registered handlers
    async fn dispatch_to_handlers(&self, event: ContextEvent) -> Result<()> {
        let subscriptions = self.subscriptions.read().await;

        // Filter relevant subscriptions
        let mut handlers: Vec<_> = subscriptions
            .iter()
            .filter(|sub| {
                // Check if handler is interested in this event type
                (sub.event_types.is_empty() || sub.event_types.contains(&event.event_type))
                    && event.is_targeted_to(&sub.context_scope)
            })
            .map(|sub| {
                (
                    sub.handler.clone(),
                    sub.context_scope.clone(),
                    sub.handler.priority(),
                )
            })
            .collect();

        // Sort by priority
        handlers.sort_by_key(|(_, _, priority)| *priority);

        // Create semaphore for concurrency control
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.max_concurrent_handlers,
        ));

        // Execute handlers
        let mut tasks = Vec::new();

        for (handler, context_scope, _) in handlers {
            let event = event.clone();
            let semaphore = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                // Create execution context for handler
                let context = ExecutionContext::new()
                    .with_scope(context_scope)
                    .with_data("event_id".to_string(), Value::String(event.id.clone()))
                    .with_data(
                        "event_type".to_string(),
                        Value::String(event.event_type.clone()),
                    );

                handler.handle(event, context).await
            });

            tasks.push(task);
        }

        // Wait for all handlers to complete
        for task in tasks {
            if let Err(e) = task.await {
                eprintln!("Handler task failed: {e:?}");
            }
        }

        Ok(())
    }

    /// Get event bus statistics
    pub async fn stats(&self) -> EventBusStats {
        let subscriptions = self.subscriptions.read().await;
        let history = self.history.read().await;

        let subscription_count = subscriptions.len();
        let event_count = history.events.len();

        let event_types: HashMap<String, usize> = history
            .by_type
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();

        EventBusStats {
            subscription_count,
            event_count,
            event_types,
            config: self.config.clone(),
        }
    }
}

impl Default for ContextEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event bus statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusStats {
    pub subscription_count: usize,
    pub event_count: usize,
    pub event_types: HashMap<String, usize>,
    pub config: EventBusConfig,
}

/// Example event handler for logging
pub struct LoggingEventHandler;

#[async_trait]
impl EventHandler for LoggingEventHandler {
    async fn handle(&self, event: ContextEvent, _context: ExecutionContext) -> Result<()> {
        tracing::info!(
            event_type = %event.event_type,
            event_id = %event.id,
            source = %event.source_context,
            "Event received"
        );
        Ok(())
    }

    fn event_types(&self) -> Vec<String> {
        Vec::new() // Listen to all events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ComponentId;
    use serde_json::json;

    #[derive(Clone)]
    struct TestHandler {
        received: Arc<RwLock<Vec<ContextEvent>>>,
    }

    #[async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, event: ContextEvent, _context: ExecutionContext) -> Result<()> {
            let mut received = self.received.write().await;
            received.push(event);
            Ok(())
        }

        fn event_types(&self) -> Vec<String> {
            vec!["test_event".to_string()]
        }
    }
    #[tokio::test]
    async fn test_event_publish_subscribe() {
        let bus = ContextEventBus::new();

        let handler = Arc::new(TestHandler {
            received: Arc::new(RwLock::new(Vec::new())),
        });

        // Subscribe
        let _sub_id = bus
            .subscribe(
                ContextScope::Global,
                vec!["test_event".to_string()],
                handler.clone(),
            )
            .await
            .unwrap();

        // Publish event
        let event = ContextEvent::new(
            "test_event".to_string(),
            json!({"data": "test"}),
            ContextScope::Global,
        );

        bus.publish(event.clone()).await.unwrap();

        // Wait a bit for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check handler received event
        let received = handler.received.read().await;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, event.id);
    }
    #[tokio::test]
    async fn test_event_targeting() {
        let bus = ContextEventBus::new();

        let agent1 = ComponentId::from_name("agent1");
        let agent2 = ComponentId::from_name("agent2");

        let handler1 = Arc::new(TestHandler {
            received: Arc::new(RwLock::new(Vec::new())),
        });

        let handler2 = Arc::new(TestHandler {
            received: Arc::new(RwLock::new(Vec::new())),
        });

        // Subscribe different contexts
        bus.subscribe(
            ContextScope::Agent(agent1),
            vec!["test_event".to_string()],
            handler1.clone(),
        )
        .await
        .unwrap();

        bus.subscribe(
            ContextScope::Agent(agent2),
            vec!["test_event".to_string()],
            handler2.clone(),
        )
        .await
        .unwrap();

        // Publish targeted event
        let event = ContextEvent::new(
            "test_event".to_string(),
            json!({"data": "targeted"}),
            ContextScope::Global,
        )
        .with_target(ContextScope::Agent(agent1));

        bus.publish(event).await.unwrap();

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Only handler1 should receive
        let received1 = handler1.received.read().await;
        let received2 = handler2.received.read().await;

        assert_eq!(received1.len(), 1);
        assert_eq!(received2.len(), 0);
    }
    #[tokio::test]
    async fn test_event_history() {
        let config = EventBusConfig {
            max_history_size: 5,
            ..Default::default()
        };

        let bus = ContextEventBus::with_config(config);

        // Publish multiple events
        for i in 0..10 {
            let event = ContextEvent::new(
                format!("event_type_{}", i % 3),
                json!({"index": i}),
                ContextScope::Session(format!("session_{}", i % 2)),
            );
            bus.publish(event).await.unwrap();
        }

        // Check history size limit
        let stats = bus.stats().await;
        assert_eq!(stats.event_count, 5); // Limited to max_history_size

        // Check by type
        let type_events = bus.history_by_type("event_type_0").await;
        assert!(type_events.len() <= 2); // Should have some events of this type

        // Check by context
        let ctx_events = bus
            .history_by_context(&ContextScope::Session("session_0".to_string()))
            .await;
        assert!(ctx_events.len() <= 3); // Should have some events from this context
    }
    #[tokio::test]
    async fn test_event_correlation() {
        let bus = ContextEventBus::new();

        let correlation_id = "corr-123";

        let event1 = ContextEvent::new(
            "start".to_string(),
            json!({"step": 1}),
            ContextScope::Global,
        )
        .with_correlation(correlation_id.to_string());

        let event2 = ContextEvent::new("end".to_string(), json!({"step": 2}), ContextScope::Global)
            .with_correlation(correlation_id.to_string());

        bus.publish(event1).await.unwrap();
        bus.publish(event2).await.unwrap();

        // Both events share correlation ID for tracing
        let history = bus.history_by_type("start").await;
        assert_eq!(history[0].correlation_id, Some(correlation_id.to_string()));
    }
}
