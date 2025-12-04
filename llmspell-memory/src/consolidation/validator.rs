//! Decision validation logic
//!
//! Validates consolidation decisions before execution:
//! - UPDATE/DELETE: Verify entity IDs exist in knowledge graph
//! - ADD: Prevent duplicate entities (check if already exists)
//! - Relationships: Validate source/target entities exist
//!
//! Returns actionable validation errors for debugging and metrics.

use crate::error::{MemoryError, Result};
use llmspell_graph::traits::KnowledgeGraph;
use std::sync::Arc;
use tracing::{debug, warn};

use super::prompt_schema::{ConsolidationResponse, DecisionPayload};

/// Decision validator for consolidation operations
///
/// Validates decisions against knowledge graph state before execution.
pub struct DecisionValidator {
    knowledge_graph: Arc<dyn KnowledgeGraph>,
}

impl DecisionValidator {
    /// Create new decision validator
    pub fn new(knowledge_graph: Arc<dyn KnowledgeGraph>) -> Self {
        Self { knowledge_graph }
    }

    /// Validate all decisions in consolidation response
    ///
    /// # Errors
    ///
    /// Returns error if any decision is invalid with actionable message.
    pub async fn validate(&self, response: &ConsolidationResponse) -> Result<()> {
        debug!("Validating {} decisions", response.decisions.len());

        for (idx, decision) in response.decisions.iter().enumerate() {
            self.validate_decision(decision, response, idx).await?;
        }

        debug!(
            "All {} decisions validated successfully",
            response.decisions.len()
        );
        Ok(())
    }

    /// Validate a single decision
    async fn validate_decision(
        &self,
        decision: &DecisionPayload,
        response: &ConsolidationResponse,
        idx: usize,
    ) -> Result<()> {
        match decision {
            DecisionPayload::Add { entity_id } => self.validate_add(entity_id, response, idx).await,
            DecisionPayload::Update { entity_id, .. } => self.validate_update(entity_id, idx).await,
            DecisionPayload::Delete { entity_id } => self.validate_delete(entity_id, idx).await,
            DecisionPayload::Noop => {
                debug!("Decision {}: NOOP (no validation needed)", idx);
                Ok(())
            }
        }
    }

    /// Validate ADD decision
    ///
    /// Checks if entity already exists to prevent duplicates.
    async fn validate_add(
        &self,
        entity_id: &str,
        response: &ConsolidationResponse,
        idx: usize,
    ) -> Result<()> {
        debug!("Decision {}: Validating ADD for entity {}", idx, entity_id);

        // Check if entity exists in knowledge graph
        match self.knowledge_graph.get_entity(entity_id).await {
            Ok(_) => {
                warn!(
                    "Decision {}: Entity {} already exists, cannot ADD",
                    idx, entity_id
                );
                Err(MemoryError::InvalidInput(format!(
                    "Decision {idx}: Cannot ADD entity {entity_id} - already exists in knowledge graph"
                )))
            }
            Err(e) if e.to_string().contains("Entity not found") => {
                // Entity doesn't exist, ADD is valid
                debug!("Decision {}: ADD validation passed for {}", idx, entity_id);

                // Verify entity payload exists
                if !response.entities.iter().any(|e| e.id == entity_id) {
                    return Err(MemoryError::InvalidInput(format!(
                        "Decision {idx}: ADD entity {entity_id} has no corresponding entity payload"
                    )));
                }

                Ok(())
            }
            Err(e) => {
                // Graph error during lookup
                Err(MemoryError::KnowledgeGraph(format!(
                    "Decision {idx}: Failed to check if entity {entity_id} exists: {e}"
                )))
            }
        }
    }

