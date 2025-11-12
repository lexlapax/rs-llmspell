//! SQLite artifact storage implementation
//!
//! Content-addressed artifact storage with deduplication using 2-table structure:
//! - artifact_content: Content storage with reference counting
//! - artifacts: Metadata and references

use crate::backends::sqlite::{Result, SqliteBackend, SqliteError};
use anyhow::Context;
use async_trait::async_trait;
use llmspell_core::traits::storage::ArtifactStorage;
use llmspell_core::types::storage::{Artifact, ArtifactId, ArtifactType, SessionStorageStats};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// SQLite artifact storage implementation
///
/// Implements ArtifactStorage trait using libsql with content-addressed deduplication.
/// Uses 2-table structure:
/// - artifact_content: Content storage (BLOB) with blake3 hash + reference counting
/// - artifacts: Metadata with foreign key to content
///
/// # Architecture
///
/// - **Content Addressing**: blake3 hash (64 hex chars) uniquely identifies content
/// - **Deduplication**: Reference counting managed in application code (no triggers)
/// - **Tenant Isolation**: Application-level filtering via tenant_id
/// - **BLOB Storage**: SQLite handles all sizes efficiently
///
/// # Examples
///
/// ```no_run
/// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
/// use llmspell_storage::backends::sqlite::SqliteArtifactStorage;
/// use llmspell_core::types::storage::{Artifact, ArtifactType};
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// let backend = Arc::new(SqliteBackend::new(SqliteConfig::new("test.db")).await?);
/// backend.set_tenant_context("tenant-1").await?;
/// let storage = SqliteArtifactStorage::new(backend.clone(), "tenant-1".to_string());
///
/// // Store artifact (automatic deduplication)
/// let artifact = Artifact::new(
///     "hash123".to_string(),
///     "session-1".to_string(),
///     ArtifactType::Code,
///     b"fn main() {}".to_vec(),
/// );
/// let id = storage.store_artifact(&artifact).await?;
/// # Ok(())
/// # }
/// ```
pub struct SqliteArtifactStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteArtifactStorage {
    /// Create new SQLite artifact storage
    ///
    /// # Arguments
    ///
    /// * `backend` - Shared SQLite backend
    /// * `tenant_id` - Tenant identifier for isolation
    pub fn new(backend: Arc<SqliteBackend>, tenant_id: String) -> Self {
        Self { backend, tenant_id }
    }

