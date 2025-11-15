//! Storage backend trait
//!
//! Defines the unified storage interface for all backends (Memory, SQLite, PostgreSQL).
//!
//! The `StorageBackend` trait provides a key-value storage abstraction with batch
//! operations, migrations, and backend introspection capabilities.
//!
//! Migrated from llmspell-storage/src/traits.rs as part of Phase 13c.3.

use crate::types::storage::backend::{StorageBackendType, StorageCharacteristics};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Unified storage backend trait
///
/// Provides a consistent key-value storage interface across different backends.
/// All backends implement the same set of operations with backend-specific
/// optimizations under the hood.
///
/// # Performance Characteristics
///
/// - Memory: <1μs read/write, non-persistent
/// - SQLite: ~50μs read, ~200μs write, persistent
/// - PostgreSQL: ~100μs read, ~300μs write, persistent + distributed
///
/// # Examples
///
/// ```no_run
/// # use llmspell_core::traits::storage::StorageBackend;
/// # async fn example(backend: impl StorageBackend) -> anyhow::Result<()> {
/// // Store and retrieve data
/// backend.set("user:123", b"Alice".to_vec()).await?;
/// let value = backend.get("user:123").await?;
/// assert_eq!(value, Some(b"Alice".to_vec()));
///
/// // Batch operations
/// let mut batch = HashMap::new();
/// batch.insert("key1".to_string(), b"val1".to_vec());
/// batch.insert("key2".to_string(), b"val2".to_vec());
/// backend.set_batch(batch).await?;
///
/// // Prefix scanning
/// let keys = backend.list_keys("user:").await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Get a value by key
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set a key-value pair
    ///
    /// Overwrites the value if the key already exists.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;

    /// Delete a key
    ///
    /// No-op if the key does not exist.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List all keys with a given prefix
    ///
    /// Returns keys in lexicographic order.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Get multiple values by keys
    ///
    /// Returns a map of key → value for all keys that exist.
    /// Missing keys are omitted from the result.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>;

    /// Set multiple key-value pairs
    ///
    /// Atomic if the backend supports transactions, otherwise best-effort.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()>;

    /// Delete multiple keys
    ///
    /// Atomic if the backend supports transactions, otherwise best-effort.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn delete_batch(&self, keys: &[String]) -> Result<()>;

    /// Clear all data (use with caution)
    ///
    /// This operation is irreversible and deletes all data in the backend.
    ///
    /// # Errors
    ///
    /// Returns error if the backend encounters an I/O or internal error.
    async fn clear(&self) -> Result<()>;

    /// Get the backend type
    ///
    /// Returns the backend implementation type (Memory, Sqlite, Postgres).
    fn backend_type(&self) -> StorageBackendType;

    /// Get backend characteristics
    ///
    /// Returns capability and performance information about this backend.
    fn characteristics(&self) -> StorageCharacteristics;

    /// Run database migrations
    ///
    /// Applies all necessary schema migrations for this backend.
    /// For Memory backend, this is a no-op.
    /// For Sqlite/Postgres, this applies versioned SQL migrations.
    ///
    /// # Errors
    ///
    /// Returns error if migrations fail to apply
    async fn run_migrations(&self) -> Result<()>;

    /// Get current migration version
    ///
    /// Returns the highest applied migration version.
    /// Returns 0 if no migrations have been applied or backend doesn't support migrations.
    ///
    /// # Errors
    ///
    /// Returns error if unable to query migration version
    async fn migration_version(&self) -> Result<usize>;
}
