// ABOUTME: SQLite agent state storage (Phase 13c.2.6)
//! ABOUTME: Storage layer for agent states with versioning and checksum validation

use super::backend::SqliteBackend;
use super::error::SqliteError;
use async_trait::async_trait;
use llmspell_core::traits::storage::StorageBackend;
use llmspell_core::types::storage::{StorageBackendType, StorageCharacteristics};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// SQLite-backed agent state storage
///
/// Stores agent states with:
/// - Tenant isolation via application-level filtering
/// - Automatic versioning (data_version auto-increments on state changes)
/// - SHA256 checksum validation for state integrity
/// - JSON functional indexes for nested field queries
///
/// # Performance Target
/// <10ms for write, <5ms for read (Task 13c.2.6)
///
/// # Architecture
/// This implements the `StorageBackend` trait, using the V6 agent_states table.
/// Keys follow the format: "agent:<agent_id>" for state storage.
///
/// # Versioning
/// The `data_version` field auto-increments via SQLite trigger when `state_data` changes.
/// This provides optimistic locking for concurrent updates.
///
/// # Checksum
/// SHA256 checksums are computed on save and verified on load to detect corruption.
#[derive(Clone)]
pub struct SqliteAgentStateStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteAgentStateStorage {
    /// Create new SQLite agent state storage
    ///
    /// # Arguments
    /// * `backend` - SQLite backend with connection pool
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
    /// use llmspell_storage::backends::sqlite::SqliteAgentStateStorage;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SqliteConfig::new("./llmspell.db");
    /// let backend = Arc::new(SqliteBackend::new(config).await?);
    /// let storage = SqliteAgentStateStorage::new(backend, "tenant-123".to_string());
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

    /// Extract agent_id from storage key
    ///
    /// Keys follow format: "agent:<agent_id>"
    fn extract_agent_id(key: &str) -> Option<&str> {
        key.strip_prefix("agent:")
    }

    /// Build storage key from agent_id
    fn build_key(agent_id: &str) -> String {
        format!("agent:{}", agent_id)
    }

    /// Compute SHA256 checksum of state data
    fn compute_checksum(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Verify checksum matches data
    fn verify_checksum(data: &[u8], expected_checksum: &str) -> bool {
        let computed = Self::compute_checksum(data);
        computed == expected_checksum
    }

    /// Get agent state by agent_id (internal method)
    async fn get_agent_state(&self, agent_id: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare(
                "SELECT state_data, checksum FROM agent_states
                 WHERE tenant_id = ?1 AND agent_id = ?2",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare get_agent_state: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![tenant_id, agent_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute get_agent_state: {}", e)))?;

