// ABOUTME: SQLite session storage (Phase 13c.2.7)
//! ABOUTME: Session persistence with lifecycle tracking and expiration

use super::backend::SqliteBackend;
use super::error::SqliteError;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::traits::storage::SessionStorage;
use llmspell_core::types::storage::SessionData;
use std::sync::Arc;

/// SQLite-backed session storage
///
/// Provides session persistence with:
/// - Lifecycle tracking (active → archived → expired)
/// - Expiration support with periodic cleanup
/// - Artifact reference counting
/// - Tenant isolation via application-level filtering
///
/// # Performance Targets
/// - create_session: <10ms
/// - get_session: <5ms
/// - list_active_sessions: <50ms
/// - cleanup_expired: <100ms
///
/// # Architecture
/// Implements SessionStorage trait using V9 sessions table.
/// Stores full SessionData as JSON in session_data column with extracted
/// fields for efficient querying (status, expires_at, artifact_count).
#[derive(Clone)]
pub struct SqliteSessionStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteSessionStorage {
    /// Create new SQLite session storage
    ///
    /// # Arguments
    /// * `backend` - SQLite backend with connection pool
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteSessionStorage};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SqliteConfig::new("./llmspell.db");
    /// let backend = Arc::new(SqliteBackend::new(config).await?);
    /// let storage = SqliteSessionStorage::new(backend, "tenant-123".to_string());
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
impl SessionStorage for SqliteSessionStorage {
    async fn create_session(&self, session_id: &str, data: &SessionData) -> Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        // Serialize session data to JSON
        let session_data_json = serde_json::to_string(&data.session_data)
            .map_err(|e| anyhow!("Failed to serialize session_data: {}", e))?;

        // Convert status to string (map Completed to 'archived' for SQL constraint)
        let status = match data.status {
            llmspell_core::types::storage::SessionStatus::Active => "active",
            llmspell_core::types::storage::SessionStatus::Completed => "archived",
            llmspell_core::types::storage::SessionStatus::Expired => "expired",
        };

        // Convert timestamps to Unix seconds
        let created_at = data.created_at.timestamp();
        let expires_at = data.expires_at.map(|dt| dt.timestamp());
        let now = Utc::now().timestamp();

        let stmt = conn
            .prepare(
                "INSERT INTO sessions
                 (tenant_id, session_id, session_data, status, created_at, last_accessed_at, expires_at, artifact_count, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare create_session: {}", e)))?;

        stmt.execute(libsql::params![
            tenant_id,
            session_id,
            session_data_json,
            status,
            created_at,
            now,
            expires_at,
            data.artifact_count as i64,
            now
        ])
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to execute create_session: {}", e)))?;

        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<SessionData>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;
        let now = Utc::now().timestamp();

        // Update last_accessed_at (throttled to 1 minute in application logic)
        let stmt_update = conn
            .prepare(
                "UPDATE sessions
                 SET last_accessed_at = ?1
                 WHERE tenant_id = ?2 AND session_id = ?3
                   AND (last_accessed_at IS NULL OR last_accessed_at < ?4)",
            )
            .await
            .map_err(|e| {
                SqliteError::Query(format!("Failed to prepare update last_accessed: {}", e))
            })?;

        // Update if last access was >60 seconds ago
        let throttle_threshold = now - 60;
        stmt_update
            .execute(libsql::params![
                now,
                tenant_id,
                session_id,
                throttle_threshold
            ])
            .await
            .ok(); // Ignore errors - update is best-effort

        // Retrieve session
        let stmt = conn
            .prepare(
                "SELECT session_data, status, created_at, expires_at, artifact_count
                 FROM sessions
                 WHERE tenant_id = ?1 AND session_id = ?2",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare get_session: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![tenant_id, session_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute get_session: {}", e)))?;

