// ABOUTME: Event correlation tests for tracking relationships between agent, tool, and LLM interactions
// ABOUTME: Validates that hook executions properly maintain correlation IDs across complex workflows

use crate::provider_hook_integration::common::{
    assertions::*, test_data::*, HookTestFixture,
};
use llmspell_hooks::{
    context::HookContext,
    persistence::ReplayOptions,
    types::{ComponentId, ComponentType, HookPoint},
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[tokio::test]
async fn test_correlation_across_agent_and_tools() {
    let fixture = HookTestFixture::new().await;
    
    // Simulate a workflow: Agent -> Tool1 -> Tool2 -> Agent Response
    let workflow_id = Uuid::new_v4();
    
    // Step 1: Agent receives request
    let mut agent_context = fixture.create_context(
        HookPoint::BeforeAgentExecution,
        "correlation-agent",
    );
    agent_context.correlation_id = workflow_id;
    agent_context.insert_data(
        "request".to_string(),
        llm_request_data("gpt-4", "Calculate the weather forecast"),
    );
    agent_context.insert_metadata("workflow_step".to_string(), "1_agent_request".to_string());
    
    fixture.execute_and_persist(&mut agent_context).await.unwrap();
    
    // Step 2: Agent calls weather tool
    let mut tool1_context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "weather-api".to_string()),
    );
    tool1_context.correlation_id = workflow_id; // Same correlation ID
    tool1_context.insert_data(
        "tool_input".to_string(),
        tool_execution_data("weather-api", "get_forecast"),
    );
    tool1_context.insert_metadata("workflow_step".to_string(), "2_weather_tool".to_string());
    tool1_context.insert_metadata("parent_component".to_string(), "correlation-agent".to_string());
    
    fixture.execute_and_persist(&mut tool1_context).await.unwrap();
    
    // Step 3: Weather tool response
    tool1_context.point = HookPoint::AfterToolExecution;
    tool1_context.insert_data(
        "tool_output".to_string(),
        json!({
            "temperature": 72,
            "conditions": "sunny",
            "humidity": 45
        }),
    );
    
    fixture.execute_and_persist(&mut tool1_context).await.unwrap();
    
    // Step 4: Agent calls calculation tool
    let mut tool2_context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "calculator".to_string()),
    );
    tool2_context.correlation_id = workflow_id;
    tool2_context.insert_data(
        "tool_input".to_string(),
        tool_execution_data("calculator", "celsius_to_fahrenheit"),
    );
    tool2_context.insert_metadata("workflow_step".to_string(), "3_calc_tool".to_string());
    tool2_context.insert_metadata("parent_component".to_string(), "correlation-agent".to_string());
    
    fixture.execute_and_persist(&mut tool2_context).await.unwrap();
    
    // Step 5: Agent final response
    agent_context.point = HookPoint::AfterAgentExecution;
    agent_context.insert_data(
        "response".to_string(),
        llm_response_data("The weather forecast is 72°F (22°C), sunny with 45% humidity", 150),
    );
    agent_context.insert_metadata("workflow_step".to_string(), "4_agent_response".to_string());
    
    fixture.execute_and_persist(&mut agent_context).await.unwrap();
    
    // Verify all events are correlated
    let stored = fixture.stored_executions.read().await;
    let executions = stored.get(&workflow_id.to_string())
        .expect("No executions found for workflow");
    
    // Should have multiple hook executions for each step
    assert!(executions.len() >= 4, "Expected at least 4 hook executions across all steps");
    
    // Verify workflow steps are tracked
    let workflow_steps = HashSet::from([
        "1_agent_request",
        "2_weather_tool",
        "3_calc_tool",
        "4_agent_response",
    ]);
    
    let mut found_steps = HashSet::new();
    for execution in executions {
        // Deserialize the context to check metadata
        if execution.metadata.contains_key("workflow_step") {
            if let Some(step) = execution.metadata.get("workflow_step") {
                if let Some(step_str) = step.as_str() {
                    found_steps.insert(step_str.to_string());
                }
            }
        }
    }
    
    for step in &workflow_steps {
        assert!(
            found_steps.contains(*step),
            "Workflow step {} not found in correlated events",
            step
        );
    }
}

