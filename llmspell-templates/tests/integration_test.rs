//! ABOUTME: Integration tests for template system
//! ABOUTME: End-to-end testing of template execution with mocked infrastructure

use llmspell_agents::FactoryRegistry;
use llmspell_providers::ProviderManager;
use llmspell_templates::{ExecutionContext, TemplateCategory, TemplateError, TemplateRegistry};
use llmspell_tools::ToolRegistry;
use llmspell_workflows::factory::DefaultWorkflowFactory;
use serde_json::json;
use std::sync::Arc;

/// Test template registry initialization with builtin templates
#[test]
fn test_registry_initialization() {
    let registry = TemplateRegistry::with_builtin_templates()
        .expect("Failed to create registry with builtin templates");

    // Should have all 8 builtin templates
    let templates = registry.list_metadata();
    assert!(
        templates.len() >= 8,
        "Expected at least 8 builtin templates, found {}",
        templates.len()
    );

    // Verify each template exists
    let expected_templates = [
        "research-assistant",
        "interactive-chat",
        "data-analysis",
        "code-generator",
        "document-processor",
        "workflow-orchestrator",
        "code-review",
        "content-generation",
    ];

    for template_id in &expected_templates {
        assert!(
            registry.get(template_id).is_ok(),
            "Template '{}' not found in registry",
            template_id
        );
    }
}

/// Test template discovery by category
#[test]
fn test_template_discovery_by_category() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    // Discover research templates
    let research_templates = registry.discover_by_category(&TemplateCategory::Research);
    assert!(
        !research_templates.is_empty(),
        "Should find at least one research template"
    );

    // Verify category matches
    for template in &research_templates {
        assert_eq!(
            template.category,
            TemplateCategory::Research,
            "Template category mismatch"
        );
    }
}

/// Test template search functionality
#[test]
fn test_template_search() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    // Search for "research"
    let search_results = registry.search("research");
    assert!(
        !search_results.is_empty(),
        "Should find templates matching 'research'"
    );

    // Search for "assistant"
    let search_results = registry.search("assistant");
    assert!(
        !search_results.is_empty(),
        "Should find templates matching 'assistant'"
    );

    // Search for nonexistent term
    let search_results = registry.search("nonexistent_term_xyz");
    assert!(
        search_results.is_empty(),
        "Should not find templates for nonexistent term"
    );
}

/// Test parameter validation across templates
#[test]
fn test_parameter_validation_required_fields() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    let template = registry
        .get("research-assistant")
        .expect("Failed to get research-assistant template");

    // Test missing required parameter
    let params = json!({
        // Missing "topic" which is required
        "max_sources": 10
    });

    let result = template.validate(&params.into());
    assert!(
        result.is_err(),
        "Should fail validation without required 'topic' parameter"
    );

    // Verify it's a validation error
    if let Err(TemplateError::ValidationFailed(_)) = result {
        // Expected error type
    } else {
        panic!("Expected ValidationFailed error, got {:?}", result);
    }
}

/// Test parameter validation with valid params
#[test]
fn test_parameter_validation_success() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    let template = registry
        .get("research-assistant")
        .expect("Failed to get research-assistant template");

    // Test with all required parameters
    let params = json!({
        "topic": "Rust async programming",
        "max_sources": 15,
        "output_format": "markdown"
    });

    let result = template.validate(&params.into());
    assert!(
        result.is_ok(),
        "Should pass validation with valid parameters: {:?}",
        result
    );
}

/// Test parameter validation with constraint violations
#[test]
fn test_parameter_validation_constraints() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    let template = registry
        .get("research-assistant")
        .expect("Failed to get research-assistant template");

    // Test max_sources out of range (should be 1-50)
    let params = json!({
        "topic": "Rust async programming",
        "max_sources": 100 // Out of range
    });

    let result = template.validate(&params.into());
    assert!(
        result.is_err(),
        "Should fail validation with max_sources out of range"
    );
}

/// Test ExecutionContext builder with minimal components
#[test]
fn test_execution_context_builder_minimal() {
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .build();

    assert!(
        context.is_ok(),
        "Should successfully build ExecutionContext with core components"
    );

    let context = context.unwrap();

    // Verify registries are available
    let _ = context.tool_registry();
    let _ = context.agent_registry();
    let _ = context.workflow_factory();
}

/// Test ExecutionContext builder missing required components
#[test]
fn test_execution_context_builder_missing_components() {
    let builder = ExecutionContext::builder();

    // Should fail without required components
    let result = builder.build();
    assert!(
        result.is_err(),
        "Should fail to build ExecutionContext without required components"
    );
}

