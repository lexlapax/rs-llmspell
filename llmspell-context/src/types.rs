//! Shared types for the context engineering pipeline

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Query intent classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryIntent {
    /// "How do I..." questions
    HowTo,
    /// "What is..." questions
    WhatIs,
    /// "Why does..." questions
    WhyDoes,
    /// Debugging/troubleshooting
    Debug,
    /// Explanation requests
    Explain,
    /// Unknown intent
    Unknown,
}

/// Query understanding output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUnderstanding {
    /// Classified intent
    pub intent: QueryIntent,
    /// Extracted entities (class names, function names, etc.)
    pub entities: Vec<String>,
    /// Extracted keywords (important terms)
    pub keywords: Vec<String>,
}

/// Retrieval strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalStrategy {
    /// Recent interactions (vector similarity)
    Episodic,
    /// Knowledge graph traversal
    Semantic,
    /// Keyword-based BM25
    BM25,
    /// Combined episodic + semantic
    Hybrid,
}

/// Memory chunk (before reranking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique chunk ID
    pub id: String,
    /// Chunk content
    pub content: String,
    /// Source (session ID, node ID, etc.)
    pub source: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Ranked chunk (after reranking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedChunk {
    /// Original chunk
    pub chunk: Chunk,
    /// Relevance score (0.0-1.0)
    pub score: f32,
    /// Reranker that produced this score
    pub ranker: String,
}

/// Assembled context ready for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledContext {
    /// Top-ranked chunks
    pub chunks: Vec<RankedChunk>,
    /// Total confidence score
    pub total_confidence: f32,
    /// Temporal span (earliest to latest timestamp)
    pub temporal_span: (DateTime<Utc>, DateTime<Utc>),
    /// Total token count
    pub token_count: usize,
    /// Formatted context string
    pub formatted: String,
}

/// BM25 parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BM25Config {
    /// Term frequency saturation parameter (default: 1.5)
    pub k1: f32,
    /// Length normalization parameter (default: 0.75)
    pub b: f32,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            k1: 1.5,
            b: 0.75,
        }
    }
}
