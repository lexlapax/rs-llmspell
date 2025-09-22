//! Multi-tenant vector manager implementation

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use llmspell_core::state::StateScope;
use llmspell_storage::{
    ScopedStats, StorageStats, VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::traits::{TenantConfig, TenantLimits};
use super::usage::{TenantUsageTracker, UsageMetrics};
use llmspell_events::{EventBus, Language, UniversalEvent};
use serde_json::json;

/// Tenant runtime information
#[derive(Debug, Clone)]
pub struct TenantInfo {
    /// Tenant configuration
    pub config: TenantConfig,

    /// Current usage metrics
    pub usage: UsageMetrics,

    /// StateScope for this tenant
    pub scope: StateScope,
}

/// Multi-tenant vector manager with isolation
pub struct MultiTenantVectorManager {
    /// Underlying vector storage implementation
    storage: Arc<dyn VectorStorage>,

    /// Tenant configurations
    tenants: DashMap<String, Arc<RwLock<TenantInfo>>>,

    /// Usage tracker
    usage_tracker: Arc<TenantUsageTracker>,

    /// Default limits for new tenants
    default_limits: TenantLimits,

    /// Event bus for notifications
    event_bus: Option<Arc<EventBus>>,
}

impl MultiTenantVectorManager {
    /// Create a new multi-tenant manager
    pub fn new(storage: Arc<dyn VectorStorage>) -> Self {
        Self {
            storage,
            tenants: DashMap::new(),
            usage_tracker: Arc::new(TenantUsageTracker::new()),
            default_limits: TenantLimits::default(),
            event_bus: None,
        }
    }

    /// Create with event bus for notifications
    pub fn with_event_bus(storage: Arc<dyn VectorStorage>, event_bus: Arc<EventBus>) -> Self {
        Self {
            storage,
            tenants: DashMap::new(),
            usage_tracker: Arc::new(TenantUsageTracker::new()),
            default_limits: TenantLimits::default(),
            event_bus: Some(event_bus),
        }
    }

    /// Emit an event to the event bus
    async fn emit_event(&self, tenant_id: &str, event_type: &str, data: serde_json::Value) {
        if let Some(bus) = &self.event_bus {
            let event = UniversalEvent::new(
                format!("tenant.{}.{}", tenant_id, event_type),
                json!({
                    "tenant_id": tenant_id,
                    "event": event_type,
                    "timestamp": SystemTime::now(),
                    "data": data,
                }),
                Language::Rust,
            );

            if let Err(e) = bus.publish(event).await {
                warn!("Failed to emit tenant event: {}", e);
            }
        }
    }

    /// Set default limits for new tenants
    pub fn with_default_limits(mut self, limits: TenantLimits) -> Self {
        self.default_limits = limits;
        self
    }

    /// Create a new tenant
    pub async fn create_tenant(&self, config: TenantConfig) -> Result<()> {
        let tenant_id = config.tenant_id.clone();

        // Check if tenant already exists
        if self.tenants.contains_key(&tenant_id) {
            return Err(anyhow!("Tenant {} already exists", tenant_id));
        }

        // Create tenant scope
        let scope = StateScope::Custom(format!("tenant:{}", tenant_id));

        // Initialize usage metrics
        let usage = self.usage_tracker.initialize_tenant(&tenant_id).await?;

        // Create tenant info
        let tenant_info = TenantInfo {
            config: config.clone(),
            usage,
            scope,
        };

        // Store tenant
        self.tenants
            .insert(tenant_id.clone(), Arc::new(RwLock::new(tenant_info)));

        // Emit creation event
        self.emit_event(
            &tenant_id,
            "created",
            json!({
                "name": config.name,
                "limits": config.limits,
            }),
        )
        .await;

        info!("Created tenant: {}", tenant_id);
        Ok(())
    }

    /// Delete a tenant and all associated data
    ///
    /// # Errors
    /// Returns error if tenant doesn't exist or deletion fails
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<()> {
        // Remove from active tenants
        let tenant = self
            .tenants
            .remove(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let tenant_info = tenant.1.read().await;

        // Delete all vectors for this tenant
        let deleted_count = self.storage.delete_scope(&tenant_info.scope).await?;

        // Clean up usage tracking
        self.usage_tracker.remove_tenant(tenant_id).await?;

        // Emit deletion event
        self.emit_event(
            tenant_id,
            "deleted",
            json!({
                "vectors_deleted": deleted_count,
            }),
        )
        .await;

        info!(
            "Deleted tenant {} with {} vectors",
            tenant_id, deleted_count
        );
        Ok(())
    }

    /// Get tenant information
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<TenantInfo> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let info = tenant.read().await;
        Ok(info.clone())
    }

    /// List all tenants
    pub async fn list_tenants(&self) -> Vec<String> {
        self.tenants.iter().map(|e| e.key().clone()).collect()
    }

    /// Check if operation is allowed for tenant
    async fn check_limits(&self, tenant_id: &str, operation: &str) -> Result<()> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        // Use a block to limit the scope of the lock
        {
            let mut tenant_info = tenant.write().await;

            // Update last accessed time
            tenant_info.config.last_accessed = SystemTime::now();

            // Check if tenant is active
            if !tenant_info.config.active {
                return Err(anyhow!("Tenant {} is inactive", tenant_id));
            }

            // Check storage limits based on operation
            if operation == "insert" {
                if let Some(max_vectors) = tenant_info.config.limits.max_vectors {
                    if tenant_info.usage.vector_count >= max_vectors
                        && !tenant_info.config.limits.allow_overflow
                    {
                        return Err(anyhow!("Vector limit exceeded for tenant {}", tenant_id));
                    }
                }

                if let Some(max_storage) = tenant_info.config.limits.max_storage_bytes {
                    if tenant_info.usage.storage_bytes >= max_storage
                        && !tenant_info.config.limits.allow_overflow
                    {
                        return Err(anyhow!("Storage limit exceeded for tenant {}", tenant_id));
                    }
                }
            }
        } // Lock is released here

        // Check rate limits without holding the write lock
        let tenant_info = tenant.read().await;
        if let Some(max_qps) = tenant_info.config.limits.max_queries_per_second {
            let current_qps = self.usage_tracker.get_queries_per_second(tenant_id)?;

            if current_qps >= max_qps && !tenant_info.config.limits.allow_overflow {
                return Err(anyhow!("Rate limit exceeded for tenant {}", tenant_id));
            }
        }

        Ok(())
    }

    /// Insert vectors for a specific tenant
    pub async fn insert_for_tenant(
        &self,
        tenant_id: &str,
        mut vectors: Vec<VectorEntry>,
    ) -> Result<Vec<String>> {
        // Check limits
        self.check_limits(tenant_id, "insert").await?;

        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        // Get scope and check dimensions in a limited scope
        let scope = {
            let tenant_info = tenant.read().await;

            // Check dimension limits
            if let Some(max_dims) = tenant_info.config.limits.max_dimensions {
                for vector in &vectors {
                    if vector.embedding.len() > max_dims {
                        return Err(anyhow!(
                            "Vector dimension {} exceeds limit {} for tenant {}",
                            vector.embedding.len(),
                            max_dims,
                            tenant_id
                        ));
                    }
                }
            }

            tenant_info.scope.clone()
        }; // Read lock released here

        // Set tenant scope for all vectors
        for vector in &mut vectors {
            vector.scope = scope.clone();
            vector.tenant_id = Some(tenant_id.to_string());
        }

        // Insert vectors
        let ids = self.storage.insert(vectors.clone()).await?;

        // Update tenant info with write lock
        {
            let mut tenant_info = tenant.write().await;
            tenant_info.usage.vector_count += vectors.len();
            tenant_info.usage.storage_bytes +=
                vectors.iter().map(|v| v.embedding.len() * 4).sum::<usize>();
            tenant_info.usage.insert_count += 1;
        }

        // Update usage metrics
        self.usage_tracker
            .record_insert(
                tenant_id,
                vectors.len(),
                vectors.iter().map(|v| v.embedding.len() * 4).sum(), // f32 = 4 bytes
            )
            .await?;

        // Emit insert event
        self.emit_event(
            tenant_id,
            "vectors_inserted",
            json!({
                "count": vectors.len(),
                "vector_ids": ids,
            }),
        )
        .await;

        debug!("Inserted {} vectors for tenant {}", ids.len(), tenant_id);
        Ok(ids)
    }

    /// Search vectors for a specific tenant
    pub async fn search_for_tenant(
        &self,
        tenant_id: &str,
        mut query: VectorQuery,
    ) -> Result<Vec<VectorResult>> {
        // Check limits
        self.check_limits(tenant_id, "search").await?;

        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let tenant_info = tenant.read().await;

        // Set tenant scope for query
        query.scope = Some(tenant_info.scope.clone());

        // Execute search
        let results = self
            .storage
            .search_scoped(&query, &tenant_info.scope)
            .await
            .unwrap_or_else(|_| Vec::new());

        // Update usage metrics
        self.usage_tracker.record_search(tenant_id).await?;

        // Emit search event
        self.emit_event(
            tenant_id,
            "vectors_searched",
            json!({
                "results_count": results.len(),
                "query_type": "similarity",
            }),
        )
        .await;

        debug!(
            "Search returned {} results for tenant {}",
            results.len(),
            tenant_id
        );
        Ok(results)
    }

    /// Get usage statistics for a tenant
    pub async fn get_tenant_stats(&self, tenant_id: &str) -> Result<ScopedStats> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let tenant_info = tenant.read().await;

        // Get stats from storage
        let stats = self.storage.stats_for_scope(&tenant_info.scope).await?;

        Ok(stats)
    }

    /// Get global storage statistics
    pub async fn get_global_stats(&self) -> Result<StorageStats> {
        self.storage.stats().await
    }

    /// Update tenant configuration
    pub async fn update_tenant_config(&self, tenant_id: &str, config: TenantConfig) -> Result<()> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let mut tenant_info = tenant.write().await;
        tenant_info.config = config;

        info!("Updated configuration for tenant {}", tenant_id);
        Ok(())
    }

    /// Suspend a tenant (make inactive)
    pub async fn suspend_tenant(&self, tenant_id: &str) -> Result<()> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let mut tenant_info = tenant.write().await;
        tenant_info.config.active = false;

        warn!("Suspended tenant {}", tenant_id);
        Ok(())
    }

    /// Resume a tenant (make active)
    pub async fn resume_tenant(&self, tenant_id: &str) -> Result<()> {
        let tenant = self
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        let mut tenant_info = tenant.write().await;
        tenant_info.config.active = true;

        info!("Resumed tenant {}", tenant_id);
        Ok(())
    }
}

