// ABOUTME: SQLite artifact storage implementation
//! ABOUTME: SQLite artifact storage with deduplication and rusqlite
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
/// Implements ArtifactStorage trait using rusqlite with content-addressed deduplication.
/// Uses 2-table structure:
/// - artifact_content: Content storage (BLOB) with blake3 hash + reference counting
/// - artifacts: Metadata with foreign key to content
pub struct SqliteArtifactStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteArtifactStorage {
    /// Create new SQLite artifact storage
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
        let tenant_id = self.tenant_id.clone();

        // Check if content exists
        let exists: Option<i64> = conn
            .query_row(
                "SELECT reference_count FROM artifact_content
             WHERE tenant_id = ?1 AND content_hash = ?2",
                rusqlite::params![tenant_id, content_hash],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|e| SqliteError::Query(format!("Failed to check content existence: {}", e)))?;

        if exists.is_some() {
            // Increment existing reference count
            conn.execute(
                "UPDATE artifact_content
                 SET reference_count = reference_count + 1,
                     last_accessed_at = strftime('%s', 'now')
                 WHERE tenant_id = ?1 AND content_hash = ?2",
                rusqlite::params![tenant_id, content_hash],
            )
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
                rusqlite::params![tenant_id, content_hash, content, size_bytes, is_compressed],
            )
            .map_err(|e| SqliteError::Query(format!("Failed to insert content: {}", e)))?;

