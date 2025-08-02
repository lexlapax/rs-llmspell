// ABOUTME: EventBridge for cross-language event propagation with EventBus integration
// ABOUTME: Manages event routing, subscription, and serialization across script languages

use crate::globals::GlobalContext;
use anyhow::{Context, Result};
use llmspell_events::{EventBus, Language as EventLanguage, UniversalEvent};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

/// Subscription handle for managing event subscriptions
#[derive(Debug, Clone)]
pub struct SubscriptionHandle {
    pub id: String,
    pub pattern: String,
    pub language: EventLanguage,
}

/// `EventBridge` for cross-language event communication
pub struct EventBridge {
    /// Underlying event bus
    event_bus: Arc<EventBus>,
    /// Active subscriptions mapped by subscription ID
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionHandle>>>,
    #[allow(dead_code)]
    /// Reference to global context for cross-component communication
    context: Arc<GlobalContext>,
}

impl EventBridge {
    /// Create a new `EventBridge` with default `EventBus`
    ///
    /// # Errors
    ///
    /// Returns an error if EventBridge initialization fails
    pub async fn new(context: Arc<GlobalContext>) -> Result<Self> {
        let event_bus = Arc::new(EventBus::new());

        Ok(Self {
            event_bus,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            context,
        })
    }

    /// Create a new `EventBridge` with custom `EventBus`
    #[must_use]
    pub fn with_event_bus(context: Arc<GlobalContext>, event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            context,
        }
    }

    /// Publish an event to the event bus
    ///
    /// # Errors
    ///
    /// Returns an error if event publication fails
    pub async fn publish_event(&self, event: UniversalEvent) -> Result<()> {
        self.event_bus
            .publish(event)
            .await
            .with_context(|| "Failed to publish event to event bus")
    }

    /// Subscribe to events matching a pattern
    pub async fn subscribe_pattern(
        &self,
        pattern: &str,
        language: EventLanguage,
    ) -> Result<(String, UnboundedReceiver<UniversalEvent>)> {
        let subscription_id = Uuid::new_v4().to_string();

        // Subscribe to the pattern through the event bus
        let receiver = self
            .event_bus
            .subscribe(pattern)
            .await
            .with_context(|| format!("Failed to subscribe to pattern: {pattern}"))?;

        // Store subscription metadata
        let handle = SubscriptionHandle {
            id: subscription_id.clone(),
            pattern: pattern.to_string(),
            language,
        };

        {
            let mut subscriptions = self.subscriptions.write();
            subscriptions.insert(subscription_id.clone(), handle);
        }

        Ok((subscription_id, receiver))
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<bool> {
        let mut subscriptions = self.subscriptions.write();

        if let Some(_handle) = subscriptions.remove(subscription_id) {
            // Note: EventBus doesn't currently support explicit unsubscribe
            // The receiver will be dropped when the script context is cleaned up
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List active subscriptions
    #[must_use]
    pub fn list_subscriptions(&self) -> Vec<SubscriptionHandle> {
        let subscriptions = self.subscriptions.read();
        subscriptions.values().cloned().collect()
    }

    /// Get subscription count
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.read();
        subscriptions.len()
    }

    /// Get the underlying event bus reference
    #[must_use]
    pub const fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Get event bus statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let stats = self.event_bus.get_stats();
        let subscription_count = self.subscription_count();

        serde_json::json!({
            "event_bus_stats": {
                "events_processed": stats.events_processed,
                "rate_limit_violations": stats.rate_limit_violations,
                "buffer_size": self.event_bus.buffer_size()
            },
            "bridge_stats": {
                "active_subscriptions": subscription_count,
                "subscriptions_by_language": self.get_subscriptions_by_language()
            }
        })
    }

    /// Get subscription breakdown by language
    fn get_subscriptions_by_language(&self) -> HashMap<String, usize> {
        let mut by_language = HashMap::new();
        {
            let subscriptions = self.subscriptions.read();
            for handle in subscriptions.values() {
                let language_str = format!("{:?}", handle.language);
                *by_language.entry(language_str).or_insert(0) += 1;
            }
        } // Explicitly drop the lock here

        by_language
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};
    use llmspell_events::Language;

    async fn create_test_context() -> Arc<GlobalContext> {
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(ProviderManager::new(Default::default()).await.unwrap());
        Arc::new(GlobalContext::new(registry, providers))
    }
    #[tokio::test]
    async fn test_event_bridge_creation() {
        let context = create_test_context().await;
        let bridge = EventBridge::new(context).await.unwrap();

        assert_eq!(bridge.subscription_count(), 0);
    }
    #[tokio::test]
    async fn test_event_publish_and_subscribe() {
        let context = create_test_context().await;
        let bridge = EventBridge::new(context).await.unwrap();

        // Subscribe to events
        let (sub_id, mut receiver) = bridge
            .subscribe_pattern("test.*", Language::Rust)
            .await
            .unwrap();
        assert_eq!(bridge.subscription_count(), 1);

        // Publish an event
        let event = UniversalEvent::new(
            "test.example",
            serde_json::json!({"message": "hello"}),
            Language::Rust,
        );

        bridge.publish_event(event.clone()).await.unwrap();

        // Receive the event
        let received = tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(received.event_type, "test.example");
        assert_eq!(received.language, Language::Rust);

        // Unsubscribe
        let unsubscribed = bridge.unsubscribe(&sub_id).await.unwrap();
        assert!(unsubscribed);
        assert_eq!(bridge.subscription_count(), 0);
    }
    #[tokio::test]
    async fn test_subscription_management() {
        let context = create_test_context().await;
        let bridge = EventBridge::new(context).await.unwrap();

        // Create multiple subscriptions
        let (sub1, _) = bridge
            .subscribe_pattern("user.*", Language::Lua)
            .await
            .unwrap();
        let (_sub2, _) = bridge
            .subscribe_pattern("system.*", Language::Rust)
            .await
            .unwrap();

        assert_eq!(bridge.subscription_count(), 2);

        let subscriptions = bridge.list_subscriptions();
        assert_eq!(subscriptions.len(), 2);

        // Verify subscription details
        let sub1_handle = subscriptions.iter().find(|s| s.id == sub1).unwrap();
        assert_eq!(sub1_handle.pattern, "user.*");
        assert_eq!(sub1_handle.language, Language::Lua);

        // Unsubscribe one
        bridge.unsubscribe(&sub1).await.unwrap();
        assert_eq!(bridge.subscription_count(), 1);
    }
    #[tokio::test]
    async fn test_stats() {
        let context = create_test_context().await;
        let bridge = EventBridge::new(context).await.unwrap();

        // Create subscriptions with different languages
        let (_sub1, _) = bridge
            .subscribe_pattern("test1.*", Language::Lua)
            .await
            .unwrap();
        let (_sub2, _) = bridge
            .subscribe_pattern("test2.*", Language::Rust)
            .await
            .unwrap();
        let (_sub3, _) = bridge
            .subscribe_pattern("test3.*", Language::Lua)
            .await
            .unwrap();

        let stats = bridge.get_stats().await;

        // Verify structure
        assert!(stats["event_bus_stats"].is_object());
        assert!(stats["bridge_stats"].is_object());
        assert_eq!(stats["bridge_stats"]["active_subscriptions"], 3);

        // Verify language breakdown
        let by_language = &stats["bridge_stats"]["subscriptions_by_language"];
        assert_eq!(by_language["Lua"], 2);
        assert_eq!(by_language["Rust"], 1);
    }
}
