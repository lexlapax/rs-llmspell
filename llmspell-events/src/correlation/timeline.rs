// ABOUTME: Timeline reconstruction system for event correlation analysis
// ABOUTME: Builds ordered event timelines with causality analysis and visualization support

use super::{EventCorrelationTracker, EventRelationship};
use crate::universal_event::UniversalEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Timeline entry representing an event in chronological order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// The event
    pub event: UniversalEvent,
    /// Position in timeline (0-based)
    pub position: usize,
    /// Relative time from timeline start
    pub relative_time: chrono::Duration,
    /// Depth in causality chain (0 = root cause)
    pub causality_depth: u32,
    /// Events that this event caused
    pub caused_events: Vec<Uuid>,
    /// Events that caused this event
    pub causing_events: Vec<Uuid>,
    /// Concurrent events (happening at similar time)
    pub concurrent_events: Vec<Uuid>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl TimelineEntry {
    /// Create a new timeline entry
    pub fn new(event: UniversalEvent, position: usize, start_time: DateTime<Utc>) -> Self {
        let relative_time = event.timestamp.signed_duration_since(start_time);

        Self {
            event,
            position,
            relative_time,
            causality_depth: 0,
            caused_events: Vec::new(),
            causing_events: Vec::new(),
            concurrent_events: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Add a caused event
    pub fn add_caused_event(&mut self, event_id: Uuid) {
        if !self.caused_events.contains(&event_id) {
            self.caused_events.push(event_id);
        }
    }

    /// Add a causing event
    pub fn add_causing_event(&mut self, event_id: Uuid) {
        if !self.causing_events.contains(&event_id) {
            self.causing_events.push(event_id);
        }
    }

    /// Add a concurrent event
    pub fn add_concurrent_event(&mut self, event_id: Uuid) {
        if !self.concurrent_events.contains(&event_id) {
            self.concurrent_events.push(event_id);
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Check if event is a root cause (no causing events)
    pub fn is_root_cause(&self) -> bool {
        self.causing_events.is_empty()
    }

    /// Check if event is a leaf effect (no caused events)
    pub fn is_leaf_effect(&self) -> bool {
        self.caused_events.is_empty()
    }
}

/// Timeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStats {
    /// Total events in timeline
    pub total_events: usize,
    /// Timeline duration
    pub duration: chrono::Duration,
    /// Number of causal chains
    pub causal_chains: usize,
    /// Maximum causality depth
    pub max_depth: u32,
    /// Number of concurrent event groups
    pub concurrent_groups: usize,
    /// Events per second average
    pub events_per_second: f64,
    /// Root cause events
    pub root_causes: usize,
    /// Leaf effect events
    pub leaf_effects: usize,
}

/// Timeline reconstruction configuration
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    /// Maximum events to include in timeline
    pub max_events: usize,
    /// Time window for considering events concurrent (milliseconds)
    pub concurrency_window_ms: i64,
    /// Include low-confidence links
    pub include_weak_links: bool,
    /// Minimum link strength to consider
    pub min_link_strength: f64,
    /// Maximum causality depth to analyze
    pub max_causality_depth: u32,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            concurrency_window_ms: 100,
            include_weak_links: false,
            min_link_strength: 0.5,
            max_causality_depth: 100,
        }
    }
}

/// Causality chain representing a sequence of cause-effect relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityChain {
    /// Chain ID
    pub id: Uuid,
    /// Events in the chain in causal order
    pub events: Vec<Uuid>,
    /// Chain start time
    pub start_time: DateTime<Utc>,
    /// Chain end time
    pub end_time: DateTime<Utc>,
    /// Chain duration
    pub duration: chrono::Duration,
    /// Chain depth (number of causal links)
    pub depth: u32,
    /// Chain tags
    pub tags: Vec<String>,
}

impl CausalityChain {
    /// Create a new causality chain
    pub fn new(root_event_id: Uuid, start_time: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            events: vec![root_event_id],
            start_time,
            end_time: start_time,
            duration: chrono::Duration::zero(),
            depth: 0,
            tags: Vec::new(),
        }
    }

    /// Add an event to the chain
    pub fn add_event(&mut self, event_id: Uuid, timestamp: DateTime<Utc>) {
        self.events.push(event_id);
        if timestamp > self.end_time {
            self.end_time = timestamp;
            self.duration = self.end_time.signed_duration_since(self.start_time);
        }
        self.depth = self.events.len() as u32 - 1;
    }

    /// Get the root event (first in chain)
    pub fn root_event(&self) -> Option<Uuid> {
        self.events.first().copied()
    }

    /// Get the leaf event (last in chain)
    pub fn leaf_event(&self) -> Option<Uuid> {
        self.events.last().copied()
    }
}

