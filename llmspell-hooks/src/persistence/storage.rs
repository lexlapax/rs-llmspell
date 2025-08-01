// ABOUTME: Hook-specific storage adapter for metadata and retention policies
// ABOUTME: Extends base storage with hook-specific functionality

use crate::types::ComponentType;
use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use uuid::Uuid;

use llmspell_state_traits::{StateManager, StateScope};

/// Hook-specific metadata for enhanced storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    /// Hook type (e.g., "rate_limit", "cost_tracking", "security")
    pub hook_type: String,
    /// Component that triggered the hook
    pub triggering_component: ComponentType,
    /// Component ID that triggered the hook
    pub component_id: String,
    /// Whether this hook modified the operation
    pub modified_operation: bool,
    /// Tags for categorization and searching
    pub tags: Vec<String>,
    /// Retention priority (higher = keep longer)
    pub retention_priority: i32,
    /// Custom properties specific to the hook type
    pub custom_properties: HashMap<String, serde_json::Value>,
    /// Timestamp when metadata was created
    pub created_at: SystemTime,
    /// Size of serialized hook context in bytes
    pub context_size: usize,
    /// Whether the hook context contains sensitive data
    pub contains_sensitive_data: bool,
}

impl HookMetadata {
    /// Create new hook metadata
    pub fn new(
        hook_type: String,
        triggering_component: ComponentType,
        component_id: String,
    ) -> Self {
        Self {
            hook_type,
            triggering_component,
            component_id,
            modified_operation: false,
            tags: Vec::new(),
            retention_priority: 0,
            custom_properties: HashMap::new(),
            created_at: SystemTime::now(),
            context_size: 0,
            contains_sensitive_data: false,
        }
    }

    /// Add a tag to the metadata
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Set a custom property
    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.custom_properties.insert(key, value);
    }

    /// Get a custom property
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_properties.get(key)
    }
}

/// Hook storage adapter for metadata persistence
pub struct HookStorageAdapter {
    /// In-memory cache for fast access
    metadata_cache: Arc<RwLock<HashMap<String, HookMetadata>>>,
    /// Statistics tracking
    storage_stats: Arc<RwLock<StorageStatistics>>,
    /// Optional persistent state manager
    persistent_state_manager: Option<Arc<dyn StateManager>>,
}

/// Storage statistics for monitoring
#[derive(Debug, Default)]
pub struct StorageStatistics {
    total_stored: u64,
    total_loaded: u64,
    cache_hits: u64,
    cache_misses: u64,
    average_metadata_size: usize,
}

impl Default for HookStorageAdapter {
    fn default() -> Self {
        Self {
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            storage_stats: Arc::new(RwLock::new(StorageStatistics::default())),
            persistent_state_manager: None,
        }
    }
}

impl HookStorageAdapter {
    /// Create a new hook storage adapter
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure with persistent state manager
    pub fn with_persistent_state(mut self, state_manager: Arc<dyn StateManager>) -> Self {
        self.persistent_state_manager = Some(state_manager);
        self
    }

    /// Set persistent state manager
    pub fn set_persistent_state_manager(&mut self, state_manager: Arc<dyn StateManager>) {
        self.persistent_state_manager = Some(state_manager);
    }

    /// Store hook metadata
    pub async fn store_metadata(
        &self,
        correlation_id: &Uuid,
        metadata: &HookMetadata,
    ) -> Result<()> {
        let key = self.generate_metadata_key(correlation_id);

        // Update cache
        self.metadata_cache
            .write()
            .insert(key.clone(), metadata.clone());

        // Store in persistent storage if available
        if let Some(state_manager) = &self.persistent_state_manager {
            let scope = StateScope::Custom(format!("hook_metadata_{}", metadata.hook_type));
            state_manager
                .set(scope, &key, serde_json::to_value(metadata)?)
                .await
                .context("Failed to store metadata in persistent storage")?;
        }

        // Update statistics
        let mut stats = self.storage_stats.write();
        stats.total_stored += 1;

        // Calculate running average of metadata size
        let serialized = serde_json::to_vec(metadata).context("Failed to serialize metadata")?;
        let size = serialized.len();

        if stats.total_stored == 1 {
            stats.average_metadata_size = size;
        } else {
            stats.average_metadata_size =
                ((stats.average_metadata_size * (stats.total_stored - 1) as usize) + size)
                    / stats.total_stored as usize;
        }

        Ok(())
    }

