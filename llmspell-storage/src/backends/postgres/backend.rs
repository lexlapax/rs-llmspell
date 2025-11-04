//! ABOUTME: PostgreSQL storage backend (Phase 13b.2 stub)
//! ABOUTME: Main backend struct with connection pool and tenant context management

use super::config::PostgresConfig;
use super::error::{PostgresError, Result};
use super::pool::PostgresPool;
use llmspell_core::TenantScoped;
use std::sync::Arc;
use tokio::sync::RwLock;

/// PostgreSQL storage backend
///
/// Phase 13b.2: Infrastructure only (connection pool, tenant context, health checks)
/// Phase 13b.4+: Storage operations (VectorStorage, StorageBackend trait implementations)
#[derive(Debug, Clone)]
pub struct PostgresBackend {
    /// Connection pool
    pool: PostgresPool,

    /// Current tenant context (for RLS)
    tenant_context: Arc<RwLock<Option<String>>>,

    /// Configuration
    config: PostgresConfig,
}

impl PostgresBackend {
    /// Create a new PostgreSQL backend
    ///
    /// # Arguments
    /// * `config` - PostgreSQL configuration
    ///
    /// # Returns
    /// * `Result<Self>` - Initialized backend or error
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    ///     let backend = PostgresBackend::new(config).await.unwrap();
    /// }
    /// ```
    pub async fn new(config: PostgresConfig) -> Result<Self> {
        let pool = PostgresPool::new(&config).await?;

        Ok(Self {
            pool,
            tenant_context: Arc::new(RwLock::new(None)),
            config,
        })
    }

    /// Set the tenant context for Row-Level Security (RLS)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
    /// # #[tokio::main]
    /// # async fn main() {
    /// #     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    /// #     let backend = PostgresBackend::new(config).await.unwrap();
    /// backend.set_tenant_context("tenant_123").await.unwrap();
    /// # }
    /// ```
    pub async fn set_tenant_context(&self, tenant_id: impl Into<String>) -> Result<()> {
        let tenant_id = tenant_id.into();

        // Set PostgreSQL session variable for RLS using set_config()
        if self.config.enable_rls {
            let client = self.pool.get().await?;
            client
                .execute(
                    "SELECT set_config('app.current_tenant_id', $1, false)",
                    &[&tenant_id],
                )
                .await
                .map_err(|e| {
                    PostgresError::Query(format!("Failed to set tenant context: {}", e))
                })?;
        }

        // Update internal context
        let mut ctx = self.tenant_context.write().await;
        *ctx = Some(tenant_id);

        Ok(())
    }

    /// Get the current tenant context
    pub async fn get_tenant_context(&self) -> Option<String> {
        self.tenant_context.read().await.clone()
    }

    /// Clear the tenant context
    pub async fn clear_tenant_context(&self) -> Result<()> {
        let mut ctx = self.tenant_context.write().await;
        *ctx = None;
        Ok(())
    }

    /// Apply Row-Level Security (RLS) policies to a table
    ///
    /// # Arguments
    /// * `table_name` - Name of the table (without schema prefix)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
    /// # #[tokio::main]
    /// # async fn main() {
    /// #     let config = PostgresConfig::new("postgresql://localhost/llmspell_dev");
    /// #     let backend = PostgresBackend::new(config).await.unwrap();
    /// // After creating a table in a migration
    /// backend.apply_rls_to_table("vector_embeddings").await.unwrap();
    /// # }
    /// ```
    ///
    /// # Phase 13b.3.1
    /// This method uses the `generate_rls_policies()` helper to create four policies
    /// (SELECT, INSERT, UPDATE, DELETE) that enforce tenant isolation via RLS.
    pub async fn apply_rls_to_table(&self, table_name: &str) -> Result<()> {
        let sql = super::rls::generate_rls_policies(table_name);
        let client = self.pool.get().await?;

        client
            .batch_execute(&sql)
            .await
            .map_err(|e| PostgresError::Migration(format!("RLS policy failed: {}", e)))?;

        Ok(())
    }

    /// Check if the backend is healthy (can connect to database)
    ///
    /// # Returns
    /// * `bool` - True if healthy, false otherwise
    pub async fn is_healthy(&self) -> bool {
        self.pool.is_healthy().await
    }

