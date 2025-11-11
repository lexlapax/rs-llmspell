//! Session persistence trait
//!
//! Provides trait abstraction for session storage with expiration tracking.

use crate::types::storage::SessionData;
use anyhow::Result;
use async_trait::async_trait;

/// Session persistence trait
///
/// Manages session lifecycle with expiration tracking and artifact references.
/// Supports session creation, retrieval, updates, and automatic cleanup of expired sessions.
///
/// # Lifecycle
///
/// Sessions transition from Active → Completed or Active → Expired.
/// Expired sessions should be cleaned up periodically via `cleanup_expired()`.
///
/// # Implementation Notes
///
/// - Session IDs must be unique
/// - Expired sessions should not be returned by `get_session()` unless explicitly requested
/// - `cleanup_expired()` should be called periodically to remove stale sessions
/// - Artifact count tracking is managed by the session (see `SessionData`)
///
/// # Examples
///
/// ```no_run
/// use llmspell_core::traits::storage::SessionStorage;
/// use llmspell_core::types::storage::{SessionData, SessionStatus};
/// # use anyhow::Result;
///
/// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
/// // Create new session
/// let mut data = SessionData::new("sess-123");
/// storage.create_session(&data.session_id, &data).await?;
///
/// // Retrieve session
/// let loaded = storage.get_session("sess-123").await?;
/// assert!(loaded.is_some());
///
/// // List active sessions
/// let active = storage.list_active_sessions().await?;
///
/// // Cleanup expired sessions
/// let cleaned = storage.cleanup_expired().await?;
/// println!("Cleaned up {} expired sessions", cleaned);
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// Create new session
    ///
    /// Stores a new session with the provided data. Session ID must be unique.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique session identifier
    /// * `data` - Complete session data to persist
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err` if session ID already exists or storage fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::SessionStorage;
    /// # use llmspell_core::types::storage::SessionData;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
    /// let data = SessionData::new("sess-123");
    /// storage.create_session(&data.session_id, &data).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn create_session(&self, session_id: &str, data: &SessionData) -> Result<()>;

    /// Retrieve session by ID
    ///
    /// Returns the session data if found and not expired.
    /// Returns `None` if session doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique session identifier
    ///
    /// # Returns
    ///
    /// - `Ok(Some(data))` if session found
    /// - `Ok(None)` if session not found
    /// - `Err` if storage operation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::SessionStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
    /// match storage.get_session("sess-123").await? {
    ///     Some(data) => println!("Session has {} artifacts", data.artifact_count),
    ///     None => println!("Session not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_session(&self, session_id: &str) -> Result<Option<SessionData>>;

    /// Update session data
    ///
    /// Updates the complete session data. Can be used to update session state,
    /// change status, or modify artifact count.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique session identifier
    /// * `data` - Updated session data
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err` if session not found or update fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::SessionStorage;
    /// # use llmspell_core::types::storage::SessionData;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
    /// let mut data = storage.get_session("sess-123").await?.unwrap();
    /// data.increment_artifacts();
    /// storage.update_session(&data.session_id, &data).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_session(&self, session_id: &str, data: &SessionData) -> Result<()>;

    /// Delete session
    ///
    /// Removes session from storage. Should also clean up associated artifacts
    /// if artifact_count > 0 (coordination with `ArtifactStorage`).
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique session identifier to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` on success (even if session not found), `Err` if delete fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::SessionStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
    /// storage.delete_session("sess-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_session(&self, session_id: &str) -> Result<()>;

    /// List active (non-expired) sessions
    ///
    /// Returns session IDs for all sessions with status Active.
    /// Expired and Completed sessions are excluded.
    ///
    /// # Returns
    ///
    /// Vector of session IDs for active sessions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::SessionStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
    /// let active = storage.list_active_sessions().await?;
    /// println!("Found {} active sessions", active.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_active_sessions(&self) -> Result<Vec<String>>;

    /// Cleanup expired sessions (batch delete)
    ///
    /// Finds all sessions where `expires_at` is in the past and deletes them.
    /// Should be called periodically (e.g., hourly) for cleanup.
    ///
    /// # Returns
    ///
    /// Number of sessions deleted
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_core::traits::storage::SessionStorage;
    /// # use anyhow::Result;
    /// # async fn example(storage: &dyn SessionStorage) -> Result<()> {
    /// let cleaned = storage.cleanup_expired().await?;
    /// if cleaned > 0 {
    ///     println!("Cleaned up {} expired sessions", cleaned);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn cleanup_expired(&self) -> Result<usize>;
}
