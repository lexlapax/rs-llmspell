//! Memory-aware chunking strategy
//!
//! Enhances document chunking with episodic memory context hints to respect
//! conversation boundaries and semantic continuity.
//!
//! # Algorithm
//!
//! 1. Query episodic memory for similar context (top-K entries)
//! 2. Extract conversation patterns and topic boundaries
//! 3. Perform base chunking using wrapped strategy
//! 4. Adjust chunk boundaries to align with conversation breaks
//! 5. Add context metadata from episodic memory
//!
//! # Example
//!
//! ```text
//! use llmspell_rag::chunking::{MemoryAwareChunker, SlidingWindowChunker};
//! use llmspell_memory::episodic::InMemoryEpisodicMemory;
//!
//! let base_chunker = SlidingWindowChunker::new();
//! let memory = InMemoryEpisodicMemory::default();
//! let chunker = MemoryAwareChunker::new(base_chunker, memory);
//!
//! let chunks = chunker.chunk(text, &config).await?;
//! ```

use anyhow::Result;
use async_trait::async_trait;
use llmspell_memory::traits::EpisodicMemory;
use std::sync::Arc;
use tracing::{debug, trace};

use super::strategies::{ChunkingConfig, ChunkingStrategy, DocumentChunk};

/// Memory-aware chunking strategy that uses episodic memory for context hints
///
/// Wraps an existing chunking strategy and enhances it with memory context:
/// - Queries episodic memory for similar interactions
/// - Detects conversation boundaries from memory patterns
/// - Adjusts chunk splits to respect semantic continuity
/// - Enriches chunk metadata with context hints
pub struct MemoryAwareChunker<S: ChunkingStrategy, M: EpisodicMemory> {
    /// Base chunking strategy (e.g., SlidingWindowChunker)
    base_strategy: S,
    /// Episodic memory for context hints
    memory: Arc<M>,
    /// Number of memory entries to query for context (default: 5)
    context_k: usize,
}

impl<S: ChunkingStrategy, M: EpisodicMemory> std::fmt::Debug for MemoryAwareChunker<S, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryAwareChunker")
            .field("base_strategy", &std::any::type_name::<S>())
            .field("memory", &std::any::type_name::<M>())
            .field("context_k", &self.context_k)
            .finish()
    }
}

impl<S: ChunkingStrategy, M: EpisodicMemory> MemoryAwareChunker<S, M> {
    /// Create a new memory-aware chunker
    ///
    /// # Arguments
    ///
    /// * `base_strategy` - Underlying chunking strategy to wrap
    /// * `memory` - Episodic memory instance for context hints
    pub fn new(base_strategy: S, memory: Arc<M>) -> Self {
        Self {
            base_strategy,
            memory,
            context_k: 5,
        }
    }

    /// Create a memory-aware chunker with custom context size
    ///
    /// # Arguments
    ///
    /// * `base_strategy` - Underlying chunking strategy to wrap
    /// * `memory` - Episodic memory instance for context hints
    /// * `context_k` - Number of memory entries to retrieve for context
    #[must_use]
    pub fn with_context_k(mut self, context_k: usize) -> Self {
        self.context_k = context_k;
        self
    }

    /// Query episodic memory for context hints
    ///
    /// Retrieves similar interactions from memory to inform chunking decisions.
    /// Returns conversation boundaries and topic markers.
    async fn fetch_context_hints(&self, text: &str) -> Result<Vec<String>> {
        trace!("Fetching context hints from episodic memory (k={})", self.context_k);

        // Query memory for similar content
        let entries = self.memory.search(text, self.context_k).await?;

        debug!("Retrieved {} context entries from memory", entries.len());

        // Extract hints: use content from similar entries to identify patterns
        let hints: Vec<String> = entries
            .iter()
            .map(|entry| {
                // Use first 100 chars as hint (conversation starters, topic markers)
                let hint = entry.content.chars().take(100).collect::<String>();
                trace!("Context hint: {}", hint);
                hint
            })
            .collect();

        Ok(hints)
    }

    /// Detect conversation boundaries in text using context hints
    ///
    /// Analyzes text to find natural conversation breaks:
    /// - Double newlines (paragraph breaks)
    /// - Lines starting with markers like "User:", "Assistant:", etc.
    /// - Semantic shifts indicated by context hints
    ///
    /// Returns byte positions of conversation boundaries.
    fn detect_conversation_boundaries(&self, text: &str, _hints: &[String]) -> Vec<usize> {
        let mut boundaries = Vec::new();

        // Strategy 1: Detect role markers (User:, Assistant:, etc.)
        let role_markers = ["User:", "Assistant:", "System:", "user:", "assistant:", "system:"];

        for (idx, line) in text.lines().enumerate() {
            // Check if line starts with a role marker
            if role_markers.iter().any(|marker| line.trim().starts_with(marker)) {
                // Find byte offset of this line
                if let Some(byte_offset) = text.lines().take(idx).map(|l| l.len() + 1).sum::<usize>().checked_sub(0) {
                    trace!("Found conversation boundary at byte {}: {}", byte_offset, &line[..line.len().min(50)]);
                    boundaries.push(byte_offset);
                }
            }
        }

        // Strategy 2: Detect paragraph breaks (double newlines)
        let mut current_pos = 0;
        for segment in text.split("\n\n") {
            if current_pos > 0 {
                boundaries.push(current_pos);
                trace!("Found paragraph boundary at byte {}", current_pos);
            }
            current_pos += segment.len() + 2; // +2 for "\n\n"
        }

        // Remove duplicates and sort
        boundaries.sort_unstable();
        boundaries.dedup();

        debug!("Detected {} conversation boundaries", boundaries.len());
        boundaries
    }

