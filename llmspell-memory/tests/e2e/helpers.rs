//! Test helpers for E2E consolidation tests
//!
//! Utilities for creating test engines, asserting graph state, and calculating DMR.

use llmspell_graph::storage::surrealdb::SurrealDBBackend;
use llmspell_graph::traits::KnowledgeGraph;
use llmspell_memory::consolidation::{
    ConsolidationMetrics, DecisionPayload, DecisionType, LLMConsolidationConfig,
    LLMConsolidationEngine,
};
use llmspell_providers::abstraction::ProviderConfig;
use llmspell_providers::local::create_ollama_provider;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Ground truth decision for DMR calculation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundTruthDecision {
    /// ADD entity with expected ID
    Add { entity_id: String },
    /// UPDATE entity with expected ID
    Update { entity_id: String },
    /// DELETE entity with expected ID
    Delete { entity_id: String },
    /// NOOP (no action)
    Noop,
}

impl GroundTruthDecision {
    /// Get decision type
    pub fn decision_type(&self) -> DecisionType {
        match self {
            Self::Add { .. } => DecisionType::Add,
            Self::Update { .. } => DecisionType::Update,
            Self::Delete { .. } => DecisionType::Delete,
            Self::Noop => DecisionType::Noop,
        }
    }
}

/// Test engine bundle
#[allow(dead_code)]
pub struct TestEngine {
    pub llm_engine: LLMConsolidationEngine,
    pub metrics: Arc<ConsolidationMetrics>,
    pub knowledge_graph: Arc<dyn KnowledgeGraph>,
    pub _temp_dir: TempDir,
}

/// Create test LLM consolidation engine with metrics
///
/// # Returns
///
/// Test engine bundle with LLM engine, metrics, knowledge graph, and temp directory
pub async fn create_test_engine() -> TestEngine {
    // Create knowledge graph with temp directory
    let knowledge_graph = Arc::new(
        SurrealDBBackend::new_temp()
            .await
            .unwrap(),
    ) as Arc<dyn KnowledgeGraph>;

    // Create temp dir for test cleanup
    let temp_dir = TempDir::new().unwrap();

    // Create Ollama provider
    let ollama_host = super::get_ollama_host();
    let mut provider_config = ProviderConfig::new_with_type("ollama", "local", "llama3.2:3b");
    provider_config.endpoint = Some(ollama_host);
    provider_config.timeout_secs = Some(60);

    let provider = Arc::from(
        create_ollama_provider(provider_config).unwrap()
    );

    // Create LLM consolidation config
    let config = LLMConsolidationConfig {
        model: "llama3.2:3b".to_string(),
        fallback_models: vec![],
        temperature: 0.0, // Deterministic for testing
        max_tokens: 2000,
        timeout_secs: 60,
        max_retries: 2,
        circuit_breaker_threshold: 5,
        version: Default::default(),
    };

    // Create LLM engine
    let llm_engine = LLMConsolidationEngine::new(
        Arc::clone(&provider),
        Arc::clone(&knowledge_graph),
        config,
    );

    // Create metrics
    let metrics = Arc::new(ConsolidationMetrics::new());

    TestEngine {
        llm_engine,
        metrics,
        knowledge_graph,
        _temp_dir: temp_dir,
    }
}

/// Assert entity exists in knowledge graph
///
/// # Arguments
///
/// * `graph` - Knowledge graph to query
/// * `entity_id` - Expected entity ID
/// * `expected_properties` - Optional expected properties (partial match)
///
/// # Panics
///
/// Panics if entity does not exist or properties don't match
#[allow(dead_code)]
pub async fn assert_entity_exists(
    graph: &Arc<dyn KnowledgeGraph>,
    entity_id: &str,
    expected_properties: Option<&HashMap<String, String>>,
) {
    let entity = graph
        .get_entity(entity_id)
        .await
        .unwrap_or_else(|_| panic!("Entity '{}' not found", entity_id));

    if let Some(expected) = expected_properties {
        for (key, expected_value) in expected {
            let actual_value = entity
                .properties
                .get(key)
                .unwrap_or_else(|| panic!("Entity '{}' missing property '{}'", entity_id, key));
            assert_eq!(
                actual_value, expected_value,
                "Entity '{}' property '{}' mismatch",
                entity_id, key
            );
        }
    }
}

