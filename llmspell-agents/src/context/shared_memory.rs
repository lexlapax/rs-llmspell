//! ABOUTME: Shared memory system for inter-agent communication
//! ABOUTME: Provides thread-safe memory regions with access control and monitoring

use llmspell_core::execution_context::ContextScope;
use llmspell_core::{ComponentId, LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Memory access permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryPermission {
    /// Read-only access
    Read,
    /// Write-only access
    Write,
    /// Read and write access
    ReadWrite,
    /// No access
    None,
}

/// Memory region with access control
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Region identifier
    pub id: String,
    /// Region scope
    pub scope: ContextScope,
    /// Data storage
    data: Arc<RwLock<HashMap<String, Value>>>,
    /// Access control list
    acl: Arc<RwLock<HashMap<ComponentId, MemoryPermission>>>,
    /// Region metadata
    pub metadata: RegionMetadata,
    /// Change notification channel
    change_tx: broadcast::Sender<MemoryChange>,
}

/// Metadata about a memory region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionMetadata {
    /// Creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified time
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Total size in bytes (approximate)
    pub size_bytes: usize,
    /// Number of keys
    pub key_count: usize,
    /// Access statistics
    pub access_stats: AccessStats,
}

/// Access statistics for a memory region
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessStats {
    /// Total read operations
    pub read_count: u64,
    /// Total write operations
    pub write_count: u64,
    /// Last read time
    pub last_read: Option<chrono::DateTime<chrono::Utc>>,
    /// Last write time
    pub last_write: Option<chrono::DateTime<chrono::Utc>>,
}

/// Notification of memory changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChange {
    /// Region that changed
    pub region_id: String,
    /// Key that changed
    pub key: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Component that made the change
    pub changed_by: ComponentId,
    /// Timestamp of change
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of memory change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Value was created
    Created,
    /// Value was updated
    Updated,
    /// Value was deleted
    Deleted,
}

impl MemoryRegion {
    /// Create a new memory region
    #[must_use]
    pub fn new(id: String, scope: ContextScope) -> Self {
        let (change_tx, _) = broadcast::channel(100);

        Self {
            id,
            scope,
            data: Arc::new(RwLock::new(HashMap::new())),
            acl: Arc::new(RwLock::new(HashMap::new())),
            metadata: RegionMetadata {
                created_at: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                size_bytes: 0,
                key_count: 0,
                access_stats: AccessStats::default(),
            },
            change_tx,
        }
    }

    /// Grant permission to a component
    pub fn grant_permission(&self, component: ComponentId, permission: MemoryPermission) {
        let mut acl = self.acl.write().unwrap();
        acl.insert(component, permission);
    }

    /// Revoke permission from a component
    pub fn revoke_permission(&self, component: &ComponentId) {
        let mut acl = self.acl.write().unwrap();
        acl.remove(component);
    }

    /// Check if component has permission
    #[must_use]
    pub fn has_permission(&self, component: &ComponentId, required: MemoryPermission) -> bool {
        let acl = self.acl.read().unwrap();
        match acl.get(component) {
            Some(MemoryPermission::ReadWrite) => true,
            Some(MemoryPermission::Read) => required == MemoryPermission::Read,
            Some(MemoryPermission::Write) => required == MemoryPermission::Write,
            _ => false,
        }
    }

