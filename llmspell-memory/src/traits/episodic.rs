//! Episodic memory trait (to be implemented in Task 13.1.2)

use async_trait::async_trait;

use crate::error::Result;
use crate::types::EpisodicEntry;

/// Episodic memory stores vector-indexed interaction history
#[async_trait]
pub trait EpisodicMemory: Send + Sync {
    /// Add a new episodic entry
    async fn add(&self, entry: EpisodicEntry) -> Result<()>;

    /// Search episodic memories by semantic similarity
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<EpisodicEntry>>;

    /// Get entries for a specific session
    async fn get_session(&self, session_id: &str) -> Result<Vec<EpisodicEntry>>;

    /// Get unprocessed entries (for consolidation)
    async fn get_unprocessed(&self, limit: usize) -> Result<Vec<EpisodicEntry>>;

    /// Mark entries as processed
    async fn mark_processed(&self, entry_ids: &[String]) -> Result<()>;

    /// Delete entries older than a certain date
    async fn delete_before(&self, timestamp: chrono::DateTime<chrono::Utc>) -> Result<usize>;
}
