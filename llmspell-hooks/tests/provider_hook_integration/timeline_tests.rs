// ABOUTME: Timeline reconstruction tests for understanding causality chains in hook executions
// ABOUTME: Validates chronological ordering and event relationships across complex interactions

use crate::provider_hook_integration::common::{test_data::*, HookTestFixture};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use llmspell_hooks::{
    context::HookContext,
    persistence::ReplayOptions,
    types::{ComponentId, ComponentType, HookPoint},
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeline_basic_reconstruction() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    let start_time = Utc::now();
    
    // Create a sequence of events
    let events = vec![
        ("user-request", HookPoint::BeforeAgentExecution, 0),
        ("llm-processing", HookPoint::AfterAgentExecution, 100),
        ("tool-call", HookPoint::BeforeToolExecution, 200),
        ("tool-response", HookPoint::AfterToolExecution, 300),
        ("final-response", HookPoint::AfterAgentExecution, 400),
    ];
    
    for (name, point, delay_ms) in events {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        
        let mut context = fixture.create_context(point, name);
        context.correlation_id = workflow_id;
        context.insert_metadata("event_name".to_string(), name.to_string());
        context.insert_metadata("timestamp".to_string(), Utc::now().to_rfc3339());
        
        fixture.execute_and_persist(&mut context).await.unwrap();
    }
    
    // Get all executions for the workflow
    let stored = fixture.stored_executions.read().await;
    let executions = stored.get(&workflow_id.to_string())
        .expect("No executions found for workflow");
    
    // Verify we have correct number of events
    assert!(executions.len() >= 5, "Missing events in executions");
    
    // Verify chronological order
    let mut prev_time = None;
    for execution in executions {
        if let Some(prev) = prev_time {
            assert!(
                execution.timestamp >= prev,
                "Executions not in chronological order"
            );
        }
        prev_time = Some(execution.timestamp);
    }
    
    // Verify duration
    if let (Some(first), Some(last)) = (executions.first(), executions.last()) {
        let duration = last.timestamp.duration_since(first.timestamp).unwrap_or_default();
        assert!(
            duration >= Duration::from_millis(400),
            "Timeline duration shorter than expected"
        );
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeline_with_parallel_execution() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Agent starts workflow
    let mut agent_context = fixture.create_context(
        HookPoint::BeforeAgentExecution,
        "parallel-coordinator",
    );
    agent_context.correlation_id = workflow_id;
    agent_context.insert_metadata("phase".to_string(), "initialization".to_string());
    
    fixture.execute_and_persist(&mut agent_context).await.unwrap();
    
    // Parallel tool executions
    let tools = vec!["data-fetcher", "analyzer", "formatter"];
    let mut handles = vec![];
    
    for (idx, tool) in tools.iter().enumerate() {
        let fixture_clone = &fixture;
        let tool = tool.to_string();
        let workflow_id = workflow_id;
        let delay = (idx * 50) as u64; // Stagger starts slightly
        
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            
            let mut context = HookContext::new(
                HookPoint::BeforeToolExecution,
                ComponentId::new(ComponentType::Tool, tool.clone()),
            );
            context.correlation_id = workflow_id;
            context.insert_metadata("parallel_group".to_string(), "tools".to_string());
            context.insert_metadata("tool_name".to_string(), tool.clone());
            context.insert_metadata("start_time".to_string(), Utc::now().to_rfc3339());
            
            fixture_clone.execute_and_persist(&mut context).await.unwrap();
            
            // Simulate tool execution time
            tokio::time::sleep(Duration::from_millis(100 + delay)).await;
            
            context.point = HookPoint::AfterToolExecution;
            context.insert_metadata("end_time".to_string(), Utc::now().to_rfc3339());
            
            fixture_clone.execute_and_persist(&mut context).await.unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all parallel executions
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Agent completes workflow
    agent_context.point = HookPoint::AfterAgentExecution;
    agent_context.insert_metadata("phase".to_string(), "completion".to_string());
    fixture.execute_and_persist(&mut agent_context).await.unwrap();
    
    // Get all executions for the workflow
    let stored = fixture.stored_executions.read().await;
    let executions = stored.get(&workflow_id.to_string())
        .expect("No executions found for workflow");
    
    // Analyze parallel execution patterns
    let mut tool_events: HashMap<String, Vec<&TimelineEvent>> = HashMap::new();
    
    for event in &timeline.events {
        if let Some(tool_name) = event.metadata.get("tool_name") {
            tool_events
                .entry(tool_name.clone())
                .or_insert_with(Vec::new)
                .push(event);
        }
    }
    
    // Verify each tool has before/after events
    for tool in &tools {
        let events = tool_events.get(*tool).expect("Tool events not found");
        assert!(events.len() >= 2, "Tool {} missing events", tool);
        
        // Find overlapping execution windows
        let start = events
            .iter()
            .find(|e| e.hook_point == HookPoint::BeforeToolExecution)
            .expect("Start event not found");
        let end = events
            .iter()
            .find(|e| e.hook_point == HookPoint::AfterToolExecution)
            .expect("End event not found");
        
        assert!(end.timestamp > start.timestamp, "Invalid event ordering");
    }
    
    // Verify timeline shows parallel execution
    println!("Timeline visualization:");
    for event in &timeline.events {
        println!(
            "{}: {} - {}",
            event.timestamp.format("%H:%M:%S%.3f"),
            event.component_name,
            event.hook_point
        );
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeline_causality_chain() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Create a causality chain: A triggers B, B triggers C
    let mut context_a = fixture.create_context(HookPoint::AfterAgentExecution, "agent-a");
    context_a.correlation_id = workflow_id;
    context_a.insert_metadata("triggers".to_string(), "tool-b".to_string());
    context_a.insert_data(
        "decision".to_string(),
        json!({"action": "call_tool", "target": "tool-b"}),
    );
    
    fixture.execute_and_persist(&mut context_a).await.unwrap();
    
    // B is triggered by A
    let mut context_b = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "tool-b".to_string()),
    );
    context_b.correlation_id = workflow_id;
    context_b.insert_metadata("triggered_by".to_string(), "agent-a".to_string());
    context_b.insert_metadata("triggers".to_string(), "tool-c".to_string());
    
    fixture.execute_and_persist(&mut context_b).await.unwrap();
    
    // B completes and triggers C
    context_b.point = HookPoint::AfterToolExecution;
    context_b.insert_data(
        "result".to_string(),
        json!({"status": "success", "next_action": "call_tool_c"}),
    );
    
    fixture.execute_and_persist(&mut context_b).await.unwrap();
    
    // C is triggered by B
    let mut context_c = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "tool-c".to_string()),
    );
    context_c.correlation_id = workflow_id;
    context_c.insert_metadata("triggered_by".to_string(), "tool-b".to_string());
    context_c.insert_metadata("causality_depth".to_string(), "3".to_string());
    
    fixture.execute_and_persist(&mut context_c).await.unwrap();
    
    // Reconstruct timeline and analyze causality
    let timeline = Timeline::reconstruct_from_correlation_id(
        &fixture.storage,
        &workflow_id.to_string(),
    )
    .await
    .unwrap();
    
    // Build causality graph
    let mut triggers: HashMap<String, String> = HashMap::new();
    let mut triggered_by: HashMap<String, String> = HashMap::new();
    
    for event in &timeline.events {
        if let Some(target) = event.metadata.get("triggers") {
            triggers.insert(event.component_name.clone(), target.clone());
        }
        if let Some(source) = event.metadata.get("triggered_by") {
            triggered_by.insert(event.component_name.clone(), source.clone());
        }
    }
    
    // Verify causality chain
    assert_eq!(triggers.get("agent-a"), Some(&"tool-b".to_string()));
    assert_eq!(triggered_by.get("tool-b"), Some(&"agent-a".to_string()));
    assert_eq!(triggers.get("tool-b"), Some(&"tool-c".to_string()));
    assert_eq!(triggered_by.get("tool-c"), Some(&"tool-b".to_string()));
    
    // Verify temporal ordering matches causality
    let event_times: HashMap<String, DateTime<Utc>> = timeline
        .events
        .iter()
        .map(|e| (e.component_name.clone(), e.timestamp))
        .collect();
    
    if let (Some(time_a), Some(time_b), Some(time_c)) = (
        event_times.get("agent-a"),
        event_times.get("tool-b"),
        event_times.get("tool-c"),
    ) {
        assert!(time_b > time_a, "B should occur after A");
        assert!(time_c > time_b, "C should occur after B");
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeline_with_retries_and_errors() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Initial attempt fails
    let mut attempt1 = fixture.create_context(HookPoint::BeforeToolExecution, "flaky-tool");
    attempt1.correlation_id = workflow_id;
    attempt1.insert_metadata("attempt".to_string(), "1".to_string());
    attempt1.insert_data(
        "request".to_string(),
        tool_execution_data("external-api", "fetch_data"),
    );
    
    fixture.execute_and_persist(&mut attempt1).await.unwrap();
    
    // First attempt fails
    attempt1.point = HookPoint::ToolError;
    attempt1.insert_metadata("error".to_string(), "Connection timeout".to_string());
    attempt1.insert_data(
        "error_details".to_string(),
        json!({"code": "TIMEOUT", "duration_ms": 30000}),
    );
    
    fixture.execute_and_persist(&mut attempt1).await.unwrap();
    
    // Retry hook triggers
    let mut retry_context = fixture.create_context(HookPoint::BeforeRetry, "flaky-tool");
    retry_context.correlation_id = workflow_id;
    retry_context.insert_metadata("retry_attempt".to_string(), "1".to_string());
    retry_context.insert_metadata("retry_delay_ms".to_string(), "1000".to_string());
    
    fixture.execute_and_persist(&mut retry_context).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Second attempt succeeds
    let mut attempt2 = fixture.create_context(HookPoint::BeforeToolExecution, "flaky-tool");
    attempt2.correlation_id = workflow_id;
    attempt2.insert_metadata("attempt".to_string(), "2".to_string());
    attempt2.insert_metadata("retry_of".to_string(), "attempt_1".to_string());
    
    fixture.execute_and_persist(&mut attempt2).await.unwrap();
    
    attempt2.point = HookPoint::AfterToolExecution;
    attempt2.insert_data(
        "result".to_string(),
        json!({"status": "success", "data": "fetched successfully"}),
    );
    
    fixture.execute_and_persist(&mut attempt2).await.unwrap();
    
    // Get all executions for the workflow
    let stored = fixture.stored_executions.read().await;
    let executions = stored.get(&workflow_id.to_string())
        .expect("No executions found for workflow");
    
    // Analyze retry pattern
    let mut attempts = vec![];
    let mut errors = vec![];
    let mut retries = vec![];
    
    for event in &timeline.events {
        if let Some(attempt) = event.metadata.get("attempt") {
            attempts.push((attempt.clone(), event.hook_point));
        }
        if event.hook_point == HookPoint::ToolError {
            errors.push(event);
        }
        if event.hook_point == HookPoint::BeforeRetry {
            retries.push(event);
        }
    }
    
    // Verify retry sequence
    assert_eq!(errors.len(), 1, "Should have one error");
    assert_eq!(retries.len(), 1, "Should have one retry");
    assert!(
        attempts.iter().any(|(a, _)| a == "2"),
        "Should have second attempt"
    );
    
    // Verify timeline shows error -> retry -> success pattern
    let error_time = errors[0].timestamp;
    let retry_time = retries[0].timestamp;
    let success_time = timeline
        .events
        .iter()
        .find(|e| {
            e.metadata.get("attempt") == Some(&"2".to_string())
                && e.hook_point == HookPoint::AfterToolExecution
        })
        .map(|e| e.timestamp)
        .expect("Success event not found");
    
    assert!(retry_time > error_time, "Retry should occur after error");
    assert!(success_time > retry_time, "Success should occur after retry");
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeline_performance_analysis() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = Uuid::new_v4();
    
    // Create workflow with performance measurements
    let steps = vec![
        ("parse_input", 50),
        ("validate_data", 150),
        ("call_llm", 500),
        ("process_response", 100),
        ("format_output", 75),
    ];
    
    let mut cumulative_time = 0;
    
    for (step_name, duration_ms) in &steps {
        let mut context = HookContext::new(
            HookPoint::BeforeStepExecution,
            ComponentId::new(ComponentType::Workflow, step_name.to_string()),
        );
        context.correlation_id = workflow_id;
        context.insert_metadata("step_name".to_string(), step_name.to_string());
        context.insert_metadata("start_time".to_string(), Utc::now().to_rfc3339());
        
        fixture.execute_and_persist(&mut context).await.unwrap();
        
        // Simulate step execution
        tokio::time::sleep(Duration::from_millis(*duration_ms)).await;
        
        context.point = HookPoint::AfterStepExecution;
        context.insert_metadata("end_time".to_string(), Utc::now().to_rfc3339());
        context.insert_metadata("duration_ms".to_string(), duration_ms.to_string());
        
        fixture.execute_and_persist(&mut context).await.unwrap();
        
        cumulative_time += duration_ms;
    }
    
    // Get all executions for the workflow
    let stored = fixture.stored_executions.read().await;
    let executions = stored.get(&workflow_id.to_string())
        .expect("No executions found for workflow");
    
    // Analyze performance bottlenecks
    let mut step_durations: HashMap<String, u64> = HashMap::new();
    
    for event in &timeline.events {
        if event.hook_point == HookPoint::AfterStepExecution {
            if let Some(duration_str) = event.metadata.get("duration_ms") {
                if let Ok(duration) = duration_str.parse::<u64>() {
                    step_durations.insert(event.component_name.clone(), duration);
                }
            }
        }
    }
    
    // Find slowest step
    let slowest_step = step_durations
        .iter()
        .max_by_key(|(_, &duration)| duration)
        .expect("No steps found");
    
    assert_eq!(slowest_step.0, "call_llm", "Wrong slowest step identified");
    assert_eq!(*slowest_step.1, 500, "Wrong duration for slowest step");
    
    // Calculate timeline statistics
    let total_duration = timeline.total_duration();
    let expected_min_duration = ChronoDuration::milliseconds(cumulative_time as i64);
    
    assert!(
        total_duration >= expected_min_duration,
        "Timeline duration shorter than sum of steps"
    );
    
    // Generate performance report
    println!("\nPerformance Analysis:");
    println!("Total Duration: {:?}", total_duration);
    println!("\nStep Breakdown:");
    for (step, duration) in &step_durations {
        let percentage = (*duration as f64 / cumulative_time as f64) * 100.0;
        println!("  {}: {}ms ({:.1}%)", step, duration, percentage);
    }
}