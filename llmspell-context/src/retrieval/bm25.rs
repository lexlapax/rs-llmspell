//! BM25 keyword-based retrieval for episodic memory
//!
//! Implements BM25 (Best Matching 25) algorithm for relevance ranking based on term frequency
//! and inverse document frequency. Used for keyword-based retrieval from episodic memory.
//!
//! # Algorithm
//!
//! BM25 score for document D and query Q:
//!
//! ```text
//! score(D, Q) = Σ IDF(qi) · (f(qi, D) · (k1 + 1)) / (f(qi, D) + k1 · (1 - b + b · |D| / avgdl))
//! ```
//!
//! Where:
//! - `IDF(qi)` = Inverse document frequency of query term qi
//! - `f(qi, D)` = Frequency of qi in document D
//! - `k1` = Term frequency saturation parameter (default: 1.5)
//! - `b` = Length normalization parameter (default: 0.75)
//! - `|D|` = Length of document D in words
//! - `avgdl` = Average document length in the corpus
//!
//! # Examples
//!
//! ```rust,ignore
//! use llmspell_context::retrieval::BM25Retriever;
//! use llmspell_memory::EpisodicMemory;
//!
//! let retriever = BM25Retriever::new(k1, b);
//! let chunks = retriever.retrieve("Rust memory safety", &memory, 10).await?;
//! ```

use crate::error::{ContextError, Result};
use crate::traits::Retriever;
use crate::types::{BM25Config, Chunk};
use async_trait::async_trait;
use llmspell_memory::traits::EpisodicMemory;
use llmspell_utils::text::stopwords::is_stopword;
use std::collections::HashMap;
use tracing::{debug, trace};

/// BM25 retriever for keyword-based search
pub struct BM25Retriever {
    config: BM25Config,
}

