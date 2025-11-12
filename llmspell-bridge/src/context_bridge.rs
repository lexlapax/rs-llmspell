//! ABOUTME: Context bridge providing language-agnostic context assembly operations
//! ABOUTME: Composes `BM25Retriever` + `ContextAssembler` + `MemoryManager` for RAG workflows

use async_trait::async_trait;
use llmspell_context::{
    assembly::ContextAssembler as ContextAssemblerImpl,
    reranking::BM25Reranker,
    retrieval::{HybridRetriever, RetrievalWeights},
    traits::Reranker,
    types::{AssembledContext, Chunk, QueryIntent, QueryUnderstanding, RankedChunk},
};
use llmspell_core::ContextAssembler;
use llmspell_memory::MemoryManager;
use llmspell_rag::pipeline::RAGRetriever;
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
    /// Optional RAG pipeline for hybrid RAG+Memory retrieval
    rag_pipeline: Option<Arc<dyn RAGRetriever>>,
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
            rag_pipeline: None,
        }
    }

    /// Add RAG pipeline for hybrid RAG+Memory retrieval (builder pattern)
    ///
    /// # Arguments
    ///
    /// * `rag` - RAG pipeline implementing `RAGRetriever` trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use llmspell_rag::pipeline::SessionRAGAdapter;
    /// use std::sync::Arc;
    ///
    /// let bridge = ContextBridge::new(memory)
    ///     .with_rag_pipeline(Arc::new(SessionRAGAdapter::new(rag_pipeline)));
    /// ```
    #[must_use]
    pub fn with_rag_pipeline(mut self, rag: Arc<dyn RAGRetriever>) -> Self {
        info!("Adding RAG pipeline to ContextBridge");
        self.rag_pipeline = Some(rag);
        self
    }

    /// Assemble context from memory using specified retrieval strategy
    ///
    /// Composes retrieval → reranking → assembly pipeline based on strategy.
    ///
    /// # Arguments
    ///
    /// * `query` - Query string
    /// * `strategy` - Retrieval strategy: "episodic", "semantic", "hybrid", or "rag" (case-sensitive)
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
    /// - Strategy is invalid (not "episodic", "semantic", "hybrid", or "rag")
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
    pub async fn assemble(
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

        Self::validate_token_budget(max_tokens)?;
        let strategy_enum = Self::parse_strategy(strategy)?;

        self.assemble_context_async(query, strategy_enum, max_tokens, session_id)
    }

    /// Validate token budget constraints
    fn validate_token_budget(max_tokens: usize) -> Result<(), String> {
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
        Ok(())
    }

    /// Parse and validate retrieval strategy string
    fn parse_strategy(strategy: &str) -> Result<RetrievalStrategy, String> {
        let result = match strategy {
            "episodic" => Ok(RetrievalStrategy::Episodic),
            "semantic" => Ok(RetrievalStrategy::Semantic),
            "hybrid" => Ok(RetrievalStrategy::Hybrid),
            "rag" => Ok(RetrievalStrategy::Rag),
            _ => {
                error!("Invalid strategy: '{}'", strategy);
                Err(format!(
                    "Unknown strategy '{strategy}'. Valid: episodic, semantic, hybrid, rag"
                ))
            }
        };

        if let Ok(ref strat) = result {
            debug!("Using {:?} retrieval strategy", strat);
        }

        result
    }

    /// Async context assembly pipeline
    async fn assemble_context_async(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
        max_tokens: usize,
        session_id: Option<&str>,
    ) -> Result<Value, String> {
        debug!("Entering async context assembly");

        let chunks = self
            .retrieve_chunks(query, strategy, max_tokens, session_id)?;

        debug!("Retrieved {} chunks", chunks.len());

        if chunks.is_empty() {
            return Ok(Self::create_empty_context(query));
        }

        self.rerank_and_assemble(chunks, query, max_tokens).await
    }

    /// Create empty context response
    fn create_empty_context(query: &str) -> Value {
        warn!("No chunks retrieved for query '{}'", query);
        serde_json::json!({
            "chunks": [],
            "total_confidence": 0.0,
            "temporal_span": [null, null],
            "token_count": 0,
            "formatted": ""
        })
    }

    /// Rerank chunks and assemble final context
    async fn rerank_and_assemble(
        &self,
        chunks: Vec<Chunk>,
        query: &str,
        max_tokens: usize,
    ) -> Result<Value, String> {
        let ranked_chunks = Self::rerank_chunks(chunks, query, max_tokens).await?;
        let context = Self::assemble_final_context(ranked_chunks, max_tokens);
        serde_json::to_value(&context).map_err(|e| format!("JSON conversion failed: {e}"))
    }

    /// Rerank chunks using BM25
    async fn rerank_chunks(
        chunks: Vec<Chunk>,
        query: &str,
        max_tokens: usize,
    ) -> Result<Vec<RankedChunk>, String> {
        debug!("Creating BM25Reranker for reranking");
        let reranker = BM25Reranker::new();
        let ranked_chunks = reranker
            .rerank(chunks, query, max_tokens / 2)
            .await
            .map_err(|e| format!("Reranking failed: {e}"))?;

        debug!("Reranked to {} top chunks", ranked_chunks.len());
        Ok(ranked_chunks)
    }

    /// Assemble final context from ranked chunks
    fn assemble_final_context(
        ranked_chunks: Vec<RankedChunk>,
        max_tokens: usize,
    ) -> AssembledContext {
        debug!("Creating ContextAssembler with max_tokens={}", max_tokens);
        let assembler = ContextAssemblerImpl::with_config(max_tokens, 0.3);

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

        context
    }

    /// Retrieve chunks from memory based on strategy
    ///
    /// Handles episodic, semantic, hybrid, and RAG retrieval strategies.
    async fn retrieve_chunks(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
        max_tokens: usize,
        session_id: Option<&str>,
    ) -> Result<Vec<Chunk>, String> {
        debug!("Retrieving chunks using {:?} strategy", strategy);

        match strategy {
            RetrievalStrategy::Episodic | RetrievalStrategy::BM25 => {
                self.retrieve_episodic(query, max_tokens).await
            }
            RetrievalStrategy::Semantic => self.retrieve_semantic(query, max_tokens).await,
            RetrievalStrategy::Hybrid => self.retrieve_hybrid_memory_only(query, max_tokens).await,
            RetrievalStrategy::Rag => {
                self.retrieve_rag_hybrid(query, max_tokens, session_id)
                    .await
            }
        }
    }

    /// Retrieve chunks using hybrid strategy (episodic + semantic, memory-only)
    ///
    /// **Performance Optimization**: Parallel retrieval from both sources using `tokio::join!`
    /// to reduce latency (Task 13.14.4).
    async fn retrieve_hybrid_memory_only(
        &self,
        query: &str,
        max_tokens: usize,
    ) -> Result<Vec<Chunk>, String> {
        debug!("Retrieving from both episodic and semantic memory (hybrid memory-only strategy, parallel)");

        // Parallel retrieval from both sources (Task 13.14.4 optimization)
        let (episodic_result, semantic_result) = tokio::join!(
            self.retrieve_episodic(query, max_tokens / 2),
            self.retrieve_semantic(query, max_tokens / 2)
        );

        let mut episodic_chunks = episodic_result?;
        let semantic_chunks = semantic_result?;

        debug!(
            "Hybrid memory-only (parallel): {} episodic + {} semantic chunks",
            episodic_chunks.len(),
            semantic_chunks.len()
        );

        episodic_chunks.extend(semantic_chunks);
        Ok(episodic_chunks)
    }

    /// Retrieve chunks using RAG+Memory hybrid strategy via `HybridRetriever`
    ///
    /// Uses `HybridRetriever` from llmspell-context to combine RAG vector search
    /// with episodic memory. Falls back to memory-only hybrid if RAG pipeline not configured.
    #[allow(clippy::cognitive_complexity)]
    async fn retrieve_rag_hybrid(
        &self,
        query: &str,
        max_tokens: usize,
        session_id: Option<&str>,
    ) -> Result<Vec<Chunk>, String> {
        if let Some(rag) = &self.rag_pipeline {
            info!("Using hybrid RAG+Memory retrieval");
            let session = session_id.unwrap_or("default");
            debug!(
                "Creating HybridRetriever for query='{}', max_tokens={}, session_id={}",
                query, max_tokens, session
            );

            // Create HybridRetriever with default weights (memory-focused: 40/60)
            let hybrid = HybridRetriever::new(
                Some(rag.clone()),
                self.memory_manager.clone(),
                RetrievalWeights::default(),
            );

            // Retrieve hybrid results
            let ranked_chunks = hybrid
                .retrieve_hybrid(query, session, max_tokens)
                .await
                .map_err(|e| format!("Hybrid retrieval failed: {e}"))?;

            // Convert RankedChunk to Chunk (already ranked, skip reranking)
            let chunks: Vec<Chunk> = ranked_chunks
                .into_iter()
                .map(|rc| Chunk {
                    id: rc.chunk.id,
                    content: rc.chunk.content,
                    source: rc.chunk.source,
                    timestamp: rc.chunk.timestamp,
                    metadata: rc.chunk.metadata,
                })
                .collect();

            debug!("Retrieved {} chunks from RAG+Memory hybrid", chunks.len());
            Ok(chunks)
        } else {
            warn!(
                "RAG strategy requested but no RAG pipeline configured, falling back to hybrid memory"
            );
            // Fallback to memory-only hybrid
            self.retrieve_hybrid_memory_only(query, max_tokens).await
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
        let entities = Self::query_entities(&self.memory_manager).await?;
        let chunks = Self::convert_entities_to_chunks(entities, max_tokens);
        Ok(chunks)
    }

    /// Query all entities from semantic memory
    async fn query_entities(
        memory_manager: &Arc<dyn MemoryManager>,
    ) -> Result<Vec<llmspell_memory::Entity>, String> {
        debug!("Querying semantic memory for entities");

        let semantic = memory_manager.semantic();

        // TODO: Phase 13.9 - Add semantic vector search instead of query_by_type()
        let entities = semantic.query_by_type("").await.map_err(|e| {
            error!("Semantic query failed: {}", e);
            format!("Failed to query semantic memory: {e}")
        })?;

        debug!("Retrieved {} entities from semantic memory", entities.len());
        Ok(entities)
    }

    /// Convert entities to chunks
    fn convert_entities_to_chunks(
        entities: Vec<llmspell_memory::Entity>,
        max_tokens: usize,
    ) -> Vec<Chunk> {
        let chunks: Vec<Chunk> = entities
            .into_iter()
            .take(max_tokens)
            .map(Self::entity_to_chunk)
            .collect();

        debug!("Converted {} entities to chunks", chunks.len());
        trace!("Semantic chunks: {:?}", chunks);

        chunks
    }

    /// Quick test query with hybrid strategy and 2000 token budget
    ///
    /// Convenience method for testing context retrieval without specifying strategy/budget.
    ///
    /// # Arguments
    ///
    /// * `query` - Query string
    /// * `session_id` - Optional session ID for episodic filtering
    ///
    /// # Returns
    ///
    /// JSON representation of `AssembledContext` (same as `assemble()`)
    ///
    /// # Errors
    ///
    /// Returns error if context assembly fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let context = bridge.test_query("test query", Some("session-123"))?;
    /// ```
    pub async fn test_query(&self, query: &str, session_id: Option<&str>) -> Result<Value, String> {
        debug!("ContextBridge::test_query called with query='{}'", query);
        // Quick test with hybrid strategy, 2000 tokens
        self.assemble(query, "hybrid", 2000, session_id).await
    }

    /// Get strategy statistics from memory manager
    ///
    /// Returns counts of episodic and semantic records, plus list of available strategies.
    ///
    /// # Returns
    ///
    /// JSON object with:
    /// - `episodic_count`: Number of episodic memory entries
    /// - `semantic_count`: Number of semantic entities
    /// - `strategies`: Array of available strategies `["episodic", "semantic", "hybrid"]`
    ///
    /// # Errors
    ///
    /// Returns error if memory manager queries fail
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = bridge.get_strategy_stats()?;
    /// println!("Episodic: {}", stats["episodic_count"]);
    /// ```
    pub async fn get_strategy_stats(&self) -> Result<Value, String> {
        debug!("ContextBridge::get_strategy_stats called");
        debug!("Entering async get_strategy_stats");

        // Get episodic count (same pattern as MemoryBridge::stats)
        let episodic_count = self
            .memory_manager
            .episodic()
            .search("", 10000)
            .await
            .map(|entries| entries.len())
            .unwrap_or(0);

        // Get semantic count
        let semantic_count = self
            .memory_manager
            .semantic()
            .query_by_type("")
            .await
            .map(|entities| entities.len())
            .unwrap_or(0);

        debug!(
            "get_strategy_stats: episodic={}, semantic={}",
            episodic_count, semantic_count
        );

        let stats = serde_json::json!({
            "episodic_count": episodic_count,
            "semantic_count": semantic_count,
            "strategies": ["episodic", "semantic", "hybrid", "rag"]
        });

        Ok(stats)
    }

    /// Convert semantic entity to chunk
    fn entity_to_chunk(entity: llmspell_memory::Entity) -> Chunk {
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
    }
}