    /// Compute SHA-256 hash of content
    fn compute_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Increment reference count for content hash
    ///
    /// Returns true if content already exists, false if new
    async fn increment_refcount(&self, content_hash: &str, content: &[u8]) -> Result<bool> {
        let conn = self.backend.get_connection().await?;

        // Check if content exists
        let mut rows = conn
            .query(
                "SELECT reference_count FROM artifact_content
                 WHERE tenant_id = ?1 AND content_hash = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(content_hash.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to check content existence: {}", e)))?;

        let exists = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch content row: {}", e)))?
            .and_then(|row| row.get::<i64>(0).ok());

        if let Some(_refcount) = exists {
            // Increment existing reference count
            conn.execute(
                "UPDATE artifact_content
                 SET reference_count = reference_count + 1,
                     last_accessed_at = strftime('%s', 'now')
                 WHERE tenant_id = ?1 AND content_hash = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(content_hash.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to increment refcount: {}", e)))?;

            Ok(true)
        } else {
            // Insert new content
            let size_bytes = content.len() as i64;
            let is_compressed = 0i64; // Not compressed in this implementation

            conn.execute(
                "INSERT INTO artifact_content
                 (tenant_id, content_hash, data, size_bytes, is_compressed, reference_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1)",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(content_hash.to_string()),
                    libsql::Value::Blob(content.to_vec()),
                    libsql::Value::Integer(size_bytes),
                    libsql::Value::Integer(is_compressed),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to insert content: {}", e)))?;

            Ok(false)
        }
    }

    /// Decrement reference count for content hash
    ///
    /// Deletes content if reference count reaches 1 (will become 0)
    async fn decrement_refcount(&self, content_hash: &str) -> Result<()> {
        let conn = self.backend.get_connection().await?;

        // Delete content if refcount = 1 (it will become 0 after decrement)
        // This avoids violating the CHECK constraint (reference_count > 0)
        let deleted = conn
            .execute(
                "DELETE FROM artifact_content
                 WHERE tenant_id = ?1 AND content_hash = ?2 AND reference_count = 1",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(content_hash.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to delete content: {}", e)))?;

        // If we didn't delete (refcount > 1), just decrement
        if deleted == 0 {
            conn.execute(
                "UPDATE artifact_content
                 SET reference_count = reference_count - 1
                 WHERE tenant_id = ?1 AND content_hash = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(content_hash.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to decrement refcount: {}", e)))?;
        }

        Ok(())
    }

    /// Parse ArtifactType from string
    fn parse_artifact_type(type_str: &str) -> anyhow::Result<ArtifactType> {
        match type_str {
            "code" => Ok(ArtifactType::Code),
            "data" => Ok(ArtifactType::Data),
            "image" => Ok(ArtifactType::Image),
            "document" => Ok(ArtifactType::Document),
            "binary" => Ok(ArtifactType::Binary),
            _ => anyhow::bail!("unknown artifact type: {}", type_str),
        }
    }
}

#[async_trait]
impl ArtifactStorage for SqliteArtifactStorage {
    async fn store_artifact(&self, artifact: &Artifact) -> anyhow::Result<ArtifactId> {
        let content_hash = Self::compute_hash(&artifact.content);

        // Increment refcount (inserts content if new)
        self.increment_refcount(&content_hash, &artifact.content)
            .await
            .context("failed to manage content reference")?;

        let conn = self.backend.get_connection().await?;

        // Generate artifact_id format: "{session_id}:{sequence}:{content_hash}"
        // Get next sequence number for session
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(sequence), -1) + 1
                 FROM artifacts
                 WHERE tenant_id = ?1 AND session_id = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(artifact.artifact_id.session_id.clone()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get next sequence: {}", e)))?;

        let sequence: i64 = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch sequence row: {}", e)))?
            .and_then(|row| row.get::<i64>(0).ok())
            .unwrap_or(0);

        let artifact_id = format!(
            "{}:{}:{}",
            artifact.artifact_id.session_id, sequence, content_hash
        );

        // Serialize metadata as JSON
        let metadata_json =
            serde_json::to_string(&artifact.metadata).context("failed to serialize metadata")?;

        let artifact_type_str = artifact.artifact_type.as_str();
        let size_bytes = artifact.content.len() as i64;
        let created_at = artifact.created_at.timestamp();

        // Extract common metadata fields
        let name = artifact
            .metadata
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");
        let mime_type = artifact
            .metadata
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream");
        let created_by = artifact.metadata.get("created_by").and_then(|v| v.as_str());

        // Insert artifact metadata
        conn.execute(
            "INSERT INTO artifacts
             (tenant_id, artifact_id, session_id, sequence, content_hash, metadata,
              name, artifact_type, mime_type, size_bytes, created_at, created_by, stored_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, strftime('%s', 'now'))",
            vec![
                libsql::Value::Text(self.tenant_id.clone()),
                libsql::Value::Text(artifact_id.clone()),
                libsql::Value::Text(artifact.artifact_id.session_id.clone()),
                libsql::Value::Integer(sequence),
                libsql::Value::Text(content_hash.clone()),
                libsql::Value::Text(metadata_json),
                libsql::Value::Text(name.to_string()),
                libsql::Value::Text(artifact_type_str.to_string()),
                libsql::Value::Text(mime_type.to_string()),
                libsql::Value::Integer(size_bytes),
                libsql::Value::Integer(created_at),
                created_by
                    .map(|s| libsql::Value::Text(s.to_string()))
                    .unwrap_or(libsql::Value::Null),
            ],
        )
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to insert artifact: {}", e)))?;

        // Update session artifact_count
        conn.execute(
            "UPDATE sessions
             SET artifact_count = artifact_count + 1
             WHERE tenant_id = ?1 AND session_id = ?2",
            vec![
                libsql::Value::Text(self.tenant_id.clone()),
                libsql::Value::Text(artifact.artifact_id.session_id.clone()),
            ],
        )
        .await
        .map_err(|e| {
            SqliteError::Query(format!("Failed to update session artifact count: {}", e))
        })?;

        // Return ArtifactId with the database artifact_id as content_hash
        // This allows delete_artifact to uniquely identify the artifact row
        Ok(ArtifactId::new(
            artifact_id.clone(),  // Full database ID: "{session_id}:{sequence}:{content_hash}"
            artifact.artifact_id.session_id.clone(),
        ))
    }

    async fn get_artifact(&self, artifact_id: &ArtifactId) -> anyhow::Result<Option<Artifact>> {
        let conn = self.backend.get_connection().await?;

        // artifact_id.content_hash contains the database artifact_id: "{session_id}:{sequence}:{content_hash}"
        // Join artifacts + artifact_content to retrieve full artifact
        let mut rows = conn
            .query(
                "SELECT a.artifact_type, a.metadata, a.size_bytes, a.created_at, c.data
                 FROM artifacts a
                 INNER JOIN artifact_content c ON
                     a.tenant_id = c.tenant_id AND a.content_hash = c.content_hash
                 WHERE a.tenant_id = ?1
                   AND a.artifact_id = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(artifact_id.content_hash.clone()),  // This is the database artifact_id
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get artifact: {}", e)))?;

        let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch artifact row: {}", e)))?
        else {
            return Ok(None);
        };

        let artifact_type_str = row
            .get::<String>(0)
            .context("failed to get artifact_type")?;
        let artifact_type = Self::parse_artifact_type(&artifact_type_str)?;

        let metadata_json = row.get::<String>(1).context("failed to get metadata")?;
        let metadata: serde_json::Value =
            serde_json::from_str(&metadata_json).context("failed to deserialize metadata")?;

        let size_bytes = row.get::<i64>(2).context("failed to get size_bytes")? as usize;

        let created_at_ts = row.get::<i64>(3).context("failed to get created_at")?;
        let created_at =
            chrono::DateTime::from_timestamp(created_at_ts, 0).context("invalid timestamp")?;

        let content = row.get::<Vec<u8>>(4).context("failed to get content")?;

        // Update last_accessed_at (throttled to 1 minute in production)
        conn.execute(
            "UPDATE artifact_content
             SET last_accessed_at = strftime('%s', 'now')
             WHERE tenant_id = ?1 AND content_hash = ?2
               AND last_accessed_at < strftime('%s', 'now', '-1 minute')",
            vec![
                libsql::Value::Text(self.tenant_id.clone()),
                libsql::Value::Text(artifact_id.content_hash.clone()),
            ],
        )
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to update last_accessed_at: {}", e)))?;

        Ok(Some(Artifact {
            artifact_id: artifact_id.clone(),
            artifact_type,
            content,
            metadata,
            size_bytes,
            created_at,
        }))
    }

    async fn delete_artifact(&self, artifact_id: &ArtifactId) -> anyhow::Result<bool> {
        let conn = self.backend.get_connection().await?;

        // artifact_id.content_hash contains the database artifact_id: "{session_id}:{sequence}:{content_hash}"
        // We need to extract the actual content_hash and delete by artifact_id

        // First, get the content_hash before deletion
        let mut rows = conn
            .query(
                "SELECT content_hash FROM artifacts
                 WHERE tenant_id = ?1 AND artifact_id = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(artifact_id.content_hash.clone()),  // This is the database artifact_id
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get content_hash: {}", e)))?;

        let content_hash = if let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch content_hash: {}", e)))?
        {
            row.get::<String>(0)
                .context("failed to get content_hash from row")?
        } else {
            // Artifact doesn't exist
            return Ok(false);
        };

        // Delete artifact metadata using artifact_id (unique identifier)
        let rows_affected = conn
            .execute(
                "DELETE FROM artifacts
                 WHERE tenant_id = ?1 AND artifact_id = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(artifact_id.content_hash.clone()),  // This is the database artifact_id
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to delete artifact: {}", e)))?;

        if rows_affected == 0 {
            return Ok(false);
        }

        // Update session artifact_count
        conn.execute(
            "UPDATE sessions
             SET artifact_count = CASE
                 WHEN artifact_count > 0 THEN artifact_count - 1
                 ELSE 0
             END
             WHERE tenant_id = ?1 AND session_id = ?2",
            vec![
                libsql::Value::Text(self.tenant_id.clone()),
                libsql::Value::Text(artifact_id.session_id.clone()),
            ],
        )
        .await
        .map_err(|e| {
            SqliteError::Query(format!("Failed to update session artifact count: {}", e))
        })?;

        // Drop connection before calling decrement_refcount to avoid nested connection acquisition
        drop(conn);

        // Only decrement refcount if we actually deleted an artifact
        self.decrement_refcount(&content_hash)
            .await
            .context("failed to decrement refcount")?;

        Ok(true)
    }

    async fn list_session_artifacts(&self, session_id: &str) -> anyhow::Result<Vec<ArtifactId>> {
        let conn = self.backend.get_connection().await?;

        let mut rows = conn
            .query(
                "SELECT artifact_id, session_id
                 FROM artifacts
                 WHERE tenant_id = ?1 AND session_id = ?2
                 ORDER BY stored_at DESC",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(session_id.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to list artifacts: {}", e)))?;

        let mut artifacts = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to iterate artifacts: {}", e)))?
        {
            let artifact_id_str = row.get::<String>(0).context("failed to get artifact_id")?;
            let session_id = row.get::<String>(1).context("failed to get session_id")?;

            // Store database artifact_id in ArtifactId.content_hash field
            artifacts.push(ArtifactId::new(artifact_id_str, session_id));
        }

        Ok(artifacts)
    }

    async fn get_storage_stats(&self, session_id: &str) -> anyhow::Result<SessionStorageStats> {
        let conn = self.backend.get_connection().await?;

        // Aggregate storage stats for session
        let mut rows = conn
            .query(
                "SELECT COUNT(*) as count, COALESCE(SUM(size_bytes), 0) as total_size
                 FROM artifacts
                 WHERE tenant_id = ?1 AND session_id = ?2",
                vec![
                    libsql::Value::Text(self.tenant_id.clone()),
                    libsql::Value::Text(session_id.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get storage stats: {}", e)))?;

        let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch stats row: {}", e)))?
        else {
            return Ok(SessionStorageStats::new());
        };

        let artifact_count = row.get::<i64>(0).context("failed to get count")? as usize;
        let total_size_bytes = row.get::<i64>(1).context("failed to get total_size")? as usize;

        Ok(SessionStorageStats {
            artifact_count,
            total_size_bytes,
            last_updated: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::SqliteConfig;
    use llmspell_core::types::storage::ArtifactType;

    async fn create_test_storage() -> (tempfile::TempDir, Arc<SqliteBackend>, SqliteArtifactStorage)
    {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Execute migrations
        let conn = backend.get_connection().await.unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .await
        .unwrap();
        conn.execute_batch(include_str!("../../../migrations/sqlite/V9__sessions.sql"))
            .await
            .unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V10__artifacts.sql"
        ))
        .await
        .unwrap();

        backend.set_tenant_context("test-tenant").await.unwrap();

        let storage = SqliteArtifactStorage::new(backend.clone(), "test-tenant".to_string());

        // Create test session
        conn.execute(
            "INSERT INTO sessions (tenant_id, session_id, session_data, status)
             VALUES ('test-tenant', 'test-session', '{}', 'active')",
            Vec::<libsql::Value>::new(),
        )
        .await
        .unwrap();

        (temp_dir, backend, storage)
    }

    #[tokio::test]
    async fn test_store_artifact() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let artifact = Artifact::new(
            "hash123".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            b"fn main() {}".to_vec(),
        );

        let id = storage.store_artifact(&artifact).await.unwrap();
        assert_eq!(id.session_id, "test-session");
    }

    #[tokio::test]
    async fn test_get_artifact() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let content = b"test content";
        let artifact = Artifact::new(
            "hash123".to_string(),
            "test-session".to_string(),
            ArtifactType::Document,
            content.to_vec(),
        );

        let id = storage.store_artifact(&artifact).await.unwrap();
        let loaded = storage.get_artifact(&id).await.unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.content, content);
        assert_eq!(loaded.artifact_type, ArtifactType::Document);
    }

    #[tokio::test]
    async fn test_delete_artifact() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let artifact = Artifact::new(
            "hash123".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            b"code".to_vec(),
        );

        let id = storage.store_artifact(&artifact).await.unwrap();
        let deleted = storage.delete_artifact(&id).await.unwrap();
        assert!(deleted);

        let loaded = storage.get_artifact(&id).await.unwrap();
        assert!(loaded.is_none());

        // Delete again should return false
        let deleted_again = storage.delete_artifact(&id).await.unwrap();
        assert!(!deleted_again);
    }

    #[tokio::test]
    async fn test_list_session_artifacts() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        // Store multiple artifacts
        let artifact1 = Artifact::new(
            "hash1".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            b"code1".to_vec(),
        );
        let artifact2 = Artifact::new(
            "hash2".to_string(),
            "test-session".to_string(),
            ArtifactType::Data,
            b"data2".to_vec(),
        );

        storage.store_artifact(&artifact1).await.unwrap();
        storage.store_artifact(&artifact2).await.unwrap();

        let artifacts = storage
            .list_session_artifacts("test-session")
            .await
            .unwrap();
        assert_eq!(artifacts.len(), 2);
    }

    #[tokio::test]
    async fn test_get_storage_stats() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let artifact = Artifact::new(
            "hash123".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            b"code content here".to_vec(),
        );

        storage.store_artifact(&artifact).await.unwrap();

        let stats = storage.get_storage_stats("test-session").await.unwrap();
        assert_eq!(stats.artifact_count, 1);
        assert_eq!(stats.total_size_bytes, b"code content here".len());
    }

    #[tokio::test]
    async fn test_deduplication() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let content = b"duplicate content";

        // Store same content twice in different artifacts
        let artifact1 = Artifact::new(
            "hash1".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            content.to_vec(),
        );
        let artifact2 = Artifact::new(
            "hash2".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            content.to_vec(),
        );

        storage.store_artifact(&artifact1).await.unwrap();
        storage.store_artifact(&artifact2).await.unwrap();

        // Both should exist
        let artifacts = storage
            .list_session_artifacts("test-session")
            .await
            .unwrap();
        assert_eq!(artifacts.len(), 2);

        // Content hash should be same (blake3 deterministic)
        let hash1 = SqliteArtifactStorage::compute_hash(content);
        let hash2 = SqliteArtifactStorage::compute_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_reference_counting() {
        let (_temp_dir, backend, storage) = create_test_storage().await;

        let content = b"shared content";

        // Store same content twice
        let artifact1 = Artifact::new(
            "hash1".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            content.to_vec(),
        );
        let artifact2 = Artifact::new(
            "hash2".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            content.to_vec(),
        );

        let id1 = storage.store_artifact(&artifact1).await.unwrap();
        let id2 = storage.store_artifact(&artifact2).await.unwrap();

        let content_hash = SqliteArtifactStorage::compute_hash(content);

        // Check reference count = 2
        {
            let conn = backend.get_connection().await.unwrap();
            let mut rows = conn
                .query(
                    "SELECT reference_count FROM artifact_content
                     WHERE tenant_id = 'test-tenant' AND content_hash = ?1",
                    vec![libsql::Value::Text(content_hash.clone())],
                )
                .await
                .unwrap();
            let refcount: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
            assert_eq!(refcount, 2);
        }

        // Delete one artifact
        storage.delete_artifact(&id1).await.unwrap();

        // Refcount should be 1
        {
            let conn = backend.get_connection().await.unwrap();
            let mut rows = conn
                .query(
                    "SELECT reference_count FROM artifact_content
                     WHERE tenant_id = 'test-tenant' AND content_hash = ?1",
                    vec![libsql::Value::Text(content_hash.clone())],
                )
                .await
                .unwrap();
            let refcount: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
            assert_eq!(refcount, 1);
        }

        // Delete second artifact
        storage.delete_artifact(&id2).await.unwrap();

        // Content should be deleted (refcount 0)
        {
            let conn = backend.get_connection().await.unwrap();
            let mut rows = conn
                .query(
                    "SELECT 1 FROM artifact_content
                     WHERE tenant_id = 'test-tenant' AND content_hash = ?1",
                    vec![libsql::Value::Text(content_hash)],
                )
                .await
                .unwrap();
            let content_exists = rows.next().await.unwrap();
            assert!(content_exists.is_none());
        }
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let (_temp_dir, backend, storage1) = create_test_storage().await;

        // Create second tenant
        backend.set_tenant_context("tenant-2").await.unwrap();
        let storage2 = SqliteArtifactStorage::new(backend.clone(), "tenant-2".to_string());

        let conn = backend.get_connection().await.unwrap();
        conn.execute(
            "INSERT INTO sessions (tenant_id, session_id, session_data, status)
             VALUES ('tenant-2', 'session-2', '{}', 'active')",
            Vec::<libsql::Value>::new(),
        )
        .await
        .unwrap();

        // Store artifacts for both tenants
        let artifact1 = Artifact::new(
            "hash1".to_string(),
            "test-session".to_string(),
            ArtifactType::Code,
            b"tenant1 data".to_vec(),
        );
        let artifact2 = Artifact::new(
            "hash2".to_string(),
            "session-2".to_string(),
            ArtifactType::Code,
            b"tenant2 data".to_vec(),
        );

        storage1.store_artifact(&artifact1).await.unwrap();
        storage2.store_artifact(&artifact2).await.unwrap();

        // Each tenant sees only their artifacts
        let artifacts1 = storage1
            .list_session_artifacts("test-session")
            .await
            .unwrap();
        assert_eq!(artifacts1.len(), 1);

        let artifacts2 = storage2.list_session_artifacts("session-2").await.unwrap();
        assert_eq!(artifacts2.len(), 1);

        // Tenant 1 can't see tenant 2's artifacts
        let artifacts_cross = storage1.list_session_artifacts("session-2").await.unwrap();
        assert_eq!(artifacts_cross.len(), 0);
    }

    #[tokio::test]
    async fn test_compute_hash() {
        let content1 = b"test content";
        let content2 = b"test content";
        let content3 = b"different content";

        let hash1 = SqliteArtifactStorage::compute_hash(content1);
        let hash2 = SqliteArtifactStorage::compute_hash(content2);
        let hash3 = SqliteArtifactStorage::compute_hash(content3);

        // Same content = same hash
        assert_eq!(hash1, hash2);

        // Different content = different hash
        assert_ne!(hash1, hash3);

        // blake3 produces 64 hex chars
        assert_eq!(hash1.len(), 64);
    }
}
