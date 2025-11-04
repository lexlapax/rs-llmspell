//! ABOUTME: PostgreSQL artifact backend (Phase 13b.10.3)
//! ABOUTME: Content-addressed artifact storage with automatic routing (BYTEA vs Large Objects)
//!
//! # Architecture
//!
//! - **Dual-table design**: artifact_content (storage) + artifacts (metadata)
//! - **Content deduplication**: Same content hash → single storage entry
//! - **Automatic routing**: BYTEA (<1MB) vs Large Objects (>=1MB)
//! - **Reference counting**: Automatic via database triggers
//!
//! # Integration
//!
//! Integrates with `llmspell-kernel::sessions::artifact::SessionArtifact`
//! for full artifact lifecycle management within sessions.

use super::error::{PostgresError, Result};
use super::large_objects::LargeObjectStream;
use super::PostgresBackend;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Storage type threshold (1MB)
const LARGE_OBJECT_THRESHOLD: usize = 1024 * 1024;

/// Storage type enum matching database constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// BYTEA storage for small artifacts (<1MB)
    Bytea,
    /// Large Object storage for large artifacts (>=1MB)
    LargeObject,
}

impl StorageType {
    /// Convert to database string value
    fn to_db_str(self) -> &'static str {
        match self {
            Self::Bytea => "bytea",
            Self::LargeObject => "large_object",
        }
    }

    /// Determine storage type based on content size
    fn from_size(size: usize) -> Self {
        if size < LARGE_OBJECT_THRESHOLD {
            Self::Bytea
        } else {
            Self::LargeObject
        }
    }
}

/// PostgreSQL artifact backend operations
impl PostgresBackend {
    /// Store artifact content (handles deduplication and automatic storage type selection)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `content_hash` - Blake3 content hash
    /// * `content` - Raw content bytes
    /// * `is_compressed` - Whether content is compressed
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Note
    /// Uses automatic deduplication - if content hash already exists, this is a no-op.
    /// Reference counting is handled automatically by database triggers.
    pub async fn store_artifact_content(
        &self,
        tenant_id: &str,
        content_hash: &str,
        content: &[u8],
        is_compressed: bool,
    ) -> Result<()> {
        let client = self.get_client().await?;

        // Determine storage type
        let storage_type = StorageType::from_size(content.len());
        let size_bytes = content.len() as i64;

        match storage_type {
            StorageType::Bytea => {
                // Store in BYTEA column
                client
                    .execute(
                        "INSERT INTO llmspell.artifact_content
                         (tenant_id, content_hash, storage_type, data, size_bytes, is_compressed)
                         VALUES ($1, $2, $3, $4, $5, $6)
                         ON CONFLICT (tenant_id, content_hash) DO NOTHING",
                        &[
                            &tenant_id,
                            &content_hash,
                            &storage_type.to_db_str(),
                            &content,
                            &size_bytes,
                            &is_compressed,
                        ],
                    )
                    .await
                    .map_err(|e| {
                        PostgresError::Query(format!("Failed to store BYTEA content: {}", e))
                    })?;
            }
            StorageType::LargeObject => {
                // Store as Large Object
                let mut lo_stream = LargeObjectStream::new(client);
                let oid = lo_stream.upload(content).await?;

                // Get client again after consuming it for Large Object
                let client = self.get_client().await?;
                client
                    .execute(
                        "INSERT INTO llmspell.artifact_content
                         (tenant_id, content_hash, storage_type, large_object_oid, size_bytes, is_compressed)
                         VALUES ($1, $2, $3, $4, $5, $6)
                         ON CONFLICT (tenant_id, content_hash) DO NOTHING",
                        &[
                            &tenant_id,
                            &content_hash,
                            &storage_type.to_db_str(),
                            &tokio_postgres::types::Oid::from(oid),
                            &size_bytes,
                            &is_compressed,
                        ],
                    )
                    .await
                    .map_err(|e| {
                        PostgresError::Query(format!("Failed to store Large Object metadata: {}", e))
                    })?;
            }
        }

        Ok(())
    }