/// Test template metadata completeness
#[test]
fn test_template_metadata_completeness() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    let templates = registry.list_metadata();

    for metadata in templates {
        // Verify required fields are present
        assert!(!metadata.id.is_empty(), "Template ID should not be empty");
        assert!(
            !metadata.name.is_empty(),
            "Template name should not be empty"
        );
        assert!(
            !metadata.description.is_empty(),
            "Template description should not be empty"
        );
        assert!(
            !metadata.version.is_empty(),
            "Template version should not be empty"
        );

        // Verify category is valid
        match metadata.category {
            TemplateCategory::Research
            | TemplateCategory::Chat
            | TemplateCategory::Analysis
            | TemplateCategory::CodeGen
            | TemplateCategory::Document
            | TemplateCategory::Workflow
            | TemplateCategory::Custom(_) => {
                // Valid category
            }
        }
    }
}

/// Test template config schema availability
#[test]
fn test_template_config_schema() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    let template = registry
        .get("research-assistant")
        .expect("Failed to get template");

    let schema = template.config_schema();

    // Schema should have parameters
    assert!(
        !schema.parameters.is_empty(),
        "Config schema should have parameters"
    );

    // Verify required parameter exists
    let topic_param = schema.parameters.iter().find(|p| p.name == "topic");
    assert!(topic_param.is_some(), "Should have 'topic' parameter");

    let topic_param = topic_param.unwrap();
    assert!(topic_param.required, "'topic' parameter should be required");
}

/// Test template registry registration and retrieval
#[test]
fn test_registry_register_custom_template() {
    let registry = TemplateRegistry::new();

    // Register builtin templates first
    let builtin_registry =
        TemplateRegistry::with_builtin_templates().expect("Failed to create builtin registry");

    let research_template = builtin_registry
        .get("research-assistant")
        .expect("Failed to get template")
        .clone();

    // Register template (ID comes from template metadata)
    registry
        .register(research_template.clone())
        .expect("Failed to register template");

    // Retrieve and verify (use template's actual ID)
    let template_id = research_template.metadata().id.clone();
    let retrieved = registry
        .get(&template_id)
        .expect("Failed to retrieve registered template");

    assert_eq!(
        retrieved.metadata().id,
        research_template.metadata().id,
        "Retrieved template metadata should match original"
    );
}

/// Test multi-template workflow
#[test]
fn test_multi_template_discovery_workflow() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    // 1. List all templates
    let all_templates = registry.list_metadata();
    assert!(all_templates.len() >= 8, "Should have at least 8 templates");

    // 2. Search for research-related templates
    let research_results = registry.search("research");
    assert!(
        !research_results.is_empty(),
        "Should find research templates"
    );

    // 3. Get specific template
    let template = registry.get("research-assistant");
    assert!(template.is_ok(), "Should retrieve specific template");

    // 4. Verify metadata
    let template = template.unwrap();
    assert_eq!(
        template.metadata().id,
        "research-assistant",
        "Template ID should match"
    );
}

/// Test error propagation in template operations
#[test]
fn test_error_propagation() {
    let registry = TemplateRegistry::new();

    // Test get nonexistent template
    let result = registry.get("nonexistent-template");
    assert!(
        result.is_err(),
        "Should return error for nonexistent template"
    );

    // Test validation error
    let builtin_registry =
        TemplateRegistry::with_builtin_templates().expect("Failed to create builtin registry");
    let template = builtin_registry
        .get("research-assistant")
        .expect("Failed to get template");

    let invalid_params = json!({
        // Missing required "topic" parameter
        "max_sources": 10
    });

    let result = template.validate(&invalid_params.into());
    assert!(result.is_err(), "Should return validation error");
}

/// Test template cost estimation
#[tokio::test]
async fn test_template_cost_estimation() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    let template = registry
        .get("research-assistant")
        .expect("Failed to get template");

    let params = json!({
        "topic": "Rust async programming",
        "max_sources": 15
    });

    let estimate = template.estimate_cost(&params.into()).await;

    // Cost estimate should have confidence score
    assert!(
        estimate.confidence >= 0.0 && estimate.confidence <= 1.0,
        "Confidence should be between 0 and 1, got {}",
        estimate.confidence
    );
}

/// Test template registry find_by_tag
#[test]
fn test_registry_find_by_tag() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    // Research assistant should have "research" tag
    let tagged_templates = registry.find_by_tag("research");
    assert!(
        !tagged_templates.is_empty(),
        "Should find templates with 'research' tag"
    );
}

/// Test template registry clear
#[test]
fn test_registry_clear() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");

    // Verify templates exist
    assert!(
        !registry.list_metadata().is_empty(),
        "Registry should have templates"
    );

    // Clear registry
    registry.clear();

    // Verify empty
    assert!(
        registry.list_metadata().is_empty(),
        "Registry should be empty after clear"
    );
}