    /// Get pool status
    pub fn pool_status(&self) -> super::pool::PoolStatus {
        self.pool.status()
    }

    /// Get a pooled client connection
    ///
    /// # Returns
    /// * `Result<deadpool_postgres::Client>` - Pooled client connection
    ///
    /// # Phase 13b.3.2
    /// Exposed as public to allow RLS testing from integration tests
    pub async fn get_client(&self) -> Result<deadpool_postgres::Client> {
        let client = self.pool.get().await?;

        // Apply current tenant context to this client connection
        if self.config.enable_rls {
            if let Some(tenant_id) = self.get_tenant_context().await {
                // Set tenant context
                client
                    .execute(
                        "SELECT set_config('app.current_tenant_id', $1, false)",
                        &[&tenant_id],
                    )
                    .await
                    .map_err(|e| {
                        PostgresError::Query(format!(
                            "Failed to set tenant context on client: {}",
                            e
                        ))
                    })?;
            } else {
                // Clear tenant context (reset to empty string blocks all RLS access)
                client
                    .execute("SELECT set_config('app.current_tenant_id', '', false)", &[])
                    .await
                    .map_err(|e| {
                        PostgresError::Query(format!(
                            "Failed to clear tenant context on client: {}",
                            e
                        ))
                    })?;
            }
        }

        Ok(client)
    }
}

// =============================================================================
// TenantScoped Trait Implementation (Phase 13b.3.4)
// =============================================================================

#[async_trait::async_trait]
impl TenantScoped for PostgresBackend {
    /// Get the current tenant ID from the backend's internal context
    ///
    /// Returns owned String to support async tenant context retrieval
    async fn tenant_id(&self) -> Option<String> {
        self.get_tenant_context().await
    }

    /// Get the state scope for this backend
    ///
    /// PostgreSQL backend operates at global scope - tenant context is managed
    /// via database session variables rather than application-level state scopes
    fn scope(&self) -> &llmspell_core::state::StateScope {
        use std::sync::OnceLock;
        static SCOPE: OnceLock<llmspell_core::state::StateScope> = OnceLock::new();
        SCOPE.get_or_init(|| llmspell_core::state::StateScope::Global)
    }

    /// Set the tenant context for this backend
    ///
    /// # Arguments
    /// * `tenant_id` - The tenant identifier to set
    /// * `_scope` - State scope (ignored - PostgreSQL uses session scope only)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Implementation Notes
    /// - Calls internal `set_tenant_context()` which updates both:
    ///   1. Internal Rust state (Arc<RwLock<Option<String>>>)
    ///   2. PostgreSQL session variable (`app.current_tenant_id`)
    /// - The scope parameter is ignored because PostgreSQL RLS operates at session scope
    /// - All subsequent `get_client()` calls will apply this tenant context
    async fn set_tenant_context(
        &self,
        tenant_id: String,
        _scope: llmspell_core::state::StateScope,
    ) -> anyhow::Result<()> {
        // Call the existing set_tenant_context method
        self.set_tenant_context(tenant_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set tenant context: {}", e))
    }
}

// =============================================================================
// StorageBackend Trait Implementation (Phase 13b.7.2)
// =============================================================================

#[async_trait::async_trait]
impl crate::traits::StorageBackend for PostgresBackend {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        // Route based on key pattern
        if key.starts_with("agent:") {
            self.get_agent_state(key).await
        } else {
            self.get_kv(key).await
        }
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        // Route based on key pattern
        if key.starts_with("agent:") {
            self.set_agent_state(key, value).await
        } else {
            self.set_kv(key, value).await
        }
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        // Route based on key pattern
        if key.starts_with("agent:") {
            self.delete_agent_state(key).await
        } else {
            self.delete_kv(key).await
        }
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        // Route based on key pattern
        if key.starts_with("agent:") {
            self.exists_agent_state(key).await
        } else {
            self.exists_kv(key).await
        }
    }

    async fn list_keys(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        // Route based on prefix pattern
        if prefix.starts_with("agent:") {
            self.list_agent_state_keys(prefix).await
        } else {
            self.list_kv_keys(prefix).await
        }
    }