/// Reconstructed timeline with causality analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimeline {
    /// Timeline ID
    pub id: Uuid,
    /// Timeline entries in chronological order
    pub entries: Vec<TimelineEntry>,
    /// Timeline start time
    pub start_time: DateTime<Utc>,
    /// Timeline end time
    pub end_time: DateTime<Utc>,
    /// Timeline duration
    pub duration: chrono::Duration,
    /// Causality chains
    pub causal_chains: Vec<CausalityChain>,
    /// Timeline statistics
    pub stats: TimelineStats,
    /// Timeline metadata
    pub metadata: HashMap<String, String>,
}

impl Default for EventTimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl EventTimeline {
    /// Create a new empty timeline
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entries: Vec::new(),
            start_time: now,
            end_time: now,
            duration: chrono::Duration::zero(),
            causal_chains: Vec::new(),
            stats: TimelineStats {
                total_events: 0,
                duration: chrono::Duration::zero(),
                causal_chains: 0,
                max_depth: 0,
                concurrent_groups: 0,
                events_per_second: 0.0,
                root_causes: 0,
                leaf_effects: 0,
            },
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to timeline
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get events within a time range
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.event.timestamp >= start && entry.event.timestamp <= end)
            .collect()
    }

    /// Get events matching a pattern
    pub fn get_events_matching(&self, pattern: &str) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.event.matches_pattern(pattern))
            .collect()
    }

    /// Get events by component
    pub fn get_events_by_component(&self, component: &str) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .event
                    .metadata
                    .source
                    .as_ref()
                    .is_some_and(|s| s == component)
            })
            .collect()
    }

    /// Get root cause events
    pub fn get_root_causes(&self) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.is_root_cause())
            .collect()
    }

    /// Get leaf effect events
    pub fn get_leaf_effects(&self) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.is_leaf_effect())
            .collect()
    }

    /// Find the causality chain containing an event
    pub fn find_chain_containing(&self, event_id: Uuid) -> Option<&CausalityChain> {
        self.causal_chains
            .iter()
            .find(|chain| chain.events.contains(&event_id))
    }

    /// Get the longest causality chain
    pub fn get_longest_chain(&self) -> Option<&CausalityChain> {
        self.causal_chains.iter().max_by_key(|chain| chain.depth)
    }

    /// Update statistics
    fn update_stats(&mut self) {
        self.stats.total_events = self.entries.len();
        self.stats.duration = self.duration;
        self.stats.causal_chains = self.causal_chains.len();
        self.stats.max_depth = self
            .causal_chains
            .iter()
            .map(|c| c.depth)
            .max()
            .unwrap_or(0);

        if self.duration.num_seconds() > 0 {
            self.stats.events_per_second =
                self.stats.total_events as f64 / self.duration.num_seconds() as f64;
        }

        self.stats.root_causes = self.entries.iter().filter(|e| e.is_root_cause()).count();
        self.stats.leaf_effects = self.entries.iter().filter(|e| e.is_leaf_effect()).count();
    }
}

/// Timeline builder for reconstructing event timelines
pub struct TimelineBuilder {
    config: TimelineConfig,
}

impl Default for TimelineBuilder {
    fn default() -> Self {
        Self::with_default_config()
    }
}

impl TimelineBuilder {
    /// Create a new timeline builder
    pub fn new(config: TimelineConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(TimelineConfig::default())
    }

    /// Build timeline from correlation tracker
    pub fn build_timeline(
        &self,
        tracker: &EventCorrelationTracker,
        correlation_id: &Uuid,
    ) -> EventTimeline {
        let events = tracker.get_events(correlation_id);
        self.build_timeline_from_events(events, tracker)
    }

    /// Build timeline from events with correlation data
    pub fn build_timeline_from_events(
        &self,
        mut events: Vec<UniversalEvent>,
        tracker: &EventCorrelationTracker,
    ) -> EventTimeline {
        if events.is_empty() {
            return EventTimeline::new();
        }

        // Sort events by timestamp
        events.sort_by_key(|e| e.timestamp);

        // Limit events if necessary
        if events.len() > self.config.max_events {
            events.truncate(self.config.max_events);
        }

        let start_time = events.first().unwrap().timestamp;
        let end_time = events.last().unwrap().timestamp;
        let duration = end_time.signed_duration_since(start_time);

        let mut timeline = EventTimeline::new();
        timeline.start_time = start_time;
        timeline.end_time = end_time;
        timeline.duration = duration;

        // Create timeline entries
        let mut entries = Vec::new();
        for (position, event) in events.iter().enumerate() {
            let entry = TimelineEntry::new(event.clone(), position, start_time);
            entries.push(entry);
        }

        // Analyze causality relationships
        self.analyze_causality(&mut entries, tracker);

        // Build causality chains
        let causal_chains = self.build_causality_chains(&entries, tracker);

        // Detect concurrent events
        self.detect_concurrent_events(&mut entries);

        timeline.entries = entries;
        timeline.causal_chains = causal_chains;
        timeline.update_stats();

        timeline
    }

