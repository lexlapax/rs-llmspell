//! ABOUTME: Lifecycle event system for agent state machine transitions and monitoring
//! ABOUTME: Provides event-driven notifications for agent lifecycle changes with hooks integration

use super::state_machine::{AgentState, StateTransition};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Lifecycle event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Agent state changed
    StateChanged,
    /// Agent initialization started
    InitializationStarted,
    /// Agent initialization completed
    InitializationCompleted,
    /// Agent execution started
    ExecutionStarted,
    /// Agent execution completed
    ExecutionCompleted,
    /// Agent paused
    AgentPaused,
    /// Agent resumed
    AgentResumed,
    /// Agent termination started
    TerminationStarted,
    /// Agent termination completed
    TerminationCompleted,
    /// Agent error occurred
    ErrorOccurred,
    /// Agent recovery started
    RecoveryStarted,
    /// Agent recovery completed
    RecoveryCompleted,
    /// Agent health check
    HealthCheck,
    /// Resource allocation
    ResourceAllocated,
    /// Resource deallocation
    ResourceDeallocated,
}

/// Lifecycle event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: LifecycleEventType,
    /// Agent ID that triggered the event
    pub agent_id: String,
    /// Timestamp when event occurred
    pub timestamp: SystemTime,
    /// Event payload data
    pub data: LifecycleEventData,
    /// Event metadata
    pub metadata: HashMap<String, String>,
    /// Event source (component that triggered the event)
    pub source: String,
}

impl LifecycleEvent {
    /// Create new lifecycle event
    #[must_use]
    pub fn new(
        event_type: LifecycleEventType,
        agent_id: String,
        data: LifecycleEventData,
        source: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            agent_id,
            timestamp: SystemTime::now(),
            data,
            metadata: HashMap::new(),
            source,
        }
    }

    /// Add metadata to event
    #[must_use]
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Get event age
    #[must_use]
    pub fn age(&self) -> Duration {
        self.timestamp
            .elapsed()
            .unwrap_or_else(|_| Duration::from_secs(0))
    }
}

/// Event payload data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEventData {
    /// State transition data
    StateTransition {
        from: AgentState,
        to: AgentState,
        duration: Option<Duration>,
        reason: Option<String>,
    },
    /// Error information
    Error {
        message: String,
        error_type: String,
        recovery_possible: bool,
    },
    /// Health status
    Health {
        is_healthy: bool,
        status: String,
        metrics: HashMap<String, String>,
    },
    /// Resource information
    Resource {
        resource_type: String,
        resource_id: String,
        amount: Option<u64>,
        status: String,
    },
    /// Generic event data
    Generic {
        message: String,
        details: HashMap<String, String>,
    },
}

/// Event listener trait
#[async_trait]
pub trait LifecycleEventListener: Send + Sync {
    /// Handle lifecycle event
    async fn handle_event(&self, event: &LifecycleEvent) -> Result<()>;

    /// Check if listener is interested in event type
    fn interested_in(&self, event_type: &LifecycleEventType) -> bool;

    /// Get listener metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Event subscription configuration
#[derive(Clone)]
pub struct EventSubscription {
    pub id: String,
    pub agent_id: Option<String>, // None means all agents
    pub event_types: Vec<LifecycleEventType>,
    pub listener: Arc<dyn LifecycleEventListener>,
    pub active: bool,
}

impl EventSubscription {
    pub fn new(listener: Arc<dyn LifecycleEventListener>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id: None,
            event_types: vec![],
            listener,
            active: true,
        }
    }

    #[must_use]
    pub fn for_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    #[must_use]
    pub fn for_event_types(mut self, event_types: Vec<LifecycleEventType>) -> Self {
        self.event_types = event_types;
        self
    }

    #[must_use]
    pub fn matches(&self, event: &LifecycleEvent) -> bool {
        if !self.active {
            return false;
        }

        // Check agent filter
        if let Some(ref agent_id) = self.agent_id {
            if agent_id != &event.agent_id {
                return false;
            }
        }

        // Check event type filter
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }

        // Check listener interest
        self.listener.interested_in(&event.event_type)
    }
}

/// Lifecycle event system
pub struct LifecycleEventSystem {
    /// Event broadcaster
    broadcaster: broadcast::Sender<LifecycleEvent>,
    /// Event subscriptions
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
    /// Event history (limited size)
    event_history: Arc<Mutex<Vec<LifecycleEvent>>>,
    /// Event statistics
    event_stats: Arc<Mutex<EventStatistics>>,
    /// Configuration
    config: EventSystemConfig,
}

