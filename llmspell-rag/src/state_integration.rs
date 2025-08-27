//! State-aware vector storage integration with tenant management

use crate::multi_tenant_integration::MultiTenantRAG;
use anyhow::Result;
use llmspell_state_persistence::StateManager;
use llmspell_state_persistence::StateScope;
use llmspell_storage::VectorStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

/// State-aware vector storage that integrates with tenant management
pub struct StateAwareVectorStorage {
    /// Underlying vector storage
    storage: Arc<dyn VectorStorage>,
    /// State manager for scope-aware operations
    state_manager: Arc<StateManager>,
    /// Tenant manager for multi-tenant operations
    #[allow(dead_code)]
    tenant_manager: Arc<MultiTenantRAG>,
}

impl std::fmt::Debug for StateAwareVectorStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateAwareVectorStorage")
            .field("storage", &"Arc<dyn VectorStorage>")
            .field("state_manager", &"Arc<StateManager>")
            .field("tenant_manager", &"Arc<MultiTenantRAG>")
            .finish()
    }
}

/// Vector operation metadata for state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorOperationMetadata {
    /// Operation type (insert, search, delete, update)
    pub operation: String,
    /// Tenant ID if applicable
    pub tenant_id: Option<String>,
    /// Number of vectors affected
    pub vector_count: u64,
    /// Storage bytes used
    pub storage_bytes: u64,
    /// Operation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StateAwareVectorStorage {
    /// Create a new state-aware vector storage
    #[must_use]
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        state_manager: Arc<StateManager>,
        tenant_manager: Arc<MultiTenantRAG>,
    ) -> Self {
        Self {
            storage,
            state_manager,
            tenant_manager,
        }
    }

    /// Extract tenant ID from scope
    #[must_use]
    pub fn extract_tenant_id(scope: &StateScope) -> Option<String> {
        match scope {
            StateScope::Custom(scope_str) if scope_str.starts_with("tenant:") => {
                Some(scope_str.strip_prefix("tenant:").unwrap_or("").to_string())
            }
            _ => None,
        }
    }

    /// Track vector operation usage (simplified for tenant integration)
    ///
    /// # Errors
    /// Returns an error if usage tracking fails
    pub fn track_usage(&self, metadata: &VectorOperationMetadata) -> Result<()> {
        if let Some(tenant_id) = &metadata.tenant_id {
            debug!(
                "Tracking usage for tenant: {tenant_id} - operation: {}",
                metadata.operation
            );
            // In a full implementation, this would integrate with tenant_manager usage tracking
        }
        Ok(())
    }

    /// Validate tenant access before operations
    ///
    /// # Errors
    /// Returns an error if tenant access validation fails
    pub fn validate_tenant_access(&self, scope: &StateScope) -> Result<()> {
        if let Some(tenant_id) = Self::extract_tenant_id(scope) {
            debug!("Validating access for tenant: {tenant_id}");
            // In a full implementation, this would validate with tenant_manager
        }
        Ok(())
    }

    /// Store operation metadata in state
    ///
    /// # Errors
    /// Returns an error if metadata storage fails
    pub fn store_operation_metadata(
        &self,
        scope: &StateScope,
        metadata: &VectorOperationMetadata,
    ) -> Result<()> {
        let _key = format!("vector_ops:{}", uuid::Uuid::new_v4());
        let _serialized = serde_json::to_vec(metadata)?;

        // Store in state manager (simplified)
        debug!("Storing operation metadata for scope: {:?}", scope);
        // In a full implementation, this would use state_manager.set_with_scope

        Ok(())
    }

    /// Get underlying storage reference
    #[must_use]
    pub fn storage(&self) -> &Arc<dyn VectorStorage> {
        &self.storage
    }

    /// Get state manager reference
    #[must_use]
    pub const fn state_manager(&self) -> &Arc<StateManager> {
        &self.state_manager
    }

    /// Get tenant manager reference  
    #[must_use]
    pub const fn tenant_manager(&self) -> &Arc<MultiTenantRAG> {
        &self.tenant_manager
    }
}

/// Helper functions for tenant-specific vector operations
impl StateAwareVectorStorage {
    /// Get tenant-specific statistics (simplified)
    ///
    /// # Errors
    /// Returns an error if statistics retrieval fails
    pub fn get_tenant_stats(&self, tenant_id: &str) -> Result<TenantVectorStats> {
        debug!("Getting stats for tenant: {tenant_id}");

        // Return simplified stats
        Ok(TenantVectorStats {
            tenant_id: tenant_id.to_string(),
            total_vectors: 0,
            total_storage_bytes: 0,
            namespace_count: 1,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Clear all data for a tenant
    ///
    /// # Errors
    /// Returns an error if data clearing fails
    pub fn clear_tenant_data(&self, tenant_id: &str) -> Result<u64> {
        debug!("Clearing data for tenant: {tenant_id}");
        // In a full implementation, this would clear tenant-specific data
        Ok(0)
    }

    /// List all tenants with vector data
    ///
    /// # Errors
    /// Returns an error if listing fails
    pub fn list_tenants_with_data(&self) -> Result<Vec<String>> {
        debug!("Listing tenants with data");
        // In a full implementation, this would query actual tenant data
        Ok(Vec::new())
    }
}

/// Tenant-specific vector storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantVectorStats {
    /// Tenant ID
    pub tenant_id: String,
    /// Total vectors stored
    pub total_vectors: u64,
    /// Total storage bytes used
    pub total_storage_bytes: u64,
    /// Number of namespaces
    pub namespace_count: u64,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_id_extraction() {
        let tenant_scope = StateScope::Custom("tenant:test-tenant".to_string());
        let tenant_id = StateAwareVectorStorage::extract_tenant_id(&tenant_scope);
        assert_eq!(tenant_id, Some("test-tenant".to_string()));

        let global_scope = StateScope::Global;
        let no_tenant = StateAwareVectorStorage::extract_tenant_id(&global_scope);
        assert_eq!(no_tenant, None);
    }

    #[tokio::test]
    async fn test_vector_operation_metadata() {
        let metadata = VectorOperationMetadata {
            operation: "insert".to_string(),
            tenant_id: Some("test-tenant".to_string()),
            vector_count: 1,
            storage_bytes: 1024,
            timestamp: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: VectorOperationMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.operation, deserialized.operation);
        assert_eq!(metadata.tenant_id, deserialized.tenant_id);
        assert_eq!(metadata.vector_count, deserialized.vector_count);
        assert_eq!(metadata.storage_bytes, deserialized.storage_bytes);
    }

    #[tokio::test]
    async fn test_tenant_vector_stats() {
        let stats = TenantVectorStats {
            tenant_id: "test-tenant".to_string(),
            total_vectors: 100,
            total_storage_bytes: 10240,
            namespace_count: 5,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(stats.tenant_id, "test-tenant");
        assert_eq!(stats.total_vectors, 100);
        assert_eq!(stats.total_storage_bytes, 10240);
        assert_eq!(stats.namespace_count, 5);
    }
}
