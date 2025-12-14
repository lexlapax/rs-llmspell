// ABOUTME: SQLite generic key-value storage (Phase 13c.2.6)
//! ABOUTME: Storage layer for generic KV fallback with binary-safe BLOB storage

use super::backend::SqliteBackend;
use super::error::SqliteError;
use async_trait::async_trait;
use llmspell_core::traits::storage::StorageBackend;
use llmspell_core::types::storage::{StorageBackendType, StorageCharacteristics};
use rusqlite::params;
use std::collections::HashMap;
use std::sync::Arc;

/// SQLite-backed generic key-value storage
///
/// Provides binary-safe storage for generic key-value pairs with:
/// - Tenant isolation via application-level filtering
/// - Binary-safe BLOB storage for values
/// - Key prefix scanning support
/// - Optional JSON metadata for each entry
///
/// # Performance Target
/// <10ms for write, <5ms for read (Task 13c.2.6)
///
/// # Architecture
/// This implements the `StorageBackend` trait, using the V7 kv_store table.
/// Used as a fallback for state keys that don't fit into specialized tables.
#[derive(Clone)]
pub struct SqliteKVStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteKVStorage {
    /// Create new SQLite KV storage
    ///
    /// # Arguments
    /// * `backend` - SQLite backend with connection pool
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
    /// use llmspell_storage::backends::sqlite::SqliteKVStorage;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SqliteConfig::new("./llmspell.db");
    /// let backend = Arc::new(SqliteBackend::new(config).await?);
    /// let storage = SqliteKVStorage::new(backend, "tenant-123".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: Arc<SqliteBackend>, tenant_id: String) -> Self {
        Self { backend, tenant_id }
    }

    /// Get tenant ID for queries
    fn get_tenant_id(&self) -> &str {
        &self.tenant_id
    }
}

#[async_trait]
impl StorageBackend for SqliteKVStorage {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let mut stmt = conn
            .prepare("SELECT value FROM kv_store WHERE tenant_id = ?1 AND key = ?2")
            .map_err(|e| SqliteError::Query(format!("Failed to prepare get query: {}", e)))?;

        let mut rows = stmt
            .query(params![tenant_id, key])
            .map_err(|e| SqliteError::Query(format!("Failed to execute get: {}", e)))?;

        match rows.next() {
            Ok(Some(row)) => {
                let value: Vec<u8> = row
                    .get(0)
                    .map_err(|e| SqliteError::Query(format!("Failed to get value: {}", e)))?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SqliteError::Query(format!("Failed to fetch row: {}", e)).into()),
        }
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;
        let now = chrono::Utc::now().timestamp();

        // UPSERT: insert if not exists, update if exists
        let mut stmt = conn
            .prepare(
                "INSERT INTO kv_store (tenant_id, key, value, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(tenant_id, key) DO UPDATE SET
                   value = excluded.value,
                   updated_at = excluded.updated_at",
            )
            .map_err(|e| SqliteError::Query(format!("Failed to prepare set query: {}", e)))?;