    /// Validate UPDATE decision
    ///
    /// Checks if entity exists before allowing update.
    async fn validate_update(&self, entity_id: &str, idx: usize) -> Result<()> {
        debug!(
            "Decision {}: Validating UPDATE for entity {}",
            idx, entity_id
        );

        match self.knowledge_graph.get_entity(entity_id).await {
            Ok(_) => {
                debug!(
                    "Decision {}: UPDATE validation passed for {}",
                    idx, entity_id
                );
                Ok(())
            }
            Err(e) if e.to_string().contains("Entity not found") => {
                warn!(
                    "Decision {}: Entity {} does not exist, cannot UPDATE",
                    idx, entity_id
                );
                Err(MemoryError::InvalidInput(format!(
                    "Decision {idx}: Cannot UPDATE entity {entity_id} - does not exist in knowledge graph"
                )))
            }
            Err(e) => Err(MemoryError::KnowledgeGraph(format!(
                "Decision {idx}: Failed to validate UPDATE for entity {entity_id}: {e}"
            ))),
        }
    }

    /// Validate DELETE decision
    ///
    /// Checks if entity exists before allowing deletion.
    async fn validate_delete(&self, entity_id: &str, idx: usize) -> Result<()> {
        debug!(
            "Decision {}: Validating DELETE for entity {}",
            idx, entity_id
        );

        match self.knowledge_graph.get_entity(entity_id).await {
            Ok(_) => {
                debug!(
                    "Decision {}: DELETE validation passed for {}",
                    idx, entity_id
                );
                Ok(())
            }
            Err(e) if e.to_string().contains("Entity not found") => {
                warn!(
                    "Decision {}: Entity {} does not exist, cannot DELETE",
                    idx, entity_id
                );
                Err(MemoryError::InvalidInput(format!(
                    "Decision {idx}: Cannot DELETE entity {entity_id} - does not exist in knowledge graph"
                )))
            }
            Err(e) => Err(MemoryError::KnowledgeGraph(format!(
                "Decision {idx}: Failed to validate DELETE for entity {entity_id}: {e}"
            ))),
        }
    }

    /// Validate relationships in response
    ///
    /// Checks that source and target entities exist.
    ///
    /// # Errors
    ///
    /// Returns error if source or target entity does not exist in knowledge graph.
    pub async fn validate_relationships(&self, response: &ConsolidationResponse) -> Result<()> {
        for (idx, relationship) in response.relationships.iter().enumerate() {
            debug!(
                "Validating relationship {}: {} -> {}",
                idx, relationship.from_entity, relationship.to_entity
            );

            // Check source entity exists
            self.validate_entity_exists(&relationship.from_entity, "source", idx)
                .await?;

            // Check target entity exists
            self.validate_entity_exists(&relationship.to_entity, "target", idx)
                .await?;
        }

        Ok(())
    }

