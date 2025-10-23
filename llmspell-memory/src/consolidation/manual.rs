//! Manual consolidation engine with regex-based extraction

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use llmspell_graph::extraction::RegexExtractor;
use llmspell_graph::traits::KnowledgeGraph;
use llmspell_graph::types::{Entity, Relationship};
use tracing::{debug, info};

use super::ConsolidationEngine;
use crate::error::Result;
use crate::types::{ConsolidationResult, EpisodicEntry};

/// Manual consolidation engine with regex-based entity extraction
///
/// Extracts entities and relationships from episodic content using pattern matching,
/// then adds them to the knowledge graph. Suitable for testing and development.
///
/// # Architecture
///
/// ```text
/// EpisodicEntry.content → RegexExtractor → (Entities, Relationships) → KnowledgeGraph
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory::consolidation::ManualConsolidationEngine;
/// use llmspell_graph::extraction::RegexExtractor;
/// use llmspell_graph::storage::surrealdb::SurrealDBBackend;
///
/// let extractor = Arc::new(RegexExtractor::new());
/// let graph = Arc::new(SurrealDBBackend::new("./data".into()));
/// let engine = ManualConsolidationEngine::new(extractor, graph);
///
/// let result = engine.consolidate(&["session-123"], &mut entries).await?;
/// ```
pub struct ManualConsolidationEngine {
    /// Regex-based entity/relationship extractor
    extractor: Arc<RegexExtractor>,

    /// Knowledge graph backend
    knowledge_graph: Arc<dyn KnowledgeGraph>,
}

