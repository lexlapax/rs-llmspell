//! LLM-driven consolidation engine
//!
//! Implements LLM-based entity extraction and consolidation decisions.
//! Uses LLM providers to make ADD/UPDATE/DELETE/NOOP decisions on episodic content.

use std::sync::atomic::{AtomicU32, Ordering};
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
    /// Fallback models to try if primary fails (e.g., `["ollama/qwen:7b", "ollama/mistral:7b"]`)
    pub fallback_models: Vec<String>,
    /// Temperature for sampling (0.0 = deterministic)
    pub temperature: f32,
    /// Maximum tokens for completion
    pub max_tokens: usize,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retry attempts on LLM failure per model
    pub max_retries: u32,
    /// Circuit breaker threshold (consecutive failures before circuit opens)
    pub circuit_breaker_threshold: u32,
    /// Prompt version to use
    pub version: PromptVersion,
}

impl Default for LLMConsolidationConfig {
    /// Default config for build-time testing only
    /// Production code should use `from_provider()` instead
    fn default() -> Self {
        Self {
            model: "test-model".to_string(),
            fallback_models: vec![],
            temperature: 0.0,
            max_tokens: 2000,
            timeout_secs: 30,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            version: PromptVersion::default(),
        }
    }
}

impl LLMConsolidationConfig {
    /// Create config from provider (PRIMARY factory method for production)
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider configuration from llmspell-config
    ///
    /// # Returns
    ///
    /// `LLMConsolidationConfig` with values from provider, using sensible defaults for missing fields
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::Consolidation` if provider is missing required `default_model` field
    pub fn from_provider(provider: &llmspell_config::ProviderConfig) -> Result<Self> {
        let model = provider
            .default_model
            .clone()
            .ok_or_else(|| MemoryError::Consolidation("provider missing default_model".into()))?;

        Ok(Self {
            model,
            fallback_models: vec![], // TODO: Add provider.fallback_models field in future
            temperature: provider.temperature.unwrap_or(0.0),
            max_tokens: provider.max_tokens.map_or(2000, |t| t as usize),
            timeout_secs: provider.timeout_seconds.unwrap_or(30),
            max_retries: provider.max_retries.unwrap_or(3),
            circuit_breaker_threshold: 5, // Not provider-configurable, consolidation-specific
            version: PromptVersion::default(),
        })
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
    /// Circuit breaker: consecutive failure counter
    consecutive_failures: AtomicU32,
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
            consecutive_failures: AtomicU32::new(0),
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
            entries_failed: 0,
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
                    result.entries_failed += 1;
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
        debug!(
            "Processing entry: session={}, role={}",
            entry.session_id, entry.role
        );

        // Step 1-3: Get LLM response
        let llm_response = self.get_llm_response(entry).await?;

        // Step 4-5: Parse and validate
        let consolidation_response = self.parse_and_validate(&llm_response).await?;

        // Step 6: Execute decisions and relationships
        let result = self.execute_consolidation(&consolidation_response).await?;

        Ok(result)
    }

    /// Helper: Get LLM response for entry
    async fn get_llm_response(&self, entry: &EpisodicEntry) -> Result<String> {
        // Step 1: Assemble semantic context
        let semantic_context = self
            .context_assembler
            .assemble_context(entry)
            .await
            .map_err(|e| MemoryError::Consolidation(format!("Failed to assemble context: {e}")))?;

        // Step 2: Build prompts
        let system_prompt = self.prompt_builder.build_system_prompt()?;
        let user_prompt = self
            .prompt_builder
            .build_user_prompt(entry, &semantic_context)?;

        // Step 3: Call LLM with retry logic
        self.call_llm_with_retry(&system_prompt, &user_prompt).await
    }

    /// Helper: Parse and validate LLM response
    async fn parse_and_validate(&self, llm_response: &str) -> Result<super::ConsolidationResponse> {
        // Parse response with error recovery (JSON with fallback to natural language)
        let consolidation_response = super::prompts::parse_llm_response(
            llm_response,
            super::OutputFormat::Json, // JSON mode automatically falls back to natural language on parse failure
        )?;

        debug!(
            "Parsed {} decisions from LLM response",
            consolidation_response.decisions.len()
        );

        // Validate decisions
        self.validator.validate(&consolidation_response).await?;

        Ok(consolidation_response)
    }

    /// Helper: Execute consolidation decisions and relationships
    async fn execute_consolidation(
        &self,
        response: &super::ConsolidationResponse,
    ) -> Result<ConsolidationResult> {
        let mut result = ConsolidationResult {
            entries_processed: 1,
            entities_added: 0,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 0,
            entries_failed: 0,
            duration_ms: 0,
        };

        // Execute decisions
        for (idx, decision) in response.decisions.iter().enumerate() {
            match self.execute_decision(decision, response, idx).await {
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
        if !response.relationships.is_empty() {
            self.execute_relationships(&response.relationships).await?;
        }

        Ok(result)
    }

    /// Call LLM with retry logic, circuit breaker, and provider fallback
    async fn call_llm_with_retry(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        // Circuit breaker: check if we've had too many consecutive failures
        self.check_circuit_breaker()?;

        // Build model fallback chain: primary + fallbacks
        let mut models = vec![self.config.model.clone()];
        models.extend(self.config.fallback_models.clone());

        let mut last_error = None;

        // Try each model in fallback chain
        for (model_idx, model) in models.iter().enumerate() {
            if model_idx > 0 {
                info!("Trying fallback model: {}", model);
            }

            match self
                .try_model_with_retries(system_prompt, user_prompt, model, model_idx)
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    warn!(
                        "All {} attempts failed for model {}, trying next model",
                        self.config.max_retries, model
                    );
                }
            }
        }

        // All models exhausted - increment circuit breaker and fail
        self.record_failure();

        Err(last_error.unwrap_or_else(|| {
            MemoryError::LLMCall(format!(
                "LLM call failed after trying {} models with {} retries each",
                models.len(),
                self.config.max_retries
            ))
        }))
    }

    /// Check circuit breaker state
    fn check_circuit_breaker(&self) -> Result<()> {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures >= self.config.circuit_breaker_threshold {
            warn!(
                "Circuit breaker OPEN: {} consecutive failures (threshold: {})",
                failures, self.config.circuit_breaker_threshold
            );
            return Err(MemoryError::LLMCall(format!(
                "Circuit breaker open after {failures} consecutive failures"
            )));
        }
        Ok(())
    }

    /// Try a model with retries
    async fn try_model_with_retries(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
        model_idx: usize,
    ) -> Result<String> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            attempts += 1;

            debug!(
                "LLM call attempt {}/{} (model: {})",
                attempts, self.config.max_retries, model
            );

            // Health check before retry (skip on first attempt)
            if attempts > 1 && !self.provider_is_healthy().await {
                return Err(last_error.unwrap_or_else(|| {
                    MemoryError::LLMCall("Provider health check failed".to_string())
                }));
            }

            match self.call_llm(system_prompt, user_prompt, model).await {
                Ok(response) => {
                    return Ok(self.handle_successful_call(response, model_idx, model));
                }
                Err(e) => {
                    last_error = self.handle_failed_call(e, attempts).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            MemoryError::LLMCall(format!("All {} attempts failed", self.config.max_retries))
        }))
    }

    /// Helper: Handle successful LLM call
    fn handle_successful_call(&self, response: String, model_idx: usize, model: &str) -> String {
        // Success! Reset circuit breaker
        self.consecutive_failures.store(0, Ordering::Relaxed);
        if model_idx > 0 {
            info!("Fallback model {} succeeded", model);
        }
        response
    }

    /// Helper: Handle failed LLM call
    async fn handle_failed_call(&self, error: MemoryError, attempts: u32) -> Option<MemoryError> {
        warn!(
            "LLM call failed (attempt {}/{}): {}",
            attempts, self.config.max_retries, error
        );

        if attempts < self.config.max_retries {
            self.exponential_backoff(attempts).await;
        }

        Some(error)
    }

    /// Check provider health
    async fn provider_is_healthy(&self) -> bool {
        match self.provider.validate().await {
            Ok(()) => {
                debug!("Provider health check passed");
                true
            }
            Err(e) => {
                warn!("Provider unhealthy, skipping retry: {}", e);
                false
            }
        }
    }

    /// Exponential backoff delay
    async fn exponential_backoff(&self, attempt: u32) {
        // Exponential backoff: 1s, 2s, 4s
        let backoff_ms = 1000 * (1 << (attempt - 1));
        debug!("Retrying after {}ms backoff", backoff_ms);
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
    }

    /// Record failure and update circuit breaker
    fn record_failure(&self) {
        let new_failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        warn!(
            "All models failed, circuit breaker counter: {}/{}",
            new_failures, self.config.circuit_breaker_threshold
        );
    }

    /// Call LLM provider with specified model
    async fn call_llm(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
    ) -> Result<String> {
        // Combine system and user prompts into single text field
        let combined_prompt = format!("{system_prompt}\n\n{user_prompt}");

        let input = AgentInput::text(combined_prompt)
            .with_parameter("model", json!(model))
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
                self.execute_add_decision(idx, entity_id, response).await
            }
            super::DecisionPayload::Update { entity_id, changes } => {
                self.execute_update_decision(idx, entity_id, changes).await
            }
            super::DecisionPayload::Delete { entity_id } => {
                self.execute_delete_decision(idx, entity_id).await
            }
            super::DecisionPayload::Noop => Ok(Self::execute_noop_decision(idx)),
        }
    }

    /// Helper: Execute ADD decision
    async fn execute_add_decision(
        &self,
        idx: usize,
        entity_id: &str,
        response: &super::ConsolidationResponse,
    ) -> Result<(usize, usize, usize, usize)> {
        info!("Decision {}: Executing ADD for entity {}", idx, entity_id);
        Self::execute_add(entity_id, response, &self.knowledge_graph).await?;
        Ok((1, 0, 0, 0))
    }

    /// Helper: Execute UPDATE decision
    async fn execute_update_decision(
        &self,
        idx: usize,
        entity_id: &str,
        changes: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(usize, usize, usize, usize)> {
        info!(
            "Decision {}: Executing UPDATE for entity {}",
            idx, entity_id
        );
        self.execute_update(entity_id, changes).await?;
        Ok((0, 1, 0, 0))
    }

    /// Helper: Execute DELETE decision
    async fn execute_delete_decision(
        &self,
        idx: usize,
        entity_id: &str,
    ) -> Result<(usize, usize, usize, usize)> {
        info!(
            "Decision {}: Executing DELETE for entity {}",
            idx, entity_id
        );
        self.execute_delete(entity_id).await?;
        Ok((0, 0, 1, 0))
    }

    /// Helper: Execute NOOP decision
    fn execute_noop_decision(idx: usize) -> (usize, usize, usize, usize) {
        debug!("Decision {}: Executing NOOP (skip)", idx);
        (0, 0, 0, 1)
    }

    /// Execute ADD decision
    async fn execute_add(
        entity_id: &str,
        response: &super::ConsolidationResponse,
        knowledge_graph: &Arc<dyn llmspell_graph::traits::KnowledgeGraph>,
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
        knowledge_graph
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
            .map_err(|e| {
                MemoryError::KnowledgeGraph(format!("Failed to get entity for deletion: {e}"))
            })?;

        // Set valid_until to now (tombstone approach)
        entity.event_time = Some(chrono::Utc::now());

        // Update entity with tombstone
        let mut changes = std::collections::HashMap::new();
        changes.insert("_deleted".to_string(), serde_json::json!(true));
        changes.insert(
            "_deleted_at".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );

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

        async fn complete(
            &self,
            _input: &AgentInput,
        ) -> std::result::Result<AgentOutput, LLMSpellError> {
            Ok(AgentOutput::text(self.response.clone()))
        }

        async fn validate(&self) -> std::result::Result<(), LLMSpellError> {
            Ok(())
        }

        fn name(&self) -> &'static str {
            "mock"
        }

        fn model(&self) -> &'static str {
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

        async fn get_relationships(
            &self,
            _entity_id: &str,
        ) -> llmspell_graph::error::Result<Vec<llmspell_graph::types::Relationship>> {
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
            .call_llm("system prompt", "user prompt", "ollama/llama3.2:3b")
            .await
            .unwrap();

        assert_eq!(response, "test response");
    }
}