    /// Load hook metadata
    pub async fn load_metadata(&self, correlation_id: &Uuid) -> Result<Option<HookMetadata>> {
        let key = self.generate_metadata_key(correlation_id);

        // Check cache first
        if let Some(metadata) = self.metadata_cache.read().get(&key) {
            self.storage_stats.write().cache_hits += 1;
            return Ok(Some(metadata.clone()));
        }

        // Cache miss - try to load from persistent storage
        if let Some(state_manager) = &self.persistent_state_manager {
            // Try different hook types to find the metadata
            let hook_types = [
                "rate_limit",
                "cost_tracking",
                "security",
                "logging",
                "metrics",
            ];
            for hook_type in &hook_types {
                let scope = StateScope::Custom(format!("hook_metadata_{}", hook_type));
                if let Ok(Some(value)) = state_manager.get(scope, &key).await {
                    if let Ok(metadata) = serde_json::from_value::<HookMetadata>(value) {
                        // Cache the loaded metadata
                        self.metadata_cache.write().insert(key, metadata.clone());
                        self.storage_stats.write().cache_misses += 1;
                        self.storage_stats.write().total_loaded += 1;
                        return Ok(Some(metadata));
                    }
                }
            }
        }

        // Not found anywhere
        self.storage_stats.write().cache_misses += 1;
        self.storage_stats.write().total_loaded += 1;
        Ok(None)
    }

    /// Delete metadata
    pub async fn delete_metadata(&self, correlation_id: &Uuid) -> Result<()> {
        let key = self.generate_metadata_key(correlation_id);
        self.metadata_cache.write().remove(&key);
        Ok(())
    }

    /// List metadata keys by prefix
    pub async fn list_metadata_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let cache = self.metadata_cache.read();
        let keys: Vec<String> = cache
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    /// Get storage statistics
    pub fn get_statistics(&self) -> StorageStatistics {
        self.storage_stats.read().clone()
    }

    /// Clear the metadata cache
    pub fn clear_cache(&self) {
        self.metadata_cache.write().clear();
    }

    /// Generate a metadata storage key
    fn generate_metadata_key(&self, correlation_id: &Uuid) -> String {
        format!("hook_metadata:{}", correlation_id)
    }
}

impl Clone for StorageStatistics {
    fn clone(&self) -> Self {
        Self {
            total_stored: self.total_stored,
            total_loaded: self.total_loaded,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            average_metadata_size: self.average_metadata_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_metadata_creation() {
        let mut metadata = HookMetadata::new(
            "rate_limit".to_string(),
            ComponentType::Agent,
            "test-agent".to_string(),
        );

        assert_eq!(metadata.hook_type, "rate_limit");
        assert_eq!(metadata.retention_priority, 0);
        assert!(!metadata.modified_operation);

        metadata.add_tag("important".to_string());
        assert_eq!(metadata.tags.len(), 1);

        metadata.set_property("limit".to_string(), serde_json::json!(100));
        assert_eq!(
            metadata.get_property("limit"),
            Some(&serde_json::json!(100))
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_storage_adapter() {
        let adapter = HookStorageAdapter::new();
        let correlation_id = Uuid::new_v4();

        let metadata = HookMetadata::new(
            "test".to_string(),
            ComponentType::Tool,
            "calculator".to_string(),
        );

        // Store metadata
        adapter
            .store_metadata(&correlation_id, &metadata)
            .await
            .unwrap();

        // Load from cache
        let loaded = adapter.load_metadata(&correlation_id).await.unwrap();
        assert!(loaded.is_some());

        let stats = adapter.get_statistics();
        assert_eq!(stats.total_stored, 1);
        assert_eq!(stats.cache_hits, 1);
    }
}
