//! LLM-driven consolidation engine
//!
//! Implements LLM-based entity extraction and consolidation decisions.
//! Uses LLM providers to make ADD/UPDATE/DELETE/NOOP decisions on episodic content.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use llmspell_core::types::AgentInput;
use llmspell_graph::traits::KnowledgeGraph;
use llmspell_providers::ProviderInstance;
use serde_json::json;
use tracing::{debug, info, warn};
use uuid;

use crate::error::{MemoryError, Result};
use crate::types::{ConsolidationResult, EpisodicEntry};

use super::context_assembly::ContextAssembler;
use super::prompts::{ConsolidationPromptBuilder, PromptVersion};
use super::validator::DecisionValidator;
use super::ConsolidationEngine;

/// Configuration for LLM consolidation
#[derive(Debug, Clone)]
pub struct LLMConsolidationConfig {
    /// LLM model to use (e.g., "ollama/llama3.2:3b")
    pub model: String,
    /// Temperature for sampling (0.0 = deterministic)
    pub temperature: f32,
    /// Maximum tokens for completion
    pub max_tokens: usize,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retry attempts on LLM failure
    pub max_retries: u32,
    /// Prompt version to use
    pub version: PromptVersion,
}

impl Default for LLMConsolidationConfig {
    fn default() -> Self {
        Self {
            model: "ollama/llama3.2:3b".to_string(),
            temperature: 0.0,
            max_tokens: 2000,
            timeout_secs: 30,
            max_retries: 3,
            version: PromptVersion::default(),
        }
    }
}

/// LLM-driven consolidation engine
///
/// Uses LLM to make intelligent consolidation decisions:
/// - ADD: Create new entities from novel information
/// - UPDATE: Merge new facts into existing entities
/// - DELETE: Remove outdated/contradictory information
/// - NOOP: Skip irrelevant episodic content
pub struct LLMConsolidationEngine {
    /// LLM provider for making decisions
    provider: Arc<dyn ProviderInstance>,
    /// Knowledge graph for entity storage
    knowledge_graph: Arc<dyn KnowledgeGraph>,
    /// Context assembler for semantic retrieval
    context_assembler: ContextAssembler,
    /// Prompt builder for LLM requests
    prompt_builder: ConsolidationPromptBuilder,
    /// Decision validator
    validator: DecisionValidator,
    /// Configuration
    config: LLMConsolidationConfig,
}