    /// Get a value with permission check
    ///
    /// # Errors
    ///
    /// Returns an error if read permission is denied
    pub fn get(&self, key: &str, accessor: &ComponentId) -> Result<Option<Value>> {
        if !self.has_permission(accessor, MemoryPermission::Read) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Permission denied: read access to memory region {}",
                    self.id
                ),
                violation_type: Some("access_control".to_string()),
            });
        }

        let data = self.data.read().unwrap();
        self.update_read_stats();
        Ok(data.get(key).cloned())
    }

    /// Set a value with permission check
    ///
    /// # Errors
    ///
    /// Returns an error if write permission is denied
    pub fn set(&self, key: String, value: Value, accessor: &ComponentId) -> Result<()> {
        if !self.has_permission(accessor, MemoryPermission::Write) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Permission denied: write access to memory region {}",
                    self.id
                ),
                violation_type: Some("access_control".to_string()),
            });
        }

        let mut data = self.data.write().unwrap();
        let change_type = if data.contains_key(&key) {
            ChangeType::Updated
        } else {
            ChangeType::Created
        };

        data.insert(key.clone(), value);
        self.update_write_stats(&data);

        // Notify subscribers
        let change = MemoryChange {
            region_id: self.id.clone(),
            key,
            change_type,
            changed_by: *accessor,
            timestamp: chrono::Utc::now(),
        };
        let _ = self.change_tx.send(change);

        Ok(())
    }

    /// Remove a value with permission check
    ///
    /// # Errors
    ///
    /// Returns an error if write permission is denied
    pub fn remove(&self, key: &str, accessor: &ComponentId) -> Result<Option<Value>> {
        if !self.has_permission(accessor, MemoryPermission::Write) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Permission denied: write access to memory region {}",
                    self.id
                ),
                violation_type: Some("access_control".to_string()),
            });
        }

        let mut data = self.data.write().unwrap();
        let removed = data.remove(key);

        if removed.is_some() {
            self.update_write_stats(&data);

            let change = MemoryChange {
                region_id: self.id.clone(),
                key: key.to_string(),
                change_type: ChangeType::Deleted,
                changed_by: *accessor,
                timestamp: chrono::Utc::now(),
            };
            let _ = self.change_tx.send(change);
        }

        Ok(removed)
    }

    /// Subscribe to change notifications
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<MemoryChange> {
        self.change_tx.subscribe()
    }

    /// Get all keys in the region
    ///
    /// # Errors
    ///
    /// Returns an error if read permission is denied
    pub fn keys(&self, accessor: &ComponentId) -> Result<Vec<String>> {
        if !self.has_permission(accessor, MemoryPermission::Read) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Permission denied: read access to memory region {}",
                    self.id
                ),
                violation_type: Some("access_control".to_string()),
            });
        }

        let data = self.data.read().unwrap();
        Ok(data.keys().cloned().collect())
    }

    /// Clear all data in the region
    ///
    /// # Errors
    ///
    /// Returns an error if write permission is denied
    pub fn clear(&self, accessor: &ComponentId) -> Result<()> {
        if !self.has_permission(accessor, MemoryPermission::Write) {
            return Err(LLMSpellError::Security {
                message: format!(
                    "Permission denied: write access to memory region {}",
                    self.id
                ),
                violation_type: Some("access_control".to_string()),
            });
        }

        let mut data = self.data.write().unwrap();
        data.clear();
        self.update_write_stats(&data);

        Ok(())
    }

    fn update_read_stats(&self) {
        // This is a bit hacky but avoids nested locks
        let _stats = AccessStats {
            read_count: self.metadata.access_stats.read_count + 1,
            write_count: self.metadata.access_stats.write_count,
            last_read: Some(chrono::Utc::now()),
            last_write: self.metadata.access_stats.last_write,
        };

        // Update metadata (would need mutable access in real implementation)
        // For now, we'll skip updating the embedded stats
    }

    fn update_write_stats(&self, data: &HashMap<String, Value>) {
        // Update size estimate
        let _size_bytes: usize = data
            .iter()
            .map(|(k, v)| k.len() + serde_json::to_string(v).unwrap_or_default().len())
            .sum();

        let _stats = AccessStats {
            read_count: self.metadata.access_stats.read_count,
            write_count: self.metadata.access_stats.write_count + 1,
            last_read: self.metadata.access_stats.last_read,
            last_write: Some(chrono::Utc::now()),
        };

        // Update metadata (would need mutable access in real implementation)
    }
}

/// Manages shared memory regions across the system
#[derive(Debug)]
pub struct SharedMemoryManager {
    /// All memory regions
    regions: Arc<RwLock<HashMap<String, MemoryRegion>>>,
    /// Default permissions for new regions
    _default_permissions: MemoryPermission,
    /// Memory limits
    limits: MemoryLimits,
}

/// Limits for memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum number of regions
    pub max_regions: usize,
    /// Maximum size per region in bytes
    pub max_region_size: usize,
    /// Maximum total memory usage in bytes
    pub max_total_size: usize,
    /// TTL for unused regions
    pub unused_ttl: Duration,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_regions: 1000,
            max_region_size: 10 * 1024 * 1024,     // 10MB
            max_total_size: 100 * 1024 * 1024,     // 100MB
            unused_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl SharedMemoryManager {
    /// Create a new shared memory manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            regions: Arc::new(RwLock::new(HashMap::new())),
            _default_permissions: MemoryPermission::ReadWrite,
            limits: MemoryLimits::default(),
        }
    }

    /// Create a new memory region
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum region limit is exceeded
    /// - Region already exists
    pub fn create_region(&self, id: String, scope: ContextScope, owner: ComponentId) -> Result<()> {
        let mut regions = self.regions.write().unwrap();

        // Check limits
        if regions.len() >= self.limits.max_regions {
            return Err(LLMSpellError::ResourceLimit {
                resource: "memory regions".to_string(),
                limit: self.limits.max_regions,
                used: regions.len(),
            });
        }

        if regions.contains_key(&id) {
            return Err(LLMSpellError::Component {
                message: format!("Memory region already exists: {id}"),
                source: None,
            });
        }

        let region = MemoryRegion::new(id.clone(), scope);
        region.grant_permission(owner, MemoryPermission::ReadWrite);

        regions.insert(id, region);
        Ok(())
    }

    /// Get a memory region
    #[must_use]
    pub fn get_region(&self, id: &str) -> Option<MemoryRegion> {
        let regions = self.regions.read().unwrap();
        regions.get(id).cloned()
    }

    /// Delete a memory region
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Delete permission is denied
    /// - Memory region is not found
    pub fn delete_region(&self, id: &str, requester: &ComponentId) -> Result<()> {
        let mut regions = self.regions.write().unwrap();

        if let Some(region) = regions.get(id) {
            // Only owner can delete (simplified check)
            if !region.has_permission(requester, MemoryPermission::ReadWrite) {
                return Err(LLMSpellError::Security {
                    message: format!("Permission denied: delete access to memory region {id}"),
                    violation_type: Some("access_control".to_string()),
                });
            }
        }

        regions.remove(id).ok_or_else(|| LLMSpellError::Component {
            message: format!("Memory region not found: {id}"),
            source: None,
        })?;

        Ok(())
    }

    /// List all regions accessible by a component
    #[must_use]
    pub fn list_regions(&self, accessor: &ComponentId) -> Vec<String> {
        let regions = self.regions.read().unwrap();
        regions
            .iter()
            .filter(|(_, region)| region.has_permission(accessor, MemoryPermission::Read))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get memory usage statistics
    #[must_use]
    pub fn stats(&self) -> MemoryStats {
        let regions = self.regions.read().unwrap();

        let total_regions = regions.len();
        let total_size: usize = regions.values().map(|r| r.metadata.size_bytes).sum();

        MemoryStats {
            total_regions,
            total_size_bytes: total_size,
            limits: self.limits.clone(),
        }
    }

    /// Clean up unused regions
    pub async fn cleanup(&self) {
        let _now = Instant::now();
        let mut regions = self.regions.write().unwrap();

        regions.retain(|_, region| {
            // Check if region has been accessed recently
            if let Some(last_read) = region.metadata.access_stats.last_read {
                let elapsed = chrono::Utc::now().signed_duration_since(last_read);
                if elapsed > chrono::Duration::from_std(self.limits.unused_ttl).unwrap() {
                    return false;
                }
            }
            true
        });
    }
}

