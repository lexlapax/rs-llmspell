//! Episodic memory trait
//!
//! Episodic memory stores vector-indexed interaction history with semantic search.
//! Each entry represents a single interaction (user message, assistant response, etc.)
//! with automatic vector embedding for similarity search.
//!
//! # Features
//!
//! - **Vector Search**: Semantic similarity via HNSW/ChromaDB/Qdrant
//! - **Session Isolation**: Filter memories by session ID
//! - **Bi-temporal Tracking**: `event_time` (when it happened) + `ingestion_time` (when we learned it)
//! - **Consolidation Support**: Mark entries as processed after knowledge extraction

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::types::EpisodicEntry;

/// Episodic memory stores vector-indexed interaction history
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory::prelude::*;
///
/// let episodic = HnswEpisodicMemory::new_in_memory().await?;
///
/// // Add an interaction
/// let id = episodic.add(EpisodicEntry::new(
///     "session-1".into(),
///     "user".into(),
///     "What is Rust?".into(),
/// )).await?;
///
/// // Search by semantic similarity
/// let results = episodic.search("programming languages", 5).await?;
///
/// // Get all session interactions
/// let session = episodic.get_session("session-1").await?;
/// ```
#[async_trait]
pub trait EpisodicMemory: Send + Sync {
    /// Add a new episodic entry and return its ID
    ///
    /// The entry will be embedded (if embedding is None) and stored
    /// with bi-temporal tracking.
    ///
    /// # Arguments
    ///
    /// * `entry` - The episodic entry to add
    ///
    /// # Returns
    ///
    /// The unique ID of the stored entry
    async fn add(&self, entry: EpisodicEntry) -> Result<String>;

    /// Get a specific episodic entry by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The entry ID to retrieve
    ///
    /// # Returns
    ///
    /// The episodic entry if found
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::NotFound` if the entry doesn't exist
    async fn get(&self, id: &str) -> Result<EpisodicEntry>;

    /// Search episodic memories by semantic similarity
    ///
    /// Performs vector similarity search using the query text.
    /// Results are ranked by cosine similarity.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query text
    /// * `top_k` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Top-K most similar entries, ordered by relevance (descending)
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<EpisodicEntry>>;

    /// Get all unprocessed entries for a session (for consolidation)
    ///
    /// Returns episodic entries that have not yet been consolidated
    /// into the semantic knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to filter by
    ///
    /// # Returns
    ///
    /// All unprocessed entries for the session
    async fn list_unprocessed(&self, session_id: &str) -> Result<Vec<EpisodicEntry>>;

    /// Get all entries for a specific session
    ///
    /// Returns all episodic entries (processed and unprocessed) for the session,
    /// ordered by timestamp.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to retrieve
    ///
    /// # Returns
    ///
    /// All entries for the session, chronologically ordered
    async fn get_session(&self, session_id: &str) -> Result<Vec<EpisodicEntry>>;

    /// Mark entries as processed after consolidation
    ///
    /// Updates the `processed` flag for the given entry IDs.
    ///
    /// # Arguments
    ///
    /// * `entry_ids` - Slice of entry IDs to mark as processed
    async fn mark_processed(&self, entry_ids: &[String]) -> Result<()>;

    /// Delete entries older than a certain date
    ///
    /// Removes episodic entries with `event_time` before the given timestamp.
    /// Useful for implementing retention policies.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Delete entries before this time
    ///
    /// # Returns
    ///
    /// Number of entries deleted
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>;
}
