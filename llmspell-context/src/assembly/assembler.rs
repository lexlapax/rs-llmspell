//! Context assembly for LLM consumption
//!
//! Assembles reranked chunks into coherent context with temporal ordering,
//! relevance filtering, and token budget management.
//!
//! # Assembly Process
//!
//! 1. **Filter by Confidence**: Remove low-confidence chunks (< `min_confidence`)
//! 2. **Sort by Temporal Order**: Recent chunks first (timestamp descending)
//! 3. **Token Budget Enforcement**: Truncate to fit `max_tokens` limit
//! 4. **Metadata Preservation**: Keep timestamps, sources, confidence scores
//!
//! # Example
//!
//! ```rust,ignore
//! use llmspell_context::assembly::ContextAssembler;
//!
//! let assembler = ContextAssembler::new(8000, 0.3); // 8K tokens, 30% min confidence
//! let context = assembler.assemble(ranked_chunks, &query_understanding);
//! ```

use crate::types::{AssembledContext, QueryUnderstanding, RankedChunk};
use chrono::{DateTime, Utc};
use tracing::{debug, trace};

/// Context assembler for structuring chunks into LLM-ready context
///
/// Implements temporal ordering, confidence filtering, and token budget management.
pub struct ContextAssembler {
    /// Maximum tokens allowed in assembled context
    max_tokens: usize,
    /// Minimum confidence threshold (0.0-1.0)
    min_confidence: f32,
}

