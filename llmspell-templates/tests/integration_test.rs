//! ABOUTME: Integration tests for template system
//! ABOUTME: End-to-end testing of template execution with mocked infrastructure

use llmspell_agents::FactoryRegistry;
use llmspell_config::{ProviderConfig, ProviderManagerConfig};
use llmspell_kernel::state::StateManager;
use llmspell_providers::ProviderManager;
use llmspell_templates::core::TemplateResult;
use llmspell_templates::{ExecutionContext, TemplateCategory, TemplateError, TemplateRegistry};
use llmspell_tools::ToolRegistry;
use llmspell_workflows::factory::DefaultWorkflowFactory;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Helper function to create a test provider config for integration tests
fn create_test_provider_config() -> ProviderManagerConfig {
    let provider = ProviderConfig {
        name: "test-provider".to_string(),
        provider_type: "ollama".to_string(),
        enabled: true,
        base_url: Some("http://localhost:11434".to_string()),
        api_key_env: None,
        api_key: None,
        default_model: Some("llama3.2:3b".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2000),
        timeout_seconds: Some(30),
        rate_limit: None,
        retry: None,
        max_retries: Some(3),
        options: HashMap::new(),
    };

    let mut providers = HashMap::new();
    providers.insert("test-provider".to_string(), provider);

    ProviderManagerConfig {
        default_provider: Some("test-provider".to_string()),
        providers,
    }
}

/// Test template registry initialization with builtin templates
#[test]
fn test_registry_initialization() {
    let registry = TemplateRegistry::with_builtin_templates()
        .expect("Failed to create registry with builtin templates");

    // Should have all 10 builtin templates
    let templates = registry.list_metadata();
    assert!(
        templates.len() >= 10,
        "Expected at least 10 builtin templates, found {}",
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
        "file-classification",
        "knowledge-management",
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
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
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
    assert!(all_templates.len() >= 9, "Should have at least 9 templates");

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

// ============================================================================
// Knowledge Management Template Tests
// ============================================================================

/// Helper function to extract text from TemplateResult
fn extract_text(result: &TemplateResult) -> String {
    match result {
        TemplateResult::Text(s) => s.clone(),
        _ => panic!("Expected Text result"),
    }
}

/// Test knowledge management template ingest operation
#[tokio::test]
async fn test_knowledge_management_ingest() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    // Create execution context with StateManager
    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager)
        .build()
        .expect("Failed to build execution context");

    // Test ingest operation
    let params = json!({
        "operation": "ingest",
        "collection": "test-collection",
        "content": "This is a test document about Rust programming language. Rust is a systems programming language.",
        "source_type": "text",
        "chunk_size": 100,
        "chunk_overlap": 20
    });

    let result = template
        .execute(params.into(), context)
        .await
        .expect("Ingest operation failed");

    assert!(extract_text(&result.result).contains("Document ingested successfully"));
    assert!(result.metrics.custom_metrics.contains_key("document_id"));
    assert!(result.metrics.custom_metrics.contains_key("chunks_created"));
}

/// Test knowledge management template query operation
#[tokio::test]
async fn test_knowledge_management_query() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager.clone())
        .build()
        .expect("Failed to build execution context");

    // First ingest a document
    let ingest_params = json!({
        "operation": "ingest",
        "collection": "test-query-collection",
        "content": "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.",
        "source_type": "text"
    });

    template
        .execute(ingest_params.into(), context.clone())
        .await
        .expect("Ingest failed");

    // Then query it
    let query_params = json!({
        "operation": "query",
        "collection": "test-query-collection",
        "query": "Rust programming language",
        "max_results": 5,
        "include_citations": true
    });

    let result = template
        .execute(query_params.into(), context)
        .await
        .expect("Query operation failed");

    let output = extract_text(&result.result);
    assert!(output.contains("KNOWLEDGE QUERY RESULTS"));
    assert!(output.contains("Rust"));
}

/// Test knowledge management template update operation
#[tokio::test]
async fn test_knowledge_management_update() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager.clone())
        .build()
        .expect("Failed to build execution context");

    // Ingest original document
    let ingest_result = template
        .execute(
            json!({
                "operation": "ingest",
                "collection": "test-update-collection",
                "content": "Original content about Python",
                "source_type": "text"
            })
            .into(),
            context.clone(),
        )
        .await
        .expect("Ingest failed");

    let doc_id = ingest_result
        .metrics
        .custom_metrics
        .get("document_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    // Update the document
    let update_result = template
        .execute(
            json!({
                "operation": "update",
                "collection": "test-update-collection",
                "document_id": doc_id,
                "content": "Updated content about Rust",
                "source_type": "text"
            })
            .into(),
            context,
        )
        .await
        .expect("Update operation failed");

    assert!(extract_text(&update_result.result).contains("Document updated successfully"));
}