        stmt.execute(params![tenant_id, key, value, now, now])
            .map_err(|e| SqliteError::Query(format!("Failed to execute set: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let mut stmt = conn
            .prepare("DELETE FROM kv_store WHERE tenant_id = ?1 AND key = ?2")
            .map_err(|e| SqliteError::Query(format!("Failed to prepare delete query: {}", e)))?;

        stmt.execute(params![tenant_id, key])
            .map_err(|e| SqliteError::Query(format!("Failed to execute delete: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let mut stmt = conn
            .prepare("SELECT 1 FROM kv_store WHERE tenant_id = ?1 AND key = ?2 LIMIT 1")
            .map_err(|e| SqliteError::Query(format!("Failed to prepare exists query: {}", e)))?;

        let mut rows = stmt
            .query(params![tenant_id, key])
            .map_err(|e| SqliteError::Query(format!("Failed to execute exists: {}", e)))?;

        Ok(rows
            .next()
            .map_err(|e| SqliteError::Query(format!("Failed to check exists: {}", e)))?
            .is_some())
    }

    async fn list_keys(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        // Use LIKE with prefix for prefix scanning
        let pattern = format!("{}%", prefix);

        let mut stmt = conn
            .prepare("SELECT key FROM kv_store WHERE tenant_id = ?1 AND key LIKE ?2 ORDER BY key")
            .map_err(|e| SqliteError::Query(format!("Failed to prepare list_keys query: {}", e)))?;

        let mut rows = stmt
            .query(params![tenant_id, pattern])
            .map_err(|e| SqliteError::Query(format!("Failed to execute list_keys: {}", e)))?;

        let mut keys = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| SqliteError::Query(format!("Failed to fetch key row: {}", e)))?
        {
            let key: String = row
                .get(0)
                .map_err(|e| SqliteError::Query(format!("Failed to get key: {}", e)))?;
            keys.push(key);
        }

        Ok(keys)
    }

    async fn get_batch(&self, keys: &[String]) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        let mut result = HashMap::new();

        for key in keys {
            if let Some(value) = self.get(key).await? {
                result.insert(key.clone(), value);
            }
        }

        Ok(result)
    }

    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> anyhow::Result<()> {
        for (key, value) in items {
            self.set(&key, value).await?;
        }
        Ok(())
    }

    async fn delete_batch(&self, keys: &[String]) -> anyhow::Result<()> {
        for key in keys {
            self.delete(key).await?;
        }
        Ok(())
    }

    async fn clear(&self) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let mut stmt = conn
            .prepare("DELETE FROM kv_store WHERE tenant_id = ?1")
            .map_err(|e| SqliteError::Query(format!("Failed to prepare clear: {}", e)))?;

        stmt.execute(params![tenant_id])
            .map_err(|e| SqliteError::Query(format!("Failed to execute clear: {}", e)))?;

        Ok(())
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Sqlite
    }

    fn characteristics(&self) -> StorageCharacteristics {
        StorageCharacteristics {
            persistent: true,
            transactional: true,
            supports_prefix_scan: true,
            supports_atomic_ops: true,
            avg_read_latency_us: 1000,  // <5ms target = <5000us
            avg_write_latency_us: 3000, // <10ms target = <10000us
        }
    }

    async fn run_migrations(&self) -> anyhow::Result<()> {
        // Delegate to underlying SqliteBackend
        self.backend.run_migrations().await
    }

    async fn migration_version(&self) -> anyhow::Result<usize> {
        // Delegate to underlying SqliteBackend
        self.backend.migration_version().await
    }
}

impl std::fmt::Debug for SqliteKVStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteKVStorage")
            .field("tenant_id", &self.tenant_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::SqliteConfig;
    use tempfile::TempDir;

    async fn create_test_storage() -> (TempDir, Arc<SqliteBackend>, SqliteKVStorage, String) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually (V1, V7 for kv_store tests)
        // Using synchronous migration
        backend.run_migrations().await.unwrap();

        // Load V7 specifically? run_migrations runs all.
        // But for test setup we might want to be explicit if run_migrations logic was partial?
        // backend.run_migrations() handles checking/running all pending.

        // But the original code manually ran V1 and V7.
        // run_migrations should be enough if it includes all.
        // Let's rely on run_migrations() which uses embedded migrations.

        // Create unique tenant ID
        let tenant_id = format!("test-tenant-{}", uuid::Uuid::new_v4());

        // Set tenant context
        backend.set_tenant_context(&tenant_id).await.unwrap();

        let storage = SqliteKVStorage::new(Arc::clone(&backend), tenant_id.clone());

        (temp_dir, backend, storage, tenant_id)
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let key = "test:key";
        let value = b"test value".to_vec();

        // Set value
        storage.set(key, value.clone()).await.unwrap();

        // Get value
        let retrieved = storage.get(key).await.unwrap().expect("Value not found");
        assert_eq!(retrieved, value);
    }

    #[tokio::test]
    async fn test_binary_safe_storage() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Test binary data with null bytes
        let binary_data = vec![0u8, 1, 2, 255, 0, 128, 0];
        storage
            .set("binary:key", binary_data.clone())
            .await
            .unwrap();

        let retrieved = storage.get("binary:key").await.unwrap().unwrap();
        assert_eq!(retrieved, binary_data);
    }

    #[tokio::test]
    async fn test_exists() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        assert!(!storage.exists("nonexistent").await.unwrap());

