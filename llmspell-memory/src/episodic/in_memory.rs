//! In-memory episodic memory implementation
//!
//! This module provides a simple, thread-safe in-memory implementation
//! of episodic memory using `HashMap` and cosine similarity for vector search.
//!
//! Perfect for:
//! - Testing and development
//! - Small-scale deployments (<10k entries)
//! - Prototyping without external dependencies

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

use crate::error::{MemoryError, Result};
use crate::traits::EpisodicMemory;
use crate::types::EpisodicEntry;

/// In-memory episodic memory storage
///
/// Thread-safe implementation using `Arc<RwLock<HashMap>>` for storage
/// and cosine similarity for vector search.
///
/// # Example
///
/// ```rust
/// use llmspell_memory::episodic::InMemoryEpisodicMemory;
/// use llmspell_memory::prelude::*;
///
/// # async fn example() -> Result<()> {
/// let memory = InMemoryEpisodicMemory::new();
///
/// // Add an entry
/// let entry = EpisodicEntry::new(
///     "session-1".into(),
///     "user".into(),
///     "What is Rust?".into(),
/// );
/// let id = memory.add(entry).await?;
///
/// // Retrieve it
/// let retrieved = memory.get(&id).await?;
/// assert_eq!(retrieved.content, "What is Rust?");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct InMemoryEpisodicMemory {
    /// Entry storage indexed by ID
    entries: Arc<RwLock<HashMap<String, EpisodicEntry>>>,
}

impl InMemoryEpisodicMemory {
    /// Create a new in-memory episodic memory instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    /// Simple text-to-embedding function for testing
    /// In production, use a real embedding model
    #[allow(clippy::cast_precision_loss)] // Test function, precision loss acceptable
    fn text_to_embedding(text: &str) -> Vec<f32> {
        // Simple character-based embedding for testing
        // Each character contributes to a 128-dim vector
        let mut embedding = vec![0.0f32; 128];

        for (i, ch) in text.chars().take(128).enumerate() {
            embedding[i] = (ch as u32 as f32) / 1000.0;
        }

        // Add some text statistics to make it more semantic
        let word_count = text.split_whitespace().count() as f32;
        let char_count = text.chars().count() as f32;

        if embedding.len() > 1 {
            embedding[0] += word_count / 100.0;
            embedding[1] += char_count / 1000.0;
        }

        embedding
    }
}

impl Default for InMemoryEpisodicMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EpisodicMemory for InMemoryEpisodicMemory {
    async fn add(&self, mut entry: EpisodicEntry) -> Result<String> {
        // Generate embedding if not provided
        if entry.embedding.is_none() {
            entry.embedding = Some(Self::text_to_embedding(&entry.content));
        }

        let id = entry.id.clone();
        self.entries.write().insert(id.clone(), entry);

        Ok(id)
    }

