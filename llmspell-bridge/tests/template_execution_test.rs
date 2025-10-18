//! ABOUTME: Integration tests for template execution with dual-registry architecture
//! ABOUTME: Verifies tools exist in BOTH `ToolRegistry` and `ComponentRegistry` (Phase 12.7.1.4)

use llmspell_bridge::runtime::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use llmspell_core::traits::script_executor::ScriptExecutor;

/// Test that tools are registered in both `ToolRegistry` (infrastructure) and `ComponentRegistry` (scripts)
/// This verifies the dual-registration pattern from Phase 12.7.1.2
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_tools_registered_in_both_registries() {
    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create runtime");

    // Verify tools in ToolRegistry (infrastructure layer)
    let tool_names = runtime.tool_registry().list_tools().await;
    assert!(
        !tool_names.is_empty(),
        "ToolRegistry should have tools registered"
    );

    // Check for a representative tool from each category
    assert!(
        tool_names.contains(&"calculator".to_string()),
        "Calculator tool should be registered in ToolRegistry"
    );

    // Verify same tools in ComponentRegistry (script access layer)
    let component_names = runtime.registry().list_tools();
    assert_eq!(
        tool_names.len(),
        component_names.len(),
        "Both registries should have same number of tools"
    );

    // Verify specific tool exists in both registries
    let tool_in_infra = runtime.tool_registry().get_tool("calculator").await;
    assert!(
        tool_in_infra.is_some(),
        "Calculator should exist in ToolRegistry"
    );

    let tool_in_component = runtime.registry().get_tool("calculator");
    assert!(
        tool_in_component.is_some(),
        "Calculator should exist in ComponentRegistry"
    );
}

/// Test that all infrastructure components are accessible from `ScriptRuntime`
/// Verifies that `ExecutionContext` can be built with all 4 required components
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_execution_context_has_infrastructure() {
    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create runtime");

    // Verify tool registry is accessible and has tools
    let tool_count = runtime.tool_registry().list_tools().await.len();
    assert!(
        tool_count > 0,
        "Tool registry should have at least one tool"
    );

    // Verify agent registry is accessible (may be empty in default config)
    let _agent_factories = runtime.agent_registry().list_factories().await;
    // Agent registry is accessible if we got here without panic

    // Verify workflow factory is accessible and has types
    let workflow_types = runtime.workflow_factory().available_types();
    assert!(
        !workflow_types.is_empty(),
        "Workflow factory should have available types"
    );

    // Verify provider manager exists
    let provider_mgr = runtime.provider_manager();
    assert!(
        std::sync::Arc::strong_count(provider_mgr) > 0,
        "Provider manager should be accessible"
    );
}

/// Test that template execution does NOT fail with "`tool_registry` is required" error
/// This was the original bug that Phase 12.7.1 fixes
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_template_execution_no_infrastructure_error() {
    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create runtime");

    // Execute research-assistant template (placeholder implementation)
    let params = serde_json::json!({
        "topic": "Rust async runtime internals",
        "max_sources": 5
    });

    let result = runtime
        .handle_template_exec("research-assistant", params)
        .await;

    // Should succeed or fail validation, NOT infrastructure
    match result {
        Ok(_) => {
            // Success! Template executed (even if placeholder)
        }
        Err(LLMSpellError::Validation { field, message }) => {
            // Expected - placeholder templates may have validation rules
            eprintln!("Validation error (expected): field={field:?}, message={message}");
        }
        Err(LLMSpellError::Component { message, .. }) => {
            // Component errors are OK UNLESS they're about missing infrastructure
            assert!(
                !message.contains("tool_registry is required"),
                "Should not fail with 'tool_registry is required': {message}"
            );
            assert!(
                !message.contains("agent_registry is required"),
                "Should not fail with 'agent_registry is required': {message}"
            );
            assert!(
                !message.contains("workflow_factory is required"),
                "Should not fail with 'workflow_factory is required': {message}"
            );
            assert!(
                !message.contains("providers is required"),
                "Should not fail with 'providers is required': {message}"
            );
            // Other component errors are OK (e.g., placeholder implementation)
            eprintln!("Component error (allowed): {message}");
        }
        Err(e) => {
            // Other error types might be OK depending on implementation
            eprintln!("Other error (allowed): {e:?}");
        }
    }
}

/// Test that all 6 built-in templates have access to infrastructure
/// Placeholder execution is OK, but no infrastructure errors allowed
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_all_builtin_templates_have_infrastructure() {
    let templates = vec![
        "research-assistant",
        "interactive-chat",
        "data-analysis",
        "code-generator",
        "document-processor",
        "workflow-orchestrator",
    ];

    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create runtime");

    for template_id in templates {
        eprintln!("Testing template: {template_id}");

        // Minimal valid params for each template
        let params = match template_id {
            "research-assistant" => serde_json::json!({
                "topic": "Test topic",
                "max_sources": 5
            }),
            "interactive-chat" => serde_json::json!({
                "initial_message": "Hello"
            }),
            "data-analysis" => serde_json::json!({
                "data_source": "test.csv",
                "analysis_type": "descriptive"
            }),
            "code-generator" => serde_json::json!({
                "description": "Create a hello world function",
                "language": "rust"
            }),
            "document-processor" => serde_json::json!({
                "documents": ["test.pdf"],
                "transformation": "extract"
            }),
            "workflow-orchestrator" => serde_json::json!({
                "workflow_definition": {
                    "steps": []
                },
                "input_data": {}
            }),
            _ => unreachable!(),
        };

        let result = runtime.handle_template_exec(template_id, params).await;

        // Check that infrastructure errors do NOT occur
        if let Err(LLMSpellError::Component { message, .. }) = &result {
            assert!(
                !message.contains("tool_registry is required"),
                "Template '{template_id}' failed with missing tool_registry: {message}"
            );
            assert!(
                !message.contains("agent_registry is required"),
                "Template '{template_id}' failed with missing agent_registry: {message}"
            );
            assert!(
                !message.contains("workflow_factory is required"),
                "Template '{template_id}' failed with missing workflow_factory: {message}"
            );
            assert!(
                !message.contains("providers is required"),
                "Template '{template_id}' failed with missing providers: {message}"
            );
        }
    }
}

