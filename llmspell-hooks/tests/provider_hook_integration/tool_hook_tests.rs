// ABOUTME: Tool-triggered hook tests for validating hook execution during tool operations
// ABOUTME: Tests hooks that fire when tools are invoked, complete, or encounter errors

use crate::provider_hook_integration::common::{
    assertions::*, test_data::*, HookTestFixture,
};
use llmspell_hooks::{
    builtin::{
        metrics::{MetricsConfig, MetricsHook},
        security::{SecurityConfig, SecurityHook},
    },
    context::HookContext,
    types::{ComponentId, ComponentType, HookPoint},
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_execution_hooks() {
    let fixture = HookTestFixture::new().await;
    
    // Configure metrics for tool tracking
    let metrics_hook = Arc::new(
        MetricsHook::new().with_custom_metrics(true),
    );
    fixture.executor.register_hook(metrics_hook.clone());
    
    // Execute calculator tool
    let mut calc_context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "calculator".to_string()),
    );
    calc_context.insert_data(
        "operation".to_string(),
        json!({
            "function": "add",
            "args": [5, 3]
        }),
    );
    calc_context.insert_metadata("tool_category".to_string(), "math".to_string());
    
    fixture.execute_and_persist(&mut calc_context).await.unwrap();
    
    // Simulate tool execution
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Tool completes
    calc_context.point = HookPoint::AfterToolExecution;
    calc_context.insert_data(
        "result".to_string(),
        json!({
            "value": 8,
            "execution_time_ms": 45
        }),
    );
    
    fixture.execute_and_persist(&mut calc_context).await.unwrap();
    
    // Verify metrics were tracked
    let metrics = metrics_hook.get_metrics();
    assert!(
        metrics.operations_by_type.contains_key(&"Tool".to_string()),
        "Tool metrics not tracked"
    );
    
    // Verify persistence
    let correlation_id = calc_context.correlation_id.to_string();
    assert_hook_persisted(&fixture.storage, &correlation_id, "MetricsHook").await;
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_security_validation() {
    let fixture = HookTestFixture::new().await;
    
    // Configure strict security for tools
    let mut sensitive_params = std::collections::HashSet::new();
    sensitive_params.insert("api_key".to_string());
    sensitive_params.insert("database_password".to_string());
    sensitive_params.insert("private_key".to_string());
    
    let security_config = SecurityConfig {
        enable_parameter_validation: true,
        sensitive_parameters: sensitive_params,
        max_parameter_length: 1000,
        block_on_violations: true,
        ..Default::default()
    };
    
    let security_hook = Arc::new(
        llmspell_hooks::builtin::SecurityHook::with_config(security_config),
    );
    fixture.executor.register_hook(security_hook.clone());
    
    // Test 1: Safe tool execution
    let mut safe_context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "file-reader".to_string()),
    );
    safe_context.insert_data(
        "parameters".to_string(),
        json!({
            "file_path": "/tmp/safe_file.txt",
            "encoding": "utf-8"
        }),
    );
    
    let result = fixture.executor.execute(&mut safe_context).await.unwrap();
    assert_hook_success(&result);
    
    // Test 2: Tool with sensitive parameters
    let mut sensitive_context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "database-connector".to_string()),
    );
    sensitive_context.insert_data(
        "database_password".to_string(),
        json!("super_secret_password"),
    );
    sensitive_context.insert_data(
        "query".to_string(),
        json!("SELECT * FROM users"),
    );
    
    fixture.execute_and_persist(&mut sensitive_context).await.unwrap();
    
    // Verify security events
    let events = security_hook.get_events();
    assert!(
        events.iter().any(|e| e.description.contains("sensitive parameter")),
        "Security hook did not detect sensitive parameter in tool"
    );
    
    // Test 3: Tool with oversized parameter
    let mut large_context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "data-processor".to_string()),
    );
    large_context.insert_data(
        "large_data".to_string(),
        json!("x".repeat(2000)), // Exceeds max_parameter_length
    );
    
    let result = fixture.executor.execute(&mut large_context).await.unwrap();
    assert!(
        matches!(result, llmspell_hooks::result::HookResult::Cancel(_)),
        "Large parameter should be blocked"
    );
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_retry_on_failure() {
    let fixture = HookTestFixture::new().await;
    
    // Configure retry for tool failures
    use llmspell_hooks::builtin::retry::RetryHook;
    
    let retry_hook = Arc::new(RetryHook::new());
    fixture.executor.register_hook(retry_hook.clone());
    
    // Simulate tool failure
    let mut context = HookContext::new(
        HookPoint::ToolError,
        ComponentId::new(ComponentType::Tool, "web-scraper".to_string()),
    );
    context.insert_metadata("error".to_string(), "connection_error: timeout after 30s".to_string());
    context.insert_data(
        "request".to_string(),
        json!({
            "url": "https://example.com/data",
            "timeout": 30000
        }),
    );
    
    let result = fixture.executor.execute(&mut context).await.unwrap();
    
    // Verify retry was triggered
    if let llmspell_hooks::result::HookResult::Retry { delay, max_attempts } = result {
        assert!(delay > Duration::ZERO);
        assert_eq!(max_attempts, 2); // 3 total attempts - 1 already used
        assert_eq!(context.get_metadata("retry_attempt"), Some("1"));
    } else {
        panic!("Expected Retry result for tool error");
    }
    
    // Verify metrics
    let metrics = retry_hook.metrics();
    assert_eq!(metrics.retry_attempts, 1);
    assert!(metrics.retry_reasons.contains_key("connection_error: timeout after 30s"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_caching_for_deterministic_operations() {
    let fixture = HookTestFixture::new().await;
    
    // Configure caching for deterministic tools
    use llmspell_hooks::builtin::caching::CachingHook;
    
    let cache_hook = Arc::new(CachingHook::new());
    fixture.executor.register_hook(cache_hook.clone());
    
    // First execution of deterministic tool
    let mut context1 = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "text-formatter".to_string()),
    );
    context1.insert_data(
        "input".to_string(),
        json!({
            "text": "hello world",
            "operation": "uppercase"
        }),
    );
    
    fixture.execute_and_persist(&mut context1).await.unwrap();
    assert_eq!(context1.get_metadata("cache_hit"), Some("false"));
    
    // Simulate tool result and cache it
    let result = llmspell_hooks::result::HookResult::Modified(json!({
        "output": "HELLO WORLD"
    }));
    
    cache_hook
        .cache()
        .put(
            llmspell_hooks::cache::CacheKey::from_context(&context1),
            result.clone(),
            None,
        )
        .unwrap();
    
    // Second execution with same input
    let mut context2 = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "text-formatter".to_string()),
    );
    context2.insert_data(
        "input".to_string(),
        json!({
            "text": "hello world",
            "operation": "uppercase"
        }),
    );
    
    let cached_result = fixture.executor.execute(&mut context2).await.unwrap();
    assert_eq!(context2.get_metadata("cache_hit"), Some("true"));
    
    // Verify cache statistics
    let stats = cache_hook.cache_stats();
    assert_eq!(stats.hits, 1);
    assert!(stats.hit_rate > 0.0);
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_cost_tracking() {
    let fixture = HookTestFixture::new().await;
    
    // Configure cost tracking for API tools
    let cost_config = llmspell_hooks::builtin::CostTrackingConfig {
        tool_costs: vec![
            ("weather-api".to_string(), 0.001),      // $0.001 per call
            ("geocoding-api".to_string(), 0.0005),   // $0.0005 per call
            ("translation-api".to_string(), 0.002),  // $0.002 per call
        ]
        .into_iter()
        .collect(),
        cost_threshold: Some(0.10), // Alert at $0.10
        track_by_component: true,
        ..Default::default()
    };
    
    let cost_hook = Arc::new(
        llmspell_hooks::builtin::CostTrackingHook::with_config(cost_config),
    );
    fixture.executor.register_hook(cost_hook.clone());
    
    // Execute multiple tool calls
    let tools = vec![
        ("weather-api", 5),      // 5 calls = $0.005
        ("geocoding-api", 10),   // 10 calls = $0.005
        ("translation-api", 20), // 20 calls = $0.04
    ];
    
    for (tool_name, call_count) in tools {
        for i in 0..call_count {
            let mut context = HookContext::new(
                HookPoint::AfterToolExecution,
                ComponentId::new(ComponentType::Tool, tool_name.to_string()),
            );
            context.insert_data("call_index".to_string(), json!(i));
            context.insert_metadata("billable".to_string(), "true".to_string());
            
            fixture.execute_and_persist(&mut context).await.unwrap();
        }
    }
    
    // Verify cost tracking
    let metrics = cost_hook.metrics();
    let expected_total = 0.005 + 0.005 + 0.04; // $0.05
    
    assert!(
        (metrics.total_cost - expected_total).abs() < 0.0001,
        "Cost tracking mismatch: expected {}, got {}",
        expected_total,
        metrics.total_cost
    );
    
    // Verify per-tool tracking
    assert_eq!(metrics.operations_by_component.get("weather-api"), Some(&5));
    assert_eq!(metrics.operations_by_component.get("geocoding-api"), Some(&10));
    assert_eq!(metrics.operations_by_component.get("translation-api"), Some(&20));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_debugging_traces() {
    let fixture = HookTestFixture::new().await;
    
    // Configure detailed debugging
    let debug_config = llmspell_hooks::builtin::DebuggingConfig {
        capture_stack_traces: true,
        include_context_data: true,
        log_to_console: false,
        min_duration_ms: 0,
        ..Default::default()
    };
    
    let debug_hook = Arc::new(
        llmspell_hooks::builtin::DebuggingHook::with_config(debug_config),
    );
    fixture.executor.register_hook(debug_hook.clone());
    
    // Execute complex tool operation
    let mut context = HookContext::new(
        HookPoint::BeforeToolExecution,
        ComponentId::new(ComponentType::Tool, "data-transformer".to_string()),
    );
    
    context.insert_data(
        "transformation".to_string(),
        json!({
            "input": {
                "users": [
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ]
            },
            "operations": [
                {"type": "filter", "field": "id", "operator": ">", "value": 0},
                {"type": "map", "transform": "uppercase_name"}
            ]
        }),
    );
    context.insert_metadata("debug_level".to_string(), "verbose".to_string());
    
    fixture.execute_and_persist(&mut context).await.unwrap();
    
    // Get debug traces
    let traces = debug_hook.get_traces();
    let tool_traces: Vec<_> = traces
        .iter()
        .filter(|t| t.component_type == "Tool")
        .collect();
    
    assert!(!tool_traces.is_empty(), "No tool traces captured");
    
    // Verify trace details
    let trace = tool_traces[0];
    assert_eq!(trace.component_name, "data-transformer");
    assert!(trace.context_data.get("transformation").is_some());
    assert!(trace.stack_trace.is_some());
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_tool_chain_hooks() {
    let fixture = HookTestFixture::new().await;
    
    let workflow_id = uuid::Uuid::new_v4();
    
    // Tool chain: fetch -> parse -> transform -> store
    let tool_chain = vec![
        ("url-fetcher", "fetch", json!({"url": "https://api.example.com/data"})),
        ("json-parser", "parse", json!({"format": "json", "strict": true})),
        ("data-transformer", "transform", json!({"operation": "normalize"})),
        ("data-store", "store", json!({"destination": "cache", "ttl": 3600})),
    ];
    
    let mut previous_output = json!(null);
    
    for (idx, (tool_name, operation, params)) in tool_chain.iter().enumerate() {
        let mut context = HookContext::new(
            HookPoint::BeforeToolExecution,
            ComponentId::new(ComponentType::Tool, tool_name.to_string()),
        );
        context.correlation_id = workflow_id;
        
        context.insert_data("operation".to_string(), json!(operation));
        context.insert_data("parameters".to_string(), params.clone());
        context.insert_data("chain_position".to_string(), json!(idx));
        
        if idx > 0 {
            context.insert_data("input_from_previous".to_string(), previous_output.clone());
        }
        
        fixture.execute_and_persist(&mut context).await.unwrap();
        
        // Simulate tool execution
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Tool produces output
        context.point = HookPoint::AfterToolExecution;
        previous_output = json!({
            "status": "success",
            "data": format!("output_from_{}", tool_name),
            "metadata": {
                "tool": tool_name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });
        context.insert_data("output".to_string(), previous_output.clone());
        
        fixture.execute_and_persist(&mut context).await.unwrap();
    }
    
    // Verify entire chain is correlated
    let stored = fixture
        .storage
        .load_by_correlation_id(&workflow_id.to_string())
        .await
        .unwrap();
    
    // Each tool should have before/after executions with multiple hooks
    let expected_min_events = tool_chain.len() * 2 * 5; // 2 states * ~5 hooks per state
    assert!(
        stored.len() >= expected_min_events,
        "Expected at least {} events, got {}",
        expected_min_events,
        stored.len()
    );
}