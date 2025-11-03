//! ABOUTME: PostgreSQL connection pool management
//! ABOUTME: Deadpool-based connection pooling with health checks

use super::config::PostgresConfig;
use super::error::{PostgresError, Result};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

/// PostgreSQL connection pool wrapper
#[derive(Debug, Clone)]
pub struct PostgresPool {
    pool: Pool,
}

impl PostgresPool {
    /// Create a new connection pool from configuration
    pub async fn new(config: &PostgresConfig) -> Result<Self> {
        // Validate configuration
        config.validate().map_err(PostgresError::Config)?;

        // Parse connection string
        let pg_config: tokio_postgres::Config =
            config
                .connection_string
                .parse()
                .map_err(|e: tokio_postgres::Error| {
                    PostgresError::Config(format!("Invalid connection string: {}", e))
                })?;

        // Create pool configuration
        let manager_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let manager = Manager::from_config(pg_config, NoTls, manager_config);

        // Build pool
        let pool = Pool::builder(manager)
            .max_size(config.max_pool_size as usize)
            .build()
            .map_err(|e| PostgresError::Pool(format!("Failed to build pool: {}", e)))?;

        Ok(Self { pool })
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Result<deadpool_postgres::Client> {
        self.pool
            .get()
            .await
            .map_err(|e| PostgresError::Pool(format!("Failed to get connection from pool: {}", e)))
    }

    /// Get pool status (active connections, idle connections)
    pub fn status(&self) -> PoolStatus {
        let status = self.pool.status();
        PoolStatus {
            size: status.size,
            available: status.available,
            max_size: status.max_size,
        }
    }

    /// Check if pool is healthy (can acquire connection)
    pub async fn is_healthy(&self) -> bool {
        self.get().await.is_ok()
    }
}

/// Pool status information
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// Current pool size
    pub size: usize,
    /// Available connections
    pub available: usize,
    /// Maximum pool size
    pub max_size: usize,
}
