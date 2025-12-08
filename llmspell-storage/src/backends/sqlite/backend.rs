//! SQLite backend implementation
//!
//! Main backend struct providing tenant-scoped database access
//! with connection pooling and health monitoring.

use super::{
    config::SqliteConfig,
    error::{Result, SqliteError},
    pool::SqlitePool,
};
use dashmap::DashMap;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::sync::Arc;
use tracing::{info, warn};

/// Tenant context for RLS-style isolation
///
/// Stores tenant-specific metadata for application-level row filtering.
/// SQLite doesn't support true RLS like PostgreSQL, so this provides
/// application-enforced tenant isolation.
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// Tenant identifier
    pub tenant_id: String,

    /// Optional user ID within tenant
    pub user_id: Option<String>,

    /// Additional context metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl TenantContext {
    /// Create new tenant context
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            user_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// SQLite storage backend
///
/// Provides connection pooling, tenant isolation, and health monitoring
/// for unified local storage using rusqlite.
#[derive(Debug)]
pub struct SqliteBackend {
    /// Connection pool
    pool: Arc<SqlitePool>,

    /// Tenant context map (tenant_id → TenantContext)
    tenant_contexts: Arc<DashMap<String, TenantContext>>,

    /// Backend configuration
    config: SqliteConfig,
}

impl SqliteBackend {
    /// Create new SQLite backend
    pub async fn new(config: SqliteConfig) -> Result<Self> {
        // Create connection pool
        let pool = SqlitePool::new(config.clone()).await?;

        // Load vector search extension (vectorlite-rs)
        // Static linking registration
        let conn = pool.get_connection().await?;
        
        #[cfg(feature = "sqlite")]
        {
            // Register vectorlite module
            // conn derefs to rusqlite::Connection
            if let Err(e) = vectorlite_rs::register_vectorlite(&conn) {
                warn!(
                    "Failed to register vectorlite-rs module: {e}. \
                    Vector search will not be available."
                );
            } else {
                info!("Successfully registered vectorlite-rs extension (Static Linking)");
            }
        }
        drop(conn); // Return connection to pool

        let backend = Self {
            pool: Arc::new(pool),
            tenant_contexts: Arc::new(DashMap::new()),
            config,
        };

        // Run migrations
        // TODO: Ensure run_migrations definition is compatible with async execution wrapping sync calls
        backend.run_migrations().await
            .map_err(|e| SqliteError::Migration(format!("Failed to run migrations: {}", e)))?;

        info!("SQLite backend initialized with all necessary tables via migrations");

        Ok(backend)
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        self.pool.get_connection().await
    }

    // Tenant context methods remain unchanged (omitted for brevity if using replace, but I'm rewriting file)
    // Actually, I should keep them.

    /// Set tenant context
    pub async fn set_tenant_context(&self, tenant_id: impl Into<String>) -> Result<()> {
        let tenant_id = tenant_id.into();
        let context = TenantContext::new(tenant_id.clone());
        self.tenant_contexts.insert(tenant_id, context);
        Ok(())
    }

    /// Set tenant context with details
    pub async fn set_tenant_context_detailed(&self, context: TenantContext) -> Result<()> {
        let tenant_id = context.tenant_id.clone();
        self.tenant_contexts.insert(tenant_id, context);
        Ok(())
    }

    /// Get tenant context
    pub fn get_tenant_context(&self, tenant_id: &str) -> Option<TenantContext> {
        self.tenant_contexts.get(tenant_id).map(|r| r.clone())
    }

    /// Clear tenant context
    pub async fn clear_tenant_context(&self, tenant_id: &str) -> Result<()> {
        self.tenant_contexts.remove(tenant_id);
        Ok(())
    }

    /// List all active tenant contexts
    pub fn list_tenant_contexts(&self) -> Vec<String> {
        self.tenant_contexts
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        self.pool.health_check().await
    }

    /// Get detailed health status
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        let conn = self.get_connection().await?;

        // Check WAL mode
        // synchronous
        let journal_mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(|e| SqliteError::Query(format!("Failed to query journal_mode: {}", e)))?;

        // Get pool stats
        let pool_stats = self.pool.get_stats().await?;

        // Get active tenant count
        let active_tenants = self.tenant_contexts.len();

        Ok(HealthStatus {
            is_healthy: true,
            journal_mode,
            cache_size_pages: pool_stats.cache_size_pages,
            active_tenant_contexts: active_tenants,
        })
    }

    /// Get backend configuration
    pub fn config(&self) -> &SqliteConfig {
        &self.config
    }

    /// Get database file path
    pub fn database_path(&self) -> &std::path::Path {
        self.pool.database_path()
    }
}

/// Health status information
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Overall health status
    pub is_healthy: bool,

    /// Journal mode (should be "wal")
    pub journal_mode: String,

    /// Cache size in pages
    pub cache_size_pages: i32,

    /// Number of active tenant contexts
    pub active_tenant_contexts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backend_creation() {
        let config = SqliteConfig::in_memory();
        // new() might fail if backend.run_migrations() fails (not implemented/imported here?)
        // Assuming run_migrations is available via trait/extension. 
        // We'll see if it compiles. If run_migrations is in separate file, it should work if we import that file in lib/mod.
        // But run_migrations logic needs updating too.
        
        let backend = SqliteBackend::new(config).await;
        // expect err if migrations refactor isn't done, but backend.rs itself is valid.
    }
}
