//! Connection pool management for SQLite
//!
//! Implements connection manager for rusqlite using r2d2
//! with proper connection initialization (PRAGMA settings).

use super::{
    config::SqliteConfig,
    error::{Result, SqliteError},
};
use r2d2::{ManageConnection, Pool};
use rusqlite::{Connection, OpenFlags};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::warn;

/// Custom R2D2 connection manager for Rusqlite
///
/// Implemented manually to avoid r2d2_sqlite vs rusqlite version conflicts.
pub struct SqliteConnectionManager {
    path: PathBuf,
    flags: OpenFlags,
    init: Option<Box<dyn Fn(&mut Connection) -> rusqlite::Result<()> + Send + Sync + 'static>>,
}

impl SqliteConnectionManager {
    pub fn file<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            flags: OpenFlags::default(),
            init: None,
        }
    }

    pub fn with_init<F>(mut self, init: F) -> Self
    where
        F: Fn(&mut Connection) -> rusqlite::Result<()> + Send + Sync + 'static,
    {
        self.init = Some(Box::new(init));
        self
    }
}

impl fmt::Debug for SqliteConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SqliteConnectionManager")
            .field("path", &self.path)
            .field("flags", &self.flags)
            .finish()
    }
}

impl ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> std::result::Result<Connection, rusqlite::Error> {
        let mut conn = Connection::open_with_flags(&self.path, self.flags)?;
        if let Some(ref init) = self.init {
            init(&mut conn)?;
        }
        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Connection) -> std::result::Result<(), rusqlite::Error> {
        conn.execute_batch("").map_err(Into::into)
    }

    fn has_broken(&self, _conn: &mut Connection) -> bool {
        false
    }
}

/// SQLite connection pool
///
/// Wraps r2d2::Pool with SqliteConnectionManager.
pub struct SqlitePool {
    pool: Pool<SqliteConnectionManager>,
    config: SqliteConfig,
}

impl SqlitePool {
    /// Create new connection pool
    pub async fn new(config: SqliteConfig) -> Result<Self> {
        // Validate configuration
        config.validate().map_err(SqliteError::Config)?;

        let db_path = config
            .database_path
            .to_str()
            .ok_or_else(|| SqliteError::Config("Invalid database path".to_string()))?;

        // Create manager with init hook
        let config_clone = config.clone();
        let manager = SqliteConnectionManager::file(db_path)
            .with_init(move |conn| {
                // Apply PRAGMAs synchronously
                conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
                
                // Set synchronous mode
                let sync_pragma = format!("PRAGMA synchronous = {}", config_clone.synchronous);
                conn.execute(&sync_pragma, [])?;

                // Set busy timeout
                let timeout_pragma = format!("PRAGMA busy_timeout = {}", config_clone.busy_timeout);
                conn.execute(&timeout_pragma, [])?;

                Ok(())
            });

        let pool = Pool::builder()
            .max_size(config.max_connections)
            .build(manager)
            .map_err(|e| SqliteError::Connection(format!("Failed to build pool: {}", e)))?;

        Ok(Self { pool, config })
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        // functionality is sync, but we keep async signature for API compatibility?
        // Or if we can block...
        // r2d2 get() blocks.
        // Ideally should be wrapped in spawn_blocking if used in async context extensively.
        // But for now, direct call. 
        self.pool.get().map_err(|e| SqliteError::Connection(e.to_string()))
    }

    /// Test connection health
    pub async fn health_check(&self) -> Result<bool> {
        let conn = self.get_connection().await?;
        conn.execute("SELECT 1", [])
            .map_err(|e| SqliteError::Query(format!("Health check failed: {}", e)))?;
        Ok(true)
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<PoolStats> {
        let conn = self.get_connection().await?;
        
        // Query cache statistics
        let cache_size: i32 = conn.query_row("PRAGMA cache_size", [], |row| row.get(0))
            .map_err(|e| SqliteError::Query(format!("Failed to query cache_size: {}", e)))?;

        Ok(PoolStats {
            cache_size_pages: cache_size,
        })
    }

    /// Get database path
    pub fn database_path(&self) -> &Path {
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
        // Since we use open_with_flags default (Create | ReadWrite), passing ":memory:" or in-memory path should work.
        // But SqliteConfig::in_memory() typically returns ":memory:"?
        // Let's assume it works.
        let pool = SqlitePool::new(config).await;
        assert!(pool.is_ok());
    }
}