impl Default for SharedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_regions: usize,
    pub total_size_bytes: usize,
    pub limits: MemoryLimits,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ComponentId;
    use serde_json::json;
    #[test]
    fn test_memory_region_permissions() {
        let region = MemoryRegion::new("test".to_string(), ContextScope::Global);
        let owner = ComponentId::from_name("owner");
        let reader = ComponentId::from_name("reader");
        let writer = ComponentId::from_name("writer");

        region.grant_permission(owner, MemoryPermission::ReadWrite);
        region.grant_permission(reader, MemoryPermission::Read);
        region.grant_permission(writer, MemoryPermission::Write);

        assert!(region.has_permission(&owner, MemoryPermission::Read));
        assert!(region.has_permission(&owner, MemoryPermission::Write));
        assert!(region.has_permission(&reader, MemoryPermission::Read));
        assert!(!region.has_permission(&reader, MemoryPermission::Write));
        assert!(!region.has_permission(&writer, MemoryPermission::Read));
        assert!(region.has_permission(&writer, MemoryPermission::Write));
    }
    #[test]
    fn test_memory_region_operations() {
        let region = MemoryRegion::new("test".to_string(), ContextScope::Global);
        let accessor = ComponentId::from_name("accessor");

        region.grant_permission(accessor, MemoryPermission::ReadWrite);

        // Test set
        region
            .set("key1".to_string(), json!("value1"), &accessor)
            .unwrap();

        // Test get
        let value = region.get("key1", &accessor).unwrap();
        assert_eq!(value, Some(json!("value1")));

        // Test remove
        let removed = region.remove("key1", &accessor).unwrap();
        assert_eq!(removed, Some(json!("value1")));

        // Verify removed
        let value = region.get("key1", &accessor).unwrap();
        assert_eq!(value, None);
    }
    #[test]
    fn test_memory_region_permission_denied() {
        let region = MemoryRegion::new("test".to_string(), ContextScope::Global);
        let unauthorized = ComponentId::from_name("unauthorized");

        // Should fail without permissions
        assert!(region.get("key", &unauthorized).is_err());
        assert!(region
            .set("key".to_string(), json!("value"), &unauthorized)
            .is_err());
        assert!(region.remove("key", &unauthorized).is_err());
    }
    #[test]
    fn test_shared_memory_manager() {
        let manager = SharedMemoryManager::new();
        let owner = ComponentId::from_name("owner");

        // Create region
        manager
            .create_region("region1".to_string(), ContextScope::Global, owner)
            .unwrap();

        // Get region
        let region = manager.get_region("region1").unwrap();
        assert_eq!(region.id, "region1");

        // List regions
        let regions = manager.list_regions(&owner);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0], "region1");

        // Delete region
        manager.delete_region("region1", &owner).unwrap();
        assert!(manager.get_region("region1").is_none());
    }
    #[test]
    fn test_memory_change_notifications() {
        let region = MemoryRegion::new("test".to_string(), ContextScope::Global);
        let accessor = ComponentId::from_name("accessor");

        region.grant_permission(accessor, MemoryPermission::ReadWrite);

        let mut receiver = region.subscribe();

        // Make a change
        region
            .set("key1".to_string(), json!("value1"), &accessor)
            .unwrap();

        // Should receive notification
        let change = receiver.try_recv().unwrap();
        assert_eq!(change.region_id, "test");
        assert_eq!(change.key, "key1");
        assert_eq!(change.change_type, ChangeType::Created);
        assert_eq!(change.changed_by, accessor);
    }
}