        storage.set("exists:key", b"value".to_vec()).await.unwrap();
        assert!(storage.exists("exists:key").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        storage.set("delete:key", b"value".to_vec()).await.unwrap();
        assert!(storage.exists("delete:key").await.unwrap());

        storage.delete("delete:key").await.unwrap();
        assert!(!storage.exists("delete:key").await.unwrap());
    }

    #[tokio::test]
    async fn test_prefix_scan() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Insert keys with common prefix
        storage.set("prefix:a", b"1".to_vec()).await.unwrap();
        storage.set("prefix:b", b"2".to_vec()).await.unwrap();
        storage.set("prefix:c", b"3".to_vec()).await.unwrap();
        storage.set("other:d", b"4".to_vec()).await.unwrap();

        // List keys with prefix
        let keys = storage.list_keys("prefix:").await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"prefix:a".to_string()));
        assert!(keys.contains(&"prefix:b".to_string()));
        assert!(keys.contains(&"prefix:c".to_string()));
        assert!(!keys.contains(&"other:d".to_string()));
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Set batch
        let mut items = HashMap::new();
        items.insert("batch:1".to_string(), b"value1".to_vec());
        items.insert("batch:2".to_string(), b"value2".to_vec());
        items.insert("batch:3".to_string(), b"value3".to_vec());

        storage.set_batch(items.clone()).await.unwrap();

        // Get batch
        let keys = vec![
            "batch:1".to_string(),
            "batch:2".to_string(),
            "batch:3".to_string(),
        ];
        let retrieved = storage.get_batch(&keys).await.unwrap();

        assert_eq!(retrieved.len(), 3);
        assert_eq!(retrieved.get("batch:1").unwrap(), b"value1");
        assert_eq!(retrieved.get("batch:2").unwrap(), b"value2");
        assert_eq!(retrieved.get("batch:3").unwrap(), b"value3");

        // Delete batch
        storage.delete_batch(&keys[0..2]).await.unwrap();
        assert!(!storage.exists("batch:1").await.unwrap());
        assert!(!storage.exists("batch:2").await.unwrap());
        assert!(storage.exists("batch:3").await.unwrap());
    }

    #[tokio::test]
    async fn test_upsert() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Initial set
        storage.set("upsert:key", b"value1".to_vec()).await.unwrap();
        let v1 = storage.get("upsert:key").await.unwrap().unwrap();
        assert_eq!(v1, b"value1");

        // Update with new value
        storage.set("upsert:key", b"value2".to_vec()).await.unwrap();
        let v2 = storage.get("upsert:key").await.unwrap().unwrap();
        assert_eq!(v2, b"value2");

        // Verify only one record exists
        let keys = storage.list_keys("upsert:").await.unwrap();
        assert_eq!(keys.len(), 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Insert multiple keys
        storage.set("clear:1", b"v1".to_vec()).await.unwrap();
        storage.set("clear:2", b"v2".to_vec()).await.unwrap();
        storage.set("clear:3", b"v3".to_vec()).await.unwrap();

        assert_eq!(storage.list_keys("clear:").await.unwrap().len(), 3);

        // Clear all
        storage.clear().await.unwrap();
        assert_eq!(storage.list_keys("clear:").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually
        backend.run_migrations().await.unwrap();

        let storage1 = SqliteKVStorage::new(Arc::clone(&backend), "tenant-1".to_string());
        let storage2 = SqliteKVStorage::new(Arc::clone(&backend), "tenant-2".to_string());

        // Tenant 1 sets a value
        storage1
            .set("shared:key", b"tenant1-value".to_vec())
            .await
            .unwrap();

        // Tenant 2 should not see tenant 1's value
        assert!(storage2.get("shared:key").await.unwrap().is_none());

        // Tenant 2 sets their own value with same key
        storage2
            .set("shared:key", b"tenant2-value".to_vec())
            .await
            .unwrap();

        // Both tenants should see their own values
        assert_eq!(
            storage1.get("shared:key").await.unwrap().unwrap(),
            b"tenant1-value"
        );
        assert_eq!(
            storage2.get("shared:key").await.unwrap().unwrap(),
            b"tenant2-value"
        );
    }
}