    /// Analyze causality relationships between events
    fn analyze_causality(&self, entries: &mut [TimelineEntry], tracker: &EventCorrelationTracker) {
        // Build event ID to entry index mapping
        let mut event_to_index: HashMap<Uuid, usize> = HashMap::new();
        for (index, entry) in entries.iter().enumerate() {
            event_to_index.insert(entry.event.id, index);
        }

        // Collect all relationship updates first to avoid borrow checker issues
        let mut updates: Vec<(usize, Uuid, bool)> = Vec::new(); // (index, related_event_id, is_causing)

        for (current_index, entry) in entries.iter().enumerate() {
            let links = tracker.get_links(&entry.event.id);

            for link in links {
                if link.strength < self.config.min_link_strength && !self.config.include_weak_links
                {
                    continue;
                }

                match link.relationship {
                    EventRelationship::CausedBy => {
                        if let Some(&causing_index) = event_to_index.get(&link.from_event_id) {
                            updates.push((current_index, link.from_event_id, true)); // Add causing event
                            updates.push((causing_index, entry.event.id, false));
                            // Add caused event
                        }
                    }
                    EventRelationship::ResponseTo => {
                        if let Some(&request_index) = event_to_index.get(&link.from_event_id) {
                            updates.push((current_index, link.from_event_id, true)); // Add causing event
                            updates.push((request_index, entry.event.id, false));
                            // Add caused event
                        }
                    }
                    EventRelationship::FollowsFrom => {
                        if let Some(&prev_index) = event_to_index.get(&link.from_event_id) {
                            updates.push((current_index, link.from_event_id, true)); // Add causing event
                            updates.push((prev_index, entry.event.id, false)); // Add caused event
                        }
                    }
                    _ => {} // Handle other relationships as needed
                }
            }
        }

        // Apply all updates
        for (index, related_event_id, is_causing) in updates {
            if let Some(entry) = entries.get_mut(index) {
                if is_causing {
                    entry.add_causing_event(related_event_id);
                } else {
                    entry.add_caused_event(related_event_id);
                }
            }
        }

        // Calculate causality depth
        self.calculate_causality_depth(entries);
    }

    /// Calculate causality depth for each event
    fn calculate_causality_depth(&self, entries: &mut [TimelineEntry]) {
        // Build adjacency list for causality graph
        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();

        for entry in entries.iter() {
            in_degree.insert(entry.event.id, entry.causing_events.len());
            adjacency.insert(entry.event.id, entry.caused_events.clone());
        }

        // Topological sort to assign depths
        let mut queue: VecDeque<Uuid> = VecDeque::new();
        let mut depths: HashMap<Uuid, u32> = HashMap::new();

        // Start with root events (no incoming edges)
        for entry in entries.iter() {
            if entry.causing_events.is_empty() {
                queue.push_back(entry.event.id);
                depths.insert(entry.event.id, 0);
            }
        }

        // Process queue
        while let Some(event_id) = queue.pop_front() {
            let current_depth = depths[&event_id];

            if let Some(caused_events) = adjacency.get(&event_id) {
                for &caused_id in caused_events {
                    let new_depth = current_depth + 1;
                    if new_depth > self.config.max_causality_depth {
                        continue;
                    }

                    let current_caused_depth = depths.get(&caused_id).copied().unwrap_or(0);
                    if new_depth > current_caused_depth {
                        depths.insert(caused_id, new_depth);
                    }

                    // Decrease in-degree and add to queue if no more dependencies
                    if let Some(degree) = in_degree.get_mut(&caused_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(caused_id);
                        }
                    }
                }
            }
        }

