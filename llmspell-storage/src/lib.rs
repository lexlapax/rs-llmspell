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
//! use llmspell_storage::MemoryBackend;
//! use llmspell_core::traits::storage::StorageBackend;
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
//! ```ignore
//! # // Note: SqliteBackend does not implement StorageBackend directly
//! # // Use domain-specific storage types (SqliteVectorStorage, SqliteGraphStorage, etc.)
//! # #[cfg(feature = "sqlite")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
//! use serde_json::json;
//!
//! let config = SqliteConfig::new("./data/storage.db");
//! let backend = SqliteBackend::new(config).await?;
//!
//! // Use domain-specific storage types built on top of SqliteBackend
//! // See SqliteVectorStorage, SqliteGraphStorage, SqliteArtifactStorage examples
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
//! ```
//! use llmspell_storage::{StorageBackendType, MemoryBackend};
//! use llmspell_core::traits::storage::StorageBackend;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Factory pattern for backend selection
//! async fn create_backend(backend_type: StorageBackendType) -> Result<Arc<dyn StorageBackend>, Box<dyn std::error::Error>> {
//!     match backend_type {
//!         StorageBackendType::Memory => Ok(Arc::new(MemoryBackend::new())),
//!         // Note: SQLite storage uses domain-specific types (SqliteVectorStorage, etc.)
//!         // not a general StorageBackend implementation
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

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod backends;
pub mod export_import;
pub mod migration;

// Re-export core traits
pub use llmspell_core::traits::storage::{
    HNSWStorage, KnowledgeGraph, ProceduralMemory, StorageBackend, VectorStorage,
};

// Re-export core types
pub use llmspell_core::types::storage::{
    DistanceMetric, HNSWConfig, NamespaceStats, ScopedStats, StorageBackendType,
    StorageCharacteristics, StorageStats, VectorEntry, VectorQuery, VectorResult,
};

// Re-export backend implementations
pub use backends::MemoryBackend;

// Re-export PostgreSQL types (Phase 13b.2+)
#[cfg(feature = "postgres")]
pub use backends::postgres::{
    LargeObjectStream, PostgreSQLVectorStorage, PostgresBackend, PostgresConfig, PostgresError,
    PostgresPool,
};

// Helper trait for serialization/deserialization
pub trait StorageSerialize: Sized {
    fn to_storage_bytes(&self) -> Result<Vec<u8>>;
    fn from_storage_bytes(bytes: &[u8]) -> Result<Self>;
}

// Default implementation for serde types
impl<T> StorageSerialize for T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn to_storage_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_storage_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