    /// Retrieve artifact content by hash
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `content_hash` - Blake3 content hash
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - Content bytes
    pub async fn retrieve_artifact_content(
        &self,
        tenant_id: &str,
        content_hash: &str,
    ) -> Result<Vec<u8>> {
        let client = self.get_client().await?;

        // Get content metadata
        let row = client
            .query_one(
                "SELECT storage_type, data, large_object_oid
                 FROM llmspell.artifact_content
                 WHERE tenant_id = $1 AND content_hash = $2",
                &[&tenant_id, &content_hash],
            )
            .await
            .map_err(|e| PostgresError::Query(format!("Artifact content not found: {}", e)))?;

        let storage_type: String = row.get(0);

        match storage_type.as_str() {
            "bytea" => {
                let data: Vec<u8> = row.get(1);
                Ok(data)
            }
            "large_object" => {
                let oid: tokio_postgres::types::Oid = row.get(2);
                let mut lo_stream = LargeObjectStream::new(client);
                lo_stream.download(oid).await
            }
            _ => Err(PostgresError::Other(format!(
                "Unknown storage type: {}",
                storage_type
            ))),
        }
    }

    /// Store artifact metadata
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `artifact_id` - Full artifact ID (format: "{session_id}:{sequence}:{content_hash}")
    /// * `session_id` - Session UUID
    /// * `sequence` - Sequence number within session
    /// * `content_hash` - Blake3 content hash
    /// * `metadata` - Artifact metadata as JSON
    /// * `name` - Artifact name
    /// * `artifact_type` - Artifact type string
    /// * `mime_type` - MIME type
    /// * `size_bytes` - Content size in bytes
    /// * `created_at` - Creation timestamp
    /// * `created_by` - Optional creator identifier
    /// * `tags` - Optional tags
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    #[allow(clippy::too_many_arguments)]
    pub async fn store_artifact_metadata(
        &self,
        tenant_id: &str,
        artifact_id: &str,
        session_id: Uuid,
        sequence: i64,
        content_hash: &str,
        metadata: &JsonValue,
        name: &str,
        artifact_type: &str,
        mime_type: &str,
        size_bytes: i64,
        created_at: DateTime<Utc>,
        created_by: Option<&str>,
        tags: Option<Vec<String>>,
    ) -> Result<()> {
        let client = self.get_client().await?;

        client
            .execute(
                "INSERT INTO llmspell.artifacts
                 (tenant_id, artifact_id, session_id, sequence, content_hash, metadata,
                  name, artifact_type, mime_type, size_bytes, created_at, created_by, tags)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                &[
                    &tenant_id,
                    &artifact_id,
                    &session_id,
                    &sequence,
                    &content_hash,
                    metadata,
                    &name,
                    &artifact_type,
                    &mime_type,
                    &size_bytes,
                    &created_at,
                    &created_by,
                    &tags.as_ref(),
                ],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to store artifact metadata: {}", e))
            })?;