#[tokio::test]
async fn test_correlation_with_parallel_tools() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Agent initiates parallel tool calls
    let mut agent_context = fixture.create_context(
        HookPoint::BeforeAgentExecution,
        "parallel-agent",
    );
    agent_context.correlation_id = workflow_id;
    agent_context.insert_data(
        "request".to_string(),
        llm_request_data("gpt-4", "Get weather and news for multiple cities"),
    );
    
    fixture.execute_and_persist(&mut agent_context).await.unwrap();
    
    // Parallel tool executions
    let cities = vec!["New York", "London", "Tokyo"];
    let tools = vec!["weather-api", "news-api"];
    
    let mut tool_futures = vec![];
    
    for city in &cities {
        for tool in &tools {
            let fixture_clone = &fixture;
            let city = city.to_string();
            let tool = tool.to_string();
            let workflow_id = workflow_id;
            
            let future = async move {
                let mut tool_context = HookContext::new(
                    HookPoint::BeforeToolExecution,
                    ComponentId::new(ComponentType::Tool, tool.clone()),
                );
                tool_context.correlation_id = workflow_id;
                tool_context.insert_data(
                    "tool_input".to_string(),
                    json!({
                        "tool": tool,
                        "city": city,
                        "parallel": true
                    }),
                );
                tool_context.insert_metadata("city".to_string(), city.clone());
                tool_context.insert_metadata("parallel_execution".to_string(), "true".to_string());
                
                fixture_clone.execute_and_persist(&mut tool_context).await
            };
            
            tool_futures.push(future);
        }
    }
    
    // Execute all tools in parallel
    let results = futures::future::join_all(tool_futures).await;
    for result in results {
        result.unwrap();
    }
    
    // Verify all parallel executions are correlated
    let stored = fixture
        .storage
        .load_by_correlation_id(&workflow_id.to_string())
        .await
        .unwrap();
    
    // Count tool executions by city
    let mut city_executions: HashMap<String, usize> = HashMap::new();
    
    for (_, data) in &stored {
        if let Ok(context) = serde_json::from_slice::<HookContext>(data) {
            if let Some(city) = context.get_metadata("city") {
                *city_executions.entry(city.to_string()).or_insert(0) += 1;
            }
        }
    }
    
    // Verify each city has executions
    for city in &cities {
        assert!(
            city_executions.get(*city).copied().unwrap_or(0) > 0,
            "No executions found for city {}",
            city
        );
    }
}

#[tokio::test]
async fn test_correlation_with_nested_workflows() {
    let fixture = HookTestFixture::new().await;
    
    // Parent workflow
    let parent_workflow_id = Uuid::new_v4();
    
    // Parent agent starts
    let mut parent_context = fixture.create_context(
        HookPoint::BeforeWorkflowExecution,
        "parent-workflow",
    );
    parent_context.correlation_id = parent_workflow_id;
    parent_context.insert_data(
        "workflow".to_string(),
        workflow_data("data-processing", vec!["fetch", "transform", "analyze"]),
    );
    parent_context.insert_metadata("workflow_level".to_string(), "parent".to_string());
    
    fixture.execute_and_persist(&mut parent_context).await.unwrap();
    
    // Child workflow 1: Fetch data
    let child1_id = Uuid::new_v4();
    let mut child1_context = fixture.create_context(
        HookPoint::BeforeWorkflowExecution,
        "fetch-workflow",
    );
    child1_context.correlation_id = child1_id;
    child1_context.insert_data(
        "workflow".to_string(),
        workflow_data("fetch-data", vec!["connect", "download", "validate"]),
    );
    child1_context.insert_metadata("workflow_level".to_string(), "child".to_string());
    child1_context.insert_metadata("parent_correlation_id".to_string(), parent_workflow_id.to_string());
    
    fixture.execute_and_persist(&mut child1_context).await.unwrap();
    
    // Child workflow 2: Transform data
    let child2_id = Uuid::new_v4();
    let mut child2_context = fixture.create_context(
        HookPoint::BeforeWorkflowExecution,
        "transform-workflow",
    );
    child2_context.correlation_id = child2_id;
    child2_context.insert_data(
        "workflow".to_string(),
        workflow_data("transform-data", vec!["parse", "clean", "format"]),
    );
    child2_context.insert_metadata("workflow_level".to_string(), "child".to_string());
    child2_context.insert_metadata("parent_correlation_id".to_string(), parent_workflow_id.to_string());
    
    fixture.execute_and_persist(&mut child2_context).await.unwrap();
    
    // Verify parent-child relationships
    let parent_stored = fixture
        .storage
        .load_by_correlation_id(&parent_workflow_id.to_string())
        .await
        .unwrap();
    
    assert!(!parent_stored.is_empty(), "Parent workflow events not found");
    
    // Check child workflows reference parent
    for child_id in &[child1_id, child2_id] {
        let child_stored = fixture
            .storage
            .load_by_correlation_id(&child_id.to_string())
            .await
            .unwrap();
        
        let mut found_parent_ref = false;
        for (_, data) in &child_stored {
            if let Ok(context) = serde_json::from_slice::<HookContext>(data) {
                if let Some(parent_id) = context.get_metadata("parent_correlation_id") {
                    assert_eq!(
                        parent_id,
                        parent_workflow_id.to_string(),
                        "Child workflow has incorrect parent reference"
                    );
                    found_parent_ref = true;
                }
            }
        }
        assert!(found_parent_ref, "Child workflow missing parent reference");
    }
}

