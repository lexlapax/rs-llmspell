//! SQLite storage backend (Phase 13c.2)
//!
//! Unified local storage using libsql with encryption, connection pooling,
//! and tenant context management for RLS-style isolation.
//!
//! # Features
//!
//! - **libsql**: SQLite fork with encryption at rest and replication
//! - **WAL Mode**: Write-Ahead Logging for concurrent readers
//! - **Connection Pooling**: Custom async pool with PRAGMA initialization  
//! - **Tenant Isolation**: Application-level row filtering via DashMap
//! - **Health Monitoring**: Connection tests and statistics
//!
//! # Examples
//!
//! ```no_run
//! use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create backend
//! let config = SqliteConfig::new("./llmspell.db")
//!     .with_max_connections(20);
//! let backend = SqliteBackend::new(config).await?;
//!
//! // Set tenant context
//! backend.set_tenant_context("tenant-123").await?;
//!
//! // Get connection
//! let conn = backend.get_connection().await?;
//!
//! // Health check
//! assert!(backend.health_check().await?);
//! # Ok(())
//! # }
//! ```

mod backend;
mod config;
mod error;
mod extensions;
mod pool;
mod vector;

pub use backend::{HealthStatus, SqliteBackend, TenantContext};
pub use config::SqliteConfig;
pub use error::{Result, SqliteError};
pub use extensions::SqliteVecExtension;
pub use pool::{PoolStats, SqlitePool};
pub use vector::SqliteVectorStorage;