impl ContextAssembler {
    /// Create a new context assembler with default settings
    ///
    /// Default settings:
    /// - `max_tokens`: 8000 (fits most LLM context windows with room for prompt)
    /// - `min_confidence`: 0.3 (30% minimum relevance score)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_tokens: 8000,
            min_confidence: 0.3,
        }
    }

    /// Create a context assembler with custom settings
    ///
    /// # Arguments
    ///
    /// * `max_tokens` - Maximum tokens for assembled context
    /// * `min_confidence` - Minimum confidence threshold (0.0-1.0)
    #[must_use]
    pub const fn with_config(max_tokens: usize, min_confidence: f32) -> Self {
        Self {
            max_tokens,
            min_confidence,
        }
    }

    /// Assemble chunks into coherent context for LLM consumption
    ///
    /// # Arguments
    ///
    /// * `chunks` - Reranked chunks from retrieval + reranking pipeline
    /// * `_query` - Query understanding (for future intent-based assembly)
    ///
    /// # Assembly Strategy
    ///
    /// 1. Filter chunks below `min_confidence` threshold
    /// 2. Sort by timestamp (recent first) for temporal coherence
    /// 3. Estimate token count (rough: 4 chars = 1 token)
    /// 4. Truncate to fit token budget
    /// 5. Calculate aggregate confidence score
    ///
    /// # Returns
    ///
    /// `AssembledContext` with filtered, ordered chunks and metadata
    ///
    /// # Panics
    ///
    /// Will not panic - uses safe unwrap on non-empty chunks after validation
    #[must_use]
    pub fn assemble(
        &self,
        mut chunks: Vec<RankedChunk>,
        _query: &QueryUnderstanding,
    ) -> AssembledContext {
        debug!("Assembling context from {} chunks", chunks.len());

        // Step 1: Filter by confidence threshold
        chunks.retain(|chunk| chunk.score >= self.min_confidence);
        debug!(
            "After confidence filter: {} chunks (min: {})",
            chunks.len(),
            self.min_confidence
        );

        if chunks.is_empty() {
            return Self::empty_context();
        }

        // Step 2: Sort by timestamp (recent first)
        chunks.sort_by(|a, b| b.chunk.timestamp.cmp(&a.chunk.timestamp));
        trace!("Sorted {} chunks by timestamp (descending)", chunks.len());

        // Step 3: Enforce token budget
        let (selected_chunks, token_count) = self.enforce_token_budget(chunks);
        debug!(
            "After token budget: {} chunks ({} tokens, max: {})",
            selected_chunks.len(),
            token_count,
            self.max_tokens
        );

        // Step 4: Calculate metadata
        let temporal_span = Self::calculate_temporal_span(&selected_chunks);
        let total_confidence = Self::calculate_average_confidence(&selected_chunks);

        debug!(
            "Assembly complete: {} chunks, {:.2} avg confidence, {} tokens",
            selected_chunks.len(),
            total_confidence,
            token_count
        );

        // Format chunks into context string
        let formatted = Self::format_context(&selected_chunks);

        AssembledContext {
            chunks: selected_chunks,
            total_confidence,
            temporal_span,
            token_count,
            formatted,
        }
    }

    /// Create empty context for when no chunks pass filtering
    fn empty_context() -> AssembledContext {
        let now = Utc::now();
        AssembledContext {
            chunks: Vec::new(),
            total_confidence: 0.0,
            temporal_span: (now, now),
            token_count: 0,
            formatted: String::new(),
        }
    }

    /// Calculate temporal span from chunk timestamps
    ///
    /// Returns (oldest, newest) timestamp tuple. For empty chunks or single chunk,
    /// returns appropriate boundary values.
    fn calculate_temporal_span(chunks: &[RankedChunk]) -> (DateTime<Utc>, DateTime<Utc>) {
        match chunks.len() {
            0 => {
                let now = Utc::now();
                (now, now)
            }
            1 => {
                let timestamp = chunks[0].chunk.timestamp;
                (timestamp, timestamp)
            }
            _ => {
                let oldest = chunks.last().unwrap().chunk.timestamp;
                let newest = chunks.first().unwrap().chunk.timestamp;
                (oldest, newest)
            }
        }
    }

    /// Calculate average confidence score across chunks
    ///
    /// Returns 0.0 for empty chunks, otherwise computes mean of chunk scores.
    fn calculate_average_confidence(chunks: &[RankedChunk]) -> f32 {
        if chunks.is_empty() {
            return 0.0;
        }

        let sum: f32 = chunks.iter().map(|c| c.score).sum();
        let count = u16::try_from(chunks.len()).unwrap_or(10_000);
        sum / f32::from(count)
    }

    /// Format chunks into readable context string
    fn format_context(chunks: &[RankedChunk]) -> String {
        chunks
            .iter()
            .map(|c| {
                format!(
                    "[{} | score: {:.2} | source: {}]\n{}",
                    c.chunk.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    c.score,
                    c.chunk.source,
                    c.chunk.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// Enforce token budget by truncating chunks to fit `max_tokens`
    ///
    /// Uses rough tokenization estimate: 4 characters ≈ 1 token
    fn enforce_token_budget(&self, chunks: Vec<RankedChunk>) -> (Vec<RankedChunk>, usize) {
        let mut selected = Vec::new();
        let mut token_count = 0;

        for chunk in chunks {
            let chunk_tokens = Self::estimate_tokens(&chunk.chunk.content);

            if token_count + chunk_tokens <= self.max_tokens {
                token_count += chunk_tokens;
                selected.push(chunk);
            } else {
                trace!(
                    "Skipping chunk (would exceed budget): {} tokens + {} > {}",
                    token_count,
                    chunk_tokens,
                    self.max_tokens
                );
                break;
            }
        }

        (selected, token_count)
    }

    /// Estimate token count for text
    ///
    /// Uses rough heuristic: 4 characters ≈ 1 token
    /// This is approximate but sufficient for budget management.
    #[must_use]
    const fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(4) // Round up
    }
}

impl Default for ContextAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Chunk, QueryIntent};
    use chrono::Duration;

    fn create_test_chunks() -> Vec<RankedChunk> {
        let now = Utc::now();
        vec![
            RankedChunk {
                chunk: Chunk {
                    id: "1".to_string(),
                    content: "Recent interaction about Rust ownership".to_string(),
                    source: "memory".to_string(),
                    timestamp: now - Duration::hours(1),
                    metadata: None,
                },
                score: 0.9,
                ranker: "deberta".to_string(),
            },
            RankedChunk {
                chunk: Chunk {
                    id: "2".to_string(),
                    content: "Old conversation about Python".to_string(),
                    source: "memory".to_string(),
                    timestamp: now - Duration::days(7),
                    metadata: None,
                },
                score: 0.5,
                ranker: "deberta".to_string(),
            },
            RankedChunk {
                chunk: Chunk {
                    id: "3".to_string(),
                    content: "Very relevant recent discussion on Rust safety".to_string(),
                    source: "memory".to_string(),
                    timestamp: now - Duration::minutes(30),
                    metadata: None,
                },
                score: 0.95,
                ranker: "deberta".to_string(),
            },
            RankedChunk {
                chunk: Chunk {
                    id: "4".to_string(),
                    content: "Low confidence chunk".to_string(),
                    source: "memory".to_string(),
                    timestamp: now - Duration::hours(2),
                    metadata: None,
                },
                score: 0.2,
                ranker: "bm25".to_string(),
            },
        ]
    }

    #[test]
    fn test_assemble_basic() {
        let assembler = ContextAssembler::new();
        let chunks = create_test_chunks();
        let query = QueryUnderstanding {
            intent: QueryIntent::WhatIs,
            entities: vec!["Rust".to_string()],
            keywords: vec!["rust".to_string(), "safety".to_string()],
        };

        let context = assembler.assemble(chunks, &query);

        // Should filter out chunk 4 (score 0.2 < 0.3 threshold)
        assert_eq!(context.chunks.len(), 3);

        // Should be sorted by timestamp (newest first)
        assert_eq!(context.chunks[0].chunk.id, "3"); // 30 min ago
        assert_eq!(context.chunks[1].chunk.id, "1"); // 1 hour ago
        assert_eq!(context.chunks[2].chunk.id, "2"); // 7 days ago

        // Should calculate average confidence
        assert!((context.total_confidence - 0.7833).abs() < 0.01); // (0.95 + 0.9 + 0.5) / 3
    }

    #[test]
    fn test_confidence_filtering() {
        let assembler = ContextAssembler::with_config(8000, 0.6);
        let chunks = create_test_chunks();
        let query = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec![],
        };

        let context = assembler.assemble(chunks, &query);

        // Only chunks with score >= 0.6 should remain
        assert_eq!(context.chunks.len(), 2);
        assert!(context.chunks.iter().all(|c| c.score >= 0.6));
    }

    #[test]
    fn test_token_budget_enforcement() {
        let assembler = ContextAssembler::with_config(50, 0.0); // Very small budget
        let chunks = create_test_chunks();
        let query = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec![],
        };

        let context = assembler.assemble(chunks, &query);

        // Should truncate to fit budget
        assert!(context.token_count <= 50);
        assert!(!context.chunks.is_empty());
    }

    #[test]
    fn test_empty_chunks() {
        let assembler = ContextAssembler::new();
        let query = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec![],
        };

        let context = assembler.assemble(vec![], &query);

        assert!(context.chunks.is_empty());
        assert!(context.total_confidence.abs() < f32::EPSILON);
        assert_eq!(context.token_count, 0);
        assert_eq!(context.formatted, "");
    }

    #[test]
    fn test_temporal_span() {
        let assembler = ContextAssembler::new();
        let chunks = create_test_chunks();
        let query = QueryUnderstanding {
            intent: QueryIntent::Unknown,
            entities: vec![],
            keywords: vec![],
        };

        let context = assembler.assemble(chunks, &query);

        // Should have temporal span from oldest to newest
        let (oldest, newest) = context.temporal_span;

        // Newest chunk is 30 min ago, oldest is 7 days ago
        let duration = newest.signed_duration_since(oldest);
        assert!(duration.num_days() >= 6);
    }

    #[test]
    fn test_estimate_tokens() {
        // 4 chars ≈ 1 token (rough estimate)
        assert_eq!(ContextAssembler::estimate_tokens("test"), 1); // 4.div_ceil(4) = 1
        assert_eq!(ContextAssembler::estimate_tokens("hello world"), 3); // 11.div_ceil(4) = 3
        assert_eq!(ContextAssembler::estimate_tokens(""), 0); // 0.div_ceil(4) = 0
        assert_eq!(ContextAssembler::estimate_tokens("a"), 1); // 1.div_ceil(4) = 1
    }
}
