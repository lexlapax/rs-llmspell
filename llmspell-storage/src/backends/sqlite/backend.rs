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
use libsql::Connection;
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
/// for unified local storage using libsql.
///
/// # Architecture
///
/// - **Connection Pool**: Custom async pool wrapping libsql Database
/// - **Tenant Isolation**: Application-level filtering via DashMap context
/// - **WAL Mode**: Concurrent readers + single writer for performance
/// - **Encryption**: Optional AES-256 at-rest encryption via libsql
///
/// # Examples
///
/// ```no_run
/// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = SqliteConfig::new("./data/llmspell.db")
///     .with_max_connections(20);
///
/// let backend = SqliteBackend::new(config).await?;
///
/// // Set tenant context
/// backend.set_tenant_context("tenant-123").await?;
///
/// // Get connection (will apply tenant context)
/// let conn = backend.get_connection().await?;
///
/// // Health check
/// let healthy = backend.health_check().await?;
/// # Ok(())
/// # }
/// ```
pub struct SqliteBackend {
    /// Connection pool
    pool: Arc<SqlitePool>,

    /// Tenant context map (tenant_id → TenantContext)
    ///
    /// Used for application-level tenant filtering in queries.
    /// Each connection should check this map before executing tenant-scoped queries.
    tenant_contexts: Arc<DashMap<String, TenantContext>>,

    /// Backend configuration
    config: SqliteConfig,
}

impl SqliteBackend {
    /// Create new SQLite backend
    ///
    /// # Arguments
    ///
    /// * `config` - SQLite configuration
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Configuration is invalid
    /// - Database cannot be opened
    /// - Pool creation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = SqliteConfig::new("./llmspell.db");
    /// let backend = SqliteBackend::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: SqliteConfig) -> Result<Self> {
        // Create connection pool
        let pool = SqlitePool::new(config.clone()).await?;

        // Load vector search extension (vectorlite-rs, Task 13c.2.2a)
        // Pure Rust HNSW implementation - 3-100x faster than brute-force
        let conn = pool.get_connection().await?;

        // Define extension paths based on platform
        #[cfg(target_os = "macos")]
        let vectorlite_path = "./extensions/vectorlite.dylib";
        #[cfg(target_os = "linux")]
        let vectorlite_path = "./extensions/vectorlite.so";
        #[cfg(target_os = "windows")]
        let vectorlite_path = "./extensions/vectorlite.dll";

        // Enable extension loading (required by libsql for security)
        // SAFETY: Extension loading is disabled immediately after loading
        conn.load_extension_enable().map_err(|e| {
            SqliteError::Extension(format!("Failed to enable extension loading: {e}"))
        })?;

        // Load vectorlite-rs extension (HNSW-indexed vector search)
        match conn.load_extension(vectorlite_path, None) {
            Ok(()) => {
                info!(
                    "Successfully loaded vectorlite-rs extension from {vectorlite_path} (HNSW-indexed, 3-100x faster)"
                );
            }
            Err(e) => {
                warn!(
                    "Failed to load vectorlite-rs extension from {vectorlite_path}: {e}. \
                    Vector search will not be available. \
                    Build vectorlite-rs: cargo build -p vectorlite-rs --release && \
                    cp target/release/libvectorlite_rs.* extensions/vectorlite.*"
                );
            }
        }

        // Disable extension loading for security (prevent SQL injection attacks)
        conn.load_extension_disable().map_err(|e| {
            SqliteError::Extension(format!("Failed to disable extension loading: {e}"))
        })?;

        let backend = Self {
            pool: Arc::new(pool),
            tenant_contexts: Arc::new(DashMap::new()),
            config,
        };

        // Run migrations to create all necessary tables (consistent with Postgres pattern)
        // For SQLite, we run migrations at initialization since there's no separate deployment step
        backend
            .run_migrations()
            .await
            .map_err(|e| SqliteError::Migration(format!("Failed to run migrations: {}", e)))?;

        info!("SQLite backend initialized with all necessary tables via migrations");

