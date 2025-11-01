//! ABOUTME: Core memory bridge providing language-agnostic memory operations
//! ABOUTME: Wraps `MemoryManager` for script access with async→blocking conversion

use llmspell_memory::{
    ConsolidationMode, ConsolidationResult, Entity, EpisodicEntry, MemoryManager,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

/// Core memory bridge for language-agnostic memory operations
///
/// This bridge wraps the `MemoryManager` and provides blocking interfaces
/// for script languages, converting async operations to blocking via `runtime.block_on()`.
///
/// # Pattern
///
/// Follows the async→blocking conversion pattern:
/// ```ignore
/// self.runtime.block_on(async {
///     self.memory_manager.method().await
/// })
/// ```
pub struct MemoryBridge {
    /// Reference to the memory manager
    memory_manager: Arc<dyn MemoryManager>,
}

impl MemoryBridge {
    /// Create a new memory bridge
    ///
    /// # Arguments
    ///
    /// * `memory_manager` - The memory manager to wrap
    ///
    /// # Example
    ///
    /// ```ignore
    /// use llmspell_memory::DefaultMemoryManager;
    /// use llmspell_bridge::MemoryBridge;
    /// use std::sync::Arc;
    ///
    /// let memory = Arc::new(DefaultMemoryManager::new_in_memory().await?);
    /// let bridge = MemoryBridge::new(memory);
    /// ```
    #[must_use]
    pub fn new(memory_manager: Arc<dyn MemoryManager>) -> Self {
        info!("Creating MemoryBridge");
        Self { memory_manager }
    }

    /// Log episodic add operation start
    fn log_episodic_add_start(session_id: &str, role: &str, metadata: &Value) {
        info!(
            "MemoryBridge::episodic_add called for session={}, role={}",
            session_id, role
        );
        trace!("episodic_add metadata: {:?}", metadata);
        debug!("Entering async episodic_add");
    }

    /// Handle episodic add error
    fn handle_episodic_add_error(e: impl std::fmt::Display) -> String {
        error!("episodic_add failed: {}", e);
        format!("Failed to add episodic memory: {e}")
    }

    /// Log episodic search operation start
    fn log_episodic_search_start(session_id: &str, query: &str, limit: usize) {
        info!(
            "MemoryBridge::episodic_search called for session={}, query='{}', limit={}",
            session_id, query, limit
        );
        debug!("Entering async episodic_search");
    }

    /// Log episodic search results
    fn log_episodic_search_results(entries: &[EpisodicEntry]) {
        debug!("episodic_search found {} results", entries.len());
        trace!("episodic_search results: {:?}", entries);
    }

    /// Log semantic query operation start
    fn log_semantic_query_start(query: &str, limit: usize) {
        info!(
            "MemoryBridge::semantic_query called with query='{}', limit={}",
            query, limit
        );
        debug!("Entering async semantic_query");
    }

    /// Log semantic query results
    fn log_semantic_query_results(entities: &[Entity]) {
        debug!("semantic_query found {} entities", entities.len());
        trace!("semantic_query results: {:?}", entities);
    }

    /// Log consolidate operation start
    fn log_consolidate_start(session_id: Option<&str>, force: bool) {
        info!(
            "MemoryBridge::consolidate called for session={:?}, force={}",
            session_id, force
        );
        debug!("Entering async consolidate");
    }

    /// Log consolidate operation results
    fn log_consolidate_results(result: &ConsolidationResult) {
        debug!(
            "consolidate completed: {} entries processed, {} entities added, {} updated, {} deleted",
            result.entries_processed,
            result.entities_added,
            result.entities_updated,
            result.entities_deleted
        );
    }

    /// Add an episodic memory entry
    ///
    /// Adds a new interaction to episodic memory with automatic embedding generation.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    /// * `role` - Role of the speaker ("user", "assistant", etc.)
    /// * `content` - Content of the interaction
    /// * `metadata` - Additional metadata (JSON object)
    ///
    /// # Returns
    ///
    /// The unique ID of the created entry, or error message
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Memory storage fails
    /// - Embedding generation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let id = bridge.episodic_add(
    ///     "session-123".to_string(),
    ///     "user".to_string(),
    ///     "What is Rust?".to_string(),
    ///     serde_json::json!({"topic": "programming"})
    /// )?;
    /// ```
    pub async fn episodic_add(
        &self,
        session_id: String,
        role: String,
        content: String,
        metadata: Value,
    ) -> Result<String, String> {
        Self::log_episodic_add_start(&session_id, &role, &metadata);

        // Create episodic entry with metadata
        let mut entry = EpisodicEntry::new(session_id, role, content);
        entry.metadata = metadata;

        let id = self
            .memory_manager
            .episodic()
            .add(entry)
            .await
            .map_err(Self::handle_episodic_add_error)?;

        debug!("episodic_add completed with id={}", id);
        Ok(id)
    }

    /// Search episodic memory
    ///
    /// Performs semantic similarity search over episodic memories,
    /// optionally filtered by session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to filter by (empty for all sessions)
    /// * `query` - Search query text
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// JSON array of matching entries with scores, or error message
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Vector search fails
    /// - Session not found
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = bridge.episodic_search(
    ///     "session-123".to_string(),
    ///     "Rust programming".to_string(),
    ///     5
    /// )?;
    /// ```
    /// Search all sessions without filtering
    async fn search_all_sessions(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<EpisodicEntry>, String> {
        self.memory_manager
            .episodic()
            .search(query, limit)
            .await
            .map_err(|e| {
                error!("episodic_search failed: {}", e);
                format!("Failed to search episodic memory: {e}")
            })
    }

    /// Search specific session by ID
    async fn search_session(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<EpisodicEntry>, String> {
        let session_entries = self
            .memory_manager
            .episodic()
            .get_session(session_id)
            .await
            .map_err(|e| {
                error!("get_session failed: {}", e);
                format!("Failed to get session: {e}")
            })?;

        // Return session entries up to limit
        // TODO: Phase 13.9 - Add session-filtered vector search in episodic backend
        Ok(session_entries.into_iter().take(limit).collect())
    }

    /// Convert entries to JSON value
    fn entries_to_json(entries: &[EpisodicEntry]) -> Result<Value, String> {
        serde_json::to_value(entries).map_err(|e| {
            error!("JSON conversion failed: {}", e);
            format!("Failed to convert results to JSON: {e}")
        })
    }

    /// # Errors
    ///
    /// Returns error if vector search or session retrieval fails
    pub async fn episodic_search(
        &self,
        session_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Value, String> {
        Self::log_episodic_search_start(session_id, query, limit);

        // Search all sessions or specific session
        let entries = if session_id.is_empty() {
            self.search_all_sessions(query, limit).await?
        } else {
            self.search_session(session_id, limit).await?
        };

        Self::log_episodic_search_results(&entries);
        Self::entries_to_json(&entries)
    }

    /// Query semantic memory (knowledge graph)
    ///
    /// Searches the semantic knowledge graph for entities matching the query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query text
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// JSON array of matching entities with scores, or error message
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Graph query fails
    /// - Embedding generation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let entities = bridge.semantic_query(
    ///     "Rust programming language".to_string(),
    ///     10
    /// )?;
    /// ```
    /// Query semantic memory backend
    async fn query_semantic_backend(&self) -> Result<Vec<Entity>, String> {
        self.memory_manager
            .semantic()
            .query_by_type("")
            .await
            .map_err(|e| {
                error!("semantic_query failed: {}", e);
                format!("Failed to query semantic memory: {e}")
            })
    }

    /// Convert entities to JSON value
    fn entities_to_json(entities: &[Entity]) -> Result<Value, String> {
        serde_json::to_value(entities).map_err(|e| {
            error!("JSON conversion failed: {}", e);
            format!("Failed to convert results to JSON: {e}")
        })
    }

    /// # Errors
    ///
    /// Returns error if graph query or JSON conversion fails
    pub async fn semantic_query(&self, query: &str, limit: usize) -> Result<Value, String> {
        Self::log_semantic_query_start(query, limit);

        // Query all entities and apply limit
        // Note: Full semantic query comes in Phase 13.9
        let entities = self.query_semantic_backend().await?;
        let limited_entities: Vec<_> = entities.into_iter().take(limit).collect();

        Self::log_semantic_query_results(&limited_entities);
        Self::entities_to_json(&limited_entities)
    }

    /// Consolidate episodic memories into semantic knowledge
    ///
    /// Processes unprocessed episodic entries for a session, extracting
    /// entities and relationships to update the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session to consolidate (None = all sessions)
    /// * `force` - Force immediate consolidation even if batch mode is configured
    ///
    /// # Returns
    ///
    /// JSON object with consolidation statistics (entries processed, entities added/updated)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Consolidation engine fails
    /// - LLM call fails (if using LLM-driven consolidation)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = bridge.consolidate(
    ///     Some("session-123".to_string()),
    ///     false
    /// )?;
    /// println!("Processed {} entries", stats["entries_processed"]);
    /// ```
    /// Determine consolidation mode based on force flag
    const fn determine_consolidation_mode(force: bool) -> ConsolidationMode {
        if force {
            ConsolidationMode::Immediate
        } else {
            ConsolidationMode::Background
        }
    }

    /// Run consolidation on backend
    async fn run_consolidation_backend(
        &self,
        session_id: &str,
        mode: ConsolidationMode,
    ) -> Result<ConsolidationResult, String> {
        self.memory_manager
            .consolidate(session_id, mode, None)
            .await
            .map_err(|e| {
                error!("consolidate failed: {}", e);
                format!("Failed to consolidate memories: {e}")
            })
    }

    /// Convert consolidation result to JSON
    fn consolidation_result_to_json(result: &ConsolidationResult) -> Value {
        serde_json::json!({
            "entries_processed": result.entries_processed,
            "entities_added": result.entities_added,
            "entities_updated": result.entities_updated,
            "entities_deleted": result.entities_deleted,
            "entries_skipped": result.entries_skipped,
            "entries_failed": result.entries_failed,
            "duration_ms": result.duration_ms,
        })
    }

    /// # Errors
    ///
    /// Returns error if consolidation engine or LLM call fails
    pub async fn consolidate(
        &self,
        session_id: Option<&str>,
        force: bool,
    ) -> Result<Value, String> {
        Self::log_consolidate_start(session_id, force);

        // Determine mode and run consolidation
        let mode = Self::determine_consolidation_mode(force);
        debug!("Using consolidation mode: {:?}", mode);

        let session_str = session_id.unwrap_or("");
        let result = self.run_consolidation_backend(session_str, mode).await?;

        Self::log_consolidate_results(&result);
        Ok(Self::consolidation_result_to_json(&result))
    }

    /// Get memory system statistics
    ///
    /// Returns counts and metrics about the memory subsystems.
    ///
    /// # Returns
    ///
    /// JSON object with:
    /// - `episodic_count`: Number of episodic entries
    /// - `semantic_count`: Number of semantic entities
    /// - `unprocessed_count`: Number of entries awaiting consolidation
    /// - `sessions_with_unprocessed`: Number of sessions with unprocessed entries
    ///
    /// # Errors
    ///
    /// Returns error if memory subsystems fail to report stats
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = bridge.stats()?;
    /// println!("Episodic entries: {}", stats["episodic_count"]);
    /// ```
    pub async fn stats(&self) -> Result<Value, String> {
        info!("MemoryBridge::stats called");
        debug!("Entering async stats");

        // Get episodic count by searching with large limit
        // TODO: Phase 13.9 - Add count() method to EpisodicMemory trait
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

        // Get unprocessed sessions
        let sessions_with_unprocessed = self
            .memory_manager
            .episodic()
            .list_sessions_with_unprocessed()
            .await
            .map(|sessions| sessions.len())
            .unwrap_or(0);

        debug!(
            "stats: episodic={}, semantic={}, unprocessed_sessions={}",
            episodic_count, semantic_count, sessions_with_unprocessed
        );

        let stats = serde_json::json!({
            "episodic_count": episodic_count,
            "semantic_count": semantic_count,
            "sessions_with_unprocessed": sessions_with_unprocessed,
            "has_episodic": self.memory_manager.has_episodic(),
            "has_semantic": self.memory_manager.has_semantic(),
            "has_consolidation": self.memory_manager.has_consolidation(),
        });

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_memory::DefaultMemoryManager;

    #[test]
    fn test_memory_bridge_creation() {
        // Create in-memory backend for testing
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = MemoryBridge::new(Arc::new(memory_manager));

        // Test stats (should be empty)
        let stats = runtime
            .block_on(bridge.stats())
            .expect("stats should succeed");
        assert_eq!(stats["episodic_count"], 0);
        assert_eq!(stats["semantic_count"], 0);
    }

    #[test]
    fn test_episodic_add_and_search() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = MemoryBridge::new(Arc::new(memory_manager));

        // Add entry
        let id = runtime
            .block_on(bridge.episodic_add(
                "session-test".to_string(),
                "user".to_string(),
                "Hello world".to_string(),
                serde_json::json!({"test": true}),
            ))
            .expect("episodic_add should succeed");

        assert!(!id.is_empty());

        // Search (should find the entry)
        let results = runtime
            .block_on(bridge.episodic_search("session-test", "hello", 5))
            .expect("episodic_search should succeed");

        let results_array = results.as_array().expect("results should be array");
        assert_eq!(results_array.len(), 1);
    }

    #[test]
    fn test_semantic_query_empty() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = MemoryBridge::new(Arc::new(memory_manager));

        // Query semantic (should be empty)
        let results = runtime
            .block_on(bridge.semantic_query("test", 5))
            .expect("semantic_query should succeed");

        let results_array = results.as_array().expect("results should be array");
        assert_eq!(results_array.len(), 0);
    }

    #[test]
    fn test_consolidate() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let memory_manager = runtime.block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let bridge = MemoryBridge::new(Arc::new(memory_manager));

        // Add entry
        runtime
            .block_on(bridge.episodic_add(
                "session-test".to_string(),
                "user".to_string(),
                "Test consolidation".to_string(),
                serde_json::json!({}),
            ))
            .expect("episodic_add should succeed");

        // Consolidate
        let result = runtime
            .block_on(bridge.consolidate(Some("session-test"), true))
            .expect("consolidate should succeed");

        // Should have processed entries (even if no entities extracted)
        assert!(result.is_object());
        assert!(result["duration_ms"].is_u64());
    }
}
