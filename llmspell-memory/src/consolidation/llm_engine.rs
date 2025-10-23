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
    /// Knowledge graph for entity storage (used in Task 13.5.2d for decision execution)
    #[allow(dead_code)]
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

        // Step 6: Execute decisions (TODO: Task 13.5.2d)

        // Placeholder: return metrics based on parsed decisions
        let mut result = ConsolidationResult {
            entries_processed: 1,
            entities_added: 0,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 0,
            duration_ms: 0,
        };

        // Count decisions for metrics (will be replaced with actual execution in 13.5.2d)
        for decision in &consolidation_response.decisions {
            match decision {
                super::DecisionPayload::Add { .. } => result.entities_added += 1,
                super::DecisionPayload::Update { .. } => result.entities_updated += 1,
                super::DecisionPayload::Delete { .. } => result.entities_deleted += 1,
                super::DecisionPayload::Noop => result.entries_skipped += 1,
            }
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