impl LLMConsolidationEngine {
    /// Create new LLM consolidation engine
    ///
    /// # Arguments
    ///
    /// * `provider` - LLM provider instance for making decisions
    /// * `knowledge_graph` - Knowledge graph for entity storage
    /// * `config` - Consolidation configuration
    pub fn new(
        provider: Arc<dyn ProviderInstance>,
        knowledge_graph: Arc<dyn KnowledgeGraph>,
        config: LLMConsolidationConfig,
    ) -> Self {
        let context_assembler = ContextAssembler::new(Arc::clone(&knowledge_graph));
        let validator = DecisionValidator::new(Arc::clone(&knowledge_graph));
        let prompt_builder = ConsolidationPromptBuilder::new()
            .with_model(config.model.clone())
            .with_temperature(config.temperature)
            .with_version(config.version);

        Self {
            provider,
            knowledge_graph,
            context_assembler,
            prompt_builder,
            validator,
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(
        provider: Arc<dyn ProviderInstance>,
        knowledge_graph: Arc<dyn KnowledgeGraph>,
    ) -> Self {
        Self::new(provider, knowledge_graph, LLMConsolidationConfig::default())
    }
}

#[async_trait]
impl ConsolidationEngine for LLMConsolidationEngine {
    async fn consolidate(
        &self,
        session_ids: &[&str],
        entries: &mut [EpisodicEntry],
    ) -> Result<ConsolidationResult> {
        let start_time = Instant::now();
        let mut result = ConsolidationResult {
            entries_processed: 0,
            entities_added: 0,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 0,
            duration_ms: 0,
        };

        debug!(
            "Starting LLM consolidation for {} entries (sessions: {:?})",
            entries.len(),
            session_ids
        );

        // Filter entries by session if specified
        let entries_to_process: Vec<_> = entries
            .iter_mut()
            .filter(|e| {
                if session_ids.is_empty() {
                    true
                } else {
                    session_ids.contains(&e.session_id.as_str())
                }
            })
            .filter(|e| !e.processed)
            .collect();

        info!(
            "Processing {} unprocessed entries",
            entries_to_process.len()
        );

        for entry in entries_to_process {
            match self.process_entry(entry).await {
                Ok(entry_result) => {
                    result.entries_processed += 1;
                    result.entities_added += entry_result.entities_added;
                    result.entities_updated += entry_result.entities_updated;
                    result.entities_deleted += entry_result.entities_deleted;
                    result.entries_skipped += entry_result.entries_skipped;
                    entry.processed = true;
                }
                Err(e) => {
                    warn!("Failed to process entry: {}", e);
                    // Continue processing other entries
                }
            }
        }

        result.duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        info!(
            "Consolidation complete: {} entries, {} entities added, {} updated, {} deleted ({} ms)",
            result.entries_processed,
            result.entities_added,
            result.entities_updated,
            result.entities_deleted,
            result.duration_ms
        );

        Ok(result)
    }

    fn is_ready(&self) -> bool {
        // Check if provider is available (basic check)
        // Full validation would require async validate() call
        true
    }
}

impl LLMConsolidationEngine {
    /// Process a single episodic entry
    ///
    /// # Errors
    ///
    /// Returns error if LLM call fails or graph operations fail.
    async fn process_entry(&self, entry: &EpisodicEntry) -> Result<ConsolidationResult> {
        debug!("Processing entry: session={}, role={}", entry.session_id, entry.role);

        // Step 1: Assemble semantic context
        let semantic_context = self
            .context_assembler
            .assemble_context(entry)
            .await
            .map_err(|e| {
                MemoryError::Consolidation(format!("Failed to assemble context: {e}"))
            })?;

        // Step 2: Build prompts
        let system_prompt = self.prompt_builder.build_system_prompt()?;
        let user_prompt = self
            .prompt_builder
            .build_user_prompt(entry, &semantic_context)?;

        // Step 3: Call LLM with retry logic
        let llm_response = self
            .call_llm_with_retry(&system_prompt, &user_prompt)
            .await?;

        // Step 4: Parse response with error recovery (JSON with fallback to natural language)
        let consolidation_response = super::prompts::parse_llm_response(
            &llm_response,
            super::OutputFormat::Json, // JSON mode automatically falls back to natural language on parse failure
        )?;

        debug!(
            "Parsed {} decisions from LLM response",
            consolidation_response.decisions.len()
        );

        // Step 5: Validate decisions
        self.validator.validate(&consolidation_response).await?;

        // Step 6: Execute decisions
        let mut result = ConsolidationResult {
            entries_processed: 1,
            entities_added: 0,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 0,
            duration_ms: 0,
        };

        for (idx, decision) in consolidation_response.decisions.iter().enumerate() {
            match self.execute_decision(decision, &consolidation_response, idx).await {
                Ok(metrics) => {
                    result.entities_added += metrics.0;
                    result.entities_updated += metrics.1;
                    result.entities_deleted += metrics.2;
                    result.entries_skipped += metrics.3;
                }
                Err(e) => {
                    warn!("Failed to execute decision {}: {}", idx, e);
                    // Continue with other decisions (partial success)
                }
            }
        }

        // Execute relationships after all entities
        if !consolidation_response.relationships.is_empty() {
            self.execute_relationships(&consolidation_response.relationships).await?;
        }

        Ok(result)
    }

    /// Call LLM with retry logic
    async fn call_llm_with_retry(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            attempts += 1;

            debug!("LLM call attempt {}/{}", attempts, self.config.max_retries);

            match self.call_llm(system_prompt, user_prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("LLM call failed (attempt {}): {}", attempts, e);
                    last_error = Some(e);

                    if attempts < self.config.max_retries {
                        // Exponential backoff: 1s, 2s, 4s
                        let backoff_ms = 1000 * (1 << (attempts - 1));
                        debug!("Retrying after {}ms backoff", backoff_ms);
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            MemoryError::LLMCall(format!(
                "LLM call failed after {} attempts",
                self.config.max_retries
            ))
        }))
    }

    /// Call LLM provider
    async fn call_llm(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        // Combine system and user prompts into single text field
        let combined_prompt = format!("{system_prompt}\n\n{user_prompt}");

        let input = AgentInput::text(combined_prompt)
            .with_parameter("temperature", json!(self.config.temperature))
            .with_parameter("max_tokens", json!(self.config.max_tokens));

        let output = self
            .provider
            .complete(&input)
            .await
            .map_err(|e| MemoryError::LLMCall(format!("Provider error: {e}")))?;

        Ok(output.text)
    }

    /// Execute a single decision
    ///
    /// Returns (`entities_added`, `entities_updated`, `entities_deleted`, `entries_skipped`)
    async fn execute_decision(
        &self,
        decision: &super::DecisionPayload,
        response: &super::ConsolidationResponse,
        idx: usize,
    ) -> Result<(usize, usize, usize, usize)> {
        match decision {
            super::DecisionPayload::Add { entity_id } => {
                info!("Decision {}: Executing ADD for entity {}", idx, entity_id);
                self.execute_add(entity_id, response).await?;
                Ok((1, 0, 0, 0))
            }
            super::DecisionPayload::Update { entity_id, changes } => {
                info!("Decision {}: Executing UPDATE for entity {}", idx, entity_id);
                self.execute_update(entity_id, changes).await?;
                Ok((0, 1, 0, 0))
            }
            super::DecisionPayload::Delete { entity_id } => {
                info!("Decision {}: Executing DELETE for entity {}", idx, entity_id);
                self.execute_delete(entity_id).await?;
                Ok((0, 0, 1, 0))
            }
            super::DecisionPayload::Noop => {
                debug!("Decision {}: Executing NOOP (skip)", idx);
                Ok((0, 0, 0, 1))
            }
        }
    }

    /// Execute ADD decision
    async fn execute_add(
        &self,
        entity_id: &str,
        response: &super::ConsolidationResponse,
    ) -> Result<()> {
        // Find entity payload
        let entity_payload = response
            .entities
            .iter()
            .find(|e| e.id == entity_id)
            .ok_or_else(|| {
                MemoryError::InvalidInput(format!("Entity payload not found for ADD: {entity_id}"))
            })?;

        // Convert to graph entity
        let entity = llmspell_graph::types::Entity {
            id: entity_payload.id.clone(),
            name: entity_payload.name.clone(),
            entity_type: entity_payload.entity_type.clone(),
            properties: entity_payload.properties.clone(),
            event_time: entity_payload.event_time,
            ingestion_time: chrono::Utc::now(),
        };

        // Add to knowledge graph
        self.knowledge_graph
            .add_entity(entity)
            .await
            .map_err(|e| MemoryError::KnowledgeGraph(format!("Failed to add entity: {e}")))?;

        info!("Successfully added entity {}", entity_id);
        Ok(())
    }

    /// Execute UPDATE decision
    async fn execute_update(
        &self,
        entity_id: &str,
        changes: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        self.knowledge_graph
            .update_entity(entity_id, changes.clone())
            .await
            .map_err(|e| MemoryError::KnowledgeGraph(format!("Failed to update entity: {e}")))?;

        info!("Successfully updated entity {}", entity_id);
        Ok(())
    }

    /// Execute DELETE decision
    async fn execute_delete(&self, entity_id: &str) -> Result<()> {
        // Get current entity to set valid_until timestamp
        let mut entity = self
            .knowledge_graph
            .get_entity(entity_id)
            .await
            .map_err(|e| MemoryError::KnowledgeGraph(format!("Failed to get entity for deletion: {e}")))?;

        // Set valid_until to now (tombstone approach)
        entity.event_time = Some(chrono::Utc::now());

        // Update entity with tombstone
        let mut changes = std::collections::HashMap::new();
        changes.insert("_deleted".to_string(), serde_json::json!(true));
        changes.insert("_deleted_at".to_string(), serde_json::json!(chrono::Utc::now()));

        self.knowledge_graph
            .update_entity(entity_id, changes)
            .await
            .map_err(|e| MemoryError::KnowledgeGraph(format!("Failed to tombstone entity: {e}")))?;

        info!("Successfully deleted (tombstoned) entity {}", entity_id);
        Ok(())
    }

    /// Execute relationships
    async fn execute_relationships(
        &self,
        relationships: &[super::RelationshipPayload],
    ) -> Result<()> {
        for (idx, rel) in relationships.iter().enumerate() {
            debug!(
                "Executing relationship {}: {} -> {} ({})",
                idx, rel.from_entity, rel.to_entity, rel.relationship_type
            );

            let relationship = llmspell_graph::types::Relationship {
                id: uuid::Uuid::new_v4().to_string(),
                from_entity: rel.from_entity.clone(),
                to_entity: rel.to_entity.clone(),
                relationship_type: rel.relationship_type.clone(),
                properties: rel.properties.clone(),
                event_time: rel.event_time,
                ingestion_time: chrono::Utc::now(),
            };

            self.knowledge_graph
                .add_relationship(relationship)
                .await
                .map_err(|e| {
                    MemoryError::KnowledgeGraph(format!("Failed to add relationship {idx}: {e}"))
                })?;
        }

        info!("Successfully added {} relationships", relationships.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use llmspell_core::error::LLMSpellError;
    use llmspell_core::types::AgentOutput;
    use llmspell_graph::types::{Entity, TemporalQuery};
    use llmspell_providers::ProviderCapabilities;
    use std::collections::HashMap;

    // Mock provider for testing
    struct MockProvider {
        response: String,
        capabilities: ProviderCapabilities,
    }

    impl MockProvider {
        fn new(response: String) -> Self {
            Self {
                response,
                capabilities: ProviderCapabilities::default(),
            }
        }
    }

    #[async_trait]
    impl ProviderInstance for MockProvider {
        fn capabilities(&self) -> &ProviderCapabilities {
            &self.capabilities
        }

        async fn complete(&self, _input: &AgentInput) -> std::result::Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text(self.response.clone()))
        }

        async fn validate(&self) -> std::result::Result<(), LLMSpellError> {
            Ok(())
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn model(&self) -> &str {
            "mock-model"
        }
    }

    // Mock knowledge graph for testing
    struct MockKnowledgeGraph;

    #[async_trait]
    impl KnowledgeGraph for MockKnowledgeGraph {
        async fn add_entity(&self, _entity: Entity) -> llmspell_graph::error::Result<String> {
            Ok("mock-id".to_string())
        }

        async fn update_entity(
            &self,
            _id: &str,
            _changes: HashMap<String, serde_json::Value>,
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
            Ok(vec![])
        }

        async fn delete_before(
            &self,
            _timestamp: chrono::DateTime<Utc>,
        ) -> llmspell_graph::error::Result<usize> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_llm_engine_creation() {
        let provider = Arc::new(MockProvider::new("test".to_string())) as Arc<dyn ProviderInstance>;
        let graph = Arc::new(MockKnowledgeGraph) as Arc<dyn KnowledgeGraph>;
        let config = LLMConsolidationConfig::default();

        let engine = LLMConsolidationEngine::new(provider, graph, config);
        assert!(engine.is_ready());
    }

    #[tokio::test]
    async fn test_llm_call() {
        let provider =
            Arc::new(MockProvider::new("test response".to_string())) as Arc<dyn ProviderInstance>;
        let graph = Arc::new(MockKnowledgeGraph) as Arc<dyn KnowledgeGraph>;

        let engine = LLMConsolidationEngine::with_defaults(provider, graph);

        let response = engine
            .call_llm("system prompt", "user prompt")
            .await
            .unwrap();

        assert_eq!(response, "test response");
    }
}
