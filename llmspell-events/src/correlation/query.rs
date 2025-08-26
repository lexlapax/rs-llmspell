// ABOUTME: Timeline query interface for filtering and searching event correlations
// ABOUTME: Provides powerful query capabilities for debugging and analysis

use super::timeline::TimelineEntry;
use super::{EventCorrelationTracker, EventRelationship};
use crate::universal_event::{Language, UniversalEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Query filter for timeline searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineQuery {
    /// Filter by correlation IDs
    pub correlation_ids: Option<Vec<Uuid>>,
    /// Filter by event types (supports wildcards)
    pub event_types: Option<Vec<String>>,
    /// Filter by time range
    pub time_range: Option<TimeRange>,
    /// Filter by source components
    pub source_components: Option<Vec<String>>,
    /// Filter by target components
    pub target_components: Option<Vec<String>>,
    /// Filter by event languages
    pub languages: Option<Vec<Language>>,
    /// Filter by event tags
    pub tags: Option<Vec<String>>,
    /// Filter by causality depth range
    pub causality_depth_range: Option<(u32, u32)>,
    /// Filter by event relationships
    pub relationships: Option<Vec<EventRelationship>>,
    /// Include only root cause events
    pub root_causes_only: bool,
    /// Include only leaf effect events
    pub leaf_effects_only: bool,
    /// Minimum event count in correlation
    pub min_events_in_correlation: Option<usize>,
    /// Maximum event count in correlation
    pub max_events_in_correlation: Option<usize>,
    /// Sort order
    pub sort_order: SortOrder,
    /// Maximum results to return
    pub limit: Option<usize>,
}

impl Default for TimelineQuery {
    fn default() -> Self {
        Self {
            correlation_ids: None,
            event_types: None,
            time_range: None,
            source_components: None,
            target_components: None,
            languages: None,
            tags: None,
            causality_depth_range: None,
            relationships: None,
            root_causes_only: false,
            leaf_effects_only: false,
            min_events_in_correlation: None,
            max_events_in_correlation: None,
            sort_order: SortOrder::ChronologicalAsc,
            limit: None,
        }
    }
}

/// Time range for filtering events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (inclusive)
    pub start: DateTime<Utc>,
    /// End time (inclusive)
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Create a time range
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Create a time range for the last N seconds
    pub fn last_seconds(seconds: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::seconds(seconds);
        Self { start, end }
    }

    /// Create a time range for the last N minutes
    pub fn last_minutes(minutes: i64) -> Self {
        Self::last_seconds(minutes * 60)
    }

    /// Create a time range for the last N hours
    pub fn last_hours(hours: i64) -> Self {
        Self::last_minutes(hours * 60)
    }

    /// Check if a timestamp is within this range
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
}

/// Sort order for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    /// Chronological order (oldest first)
    ChronologicalAsc,
    /// Reverse chronological order (newest first)
    ChronologicalDesc,
    /// By causality depth (shallow first)
    CausalityDepthAsc,
    /// By causality depth (deep first)
    CausalityDepthDesc,
    /// By event sequence number
    SequenceOrder,
    /// By correlation ID
    CorrelationId,
}

/// Query result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultEntry {
    /// The timeline entry
    pub entry: TimelineEntry,
    /// Match score (0.0 to 1.0)
    pub match_score: f64,
    /// Matching criteria
    pub match_reasons: Vec<String>,
}

