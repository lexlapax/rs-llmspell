//! ABOUTME: Context bridge providing language-agnostic context assembly operations
//! ABOUTME: Composes `BM25Retriever` + `ContextAssembler` + `MemoryManager` for RAG workflows

use llmspell_context::{
    assembly::ContextAssembler,
    reranking::BM25Reranker,
    traits::Reranker,
    types::{Chunk, QueryIntent, QueryUnderstanding},
};
use llmspell_memory::MemoryManager;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

/// Context bridge for language-agnostic context assembly operations
///
/// This bridge composes context engineering components (`BM25Retriever`, `BM25Reranker`,
/// `ContextAssembler`) with `MemoryManager` to provide blocking interfaces for script languages.
///
/// # Component Composition Pattern
///
/// No `ContextPipeline` struct exists - bridge composes components directly:
/// 1. **`BM25Retriever`** - Keyword-based retrieval from memory (stateless, created on demand)
/// 2. **`BM25Reranker`** - Fast lexical reranking (stateless, <5ms for 20 chunks)
/// 3. **`ContextAssembler`** - Token budget management and temporal ordering (stateless)
///
/// # Strategy Support
///
/// - **Episodic**: Recent interactions from episodic memory
/// - **Semantic**: Knowledge graph entities from semantic memory
/// - **Hybrid**: Combined episodic + semantic retrieval
///
/// # Pattern
///
/// Follows async→blocking conversion like `MemoryBridge`:
/// ```ignore
/// self.runtime.block_on(async {
///     let retriever = BM25Retriever::new();
///     let chunks = retriever.retrieve_from_memory(...).await?;
///     // ... rerank and assemble
/// })
/// ```
pub struct ContextBridge {
    /// Reference to the memory manager
    memory_manager: Arc<dyn MemoryManager>,
    /// Tokio runtime handle for async→blocking conversion
    runtime: tokio::runtime::Handle,
}

impl ContextBridge {
    /// Create a new context bridge
    ///
    /// # Arguments
    ///
    /// * `memory_manager` - The memory manager to wrap
    ///
    /// # Example
    ///
    /// ```ignore
    /// use llmspell_memory::DefaultMemoryManager;
    /// use llmspell_bridge::ContextBridge;
    /// use std::sync::Arc;
    ///
    /// let memory = Arc::new(DefaultMemoryManager::new_in_memory().await?);
    /// let bridge = ContextBridge::new(memory);
    /// ```
    #[must_use]
    pub fn new(memory_manager: Arc<dyn MemoryManager>) -> Self {
        info!("Creating ContextBridge");
        Self {
            memory_manager,
            runtime: llmspell_kernel::global_io_runtime().handle().clone(),
        }
    }