        match rows.next().await {
            Ok(Some(row)) => {
                let session_data_json: String = row
                    .get(0)
                    .map_err(|e| anyhow!("Failed to get session_data: {}", e))?;
                let status_str: String = row
                    .get(1)
                    .map_err(|e| anyhow!("Failed to get status: {}", e))?;
                let created_at_ts: i64 = row
                    .get(2)
                    .map_err(|e| anyhow!("Failed to get created_at: {}", e))?;
                let expires_at_ts: Option<i64> = row
                    .get(3)
                    .map_err(|e| anyhow!("Failed to get expires_at: {}", e))?;
                let artifact_count: i64 = row
                    .get(4)
                    .map_err(|e| anyhow!("Failed to get artifact_count: {}", e))?;

                // Parse session_data JSON
                let session_data_value: serde_json::Value =
                    serde_json::from_str(&session_data_json)
                        .map_err(|e| anyhow!("Failed to parse session_data JSON: {}", e))?;

                // Parse status
                let status = match status_str.as_str() {
                    "active" => llmspell_core::types::storage::SessionStatus::Active,
                    "archived" => llmspell_core::types::storage::SessionStatus::Completed, // Map archived to Completed
                    "expired" => llmspell_core::types::storage::SessionStatus::Expired,
                    _ => return Err(anyhow!("Invalid session status: {}", status_str)),
                };

                // Convert timestamps
                let created_at = DateTime::from_timestamp(created_at_ts, 0)
                    .ok_or_else(|| anyhow!("Invalid created_at timestamp: {}", created_at_ts))?;
                let expires_at = expires_at_ts.and_then(|ts| DateTime::from_timestamp(ts, 0));

                Ok(Some(SessionData {
                    session_id: session_id.to_string(),
                    status,
                    session_data: session_data_value,
                    created_at,
                    expires_at,
                    artifact_count: artifact_count as usize,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow!("Failed to fetch session row: {}", e)),
        }
    }

    async fn update_session(&self, session_id: &str, data: &SessionData) -> Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        // Serialize session data to JSON
        let session_data_json = serde_json::to_string(&data.session_data)
            .map_err(|e| anyhow!("Failed to serialize session_data: {}", e))?;

        // Convert status to string (map Completed to 'archived' for SQL constraint)
        let status = match data.status {
            llmspell_core::types::storage::SessionStatus::Active => "active",
            llmspell_core::types::storage::SessionStatus::Completed => "archived",
            llmspell_core::types::storage::SessionStatus::Expired => "expired",
        };
        let expires_at = data.expires_at.map(|dt| dt.timestamp());
        let now = Utc::now().timestamp();

        let stmt = conn
            .prepare(
                "UPDATE sessions
                 SET session_data = ?1, status = ?2, expires_at = ?3, artifact_count = ?4, updated_at = ?5
                 WHERE tenant_id = ?6 AND session_id = ?7",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare update_session: {}", e)))?;

        let rows_affected = stmt
            .execute(libsql::params![
                session_data_json,
                status,
                expires_at,
                data.artifact_count as i64,
                now,
                tenant_id,
                session_id
            ])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute update_session: {}", e)))?;

        if rows_affected == 0 {
            return Err(anyhow!("Session not found: {}", session_id));
        }

        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare("DELETE FROM sessions WHERE tenant_id = ?1 AND session_id = ?2")
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare delete_session: {}", e)))?;

        stmt.execute(libsql::params![tenant_id, session_id])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute delete_session: {}", e)))?;