/// Query execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Query that was executed
    pub query: TimelineQuery,
    /// Matching entries
    pub entries: Vec<QueryResultEntry>,
    /// Total matches before limit was applied
    pub total_matches: usize,
    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
    /// Result metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(query: TimelineQuery) -> Self {
        Self {
            query,
            entries: Vec::new(),
            total_matches: 0,
            execution_time_ms: 0,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a matching entry
    pub fn add_entry(&mut self, entry: QueryResultEntry) {
        self.entries.push(entry);
    }

    /// Get just the timeline entries
    pub fn get_entries(&self) -> Vec<&TimelineEntry> {
        self.entries.iter().map(|r| &r.entry).collect()
    }

    /// Get entries sorted by match score
    pub fn get_entries_by_score(&self) -> Vec<&QueryResultEntry> {
        let mut sorted = self.entries.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            b.match_score
                .partial_cmp(&a.match_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}

/// Timeline query executor
pub struct TimelineQueryExecutor {
    /// Reference to correlation tracker
    tracker: EventCorrelationTracker,
}

impl TimelineQueryExecutor {
    /// Create a new query executor
    pub fn new(tracker: EventCorrelationTracker) -> Self {
        Self { tracker }
    }

    /// Execute a timeline query
    pub fn execute_query(&self, query: TimelineQuery) -> QueryResult {
        let start_time = std::time::Instant::now();
        let mut result = QueryResult::new(query.clone());

        // Get all correlations to search
        let correlations = if let Some(ref correlation_ids) = query.correlation_ids {
            // Filter to specific correlations
            let mut filtered = std::collections::HashMap::new();
            for &id in correlation_ids {
                let events = self.tracker.get_events(&id);
                if !events.is_empty() {
                    filtered.insert(id, events);
                }
            }
            filtered
        } else {
            // Get all correlations
            self.tracker.get_all_correlations()
        };

        // Apply filters to each correlation
        let mut matching_entries = Vec::new();
        let correlations_count = correlations.len();
        let total_events_examined = correlations.values().map(Vec::len).sum::<usize>();

        for (correlation_id, events) in correlations {
            let timeline_entries = self.build_timeline_entries(events, correlation_id);

            for entry in timeline_entries {
                if let Some(result_entry) = self.evaluate_entry(&entry, &query) {
                    matching_entries.push(result_entry);
                }
            }
        }

        // Sort results
        self.sort_entries(&mut matching_entries, &query.sort_order);

        // Apply limit
        result.total_matches = matching_entries.len();
        if let Some(limit) = query.limit {
            matching_entries.truncate(limit);
        }

        result.entries = matching_entries;
        #[allow(clippy::cast_possible_truncation)]
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        result.execution_time_ms = elapsed_ms;

        // Add result metadata
        result.metadata.insert(
            "correlations_searched".to_string(),
            correlations_count.to_string(),
        );
        result.metadata.insert(
            "total_events_examined".to_string(),
            total_events_examined.to_string(),
        );

        result
    }

    /// Execute a simple query by event type pattern
    pub fn query_by_pattern(&self, pattern: &str) -> QueryResult {
        let query = TimelineQuery {
            event_types: Some(vec![pattern.to_string()]),
            ..Default::default()
        };
        self.execute_query(query)
    }

    /// Execute a query for events in a time range
    pub fn query_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> QueryResult {
        let query = TimelineQuery {
            time_range: Some(TimeRange::new(start, end)),
            ..Default::default()
        };
        self.execute_query(query)
    }

    /// Execute a query for events from a specific component
    pub fn query_by_component(&self, component: &str) -> QueryResult {
        let query = TimelineQuery {
            source_components: Some(vec![component.to_string()]),
            ..Default::default()
        };
        self.execute_query(query)
    }

    /// Execute a query for root cause events
    pub fn query_root_causes(&self) -> QueryResult {
        let query = TimelineQuery {
            root_causes_only: true,
            ..Default::default()
        };
        self.execute_query(query)
    }

    /// Execute a query for leaf effect events
    pub fn query_leaf_effects(&self) -> QueryResult {
        let query = TimelineQuery {
            leaf_effects_only: true,
            ..Default::default()
        };
        self.execute_query(query)
    }

    /// Build timeline entries from events
    fn build_timeline_entries(
        &self,
        events: Vec<UniversalEvent>,
        _correlation_id: Uuid,
    ) -> Vec<TimelineEntry> {
        if events.is_empty() {
            return Vec::new();
        }

        let start_time = events.iter().map(|e| e.timestamp).min().unwrap();
        let mut entries = Vec::new();

        for (position, event) in events.iter().enumerate() {
            let mut entry = TimelineEntry::new(event.clone(), position, start_time);

            // Analyze relationships for this event
            let links = self.tracker.get_links(&event.id);
            for link in links {
                match link.relationship {
                    EventRelationship::CausedBy
                    | EventRelationship::ResponseTo
                    | EventRelationship::FollowsFrom => {
                        entry.add_causing_event(link.from_event_id);
                    }
                    EventRelationship::ConcurrentWith => {
                        entry.add_concurrent_event(link.to_event_id);
                    }
                    _ => {}
                }
            }

            entries.push(entry);
        }

        entries
    }

    /// Evaluate if an entry matches the query
    fn evaluate_entry(
        &self,
        entry: &TimelineEntry,
        query: &TimelineQuery,
    ) -> Option<QueryResultEntry> {
        let mut match_score = 1.0;
        let mut match_reasons = Vec::new();

        // Time range filter
        if let Some(ref time_range) = query.time_range {
            if !time_range.contains(entry.event.timestamp) {
                return None;
            }
            match_reasons.push("time_range".to_string());
        }

        // Event type filter
        if let Some(ref event_types) = query.event_types {
            let matches = event_types
                .iter()
                .any(|pattern| entry.event.matches_pattern(pattern));
            if !matches {
                return None;
            }
            match_reasons.push("event_type".to_string());
        }

        // Source component filter
        if let Some(ref sources) = query.source_components {
            if let Some(ref source) = entry.event.metadata.source {
                if !sources.contains(source) {
                    return None;
                }
                match_reasons.push("source_component".to_string());
            } else {
                return None;
            }
        }

        // Target component filter
        if let Some(ref targets) = query.target_components {
            if let Some(ref target) = entry.event.metadata.target {
                if !targets.contains(target) {
                    return None;
                }
                match_reasons.push("target_component".to_string());
            } else {
                return None;
            }
        }

        // Language filter
        if let Some(ref languages) = query.languages {
            if !languages.contains(&entry.event.language) {
                return None;
            }
            match_reasons.push("language".to_string());
        }

        // Tags filter
        if let Some(ref tags) = query.tags {
            let entry_tags: HashSet<_> = entry.event.metadata.tags.iter().collect();
            let query_tags: HashSet<_> = tags.iter().collect();

            if query_tags.intersection(&entry_tags).count() == 0 {
                return None;
            }
            match_reasons.push("tags".to_string());
        }

        // Causality depth range filter
        if let Some((min_depth, max_depth)) = query.causality_depth_range {
            if entry.causality_depth < min_depth || entry.causality_depth > max_depth {
                return None;
            }
            match_reasons.push("causality_depth".to_string());
        }

        // Root causes only filter
        if query.root_causes_only && !entry.is_root_cause() {
            return None;
        }

        // Leaf effects only filter
        if query.leaf_effects_only && !entry.is_leaf_effect() {
            return None;
        }

        // Calculate match score based on multiple criteria
        if match_reasons.len() > 3 {
            match_score = 1.0; // Perfect match
        } else if match_reasons.len() > 1 {
            match_score = 0.8; // Good match
        } else if match_reasons.len() == 1 {
            match_score = 0.6; // Partial match
        }

        Some(QueryResultEntry {
            entry: entry.clone(),
            match_score,
            match_reasons,
        })
    }

    /// Sort query result entries
    fn sort_entries(&self, entries: &mut [QueryResultEntry], sort_order: &SortOrder) {
        match sort_order {
            SortOrder::ChronologicalAsc => {
                entries.sort_by_key(|e| e.entry.event.timestamp);
            }
            SortOrder::ChronologicalDesc => {
                entries.sort_by_key(|e| std::cmp::Reverse(e.entry.event.timestamp));
            }
            SortOrder::CausalityDepthAsc => {
                entries.sort_by_key(|e| e.entry.causality_depth);
            }
            SortOrder::CausalityDepthDesc => {
                entries.sort_by_key(|e| std::cmp::Reverse(e.entry.causality_depth));
            }
            SortOrder::SequenceOrder => {
                entries.sort_by_key(|e| e.entry.event.sequence);
            }
            SortOrder::CorrelationId => {
                entries.sort_by_key(|e| e.entry.event.metadata.correlation_id);
            }
        }
    }
}

/// Query builder for constructing complex timeline queries
pub struct TimelineQueryBuilder {
    query: TimelineQuery,
}

impl TimelineQueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            query: TimelineQuery::default(),
        }
    }

    /// Filter by correlation IDs
    pub fn correlation_ids(mut self, ids: Vec<Uuid>) -> Self {
        self.query.correlation_ids = Some(ids);
        self
    }

    /// Filter by event type patterns
    pub fn event_types(mut self, patterns: Vec<String>) -> Self {
        self.query.event_types = Some(patterns);
        self
    }

    /// Filter by time range
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.query.time_range = Some(TimeRange::new(start, end));
        self
    }

    /// Filter by the last N seconds
    pub fn last_seconds(mut self, seconds: i64) -> Self {
        self.query.time_range = Some(TimeRange::last_seconds(seconds));
        self
    }

    /// Filter by source components
    pub fn source_components(mut self, components: Vec<String>) -> Self {
        self.query.source_components = Some(components);
        self
    }

    /// Filter by target components
    pub fn target_components(mut self, components: Vec<String>) -> Self {
        self.query.target_components = Some(components);
        self
    }

    /// Filter by languages
    pub fn languages(mut self, languages: Vec<Language>) -> Self {
        self.query.languages = Some(languages);
        self
    }

    /// Filter by tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.query.tags = Some(tags);
        self
    }

    /// Filter by causality depth range
    pub fn causality_depth_range(mut self, min: u32, max: u32) -> Self {
        self.query.causality_depth_range = Some((min, max));
        self
    }

    /// Include only root cause events
    pub fn root_causes_only(mut self) -> Self {
        self.query.root_causes_only = true;
        self
    }

    /// Include only leaf effect events
    pub fn leaf_effects_only(mut self) -> Self {
        self.query.leaf_effects_only = true;
        self
    }

    /// Set sort order
    pub fn sort_by(mut self, order: SortOrder) -> Self {
        self.query.sort_order = order;
        self
    }

    /// Limit results
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// Build the query
    pub fn build(self) -> TimelineQuery {
        self.query
    }
}