    async fn get(&self, id: &str) -> Result<EpisodicEntry> {
        self.entries
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| MemoryError::NotFound(format!("Entry not found: {id}")))
    }

    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<EpisodicEntry>> {
        let query_embedding = Self::text_to_embedding(query);

        let mut results: Vec<(f32, EpisodicEntry)> = {
            let entries = self.entries.read();
            entries
                .values()
                .filter_map(|entry| {
                    entry.embedding.as_ref().map(|emb| {
                        let similarity = Self::cosine_similarity(&query_embedding, emb);
                        (similarity, entry.clone())
                    })
                })
                .collect()
        }; // Lock dropped here

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k results
        Ok(results
            .into_iter()
            .take(top_k)
            .map(|(_, entry)| entry)
            .collect())
    }

    async fn list_unprocessed(&self, session_id: &str) -> Result<Vec<EpisodicEntry>> {
        let entries = self.entries.read();

        Ok(entries
            .values()
            .filter(|entry| entry.session_id == session_id && !entry.processed)
            .cloned()
            .collect())
    }

    async fn get_session(&self, session_id: &str) -> Result<Vec<EpisodicEntry>> {
        let mut session_entries: Vec<EpisodicEntry> = {
            let entries = self.entries.read();
            entries
                .values()
                .filter(|entry| entry.session_id == session_id)
                .cloned()
                .collect()
        }; // Lock dropped here

        // Sort by timestamp (chronological order)
        session_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(session_entries)
    }

    async fn mark_processed(&self, entry_ids: &[String]) -> Result<()> {
        let mut entries = self.entries.write();

        for id in entry_ids {
            if let Some(entry) = entries.get_mut(id) {
                entry.mark_processed();
            }
        }

        Ok(())
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize> {
        let to_delete: Vec<String> = {
            let entries = self.entries.read();
            entries
                .iter()
                .filter(|(_, entry)| entry.timestamp < timestamp)
                .map(|(id, _)| id.clone())
                .collect()
        }; // Read lock dropped here

        let count = to_delete.len();

        {
            let mut entries = self.entries.write();
            for id in to_delete {
                entries.remove(&id);
            }
        } // Write lock dropped here

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get() {
        let memory = InMemoryEpisodicMemory::new();
        let entry = EpisodicEntry::new("session-1".into(), "user".into(), "test content".into());

        let id = memory.add(entry.clone()).await.expect("add failed");
        let retrieved = memory.get(&id).await.expect("get failed");

        assert_eq!(retrieved.content, "test content");
        assert_eq!(retrieved.session_id, "session-1");
    }

    #[tokio::test]
    async fn test_search() {
        let memory = InMemoryEpisodicMemory::new();

        // Add some entries
        memory
            .add(EpisodicEntry::new(
                "session-1".into(),
                "user".into(),
                "What is Rust programming language?".into(),
            ))
            .await
            .unwrap();

        memory
            .add(EpisodicEntry::new(
                "session-1".into(),
                "assistant".into(),
                "Rust is a systems programming language".into(),
            ))
            .await
            .unwrap();

        memory
            .add(EpisodicEntry::new(
                "session-1".into(),
                "user".into(),
                "What is Python?".into(),
            ))
            .await
            .unwrap();

        // Search for Rust-related content
        let results = memory.search("Rust programming", 2).await.unwrap();

        assert!(!results.is_empty());
        // Results should be ordered by relevance
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_session_isolation() {
        let memory = InMemoryEpisodicMemory::new();

        memory
            .add(EpisodicEntry::new(
                "session-1".into(),
                "user".into(),
                "session 1 content".into(),
            ))
            .await
            .unwrap();

        memory
            .add(EpisodicEntry::new(
                "session-2".into(),
                "user".into(),
                "session 2 content".into(),
            ))
            .await
            .unwrap();

        let session1 = memory.get_session("session-1").await.unwrap();
        let session2 = memory.get_session("session-2").await.unwrap();

        assert_eq!(session1.len(), 1);
        assert_eq!(session2.len(), 1);
        assert_eq!(session1[0].content, "session 1 content");
        assert_eq!(session2[0].content, "session 2 content");
    }

    #[tokio::test]
    async fn test_mark_processed() {
        let memory = InMemoryEpisodicMemory::new();

        let entry = EpisodicEntry::new("session-1".into(), "user".into(), "test".into());
        let id = memory.add(entry).await.unwrap();

        // Should be unprocessed initially
        let unprocessed = memory.list_unprocessed("session-1").await.unwrap();
        assert_eq!(unprocessed.len(), 1);

        // Mark as processed
        memory
            .mark_processed(std::slice::from_ref(&id))
            .await
            .unwrap();

        // Should now be empty
        let unprocessed = memory.list_unprocessed("session-1").await.unwrap();
        assert_eq!(unprocessed.len(), 0);

        // But still retrievable
        let retrieved = memory.get(&id).await.unwrap();
        assert!(retrieved.processed);
    }

    #[tokio::test]
    async fn test_delete_before() {
        let memory = InMemoryEpisodicMemory::new();

        let mut old_entry = EpisodicEntry::new("session-1".into(), "user".into(), "old".into());
        old_entry.timestamp = Utc::now() - chrono::Duration::days(30);
        memory.add(old_entry).await.unwrap();

        let new_entry = EpisodicEntry::new("session-1".into(), "user".into(), "new".into());
        memory.add(new_entry).await.unwrap();

        // Delete entries older than 7 days
        let cutoff = Utc::now() - chrono::Duration::days(7);
        let deleted = memory.delete_before(cutoff).await.unwrap();

        assert_eq!(deleted, 1);

        let remaining = memory.get_session("session-1").await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "new");
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![1.0, 0.0, 0.0];
        let vec_c = vec![0.0, 1.0, 0.0];

        assert!((InMemoryEpisodicMemory::cosine_similarity(&vec_a, &vec_b) - 1.0).abs() < 0.001);
        assert!((InMemoryEpisodicMemory::cosine_similarity(&vec_a, &vec_c)).abs() < 0.001);
    }
}