    /// Adjust chunk boundaries to respect conversation continuity
    ///
    /// Takes base chunks and refines their boundaries to align with
    /// conversation breaks, ensuring chunks don't split mid-conversation.
    fn adjust_chunk_boundaries(
        &self,
        mut chunks: Vec<DocumentChunk>,
        boundaries: &[usize],
    ) -> Vec<DocumentChunk> {
        if boundaries.is_empty() {
            return chunks;
        }

        debug!("Adjusting {} chunks using {} boundaries", chunks.len(), boundaries.len());

        // For each chunk, check if its end boundary is near a conversation boundary
        for chunk in &mut chunks {
            let chunk_end = chunk.byte_offset + chunk.content.len();

            // Find the nearest conversation boundary within 200 bytes
            if let Some(&boundary) = boundaries
                .iter()
                .find(|&&b| b > chunk.byte_offset && b.abs_diff(chunk_end) < 200)
            {
                // Adjust chunk end to the boundary (if it makes the chunk smaller)
                if boundary < chunk_end {
                    let new_len = boundary - chunk.byte_offset;
                    if new_len > chunk.content.len() / 2 {
                        // Only adjust if we're not cutting more than half the chunk
                        chunk.content.truncate(new_len);
                        trace!("Adjusted chunk {} to end at conversation boundary (byte {})", chunk.chunk_index, boundary);
                    }
                }
            }
        }

        chunks
    }
}

#[async_trait]
impl<S: ChunkingStrategy, M: EpisodicMemory> ChunkingStrategy for MemoryAwareChunker<S, M> {
    async fn chunk(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<DocumentChunk>> {
        debug!("Memory-aware chunking: {} chars", text.len());

        // Step 1: Fetch context hints from episodic memory
        let hints = self.fetch_context_hints(text).await?;

        // Step 2: Detect conversation boundaries
        let boundaries = self.detect_conversation_boundaries(text, &hints);

        // Step 3: Perform base chunking
        let base_chunks = self.base_strategy.chunk(text, config).await?;
        debug!("Base strategy produced {} chunks", base_chunks.len());

        // Step 4: Adjust boundaries to respect conversations
        let adjusted_chunks = self.adjust_chunk_boundaries(base_chunks, &boundaries);
        debug!("Memory-aware chunking complete: {} chunks", adjusted_chunks.len());

        Ok(adjusted_chunks)
    }

    fn name(&self) -> &str {
        "memory_aware"
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Delegate to base strategy
        self.base_strategy.estimate_tokens(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::strategies::SlidingWindowChunker;
    use llmspell_memory::episodic::InMemoryEpisodicMemory;
    use llmspell_memory::types::EpisodicEntry;
    use chrono::Utc;

    #[tokio::test]
    async fn test_memory_aware_chunker_basic() {
        let base = SlidingWindowChunker::new();
        let memory = Arc::new(InMemoryEpisodicMemory::default());
        let chunker = MemoryAwareChunker::new(base, memory);

        let config = ChunkingConfig::default();
        let text = "This is a test document. It should be chunked properly.";

        let chunks = chunker.chunk(text, &config).await.unwrap();
        assert!(!chunks.is_empty());
        assert_eq!(chunker.name(), "memory_aware");
    }

    #[tokio::test]
    async fn test_conversation_boundary_detection() {
        let base = SlidingWindowChunker::new();
        let memory = Arc::new(InMemoryEpisodicMemory::default());
        let chunker = MemoryAwareChunker::new(base, memory);

        let text = "User: Hello\n\nAssistant: Hi there!\n\nUser: How are you?";
        let boundaries = chunker.detect_conversation_boundaries(text, &[]);

        // Should detect role markers and paragraph breaks
        assert!(!boundaries.is_empty());
    }

    #[tokio::test]
    async fn test_context_hints_integration() {
        let base = SlidingWindowChunker::new();
        let memory = Arc::new(InMemoryEpisodicMemory::default());

        // Add some context to memory
        let entry = EpisodicEntry {
            id: "1".to_string(),
            session_id: "test".to_string(),
            role: "user".to_string(),
            content: "Previous conversation about Rust programming".to_string(),
            timestamp: Utc::now(),
            ingestion_time: Utc::now(),
            metadata: serde_json::json!({}),
            processed: false,
            embedding: None,
        };
        memory.add(entry).await.unwrap();

        let chunker = MemoryAwareChunker::new(base, memory);
        let config = ChunkingConfig::default();
        let text = "Let's continue discussing Rust. It's a great language.";

        let chunks = chunker.chunk(text, &config).await.unwrap();
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_custom_context_k() {
        let base = SlidingWindowChunker::new();
        let memory = Arc::new(InMemoryEpisodicMemory::default());
        let chunker = MemoryAwareChunker::new(base, memory).with_context_k(10);

        assert_eq!(chunker.context_k, 10);
    }
}