impl ManualConsolidationEngine {
    /// Create new manual consolidation engine
    ///
    /// # Arguments
    ///
    /// * `extractor` - Regex-based entity/relationship extractor
    /// * `knowledge_graph` - Knowledge graph backend for storage
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let engine = ManualConsolidationEngine::new(
    ///     Arc::new(RegexExtractor::new()),
    ///     Arc::new(SurrealDBBackend::new("./data".into()))
    /// );
    /// ```
    #[must_use]
    pub fn new(extractor: Arc<RegexExtractor>, knowledge_graph: Arc<dyn KnowledgeGraph>) -> Self {
        Self {
            extractor,
            knowledge_graph,
        }
    }

    /// Extract entities from episodic content
    ///
    /// Combines content from all entries and runs regex extraction.
    fn extract_entities(&self, entries: &[EpisodicEntry]) -> Vec<Entity> {
        let combined_text = entries
            .iter()
            .map(|e| e.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        self.extractor.extract_entities(&combined_text)
    }

    /// Extract relationships from episodic content
    ///
    /// Combines content from all entries and runs regex extraction.
    fn extract_relationships(&self, entries: &[EpisodicEntry]) -> Vec<Relationship> {
        let combined_text = entries
            .iter()
            .map(|e| e.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        self.extractor.extract_relationships(&combined_text)
    }

    /// Add entities to knowledge graph
    ///
    /// Returns count of entities successfully added.
    async fn add_entities(&self, entities: Vec<Entity>) -> Result<usize> {
        let mut added = 0;

        for entity in entities {
            match self.knowledge_graph.add_entity(entity).await {
                Ok(_) => added += 1,
                Err(e) => {
                    debug!("Failed to add entity: {}", e);
                    // Continue processing other entities
                }
            }
        }

        Ok(added)
    }

    /// Add relationships to knowledge graph
    ///
    /// Returns count of relationships successfully added.
    async fn add_relationships(&self, relationships: Vec<Relationship>) -> Result<usize> {
        let mut added = 0;

        for relationship in relationships {
            match self.knowledge_graph.add_relationship(relationship).await {
                Ok(_) => added += 1,
                Err(e) => {
                    debug!("Failed to add relationship: {}", e);
                    // Continue processing other relationships
                }
            }
        }

        Ok(added)
    }
}

#[async_trait]
impl ConsolidationEngine for ManualConsolidationEngine {
    async fn consolidate(
        &self,
        session_ids: &[&str],
        entries: &mut [EpisodicEntry],
    ) -> Result<ConsolidationResult> {
        let start = Instant::now();

        // Filter entries by session_ids (if specified)
        let entries_to_process: Vec<&mut EpisodicEntry> = if session_ids.is_empty() {
            entries.iter_mut().filter(|e| !e.processed).collect()
        } else {
            entries
                .iter_mut()
                .filter(|e| !e.processed && session_ids.contains(&e.session_id.as_str()))
                .collect()
        };

        let entries_count = entries_to_process.len();
        info!("Consolidating {} episodic entries", entries_count);

        if entries_count == 0 {
            return Ok(ConsolidationResult {
                entries_processed: 0,
                entities_added: 0,
                entities_updated: 0,
                entities_deleted: 0,
                entries_skipped: 0,
                duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
            });
        }

        // Collect into owned vector for extraction (need slice, not Vec of refs)
        let entries_slice: Vec<EpisodicEntry> =
            entries_to_process.iter().map(|e| (*e).clone()).collect();

        // Extract entities and relationships
        let entities = self.extract_entities(&entries_slice);
        let relationships = self.extract_relationships(&entries_slice);

        debug!(
            "Extracted {} entities and {} relationships",
            entities.len(),
            relationships.len()
        );

        // Add to knowledge graph
        let entities_added = self.add_entities(entities).await?;
        let _relationships_added = self.add_relationships(relationships).await?;

        // Mark entries as processed
        for entry in entries_to_process {
            entry.mark_processed();
        }

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        info!(
            "Consolidation complete: {} entries processed, {} entities added in {}ms",
            entries_count, entities_added, duration_ms
        );

        Ok(ConsolidationResult {
            entries_processed: entries_count,
            entities_added,
            entities_updated: 0, // Regex extractor doesn't update existing entities
            entities_deleted: 0, // Regex extractor doesn't delete entities
            entries_skipped: 0,
            duration_ms,
        })
    }

    fn is_ready(&self) -> bool {
        true // Manual engine is always ready (no external dependencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_graph::storage::surrealdb::SurrealDBBackend;
    use tempfile::TempDir;

    async fn create_test_engine() -> (ManualConsolidationEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let extractor = Arc::new(RegexExtractor::new());
        let graph: Arc<dyn KnowledgeGraph> = Arc::new(
            SurrealDBBackend::new(temp_dir.path().to_path_buf())
                .await
                .unwrap(),
        );
        let engine = ManualConsolidationEngine::new(extractor, graph);
        (engine, temp_dir)
    }

    #[tokio::test]
    async fn test_manual_consolidation_basic() {
        let (engine, _temp) = create_test_engine().await;

        let mut entries = vec![
            EpisodicEntry::new(
                "session-1".to_string(),
                "user".to_string(),
                "Rust is a systems programming language.".to_string(),
            ),
            EpisodicEntry::new(
                "session-1".to_string(),
                "assistant".to_string(),
                "Rust has memory safety.".to_string(),
            ),
        ];

        let result = engine
            .consolidate(&["session-1"], &mut entries)
            .await
            .unwrap();

        assert_eq!(result.entries_processed, 2);
        assert!(
            result.entities_added > 0,
            "Should extract at least one entity"
        );
        assert!(entries[0].processed);
        assert!(entries[1].processed);
    }

    #[tokio::test]
    async fn test_session_filtering() {
        let (engine, _temp) = create_test_engine().await;

        let mut entries = vec![
            EpisodicEntry::new(
                "session-1".to_string(),
                "user".to_string(),
                "Python is a language.".to_string(),
            ),
            EpisodicEntry::new(
                "session-2".to_string(),
                "user".to_string(),
                "JavaScript is a language.".to_string(),
            ),
        ];

        let result = engine
            .consolidate(&["session-1"], &mut entries)
            .await
            .unwrap();

        assert_eq!(result.entries_processed, 1);
        assert!(entries[0].processed);
        assert!(!entries[1].processed);
    }

    #[tokio::test]
    async fn test_empty_consolidation() {
        let (engine, _temp) = create_test_engine().await;

        let mut entries: Vec<EpisodicEntry> = vec![];

        let result = engine.consolidate(&[], &mut entries).await.unwrap();

        assert_eq!(result.entries_processed, 0);
        assert_eq!(result.entities_added, 0);
    }

    #[tokio::test]
    async fn test_already_processed_entries() {
        let (engine, _temp) = create_test_engine().await;

        let mut entry = EpisodicEntry::new(
            "session-1".to_string(),
            "user".to_string(),
            "Rust is great.".to_string(),
        );
        entry.mark_processed();

        let mut entries = vec![entry];

        let result = engine.consolidate(&[], &mut entries).await.unwrap();

        assert_eq!(result.entries_processed, 0);
    }

    #[tokio::test]
    async fn test_is_ready() {
        let (engine, _temp) = create_test_engine().await;
        assert!(engine.is_ready());
    }
}
