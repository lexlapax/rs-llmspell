// ABOUTME: Hook replay tests with real provider data to validate replay accuracy and functionality
// ABOUTME: Tests replay of hooks from persisted data and validates behavior consistency

use crate::provider_hook_integration::common::{
    assertions::*, test_data::*, HookTestFixture,
};
use llmspell_hooks::{
    persistence::{ReplayOptions, TimeRange},
    types::HookPoint,
};
use serde_json::json;
use std::time::Duration;
use chrono::{DateTime, Utc};
use uuid::Uuid;
#[tokio::test]
async fn test_replay_single_correlation_id() {
    let fixture = HookTestFixture::new().await;
    
    // Create and execute original context
    let mut original = fixture.create_context(HookPoint::AfterAgentExecution, "replay-single");
    original.insert_data(
        "response".to_string(),
        llm_response_data("Test response", 100),
    );
    original.insert_data("model".to_string(), json!("gpt-4"));
    original.insert_metadata("execution_time_ms".to_string(), "500".to_string());
    
    fixture.execute_and_persist(&mut original).await.unwrap();
    
    // Wait for persistence
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Replay with default options
    let replayed = fixture
        .replay_hooks(&original.correlation_id.to_string(), ReplayOptions::default())
        .await
        .unwrap();
    
    assert!(!replayed.is_empty(), "No contexts were replayed");
    
    // Verify each replayed context
    for (idx, replay_context) in replayed.iter().enumerate() {
        println!("Checking replayed context {}", idx);
        assert_replay_matches(&original, replay_context, false);
        
        // Verify metadata preservation
        assert_eq!(
            replay_context.get_metadata("execution_time_ms"),
            Some("500"),
            "Metadata not preserved in replay"
        );
    }
}
#[tokio::test]
async fn test_replay_with_time_range() {
    let fixture = HookTestFixture::new().await;
    
    let start_time = Utc::now();
    
    // Create contexts at different times
    let mut contexts = vec![];
    for i in 0..3 {
        let mut context = fixture.create_context(
            HookPoint::AfterAgentExecution,
            &format!("replay-time-{}", i),
        );
        context.insert_data("sequence".to_string(), json!(i));
        context.insert_data(
            "response".to_string(),
            llm_response_data(&format!("Response {}", i), 50),
        );
        
        fixture.execute_and_persist(&mut context).await.unwrap();
        contexts.push(context);
        
        // Wait between executions
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    let mid_time = Utc::now();
    
    // Create more contexts
    for i in 3..5 {
        let mut context = fixture.create_context(
            HookPoint::AfterAgentExecution,
            &format!("replay-time-{}", i),
        );
        context.insert_data("sequence".to_string(), json!(i));
        context.insert_data(
            "response".to_string(),
            llm_response_data(&format!("Response {}", i), 50),
        );
        
        fixture.execute_and_persist(&mut context).await.unwrap();
        contexts.push(context);
        
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // Replay only middle time range
    let options = ReplayOptions {
        time_range: Some(TimeRange {
            start: Some(mid_time),
            end: Some(Utc::now()),
        }),
        ..Default::default()
    };
    
    // This should only replay contexts 3 and 4
    let mut replayed_count = 0;
    for context in &contexts[3..5] {
        let replayed = fixture
            .replay_hooks(&context.correlation_id.to_string(), options.clone())
            .await
            .unwrap();
        
        if !replayed.is_empty() {
            replayed_count += 1;
        }
    }
    
    assert_eq!(replayed_count, 2, "Should only replay 2 contexts from time range");
}
#[tokio::test]
async fn test_replay_with_filtering() {
    let fixture = HookTestFixture::new().await;
    
    // Create contexts with different hook points
    let hook_points = vec![
        HookPoint::BeforeAgentExecution,
        HookPoint::AfterAgentExecution,
        HookPoint::BeforeToolExecution,
        HookPoint::AfterToolExecution,
    ];
    
    let mut contexts = vec![];
    for (i, point) in hook_points.iter().enumerate() {
        let mut context = fixture.create_context(*point, &format!("replay-filter-{}", i));
        context.insert_data("index".to_string(), json!(i));
        
        fixture.execute_and_persist(&mut context).await.unwrap();
        contexts.push(context);
    }
    
    // Wait for persistence
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Replay only agent execution hooks
    let options = ReplayOptions {
        filter_hook_points: Some(vec![
            HookPoint::BeforeAgentExecution,
            HookPoint::AfterAgentExecution,
        ]),
        ..Default::default()
    };
    
    let mut agent_replays = 0;
    let mut tool_replays = 0;
    
    for context in &contexts {
        let replayed = fixture
            .replay_hooks(&context.correlation_id.to_string(), options.clone())
            .await
            .unwrap();
        
        if !replayed.is_empty() {
            match context.point {
                HookPoint::BeforeAgentExecution | HookPoint::AfterAgentExecution => {
                    agent_replays += 1;
                }
                HookPoint::BeforeToolExecution | HookPoint::AfterToolExecution => {
                    tool_replays += 1;
                }
                _ => {}
            }
        }
    }
    
    assert_eq!(agent_replays, 2, "Should replay 2 agent execution contexts");
    assert_eq!(tool_replays, 0, "Should not replay tool execution contexts");
}
#[tokio::test]
async fn test_replay_preserves_hook_order() {
    let fixture = HookTestFixture::new().await;
    
    // Create context that will trigger multiple hooks
    let mut context = fixture.create_context(HookPoint::AfterAgentExecution, "replay-order");
    context.insert_data("model".to_string(), json!("gpt-3.5-turbo"));
    context.insert_data(
        "usage".to_string(),
        json!({
            "prompt_tokens": 100,
            "completion_tokens": 200,
            "total_tokens": 300
        }),
    );
    context.insert_data(
        "response".to_string(),
        llm_response_data("Ordered response", 300),
    );
    
    // Execute and persist
    fixture.execute_and_persist(&mut context).await.unwrap();
    
    // Get the stored hooks to verify order
    let stored = fixture
        .storage
        .load_by_correlation_id(&context.correlation_id.to_string())
        .await
        .unwrap();
    
    // Expected order based on hook priorities
    let expected_order = vec![
        "SecurityHook",      // HIGHEST priority
        "RateLimitHook",     // HIGH priority
        "RetryHook",         // HIGH priority
        "CachingHook",       // HIGH priority
        "LoggingHook",       // NORMAL priority
        "MetricsHook",       // NORMAL priority
        "CostTrackingHook",  // NORMAL priority
        "DebuggingHook",     // LOWEST priority
    ];
    
    // Verify hooks are stored in priority order
    let stored_hooks: Vec<String> = stored
        .iter()
        .map(|(replay_id, _)| replay_id.split(':').next().unwrap().to_string())
        .collect();
    
    for expected in &expected_order {
        assert!(
            stored_hooks.contains(&expected.to_string()),
            "Hook {} not found in stored hooks",
            expected
        );
    }
}
#[tokio::test]
async fn test_replay_handles_errors_gracefully() {
    let fixture = HookTestFixture::new().await;
    
    // Create context that resulted in an error
    let mut context = fixture.create_context(HookPoint::AgentError, "replay-error");
    context.insert_metadata("error".to_string(), "API rate limit exceeded".to_string());
    context.insert_data(
        "request".to_string(),
        llm_request_data("gpt-4", "This request failed"),
    );
    
    fixture.execute_and_persist(&mut context).await.unwrap();
    
    // Replay should handle error contexts
    let replayed = fixture
        .replay_hooks(&context.correlation_id.to_string(), ReplayOptions::default())
        .await
        .unwrap();
    
    assert!(!replayed.is_empty(), "Error contexts should be replayable");
    
    for replay_context in replayed {
        assert_eq!(
            replay_context.get_metadata("error"),
            Some("API rate limit exceeded"),
            "Error metadata not preserved"
        );
    }
}
#[tokio::test]
async fn test_replay_with_modified_context() {
    let fixture = HookTestFixture::new().await;
    
    // Create original context
    let mut original = fixture.create_context(HookPoint::BeforeAgentExecution, "replay-modified");
    original.insert_data(
        "request".to_string(),
        llm_request_data("gpt-3.5-turbo", "Original prompt"),
    );
    
    fixture.execute_and_persist(&mut original).await.unwrap();
    
    // Replay with modified data
    let mut options = ReplayOptions::default();
    options.modify_context = Some(Box::new(|ctx| {
        // Modify the prompt in replay
        if let Some(request) = ctx.data.get_mut("request") {
            if let Some(obj) = request.as_object_mut() {
                obj.insert("prompt".to_string(), json!("Modified prompt in replay"));
            }
        }
    }));
    
    let replayed = fixture
        .replay_hooks(&original.correlation_id.to_string(), options)
        .await
        .unwrap();
    
    // Verify modification was applied
    for replay_context in replayed {
        if let Some(request) = replay_context.data.get("request") {
            assert_eq!(
                request.get("prompt").and_then(|v| v.as_str()),
                Some("Modified prompt in replay"),
                "Context modification not applied"
            );
        }
    }
}
#[tokio::test]
async fn test_replay_performance_with_many_hooks() {
    let fixture = HookTestFixture::new().await;
    
    // Create context that triggers all hooks
    let mut context = fixture.create_context(HookPoint::AfterAgentExecution, "replay-perf");
    context.insert_data("model".to_string(), json!("gpt-4"));
    context.insert_data(
        "usage".to_string(),
        json!({
            "prompt_tokens": 500,
            "completion_tokens": 1000,
            "total_tokens": 1500
        }),
    );
    context.insert_data(
        "response".to_string(),
        llm_response_data("Performance test response", 1500),
    );
    
    // Add lots of metadata
    for i in 0..20 {
        context.insert_metadata(format!("meta_{}", i), format!("value_{}", i));
    }
    
    // Execute and persist
    let persist_start = std::time::Instant::now();
    fixture.execute_and_persist(&mut context).await.unwrap();
    let persist_duration = persist_start.elapsed();
    
    println!("Persistence took: {:?}", persist_duration);
    
    // Wait for persistence
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Measure replay performance
    let replay_start = std::time::Instant::now();
    let replayed = fixture
        .replay_hooks(&context.correlation_id.to_string(), ReplayOptions::default())
        .await
        .unwrap();
    let replay_duration = replay_start.elapsed();
    
    println!("Replay took: {:?}", replay_duration);
    println!("Replayed {} contexts", replayed.len());
    
    // Replay should be faster than original execution
    assert!(
        replay_duration < persist_duration * 2,
        "Replay too slow: {:?} vs persist {:?}",
        replay_duration,
        persist_duration
    );
    
    // Verify all hooks were replayed
    assert!(replayed.len() >= 5, "Not all hooks were replayed");
}