/// Test knowledge management template delete operation
#[tokio::test]
async fn test_knowledge_management_delete() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager.clone())
        .build()
        .expect("Failed to build execution context");

    // Ingest document
    let ingest_result = template
        .execute(
            json!({
                "operation": "ingest",
                "collection": "test-delete-collection",
                "content": "Document to be deleted",
                "source_type": "text"
            })
            .into(),
            context.clone(),
        )
        .await
        .expect("Ingest failed");

    let doc_id = ingest_result
        .metrics
        .custom_metrics
        .get("document_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    // Delete the document
    let delete_result = template
        .execute(
            json!({
                "operation": "delete",
                "collection": "test-delete-collection",
                "document_id": doc_id
            })
            .into(),
            context,
        )
        .await
        .expect("Delete operation failed");

    assert!(extract_text(&delete_result.result).contains("Document deleted successfully"));
}

/// Test knowledge management template list operation
#[tokio::test]
async fn test_knowledge_management_list() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager.clone())
        .build()
        .expect("Failed to build execution context");

    // Ingest multiple documents
    for i in 1..=3 {
        template
            .execute(
                json!({
                    "operation": "ingest",
                    "collection": "test-list-collection",
                    "content": format!("Document {} about Rust", i),
                    "source_type": "text"
                })
                .into(),
                context.clone(),
            )
            .await
            .expect("Ingest failed");
    }

    // List documents
    let list_result = template
        .execute(
            json!({
                "operation": "list",
                "collection": "test-list-collection",
                "output_format": "text"
            })
            .into(),
            context,
        )
        .await
        .expect("List operation failed");

    let output = extract_text(&list_result.result);
    assert!(output.contains("KNOWLEDGE BASE: test-list-collection"));
    assert!(output.contains("Total Documents: 3"));
}

/// Test knowledge management full CRUD cycle
#[tokio::test]
async fn test_knowledge_management_full_cycle() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager.clone())
        .build()
        .expect("Failed to build execution context");

    // 1. Ingest
    let ingest_result = template
        .execute(
            json!({
                "operation": "ingest",
                "collection": "test-full-cycle",
                "content": "Rust is a systems programming language",
                "source_type": "text"
            })
            .into(),
            context.clone(),
        )
        .await
        .expect("Ingest failed");

    let doc_id = ingest_result
        .metrics
        .custom_metrics
        .get("document_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    // 2. Query
    let query_result = template
        .execute(
            json!({
                "operation": "query",
                "collection": "test-full-cycle",
                "query": "Rust programming",
                "max_results": 5
            })
            .into(),
            context.clone(),
        )
        .await
        .expect("Query failed");

    assert!(extract_text(&query_result.result).contains("Rust"));

    // 3. Update
    let update_result = template
        .execute(
            json!({
                "operation": "update",
                "collection": "test-full-cycle",
                "document_id": doc_id.clone(),
                "content": "Rust is a memory-safe systems programming language",
                "source_type": "text"
            })
            .into(),
            context.clone(),
        )
        .await
        .expect("Update failed");

    assert!(extract_text(&update_result.result).contains("Document updated successfully"));

    // 4. Query again to verify update
    let query_result2 = template
        .execute(
            json!({
                "operation": "query",
                "collection": "test-full-cycle",
                "query": "memory-safe",
                "max_results": 5
            })
            .into(),
            context.clone(),
        )
        .await
        .expect("Query failed");

    assert!(extract_text(&query_result2.result).contains("memory-safe"));

    // 5. Delete
    let delete_result = template
        .execute(
            json!({
                "operation": "delete",
                "collection": "test-full-cycle",
                "document_id": doc_id
            })
            .into(),
            context,
        )
        .await
        .expect("Delete failed");

    assert!(extract_text(&delete_result.result).contains("Document deleted successfully"));
}

/// Test knowledge management error handling
#[tokio::test]
async fn test_knowledge_management_error_handling() {
    let registry = TemplateRegistry::with_builtin_templates().expect("Failed to create registry");
    let template = registry
        .get("knowledge-management")
        .expect("Failed to get knowledge-management template");

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );
    let tool_registry = Arc::new(ToolRegistry::new());
    let agent_registry = Arc::new(FactoryRegistry::new());
    let workflow_factory = Arc::new(DefaultWorkflowFactory::new());
    let provider_manager = Arc::new(ProviderManager::new());
    let provider_config = Arc::new(create_test_provider_config());

    let context = ExecutionContext::builder()
        .with_tool_registry(tool_registry)
        .with_agent_registry(agent_registry)
        .with_workflow_factory(workflow_factory)
        .with_providers(provider_manager)
        .with_provider_config(provider_config)
        .with_state_manager(state_manager)
        .build()
        .expect("Failed to build execution context");

    // Test query on empty collection
    let query_result = template
        .execute(
            json!({
                "operation": "query",
                "collection": "nonexistent-collection",
                "query": "test query"
            })
            .into(),
            context.clone(),
        )
        .await;

    assert!(
        query_result.is_err(),
        "Should fail querying empty collection"
    );

    // Test delete nonexistent document
    let delete_result = template
        .execute(
            json!({
                "operation": "delete",
                "collection": "nonexistent-collection",
                "document_id": "nonexistent-doc"
            })
            .into(),
            context,
        )
        .await;

    assert!(
        delete_result.is_err(),
        "Should fail deleting nonexistent document"
    );
}
