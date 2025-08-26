// ABOUTME: Storage adapter that bridges event domain to unified storage backend
// ABOUTME: Implements EventStorage using llmspell-storage's StorageBackend trait

use crate::universal_event::UniversalEvent;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_storage::{StorageBackend, StorageSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Event storage interface (domain-specific)
#[async_trait]
pub trait EventStorage: Send + Sync {
    /// Store an event
    async fn store_event(&self, event: &UniversalEvent) -> Result<()>;

    /// Retrieve events by pattern
    async fn get_events_by_pattern(&self, pattern: &str) -> Result<Vec<UniversalEvent>>;

    /// Retrieve events in a time range
    async fn get_events_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<UniversalEvent>>;

    /// Retrieve events by correlation ID
    async fn get_events_by_correlation_id(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<UniversalEvent>>;

    /// Delete events older than a certain time
    async fn cleanup_old_events(&self, before: DateTime<Utc>) -> Result<usize>;

    /// Get storage statistics
    async fn get_storage_stats(&self) -> Result<StorageStats>;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    /// Total number of stored events
    pub total_events: u64,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    /// Oldest event timestamp
    pub oldest_event: Option<DateTime<Utc>>,
    /// Newest event timestamp
    pub newest_event: Option<DateTime<Utc>>,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
}

/// Event storage adapter that uses any StorageBackend
pub struct EventStorageAdapter<B: StorageBackend> {
    backend: Arc<B>,
}

impl<B: StorageBackend> EventStorageAdapter<B> {
    /// Create new adapter with storage backend
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }

    /// Generate event key for storage
    /// Format: "event:{timestamp}:{sequence}:{id}"
    fn event_key(event: &UniversalEvent) -> String {
        format!(
            "event:{}:{}:{}",
            event.timestamp.format("%Y%m%dT%H%M%S%.3fZ"),
            event.sequence,
            event.id
        )
    }

    /// Generate pattern reference key
    /// Format: "pattern:{pattern}:{event_id}"
    fn pattern_key(pattern: &str, event_id: &Uuid) -> String {
        format!("pattern:{}:{}", pattern, event_id)
    }

    /// Generate correlation reference key
    /// Format: "correlation:{correlation_id}:{event_id}"
    fn correlation_key(correlation_id: &Uuid, event_id: &Uuid) -> String {
        format!("correlation:{}:{}", correlation_id, event_id)
    }

    /// Generate stats key
    fn stats_key() -> &'static str {
        "event_stats"
    }
}

#[async_trait]
impl<B: StorageBackend> EventStorage for EventStorageAdapter<B> {
    async fn store_event(&self, event: &UniversalEvent) -> Result<()> {
        let event_key = Self::event_key(event);
        let event_data = event.to_storage_bytes()?;

        // Store the event
        self.backend.set(&event_key, event_data).await?;

        // Create pattern reference for efficient pattern matching
        let pattern_key = Self::pattern_key(&event.event_type, &event.id);
        let reference_data = event.id.to_storage_bytes()?;
        self.backend
            .set(&pattern_key, reference_data.clone())
            .await?;

        // Create correlation reference
        let correlation_key = Self::correlation_key(&event.metadata.correlation_id, &event.id);
        self.backend.set(&correlation_key, reference_data).await?;

        // Update statistics
        self.update_stats_for_new_event(event).await?;

        Ok(())
    }