    async fn get_batch(
        &self,
        keys: &[String],
    ) -> anyhow::Result<std::collections::HashMap<String, Vec<u8>>> {
        use std::collections::HashMap;

        // Partition keys by routing destination
        let (agent_keys, kv_keys): (Vec<_>, Vec<_>) =
            keys.iter().partition(|k| k.starts_with("agent:"));

        // Fetch from both sources
        let mut results = HashMap::new();

        for key in agent_keys {
            if let Some(value) = self.get_agent_state(key).await? {
                results.insert(key.to_string(), value);
            }
        }

        for key in kv_keys {
            if let Some(value) = self.get_kv(key).await? {
                results.insert(key.to_string(), value);
            }
        }

        Ok(results)
    }

    async fn set_batch(&self, items: std::collections::HashMap<String, Vec<u8>>) -> anyhow::Result<()> {
        use std::collections::HashMap;

        // Partition items by routing destination
        let mut agent_items = HashMap::new();
        let mut kv_items = HashMap::new();

        for (key, value) in items {
            if key.starts_with("agent:") {
                agent_items.insert(key, value);
            } else {
                kv_items.insert(key, value);
            }
        }

        // Store in both destinations
        for (key, value) in agent_items {
            self.set_agent_state(&key, value).await?;
        }

        for (key, value) in kv_items {
            self.set_kv(&key, value).await?;
        }

        Ok(())
    }

    async fn delete_batch(&self, keys: &[String]) -> anyhow::Result<()> {
        for key in keys {
            self.delete(key).await?;
        }
        Ok(())
    }

    async fn clear(&self) -> anyhow::Result<()> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;

        // Clear both tables (RLS will scope to current tenant)
        client
            .execute("DELETE FROM llmspell.agent_states", &[])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to clear agent_states: {}", e))?;

        client
            .execute("DELETE FROM llmspell.kv_store", &[])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to clear kv_store: {}", e))?;

        Ok(())
    }

    fn backend_type(&self) -> crate::traits::StorageBackendType {
        crate::traits::StorageBackendType::Postgres
    }

    fn characteristics(&self) -> crate::traits::StorageCharacteristics {
        crate::traits::StorageCharacteristics {
            persistent: true,
            transactional: true,
            supports_prefix_scan: true,
            supports_atomic_ops: true,
            avg_read_latency_us: 2000,  // ~2ms for network + query
            avg_write_latency_us: 3000, // ~3ms for network + query + fsync
        }
    }
}

// =============================================================================
// Agent State Operations (Specialized Path)
// =============================================================================