/// Event system configuration
#[derive(Debug, Clone)]
pub struct EventSystemConfig {
    /// Maximum number of events in history
    pub max_history_size: usize,
    /// Enable event persistence
    pub enable_persistence: bool,
    /// Event processing timeout
    pub processing_timeout: Duration,
    /// Maximum number of concurrent listeners
    pub max_concurrent_listeners: usize,
    /// Enable detailed logging
    pub enable_logging: bool,
}

impl Default for EventSystemConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            enable_persistence: false,
            processing_timeout: Duration::from_secs(5),
            max_concurrent_listeners: 100,
            enable_logging: true,
        }
    }
}

/// Event statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventStatistics {
    pub total_events: u64,
    pub events_by_type: HashMap<LifecycleEventType, u64>,
    pub events_by_agent: HashMap<String, u64>,
    pub error_events: u64,
    pub processing_failures: u64,
    pub average_processing_time: Duration,
}

impl Default for LifecycleEventSystem {
    fn default() -> Self {
        Self::new(EventSystemConfig::default())
    }
}

impl LifecycleEventSystem {
    /// Create new event system
    #[must_use]
    pub fn new(config: EventSystemConfig) -> Self {
        let (broadcaster, _) = broadcast::channel(1000);

        Self {
            broadcaster,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(Mutex::new(Vec::new())),
            event_stats: Arc::new(Mutex::new(EventStatistics::default())),
            config,
        }
    }

    /// Subscribe to specific event types with a closure
    pub async fn subscribe_filtered<F>(
        &self,
        _name: &str,
        handler: F,
        event_types: Vec<LifecycleEventType>,
    ) -> String
    where
        F: Fn(&LifecycleEvent) + Send + Sync + 'static,
    {
        struct ClosureListener<F> {
            handler: F,
            event_types: Vec<LifecycleEventType>,
        }

        #[async_trait]
        impl<F> LifecycleEventListener for ClosureListener<F>
        where
            F: Fn(&LifecycleEvent) + Send + Sync,
        {
            async fn handle_event(&self, event: &LifecycleEvent) -> Result<()> {
                (self.handler)(event);
                Ok(())
            }

            fn interested_in(&self, event_type: &LifecycleEventType) -> bool {
                self.event_types.is_empty() || self.event_types.contains(event_type)
            }
        }

        let listener = Arc::new(ClosureListener {
            handler,
            event_types: event_types.clone(),
        });

        let subscription = EventSubscription::new(listener).for_event_types(event_types);

        self.subscribe(subscription).await
    }