/// Assert relationship exists in knowledge graph
///
/// # Arguments
///
/// * `graph` - Knowledge graph to query
/// * `from_id` - Source entity ID
/// * `to_id` - Target entity ID
/// * `rel_type` - Relationship type
///
/// # Panics
///
/// Panics if relationship does not exist
#[allow(dead_code)]
pub async fn assert_relationship_exists(
    graph: &Arc<dyn KnowledgeGraph>,
    from_id: &str,
    to_id: &str,
    rel_type: &str,
) {
    // Use get_related to verify the relationship exists
    let related = graph
        .get_related(from_id, rel_type)
        .await
        .unwrap_or_else(|_| panic!("Failed to get related entities for '{}'", from_id));

    let found = related.iter().any(|entity| entity.id == to_id);

    assert!(
        found,
        "Relationship '{}' -[{}]-> '{}' not found",
        from_id, rel_type, to_id
    );
}

/// Assert entity does NOT exist in knowledge graph (for DELETE tests)
///
/// # Arguments
///
/// * `graph` - Knowledge graph to query
/// * `entity_id` - Entity ID that should not exist
#[allow(dead_code)]
pub async fn assert_entity_not_exists(graph: &Arc<dyn KnowledgeGraph>, entity_id: &str) {
    match graph.get_entity(entity_id).await {
        Ok(entity) => {
            // Check if entity is tombstoned (has _deleted metadata)
            if let Some(deleted) = entity.properties.get("_deleted") {
                if deleted == "true" {
                    // Entity properly tombstoned - acceptable
                    return;
                }
            }
            panic!("Entity '{}' should not exist or should be deleted", entity_id);
        }
        Err(_) => {
            // Entity not found - expected
        }
    }
}

/// Calculate Decision Match Rate (DMR)
///
/// DMR = (matching_decisions / total_decisions) * 100
///
/// Matching criteria:
/// - Decision type must match (ADD/UPDATE/DELETE/NOOP)
/// - For entity decisions (ADD/UPDATE/DELETE), entity_id must match
///
/// # Arguments
///
/// * `actual_decisions` - Decisions made by LLM
/// * `ground_truth` - Expected decisions
///
/// # Returns
///
/// DMR as a ratio (0.0 to 1.0)
#[allow(clippy::cast_precision_loss, dead_code)]
pub fn calculate_dmr(
    actual_decisions: &[DecisionPayload],
    ground_truth: &[GroundTruthDecision],
) -> f64 {
    if ground_truth.is_empty() {
        return 1.0; // No ground truth = perfect match
    }

    let mut matching = 0;
    let total = ground_truth.len();

    for gt_decision in ground_truth {
        // Find matching actual decision
        let matches = actual_decisions.iter().any(|actual| {
            match (actual, gt_decision) {
                (DecisionPayload::Add { entity_id: actual_id }, GroundTruthDecision::Add { entity_id: expected_id }) => {
                    actual_id == expected_id
                }
                (DecisionPayload::Update { entity_id: actual_id, .. }, GroundTruthDecision::Update { entity_id: expected_id }) => {
                    actual_id == expected_id
                }
                (DecisionPayload::Delete { entity_id: actual_id }, GroundTruthDecision::Delete { entity_id: expected_id }) => {
                    actual_id == expected_id
                }
                (DecisionPayload::Noop, GroundTruthDecision::Noop) => true,
                _ => false,
            }
        });

        if matches {
            matching += 1;
        }
    }

    matching as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dmr_perfect_match() {
        let actual = vec![
            DecisionPayload::Add {
                entity_id: "rust".to_string(),
            },
            DecisionPayload::Noop,
        ];

        let ground_truth = vec![
            GroundTruthDecision::Add {
                entity_id: "rust".to_string(),
            },
            GroundTruthDecision::Noop,
        ];

        let dmr = calculate_dmr(&actual, &ground_truth);
        assert!((dmr - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_dmr_partial_match() {
        let actual = vec![
            DecisionPayload::Add {
                entity_id: "rust".to_string(),
            },
            DecisionPayload::Add {
                entity_id: "wrong".to_string(),
            },
        ];

        let ground_truth = vec![
            GroundTruthDecision::Add {
                entity_id: "rust".to_string(),
            },
            GroundTruthDecision::Add {
                entity_id: "python".to_string(),
            },
        ];

        let dmr = calculate_dmr(&actual, &ground_truth);
        assert!((dmr - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_dmr_no_match() {
        let actual = vec![DecisionPayload::Noop];

        let ground_truth = vec![GroundTruthDecision::Add {
            entity_id: "rust".to_string(),
        }];

        let dmr = calculate_dmr(&actual, &ground_truth);
        assert!((dmr - 0.0).abs() < 0.01);
    }
}