        match rows.next().await {
            Ok(Some(row)) => {
                let state_data: String = row
                    .get(0)
                    .map_err(|e| SqliteError::Query(format!("Failed to get state_data: {}", e)))?;

                let checksum: String = row
                    .get(1)
                    .map_err(|e| SqliteError::Query(format!("Failed to get checksum: {}", e)))?;

                let data_bytes = state_data.as_bytes();

                // Verify checksum
                if !Self::verify_checksum(data_bytes, &checksum) {
                    return Err(SqliteError::Query(format!(
                        "Checksum mismatch for agent_id {}: expected {}, got {}",
                        agent_id,
                        checksum,
                        Self::compute_checksum(data_bytes)
                    ))
                    .into());
                }

                Ok(Some(data_bytes.to_vec()))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SqliteError::Query(format!("Failed to fetch agent state: {}", e)).into()),
        }
    }

    /// Set agent state by agent_id (internal method)
    async fn set_agent_state(
        &self,
        agent_id: &str,
        agent_type: &str,
        value: Vec<u8>,
    ) -> anyhow::Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;
        let now = chrono::Utc::now().timestamp();

        // Compute checksum
        let checksum = Self::compute_checksum(&value);
        let state_data = String::from_utf8(value)
            .map_err(|e| SqliteError::Query(format!("Invalid UTF-8 in state_data: {}", e)))?;

        // UPSERT with version trigger handling
        let stmt = conn
            .prepare(
                "INSERT INTO agent_states
                 (tenant_id, agent_id, agent_type, state_data, checksum, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(tenant_id, agent_id) DO UPDATE SET
                   agent_type = excluded.agent_type,
                   state_data = excluded.state_data,
                   checksum = excluded.checksum,
                   updated_at = excluded.updated_at",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare set_agent_state: {}", e)))?;

        stmt.execute(libsql::params![
            tenant_id, agent_id, agent_type, state_data, checksum, now, now
        ])
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to execute set_agent_state: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl StorageBackend for SqliteAgentStateStorage {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let agent_id = Self::extract_agent_id(key)
            .ok_or_else(|| SqliteError::Query(format!("Invalid key format: {}", key)))?;

        self.get_agent_state(agent_id).await
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        let agent_id = Self::extract_agent_id(key)
            .ok_or_else(|| SqliteError::Query(format!("Invalid key format: {}", key)))?;

        // Extract agent_type from key or use default
        // Key format can be "agent:<agent_id>:<agent_type>"
        let parts: Vec<&str> = key.split(':').collect();
        let agent_type = if parts.len() >= 3 {
            parts[2]
        } else {
            "unknown"
        };

        self.set_agent_state(agent_id, agent_type, value).await
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let agent_id = Self::extract_agent_id(key)
            .ok_or_else(|| SqliteError::Query(format!("Invalid key format: {}", key)))?;

        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare("DELETE FROM agent_states WHERE tenant_id = ?1 AND agent_id = ?2")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare delete: {}", e)))?;

        stmt.execute(libsql::params![tenant_id, agent_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute delete: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        let agent_id = Self::extract_agent_id(key)
            .ok_or_else(|| SqliteError::Query(format!("Invalid key format: {}", key)))?;

        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare("SELECT 1 FROM agent_states WHERE tenant_id = ?1 AND agent_id = ?2 LIMIT 1")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare exists: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![tenant_id, agent_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute exists: {}", e)))?;

        Ok(rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to check exists: {}", e)))?
            .is_some())
    }

    async fn list_keys(&self, prefix: &str) -> anyhow::Result<Vec<String>> {
        // For agent states, prefix scanning lists all agents with matching agent_id prefix
        let agent_id_prefix = Self::extract_agent_id(prefix).unwrap_or("");

        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let pattern = format!("{}%", agent_id_prefix);
        let stmt = conn
            .prepare(
                "SELECT agent_id FROM agent_states
                 WHERE tenant_id = ?1 AND agent_id LIKE ?2
                 ORDER BY agent_id",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare list_keys: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![tenant_id, pattern])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute list_keys: {}", e)))?;

        let mut keys = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch key row: {}", e)))?
        {
            let agent_id: String = row
                .get(0)
                .map_err(|e| SqliteError::Query(format!("Failed to get agent_id: {}", e)))?;
            keys.push(Self::build_key(&agent_id));
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

        let stmt = conn
            .prepare("DELETE FROM agent_states WHERE tenant_id = ?1")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare clear: {}", e)))?;

        stmt.execute(libsql::params![tenant_id])
            .await
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
            avg_read_latency_us: 1000,  // <5ms target
            avg_write_latency_us: 3000, // <10ms target
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

impl std::fmt::Debug for SqliteAgentStateStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteAgentStateStorage")
            .field("tenant_id", &self.tenant_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::SqliteConfig;
    use tempfile::TempDir;

    async fn create_test_storage() -> (TempDir, Arc<SqliteBackend>, SqliteAgentStateStorage, String)
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually (V1, V6 for agent_state tests)
        let conn = backend.get_connection().await.unwrap();

        // V1: Initial setup
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();

        // V6: Agent states
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V6__agent_state.sql"
        ))
        .await
        .unwrap();

        // Create unique tenant ID
        let tenant_id = format!("test-tenant-{}", uuid::Uuid::new_v4());

        // Set tenant context
        backend.set_tenant_context(&tenant_id).await.unwrap();

        let storage = SqliteAgentStateStorage::new(Arc::clone(&backend), tenant_id.clone());

        (temp_dir, backend, storage, tenant_id)
    }

    #[tokio::test]
    async fn test_set_and_get_agent_state() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let key = "agent:test-agent-1";
        let state_data = b"{\"status\":\"running\",\"step\":5}";

        storage.set(key, state_data.to_vec()).await.unwrap();

        let retrieved = storage.get(key).await.unwrap().expect("State not found");
        assert_eq!(retrieved, state_data);
    }

    #[tokio::test]
    async fn test_checksum_validation() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let key = "agent:checksum-test";
        let state_data = b"{\"validated\":true}";

        // Save with checksum
        storage.set(key, state_data.to_vec()).await.unwrap();

        // Retrieve should verify checksum
        let retrieved = storage.get(key).await.unwrap().unwrap();
        assert_eq!(retrieved, state_data);
    }

    #[tokio::test]
    async fn test_agent_state_versioning() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let key = "agent:versioned-agent";

        // Initial state
        storage.set(key, b"{\"version\":1}".to_vec()).await.unwrap();

        // Update state (should trigger version increment via trigger)
        storage.set(key, b"{\"version\":2}".to_vec()).await.unwrap();

        // Verify we can still retrieve the updated state
        let state = storage.get(key).await.unwrap().unwrap();
        assert_eq!(state, b"{\"version\":2}");
    }

    #[tokio::test]
    async fn test_exists() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let key = "agent:exists-test";
        assert!(!storage.exists(key).await.unwrap());

        storage.set(key, b"{}".to_vec()).await.unwrap();
        assert!(storage.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let key = "agent:delete-test";
        storage.set(key, b"{}".to_vec()).await.unwrap();
        assert!(storage.exists(key).await.unwrap());

        storage.delete(key).await.unwrap();
        assert!(!storage.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn test_list_keys_prefix() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create multiple agents
        storage.set("agent:alpha-1", b"{}".to_vec()).await.unwrap();
        storage.set("agent:alpha-2", b"{}".to_vec()).await.unwrap();
        storage.set("agent:beta-1", b"{}".to_vec()).await.unwrap();

        // List all agents
        let all_keys = storage.list_keys("agent:").await.unwrap();
        assert_eq!(all_keys.len(), 3);

        // List with prefix filter
        let alpha_keys = storage.list_keys("agent:alpha").await.unwrap();
        assert_eq!(alpha_keys.len(), 2);
        assert!(alpha_keys.contains(&"agent:alpha-1".to_string()));
        assert!(alpha_keys.contains(&"agent:alpha-2".to_string()));
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Set batch
        let mut items = HashMap::new();
        items.insert("agent:batch-1".to_string(), b"{\"id\":1}".to_vec());
        items.insert("agent:batch-2".to_string(), b"{\"id\":2}".to_vec());
        items.insert("agent:batch-3".to_string(), b"{\"id\":3}".to_vec());

        storage.set_batch(items.clone()).await.unwrap();

        // Get batch
        let keys = vec![
            "agent:batch-1".to_string(),
            "agent:batch-2".to_string(),
            "agent:batch-3".to_string(),
        ];
        let retrieved = storage.get_batch(&keys).await.unwrap();
        assert_eq!(retrieved.len(), 3);

        // Delete batch
        storage.delete_batch(&keys[0..2]).await.unwrap();
        assert!(!storage.exists("agent:batch-1").await.unwrap());
        assert!(!storage.exists("agent:batch-2").await.unwrap());
        assert!(storage.exists("agent:batch-3").await.unwrap());
    }

    #[tokio::test]
    async fn test_clear() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        storage.set("agent:clear-1", b"{}".to_vec()).await.unwrap();
        storage.set("agent:clear-2", b"{}".to_vec()).await.unwrap();
        storage.set("agent:clear-3", b"{}".to_vec()).await.unwrap();

        assert_eq!(storage.list_keys("agent:").await.unwrap().len(), 3);

        storage.clear().await.unwrap();
        assert_eq!(storage.list_keys("agent:").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually
        let conn = backend.get_connection().await.unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V6__agent_state.sql"
        ))
        .await
        .unwrap();

        let storage1 = SqliteAgentStateStorage::new(Arc::clone(&backend), "tenant-1".to_string());
        let storage2 = SqliteAgentStateStorage::new(Arc::clone(&backend), "tenant-2".to_string());

        // Tenant 1 saves agent state
        storage1
            .set("agent:shared-id", b"{\"tenant\":1}".to_vec())
            .await
            .unwrap();

        // Tenant 2 should not see tenant 1's state
        assert!(storage2.get("agent:shared-id").await.unwrap().is_none());

        // Tenant 2 can save their own state with same agent_id
        storage2
            .set("agent:shared-id", b"{\"tenant\":2}".to_vec())
            .await
            .unwrap();

        // Both tenants see their own states
        let state1 = storage1.get("agent:shared-id").await.unwrap().unwrap();
        let state2 = storage2.get("agent:shared-id").await.unwrap().unwrap();
        assert_eq!(state1, b"{\"tenant\":1}");
        assert_eq!(state2, b"{\"tenant\":2}");
    }

    #[tokio::test]
    async fn test_agent_type_handling() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Key with agent_type
        let key_with_type = "agent:typed-agent:worker";
        storage
            .set(key_with_type, b"{\"type\":\"worker\"}".to_vec())
            .await
            .unwrap();

        // Should be retrievable with full key
        assert!(storage.get(key_with_type).await.unwrap().is_some());
    }
}
