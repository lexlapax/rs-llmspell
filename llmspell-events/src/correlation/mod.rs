// ABOUTME: Event correlation system for linking related events across components
// ABOUTME: Enables timeline reconstruction and causality analysis for debugging

pub mod query;
pub mod timeline;

use crate::universal_event::UniversalEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use uuid::Uuid;

/// Correlation context for tracking related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationContext {
    /// Primary correlation ID for this context
    pub correlation_id: Uuid,
    /// Parent correlation ID (for nested operations)
    pub parent_id: Option<Uuid>,
    /// Root correlation ID (for tracing back to origin)
    pub root_id: Uuid,
    /// Context creation timestamp
    pub created_at: DateTime<Utc>,
    /// Context metadata
    pub metadata: HashMap<String, String>,
    /// Tags for filtering and categorization
    pub tags: Vec<String>,
}

impl CorrelationContext {
    /// Create a new root correlation context
    pub fn new_root() -> Self {
        let correlation_id = Uuid::new_v4();
        Self {
            correlation_id,
            parent_id: None,
            root_id: correlation_id,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Create a child correlation context
    pub fn create_child(&self) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            parent_id: Some(self.correlation_id),
            root_id: self.root_id,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add a tag to the context
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if context has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Relationship type between correlated events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventRelationship {
    /// One event caused another
    CausedBy,
    /// Events are part of the same operation
    PartOf,
    /// Events are related but not directly causal
    RelatedTo,
    /// Event is a response to another
    ResponseTo,
    /// Event follows another in sequence
    FollowsFrom,
    /// Events are concurrent/parallel
    ConcurrentWith,
}

/// Link between two correlated events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLink {
    /// Source event ID
    pub from_event_id: Uuid,
    /// Target event ID
    pub to_event_id: Uuid,
    /// Type of relationship
    pub relationship: EventRelationship,
    /// Link strength (0.0 to 1.0)
    pub strength: f64,
    /// Link metadata
    pub metadata: HashMap<String, String>,
    /// Link creation timestamp
    pub created_at: DateTime<Utc>,
}

impl EventLink {
    /// Create a new event link
    pub fn new(from_event_id: Uuid, to_event_id: Uuid, relationship: EventRelationship) -> Self {
        Self {
            from_event_id,
            to_event_id,
            relationship,
            strength: 1.0,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Set link strength
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Add metadata to the link
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Configuration for correlation tracking
#[derive(Debug, Clone)]
pub struct CorrelationConfig {
    /// Maximum number of events to track per correlation
    pub max_events_per_correlation: usize,
    /// Maximum age for correlation data
    pub max_correlation_age: Duration,
    /// Maximum number of total correlations to track
    pub max_total_correlations: usize,
    /// Enable automatic link detection
    pub auto_link_detection: bool,
    /// Link detection time window
    pub link_detection_window: Duration,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            max_events_per_correlation: 1000,
            max_correlation_age: Duration::from_secs(3600), // 1 hour
            max_total_correlations: 10000,
            auto_link_detection: true,
            link_detection_window: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Statistics for correlation tracking
#[derive(Debug, Default, Clone)]
pub struct CorrelationStats {
    /// Total events tracked
    pub total_events: u64,
    /// Total correlations active
    pub active_correlations: usize,
    /// Total links detected
    pub total_links: u64,
    /// Events processed per second (approximate)
    pub events_per_second: f64,
    /// Memory usage estimate in bytes
    pub estimated_memory_usage: usize,
}

/// Event correlation tracker
#[derive(Debug)]
pub struct EventCorrelationTracker {
    /// Configuration for correlation tracking
    config: CorrelationConfig,
    /// Events organized by correlation ID
    correlations: Arc<RwLock<HashMap<Uuid, VecDeque<UniversalEvent>>>>,
    /// Links between events
    links: Arc<RwLock<HashMap<Uuid, Vec<EventLink>>>>,
    /// Correlation contexts
    contexts: Arc<RwLock<HashMap<Uuid, CorrelationContext>>>,
    /// Statistics
    stats: Arc<RwLock<CorrelationStats>>,
    /// Event index for quick lookups
    event_index: Arc<RwLock<HashMap<Uuid, (Uuid, usize)>>>, // event_id -> (correlation_id, index)
}

impl Default for EventCorrelationTracker {
    fn default() -> Self {
        Self::with_default_config()
    }
}

impl EventCorrelationTracker {
    /// Create a new correlation tracker
    pub fn new(config: CorrelationConfig) -> Self {
        Self {
            config,
            correlations: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CorrelationStats::default())),
            event_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(CorrelationConfig::default())
    }

    /// Track a new event
    pub fn track_event(&self, event: UniversalEvent) {
        let correlation_id = event.metadata.correlation_id;

        // Add event to correlation
        {
            let mut correlations = self.correlations.write().unwrap();
            let events = correlations.entry(correlation_id).or_default();

            // Enforce size limits
            if events.len() >= self.config.max_events_per_correlation {
                if let Some(old_event) = events.pop_front() {
                    // Remove from event index
                    self.event_index.write().unwrap().remove(&old_event.id);
                }
            }

            let index = events.len();
            events.push_back(event.clone());

            // Add to event index
            self.event_index
                .write()
                .unwrap()
                .insert(event.id, (correlation_id, index));
        }

        // Auto-detect links if enabled
        if self.config.auto_link_detection {
            self.auto_detect_links(&event);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_events += 1;
            stats.active_correlations = self.correlations.read().unwrap().len();
        }

        // Clean up old correlations
        self.cleanup_old_correlations();
    }

    /// Add a correlation context
    pub fn add_context(&self, context: CorrelationContext) {
        self.contexts
            .write()
            .unwrap()
            .insert(context.correlation_id, context);
    }

    /// Get correlation context
    pub fn get_context(&self, correlation_id: &Uuid) -> Option<CorrelationContext> {
        self.contexts.read().unwrap().get(correlation_id).cloned()
    }

    /// Add a manual link between events
    pub fn add_link(&self, link: EventLink) {
        let mut links = self.links.write().unwrap();
        links
            .entry(link.from_event_id)
            .or_default()
            .push(link.clone());

        // Also add reverse link for bidirectional lookup
        links.entry(link.to_event_id).or_default().push(EventLink {
            from_event_id: link.to_event_id,
            to_event_id: link.from_event_id,
            relationship: link.relationship,
            strength: link.strength,
            metadata: link.metadata,
            created_at: link.created_at,
        });

        // Update statistics
        self.stats.write().unwrap().total_links += 1;
    }

    /// Get events for a correlation ID
    pub fn get_events(&self, correlation_id: &Uuid) -> Vec<UniversalEvent> {
        self.correlations
            .read()
            .unwrap()
            .get(correlation_id)
            .map(|events| events.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get links for an event
    pub fn get_links(&self, event_id: &Uuid) -> Vec<EventLink> {
        self.links
            .read()
            .unwrap()
            .get(event_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all correlations
    pub fn get_all_correlations(&self) -> HashMap<Uuid, Vec<UniversalEvent>> {
        self.correlations
            .read()
            .unwrap()
            .iter()
            .map(|(id, events)| (*id, events.iter().cloned().collect()))
            .collect()
    }

    /// Get correlation statistics
    pub fn get_stats(&self) -> CorrelationStats {
        let mut stats = self.stats.read().unwrap().clone();

        // Update memory usage estimate
        let _correlations_count = self.correlations.read().unwrap().len();
        let links_count = self
            .links
            .read()
            .unwrap()
            .values()
            .map(Vec::len)
            .sum::<usize>();
        let contexts_count = self.contexts.read().unwrap().len();

        // Rough estimate: each event ~1KB, each link ~200B, each context ~500B
        #[allow(clippy::cast_possible_truncation)]
        let total_events_usize = stats.total_events as usize;
        stats.estimated_memory_usage =
            (total_events_usize * 1024) + (links_count * 200) + (contexts_count * 500);

        stats
    }

    /// Clear all correlation data
    pub fn clear(&self) {
        self.correlations.write().unwrap().clear();
        self.links.write().unwrap().clear();
        self.contexts.write().unwrap().clear();
        self.event_index.write().unwrap().clear();
        *self.stats.write().unwrap() = CorrelationStats::default();
    }

    /// Auto-detect links between events
    fn auto_detect_links(&self, event: &UniversalEvent) {
        let correlation_id = event.metadata.correlation_id;
        let cutoff_time = Utc::now()
            - chrono::Duration::from_std(self.config.link_detection_window).unwrap_or_default();

        // Look for events in the same correlation within the time window
        if let Some(events) = self.correlations.read().unwrap().get(&correlation_id) {
            for other_event in events.iter().rev().take(10) {
                // Check last 10 events
                if other_event.id == event.id || other_event.timestamp < cutoff_time {
                    continue;
                }

                // Detect potential relationships
                let relationship = self.detect_relationship(other_event, event);
                if let Some(rel) = relationship {
                    let link = EventLink::new(other_event.id, event.id, rel)
                        .with_strength(0.8) // Auto-detected links have lower confidence
                        .with_metadata("detection", "automatic");

                    self.add_link(link);
                }
            }
        }
    }

    /// Detect relationship between two events
    fn detect_relationship(
        &self,
        from: &UniversalEvent,
        to: &UniversalEvent,
    ) -> Option<EventRelationship> {
        // Simple heuristics for relationship detection
        let time_diff = to.timestamp.signed_duration_since(from.timestamp);

        // If events are very close in time, they might be concurrent
        if time_diff.num_milliseconds().abs() < 100 {
            return Some(EventRelationship::ConcurrentWith);
        }

        // If events are in sequence, detect causality patterns
        if time_diff.num_milliseconds() > 0 && time_diff.num_seconds() < 5 {
            // Check for common causality patterns
            if from.event_type.contains("request") && to.event_type.contains("response") {
                return Some(EventRelationship::ResponseTo);
            }

            if from.event_type.contains("start") && to.event_type.contains("end") {
                return Some(EventRelationship::PartOf);
            }

            // Default to follows-from for sequential events
            return Some(EventRelationship::FollowsFrom);
        }

        None
    }

    /// Clean up old correlations to prevent memory leaks
    fn cleanup_old_correlations(&self) {
        let cutoff_time = Utc::now()
            - chrono::Duration::from_std(self.config.max_correlation_age).unwrap_or_default();
        let max_correlations = self.config.max_total_correlations;

        let mut correlations = self.correlations.write().unwrap();
        let mut event_index = self.event_index.write().unwrap();
        let mut contexts = self.contexts.write().unwrap();
        let mut links = self.links.write().unwrap();

        // Remove correlations that are too old
        correlations.retain(|correlation_id, events| {
            if let Some(latest_event) = events.back() {
                if latest_event.timestamp < cutoff_time {
                    // Remove associated data
                    for event in events.iter() {
                        event_index.remove(&event.id);
                        links.remove(&event.id);
                    }
                    contexts.remove(correlation_id);
                    return false;
                }
            }
            true
        });

        // If still too many correlations, remove oldest ones
        if correlations.len() > max_correlations {
            let mut correlation_ages: Vec<_> = correlations
                .iter()
                .map(|(id, events)| {
                    let oldest_time = events.front().map(|e| e.timestamp).unwrap_or(Utc::now());
                    (*id, oldest_time)
                })
                .collect();

            correlation_ages.sort_by_key(|(_, time)| *time);

            let to_remove = correlation_ages.len() - max_correlations;
            for (correlation_id, _) in correlation_ages.into_iter().take(to_remove) {
                if let Some(events) = correlations.remove(&correlation_id) {
                    for event in events.iter() {
                        event_index.remove(&event.id);
                        links.remove(&event.id);
                    }
                    contexts.remove(&correlation_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_event::{Language, UniversalEvent};
    use serde_json::Value;
    #[test]
    fn test_correlation_context_creation() {
        let root_context = CorrelationContext::new_root();
        assert_eq!(root_context.correlation_id, root_context.root_id);
        assert!(root_context.parent_id.is_none());

        let child_context = root_context.create_child();
        assert_eq!(child_context.root_id, root_context.root_id);
        assert_eq!(child_context.parent_id, Some(root_context.correlation_id));
        assert_ne!(child_context.correlation_id, root_context.correlation_id);
    }
    #[test]
    fn test_event_tracking() {
        let tracker = EventCorrelationTracker::default();
        let correlation_id = Uuid::new_v4();

        let mut event = UniversalEvent::new("test.event", Value::Null, Language::Rust);
        event.metadata.correlation_id = correlation_id;

        tracker.track_event(event.clone());

        let events = tracker.get_events(&correlation_id);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);

        let stats = tracker.get_stats();
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.active_correlations, 1);
    }
    #[test]
    fn test_event_links() {
        let tracker = EventCorrelationTracker::default();

        let event1_id = Uuid::new_v4();
        let event2_id = Uuid::new_v4();

        let link = EventLink::new(event1_id, event2_id, EventRelationship::CausedBy)
            .with_strength(0.9)
            .with_metadata("test", "manual");

        tracker.add_link(link);

        let links = tracker.get_links(&event1_id);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].relationship, EventRelationship::CausedBy);
        assert_eq!(links[0].strength, 0.9);

        let stats = tracker.get_stats();
        assert_eq!(stats.total_links, 1);
    }
    #[test]
    fn test_correlation_cleanup() {
        let config = CorrelationConfig {
            max_events_per_correlation: 2,
            ..Default::default()
        };

        let tracker = EventCorrelationTracker::new(config);
        let correlation_id = Uuid::new_v4();

        // Add more events than the limit
        for i in 0..5 {
            let mut event =
                UniversalEvent::new(format!("test.event.{}", i), Value::Null, Language::Rust);
            event.metadata.correlation_id = correlation_id;
            tracker.track_event(event);
        }

        let events = tracker.get_events(&correlation_id);
        assert_eq!(events.len(), 2); // Should only keep the last 2 events
    }
    #[test]
    fn test_context_metadata() {
        let context = CorrelationContext::new_root()
            .with_metadata("component", "test")
            .with_tag("important");

        assert_eq!(context.get_metadata("component"), Some(&"test".to_string()));
        assert!(context.has_tag("important"));
        assert!(!context.has_tag("unimportant"));
    }
}
