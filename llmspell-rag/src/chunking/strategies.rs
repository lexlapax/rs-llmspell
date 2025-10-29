//! Chunking strategies for document processing

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for chunking strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Maximum chunk size in tokens
    pub max_tokens: usize,

    /// Overlap size in tokens (for sliding window)
    pub overlap_tokens: usize,

    /// Minimum chunk size (avoid tiny chunks)
    pub min_tokens: usize,

    /// Whether to respect sentence boundaries
    pub respect_sentences: bool,

    /// Whether to respect paragraph boundaries
    pub respect_paragraphs: bool,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            overlap_tokens: 64,
            min_tokens: 32,
            respect_sentences: true,
            respect_paragraphs: false,
        }
    }
}

/// A chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// The chunk text
    pub content: String,

    /// Byte offset in the original document
    pub byte_offset: usize,

    /// Token count
    pub token_count: usize,

    /// Chunk index in document
    pub chunk_index: usize,

    /// Metadata (e.g., source document, page number, etc.)
    pub metadata: serde_json::Value,
}

impl DocumentChunk {
    /// Create a new document chunk
    #[must_use]
    pub fn new(
        content: String,
        byte_offset: usize,
        token_count: usize,
        chunk_index: usize,
    ) -> Self {
        Self {
            content,
            byte_offset,
            token_count,
            chunk_index,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Add metadata to the chunk
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Trait for chunking strategies
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    /// Split text into chunks
    ///
    /// # Errors
    ///
    /// Returns an error if chunking fails due to invalid configuration or tokenization issues
    async fn chunk(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<DocumentChunk>>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> usize;
}

/// Sliding window chunking strategy
///
/// Splits text into fixed-size overlapping chunks
#[derive(Default)]
pub struct SlidingWindowChunker {
    /// Optional tokenizer for accurate token counting
    tokenizer: Option<Box<dyn super::tokenizer::TokenCounter>>,
}

impl std::fmt::Debug for SlidingWindowChunker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlidingWindowChunker")
            .field("has_tokenizer", &self.tokenizer.is_some())
            .finish()
    }
}

impl SlidingWindowChunker {
    /// Create new sliding window chunker
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tokenizer for accurate token counting
    #[must_use]
    pub fn with_tokenizer(mut self, tokenizer: Box<dyn super::tokenizer::TokenCounter>) -> Self {
        self.tokenizer = Some(tokenizer);
        self
    }

    /// Find the next sentence boundary
    fn find_sentence_boundary(text: &str, start: usize, max_pos: usize) -> usize {
        let search_text = &text[start..max_pos.min(text.len())];

        // Look for sentence endings
        for (i, ch) in search_text.char_indices().rev() {
            if matches!(ch, '.' | '!' | '?') {
                // Check if followed by whitespace or end of text
                let next_pos = start + i + ch.len_utf8();
                if next_pos >= text.len()
                    || text
                        .chars()
                        .nth(next_pos / ch.len_utf8())
                        .is_some_and(char::is_whitespace)
                {
                    return next_pos;
                }
            }
        }

        // No sentence boundary found, return max position
        max_pos.min(text.len())
    }

    /// Find the next paragraph boundary
    fn find_paragraph_boundary(text: &str, start: usize, max_pos: usize) -> usize {
        let search_text = &text[start..max_pos.min(text.len())];

        // Look for double newlines
        if let Some(pos) = search_text.rfind("\n\n") {
            return start + pos + 2;
        }

        // Look for single newline as fallback
        if let Some(pos) = search_text.rfind('\n') {
            return start + pos + 1;
        }

        max_pos.min(text.len())
    }
}

#[async_trait]
impl ChunkingStrategy for SlidingWindowChunker {
    async fn chunk(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<DocumentChunk>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut current_pos = 0;
        let mut chunk_index = 0;

        while current_pos < text.len() {
            // Estimate chunk end position (estimate_chunk_size returns absolute position)
            let estimated_end = self.estimate_chunk_size(text, current_pos, config.max_tokens);

            // Adjust for boundaries if configured
            let chunk_end = if config.respect_paragraphs {
                Self::find_paragraph_boundary(text, current_pos, estimated_end)
            } else if config.respect_sentences {
                Self::find_sentence_boundary(text, current_pos, estimated_end)
            } else {
                estimated_end.min(text.len())
            };

            // Extract chunk
            let chunk_text = &text[current_pos..chunk_end];
            let token_count = self.estimate_tokens(chunk_text);

            // Only add if meets minimum size or is the last chunk
            if token_count >= config.min_tokens || chunk_end >= text.len() {
                chunks.push(DocumentChunk::new(
                    chunk_text.to_string(),
                    current_pos,
                    token_count,
                    chunk_index,
                ));
                chunk_index += 1;
            }

            // Move to next position with overlap
            if chunk_end >= text.len() {
                break;
            }

            // Ensure we make progress to avoid infinite loop
            if chunk_end <= current_pos {
                current_pos += 1;
            } else if config.overlap_tokens > 0 {
                // Calculate overlap - estimate bytes for overlap_tokens
                let overlap_bytes = config.overlap_tokens * 4; // Rough estimate
                current_pos = chunk_end.saturating_sub(overlap_bytes);
                // Ensure forward progress
                if current_pos <= chunk_end.saturating_sub(chunk_text.len()) {
                    current_pos = chunk_end;
                }
            } else {
                current_pos = chunk_end;
            }
        }

        Ok(chunks)
    }

    fn name(&self) -> &'static str {
        "sliding_window"
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        self.tokenizer.as_ref().map_or_else(
            || text.len() / 4, // Rough estimate: 1 token per 4 characters
            |tokenizer| tokenizer.count_tokens(text),
        )
    }
}