#[tokio::test]
async fn test_correlation_with_error_propagation() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Start workflow
    let mut start_context = fixture.create_context(
        HookPoint::BeforeWorkflowExecution,
        "error-workflow",
    );
    start_context.correlation_id = workflow_id;
    start_context.insert_data(
        "workflow".to_string(),
        workflow_data("risky-operation", vec!["validate", "process", "save"]),
    );
    
    fixture.execute_and_persist(&mut start_context).await.unwrap();
    
    // Step succeeds
    let mut step1_context = HookContext::new(
        HookPoint::AfterStepExecution,
        ComponentId::new(ComponentType::Workflow, "validate".to_string()),
    );
    step1_context.correlation_id = workflow_id;
    step1_context.insert_data("step_result".to_string(), json!({"status": "success"}));
    
    fixture.execute_and_persist(&mut step1_context).await.unwrap();
    
    // Step fails
    let mut step2_context = HookContext::new(
        HookPoint::WorkflowError,
        ComponentId::new(ComponentType::Workflow, "process".to_string()),
    );
    step2_context.correlation_id = workflow_id;
    step2_context.insert_metadata("error".to_string(), "Data validation failed".to_string());
    step2_context.insert_data(
        "error_details".to_string(),
        json!({
            "code": "VALIDATION_ERROR",
            "message": "Invalid data format",
            "step": "process",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    );
    
    fixture.execute_and_persist(&mut step2_context).await.unwrap();
    
    // Workflow handles error
    let mut error_context = HookContext::new(
        HookPoint::AfterWorkflowExecution,
        ComponentId::new(ComponentType::Workflow, "error-workflow".to_string()),
    );
    error_context.correlation_id = workflow_id;
    error_context.insert_data(
        "workflow_result".to_string(),
        json!({
            "status": "failed",
            "error": "Data validation failed",
            "completed_steps": ["validate"],
            "failed_step": "process"
        }),
    );
    
    fixture.execute_and_persist(&mut error_context).await.unwrap();
    
    // Verify error propagation through correlation
    let stored = fixture
        .storage
        .load_by_correlation_id(&workflow_id.to_string())
        .await
        .unwrap();
    
    // Find error events
    let mut error_events = 0;
    let mut has_workflow_error = false;
    
    for (_, data) in &stored {
        if let Ok(context) = serde_json::from_slice::<HookContext>(data) {
            if context.point == HookPoint::WorkflowError {
                error_events += 1;
            }
            if let Some(result) = context.data.get("workflow_result") {
                if let Some(status) = result.get("status") {
                    if status.as_str() == Some("failed") {
                        has_workflow_error = true;
                    }
                }
            }
        }
    }
    
    assert!(error_events > 0, "No error events found in correlation");
    assert!(has_workflow_error, "Workflow error not properly correlated");
}

#[tokio::test]
async fn test_correlation_replay_preserves_relationships() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Create related events
    let components = vec![
        ("agent", ComponentType::Agent),
        ("tool1", ComponentType::Tool),
        ("tool2", ComponentType::Tool),
    ];
    
    for (name, comp_type) in &components {
        let mut context = HookContext::new(
            HookPoint::BeforeExecution,
            ComponentId::new(comp_type.clone(), name.to_string()),
        );
        context.correlation_id = workflow_id;
        context.insert_metadata("sequence".to_string(), name.to_string());
        
        fixture.execute_and_persist(&mut context).await.unwrap();
    }
    
    // Replay all correlated events
    let replayed = fixture
        .replay_hooks(&workflow_id.to_string(), ReplayOptions::default())
        .await
        .unwrap();
    
    // Verify all components are present in replay
    let mut replayed_components = HashSet::new();
    for context in &replayed {
        if let Some(seq) = context.get_metadata("sequence") {
            replayed_components.insert(seq.to_string());
        }
    }
    
    for (name, _) in &components {
        assert!(
            replayed_components.contains(*name),
            "Component {} missing from replay",
            name
        );
    }
    
    // All replayed contexts should have same correlation ID
    for context in &replayed {
        assert_eq!(
            context.correlation_id,
            workflow_id,
            "Correlation ID not preserved in replay"
        );
    }
}