#[async_trait]
impl VectorStorage for MultiTenantVectorManager {
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>> {
        // Extract tenant ID from first vector or use default
        let tenant_id = vectors
            .first()
            .and_then(|v| v.tenant_id.clone())
            .ok_or_else(|| anyhow!("No tenant ID specified in vectors"))?;

        self.insert_for_tenant(&tenant_id, vectors).await
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>> {
        // This requires a tenant scope
        if let Some(StateScope::Custom(scope_str)) = &query.scope {
            if let Some(tenant_id) = scope_str.strip_prefix("tenant:") {
                return self.search_for_tenant(tenant_id, query.clone()).await;
            }
        }

        Err(anyhow!("Search requires a tenant scope"))
    }

    async fn search_scoped(
        &self,
        query: &VectorQuery,
        scope: &StateScope,
    ) -> Result<Vec<VectorResult>> {
        if let StateScope::Custom(scope_str) = scope {
            if let Some(tenant_id) = scope_str.strip_prefix("tenant:") {
                return self.search_for_tenant(tenant_id, query.clone()).await;
            }
        }

        Err(anyhow!("Invalid tenant scope"))
    }

    async fn update_metadata(
        &self,
        id: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Delegate to underlying storage
        self.storage.update_metadata(id, metadata).await
    }

    async fn delete(&self, ids: &[String]) -> Result<()> {
        // Delegate to underlying storage
        self.storage.delete(ids).await
    }

    async fn delete_scope(&self, scope: &StateScope) -> Result<usize> {
        if let StateScope::Custom(scope_str) = scope {
            if let Some(tenant_id) = scope_str.strip_prefix("tenant:") {
                // Update usage tracking
                let count = self.storage.delete_scope(scope).await?;
                self.usage_tracker.record_delete(tenant_id, count).await?;
                return Ok(count);
            }
        }

        self.storage.delete_scope(scope).await
    }

    async fn stats(&self) -> Result<StorageStats> {
        self.get_global_stats().await
    }

    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats> {
        if let StateScope::Custom(scope_str) = scope {
            if let Some(tenant_id) = scope_str.strip_prefix("tenant:") {
                return self.get_tenant_stats(tenant_id).await;
            }
        }

        self.storage.stats_for_scope(scope).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_storage::backends::vector::HNSWVectorStorage;

    #[tokio::test]
    async fn test_tenant_creation() {
        let storage = Arc::new(HNSWVectorStorage::new(3, Default::default()));
        let manager = MultiTenantVectorManager::new(storage);

        let config = TenantConfig {
            tenant_id: "test-tenant".to_string(),
            name: "Test Tenant".to_string(),
            limits: TenantLimits::default(),
            active: true,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            custom_config: None,
        };

        manager.create_tenant(config).await.unwrap();

        let tenants = manager.list_tenants().await;
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0], "test-tenant");
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let storage = Arc::new(HNSWVectorStorage::new(3, Default::default()));
        let manager = MultiTenantVectorManager::new(storage);

        // Create two tenants
        for tenant_id in &["tenant-1", "tenant-2"] {
            let config = TenantConfig {
                tenant_id: tenant_id.to_string(),
                name: format!("Tenant {}", tenant_id),
                limits: TenantLimits::default(),
                active: true,
                metadata: HashMap::new(),
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
                custom_config: None,
            };
            manager.create_tenant(config).await.unwrap();
        }

        // Insert vectors for tenant-1
        let vectors = vec![
            VectorEntry::new("vec1".to_string(), vec![0.1, 0.2, 0.3]),
            VectorEntry::new("vec2".to_string(), vec![0.4, 0.5, 0.6]),
        ];

        manager
            .insert_for_tenant("tenant-1", vectors)
            .await
            .unwrap();

        // Search from tenant-1 should find results
        let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 10);
        let results = manager
            .search_for_tenant("tenant-1", query.clone())
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        // Search from tenant-2 should find no results
        let results = manager.search_for_tenant("tenant-2", query).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_tenant_limits() {
        let storage = Arc::new(HNSWVectorStorage::new(3, Default::default()));
        let manager = MultiTenantVectorManager::new(storage);

        let config = TenantConfig {
            tenant_id: "limited-tenant".to_string(),
            name: "Limited Tenant".to_string(),
            limits: TenantLimits {
                max_vectors: Some(1),
                max_storage_bytes: Some(1024),
                max_queries_per_second: Some(10),
                max_dimensions: Some(3),
                allow_overflow: false,
                custom_limits: HashMap::new(),
            },
            active: true,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            custom_config: None,
        };

        manager.create_tenant(config).await.unwrap();

        // First insert should succeed
        let vector = vec![VectorEntry::new("vec1".to_string(), vec![0.1, 0.2, 0.3])];
        manager
            .insert_for_tenant("limited-tenant", vector)
            .await
            .unwrap();

        // Second insert should fail due to vector limit
        let vector = vec![VectorEntry::new("vec2".to_string(), vec![0.4, 0.5, 0.6])];
        let result = manager.insert_for_tenant("limited-tenant", vector).await;
        assert!(result.is_err());
    }
}