        Ok(())
    }

    /// Retrieve artifact metadata
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `artifact_id` - Full artifact ID
    ///
    /// # Returns
    /// * `Result<JsonValue>` - Metadata as JSON
    pub async fn retrieve_artifact_metadata(
        &self,
        tenant_id: &str,
        artifact_id: &str,
    ) -> Result<JsonValue> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                "SELECT metadata FROM llmspell.artifacts
                 WHERE tenant_id = $1 AND artifact_id = $2",
                &[&tenant_id, &artifact_id],
            )
            .await
            .map_err(|e| PostgresError::Query(format!("Artifact metadata not found: {}", e)))?;

        Ok(row.get(0))
    }

    /// List artifacts in a session
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `session_id` - Session UUID
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of artifact IDs
    pub async fn list_session_artifacts(
        &self,
        tenant_id: &str,
        session_id: Uuid,
    ) -> Result<Vec<String>> {
        let client = self.get_client().await?;

        let rows = client
            .query(
                "SELECT artifact_id FROM llmspell.artifacts
                 WHERE tenant_id = $1 AND session_id = $2
                 ORDER BY sequence ASC",
                &[&tenant_id, &session_id],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to list session artifacts: {}", e))
            })?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    /// Delete artifact (decrements reference count, may delete content if last reference)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `artifact_id` - Full artifact ID
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Note
    /// Content is automatically deleted when reference count reaches zero (via trigger).
    /// Large Objects are cleaned up via cascade delete.
    pub async fn delete_artifact(&self, tenant_id: &str, artifact_id: &str) -> Result<()> {
        let client = self.get_client().await?;

        // Get content hash and storage type before deletion
        let row = client
            .query_opt(
                "SELECT a.content_hash, ac.storage_type, ac.large_object_oid
                 FROM llmspell.artifacts a
                 JOIN llmspell.artifact_content ac
                   ON a.tenant_id = ac.tenant_id AND a.content_hash = ac.content_hash
                 WHERE a.tenant_id = $1 AND a.artifact_id = $2",
                &[&tenant_id, &artifact_id],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to get artifact for deletion: {}", e))
            })?;

        if let Some(row) = row {
            let content_hash: String = row.get(0);
            let storage_type: String = row.get(1);
            let large_object_oid: Option<tokio_postgres::types::Oid> = row.get(2);

            // Delete artifact metadata (trigger will decrement reference count)
            client
                .execute(
                    "DELETE FROM llmspell.artifacts
                     WHERE tenant_id = $1 AND artifact_id = $2",
                    &[&tenant_id, &artifact_id],
                )
                .await
                .map_err(|e| {
                    PostgresError::Query(format!("Failed to delete artifact metadata: {}", e))
                })?;

            // Check if content should be deleted (reference count = 0)
            let count_row = client
                .query_one(
                    "SELECT reference_count FROM llmspell.artifact_content
                     WHERE tenant_id = $1 AND content_hash = $2",
                    &[&tenant_id, &content_hash],
                )
                .await
                .map_err(|e| {
                    PostgresError::Query(format!("Failed to check reference count: {}", e))
                })?;

            let ref_count: i32 = count_row.get(0);

            // Delete content if ref_count == 1 (only initial reference, no artifacts)
            if ref_count == 1 {
                // Delete content
                if storage_type == "large_object" {
                    // Delete Large Object
                    if let Some(oid) = large_object_oid {
                        let mut lo_stream = LargeObjectStream::new(client);
                        let _ = lo_stream.delete(oid).await; // Ignore errors
                    }
                }

                // Delete content record (will cascade to Large Object if needed)
                let client = self.get_client().await?;
                client
                    .execute(
                        "DELETE FROM llmspell.artifact_content
                         WHERE tenant_id = $1 AND content_hash = $2",
                        &[&tenant_id, &content_hash],
                    )
                    .await
                    .map_err(|e| {
                        PostgresError::Query(format!("Failed to delete artifact content: {}", e))
                    })?;
            }
        }

        Ok(())
    }

    /// Get artifact statistics
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    ///
    /// # Returns
    /// * `Result<ArtifactStats>` - Statistics about artifacts
    pub async fn get_artifact_stats(&self, tenant_id: &str) -> Result<ArtifactStats> {
        let client = self.get_client().await?;

        let row = client
            .query_one(
                "SELECT
                     COUNT(*) as total_artifacts,
                     COUNT(DISTINCT content_hash) as unique_contents,
                     CAST(COALESCE(SUM(size_bytes), 0) AS BIGINT) as total_size,
                     CAST(COALESCE(AVG(size_bytes), 0) AS DOUBLE PRECISION) as avg_size
                 FROM llmspell.artifacts
                 WHERE tenant_id = $1",
                &[&tenant_id],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to get artifact statistics: {}", e))
            })?;

        let total_artifacts: i64 = row.get(0);
        let unique_contents: i64 = row.get(1);
        let total_size: i64 = row.get(2);
        let avg_size: f64 = row.get(3);

        // Get storage type breakdown
        let storage_row = client
            .query_one(
                "SELECT
                     COUNT(CASE WHEN storage_type = 'bytea' THEN 1 END) as bytea_count,
                     COUNT(CASE WHEN storage_type = 'large_object' THEN 1 END) as large_object_count
                 FROM llmspell.artifact_content
                 WHERE tenant_id = $1",
                &[&tenant_id],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to get storage type breakdown: {}", e))
            })?;

        let bytea_count: i64 = storage_row.get(0);
        let large_object_count: i64 = storage_row.get(1);

        Ok(ArtifactStats {
            total_artifacts: total_artifacts as usize,
            unique_contents: unique_contents as usize,
            total_size_bytes: total_size as usize,
            average_size_bytes: avg_size,
            bytea_count: bytea_count as usize,
            large_object_count: large_object_count as usize,
        })
    }
}

/// Artifact statistics
#[derive(Debug, Clone)]
pub struct ArtifactStats {
    /// Total number of artifacts
    pub total_artifacts: usize,
    /// Number of unique content hashes
    pub unique_contents: usize,
    /// Total size in bytes across all artifacts
    pub total_size_bytes: usize,
    /// Average artifact size in bytes
    pub average_size_bytes: f64,
    /// Number of artifacts stored in BYTEA
    pub bytea_count: usize,
    /// Number of artifacts stored as Large Objects
    pub large_object_count: usize,
}

// Tests are in separate test file: tests/postgres_artifacts_backend_tests.rs
