//! Artifact storage trait
//!
//! Provides trait abstraction for content-addressed artifact storage with deduplication.

use crate::types::storage::{Artifact, ArtifactId, SessionStorageStats};
use anyhow::Result;
use async_trait::async_trait;

/// Artifact storage trait
///
/// Manages persistent storage for artifacts with content-addressed deduplication.
/// Implementations should handle content hashing, deduplication, and quota tracking.
///
/// # Content Addressing
///
/// Artifacts are identified by their content hash (SHA-256), enabling automatic
/// deduplication of identical content across sessions. The `ArtifactId` combines
/// content hash with session ownership for access control.
///
/// # Implementation Notes
///
/// - Content hashes must be SHA-256 hex strings
/// - Implementations should verify hash integrity on retrieval
/// - Storage stats should be updated atomically with artifact operations
/// - Deduplication should be transparent to callers
/// - Session cleanup should cascade to artifacts
///
/// # Examples
///
/// ```no_run
/// use llmspell_core::traits::storage::ArtifactStorage;
/// use llmspell_core::types::storage::{Artifact, ArtifactType};
/// # use anyhow::Result;
///
/// # async fn example(storage: &dyn ArtifactStorage) -> Result<()> {
/// // Store new artifact
/// let artifact = Artifact::new(
///     "abc123hash".to_string(),
///     "session-1".to_string(),
///     ArtifactType::Code,
///     b"fn main() {}".to_vec(),
/// );
/// let id = storage.store_artifact(&artifact).await?;
///
/// // Retrieve artifact
/// let loaded = storage.get_artifact(&id).await?;
/// assert!(loaded.is_some());
///
/// // List session artifacts
/// let artifacts = storage.list_session_artifacts("session-1").await?;
///
/// // Get storage stats
/// let stats = storage.get_storage_stats("session-1").await?;
/// println!("Session has {} artifacts using {} bytes",
///     stats.artifact_count, stats.total_size_bytes);
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ArtifactStorage: Send + Sync {
    /// Store artifact with content-addressed deduplication
    ///
    /// Stores artifact content and metadata. If content with the same hash
    /// already exists, only metadata is stored (deduplication). Updates
    /// session storage statistics.
    ///
    /// # Arguments
    ///
    /// * `artifact` - Complete artifact data to persist
    ///
    /// # Returns
    ///
    /// The artifact ID on success
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Storage operation fails
    /// - Content hash verification fails
    /// - Session quota exceeded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::ArtifactStorage;
    /// # use llmspell_core::types::storage::{Artifact, ArtifactType};
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn ArtifactStorage) -> Result<()> {
    /// let artifact = Artifact::new(
    ///     "hash123".to_string(),
    ///     "session-1".to_string(),
    ///     ArtifactType::Document,
    ///     b"content".to_vec(),
    /// );
    /// let id = storage.store_artifact(&artifact).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn store_artifact(&self, artifact: &Artifact) -> Result<ArtifactId>;

    /// Retrieve artifact by ID
    ///
    /// Returns complete artifact including content and metadata.
    /// Returns `None` if artifact not found.
    ///
    /// # Arguments
    ///
    /// * `artifact_id` - Unique artifact identifier
    ///
    /// # Returns
    ///
    /// - `Ok(Some(artifact))` if found
    /// - `Ok(None)` if not found
    /// - `Err` if storage operation fails or hash verification fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::ArtifactStorage;
    /// # use llmspell_core::types::storage::ArtifactId;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn ArtifactStorage, id: &ArtifactId) -> Result<()> {
    /// match storage.get_artifact(id).await? {
    ///     Some(artifact) => println!("Found artifact: {} bytes", artifact.size_bytes),
    ///     None => println!("Artifact not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_artifact(&self, artifact_id: &ArtifactId) -> Result<Option<Artifact>>;

    /// Delete artifact
    ///
    /// Removes artifact metadata and content (if not referenced by other artifacts).
    /// Updates session storage statistics. Content deduplication means the same
    /// content may be referenced by multiple artifacts - deletion only removes
    /// this specific artifact's metadata.
    ///
    /// # Arguments
    ///
    /// * `artifact_id` - Unique artifact identifier to delete
    ///
    /// # Returns
    ///
    /// `true` if artifact was deleted, `false` if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::ArtifactStorage;
    /// # use llmspell_core::types::storage::ArtifactId;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn ArtifactStorage, id: &ArtifactId) -> Result<()> {
    /// let deleted = storage.delete_artifact(id).await?;
    /// if deleted {
    ///     println!("Artifact deleted");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_artifact(&self, artifact_id: &ArtifactId) -> Result<bool>;

    /// List all artifacts for a session
    ///
    /// Returns artifact IDs for all artifacts belonging to the specified session.
    /// Useful for session cleanup and artifact browsing.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    ///
    /// # Returns
    ///
    /// Vector of artifact IDs (may be empty)
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::ArtifactStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn ArtifactStorage) -> Result<()> {
    /// let artifacts = storage.list_session_artifacts("session-1").await?;
    /// println!("Found {} artifacts", artifacts.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_session_artifacts(&self, session_id: &str) -> Result<Vec<ArtifactId>>;

    /// Get storage statistics for a session
    ///
    /// Returns aggregate storage metrics including total size and artifact count.
    /// Used for quota management and monitoring.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    ///
    /// # Returns
    ///
    /// Storage statistics (returns default/empty stats if session not found)
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::ArtifactStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn ArtifactStorage) -> Result<()> {
    /// let stats = storage.get_storage_stats("session-1").await?;
    /// println!("Total storage: {} bytes in {} artifacts",
    ///     stats.total_size_bytes, stats.artifact_count);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_storage_stats(&self, session_id: &str) -> Result<SessionStorageStats>;
}