impl PostgresBackend {
    /// Get agent state from agent_states table
    async fn get_agent_state(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let (agent_id, _agent_type) = self.parse_agent_key(key)?;

        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        let row = client
            .query_opt(
                "SELECT state_data FROM llmspell.agent_states
                 WHERE tenant_id = $1 AND agent_id = $2",
                &[&tenant_id, &agent_id],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get agent state: {}", e))?;

        if let Some(row) = row {
            let state_data: serde_json::Value = row.get(0);
            let bytes = serde_json::to_vec(&state_data)?;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }

    /// Set agent state in agent_states table
    async fn set_agent_state(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        let (agent_id, agent_type) = self.parse_agent_key(key)?;

        // Parse value as JSON
        let state_data: serde_json::Value = serde_json::from_slice(&value)?;

        // Compute checksum
        let checksum = self.compute_checksum(&value);

        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        // Upsert with version tracking
        client
            .execute(
                "INSERT INTO llmspell.agent_states
                    (tenant_id, agent_id, agent_type, state_data, checksum)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (tenant_id, agent_id)
                 DO UPDATE SET
                    state_data = EXCLUDED.state_data,
                    checksum = EXCLUDED.checksum,
                    updated_at = now()",
                &[&tenant_id, &agent_id, &agent_type, &state_data, &checksum],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set agent state: {}", e))?;

        Ok(())
    }

    /// Delete agent state from agent_states table
    async fn delete_agent_state(&self, key: &str) -> anyhow::Result<()> {
        let (agent_id, _) = self.parse_agent_key(key)?;

        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        client
            .execute(
                "DELETE FROM llmspell.agent_states
                 WHERE tenant_id = $1 AND agent_id = $2",
                &[&tenant_id, &agent_id],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete agent state: {}", e))?;

        Ok(())
    }

    /// Check if agent state exists
    async fn exists_agent_state(&self, key: &str) -> anyhow::Result<bool> {
        let (agent_id, _) = self.parse_agent_key(key)?;

        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        let row = client
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM llmspell.agent_states
                    WHERE tenant_id = $1 AND agent_id = $2
                )",
                &[&tenant_id, &agent_id],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check agent state existence: {}", e))?;

        Ok(row.get(0))
    }

    /// List agent state keys with prefix
    async fn list_agent_state_keys(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        // Remove "agent:" prefix for pattern matching
        let agent_prefix = prefix.strip_prefix("agent:").unwrap_or(prefix);

        let rows = client
            .query(
                "SELECT agent_id, agent_type FROM llmspell.agent_states
                 WHERE tenant_id = $1 AND agent_id LIKE $2
                 ORDER BY agent_id",
                &[&tenant_id, &format!("{}%", agent_prefix)],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list agent state keys: {}", e))?;

        let keys = rows
            .iter()
            .map(|row| {
                let agent_id: String = row.get(0);
                let agent_type: String = row.get(1);
                format!("agent:{}:{}", agent_type, agent_id)
            })
            .collect();

        Ok(keys)
    }

    /// Parse agent key format: "agent:<agent_type>:<agent_id>"
    fn parse_agent_key(&self, key: &str) -> anyhow::Result<(String, String)> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() != 3 || parts[0] != "agent" {
            return Err(anyhow::anyhow!(
                "Invalid agent key format. Expected 'agent:<agent_type>:<agent_id>', got '{}'",
                key
            ));
        }
        Ok((parts[2].to_string(), parts[1].to_string()))
    }

    /// Compute SHA-256 checksum of value
    fn compute_checksum(&self, value: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(value);
        format!("{:x}", hasher.finalize())
    }
}

// =============================================================================
// Generic KV Operations (Fallback Path)
// =============================================================================

impl PostgresBackend {
    /// Get value from kv_store table
    async fn get_kv(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        let row = client
            .query_opt(
                "SELECT value FROM llmspell.kv_store
                 WHERE tenant_id = $1 AND key = $2",
                &[&tenant_id, &key],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get kv: {}", e))?;

        if let Some(row) = row {
            let value: Vec<u8> = row.get(0);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Set value in kv_store table
    async fn set_kv(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        // Upsert
        client
            .execute(
                "INSERT INTO llmspell.kv_store (tenant_id, key, value)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (tenant_id, key)
                 DO UPDATE SET
                    value = EXCLUDED.value,
                    updated_at = now()",
                &[&tenant_id, &key, &value],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set kv: {}", e))?;

        Ok(())
    }

    /// Delete value from kv_store table
    async fn delete_kv(&self, key: &str) -> anyhow::Result<()> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        client
            .execute(
                "DELETE FROM llmspell.kv_store
                 WHERE tenant_id = $1 AND key = $2",
                &[&tenant_id, &key],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete kv: {}", e))?;

        Ok(())
    }

    /// Check if key exists in kv_store
    async fn exists_kv(&self, key: &str) -> anyhow::Result<bool> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        let row = client
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM llmspell.kv_store
                    WHERE tenant_id = $1 AND key = $2
                )",
                &[&tenant_id, &key],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check kv existence: {}", e))?;

        Ok(row.get(0))
    }

    /// List kv keys with prefix
    async fn list_kv_keys(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        let client = self.get_client().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let tenant_id = self.get_tenant_context().await
            .ok_or_else(|| anyhow::anyhow!("Tenant context not set"))?;

        let rows = client
            .query(
                "SELECT key FROM llmspell.kv_store
                 WHERE tenant_id = $1 AND key LIKE $2
                 ORDER BY key",
                &[&tenant_id, &format!("{}%", prefix)],
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list kv keys: {}", e))?;

        let keys = rows.iter().map(|row| row.get(0)).collect();

        Ok(keys)
    }
}

// Note: VectorStorage trait implementation deferred to Phase 13b.4
