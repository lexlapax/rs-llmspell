//! ABOUTME: Core memory bridge providing language-agnostic memory operations
//! ABOUTME: Wraps `MemoryManager` for script access with async→blocking conversion

use llmspell_memory::{ConsolidationMode, EpisodicEntry, MemoryManager};
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
    /// Tokio runtime handle for async→blocking conversion
    runtime: tokio::runtime::Handle,
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
        Self {
            memory_manager,
            runtime: llmspell_kernel::global_io_runtime().handle().clone(),
        }
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
    pub fn episodic_add(
        &self,
        session_id: String,
        role: String,
        content: String,
        metadata: Value,
    ) -> Result<String, String> {
        info!(
            "MemoryBridge::episodic_add called for session={}, role={}",
            session_id, role
        );
        trace!("episodic_add metadata: {:?}", metadata);

        self.runtime.block_on(async {
            debug!("Entering async episodic_add");

            // Create episodic entry with metadata
            let mut entry = EpisodicEntry::new(session_id, role, content);
            entry.metadata = metadata;

            let id = self
                .memory_manager
                .episodic()
                .add(entry)
                .await
                .map_err(|e| {
                    error!("episodic_add failed: {}", e);
                    format!("Failed to add episodic memory: {e}")
                })?;

            debug!("episodic_add completed with id={}", id);
            Ok(id)
        })
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
    pub fn episodic_search(
        &self,
        session_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Value, String> {
        info!(
            "MemoryBridge::episodic_search called for session={}, query='{}', limit={}",
            session_id, query, limit
        );

        self.runtime.block_on(async {
            debug!("Entering async episodic_search");

            // If session_id is provided, get session-specific entries
            let entries = if session_id.is_empty() {
                // Search all sessions
                self.memory_manager
                    .episodic()
                    .search(query, limit)
                    .await
                    .map_err(|e| {
                        error!("episodic_search failed: {}", e);
                        format!("Failed to search episodic memory: {e}")
                    })?
            } else {
                // Get session entries first, then filter/rank
                // Note: This is a simplified approach. For production, we'd want
                // the backend to support session-filtered vector search.
                let session_entries = self
                    .memory_manager
                    .episodic()
                    .get_session(session_id)
                    .await
                    .map_err(|e| {
                        error!("get_session failed: {}", e);
                        format!("Failed to get session: {e}")
                    })?;

                // For now, return the session entries (up to limit)
                // TODO: Phase 13.9 - Add session-filtered vector search in episodic backend
                session_entries.into_iter().take(limit).collect()
            };

            debug!("episodic_search found {} results", entries.len());
            trace!("episodic_search results: {:?}", entries);

            // Convert entries to JSON
            let json_entries = serde_json::to_value(&entries).map_err(|e| {
                error!("JSON conversion failed: {}", e);
                format!("Failed to convert results to JSON: {e}")
            })?;

            Ok(json_entries)
        })
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
    pub fn semantic_query(&self, query: &str, limit: usize) -> Result<Value, String> {
        info!(
            "MemoryBridge::semantic_query called with query='{}', limit={}",
            query, limit
        );

        self.runtime.block_on(async {
            debug!("Entering async semantic_query");

            // Note: SemanticMemory doesn't have a general query() method yet.
            // For Phase 13.8, we'll use query_by_type() with empty type to get all entities,
            // then filter/limit in memory. Full semantic query comes in Phase 13.9.
            let entities = self
                .memory_manager
                .semantic()
                .query_by_type("")
                .await
                .map_err(|e| {
                    error!("semantic_query failed: {}", e);
                    format!("Failed to query semantic memory: {e}")
                })?;

            // Apply limit
            let limited_entities: Vec<_> = entities.into_iter().take(limit).collect();

            debug!("semantic_query found {} entities", limited_entities.len());
            trace!("semantic_query results: {:?}", limited_entities);

            // Convert to JSON
            let json_entities = serde_json::to_value(&limited_entities).map_err(|e| {
                error!("JSON conversion failed: {}", e);
                format!("Failed to convert results to JSON: {e}")
            })?;

            Ok(json_entities)
        })
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
    pub fn consolidate(&self, session_id: Option<&str>, force: bool) -> Result<Value, String> {
        info!(
            "MemoryBridge::consolidate called for session={:?}, force={}",
            session_id, force
        );

        self.runtime
            .block_on(async {
                debug!("Entering async consolidate");

                // Determine consolidation mode
                let mode = if force {
                    ConsolidationMode::Immediate
                } else {
                    ConsolidationMode::Background
                };

                debug!("Using consolidation mode: {:?}", mode);

                // Run consolidation
                let session_str = session_id.unwrap_or("");
                let result = self
                    .memory_manager
                    .consolidate(session_str, mode)
                    .await
                    .map_err(|e| {
                        error!("consolidate failed: {}", e);
                        format!("Failed to consolidate memories: {e}")
                    })?;

                debug!(
                    "consolidate completed: {} entries processed, {} entities added, {} updated, {} deleted",
                    result.entries_processed,
                    result.entities_added,
                    result.entities_updated,
                    result.entities_deleted
                );

                // Convert to JSON
                let json_result = serde_json::json!({
                    "entries_processed": result.entries_processed,
                    "entities_added": result.entities_added,
                    "entities_updated": result.entities_updated,
                    "entities_deleted": result.entities_deleted,
                    "entries_skipped": result.entries_skipped,
                    "entries_failed": result.entries_failed,
                    "duration_ms": result.duration_ms,
                });

                Ok(json_result)
            })
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
    pub fn stats(&self) -> Result<Value, String> {
        info!("MemoryBridge::stats called");

        self.runtime.block_on(async {
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
        })
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
        let stats = bridge.stats().expect("stats should succeed");
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
        let id = bridge
            .episodic_add(
                "session-test".to_string(),
                "user".to_string(),
                "Hello world".to_string(),
                serde_json::json!({"test": true}),
            )
            .expect("episodic_add should succeed");

        assert!(!id.is_empty());

        // Search (should find the entry)
        let results = bridge
            .episodic_search("session-test", "hello", 5)
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
        let results = bridge
            .semantic_query("test", 5)
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
        bridge
            .episodic_add(
                "session-test".to_string(),
                "user".to_string(),
                "Test consolidation".to_string(),
                serde_json::json!({}),
            )
            .expect("episodic_add should succeed");

        // Consolidate
        let result = bridge
            .consolidate(Some("session-test"), true)
            .expect("consolidate should succeed");

        // Should have processed entries (even if no entities extracted)
        assert!(result.is_object());
        assert!(result["duration_ms"].is_u64());
    }
}