        // Assign calculated depths to entries
        for entry in entries.iter_mut() {
            entry.causality_depth = depths.get(&entry.event.id).copied().unwrap_or(0);
        }
    }

    /// Build causality chains from the timeline
    fn build_causality_chains(
        &self,
        entries: &[TimelineEntry],
        _tracker: &EventCorrelationTracker,
    ) -> Vec<CausalityChain> {
        let mut chains = Vec::new();
        let mut visited = HashSet::new();

        // Start from root events and build chains
        for entry in entries.iter() {
            if entry.is_root_cause() && !visited.contains(&entry.event.id) {
                let chain = self.build_chain_from_root(entry, entries, &mut visited);
                chains.push(chain);
            }
        }

        chains
    }

    /// Build a causality chain starting from a root event
    fn build_chain_from_root(
        &self,
        root_entry: &TimelineEntry,
        entries: &[TimelineEntry],
        visited: &mut HashSet<Uuid>,
    ) -> CausalityChain {
        let mut chain = CausalityChain::new(root_entry.event.id, root_entry.event.timestamp);
        visited.insert(root_entry.event.id);

        // Build event ID to entry mapping
        let entry_map: HashMap<Uuid, &TimelineEntry> =
            entries.iter().map(|e| (e.event.id, e)).collect();

        // Follow the chain of caused events
        let mut current_events = vec![root_entry.event.id];

        while !current_events.is_empty() {
            let mut next_events = Vec::new();

            for &event_id in &current_events {
                if let Some(entry) = entry_map.get(&event_id) {
                    for &caused_id in &entry.caused_events {
                        if !visited.contains(&caused_id) {
                            if let Some(caused_entry) = entry_map.get(&caused_id) {
                                chain.add_event(caused_id, caused_entry.event.timestamp);
                                visited.insert(caused_id);
                                next_events.push(caused_id);
                            }
                        }
                    }
                }
            }

            current_events = next_events;
        }

        chain
    }

    /// Detect concurrent events within the timeline
    fn detect_concurrent_events(&self, entries: &mut [TimelineEntry]) {
        let window = chrono::Duration::milliseconds(self.config.concurrency_window_ms);

        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let time_diff = entries[j]
                    .event
                    .timestamp
                    .signed_duration_since(entries[i].event.timestamp)
                    .abs();

                if time_diff <= window {
                    entries[i].add_concurrent_event(entries[j].event.id);
                    entries[j].add_concurrent_event(entries[i].event.id);
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "event")]
mod tests {
    use super::*;
    use crate::correlation::EventCorrelationTracker;
    use crate::universal_event::{Language, UniversalEvent};
    use serde_json::Value;

    fn create_test_event(event_type: &str, correlation_id: Uuid) -> UniversalEvent {
        let mut event = UniversalEvent::new(event_type, Value::Null, Language::Rust);
        event.metadata.correlation_id = correlation_id;
        event
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_timeline_entry_creation() {
        let event = create_test_event("test.event", Uuid::new_v4());
        let start_time = Utc::now();
        let entry = TimelineEntry::new(event, 0, start_time);

        assert_eq!(entry.position, 0);
        assert_eq!(entry.causality_depth, 0);
        assert!(entry.is_root_cause());
        assert!(entry.is_leaf_effect());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_timeline_building() {
        let tracker = EventCorrelationTracker::default();
        let correlation_id = Uuid::new_v4();

        // Create a sequence of events
        let event1 = create_test_event("start", correlation_id);
        let event2 = create_test_event("middle", correlation_id);
        let event3 = create_test_event("end", correlation_id);

        tracker.track_event(event1);
        tracker.track_event(event2);
        tracker.track_event(event3);

        let builder = TimelineBuilder::default();
        let timeline = builder.build_timeline(&tracker, &correlation_id);

        assert_eq!(timeline.entries.len(), 3);
        assert_eq!(timeline.stats.total_events, 3);
        assert!(timeline.duration >= chrono::Duration::zero());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_causality_chain_creation() {
        let root_id = Uuid::new_v4();
        let start_time = Utc::now();
        let mut chain = CausalityChain::new(root_id, start_time);

        assert_eq!(chain.root_event(), Some(root_id));
        assert_eq!(chain.depth, 0);

        let child_id = Uuid::new_v4();
        chain.add_event(child_id, start_time + chrono::Duration::seconds(1));

        assert_eq!(chain.leaf_event(), Some(child_id));
        assert_eq!(chain.depth, 1);
        assert_eq!(chain.duration, chrono::Duration::seconds(1));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_timeline_queries() {
        let mut timeline = EventTimeline::new();
        timeline.add_metadata("test".to_string(), "value".to_string());

        // Add some test entries
        let event1 = create_test_event("system.start", Uuid::new_v4());
        let event2 = create_test_event("agent.execute", Uuid::new_v4());

        let entry1 = TimelineEntry::new(event1, 0, Utc::now());
        let entry2 = TimelineEntry::new(event2, 1, Utc::now());

        timeline.entries.push(entry1);
        timeline.entries.push(entry2);

        let system_events = timeline.get_events_matching("system.*");
        assert_eq!(system_events.len(), 1);

        let agent_events = timeline.get_events_matching("agent.*");
        assert_eq!(agent_events.len(), 1);
    }
}