    /// Subscribe to events
    pub async fn subscribe(&self, subscription: EventSubscription) -> String {
        let subscription_id = subscription.id.clone();
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        if self.config.enable_logging {
            debug!("New event subscription registered: {}", subscription_id);
        }

        subscription_id
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        if subscriptions.remove(subscription_id).is_some() {
            if self.config.enable_logging {
                debug!("Event subscription removed: {}", subscription_id);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Subscription not found: {}",
                subscription_id
            ))
        }
    }

    /// Emit lifecycle event
    pub async fn emit(&self, event: LifecycleEvent) -> Result<()> {
        let start_time = Instant::now();

        if self.config.enable_logging {
            debug!(
                "Emitting event {:?} for agent {}",
                event.event_type, event.agent_id
            );
        }

        // Update statistics
        {
            let mut stats = self.event_stats.lock().await;
            stats.total_events += 1;
            *stats
                .events_by_type
                .entry(event.event_type.clone())
                .or_insert(0) += 1;
            *stats
                .events_by_agent
                .entry(event.agent_id.clone())
                .or_insert(0) += 1;

            if matches!(event.event_type, LifecycleEventType::ErrorOccurred) {
                stats.error_events += 1;
            }
        }

        // Add to history
        {
            let mut history = self.event_history.lock().await;
            history.push(event.clone());

            // Trim history if needed
            if history.len() > self.config.max_history_size {
                history.remove(0);
            }
        }

        // Broadcast event
        if let Err(e) = self.broadcaster.send(event.clone()) {
            warn!("Failed to broadcast event: {}", e);
        }

        // Process subscriptions
        self.process_subscriptions(&event).await?;

        // Update processing time statistics
        {
            let mut stats = self.event_stats.lock().await;
            let processing_time = start_time.elapsed();
            stats.average_processing_time = if stats.total_events == 1 {
                processing_time
            } else {
                (stats.average_processing_time
                    * u32::try_from(stats.total_events - 1).unwrap_or(u32::MAX)
                    + processing_time)
                    / u32::try_from(stats.total_events).unwrap_or(1)
            };
        }

        Ok(())
    }

    /// Process event subscriptions
    async fn process_subscriptions(&self, event: &LifecycleEvent) -> Result<()> {
        let subscriptions = self.subscriptions.read().await;
        let matching_subscriptions: Vec<_> = subscriptions
            .values()
            .filter(|sub| sub.matches(event))
            .collect();

        if matching_subscriptions.is_empty() {
            return Ok(());
        }

        // Process listeners concurrently with timeout
        let handles: Vec<_> = matching_subscriptions
            .into_iter()
            .map(|subscription| {
                let listener = subscription.listener.clone();
                let event = event.clone();
                let timeout = self.config.processing_timeout;

                tokio::spawn(async move {
                    match tokio::time::timeout(timeout, listener.handle_event(&event)).await {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(e)) => {
                            error!("Event listener failed: {}", e);
                            Err(e)
                        }
                        Err(_) => {
                            error!("Event listener timed out");
                            Err(anyhow::anyhow!("Event processing timeout"))
                        }
                    }
                })
            })
            .collect();

        // Wait for all listeners to complete
        let mut processing_failures = 0;
        for handle in handles {
            if handle
                .await
                .unwrap_or_else(|_| Err(anyhow::anyhow!("Task failed")))
                .is_err()
            {
                processing_failures += 1;
            }
        }

        // Update failure statistics
        if processing_failures > 0 {
            let mut stats = self.event_stats.lock().await;
            stats.processing_failures += processing_failures;
        }

        Ok(())
    }

    /// Emit state transition event
    pub async fn emit_state_transition(
        &self,
        agent_id: String,
        transition: StateTransition,
    ) -> Result<()> {
        let event = LifecycleEvent::new(
            LifecycleEventType::StateChanged,
            agent_id,
            LifecycleEventData::StateTransition {
                from: transition.from,
                to: transition.to,
                duration: transition.duration,
                reason: transition.reason,
            },
            "state_machine".to_string(),
        );

        self.emit(event).await
    }

    /// Emit error event
    pub async fn emit_error(
        &self,
        agent_id: String,
        error_message: String,
        recovery_possible: bool,
    ) -> Result<()> {
        let event = LifecycleEvent::new(
            LifecycleEventType::ErrorOccurred,
            agent_id,
            LifecycleEventData::Error {
                message: error_message.clone(),
                error_type: "agent_error".to_string(),
                recovery_possible,
            },
            "agent".to_string(),
        )
        .with_metadata("severity", "error");

        self.emit(event).await
    }

    /// Emit health check event
    pub async fn emit_health_check(
        &self,
        agent_id: String,
        is_healthy: bool,
        status: String,
    ) -> Result<()> {
        let event = LifecycleEvent::new(
            LifecycleEventType::HealthCheck,
            agent_id,
            LifecycleEventData::Health {
                is_healthy,
                status,
                metrics: HashMap::new(),
            },
            "health_monitor".to_string(),
        );

        self.emit(event).await
    }

    /// Get event receiver for custom processing
    #[must_use]
    pub fn subscribe_to_broadcast(&self) -> broadcast::Receiver<LifecycleEvent> {
        self.broadcaster.subscribe()
    }

    /// Get event history
    pub async fn get_event_history(&self) -> Vec<LifecycleEvent> {
        let history = self.event_history.lock().await;
        history.clone()
    }

    /// Get event statistics
    pub async fn get_statistics(&self) -> EventStatistics {
        let stats = self.event_stats.lock().await;
        stats.clone()
    }

    /// Get events for specific agent
    pub async fn get_agent_events(&self, agent_id: &str) -> Vec<LifecycleEvent> {
        let history = self.event_history.lock().await;
        history
            .iter()
            .filter(|event| event.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub async fn get_events_by_type(&self, event_type: LifecycleEventType) -> Vec<LifecycleEvent> {
        let history = self.event_history.lock().await;
        history
            .iter()
            .filter(|event| event.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Clear event history
    pub async fn clear_history(&self) {
        let mut history = self.event_history.lock().await;
        history.clear();

        if self.config.enable_logging {
            debug!("Event history cleared");
        }
    }

    /// Get active subscriptions count
    pub async fn get_subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.len()
    }
}

/// Default logging event listener
pub struct LoggingEventListener {
    log_level: tracing::Level,
}

impl Default for LoggingEventListener {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingEventListener {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            log_level: tracing::Level::INFO,
        }
    }

    #[must_use]
    pub const fn with_level(mut self, level: tracing::Level) -> Self {
        self.log_level = level;
        self
    }
}

#[async_trait]
impl LifecycleEventListener for LoggingEventListener {
    async fn handle_event(&self, event: &LifecycleEvent) -> Result<()> {
        let message = match &event.data {
            LifecycleEventData::StateTransition { from, to, .. } => {
                format!(
                    "Agent {} transitioned from {:?} to {:?}",
                    event.agent_id, from, to
                )
            }
            LifecycleEventData::Error { message, .. } => {
                format!("Agent {} error: {}", event.agent_id, message)
            }
            LifecycleEventData::Health {
                is_healthy, status, ..
            } => {
                format!(
                    "Agent {} health: {} ({})",
                    event.agent_id,
                    if *is_healthy { "healthy" } else { "unhealthy" },
                    status
                )
            }
            LifecycleEventData::Resource {
                resource_type,
                status,
                ..
            } => {
                format!(
                    "Agent {} resource {}: {}",
                    event.agent_id, resource_type, status
                )
            }
            LifecycleEventData::Generic { message, .. } => {
                format!("Agent {} event: {}", event.agent_id, message)
            }
        };

        match self.log_level {
            tracing::Level::DEBUG => debug!("{}", message),
            tracing::Level::INFO => info!("{}", message),
            tracing::Level::WARN => warn!("{}", message),
            tracing::Level::ERROR => error!("{}", message),
            _ => info!("{}", message),
        }

        Ok(())
    }

    fn interested_in(&self, _event_type: &LifecycleEventType) -> bool {
        true // Log all events
    }
}

/// Metrics collection event listener
pub struct MetricsEventListener {
    metrics: Arc<Mutex<HashMap<String, u64>>>,
}

impl Default for MetricsEventListener {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsEventListener {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_metrics(&self) -> HashMap<String, u64> {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }
}

#[async_trait]
impl LifecycleEventListener for MetricsEventListener {
    async fn handle_event(&self, event: &LifecycleEvent) -> Result<()> {
        let metric_key = format!("{}_{:?}", event.agent_id, event.event_type);
        let mut metrics = self.metrics.lock().await;
        *metrics.entry(metric_key).or_insert(0) += 1;
        Ok(())
    }

    fn interested_in(&self, _event_type: &LifecycleEventType) -> bool {
        true // Collect metrics for all events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;

    struct TestEventListener {
        events_received: Arc<AtomicUsize>,
        interested_types: Vec<LifecycleEventType>,
    }

    impl TestEventListener {
        fn new(interested_types: Vec<LifecycleEventType>) -> Self {
            Self {
                events_received: Arc::new(AtomicUsize::new(0)),
                interested_types,
            }
        }

        fn get_events_received(&self) -> usize {
            self.events_received.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl LifecycleEventListener for TestEventListener {
        async fn handle_event(&self, _event: &LifecycleEvent) -> Result<()> {
            self.events_received.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn interested_in(&self, event_type: &LifecycleEventType) -> bool {
            self.interested_types.is_empty() || self.interested_types.contains(event_type)
        }
    }
    #[tokio::test]
    async fn test_event_system_basic() {
        let event_system = LifecycleEventSystem::default();

        // Create test listener
        let listener = Arc::new(TestEventListener::new(vec![]));
        let subscription = EventSubscription::new(listener.clone());

        event_system.subscribe(subscription).await;

        // Emit test event
        let event = LifecycleEvent::new(
            LifecycleEventType::StateChanged,
            "test-agent".to_string(),
            LifecycleEventData::Generic {
                message: "Test event".to_string(),
                details: HashMap::new(),
            },
            "test".to_string(),
        );

        event_system.emit(event).await.unwrap();

        // Wait for event processing
        sleep(Duration::from_millis(10)).await;

        // Check if listener received event
        assert_eq!(listener.get_events_received(), 1);

        // Check statistics
        let stats = event_system.get_statistics().await;
        assert_eq!(stats.total_events, 1);
        assert_eq!(
            stats.events_by_type.get(&LifecycleEventType::StateChanged),
            Some(&1)
        );
    }
    #[tokio::test]
    async fn test_event_filtering() {
        let event_system = LifecycleEventSystem::default();

        // Create listener interested only in error events
        let listener = Arc::new(TestEventListener::new(vec![
            LifecycleEventType::ErrorOccurred,
        ]));
        let subscription = EventSubscription::new(listener.clone())
            .for_event_types(vec![LifecycleEventType::ErrorOccurred]);

        event_system.subscribe(subscription).await;

        // Emit different types of events
        let events = vec![
            LifecycleEvent::new(
                LifecycleEventType::StateChanged,
                "test-agent".to_string(),
                LifecycleEventData::Generic {
                    message: "State change".to_string(),
                    details: HashMap::new(),
                },
                "test".to_string(),
            ),
            LifecycleEvent::new(
                LifecycleEventType::ErrorOccurred,
                "test-agent".to_string(),
                LifecycleEventData::Error {
                    message: "Test error".to_string(),
                    error_type: "test".to_string(),
                    recovery_possible: true,
                },
                "test".to_string(),
            ),
        ];

        for event in events {
            event_system.emit(event).await.unwrap();
        }

        // Wait for event processing
        sleep(Duration::from_millis(10)).await;

        // Should only receive the error event
        assert_eq!(listener.get_events_received(), 1);
    }
    #[tokio::test]
    async fn test_agent_specific_filtering() {
        let event_system = LifecycleEventSystem::default();

        // Create listener interested only in events from specific agent
        let listener = Arc::new(TestEventListener::new(vec![]));
        let subscription =
            EventSubscription::new(listener.clone()).for_agent("target-agent".to_string());

        event_system.subscribe(subscription).await;

        // Emit events from different agents
        let events = vec![
            LifecycleEvent::new(
                LifecycleEventType::StateChanged,
                "other-agent".to_string(),
                LifecycleEventData::Generic {
                    message: "Other agent event".to_string(),
                    details: HashMap::new(),
                },
                "test".to_string(),
            ),
            LifecycleEvent::new(
                LifecycleEventType::StateChanged,
                "target-agent".to_string(),
                LifecycleEventData::Generic {
                    message: "Target agent event".to_string(),
                    details: HashMap::new(),
                },
                "test".to_string(),
            ),
        ];

        for event in events {
            event_system.emit(event).await.unwrap();
        }

        // Wait for event processing
        sleep(Duration::from_millis(10)).await;

        // Should only receive the target agent event
        assert_eq!(listener.get_events_received(), 1);
    }
    #[tokio::test]
    async fn test_event_history() {
        let event_system = LifecycleEventSystem::default();

        // Emit multiple events
        for i in 0..5 {
            let event = LifecycleEvent::new(
                LifecycleEventType::StateChanged,
                format!("agent-{}", i),
                LifecycleEventData::Generic {
                    message: format!("Event {}", i),
                    details: HashMap::new(),
                },
                "test".to_string(),
            );
            event_system.emit(event).await.unwrap();
        }

        // Check history
        let history = event_system.get_event_history().await;
        assert_eq!(history.len(), 5);

        // Check agent-specific events
        let agent_events = event_system.get_agent_events("agent-2").await;
        assert_eq!(agent_events.len(), 1);
        assert_eq!(agent_events[0].agent_id, "agent-2");
    }
    #[tokio::test]
    async fn test_logging_listener() {
        let listener = LoggingEventListener::new();

        let event = LifecycleEvent::new(
            LifecycleEventType::StateChanged,
            "test-agent".to_string(),
            LifecycleEventData::StateTransition {
                from: AgentState::Ready,
                to: AgentState::Running,
                duration: Some(Duration::from_millis(100)),
                reason: Some("Starting execution".to_string()),
            },
            "test".to_string(),
        );

        // Should not panic
        listener.handle_event(&event).await.unwrap();
    }
    #[tokio::test]
    async fn test_metrics_listener() {
        let listener = MetricsEventListener::new();

        // Send multiple events
        for i in 0..3 {
            let event = LifecycleEvent::new(
                LifecycleEventType::StateChanged,
                "test-agent".to_string(),
                LifecycleEventData::Generic {
                    message: format!("Event {}", i),
                    details: HashMap::new(),
                },
                "test".to_string(),
            );
            listener.handle_event(&event).await.unwrap();
        }

        let metrics = listener.get_metrics().await;
        assert_eq!(metrics.get("test-agent_StateChanged"), Some(&3));
    }
}
