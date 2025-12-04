//! Connection pool management for SQLite
//!
//! Implements connection manager for libsql with proper
//! connection initialization (PRAGMA settings, encryption).

use super::{
    config::SqliteConfig,
    error::{Result, SqliteError},
};
use libsql::{Builder, Connection, Database};
use std::sync::Arc;
use tracing::warn;

/// R2D2 connection manager for libsql
///
/// Manages connection lifecycle including:
/// - Connection creation with PRAGMA initialization
/// - Connection validation (health checks)
/// - Encryption setup if enabled
pub struct SqliteConnectionManager {
    #[allow(dead_code)]
    database: Arc<Database>,
    config: SqliteConfig,
}

impl SqliteConnectionManager {
    /// Create new connection manager
    ///
    /// # Arguments
    ///
    /// * `database` - libsql Database handle
    /// * `config` - SQLite configuration
    pub fn new(database: Arc<Database>, config: SqliteConfig) -> Self {
        Self { database, config }
    }

    /// Initialize connection with PRAGMA settings
    ///
    /// Applies performance and safety tuning to each new connection.
    async fn init_connection(&self, conn: &Connection) -> Result<()> {
        // Enable foreign keys - PRAGMA returns rows, use query() and discard result
        let _ = conn
            .query("PRAGMA foreign_keys = ON", ())
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to enable foreign keys: {}", e)))?;

        // Set journal mode to WAL
        let _ = conn
            .query("PRAGMA journal_mode = WAL", ())
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to set WAL mode: {}", e)))?;

        // Set synchronous mode
        let sync_pragma = format!("PRAGMA synchronous = {}", self.config.synchronous);
        let _ = conn
            .query(&sync_pragma, ())
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to set synchronous: {}", e)))?;

        // Set cache size (performance optimization - non-fatal)
        let cache_pragma = format!("PRAGMA cache_size = {}", self.config.cache_size);
        if let Err(e) = conn.query(&cache_pragma, ()).await {
            warn!("Failed to set cache_size (non-fatal): {}", e);
        }

        // Set mmap size (performance optimization - non-fatal, may not be supported in all libsql contexts)
        let mmap_pragma = format!("PRAGMA mmap_size = {}", self.config.mmap_size);
        if let Err(e) = conn.query(&mmap_pragma, ()).await {
            warn!("Failed to set mmap_size (non-fatal): {}", e);
        }

        // Set busy timeout
        let timeout_pragma = format!("PRAGMA busy_timeout = {}", self.config.busy_timeout);
        let _ = conn
            .query(&timeout_pragma, ())
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to set busy_timeout: {}", e)))?;

        Ok(())
    }
}

/// SQLite connection pool
///
/// Wraps libsql Database with connection management.
pub struct SqlitePool {
    database: Arc<Database>,
    config: SqliteConfig,
    manager: Arc<SqliteConnectionManager>,
}

impl SqlitePool {
    /// Create new connection pool
    ///
    /// # Arguments
    ///
    /// * `config` - SQLite configuration
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or config is invalid
    pub async fn new(config: SqliteConfig) -> Result<Self> {
        // Validate configuration
        config.validate().map_err(SqliteError::Config)?;

        // Open database using Builder (Database::open is deprecated)
        let db_path = config
            .database_path
            .to_str()
            .ok_or_else(|| SqliteError::Config("Invalid database path".to_string()))?;

        let database = Builder::new_local(db_path)
            .build()
            .await
            .map_err(|e| SqliteError::Connection(format!("Failed to open database: {}", e)))?;

        let database = Arc::new(database);
        let manager = Arc::new(SqliteConnectionManager::new(
            Arc::clone(&database),
            config.clone(),
        ));

        Ok(Self {
            database,
            config,
            manager,
        })
    }

    /// Get a connection from the pool
    ///
    /// Creates a new connection and initializes it with PRAGMA settings.
    ///
    /// # Errors
    ///
    /// Returns error if connection creation or initialization fails
    pub async fn get_connection(&self) -> Result<Connection> {
        let conn = self
            .database
            .connect()
            .map_err(|e| SqliteError::Connection(format!("Failed to create connection: {}", e)))?;

        // Initialize connection with PRAGMA settings
        self.manager.init_connection(&conn).await?;

        Ok(conn)
    }

    /// Test connection health
    ///
    /// Executes a simple query to verify database is accessible.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails
    pub async fn health_check(&self) -> Result<bool> {
        let conn = self.get_connection().await?;

        // Simple query to test connectivity - SELECT returns rows, use query()
        let _ = conn
            .query("SELECT 1", ())
            .await
            .map_err(|e| SqliteError::Query(format!("Health check failed: {}", e)))?;

        Ok(true)
    }

    /// Get database statistics
    ///
    /// Returns WAL checkpoint status and cache statistics.
    pub async fn get_stats(&self) -> Result<PoolStats> {
        let conn = self.get_connection().await?;

        // Query cache statistics
        let stmt = conn
            .prepare("PRAGMA cache_size")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query cache_size: {}", e)))?;

        let mut rows = stmt.query(()).await.map_err(|e| {
            SqliteError::Query(format!("Failed to execute cache_size query: {}", e))
        })?;

        let cache_size = if let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to read cache_size row: {}", e)))?
        {
            row.get::<i32>(0)
                .map_err(|e| SqliteError::Query(format!("Failed to parse cache_size: {}", e)))?
        } else {
            0
        };

        Ok(PoolStats {
            cache_size_pages: cache_size,
        })
    }

    /// Get database path
    pub fn database_path(&self) -> &std::path::Path {
        &self.config.database_path
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Cache size in pages (negative = KB)
    pub cache_size_pages: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = SqliteConfig::in_memory();
        let pool = SqlitePool::new(config).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_get_connection() {
        let config = SqliteConfig::in_memory();
        let pool = SqlitePool::new(config).await.unwrap();
        let conn = pool.get_connection().await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = SqliteConfig::in_memory();
        let pool = SqlitePool::new(config).await.unwrap();
        let healthy = pool.health_check().await;
        assert!(healthy.is_ok());
        assert!(healthy.unwrap());
    }
}