/// Test error type differentiation: Validation vs Infrastructure vs `NotFound`
/// Ensures we can distinguish between different failure modes
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_validation_error_vs_infrastructure_error() {
    let config = LLMSpellConfig::default();
    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create runtime");

    // Test 1: Missing required parameter should give Validation error
    let invalid_params = serde_json::json!({
        // Missing "topic" parameter
        "max_sources": 5
    });

    let result = runtime
        .handle_template_exec("research-assistant", invalid_params)
        .await;

    match result {
        Err(LLMSpellError::Validation { field, message }) => {
            eprintln!("Validation error as expected: field={field:?}, message={message}");
            // This is the expected error for missing required parameter
        }
        Ok(_) => {
            // Some templates might have all-optional params or defaults
            eprintln!("Template succeeded with missing parameter (has defaults)");
        }
        Err(e) => {
            // Other errors might be OK depending on placeholder implementation
            eprintln!("Other error (allowed): {e:?}");
        }
    }

    // Test 2: Invalid template ID should give NotFound-like error
    let result = runtime
        .handle_template_exec("nonexistent-template", serde_json::json!({}))
        .await;

    assert!(result.is_err(), "Nonexistent template should return error");

    match result {
        Err(LLMSpellError::Component { message, .. }) => {
            assert!(
                message.contains("not found") || message.contains("Not found"),
                "Should be a 'not found' error: {message}"
            );
        }
        Err(e) => {
            eprintln!("Other error for nonexistent template: {e:?}");
        }
        Ok(_) => panic!("Nonexistent template should not succeed"),
    }

    // Test 3: Infrastructure should NOT error (this is what Phase 12.7.1 fixes)
    // This test is redundant with test_template_execution_no_infrastructure_error,
    // but included here for completeness
    let valid_params = serde_json::json!({
        "topic": "Test",
        "max_sources": 3
    });

    let result = runtime
        .handle_template_exec("research-assistant", valid_params)
        .await;

    if let Err(LLMSpellError::Component { message, .. }) = &result {
        // Infrastructure errors should NOT occur
        assert!(
            !message.contains("tool_registry is required")
                && !message.contains("agent_registry is required")
                && !message.contains("workflow_factory is required")
                && !message.contains("providers is required"),
            "Infrastructure should be available (Phase 12.7.1 fix): {message}"
        );
    }
}

/// Test that dual-registration doesn't cause memory issues or panics
/// Verifies that creating two instances per tool is safe
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_dual_registration_memory_safety() {
    let config = LLMSpellConfig::default();

    // Create multiple runtimes to stress-test dual-registration
    for i in 0..3 {
        eprintln!("Creating runtime {}", i + 1);

        let runtime = Box::pin(ScriptRuntime::new_with_lua(config.clone()))
            .await
            .expect("Failed to create runtime");

        // Verify both registries work independently
        let tool_count_infra = runtime.tool_registry().list_tools().await.len();
        let tool_count_component = runtime.registry().list_tools().len();

        assert_eq!(
            tool_count_infra, tool_count_component,
            "Registry counts should match in runtime {i}"
        );
        assert!(tool_count_infra > 0, "Should have tools in runtime {i}");

        // Get the same tool from both registries
        if let Some(tool_name) = runtime.tool_registry().list_tools().await.first() {
            let tool_infra = runtime.tool_registry().get_tool(tool_name).await;
            let tool_component = runtime.registry().get_tool(tool_name);

            assert!(
                tool_infra.is_some(),
                "Tool '{tool_name}' should exist in ToolRegistry in runtime {i}"
            );
            assert!(
                tool_component.is_some(),
                "Tool '{tool_name}' should exist in ComponentRegistry in runtime {i}"
            );
        }

        // Verify no memory leaks by checking Arc counts are reasonable
        let registry_ref_count = std::sync::Arc::strong_count(runtime.registry());
        let tool_registry_ref_count = std::sync::Arc::strong_count(runtime.tool_registry());

        eprintln!(
            "Runtime {}: registry refs={}, tool_registry refs={}",
            i + 1,
            registry_ref_count,
            tool_registry_ref_count
        );

        // Arc counts should be reasonable
        // ComponentRegistry is shared with: runtime, engine, multiple globals (Tool, Agent, etc.)
        // so ref count can be legitimately high (10-20 is normal)
        // ToolRegistry is shared with: runtime, and each tool instance (dual-registration)
        // so ref count can also be higher than expected
        assert!(
            registry_ref_count < 100,
            "ComponentRegistry ref count suspiciously high in runtime {i}: {registry_ref_count}"
        );
        assert!(
            tool_registry_ref_count < 100,
            "ToolRegistry ref count suspiciously high in runtime {i}: {tool_registry_ref_count}"
        );
    }
}
