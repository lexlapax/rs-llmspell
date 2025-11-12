//! Context assembly for consolidation prompts
//!
//! Assembles semantic context from knowledge graph entities using BM25 retrieval
//! to provide relevant existing knowledge for LLM consolidation decisions.

use crate::error::{MemoryError, Result};
use crate::types::EpisodicEntry;

use llmspell_graph::traits::KnowledgeGraph;
use llmspell_graph::types::{Entity, TemporalQuery};
use std::sync::Arc;
use tracing::{debug, trace};

/// Context assembler for consolidation prompts
///
/// Retrieves relevant entities from knowledge graph and formats them
/// as semantic context for the LLM consolidation prompt.
pub struct ContextAssembler {
    /// Knowledge graph for entity retrieval
    knowledge_graph: Arc<dyn KnowledgeGraph>,
    /// Maximum entities to retrieve
    max_entities: usize,
}

impl ContextAssembler {
    /// Create new context assembler
    #[must_use]
    pub fn new(knowledge_graph: Arc<dyn KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            max_entities: 10, // Default: top 10 entities
        }
    }

    /// Set maximum entities to retrieve
    #[must_use]
    pub const fn with_max_entities(mut self, max_entities: usize) -> Self {
        self.max_entities = max_entities;
        self
    }

    /// Assemble semantic context for consolidation
    ///
    /// Retrieves relevant entities from knowledge graph based on episodic content.
    /// Uses simple keyword matching for now (BM25 integration in future enhancement).
    ///
    /// # Errors
    ///
    /// Returns error if knowledge graph query fails.
    pub async fn assemble_context(&self, episodic: &EpisodicEntry) -> Result<String> {
        // Extract keywords from episodic content for entity matching
        let keywords = Self::extract_keywords(&episodic.content);

        debug!(
            "Assembling context with {} keywords for session {}",
            keywords.len(),
            episodic.session_id
        );

        // Query knowledge graph for recent entities
        // Priority: entities with recent ingestion time (temporal ordering)
        let mut query = TemporalQuery::new();
        query.limit = Some(self.max_entities);

        let entities = self
            .knowledge_graph
            .query_temporal(query)
            .await
            .map_err(|e| MemoryError::KnowledgeGraph(format!("Failed to query entities: {e}")))?;

        trace!("Retrieved {} entities from knowledge graph", entities.len());

        // Format entities as semantic context
        let context = self.format_entities(&entities, &keywords);

        Ok(context)
    }

    /// Extract keywords from text (simple tokenization)
    ///
    /// In future enhancement, this will integrate with Phase 13.4 `QueryAnalyzer`
    /// for proper BM25-based relevance scoring.
    fn extract_keywords(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3) // Filter short words
            .take(10) // Top 10 keywords
            .map(str::to_string)
            .collect()
    }

    /// Format entities as semantic context string
    ///
    /// Formats entities with temporal information for bi-temporal reasoning.
    /// Prioritizes entities matching keywords from episodic content.
    fn format_entities(&self, entities: &[Entity], keywords: &[String]) -> String {
        if entities.is_empty() {
            return String::new();
        }

        // Score entities by keyword match
        let mut scored_entities: Vec<_> = entities
            .iter()
            .map(|entity| {
                let score = Self::calculate_relevance_score(entity, keywords);
                (entity, score)
            })
            .collect();

        // Sort by relevance score (descending)
        scored_entities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Format top entities
        let formatted = scored_entities
            .iter()
            .take(self.max_entities)
            .filter(|(_, score)| *score > 0.0) // Only include entities with matches
            .map(|(entity, _)| Self::format_entity(entity))
            .collect::<Vec<_>>()
            .join("\n");

        if formatted.is_empty() {
            // If no keyword matches, return recent entities anyway
            entities
                .iter()
                .take(5)
                .map(Self::format_entity)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            formatted
        }
    }

    /// Calculate relevance score for entity based on keyword matches
    fn calculate_relevance_score(entity: &Entity, keywords: &[String]) -> f32 {
        let entity_text = format!(
            "{} {} {:?}",
            entity.name, entity.entity_type, entity.properties
        )
        .to_lowercase();

        let mut score = 0.0;
        for keyword in keywords {
            if entity_text.contains(keyword) {
                score += 1.0;
            }
        }
        score
    }

    /// Format single entity for semantic context
    ///
    /// Includes temporal information for bi-temporal reasoning.
    fn format_entity(entity: &Entity) -> String {
        let event_time_str = entity.event_time.map_or_else(
            || "event_time=unknown".to_string(),
            |t| format!("event_time={}", t.format("%Y-%m-%d")),
        );

        format!(
            "- Entity(id={}, name=\"{}\", type={}, properties={}, {}, ingestion={})",
            entity.id,
            entity.name,
            entity.entity_type,
            entity.properties,
            event_time_str,
            entity.ingestion_time.format("%Y-%m-%d")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    // Mock knowledge graph for testing
    struct MockKnowledgeGraph {
        entities: Vec<Entity>,
    }

    #[async_trait::async_trait]
    impl KnowledgeGraph for MockKnowledgeGraph {
        async fn add_entity(&self, _entity: Entity) -> llmspell_graph::error::Result<String> {
            Ok("mock-id".to_string())
        }

        async fn update_entity(
            &self,
            _id: &str,
            _changes: std::collections::HashMap<String, serde_json::Value>,
        ) -> llmspell_graph::error::Result<()> {
            Ok(())
        }

        async fn get_entity(&self, _id: &str) -> llmspell_graph::error::Result<Entity> {
            Err(llmspell_graph::error::GraphError::EntityNotFound(
                "mock".to_string(),
            ))
        }

        async fn get_entity_at(
            &self,
            _id: &str,
            _event_time: chrono::DateTime<Utc>,
        ) -> llmspell_graph::error::Result<Entity> {
            Err(llmspell_graph::error::GraphError::EntityNotFound(
                "mock".to_string(),
            ))
        }

        async fn add_relationship(
            &self,
            _relationship: llmspell_graph::types::Relationship,
        ) -> llmspell_graph::error::Result<String> {
            Ok("mock-rel-id".to_string())
        }

        async fn get_related(
            &self,
            _entity_id: &str,
            _relationship_type: &str,
        ) -> llmspell_graph::error::Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn query_temporal(
            &self,
            _query: TemporalQuery,
        ) -> llmspell_graph::error::Result<Vec<Entity>> {
            Ok(self.entities.clone())
        }

        async fn delete_before(
            &self,
            _timestamp: chrono::DateTime<Utc>,
        ) -> llmspell_graph::error::Result<usize> {
            Ok(0)
        }

        async fn traverse(
            &self,
            _start_entity: &str,
            _relationship_type: Option<&str>,
            _max_depth: usize,
            _at_time: Option<chrono::DateTime<Utc>>,
        ) -> llmspell_graph::error::Result<Vec<(Entity, usize, String)>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_assemble_context_with_entities() {
        let entities = vec![
            Entity {
                id: "rust-1".to_string(),
                name: "Rust".to_string(),
                entity_type: "programming_language".to_string(),
                properties: json!({"paradigm": "multi-paradigm"}),
                event_time: None,
                ingestion_time: Utc::now(),
            },
            Entity {
                id: "python-1".to_string(),
                name: "Python".to_string(),
                entity_type: "programming_language".to_string(),
                properties: json!({"paradigm": "object-oriented"}),
                event_time: None,
                ingestion_time: Utc::now(),
            },
        ];

        let graph = Arc::new(MockKnowledgeGraph {
            entities: entities.clone(),
        });
        let assembler = ContextAssembler::new(graph);

        let episodic = EpisodicEntry::new(
            "session-1".to_string(),
            "user".to_string(),
            "Rust is a systems programming language".to_string(),
        );

        let context = assembler.assemble_context(&episodic).await.unwrap();

        assert!(!context.is_empty());
        assert!(context.contains("Rust"));
        assert!(context.contains("programming_language"));
    }

    #[tokio::test]
    async fn test_assemble_context_empty_graph() {
        let graph = Arc::new(MockKnowledgeGraph { entities: vec![] });
        let assembler = ContextAssembler::new(graph);

        let episodic = EpisodicEntry::new(
            "session-1".to_string(),
            "user".to_string(),
            "Test content".to_string(),
        );

        let context = assembler.assemble_context(&episodic).await.unwrap();

        assert!(context.is_empty());
    }

    #[test]
    fn test_extract_keywords() {
        let text = "Rust is a systems programming language with memory safety";
        let keywords = ContextAssembler::extract_keywords(text);

        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"systems".to_string()));
        assert!(keywords.contains(&"programming".to_string()));
        assert!(!keywords.contains(&"is".to_string())); // Too short
        assert!(!keywords.contains(&"a".to_string())); // Too short
    }

    #[test]
    fn test_calculate_relevance_score() {
        let entity = Entity {
            id: "rust-1".to_string(),
            name: "Rust".to_string(),
            entity_type: "programming_language".to_string(),
            properties: json!({"feature": "memory safety"}),
            event_time: None,
            ingestion_time: Utc::now(),
        };

        let keywords = vec![
            "rust".to_string(),
            "memory".to_string(),
            "safety".to_string(),
        ];
        let score = ContextAssembler::calculate_relevance_score(&entity, &keywords);

        assert!(score >= 3.0); // All 3 keywords match
    }

    #[test]
    fn test_format_entity() {
        let entity = Entity {
            id: "test-1".to_string(),
            name: "Test Entity".to_string(),
            entity_type: "concept".to_string(),
            properties: json!({"key": "value"}),
            event_time: None,
            ingestion_time: Utc::now(),
        };

        let formatted = ContextAssembler::format_entity(&entity);

        assert!(formatted.contains("Test Entity"));
        assert!(formatted.contains("concept"));
        assert!(formatted.contains("event_time=unknown"));
    }

    #[test]
    fn test_with_max_entities() {
        let graph = Arc::new(MockKnowledgeGraph { entities: vec![] });
        let assembler = ContextAssembler::new(graph).with_max_entities(5);

        assert_eq!(assembler.max_entities, 5);
    }
}