    async fn get_events_by_pattern(&self, pattern: &str) -> Result<Vec<UniversalEvent>> {
        // Get all pattern keys that match
        let pattern_prefix = format!("pattern:{}", pattern.replace('*', ""));
        let pattern_keys = self.backend.list_keys(&pattern_prefix).await?;

        let mut events = Vec::new();

        for pattern_key in pattern_keys {
            // Extract the actual pattern from the key
            if let Some(stored_pattern) = pattern_key
                .strip_prefix("pattern:")
                .and_then(|s| s.rsplit(':').nth(1))
            {
                if crate::pattern::PatternMatcher::new().matches(stored_pattern, pattern) {
                    // Get the event ID from the pattern key
                    if let Some(event_id_str) = pattern_key.rsplit(':').next() {
                        if let Ok(event_id) = event_id_str.parse::<Uuid>() {
                            // Find the actual event by scanning event keys
                            let event_prefix = "event:";
                            let event_keys = self.backend.list_keys(event_prefix).await?;

                            for event_key in event_keys {
                                if event_key.ends_with(&event_id.to_string()) {
                                    if let Some(event_data) = self.backend.get(&event_key).await? {
                                        let event =
                                            UniversalEvent::from_storage_bytes(&event_data)?;
                                        events.push(event);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by sequence number
        events.sort_by_key(|e| e.sequence);
        Ok(events)
    }

    async fn get_events_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<UniversalEvent>> {
        let event_prefix = "event:";
        let event_keys = self.backend.list_keys(event_prefix).await?;

        let mut events = Vec::new();

        for event_key in event_keys {
            if let Some(event_data) = self.backend.get(&event_key).await? {
                let event = UniversalEvent::from_storage_bytes(&event_data)?;
                if event.timestamp >= start && event.timestamp <= end {
                    events.push(event);
                }
            }
        }

        events.sort_by_key(|e| e.sequence);
        Ok(events)
    }

    async fn get_events_by_correlation_id(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<UniversalEvent>> {
        let correlation_prefix = format!("correlation:{}", correlation_id);
        let correlation_keys = self.backend.list_keys(&correlation_prefix).await?;

        let mut events = Vec::new();

        for correlation_key in correlation_keys {
            if let Some(event_id_data) = self.backend.get(&correlation_key).await? {
                let event_id = Uuid::from_storage_bytes(&event_id_data)?;

                // Find the actual event
                let event_prefix = "event:";
                let event_keys = self.backend.list_keys(event_prefix).await?;

                for event_key in event_keys {
                    if event_key.ends_with(&event_id.to_string()) {
                        if let Some(event_data) = self.backend.get(&event_key).await? {
                            let event = UniversalEvent::from_storage_bytes(&event_data)?;
                            events.push(event);
                            break;
                        }
                    }
                }
            }
        }

        events.sort_by_key(|e| e.sequence);
        Ok(events)
    }

    async fn cleanup_old_events(&self, before: DateTime<Utc>) -> Result<usize> {
        let event_prefix = "event:";
        let event_keys = self.backend.list_keys(event_prefix).await?;

        let mut deleted_count = 0;
        let mut keys_to_delete = Vec::new();

        for event_key in event_keys {
            if let Some(event_data) = self.backend.get(&event_key).await? {
                let event = UniversalEvent::from_storage_bytes(&event_data)?;
                if event.timestamp < before {
                    keys_to_delete.push(event_key);

                    // Also delete pattern and correlation references
                    let pattern_key = Self::pattern_key(&event.event_type, &event.id);
                    let correlation_key =
                        Self::correlation_key(&event.metadata.correlation_id, &event.id);
                    keys_to_delete.push(pattern_key);
                    keys_to_delete.push(correlation_key);

                    deleted_count += 1;
                }
            }
        }

        if !keys_to_delete.is_empty() {
            self.backend.delete_batch(&keys_to_delete).await?;
        }

        Ok(deleted_count)
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
        // Try to get cached stats first
        let stats_key = Self::stats_key();
        if let Some(stats_data) = self.backend.get(stats_key).await? {
            if let Ok(stats) = StorageStats::from_storage_bytes(&stats_data) {
                return Ok(stats);
            }
        }

        // Rebuild stats from scratch
        let event_prefix = "event:";
        let event_keys = self.backend.list_keys(event_prefix).await?;

        let mut stats = StorageStats::default();

        for event_key in event_keys {
            if let Some(event_data) = self.backend.get(&event_key).await? {
                let event = UniversalEvent::from_storage_bytes(&event_data)?;

                stats.total_events += 1;
                stats.storage_size_bytes += event_data.len() as u64;

                // Update oldest/newest
                if stats.oldest_event.is_none() || Some(event.timestamp) < stats.oldest_event {
                    stats.oldest_event = Some(event.timestamp);
                }
                if stats.newest_event.is_none() || Some(event.timestamp) > stats.newest_event {
                    stats.newest_event = Some(event.timestamp);
                }

                // Update events by type
                *stats
                    .events_by_type
                    .entry(event.event_type.clone())
                    .or_insert(0) += 1;
            }
        }

        // Cache the stats
        let stats_data = stats.to_storage_bytes()?;
        let _ = self.backend.set(stats_key, stats_data).await; // Don't fail if caching fails

        Ok(stats)
    }
}

impl<B: StorageBackend> EventStorageAdapter<B> {
    /// Update statistics when a new event is stored
    async fn update_stats_for_new_event(&self, event: &UniversalEvent) -> Result<()> {
        let stats_key = Self::stats_key();

        let mut stats = if let Some(stats_data) = self.backend.get(stats_key).await? {
            StorageStats::from_storage_bytes(&stats_data).unwrap_or_default()
        } else {
            StorageStats::default()
        };

        stats.total_events += 1;
        stats.storage_size_bytes += event.to_json().unwrap_or_default().len() as u64;

        // Update oldest/newest
        if stats.oldest_event.is_none() || Some(event.timestamp) < stats.oldest_event {
            stats.oldest_event = Some(event.timestamp);
        }
        if stats.newest_event.is_none() || Some(event.timestamp) > stats.newest_event {
            stats.newest_event = Some(event.timestamp);
        }

        // Update events by type
        *stats
            .events_by_type
            .entry(event.event_type.clone())
            .or_insert(0) += 1;

        // Store updated stats
        let stats_data = stats.to_storage_bytes()?;
        self.backend.set(stats_key, stats_data).await?;

        Ok(())
    }
}

/// Event persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Enable event persistence
    pub enabled: bool,
    /// Maximum events to store before cleanup
    pub max_events: Option<usize>,
    /// Automatic cleanup interval
    pub cleanup_interval: Option<std::time::Duration>,
    /// Event TTL for automatic cleanup
    pub event_ttl: Option<std::time::Duration>,
    /// Patterns to persist (None means persist all)
    pub persist_patterns: Option<Vec<String>>,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_events: Some(100_000),
            cleanup_interval: Some(std::time::Duration::from_secs(3600)), // 1 hour
            event_ttl: Some(std::time::Duration::from_secs(86400 * 7)),   // 7 days
            persist_patterns: None,                                       // Persist all events
        }
    }
}

/// Event persistence manager using storage adapter
pub struct EventPersistenceManager<B: StorageBackend> {
    storage: Arc<EventStorageAdapter<B>>,
    config: PersistenceConfig,
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl<B: StorageBackend + 'static> EventPersistenceManager<B> {
    /// Create a new persistence manager
    pub fn new(storage: EventStorageAdapter<B>, config: PersistenceConfig) -> Self {
        Self {
            storage: Arc::new(storage),
            config,
            cleanup_task: None,
        }
    }

    /// Start the persistence manager with automatic cleanup
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // For now, disable automatic cleanup to avoid thread safety issues
        // This will be implemented properly in a future version
        tracing::info!("Persistence manager started (automatic cleanup disabled for now)");

        Ok(())
    }

    /// Stop the persistence manager
    pub async fn stop(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
            let _ = task.await;
        }
    }

    /// Store an event if it matches persistence criteria
    pub async fn maybe_store_event(&self, event: &UniversalEvent) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        // Check if event matches persistence patterns
        if let Some(patterns) = &self.config.persist_patterns {
            let matches = patterns
                .iter()
                .any(|pattern| event.matches_pattern(pattern));
            if !matches {
                return Ok(false);
            }
        }

        self.storage.store_event(event).await?;
        Ok(true)
    }

    /// Get storage reference
    pub fn storage(&self) -> &EventStorageAdapter<B> {
        &self.storage
    }
}

impl<B: StorageBackend> Drop for EventPersistenceManager<B> {
    fn drop(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_event::{Language, UniversalEvent};
    use llmspell_storage::MemoryBackend;
    use serde_json::Value;

    fn create_test_event(event_type: &str) -> UniversalEvent {
        UniversalEvent::new(event_type, Value::Null, Language::Rust)
    }
    #[tokio::test]
    async fn test_storage_adapter_basic_operations() {
        let backend = MemoryBackend::new();
        let adapter = EventStorageAdapter::new(backend);
        let event = create_test_event("test.event");

        adapter.store_event(&event).await.unwrap();

        let events = adapter.get_events_by_pattern("test.*").await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test.event");
    }
    #[tokio::test]
    async fn test_storage_stats() {
        let backend = MemoryBackend::new();
        let adapter = EventStorageAdapter::new(backend);

        let event1 = create_test_event("test.event1");
        let event2 = create_test_event("test.event2");

        adapter.store_event(&event1).await.unwrap();
        adapter.store_event(&event2).await.unwrap();

        let stats = adapter.get_storage_stats().await.unwrap();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.events_by_type["test.event1"], 1);
        assert_eq!(stats.events_by_type["test.event2"], 1);
    }
    #[tokio::test]
    async fn test_persistence_manager() {
        let backend = MemoryBackend::new();
        let adapter = EventStorageAdapter::new(backend);
        let config = PersistenceConfig {
            enabled: true,
            persist_patterns: Some(vec!["important.*".to_string()]),
            ..Default::default()
        };

        let manager = EventPersistenceManager::new(adapter, config);

        let important_event = create_test_event("important.event");
        let regular_event = create_test_event("regular.event");

        let stored1 = manager.maybe_store_event(&important_event).await.unwrap();
        let stored2 = manager.maybe_store_event(&regular_event).await.unwrap();

        assert!(stored1); // Should be stored
        assert!(!stored2); // Should not be stored

        let events = manager.storage().get_events_by_pattern("*").await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "important.event");
    }
}
