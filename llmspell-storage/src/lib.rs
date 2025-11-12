//! Storage backends and persistence for rs-llmspell
//!
//! Provides a unified storage interface with multiple backend implementations,
//! enabling flexible data persistence across different storage systems.
//!
//! # Overview
//!
//! The storage module abstracts data persistence through a common `StorageBackend`
//! trait that can be implemented by various storage systems. This allows applications
//! to switch between storage backends without changing business logic.
//!
//! # Available Backends
//!
//! - **MemoryBackend**: In-memory storage for testing and temporary data
//! - **PostgresBackend**: PostgreSQL for production multi-tenancy (Phase 13b.2)
//! - **SqliteBackend**: SQLite/libsql for embedded production storage (Phase 13c.2)
//! - **Vector Storage**: vectorlite-rs HNSW via SQLite extension (Phase 13c.2.2a)
//! - Custom backends can be implemented via the `StorageBackend` trait
//!
//! # Examples
//!
//! ## Using Memory Backend
//!
//! ```
//! use llmspell_storage::{MemoryBackend, StorageBackend};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backend = MemoryBackend::new();
//!
//! // Store data
//! let value = json!({"name": "Alice"});
//! backend.set("user:123", serde_json::to_vec(&value)?).await?;
//!
//! // Retrieve data
//! let data = backend.get("user:123").await?;
//! if let Some(bytes) = data {
//!     let retrieved: serde_json::Value = serde_json::from_slice(&bytes)?;
//!     assert_eq!(retrieved, json!({"name": "Alice"}));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Using SQLite Backend
//!
//! ```no_run
//! # #[cfg(feature = "sqlite")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use llmspell_storage::backends::sqlite::SqliteBackend;
//! use llmspell_storage::StorageBackend;
//! use serde_json::json;
//!
//! let backend = SqliteBackend::new("./data/storage.db").await?;
//!
//! // Data persists across restarts
//! let value = json!({"version": "1.0"});
//! backend.set("config:app", serde_json::to_vec(&value)?).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Characteristics
//!
//! ## MemoryBackend
//! - **Read**: O(1) average, <1μs
//! - **Write**: O(1) average, <1μs
//! - **Memory**: All data in RAM
//! - **Persistence**: None (data lost on restart)
//! - **Use Case**: Testing, caching, temporary data
//!
//! ## SqliteBackend
//! - **Read**: O(log n), <50μs for indexed operations
//! - **Write**: O(log n), <500μs with WAL mode
//! - **Memory**: Configurable cache, defaults to OS page cache
//! - **Persistence**: ACID compliant, crash-safe with WAL
//! - **Use Case**: Production embedded database, multi-tenancy support
//!
//! # Integration Example
//!
//! ```no_run
//! use llmspell_storage::{StorageBackend, StorageBackendType, MemoryBackend};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Factory pattern for backend selection
//! async fn create_backend(backend_type: StorageBackendType) -> Result<Arc<dyn StorageBackend>, Box<dyn std::error::Error>> {
//!     match backend_type {
//!         StorageBackendType::Memory => Ok(Arc::new(MemoryBackend::new())),
//!         #[cfg(feature = "sqlite")]
//!         StorageBackendType::Sqlite => {
//!             use llmspell_storage::backends::sqlite::SqliteBackend;
//!             Ok(Arc::new(SqliteBackend::new("./data/storage.db").await?))
//!         }
//!         _ => Err("Unsupported backend".into()),
//!     }
//! }
//!
//! // Use backend agnostically
//! let backend = create_backend(StorageBackendType::Memory).await?;
//! let value = serde_json::json!({"value": 42});
//! backend.set("key", serde_json::to_vec(&value)?).await?;
//! # Ok(())
//! # }
//! ```

pub mod backends;
pub mod migration;
pub mod traits;
pub mod vector_storage;

// Re-export commonly used types
pub use backends::MemoryBackend;
pub use traits::{StorageBackend, StorageBackendType, StorageCharacteristics, StorageSerialize};

// Re-export PostgreSQL types (Phase 13b.2+)
#[cfg(feature = "postgres")]
pub use backends::postgres::{
    LargeObjectStream, PostgreSQLVectorStorage, PostgresBackend, PostgresConfig, PostgresError,
    PostgresPool,
};

// Re-export vector storage types
pub use vector_storage::{
    DistanceMetric, HNSWConfig, HNSWStorage, NamespaceStats, ScopedStats, StorageStats,
    VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