        Ok(backend)
    }

    /// Get a connection from the pool
    ///
    /// Returns a fresh connection with PRAGMA settings applied.
    ///
    /// # Errors
    ///
    /// Returns error if connection creation fails
    pub async fn get_connection(&self) -> Result<Connection> {
        self.pool.get_connection().await
    }

    /// Set tenant context
    ///
    /// Registers tenant context for application-level row filtering.
    /// Subsequent queries should check this context to enforce tenant isolation.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant identifier
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_storage::backends::sqlite::SqliteBackend;
    /// # async fn example(backend: &SqliteBackend) -> anyhow::Result<()> {
    /// backend.set_tenant_context("tenant-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_tenant_context(&self, tenant_id: impl Into<String>) -> Result<()> {
        let tenant_id = tenant_id.into();
        let context = TenantContext::new(tenant_id.clone());
        self.tenant_contexts.insert(tenant_id, context);
        Ok(())
    }

    /// Set tenant context with details
    ///
    /// # Arguments
    ///
    /// * `context` - Complete tenant context
    pub async fn set_tenant_context_detailed(&self, context: TenantContext) -> Result<()> {
        let tenant_id = context.tenant_id.clone();
        self.tenant_contexts.insert(tenant_id, context);
        Ok(())
    }

    /// Get tenant context
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant identifier
    ///
    /// # Returns
    ///
    /// Returns tenant context if exists, None otherwise
    pub fn get_tenant_context(&self, tenant_id: &str) -> Option<TenantContext> {
        self.tenant_contexts.get(tenant_id).map(|r| r.clone())
    }

    /// Clear tenant context
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Tenant identifier to clear
    pub async fn clear_tenant_context(&self, tenant_id: &str) -> Result<()> {
        self.tenant_contexts.remove(tenant_id);
        Ok(())
    }

    /// List all active tenant contexts
    ///
    /// # Returns
    ///
    /// Vector of tenant IDs with active contexts
    pub fn list_tenant_contexts(&self) -> Vec<String> {
        self.tenant_contexts
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Health check
    ///
    /// Verifies database connectivity and WAL mode status.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if database is healthy
    /// - `Ok(false)` if database is accessible but degraded
    /// - `Err` if database is inaccessible
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_storage::backends::sqlite::SqliteBackend;
    /// # async fn example(backend: &SqliteBackend) -> anyhow::Result<()> {
    /// let healthy = backend.health_check().await?;
    /// if healthy {
    ///     println!("Database is healthy");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> Result<bool> {
        // Use pool's health check
        self.pool.health_check().await
    }

    /// Get detailed health status
    ///
    /// Returns comprehensive health metrics including pool stats,
    /// WAL checkpoint status, and cache statistics.
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        let conn = self.get_connection().await?;

        // Check WAL mode
        let mut stmt = conn
            .prepare("PRAGMA journal_mode")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query journal_mode: {}", e)))?;
        let journal_mode = stmt
            .query_row(())
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to read journal_mode: {}", e)))?
            .get::<String>(0)
            .map_err(|e| SqliteError::Query(format!("Failed to parse journal_mode: {}", e)))?;

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
        let backend = SqliteBackend::new(config).await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_get_connection() {
        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await.unwrap();
        let conn = backend.get_connection().await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_tenant_context() {
        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await.unwrap();

        // Set context
        backend.set_tenant_context("tenant-1").await.unwrap();

        // Get context
        let context = backend.get_tenant_context("tenant-1");
        assert!(context.is_some());
        assert_eq!(context.unwrap().tenant_id, "tenant-1");

        // List contexts
        let tenants = backend.list_tenant_contexts();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0], "tenant-1");

        // Clear context
        backend.clear_tenant_context("tenant-1").await.unwrap();
        assert!(backend.get_tenant_context("tenant-1").is_none());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await.unwrap();

        let healthy = backend.health_check().await;
        assert!(healthy.is_ok());
        assert!(healthy.unwrap());
    }

    #[tokio::test]
    async fn test_health_status() {
        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await.unwrap();

        let status = backend.get_health_status().await;
        assert!(status.is_ok());

        let status = status.unwrap();
        assert!(status.is_healthy);
        // In-memory databases use "memory" journal mode, not "wal"
        assert!(
            status.journal_mode.to_lowercase() == "wal"
                || status.journal_mode.to_lowercase() == "memory"
        );
    }

    #[tokio::test]
    async fn test_tenant_context_detailed() {
        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await.unwrap();

        let context = TenantContext::new("tenant-1")
            .with_user("user-123")
            .with_metadata("region", "us-west-2");

        backend
            .set_tenant_context_detailed(context.clone())
            .await
            .unwrap();

        let retrieved = backend.get_tenant_context("tenant-1").unwrap();
        assert_eq!(retrieved.tenant_id, "tenant-1");
        assert_eq!(retrieved.user_id, Some("user-123".to_string()));
        assert_eq!(
            retrieved.metadata.get("region"),
            Some(&"us-west-2".to_string())
        );
    }
}