    /// Helper to validate entity existence for relationships
    async fn validate_entity_exists(
        &self,
        entity_id: &str,
        entity_type: &str,
        idx: usize,
    ) -> Result<()> {
        match self.knowledge_graph.get_entity(entity_id).await {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("Entity not found") => Err(MemoryError::InvalidInput(
                format!("Relationship {idx}: {entity_type} entity {entity_id} does not exist"),
            )),
            Err(e) => Err(MemoryError::KnowledgeGraph(format!(
                "Relationship {idx}: Failed to validate {entity_type} entity {entity_id}: {e}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::EntityPayload;
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use llmspell_graph::{Entity, TemporalQuery};
    use std::collections::HashMap;

    // Mock knowledge graph for testing
    struct MockKnowledgeGraph {
        existing_entities: Vec<String>,
    }

    impl MockKnowledgeGraph {
        fn new(existing: Vec<String>) -> Self {
            Self {
                existing_entities: existing,
            }
        }
    }

    #[async_trait]
    impl KnowledgeGraph for MockKnowledgeGraph {
        async fn add_entity(&self, _entity: Entity) -> anyhow::Result<String> {
            Ok("mock-id".to_string())
        }

        async fn update_entity(
            &self,
            _id: &str,
            _changes: HashMap<String, serde_json::Value>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_entity(&self, id: &str) -> anyhow::Result<Entity> {
            if self.existing_entities.contains(&id.to_string()) {
                Ok(Entity {
                    id: id.to_string(),
                    name: "mock".to_string(),
                    entity_type: "mock".to_string(),
                    properties: serde_json::json!({}),
                    event_time: Some(Utc::now()),
                    ingestion_time: Utc::now(),
                })
            } else {
                Err(anyhow::anyhow!("Entity not found: {id}"))
            }
        }

        async fn get_entity_at(
            &self,
            _id: &str,
            _event_time: chrono::DateTime<Utc>,
        ) -> anyhow::Result<Entity> {
            Err(anyhow::anyhow!("Entity not found: mock"))
        }

        async fn add_relationship(
            &self,
            _relationship: llmspell_graph::Relationship,
        ) -> anyhow::Result<String> {
            Ok("mock-rel-id".to_string())
        }

        async fn get_related(
            &self,
            _entity_id: &str,
            _relationship_type: &str,
        ) -> anyhow::Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn get_relationships(
            &self,
            _entity_id: &str,
        ) -> anyhow::Result<Vec<llmspell_graph::Relationship>> {
            Ok(vec![])
        }

        async fn query_temporal(&self, _query: TemporalQuery) -> anyhow::Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn delete_before(&self, _timestamp: chrono::DateTime<Utc>) -> anyhow::Result<usize> {
            Ok(0)
        }

        async fn traverse(
            &self,
            _start_entity: &str,
            _relationship_type: Option<&str>,
            _max_depth: usize,
            _at_time: Option<chrono::DateTime<Utc>>,
        ) -> anyhow::Result<Vec<(Entity, usize, String)>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_validate_add_success() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec![])) as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![EntityPayload {
                id: "new-entity-123".to_string(),
                name: "Test".to_string(),
                entity_type: "test".to_string(),
                properties: serde_json::json!({}),
                event_time: None,
            }],
            relationships: vec![],
            decisions: vec![DecisionPayload::Add {
                entity_id: "new-entity-123".to_string(),
            }],
            reasoning: None,
            prompt_version: None,
        };

        validator.validate(&response).await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_add_duplicate() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec!["existing-123".to_string()]))
            as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![EntityPayload {
                id: "existing-123".to_string(),
                name: "Test".to_string(),
                entity_type: "test".to_string(),
                properties: serde_json::json!({}),
                event_time: None,
            }],
            relationships: vec![],
            decisions: vec![DecisionPayload::Add {
                entity_id: "existing-123".to_string(),
            }],
            reasoning: None,
            prompt_version: None,
        };

        let result = validator.validate(&response).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_validate_update_success() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec!["existing-123".to_string()]))
            as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Update {
                entity_id: "existing-123".to_string(),
                changes: HashMap::new(),
            }],
            reasoning: None,
            prompt_version: None,
        };

        validator.validate(&response).await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_update_not_found() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec![])) as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Update {
                entity_id: "nonexistent-123".to_string(),
                changes: HashMap::new(),
            }],
            reasoning: None,
            prompt_version: None,
        };

        let result = validator.validate(&response).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_validate_delete_success() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec!["existing-123".to_string()]))
            as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Delete {
                entity_id: "existing-123".to_string(),
            }],
            reasoning: None,
            prompt_version: None,
        };

        validator.validate(&response).await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_delete_not_found() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec![])) as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Delete {
                entity_id: "nonexistent-123".to_string(),
            }],
            reasoning: None,
            prompt_version: None,
        };

        let result = validator.validate(&response).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_validate_noop() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec![])) as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![],
            relationships: vec![],
            decisions: vec![DecisionPayload::Noop],
            reasoning: None,
            prompt_version: None,
        };

        validator.validate(&response).await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_multiple_decisions() {
        let graph = Arc::new(MockKnowledgeGraph::new(vec!["existing-123".to_string()]))
            as Arc<dyn KnowledgeGraph>;
        let validator = DecisionValidator::new(graph);

        let response = ConsolidationResponse {
            entities: vec![EntityPayload {
                id: "new-entity-456".to_string(),
                name: "Test".to_string(),
                entity_type: "test".to_string(),
                properties: serde_json::json!({}),
                event_time: None,
            }],
            relationships: vec![],
            decisions: vec![
                DecisionPayload::Add {
                    entity_id: "new-entity-456".to_string(),
                },
                DecisionPayload::Update {
                    entity_id: "existing-123".to_string(),
                    changes: HashMap::new(),
                },
                DecisionPayload::Noop,
            ],
            reasoning: None,
            prompt_version: None,
        };

        validator.validate(&response).await.unwrap();
    }
}