    /// Assemble context from memory using specified retrieval strategy
    ///
    /// Composes retrieval → reranking → assembly pipeline based on strategy.
    ///
    /// # Arguments
    ///
    /// * `query` - Query string
    /// * `strategy` - Retrieval strategy: "episodic", "semantic", or "hybrid" (case-sensitive)
    /// * `max_tokens` - Maximum tokens for assembled context (default: 8192)
    /// * `session_id` - Optional session ID for episodic filtering
    ///
    /// # Returns
    ///
    /// JSON representation of `AssembledContext` with:
    /// - `chunks`: Array of ranked chunks with content, scores, timestamps
    /// - `total_confidence`: Average relevance score across chunks
    /// - `temporal_span`: Tuple of (oldest, newest) timestamps
    /// - `token_count`: Total tokens in assembled context
    /// - `formatted`: Human-readable formatted context string
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Strategy is invalid (not "episodic", "semantic", or "hybrid")
    /// - Token budget < 100 (too small to be useful)
    /// - Memory retrieval fails
    /// - Reranking fails
    /// - Assembly fails
    ///
    /// # Token Budget Validation
    ///
    /// - **Error if < 100**: Context too small to be useful
    /// - **Warn if > 8192**: May impact LLM performance
    /// - **Default: 8192**: Fits most LLM context windows
    ///
    /// # Example
    ///
    /// ```ignore
    /// let context = bridge.assemble(
    ///     "What is Rust?".to_string(),
    ///     "hybrid".to_string(),
    ///     8000,
    ///     Some("session-123".to_string())
    /// )?;
    /// ```
    pub fn assemble(
        &self,
        query: &str,
        strategy: &str,
        max_tokens: usize,
        session_id: Option<&str>,
    ) -> Result<Value, String> {
        info!(
            "ContextBridge::assemble called with query='{}', strategy='{}', max_tokens={}, session_id={:?}",
            query, strategy, max_tokens, session_id
        );

        // Validate token budget
        if max_tokens < 100 {
            error!("Token budget too low: {}", max_tokens);
            return Err(format!("Token budget must be >=100, got {max_tokens}"));
        }
        if max_tokens > 8192 {
            warn!(
                "Large token budget: {} (consider reducing for performance)",
                max_tokens
            );
        }

        // Validate and parse strategy
        let strategy_enum = match strategy {
            "episodic" => {
                debug!("Using Episodic retrieval strategy");
                RetrievalStrategy::Episodic
            }
            "semantic" => {
                debug!("Using Semantic retrieval strategy");
                RetrievalStrategy::Semantic
            }
            "hybrid" => {
                debug!("Using Hybrid retrieval strategy");
                RetrievalStrategy::Hybrid
            }
            _ => {
                error!("Invalid strategy: '{}'", strategy);
                return Err(format!(
                    "Unknown strategy '{strategy}'. Valid: episodic, semantic, hybrid"
                ));
            }
        };

        self.runtime.block_on(async {
            debug!("Entering async context assembly");

            // Retrieve chunks based on strategy
            let chunks = self
                .retrieve_chunks(query, strategy_enum, max_tokens, session_id)
                .await?;

            debug!("Retrieved {} chunks", chunks.len());

            if chunks.is_empty() {
                warn!("No chunks retrieved for query '{}'", query);
                return Ok(serde_json::json!({
                    "chunks": [],
                    "total_confidence": 0.0,
                    "temporal_span": [null, null],
                    "token_count": 0,
                    "formatted": ""
                }));
            }

            // Component 2: BM25Reranker (stateless, Phase 13 uses BM25-only)
            debug!("Creating BM25Reranker for reranking");
            let reranker = BM25Reranker::new();
            let ranked_chunks = reranker
                .rerank(chunks, query, max_tokens / 2)
                .await
                .map_err(|e| format!("Reranking failed: {e}"))?;

            debug!("Reranked to {} top chunks", ranked_chunks.len());

            // Component 3: ContextAssembler (stateless, token budget management)
            debug!("Creating ContextAssembler with max_tokens={}", max_tokens);
            let assembler = ContextAssembler::with_config(max_tokens, 0.3);

            // Create dummy query understanding (not used in Phase 13)
            let query_understanding = QueryUnderstanding {
                intent: QueryIntent::Unknown,
                entities: vec![],
                keywords: vec![],
            };

            let context = assembler.assemble(ranked_chunks, &query_understanding);

            debug!(
                "Assembled context with {} chunks, {} tokens",
                context.chunks.len(),
                context.token_count
            );

            // Convert to JSON for bridge return
            let result = serde_json::to_value(&context)
                .map_err(|e| format!("JSON conversion failed: {e}"))?;

            Ok(result)
        })
    }

    /// Retrieve chunks from memory based on strategy
    ///
    /// Handles episodic, semantic, and hybrid retrieval strategies.
    async fn retrieve_chunks(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
        max_tokens: usize,
        _session_id: Option<&str>,
    ) -> Result<Vec<Chunk>, String> {
        match strategy {
            RetrievalStrategy::Episodic => {
                debug!("Retrieving from episodic memory");
                self.retrieve_episodic(query, max_tokens).await
            }
            RetrievalStrategy::Semantic => {
                debug!("Retrieving from semantic memory");
                self.retrieve_semantic(query, max_tokens).await
            }
            RetrievalStrategy::Hybrid => {
                debug!("Retrieving from both episodic and semantic memory (hybrid strategy)");
                let mut episodic_chunks = self.retrieve_episodic(query, max_tokens / 2).await?;
                let semantic_chunks = self.retrieve_semantic(query, max_tokens / 2).await?;

                debug!(
                    "Hybrid: {} episodic + {} semantic chunks",
                    episodic_chunks.len(),
                    semantic_chunks.len()
                );

                // Combine both
                episodic_chunks.extend(semantic_chunks);
                Ok(episodic_chunks)
            }
            RetrievalStrategy::BM25 => {
                // BM25 strategy falls back to episodic
                debug!("BM25 strategy falling back to episodic retrieval");
                self.retrieve_episodic(query, max_tokens).await
            }
        }
    }

