//! ABOUTME: Test storage factory for creating temporary storage backends in tests
//! ABOUTME: Provides convenient helpers for creating in-memory and temporary SQLite storage

use anyhow::Result;
use llmspell_core::traits::storage::{StorageBackend, VectorStorage};
use llmspell_storage::backends::memory::MemoryBackend;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use std::sync::Arc;
use tempfile::TempDir;

/// Test storage factory for creating storage backends in tests
///
/// # Examples
///
/// ```rust,no_run
/// use llmspell_testing::storage::TestStorageFactory;
///
/// #[tokio::test]
/// async fn test_with_memory_backend() {
///     let backend = TestStorageFactory::memory_backend();
///     // Use backend in test
/// }
///
/// #[tokio::test]
/// async fn test_with_sqlite_backend() {
///     let backend = TestStorageFactory::temp_sqlite_backend().await.unwrap();
///     // Use backend in test
/// }
/// ```
pub struct TestStorageFactory;

impl TestStorageFactory {
    /// Create an in-memory storage backend for testing
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_testing::storage::TestStorageFactory;
    ///
    /// let backend = TestStorageFactory::memory_backend();
    /// ```
    pub fn memory_backend() -> Arc<dyn StorageBackend> {
        Arc::new(MemoryBackend::new())
    }

    /// Create a temporary SQLite storage backend for testing
    ///
    /// The backend is created in a temporary directory that will be cleaned up
    /// when the returned TempStorageBackend is dropped.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_testing::storage::TestStorageFactory;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let temp = TestStorageFactory::temp_sqlite_backend().await?;
    /// let backend = temp.backend();
    /// // Use backend in test
    /// # Ok(())
    /// # }
    /// ```
    pub async fn temp_sqlite_backend() -> Result<TempStorageBackend> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig {
            database_path: db_path,
            ..Default::default()
        };
        let backend = SqliteBackend::new(config).await?;

        Ok(TempStorageBackend {
            backend: Arc::new(backend),
            _temp_dir: temp_dir,
        })
    }

    /// Create a temporary SQLite vector storage for testing
    ///
    /// # Arguments
    ///
    /// * `dimension` - The dimension of vectors to store
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_testing::storage::TestStorageFactory;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let temp = TestStorageFactory::temp_vector_storage(384).await?;
    /// let storage = temp.storage();
    /// // Use storage in test
    /// # Ok(())
    /// # }
    /// ```
    pub async fn temp_vector_storage(dimension: usize) -> Result<TempVectorStorage> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_vectors.db");

        let config = SqliteConfig {
            database_path: db_path,
            ..Default::default()
        };
        let backend = SqliteBackend::new(config).await?;
        let backend_arc = Arc::new(backend);
        let storage = SqliteVectorStorage::new(backend_arc, dimension).await?;

        Ok(TempVectorStorage {
            storage: Arc::new(storage),
            _temp_dir: temp_dir,
        })
    }
}

/// Temporary SQLite storage backend that cleans up on drop
///
/// This struct wraps a SQLite backend and a temporary directory. When dropped,
/// the temporary directory and all its contents are automatically removed.
pub struct TempStorageBackend {
    backend: Arc<SqliteBackend>,
    _temp_dir: TempDir,
}

impl TempStorageBackend {
    /// Get a reference to the storage backend
    pub fn backend(&self) -> &Arc<SqliteBackend> {
        &self.backend
    }

    /// Get a cloned Arc to the storage backend
    pub fn backend_arc(&self) -> Arc<SqliteBackend> {
        Arc::clone(&self.backend)
    }
}

/// Temporary SQLite vector storage that cleans up on drop
///
/// This struct wraps a SQLite vector storage and a temporary directory. When dropped,
/// the temporary directory and all its contents are automatically removed.
pub struct TempVectorStorage {
    storage: Arc<SqliteVectorStorage>,
    _temp_dir: TempDir,
}

impl TempVectorStorage {
    /// Get a reference to the vector storage
    pub fn storage(&self) -> &Arc<SqliteVectorStorage> {
        &self.storage
    }

    /// Get a cloned Arc to the vector storage
    pub fn storage_arc(&self) -> Arc<SqliteVectorStorage> {
        Arc::clone(&self.storage)
    }

    /// Get a trait object reference to the vector storage
    pub fn as_vector_storage(&self) -> Arc<dyn VectorStorage> {
        Arc::clone(&self.storage) as Arc<dyn VectorStorage>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::types::storage::StorageBackendType;

    #[test]
    fn test_memory_backend_creation() {
        let backend = TestStorageFactory::memory_backend();
        assert_eq!(backend.backend_type(), StorageBackendType::Memory);
    }

    #[tokio::test]
    async fn test_temp_sqlite_backend_creation() {
        let temp = TestStorageFactory::temp_sqlite_backend().await.unwrap();
        let backend = temp.backend();
        assert_eq!(backend.backend_type(), StorageBackendType::Sqlite);
    }

    #[tokio::test]
    async fn test_temp_vector_storage_creation() {
        let temp = TestStorageFactory::temp_vector_storage(384).await.unwrap();
        let _storage = temp.storage();
        // Just verify we can create it without panicking
    }
}