        Ok(())
    }

    async fn list_active_sessions(&self) -> Result<Vec<String>> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;

        let stmt = conn
            .prepare(
                "SELECT session_id FROM sessions
                 WHERE tenant_id = ?1 AND status = 'active'
                 ORDER BY created_at DESC",
            )
            .await
            .map_err(|e| {
                SqliteError::Query(format!("Failed to prepare list_active_sessions: {}", e))
            })?;

        let mut rows = stmt.query(libsql::params![tenant_id]).await.map_err(|e| {
            SqliteError::Query(format!("Failed to execute list_active_sessions: {}", e))
        })?;

        let mut session_ids = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch session row: {}", e)))?
        {
            let session_id: String = row
                .get(0)
                .map_err(|e| anyhow!("Failed to get session_id: {}", e))?;
            session_ids.push(session_id);
        }

        Ok(session_ids)
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let tenant_id = self.get_tenant_id();
        let conn = self.backend.get_connection().await?;
        let now = Utc::now().timestamp();

        let stmt = conn
            .prepare(
                "DELETE FROM sessions
                 WHERE tenant_id = ?1
                   AND expires_at IS NOT NULL
                   AND expires_at < ?2",
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to prepare cleanup_expired: {}", e)))?;

        let rows_affected = stmt
            .execute(libsql::params![tenant_id, now])
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to execute cleanup_expired: {}", e)))?;

        Ok(rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use llmspell_core::types::storage::{SessionData, SessionStatus};

    async fn create_test_storage() -> (
        tempfile::TempDir,
        Arc<SqliteBackend>,
        SqliteSessionStorage,
        String,
    ) {
        use crate::backends::sqlite::SqliteConfig;
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually (V1, V9 for sessions tests)
        let conn = backend.get_connection().await.unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();
        conn.execute_batch(include_str!("../../../migrations/sqlite/V9__sessions.sql"))
            .await
            .unwrap();

        let tenant_id = format!("test_tenant_{}", uuid::Uuid::new_v4());
        let storage = SqliteSessionStorage::new(backend.clone(), tenant_id.clone());
        (temp_dir, backend, storage, tenant_id)
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let data = SessionData::new("sess-123");
        storage
            .create_session(&data.session_id, &data)
            .await
            .unwrap();

        let retrieved = storage.get_session("sess-123").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.session_id, "sess-123");
        assert_eq!(retrieved.status, SessionStatus::Active);
        assert_eq!(retrieved.artifact_count, 0);
    }

    #[tokio::test]
    async fn test_update_session() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let mut data = SessionData::new("sess-123");
        storage
            .create_session(&data.session_id, &data)
            .await
            .unwrap();

        data.increment_artifacts();
        data.status = SessionStatus::Completed;
        storage
            .update_session(&data.session_id, &data)
            .await
            .unwrap();

        let retrieved = storage.get_session("sess-123").await.unwrap().unwrap();
        assert_eq!(retrieved.artifact_count, 1);
        assert_eq!(retrieved.status, SessionStatus::Completed);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let data = SessionData::new("sess-123");
        storage
            .create_session(&data.session_id, &data)
            .await
            .unwrap();

        storage.delete_session("sess-123").await.unwrap();

        let retrieved = storage.get_session("sess-123").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_active_sessions() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let data1 = SessionData::new("sess-1");
        let data2 = SessionData::new("sess-2");
        let mut data3 = SessionData::new("sess-3");
        data3.status = SessionStatus::Expired;

        storage
            .create_session(&data1.session_id, &data1)
            .await
            .unwrap();
        storage
            .create_session(&data2.session_id, &data2)
            .await
            .unwrap();
        storage
            .create_session(&data3.session_id, &data3)
            .await
            .unwrap();

        let active = storage.list_active_sessions().await.unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.contains(&"sess-1".to_string()));
        assert!(active.contains(&"sess-2".to_string()));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let past = Utc::now() - Duration::hours(1);
        let future = Utc::now() + Duration::hours(1);

        let data1 = SessionData::with_expiration("sess-1", past);
        let data2 = SessionData::with_expiration("sess-2", future);
        let data3 = SessionData::new("sess-3"); // No expiration

        storage
            .create_session(&data1.session_id, &data1)
            .await
            .unwrap();
        storage
            .create_session(&data2.session_id, &data2)
            .await
            .unwrap();
        storage
            .create_session(&data3.session_id, &data3)
            .await
            .unwrap();

        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Verify sess-1 deleted, sess-2 and sess-3 remain
        assert!(storage.get_session("sess-1").await.unwrap().is_none());
        assert!(storage.get_session("sess-2").await.unwrap().is_some());
        assert!(storage.get_session("sess-3").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_session_with_expiration() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let expires = Utc::now() + Duration::hours(1);
        let data = SessionData::with_expiration("sess-123", expires);

        storage
            .create_session(&data.session_id, &data)
            .await
            .unwrap();

        let retrieved = storage.get_session("sess-123").await.unwrap().unwrap();
        assert!(retrieved.expires_at.is_some());
        assert!(!retrieved.is_expired());
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        use crate::backends::sqlite::SqliteConfig;
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually
        let conn = backend.get_connection().await.unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();
        conn.execute_batch(include_str!("../../../migrations/sqlite/V9__sessions.sql"))
            .await
            .unwrap();

        let tenant1_id = "tenant-1".to_string();
        let tenant2_id = "tenant-2".to_string();

        let storage1 = SqliteSessionStorage::new(backend.clone(), tenant1_id);
        let storage2 = SqliteSessionStorage::new(backend.clone(), tenant2_id);

        let data = SessionData::new("sess-123");
        storage1
            .create_session(&data.session_id, &data)
            .await
            .unwrap();

        // Tenant 2 should not see tenant 1's session
        let retrieved = storage2.get_session("sess-123").await.unwrap();
        assert!(retrieved.is_none());
    }
}