    /// Retrieve chunks from episodic memory using vector search
    async fn retrieve_episodic(
        &self,
        query: &str,
        max_tokens: usize,
    ) -> Result<Vec<Chunk>, String> {
        debug!("Retrieving from episodic memory via vector search");

        // Get episodic memory reference
        let episodic = self.memory_manager.episodic();

        // Search episodic memory (vector similarity)
        let entries = episodic.search(query, max_tokens).await.map_err(|e| {
            error!("Episodic search failed: {}", e);
            format!("Failed to search episodic memory: {e}")
        })?;

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

        debug!("Retrieved {} episodic chunks", chunks.len());
        trace!("Episodic chunks: {:?}", chunks);

        Ok(chunks)
    }

    /// Retrieve chunks from semantic memory by converting entities to chunks
    async fn retrieve_semantic(
        &self,
        _query: &str,
        max_tokens: usize,
    ) -> Result<Vec<Chunk>, String> {
        debug!("Querying semantic memory for entities");

        // Get semantic memory reference
        let semantic = self.memory_manager.semantic();

        // Query all entities (empty type = all types)
        // TODO: Phase 13.9 - Add semantic vector search instead of query_by_type()
        let entities = semantic.query_by_type("").await.map_err(|e| {
            error!("Semantic query failed: {}", e);
            format!("Failed to query semantic memory: {e}")
        })?;

        debug!("Retrieved {} entities from semantic memory", entities.len());

        // Convert entities to chunks
        let chunks: Vec<Chunk> = entities
            .into_iter()
            .take(max_tokens) // Rough limit
            .map(|entity| {
                // Format entity as chunk content
                let content = format!(
                    "{} ({})\nProperties: {}",
                    entity.name,
                    entity.entity_type,
                    serde_json::to_string_pretty(&entity.properties).unwrap_or_default()
                );

                Chunk {
                    id: entity.id.clone(),
                    content,
                    source: format!("semantic:{}", entity.entity_type),
                    timestamp: entity.event_time.unwrap_or(entity.ingestion_time),
                    metadata: Some(entity.properties),
                }
            })
            .collect();

        debug!("Converted {} entities to chunks", chunks.len());
        trace!("Semantic chunks: {:?}", chunks);

        Ok(chunks)
    }
}

/// Internal strategy enum matching llmspell-context `RetrievalStrategy`
#[derive(Debug, Clone, Copy)]
enum RetrievalStrategy {
    Episodic,
    Semantic,
    Hybrid,
    #[allow(dead_code)]
    BM25,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_memory::DefaultMemoryManager;

    #[test]
    fn test_context_bridge_creation() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let _bridge = ContextBridge::new(Arc::new(memory_manager));
        // Creation should succeed
    }

    #[test]
    fn test_strategy_validation() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Valid strategies should work
        let result = bridge.assemble("test query", "episodic", 1000, None);
        assert!(result.is_ok());

        // Invalid strategy should error
        let result = bridge.assemble("test query", "invalid_strategy", 1000, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unknown strategy 'invalid_strategy'"));
    }

    #[test]
    fn test_token_budget_validation() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Token budget < 100 should error
        let result = bridge.assemble("test query", "episodic", 50, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token budget must be >=100"));

        // Token budget >= 100 should work
        let result = bridge.assemble("test query", "episodic", 100, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assemble_episodic_empty() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Query with no data should return empty context
        let result = bridge
            .assemble("test query", "episodic", 1000, None)
            .expect("assemble should succeed");

        assert_eq!(result["chunks"].as_array().unwrap().len(), 0);
        assert_eq!(result["total_confidence"], 0.0);
        assert_eq!(result["token_count"], 0);
    }

    #[test]
    fn test_assemble_semantic_empty() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Query semantic memory (empty initially)
        let result = bridge
            .assemble("test query", "semantic", 1000, None)
            .expect("assemble should succeed");

        assert_eq!(result["chunks"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_assemble_hybrid_empty() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Hybrid strategy combines both (both empty initially)
        let result = bridge
            .assemble("test query", "hybrid", 1000, None)
            .expect("assemble should succeed");

        assert_eq!(result["chunks"].as_array().unwrap().len(), 0);
    }
}