impl Default for TimelineQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correlation::EventCorrelationTracker;
    use crate::universal_event::{Language, UniversalEvent};
    use serde_json::Value;

    // Local test helper to avoid circular dependency with llmspell-testing
    fn create_test_event(event_type: &str, correlation_id: Uuid) -> UniversalEvent {
        UniversalEvent::new(event_type, Value::Null, Language::Rust)
            .with_correlation_id(correlation_id)
    }
    #[test]
    fn test_time_range() {
        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now();
        let range = TimeRange::new(start, end);

        let now = Utc::now() - chrono::Duration::minutes(30);
        assert!(range.contains(now));

        let past = Utc::now() - chrono::Duration::hours(2);
        assert!(!range.contains(past));

        let future = Utc::now() + chrono::Duration::hours(1);
        assert!(!range.contains(future));
    }
    #[test]
    fn test_query_builder() {
        let correlation_id = Uuid::new_v4();
        let query = TimelineQueryBuilder::new()
            .correlation_ids(vec![correlation_id])
            .event_types(vec!["test.*".to_string()])
            .last_seconds(3600)
            .root_causes_only()
            .sort_by(SortOrder::ChronologicalDesc)
            .limit(100)
            .build();

        assert_eq!(query.correlation_ids, Some(vec![correlation_id]));
        assert_eq!(query.event_types, Some(vec!["test.*".to_string()]));
        assert!(query.time_range.is_some());
        assert!(query.root_causes_only);
        assert_eq!(query.limit, Some(100));
    }
    #[test]
    fn test_query_execution() {
        let tracker = EventCorrelationTracker::default();
        let correlation_id = Uuid::new_v4();

        // Add test events
        let event1 = create_test_event("system.start", correlation_id);
        let event2 = create_test_event("agent.execute", correlation_id);
        let event3 = create_test_event("system.end", correlation_id);

        tracker.track_event(event1);
        tracker.track_event(event2);
        tracker.track_event(event3);

        let executor = TimelineQueryExecutor::new(tracker);

        // Test pattern query
        let result = executor.query_by_pattern("system.*");
        assert_eq!(result.entries.len(), 2); // start and end events

        // Test component query
        let result = executor.query_by_component("test-component");
        // Should be 0 since our test events don't have source components set
        assert_eq!(result.entries.len(), 0);

        // Test time range query
        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let result = executor.query_by_time_range(start, end);
        assert_eq!(result.entries.len(), 3); // All events should be in range
    }
    #[test]
    fn test_query_result_sorting() {
        let query = TimelineQuery::default();
        let mut result = QueryResult::new(query);

        // Add entries with different scores
        let correlation_id = Uuid::new_v4();
        let event1 = create_test_event("event1", correlation_id);
        let event2 = create_test_event("event2", correlation_id);

        let entry1 = QueryResultEntry {
            entry: TimelineEntry::new(event1, 0, Utc::now()),
            match_score: 0.5,
            match_reasons: vec!["partial".to_string()],
        };

        let entry2 = QueryResultEntry {
            entry: TimelineEntry::new(event2, 1, Utc::now()),
            match_score: 0.9,
            match_reasons: vec!["good".to_string()],
        };

        result.add_entry(entry1);
        result.add_entry(entry2);

        let sorted = result.get_entries_by_score();
        assert_eq!(sorted.len(), 2);
        assert!(sorted[0].match_score > sorted[1].match_score);
    }
}
