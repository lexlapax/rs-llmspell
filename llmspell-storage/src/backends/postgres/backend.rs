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

// Note: StorageBackend trait implementation deferred to Phase 13b.4
// Note: VectorStorage trait implementation deferred to Phase 13b.4