/// Internal strategy enum matching llmspell-context `RetrievalStrategy`
#[derive(Debug, Clone, Copy)]
enum RetrievalStrategy {
    Episodic,
    Semantic,
    Hybrid,
    Rag,
    #[allow(dead_code)]
    BM25,
}

// ============================================================================
// ContextAssembler Trait Implementation (Task 13.11.1a)
// ============================================================================

/// Implement `ContextAssembler` trait from llmspell-core for `ContextBridge`
///
/// This enables type-safe context assembly without circular dependencies:
/// - llmspell-core defines the trait
/// - llmspell-bridge implements the trait
/// - llmspell-templates depends on the trait (not the implementation)
///
/// This follows Dependency Inversion Principle - depend on abstractions, not concretions.
#[async_trait]
impl ContextAssembler for ContextBridge {
    /// Assemble context from memory using specified retrieval strategy
    ///
    /// Delegates to existing `ContextBridge::assemble()` implementation.
    /// The trait method signature matches the existing method, so this is a simple forwarding impl.
    async fn assemble(
        &self,
        query: &str,
        strategy: &str,
        max_tokens: usize,
        session_id: Option<&str>,
    ) -> Result<Value, String> {
        // Forward to existing implementation
        // Note: This calls the inherent method on ContextBridge, not the trait method
        // Rust resolves to inherent methods first, avoiding infinite recursion
        self.assemble(query, strategy, max_tokens, session_id).await
    }
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
        let result = runtime.block_on(bridge.assemble("test query", "episodic", 1000, None));
        assert!(result.is_ok());

        // Invalid strategy should error
        let result =
            runtime.block_on(bridge.assemble("test query", "invalid_strategy", 1000, None));
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
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Token budget < 100 should error
        let result = runtime.block_on(bridge.assemble("test query", "episodic", 50, None));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token budget must be >=100"));

        // Token budget >= 100 should work
        let result = runtime.block_on(bridge.assemble("test query", "episodic", 100, None));
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
        let result = runtime
            .block_on(bridge.assemble("test query", "episodic", 1000, None))
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
                .expect("Failed to create memory manager")
        });

        let bridge = ContextBridge::new(Arc::new(memory_manager));

        // Query semantic memory (empty initially)
        let result = runtime
            .block_on(bridge.assemble("test query", "semantic", 1000, None))
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
        let result = runtime
            .block_on(bridge.assemble("test query", "hybrid", 1000, None))
            .expect("assemble should succeed");

        assert_eq!(result["chunks"].as_array().unwrap().len(), 0);
    }
}