impl BM25Retriever {
    /// Create a new BM25 retriever with default parameters
    ///
    /// Default parameters:
    /// - k1 = 1.5 (term frequency saturation)
    /// - b = 0.75 (length normalization)
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BM25Config::default(),
        }
    }

    /// Create a new BM25 retriever with custom parameters
    ///
    /// # Arguments
    ///
    /// * `k1` - Term frequency saturation parameter (typical range: 1.2-2.0)
    /// * `b` - Length normalization parameter (typical range: 0.5-0.9)
    #[must_use]
    pub const fn with_config(config: BM25Config) -> Self {
        Self { config }
    }

    /// Tokenize text for BM25 processing
    ///
    /// - Lowercase text
    /// - Split on whitespace and punctuation
    /// - Filter stopwords using llmspell-utils
    /// - Remove empty strings
    #[inline]
    fn tokenize_text(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter_map(|token| {
                let trimmed = token.trim();
                if trimmed.is_empty() {
                    return None;
                }

                // Filter stopwords (capitalized form for is_stopword check)
                let capitalized = capitalize(trimmed);
                if is_stopword(&capitalized) {
                    trace!("Filtered stopword: {trimmed}");
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect()
    }

    /// Compute IDF (Inverse Document Frequency) for each term
    ///
    /// IDF(t) = log((N - df(t) + 0.5) / (df(t) + 0.5) + 1)
    ///
    /// Where:
    /// - N = total number of documents
    /// - df(t) = number of documents containing term t
    #[allow(clippy::cast_precision_loss)]
    fn compute_idf_for_terms(chunks: &[Chunk], terms: &[String]) -> HashMap<String, f32> {
        let n = chunks.len() as f32;
        let mut doc_freq: HashMap<String, usize> = HashMap::new();

        // Count document frequency for each term
        for chunk in chunks {
            let doc_tokens = Self::tokenize_text(&chunk.content);
            let mut seen_terms = std::collections::HashSet::new();
            for token in doc_tokens {
                if terms.contains(&token) && seen_terms.insert(token.clone()) {
                    *doc_freq.entry(token).or_insert(0) += 1;
                }
            }
        }

        // Compute IDF for each query term
        terms
            .iter()
            .map(|term| {
                let df = doc_freq.get(term).copied().unwrap_or(0) as f32;
                let idf = ((n - df + 0.5) / (df + 0.5)).ln_1p();
                (term.clone(), idf)
            })
            .collect()
    }

    /// Compute BM25 score for a document given query terms
    #[allow(clippy::cast_precision_loss)]
    fn compute_score(
        &self,
        doc_tokens: &[String],
        query_terms: &[String],
        idf: &HashMap<String, f32>,
        avg_doc_len: f32,
    ) -> f32 {
        let doc_len = doc_tokens.len() as f32;
        let k1 = self.config.k1;
        let b = self.config.b;

        // Compute term frequencies for document
        let mut term_freq: HashMap<String, usize> = HashMap::new();
        for token in doc_tokens {
            *term_freq.entry(token.clone()).or_insert(0) += 1;
        }

        // Sum BM25 score for each query term
        query_terms
            .iter()
            .map(|term| {
                let tf = term_freq.get(term).copied().unwrap_or(0) as f32;
                let idf_score = idf.get(term).copied().unwrap_or(0.0);

                // BM25 formula: IDF(qi) · (f(qi, D) · (k1 + 1)) / (f(qi, D) + k1 · (1 - b + b · |D| / avgdl))
                let denominator = k1.mul_add(1.0 - b + b * doc_len / avg_doc_len, tf);
                idf_score * (tf * (k1 + 1.0)) / denominator
            })
            .sum()
    }

    /// Retrieve top-k chunks using BM25 scoring
    ///
    /// # Arguments
    ///
    /// * `query` - Query string
    /// * `chunks` - Candidate chunks to score
    /// * `top_k` - Number of top results to return
    ///
    /// # Returns
    ///
    /// Vector of top-k chunks sorted by BM25 score (descending)
    #[allow(clippy::cast_precision_loss)]
    pub fn retrieve_from_chunks(&self, query: &str, chunks: &[Chunk], top_k: usize) -> Vec<Chunk> {
        if chunks.is_empty() {
            return Vec::new();
        }

        // Extract query terms
        let query_terms = Self::tokenize_text(query);
        if query_terms.is_empty() {
            debug!("No query terms extracted from: {query}");
            return Vec::new();
        }

        debug!("Query terms: {:?}", query_terms);

        // Compute IDF for query terms
        let idf = Self::compute_idf_for_terms(chunks, &query_terms);

        // Tokenize all documents and compute average length
        let doc_tokens: Vec<Vec<String>> = chunks
            .iter()
            .map(|chunk| Self::tokenize_text(&chunk.content))
            .collect();

        let avg_doc_len =
            doc_tokens.iter().map(Vec::len).sum::<usize>() as f32 / doc_tokens.len() as f32;

        // Score each document
        let mut scored_chunks: Vec<(usize, f32)> = doc_tokens
            .iter()
            .enumerate()
            .map(|(idx, tokens)| {
                let score = self.compute_score(tokens, &query_terms, &idf, avg_doc_len);
                (idx, score)
            })
            .collect();

        // Sort by score (descending)
        scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top-k chunks
        scored_chunks
            .into_iter()
            .take(top_k)
            .filter(|(_, score)| *score > 0.0)
            .map(|(idx, _score)| chunks[idx].clone())
            .collect()
    }

    /// Retrieve top-k chunks from episodic memory using BM25 scoring
    ///
    /// This method integrates BM25 with episodic memory by:
    /// 1. Fetching candidates from episodic memory using vector similarity
    /// 2. Converting episodic entries to chunks
    /// 3. Scoring with BM25 algorithm
    /// 4. Returning top-k results
    ///
    /// # Arguments
    ///
    /// * `query` - Query string
    /// * `memory` - Episodic memory instance to retrieve from
    /// * `candidate_size` - Number of candidates to fetch from memory (before BM25 reranking)
    /// * `top_k` - Number of top results to return after BM25 reranking
    ///
    /// # Returns
    ///
    /// Vector of top-k chunks sorted by BM25 score (descending)
    ///
    /// # Errors
    ///
    /// Returns `ContextError::MemoryError` if episodic memory retrieval fails
    pub async fn retrieve_from_memory<M: EpisodicMemory>(
        &self,
        query: &str,
        memory: &M,
        candidate_size: usize,
        top_k: usize,
    ) -> Result<Vec<Chunk>> {
        // Fetch candidates from episodic memory using vector similarity
        let entries = memory
            .search(query, candidate_size)
            .await
            .map_err(|e| ContextError::MemoryError(e.to_string()))?;

        // Convert episodic entries to chunks
        let chunks: Vec<Chunk> = entries
            .into_iter()
            .map(|entry| Chunk {
                id: entry.id,
                content: entry.content,
                source: entry.session_id,
                timestamp: entry.timestamp,
                metadata: Some(entry.metadata),
            })
            .collect();

        // Score and rerank using BM25
        Ok(self.retrieve_from_chunks(query, &chunks, top_k))
    }
}

impl Default for BM25Retriever {
    fn default() -> Self {
        Self::new()
    }
}

/// Capitalize first character of string (for stopword checking)
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[async_trait]
impl Retriever for BM25Retriever {
    async fn retrieve(&self, _query: &str, _top_k: usize) -> Result<Vec<Chunk>> {
        // This will be implemented when we integrate with EpisodicMemory in a future step
        Err(ContextError::RetrievalError(
            "BM25 retrieval requires memory integration - use retrieve_from_chunks for now"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_chunks() -> Vec<Chunk> {
        vec![
            Chunk {
                id: "1".to_string(),
                content: "Rust is a systems programming language".to_string(),
                source: "test".to_string(),
                timestamp: Utc::now(),
                metadata: None,
            },
            Chunk {
                id: "2".to_string(),
                content: "Rust has memory safety guarantees".to_string(),
                source: "test".to_string(),
                timestamp: Utc::now(),
                metadata: None,
            },
            Chunk {
                id: "3".to_string(),
                content: "Python is a high-level language".to_string(),
                source: "test".to_string(),
                timestamp: Utc::now(),
                metadata: None,
            },
        ]
    }

    #[test]
    fn test_tokenize() {
        let tokens = BM25Retriever::tokenize_text("Rust is a systems programming language");

        // "is" and "a" should be filtered as stopwords
        assert_eq!(tokens, vec!["rust", "systems", "programming", "language"]);
    }

    #[test]
    fn test_tokenize_with_punctuation() {
        let tokens = BM25Retriever::tokenize_text("What is Rust? It's great!");

        // "What", "is", "It" should be filtered, punctuation removed
        assert_eq!(tokens, vec!["rust", "s", "great"]);
    }

    #[test]
    fn test_retrieve_empty() {
        let retriever = BM25Retriever::new();
        let results = retriever.retrieve_from_chunks("Rust", &[], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_retrieve_no_terms() {
        let retriever = BM25Retriever::new();
        let chunks = create_test_chunks();
        // Query with only stopwords
        let results = retriever.retrieve_from_chunks("is a the", &chunks, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_retrieve_basic() {
        let retriever = BM25Retriever::new();
        let chunks = create_test_chunks();
        let results = retriever.retrieve_from_chunks("Rust programming", &chunks, 10);

        // Should return chunks containing "Rust" and "programming"
        assert!(!results.is_empty());
        assert!(results[0].id == "1" || results[0].id == "2");
    }

    #[test]
    fn test_retrieve_ranking() {
        let retriever = BM25Retriever::new();
        let chunks = create_test_chunks();
        let results = retriever.retrieve_from_chunks("Rust memory safety", &chunks, 10);

        // Chunk 2 has both "Rust" and "memory safety", should rank highest
        assert_eq!(results[0].id, "2");
    }

    #[test]
    fn test_retrieve_top_k() {
        let retriever = BM25Retriever::new();
        let chunks = create_test_chunks();
        let results = retriever.retrieve_from_chunks("language", &chunks, 2);

        // Should return at most 2 results
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_retrieve_from_memory() {
        use llmspell_memory::episodic::InMemoryEpisodicMemory;
        use llmspell_memory::types::EpisodicEntry;

        // Create in-memory episodic storage
        let memory = InMemoryEpisodicMemory::default();

        // Add some test entries
        let entry1 = EpisodicEntry {
            id: "1".to_string(),
            session_id: "test".to_string(),
            role: "user".to_string(),
            content: "Rust is a systems programming language".to_string(),
            timestamp: Utc::now(),
            ingestion_time: Utc::now(),
            metadata: serde_json::json!({}),
            processed: false,
            embedding: None,
        };

        let entry2 = EpisodicEntry {
            id: "2".to_string(),
            session_id: "test".to_string(),
            role: "assistant".to_string(),
            content: "Rust has memory safety guarantees".to_string(),
            timestamp: Utc::now(),
            ingestion_time: Utc::now(),
            metadata: serde_json::json!({}),
            processed: false,
            embedding: None,
        };

        memory.add(entry1).await.unwrap();
        memory.add(entry2).await.unwrap();

        // Retrieve using BM25
        let retriever = BM25Retriever::new();
        let results = retriever
            .retrieve_from_memory("Rust memory safety", &memory, 10, 5)
            .await
            .unwrap();

        // Should return relevant results
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "2"); // Entry 2 has both "Rust" and "memory safety"
    }
}
