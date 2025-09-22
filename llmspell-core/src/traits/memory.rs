//! Memory integration traits for Phase 10: Adaptive Memory System
//!
//! Provides traits and types for implementing an Adaptive Temporal Knowledge Graph (A-TKG)
//! memory architecture with working, episodic, and semantic memory types.

use crate::error::LLMSpellError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// Represents a single interaction log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionLog {
    /// Unique identifier for this interaction
    pub id: String,
    /// Timestamp of the interaction
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User input that triggered the interaction
    pub user_input: String,
    /// Agent response to the user
    pub agent_response: String,
    /// Metadata about the interaction
    pub metadata: HashMap<String, serde_json::Value>,
    /// Session ID this interaction belongs to
    pub session_id: Option<String>,
}

/// Represents a memory item retrieved from the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier for this memory
    pub id: String,
    /// The actual content of the memory
    pub content: String,
    /// Type of memory (episodic, semantic, working)
    pub memory_type: MemoryType,
    /// Relevance score for retrieval
    pub relevance: f32,
    /// Temporal information
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed time
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of memory in the adaptive memory system
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    /// Immediate session context
    Working,
    /// Raw interactions indexed by vectors
    Episodic,
    /// Temporal Knowledge Graph storing facts, entities, relationships
    Semantic,
}

/// Query for retrieving context from memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuery {
    /// The query text
    pub query: String,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Memory types to search
    pub memory_types: Vec<MemoryType>,
    /// Minimum relevance score
    pub min_relevance: f32,
    /// Time range filter
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
}

/// Result of memory consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    /// Number of memories consolidated
    pub consolidated_count: usize,
    /// Number of memories deleted
    pub deleted_count: usize,
    /// Number of new connections created
    pub new_connections: usize,
    /// Time taken for consolidation
    pub duration: std::time::Duration,
}

/// Memory system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of memories
    pub total_memories: usize,
    /// Breakdown by memory type
    pub by_type: HashMap<String, usize>,
    /// Storage size in bytes
    pub storage_size: usize,
    /// Last consolidation time
    pub last_consolidation: Option<chrono::DateTime<chrono::Utc>>,
}

/// Memory Integration trait for Phase 10 Adaptive Memory System
///
/// This trait defines the interface for implementing an adaptive memory system
/// that can store interactions, query context, and consolidate memories using
/// LLM-driven logic. Implementations should provide efficient retrieval with
/// P95 latency <300ms and support for 1M+ memory items.
#[async_trait]
pub trait MemoryIntegration: Send + Sync + Debug {
    /// Store an interaction in the memory system
    async fn store_interaction(&self, interaction: InteractionLog) -> Result<(), LLMSpellError>;

    /// Query context from the memory system
    async fn query_context(&self, query: ContextQuery) -> Result<Vec<MemoryItem>, LLMSpellError>;

    /// Consolidate memories using LLM-driven logic
    async fn consolidate_memories(&self) -> Result<ConsolidationResult, LLMSpellError>;

    /// Clear working memory (session cleanup)
    async fn clear_working_memory(&self, session_id: &str) -> Result<(), LLMSpellError>;

    /// Get memory statistics
    async fn get_memory_stats(&self) -> Result<MemoryStats, LLMSpellError>;
}
