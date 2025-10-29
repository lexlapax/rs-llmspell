//! Context assembly trait for memory-enhanced retrieval
//!
//! Provides abstraction for hybrid retrieval combining episodic memory,
//! semantic memory, and RAG. Implemented by ContextBridge in llmspell-bridge.
//!
//! # Architecture
//!
//! This trait lives in llmspell-core to enable both llmspell-bridge (implementation)
//! and llmspell-templates (consumer) to depend on the abstraction without circular
//! dependencies. Follows Dependency Inversion Principle.
//!
//! # Dependency Graph
//!
//! ```text
//! llmspell-core (ContextAssembler trait)
//!   ↓                  ↓
//! llmspell-bridge    llmspell-templates
//! (implements)       (consumes)
//! ```
//!
//! # Design Decision
//!
//! Task 13.11.0 initially used type erasure (Arc<dyn Any>) to avoid circular dependency.
//! Task 13.11.1a extracts this trait to enable compile-time type safety while maintaining
//! clean layering.

use async_trait::async_trait;
use serde_json::Value;

/// Context assembler for memory-enhanced retrieval
///
/// Composes retrieval strategies (episodic, semantic, hybrid, RAG) with
/// memory manager and RAG pipeline for context-aware LLM interactions.
///
/// # Retrieval Strategies
///
/// - **episodic**: Recent interactions from episodic memory
/// - **semantic**: Knowledge graph entities from semantic memory
/// - **hybrid**: Combined episodic + semantic retrieval
/// - **rag**: RAG vector search + memory hybrid retrieval
///
/// # Token Budget
///
/// - **Minimum**: 100 tokens (error if lower)
/// - **Default**: 2000 tokens (balanced context)
/// - **Maximum**: 8192 tokens (warning if higher)
///
/// # Example
///
/// ```ignore
/// use llmspell_core::ContextAssembler;
/// use std::sync::Arc;
///
/// async fn assemble_context(
///     assembler: &Arc<dyn ContextAssembler>,
///     query: &str,
/// ) -> Result<serde_json::Value, String> {
///     assembler.assemble(
///         query,
///         "hybrid",
///         2000,
///         Some("session-123")
///     ).await
/// }
/// ```
#[async_trait]
pub trait ContextAssembler: Send + Sync {
    /// Assemble context from memory using specified retrieval strategy
    ///
    /// Composes retrieval → reranking → assembly pipeline based on strategy.
    ///
    /// # Arguments
    ///
    /// * `query` - Query string for retrieval
    /// * `strategy` - Retrieval strategy: "episodic", "semantic", "hybrid", or "rag" (case-sensitive)
    /// * `max_tokens` - Maximum tokens for assembled context (100-8192)
    /// * `session_id` - Optional session ID for episodic filtering
    ///
    /// # Returns
    ///
    /// JSON representation of assembled context with:
    /// - `chunks`: Array of ranked chunks with content, scores, timestamps
    /// - `total_confidence`: Average relevance score across chunks
    /// - `temporal_span`: Tuple of (oldest, newest) timestamps
    /// - `token_count`: Total tokens in assembled context
    /// - `formatted`: Human-readable formatted context string
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Strategy is invalid (not "episodic", "semantic", "hybrid", or "rag")
    /// - Token budget < 100 (too small to be useful)
    /// - Memory retrieval fails
    /// - Reranking fails
    /// - Assembly fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let context = assembler.assemble(
    ///     "What is Rust ownership?",
    ///     "hybrid",
    ///     2000,
    ///     Some("session-123")
    /// ).await?;
    ///
    /// println!("Assembled {} chunks, {} tokens",
    ///     context["chunks"].as_array().unwrap().len(),
    ///     context["token_count"].as_u64().unwrap()
    /// );
    /// ```
    async fn assemble(
        &self,
        query: &str,
        strategy: &str,
        max_tokens: usize,
        session_id: Option<&str>,
    ) -> Result<Value, String>;
}
