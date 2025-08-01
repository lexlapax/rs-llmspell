// ABOUTME: Workflow hook tests for validating hooks during complex multi-step agent interactions
// ABOUTME: Tests hook execution across sequential, parallel, and conditional workflow patterns

use crate::provider_hook_integration::common::{
    assertions::*, test_data::*, HookTestFixture,
};
use llmspell_hooks::{
    builtin::{
        metrics::MetricsHook,
        rate_limit::RateLimitHook,
    },
    context::HookContext,
    types::{ComponentId, ComponentType, HookPoint},
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_sequential_workflow_hooks() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Start workflow
    let mut workflow_context = HookContext::new(
        HookPoint::BeforeWorkflowStart,
        ComponentId::new(ComponentType::Workflow, "document-processing".to_string()),
    );
    workflow_context.correlation_id = workflow_id;
    workflow_context.insert_data(
        "workflow_config".to_string(),
        workflow_data(
            "document-processing",
            vec!["load", "validate", "parse", "transform", "save"],
        ),
    );
    workflow_context.insert_metadata("workflow_type".to_string(), "sequential".to_string());
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Execute workflow steps sequentially
    let steps = vec![
        ("load", json!({"source": "input.pdf", "pages": 10})),
        ("validate", json!({"format": "pdf", "max_size": "10MB"})),
        ("parse", json!({"extract": ["text", "images", "tables"]})),
        ("transform", json!({"format": "markdown", "preserve_formatting": true})),
        ("save", json!({"destination": "output.md", "compress": false})),
    ];
    
    for (idx, (step_name, step_data)) in steps.iter().enumerate() {
        // Before step execution
        let mut step_context = HookContext::new(
            HookPoint::BeforeStepExecution,
            ComponentId::new(ComponentType::Workflow, step_name.to_string()),
        );
        step_context.correlation_id = workflow_id;
        step_context.insert_data("step_config".to_string(), step_data.clone());
        step_context.insert_metadata("step_index".to_string(), idx.to_string());
        step_context.insert_metadata("total_steps".to_string(), steps.len().to_string());
        
        fixture.execute_and_persist(&mut step_context).await.unwrap();
        
        // Simulate step execution
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // After step execution
        step_context.point = HookPoint::AfterStepExecution;
        step_context.insert_data(
            "step_result".to_string(),
            json!({
                "status": "success",
                "duration_ms": 95,
                "output": format!("{}_completed", step_name)
            }),
        );
        
        fixture.execute_and_persist(&mut step_context).await.unwrap();
    }
    
    // Complete workflow
    workflow_context.point = HookPoint::AfterWorkflowComplete;
    workflow_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "completed",
            "total_duration_ms": 500,
            "steps_completed": 5,
            "output_file": "output.md"
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Verify workflow execution
    let stored = fixture
        .storage
        .load_by_correlation_id(&workflow_id.to_string())
        .await
        .unwrap();
    
    // Should have events for workflow start/end + each step before/after
    let min_expected_events = 2 + (steps.len() * 2);
    assert!(
        stored.len() >= min_expected_events * 5, // Multiple hooks per event
        "Insufficient events for sequential workflow"
    );
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_parallel_workflow_hooks() {
    let fixture = HookTestFixture::new().await;
    
    // Configure metrics to track parallel execution
    let metrics_config = MetricsConfig {
        enable_detailed_tracking: true,
        track_workflow_metrics: true,
        track_parallel_execution: true,
        ..Default::default()
    };
    
    let metrics_hook = Arc::new(
        llmspell_hooks::builtin::MetricsHook::with_config(metrics_config),
    );
    fixture.executor.register_hook(metrics_hook.clone());
    
    let workflow_id = Uuid::new_v4();
    
    // Start parallel workflow
    let mut workflow_context = HookContext::new(
        HookPoint::BeforeWorkflowStart,
        ComponentId::new(ComponentType::Workflow, "multi-source-aggregation".to_string()),
    );
    workflow_context.correlation_id = workflow_id;
    workflow_context.insert_data(
        "workflow_config".to_string(),
        json!({
            "name": "multi-source-aggregation",
            "type": "parallel",
            "branches": [
                {"name": "fetch-api-1", "timeout": 5000},
                {"name": "fetch-api-2", "timeout": 5000},
                {"name": "fetch-api-3", "timeout": 5000}
            ],
            "merge_strategy": "combine_all"
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Execute parallel branches
    let branches = vec![
        ("fetch-api-1", "weather", 150),
        ("fetch-api-2", "news", 200),
        ("fetch-api-3", "stocks", 100),
    ];
    
    let mut handles = vec![];
    
    for (branch_name, data_type, delay_ms) in branches {
        let fixture_clone = &fixture;
        let workflow_id = workflow_id;
        
        let handle = tokio::spawn(async move {
            let mut branch_context = HookContext::new(
                HookPoint::BeforeStepExecution,
                ComponentId::new(ComponentType::Workflow, branch_name.to_string()),
            );
            branch_context.correlation_id = workflow_id;
            branch_context.insert_data(
                "branch_config".to_string(),
                json!({
                    "data_type": data_type,
                    "parallel": true
                }),
            );
            branch_context.insert_metadata("execution_mode".to_string(), "parallel".to_string());
            
            fixture_clone.execute_and_persist(&mut branch_context).await.unwrap();
            
            // Simulate API call
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            
            branch_context.point = HookPoint::AfterStepExecution;
            branch_context.insert_data(
                "branch_result".to_string(),
                json!({
                    "status": "success",
                    "data": format!("{}_data", data_type),
                    "records": 100
                }),
            );
            
            fixture_clone.execute_and_persist(&mut branch_context).await.unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all branches
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Merge results
    let mut merge_context = HookContext::new(
        HookPoint::BeforeStepExecution,
        ComponentId::new(ComponentType::Workflow, "merge-results".to_string()),
    );
    merge_context.correlation_id = workflow_id;
    merge_context.insert_data(
        "merge_operation".to_string(),
        json!({
            "strategy": "combine_all",
            "sources": ["fetch-api-1", "fetch-api-2", "fetch-api-3"]
        }),
    );
    
    fixture.execute_and_persist(&mut merge_context).await.unwrap();
    
    // Complete workflow
    workflow_context.point = HookPoint::AfterWorkflowComplete;
    workflow_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "completed",
            "parallel_branches": 3,
            "total_records": 300
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Verify metrics
    let metrics = metrics_hook.get_metrics();
    assert!(
        metrics.operations_by_type.contains_key(&"Workflow".to_string()),
        "Workflow metrics not tracked"
    );
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_conditional_workflow_hooks() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Start conditional workflow
    let mut workflow_context = HookContext::new(
        HookPoint::BeforeWorkflowStart,
        ComponentId::new(ComponentType::Workflow, "smart-routing".to_string()),
    );
    workflow_context.correlation_id = workflow_id;
    workflow_context.insert_data(
        "workflow_config".to_string(),
        json!({
            "name": "smart-routing",
            "type": "conditional",
            "initial_condition": "check_input_type"
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Step 1: Check condition
    let mut condition_context = HookContext::new(
        HookPoint::BeforeStepExecution,
        ComponentId::new(ComponentType::Workflow, "check_input_type".to_string()),
    );
    condition_context.correlation_id = workflow_id;
    condition_context.insert_data(
        "input".to_string(),
        json!({
            "type": "image",
            "size": 1024000,
            "format": "jpeg"
        }),
    );
    
    fixture.execute_and_persist(&mut condition_context).await.unwrap();
    
    // Condition evaluation
    condition_context.point = HookPoint::AfterStepExecution;
    condition_context.insert_data(
        "condition_result".to_string(),
        json!({
            "branch": "image_processing",
            "reason": "Input type is image"
        }),
    );
    
    fixture.execute_and_persist(&mut condition_context).await.unwrap();
    
    // Branch: Image processing
    let mut image_branch = HookContext::new(
        HookPoint::BeforeStepExecution,
        ComponentId::new(ComponentType::Workflow, "image_processing".to_string()),
    );
    image_branch.correlation_id = workflow_id;
    image_branch.insert_metadata("branch_taken".to_string(), "image_processing".to_string());
    image_branch.insert_data(
        "processing_config".to_string(),
        json!({
            "operations": ["resize", "compress", "watermark"]
        }),
    );
    
    fixture.execute_and_persist(&mut image_branch).await.unwrap();
    
    // Image processing complete
    image_branch.point = HookPoint::AfterStepExecution;
    image_branch.insert_data(
        "processing_result".to_string(),
        json!({
            "status": "success",
            "output_size": 512000,
            "compression_ratio": 0.5
        }),
    );
    
    fixture.execute_and_persist(&mut image_branch).await.unwrap();
    
    // Workflow complete
    workflow_context.point = HookPoint::AfterWorkflowComplete;
    workflow_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "completed",
            "path_taken": "check_input_type -> image_processing",
            "skipped_branches": ["text_processing", "audio_processing"]
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Verify conditional execution
    let stored = fixture
        .storage
        .load_by_correlation_id(&workflow_id.to_string())
        .await
        .unwrap();
    
    // Check that only the taken branch was executed
    let mut executed_branches = std::collections::HashSet::new();
    for (_, data) in &stored {
        if let Ok(context) = serde_json::from_slice::<HookContext>(data) {
            if let Some(branch) = context.metadata.get("branch_taken") {
                executed_branches.insert(branch.clone());
            }
        }
    }
    
    assert!(executed_branches.contains("image_processing"));
    assert!(!executed_branches.contains("text_processing"));
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_workflow_error_handling_hooks() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Start workflow that will encounter an error
    let mut workflow_context = HookContext::new(
        HookPoint::BeforeWorkflowStart,
        ComponentId::new(ComponentType::Workflow, "data-pipeline".to_string()),
    );
    workflow_context.correlation_id = workflow_id;
    workflow_context.insert_data(
        "workflow_config".to_string(),
        workflow_data("data-pipeline", vec!["extract", "transform", "load"]),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Step 1: Extract - Success
    let mut extract_context = HookContext::new(
        HookPoint::BeforeStepExecution,
        ComponentId::new(ComponentType::Workflow, "extract".to_string()),
    );
    extract_context.correlation_id = workflow_id;
    fixture.execute_and_persist(&mut extract_context).await.unwrap();
    
    extract_context.point = HookPoint::AfterStepExecution;
    extract_context.insert_data(
        "result".to_string(),
        json!({"status": "success", "records": 1000}),
    );
    fixture.execute_and_persist(&mut extract_context).await.unwrap();
    
    // Step 2: Transform - Error
    let mut transform_context = HookContext::new(
        HookPoint::BeforeStepExecution,
        ComponentId::new(ComponentType::Workflow, "transform".to_string()),
    );
    transform_context.correlation_id = workflow_id;
    fixture.execute_and_persist(&mut transform_context).await.unwrap();
    
    // Error occurs
    transform_context.point = HookPoint::WorkflowError;
    transform_context.insert_metadata("error".to_string(), "Invalid data format".to_string());
    transform_context.insert_data(
        "error_details".to_string(),
        json!({
            "step": "transform",
            "error_type": "DataValidationError",
            "message": "Expected JSON, got XML",
            "recoverable": false
        }),
    );
    
    fixture.execute_and_persist(&mut transform_context).await.unwrap();
    
    // Workflow handles error
    workflow_context.point = HookPoint::AfterWorkflowComplete;
    workflow_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "failed",
            "completed_steps": ["extract"],
            "failed_step": "transform",
            "error": "Invalid data format",
            "cleanup_performed": true
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Verify error handling
    assert_hook_persisted(&fixture.storage, &workflow_id.to_string(), "SecurityHook").await;
    assert_hook_persisted(&fixture.storage, &workflow_id.to_string(), "LoggingHook").await;
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_workflow_rate_limiting() {
    let fixture = HookTestFixture::new().await;
    
    // Configure rate limiting for workflows
    let rate_config = RateLimitConfig {
        requests_per_minute: Some(2), // Very low for testing
        concurrent_workflows: Some(1),
        ..Default::default()
    };
    
    let rate_hook = Arc::new(
        llmspell_hooks::builtin::RateLimitHook::with_config(rate_config),
    );
    fixture.executor.register_hook(rate_hook.clone());
    
    // Try to start multiple workflows rapidly
    let mut workflow_results = vec![];
    
    for i in 0..3 {
        let mut context = HookContext::new(
            HookPoint::BeforeWorkflowStart,
            ComponentId::new(ComponentType::Workflow, format!("workflow-{}", i)),
        );
        context.insert_data(
            "workflow_config".to_string(),
            workflow_data(&format!("test-workflow-{}", i), vec!["step1", "step2"]),
        );
        
        let result = fixture.executor.execute(&mut context).await.unwrap();
        workflow_results.push((i, result));
        
        // Small delay between attempts
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // First 2 should succeed, 3rd should be rate limited
    assert_hook_success(&workflow_results[0].1);
    assert_hook_success(&workflow_results[1].1);
    assert!(
        matches!(
            workflow_results[2].1,
            llmspell_hooks::result::HookResult::Cancel(_)
        ),
        "Third workflow was not rate limited"
    );
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_nested_workflow_hooks() {
    let fixture = HookTestFixture::new().await;
    
    let parent_workflow_id = Uuid::new_v4();
    
    // Parent workflow
    let mut parent_context = HookContext::new(
        HookPoint::BeforeWorkflowStart,
        ComponentId::new(ComponentType::Workflow, "orchestrator".to_string()),
    );
    parent_context.correlation_id = parent_workflow_id;
    parent_context.insert_data(
        "workflow_config".to_string(),
        json!({
            "name": "orchestrator",
            "steps": ["prepare", "execute_sub_workflows", "aggregate"]
        }),
    );
    
    fixture.execute_and_persist(&mut parent_context).await.unwrap();
    
    // Execute sub-workflows
    let sub_workflows = vec!["data-fetcher", "processor", "validator"];
    
    for sub_name in &sub_workflows {
        let sub_workflow_id = Uuid::new_v4();
        
        let mut sub_context = HookContext::new(
            HookPoint::BeforeWorkflowStart,
            ComponentId::new(ComponentType::Workflow, sub_name.to_string()),
        );
        sub_context.correlation_id = sub_workflow_id;
        sub_context.insert_metadata(
            "parent_workflow_id".to_string(),
            parent_workflow_id.to_string(),
        );
        sub_context.insert_metadata("nesting_level".to_string(), "1".to_string());
        sub_context.insert_data(
            "sub_workflow_config".to_string(),
            json!({
                "name": sub_name,
                "parent": "orchestrator"
            }),
        );
        
        fixture.execute_and_persist(&mut sub_context).await.unwrap();
        
        // Complete sub-workflow
        sub_context.point = HookPoint::AfterWorkflowComplete;
        sub_context.insert_data(
            "result".to_string(),
            json!({
                "status": "completed",
                "output": format!("{}_output", sub_name)
            }),
        );
        
        fixture.execute_and_persist(&mut sub_context).await.unwrap();
    }
    
    // Complete parent workflow
    parent_context.point = HookPoint::AfterWorkflowComplete;
    parent_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "completed",
            "sub_workflows_executed": 3,
            "aggregated_results": "combined_output"
        }),
    );
    
    fixture.execute_and_persist(&mut parent_context).await.unwrap();
    
    // Verify parent workflow execution
    let parent_stored = fixture
        .storage
        .load_by_correlation_id(&parent_workflow_id.to_string())
        .await
        .unwrap();
    
    assert!(!parent_stored.is_empty(), "Parent workflow events not found");
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_workflow_with_llm_and_tools() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Complex workflow: User Query -> LLM -> Tool Selection -> Tool Execution -> LLM Summary
    let mut workflow_context = HookContext::new(
        HookPoint::BeforeWorkflowStart,
        ComponentId::new(ComponentType::Workflow, "ai-assistant".to_string()),
    );
    workflow_context.correlation_id = workflow_id;
    workflow_context.insert_data(
        "workflow_config".to_string(),
        json!({
            "name": "ai-assistant",
            "steps": ["understand_query", "select_tools", "execute_tools", "generate_response"]
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Step 1: LLM understands query
    let mut llm_context = HookContext::new(
        HookPoint::BeforeAgentExecution,
        ComponentId::new(ComponentType::Agent, "gpt-4".to_string()),
    );
    llm_context.correlation_id = workflow_id;
    llm_context.insert_data(
        "request".to_string(),
        llm_request_data("gpt-4", "What's the weather in NYC and latest tech news?"),
    );
    
    fixture.execute_and_persist(&mut llm_context).await.unwrap();
    
    llm_context.point = HookPoint::AfterAgentExecution;
    llm_context.insert_data(
        "response".to_string(),
        json!({
            "tools_needed": ["weather-api", "news-api"],
            "parameters": {
                "weather-api": {"city": "New York"},
                "news-api": {"category": "technology", "limit": 5}
            }
        }),
    );
    
    fixture.execute_and_persist(&mut llm_context).await.unwrap();
    
    // Step 2: Execute selected tools
    for tool in &["weather-api", "news-api"] {
        let mut tool_context = HookContext::new(
            HookPoint::BeforeToolExecution,
            ComponentId::new(ComponentType::Tool, tool.to_string()),
        );
        tool_context.correlation_id = workflow_id;
        fixture.execute_and_persist(&mut tool_context).await.unwrap();
        
        tool_context.point = HookPoint::AfterToolExecution;
        tool_context.insert_data(
            "result".to_string(),
            json!({"status": "success", "data": format!("{}_data", tool)}),
        );
        fixture.execute_and_persist(&mut tool_context).await.unwrap();
    }
    
    // Step 3: LLM generates final response
    let mut final_llm = HookContext::new(
        HookPoint::BeforeAgentExecution,
        ComponentId::new(ComponentType::Agent, "gpt-4".to_string()),
    );
    final_llm.correlation_id = workflow_id;
    final_llm.insert_data(
        "request".to_string(),
        json!({
            "prompt": "Summarize the weather and news data",
            "context": {
                "weather": "weather_api_data",
                "news": "news_api_data"
            }
        }),
    );
    
    fixture.execute_and_persist(&mut final_llm).await.unwrap();
    
    // Complete workflow
    workflow_context.point = HookPoint::AfterWorkflowComplete;
    workflow_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "completed",
            "llm_calls": 2,
            "tool_calls": 2,
            "total_tokens": 500,
            "response": "Here's the weather in NYC and latest tech news..."
        }),
    );
    
    fixture.execute_and_persist(&mut workflow_context).await.unwrap();
    
    // Verify complete workflow execution
    let stored = fixture
        .storage
        .load_by_correlation_id(&workflow_id.to_string())
        .await
        .unwrap();
    
    // Should have many events across agents, tools, and workflow steps
    assert!(
        stored.len() >= 20,
        "Complex workflow should generate many hook events"
    );
}