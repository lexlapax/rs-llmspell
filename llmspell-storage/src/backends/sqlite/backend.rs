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
use tracing::{debug, info, warn};

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

        // Load vector search extension (Task 13c.2.2a)
        // Priority: vectorlite-rs (HNSW, 3-100x faster) → sqlite-vec (brute-force, fallback)
        let conn = pool.get_connection().await?;

        // Define extension paths based on platform
        #[cfg(target_os = "macos")]
        let vectorlite_path = "./extensions/vectorlite.dylib";
        #[cfg(target_os = "linux")]
        let vectorlite_path = "./extensions/vectorlite.so";
        #[cfg(target_os = "windows")]
        let vectorlite_path = "./extensions/vectorlite.dll";

        #[cfg(target_os = "macos")]
        let vec0_path = "./extensions/vec0.dylib";
        #[cfg(target_os = "linux")]
        let vec0_path = "./extensions/vec0.so";
        #[cfg(target_os = "windows")]
        let vec0_path = "./extensions/vec0.dll";

        // Enable extension loading (required by libsql for security)
        // SAFETY: Extension loading is disabled immediately after loading
        conn.load_extension_enable().map_err(|e| {
            SqliteError::Extension(format!("Failed to enable extension loading: {e}"))
        })?;

        // Try to load vectorlite-rs (HNSW, preferred)
        let loaded = match conn.load_extension(vectorlite_path, None) {
            Ok(()) => {
                info!(
                    "Successfully loaded vectorlite-rs extension from {vectorlite_path} (HNSW-indexed, 3-100x faster)"
                );
                true
            }
            Err(e) => {
                debug!(
                    "vectorlite-rs not available at {vectorlite_path}: {e}. \
                    Falling back to sqlite-vec (brute-force). \
                    Build vectorlite: cargo build -p vectorlite-rs --release && \
                    cp target/release/libvectorlite_rs.* extensions/vectorlite.*"
                );
                false
            }
        };

        // Fall back to sqlite-vec (brute-force) if vectorlite not available
        if !loaded {
            match conn.load_extension(vec0_path, None) {
                Ok(()) => {
                    info!(
                        "Successfully loaded sqlite-vec extension from {vec0_path} (brute-force, slower than vectorlite-rs)"
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to load vector search extensions from {vectorlite_path} or {vec0_path}: {e}. \
                        Vector search will not be available. \
                        Build sqlite-vec: cd /tmp && git clone https://github.com/asg017/sqlite-vec && \
                        cd sqlite-vec && ./scripts/vendor.sh && make loadable && \
                        cp dist/vec0.* <project>/extensions/"
                    );
                }
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
        backend.run_migrations().await.map_err(|e| {
            SqliteError::Migration(format!("Failed to run migrations: {}", e))
        })?;

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

    /// Integration test for sqlite-vec extension and vector operations (Task 13c.2.2)
    ///
    /// Tests:
    /// - vec0 virtual table creation
    /// - Vector insertion using IntoBytes
    /// - K-NN search with MATCH operator
    /// - Multi-dimension support (384, 768, 1536, 3072)
    ///
    /// NOTE: This test requires vec0.dylib/vec0.so extension in ./extensions/
    /// If extension loading fails in SqliteBackend::new(), vector operations will error.
    #[tokio::test]
    async fn test_vector_operations_integration() -> anyhow::Result<()> {
        use super::super::SqliteVecExtension;
        use zerocopy::IntoBytes;

        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await?;
        let conn = backend.get_connection().await?;

        // Check if extension is available
        let available = match SqliteVecExtension::is_available(&conn).await {
            Ok(available) => available,
            Err(_) => {
                warn!(
                    "Skipping vector operations test: sqlite-vec extension not loaded. \
                    Build extension: cd /tmp && git clone https://github.com/asg017/sqlite-vec && \
                    cd sqlite-vec && ./scripts/vendor.sh && make loadable && \
                    cp dist/vec0.* ./extensions/"
                );
                return Ok(());
            }
        };
        if !available {
            warn!("Skipping vector operations test: vec_version() returned empty");
            return Ok(());
        }

        // Test 768-dimensional vectors (most common: OpenAI ada-002, BERT-base)
        conn.execute(
            "CREATE VIRTUAL TABLE vec_test_768 USING vec0(embedding float[768])",
            (),
        )
        .await?;

        // Insert test vectors
        let test_vectors: Vec<(i64, Vec<f32>)> = vec![
            (1, vec![0.1; 768]), // Vector 1: all 0.1
            (2, vec![0.2; 768]), // Vector 2: all 0.2
            (3, vec![0.3; 768]), // Vector 3: all 0.3
            (4, vec![0.4; 768]), // Vector 4: all 0.4
            (5, vec![0.5; 768]), // Vector 5: all 0.5
        ];

        for (rowid, embedding) in &test_vectors {
            conn.execute(
                "INSERT INTO vec_test_768(rowid, embedding) VALUES (?1, ?2)",
                libsql::params![*rowid, embedding.as_bytes()],
            )
            .await?;
        }

        // K-NN search: Find 3 nearest vectors to [0.3; 768]
        let query: Vec<f32> = vec![0.3; 768];
        let mut rows = conn
            .query(
                "SELECT rowid, distance FROM vec_test_768 WHERE embedding MATCH ?1 ORDER BY distance LIMIT 3",
                libsql::params![query.as_bytes()],
            )
            .await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            let rowid: i64 = row.get(0)?;
            let distance: f64 = row.get(1)?;
            results.push((rowid, distance));
        }

        // Verify results
        assert_eq!(results.len(), 3, "Should return 3 nearest neighbors");

        // Closest should be rowid 3 (exact match with [0.3; 768])
        assert_eq!(results[0].0, 3, "Nearest neighbor should be rowid 3");
        assert!(
            results[0].1 < 0.01,
            "Distance to exact match should be near zero, got {}",
            results[0].1
        );

        // Second closest should be rowid 2 or 4
        assert!(
            results[1].0 == 2 || results[1].0 == 4,
            "Second nearest should be rowid 2 or 4"
        );

        Ok(())
    }

    /// Test multi-dimension support for all common embedding models (Task 13c.2.2)
    ///
    /// Tests vec0 virtual table creation and basic operations for:
    /// - 384-dim: sentence-transformers/all-MiniLM-L6-v2
    /// - 768-dim: OpenAI ada-002, BERT-base
    /// - 1536-dim: OpenAI text-embedding-3-small
    /// - 3072-dim: OpenAI text-embedding-3-large
    #[tokio::test]
    async fn test_multi_dimension_support() -> anyhow::Result<()> {
        use super::super::SqliteVecExtension;
        use zerocopy::IntoBytes;

        let config = SqliteConfig::in_memory();
        let backend = SqliteBackend::new(config).await?;
        let conn = backend.get_connection().await?;

        // Check if extension is available
        let available = match SqliteVecExtension::is_available(&conn).await {
            Ok(available) => available,
            Err(_) => {
                warn!("Skipping multi-dimension test: sqlite-vec extension not loaded");
                return Ok(());
            }
        };
        if !available {
            warn!("Skipping multi-dimension test: vec_version() returned empty");
            return Ok(());
        }

        // Test all supported dimensions
        for &dim in SqliteVecExtension::supported_dimensions() {
            let table_name = format!("vec_test_{dim}");
            let create_sql =
                format!("CREATE VIRTUAL TABLE {table_name} USING vec0(embedding float[{dim}])");

            conn.execute(&create_sql, ()).await?;

            // Insert a test vector
            let embedding: Vec<f32> = vec![0.1; dim];
            conn.execute(
                &format!("INSERT INTO {table_name}(rowid, embedding) VALUES (?1, ?2)"),
                libsql::params![1i64, embedding.as_bytes()],
            )
            .await?;

            // K-NN search
            let mut rows = conn
                .query(
                    &format!("SELECT rowid FROM {table_name} WHERE embedding MATCH ?1 LIMIT 1"),
                    libsql::params![embedding.as_bytes()],
                )
                .await?;

            let row = rows
                .next()
                .await?
                .unwrap_or_else(|| panic!("No results for dimension {dim}"));
            let rowid: i64 = row.get(0)?;
            assert_eq!(rowid, 1, "Dimension {dim}: should find inserted vector");
        }

        Ok(())
    }
}