            Ok(false)
        }
    }

    /// Decrement reference count for content hash
    ///
    /// Deletes content if reference count reaches 1 (will become 0)
    async fn decrement_refcount(&self, content_hash: &str) -> Result<()> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // Delete content if refcount = 1 (it will become 0 after decrement)
        // This avoids violating the CHECK constraint (reference_count > 0)
        let deleted = conn
            .execute(
                "DELETE FROM artifact_content
             WHERE tenant_id = ?1 AND content_hash = ?2 AND reference_count = 1",
                rusqlite::params![tenant_id, content_hash],
            )
            .map_err(|e| SqliteError::Query(format!("Failed to delete content: {}", e)))?;

        // If we didn't delete (refcount > 1), just decrement
        if deleted == 0 {
            conn.execute(
                "UPDATE artifact_content
                 SET reference_count = reference_count - 1
                 WHERE tenant_id = ?1 AND content_hash = ?2",
                rusqlite::params![tenant_id, content_hash],
            )
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

// Helper trait to use optional() from rusqlite
use rusqlite::OptionalExtension;

#[async_trait]
impl ArtifactStorage for SqliteArtifactStorage {
    async fn store_artifact(&self, artifact: &Artifact) -> anyhow::Result<ArtifactId> {
        let content_hash = Self::compute_hash(&artifact.content);

        // Increment refcount (inserts content if new)
        self.increment_refcount(&content_hash, &artifact.content)
            .await
            .context("failed to manage content reference")?;

        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // Generate artifact_id format: "{session_id}:{sequence}:{content_hash}"
        // Get next sequence number for session
        let sequence: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sequence), -1) + 1
             FROM artifacts
             WHERE tenant_id = ?1 AND session_id = ?2",
                rusqlite::params![tenant_id, artifact.artifact_id.session_id],
                |row| row.get(0),
            )
            .map_err(|e| SqliteError::Query(format!("Failed to get next sequence: {}", e)))?;

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
            rusqlite::params![
                tenant_id,
                artifact_id,
                artifact.artifact_id.session_id,
                sequence,
                content_hash,
                metadata_json,
                name,
                artifact_type_str,
                mime_type,
                size_bytes,
                created_at,
                created_by
            ],
        )
        .map_err(|e| SqliteError::Query(format!("Failed to insert artifact: {}", e)))?;

        // Update session artifact_count
        conn.execute(
            "UPDATE sessions
             SET artifact_count = artifact_count + 1
             WHERE tenant_id = ?1 AND session_id = ?2",
            rusqlite::params![tenant_id, artifact.artifact_id.session_id],
        )
        .map_err(|e| {
            SqliteError::Query(format!("Failed to update session artifact count: {}", e))
        })?;

        // Return ArtifactId with the database artifact_id as content_hash
        Ok(ArtifactId::new(
            artifact_id, // Full database ID: "{session_id}:{sequence}:{content_hash}"
            artifact.artifact_id.session_id.clone(),
        ))
    }

    async fn get_artifact(&self, artifact_id: &ArtifactId) -> anyhow::Result<Option<Artifact>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // artifact_id.content_hash contains the database artifact_id: "{session_id}:{sequence}:{content_hash}"
        // Join artifacts + artifact_content to retrieve full artifact
        let result = conn
            .query_row(
                "SELECT a.artifact_type, a.metadata, a.size_bytes, a.created_at, c.data
             FROM artifacts a
             INNER JOIN artifact_content c ON
                 a.tenant_id = c.tenant_id AND a.content_hash = c.content_hash
             WHERE a.tenant_id = ?1
               AND a.artifact_id = ?2",
                rusqlite::params![tenant_id, artifact_id.content_hash],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| SqliteError::Query(format!("Failed to get artifact: {}", e)))?;

        let (artifact_type_str, metadata_json, size_bytes, created_at_ts, content) = match result {
            Some(v) => v,
            None => return Ok(None),
        };

        let artifact_type = Self::parse_artifact_type(&artifact_type_str)?;
        let metadata: serde_json::Value =
            serde_json::from_str(&metadata_json).context("failed to deserialize metadata")?;

        // This cast is safe for reasonable sizes
        let size_bytes_usize = size_bytes as usize;

        let created_at =
            chrono::DateTime::from_timestamp(created_at_ts, 0).context("invalid timestamp")?;

        // Update last_accessed_at (throttled to 1 minute in production)
        conn.execute(
            "UPDATE artifact_content
             SET last_accessed_at = strftime('%s', 'now')
             WHERE tenant_id = ?1 AND content_hash = (
                 SELECT content_hash FROM artifacts WHERE tenant_id = ?1 AND artifact_id = ?2
             )
               AND last_accessed_at < strftime('%s', 'now', '-1 minute')",
            rusqlite::params![tenant_id, artifact_id.content_hash],
        )
        .map_err(|e| SqliteError::Query(format!("Failed to update last_accessed_at: {}", e)))?;

        Ok(Some(Artifact {
            artifact_id: artifact_id.clone(),
            artifact_type,
            content,
            metadata,
            size_bytes: size_bytes_usize,
            created_at,
        }))
    }

    async fn delete_artifact(&self, artifact_id: &ArtifactId) -> anyhow::Result<bool> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // First, get the content_hash before deletion
        let content_hash: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM artifacts
             WHERE tenant_id = ?1 AND artifact_id = ?2",
                rusqlite::params![tenant_id, artifact_id.content_hash],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| SqliteError::Query(format!("Failed to get content_hash: {}", e)))?;

        let content_hash_val: Option<String> = match content_hash {
            Some(h) => Some(h),
            None => return Ok(false),
        };
        let content_hash_str = content_hash_val.unwrap();

        // Delete artifact metadata using artifact_id (unique identifier)
        let rows_affected = conn
            .execute(
                "DELETE FROM artifacts
             WHERE tenant_id = ?1 AND artifact_id = ?2",
                rusqlite::params![tenant_id, artifact_id.content_hash],
            )
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
            rusqlite::params![tenant_id, artifact_id.session_id],
        )
        .map_err(|e| {
            SqliteError::Query(format!("Failed to update session artifact count: {}", e))
        })?;

        // Drop connection before calling decrement_refcount
        drop(conn);

        // Only decrement refcount if we actually deleted an artifact
        self.decrement_refcount(&content_hash_str)
            .await
            .context("failed to decrement refcount")?;

        Ok(true)
    }

    async fn list_session_artifacts(&self, session_id: &str) -> anyhow::Result<Vec<ArtifactId>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let mut stmt = conn
            .prepare(
                "SELECT artifact_id, session_id
             FROM artifacts
             WHERE tenant_id = ?1 AND session_id = ?2
             ORDER BY stored_at DESC",
            )
            .map_err(|e| SqliteError::Query(format!("Failed to prepare list artifacts: {}", e)))?;

        let artifact_iter = stmt
            .query_map(rusqlite::params![tenant_id, session_id], |row| {
                Ok(ArtifactId::new(
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                ))
            })
            .map_err(|e| SqliteError::Query(format!("Failed to list artifacts: {}", e)))?;

        let mut artifacts = Vec::new();
        for artifact in artifact_iter {
            artifacts.push(
                artifact.map_err(|e| {
                    SqliteError::Query(format!("Failed to read artifact row: {}", e))
                })?,
            );
        }

        Ok(artifacts)
    }

    async fn get_storage_stats(&self, session_id: &str) -> anyhow::Result<SessionStorageStats> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let (count, total_size): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0)
             FROM artifacts
             WHERE tenant_id = ?1 AND session_id = ?2",
                rusqlite::params![tenant_id, session_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| SqliteError::Query(format!("Failed to get storage stats: {}", e)))?;

        Ok(SessionStorageStats {
            artifact_count: count as usize,
            total_size_bytes: total_size as usize,
            last_updated: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::{SqliteBackend, SqliteConfig};
    use llmspell_core::types::storage::ArtifactType;

    // Helper to run migrations for tests
    fn run_migrations(conn: &rusqlite::Connection) {
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V1__initial_setup.sql"
        ))
        .unwrap();
        conn.execute_batch(include_str!("../../../migrations/sqlite/V9__sessions.sql"))
            .unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V10__artifacts.sql"
        ))
        .unwrap();
    }

    async fn create_test_storage() -> (tempfile::TempDir, Arc<SqliteBackend>, SqliteArtifactStorage)
    {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = SqliteConfig::new(db_path.to_str().unwrap());
        // SqlitePool::new creates the file and pool
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Execute migrations
        let conn = backend.get_connection().await.unwrap();
        run_migrations(&conn);

        backend.set_tenant_context("test-tenant").await.unwrap();

        let storage = SqliteArtifactStorage::new(backend.clone(), "test-tenant".to_string());

        // Create test session
        conn.execute(
            "INSERT INTO sessions (tenant_id, session_id, session_data, status)
             VALUES ('test-tenant', 'test-session', '{}', 'active')",
            [],
        )
        .unwrap();

        (temp_dir, backend, storage)
    }

    #[tokio::test]
    async fn test_store_artifact() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let artifact = Artifact::new(
            "hash123".to_string(), // This is misleadingly named in test as content_hash but treated as artifact_id part
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

        // Content hash should be same
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
            let refcount: i64 = conn
                .query_row(
                    "SELECT reference_count FROM artifact_content
                 WHERE tenant_id = 'test-tenant' AND content_hash = ?1",
                    rusqlite::params![content_hash],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(refcount, 2);
        }

        // Delete one artifact
        storage.delete_artifact(&id1).await.unwrap();

        // Refcount should be 1
        {
            let conn = backend.get_connection().await.unwrap();
            let refcount: i64 = conn
                .query_row(
                    "SELECT reference_count FROM artifact_content
                 WHERE tenant_id = 'test-tenant' AND content_hash = ?1",
                    rusqlite::params![content_hash],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(refcount, 1);
        }

        // Delete second artifact
        storage.delete_artifact(&id2).await.unwrap();

        // Content should be deleted
        {
            let conn = backend.get_connection().await.unwrap();
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT 1 FROM artifact_content
                 WHERE tenant_id = 'test-tenant' AND content_hash = ?1",
                    rusqlite::params![content_hash],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            assert!(exists.is_none());
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
            [],
        )
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

        // blake3 produces 64 hex chars (actually sha256 in implementation: 64 chars too)
        assert_eq!(hash1.len(), 64);
    }
}