impl SlidingWindowChunker {
    /// Estimate byte size for a given token count
    fn estimate_chunk_size(&self, text: &str, start: usize, target_tokens: usize) -> usize {
        self.tokenizer.as_ref().map_or_else(
            || {
                // Simple estimation without tokenizer: ~4 chars per token
                let estimated_chars = target_tokens * 4;
                start + estimated_chars.min(text.len() - start)
            },
            |tokenizer| {
                // Use binary search for efficient estimation
                let text_slice = &text[start..];
                if text_slice.is_empty() {
                    return start;
                }

                // Binary search for the right chunk size
                let mut left = 0;
                let mut right = text_slice.len();
                let mut best_end = start;

                while left < right {
                    let mid = (left + right).div_ceil(2);

                    // Find char boundary at mid position
                    let mid_byte = if mid >= text_slice.len() {
                        text_slice.len()
                    } else {
                        // Find the next valid UTF-8 boundary
                        let mut boundary = mid;
                        while boundary < text_slice.len() && !text_slice.is_char_boundary(boundary)
                        {
                            boundary += 1;
                        }
                        boundary
                    };

                    let token_count = tokenizer.count_tokens(&text_slice[..mid_byte]);

                    match token_count.cmp(&target_tokens) {
                        std::cmp::Ordering::Less => {
                            left = mid_byte + 1;
                            best_end = start + mid_byte;
                        }
                        std::cmp::Ordering::Greater => {
                            right = mid_byte.saturating_sub(1);
                        }
                        std::cmp::Ordering::Equal => {
                            // Exact match
                            return start + mid_byte;
                        }
                    }
                }

                // Return the best position found
                best_end.max(start + 1) // Ensure we make progress
            },
        )
    }
}

/// Semantic chunking strategy (placeholder)
///
/// TODO: Implement semantic chunking using sentence embeddings
/// to find natural break points based on semantic similarity
#[derive(Debug, Default)]
pub struct SemanticChunker {
    /// Similarity threshold for splitting
    similarity_threshold: f32,
}

impl SemanticChunker {
    /// Create new semantic chunker
    #[must_use]
    pub const fn new() -> Self {
        Self {
            similarity_threshold: 0.5,
        }
    }

    /// Set similarity threshold
    #[must_use]
    pub const fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }
}

#[async_trait]
impl ChunkingStrategy for SemanticChunker {
    async fn chunk(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<DocumentChunk>> {
        // For now, fall back to sliding window
        // TODO: Implement semantic chunking
        let chunker = SlidingWindowChunker::new();
        chunker.chunk(text, config).await
    }

    fn name(&self) -> &'static str {
        "semantic"
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: 1 token per 4 characters
        text.len() / 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sliding_window_chunking() {
        let chunker = SlidingWindowChunker::new();
        let config = ChunkingConfig {
            max_tokens: 10,
            overlap_tokens: 2,
            min_tokens: 3,
            respect_sentences: false,
            respect_paragraphs: false,
        };

        let text = "This is a test document with multiple words that should be chunked properly.";
        let chunks = chunker.chunk(text, &config).await.unwrap();

        assert!(!chunks.is_empty());
        assert!(chunks.len() > 1);

        // Check chunk properties
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i);
            assert!(chunk.token_count >= config.min_tokens || i == chunks.len() - 1);
        }
    }

    #[tokio::test]
    async fn test_respect_sentence_boundaries() {
        let chunker = SlidingWindowChunker::new();
        let config = ChunkingConfig {
            max_tokens: 15,
            overlap_tokens: 0,
            min_tokens: 3,
            respect_sentences: true,
            respect_paragraphs: false,
        };

        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let chunks = chunker.chunk(text, &config).await.unwrap();

        // Check that chunks end at sentence boundaries
        for chunk in &chunks {
            let last_char = chunk.content.chars().last();
            assert!(
                last_char == Some('.') || chunk.content == text,
                "Chunk should end at sentence boundary: '{}'",
                chunk.content
            );
        }
    }

    #[tokio::test]
    async fn test_respect_paragraph_boundaries() {
        let chunker = SlidingWindowChunker::new();
        let config = ChunkingConfig {
            max_tokens: 8, // Smaller to ensure we split at paragraph boundaries
            overlap_tokens: 0,
            min_tokens: 3,
            respect_sentences: false,
            respect_paragraphs: true,
        };

        let text = "First paragraph here.\n\nSecond paragraph here.\n\nThird paragraph.";
        let chunks = chunker.chunk(text, &config).await.unwrap();

        // Check that chunks respect paragraph boundaries
        // When respecting paragraphs, we should get one chunk per paragraph
        // unless a paragraph is too long
        assert!(chunks.len() >= 2, "Should have multiple chunks");

        // Each chunk should typically contain a single paragraph
        for chunk in &chunks {
            // Count internal double newlines (not at the end)
            let content_without_trailing = chunk.content.trim_end();
            let internal_breaks = content_without_trailing.matches("\n\n").count();
            assert_eq!(
                internal_breaks, 0,
                "Chunk should not have internal paragraph breaks: '{}'",
                chunk.content
            );
        }
    }

    #[tokio::test]
    async fn test_empty_text() {
        let chunker = SlidingWindowChunker::new();
        let config = ChunkingConfig::default();

        let chunks = chunker.chunk("", &config).await.unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_metadata() {
        let chunk = DocumentChunk::new("test content".to_string(), 0, 3, 0).with_metadata(
            serde_json::json!({
                "source": "test.txt",
                "page": 1
            }),
        );

        assert_eq!(chunk.content, "test content");
        assert_eq!(chunk.metadata["source"], "test.txt");
        assert_eq!(chunk.metadata["page"], 1);
    }
}
