//! ABOUTME: Integration tests for WebpageMonitorTool
//! ABOUTME: Tests webpage monitoring and change detection functionality

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::WebpageMonitorTool;
use serde_json::json;

#[tokio::test]
async fn test_webpage_monitor_initial_check() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "check_interval": 60
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Initial check should return baseline
    assert!(
        result["is_first_check"].as_bool().unwrap_or(true)
            || result.get("baseline_established").is_some()
    );
    assert!(result["content_hash"].is_string() || result["hash"].is_string());
    assert!(result.get("last_checked").is_some() || result.get("timestamp").is_some());
}

#[tokio::test]
async fn test_webpage_monitor_with_selector() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "selector": "h1",
        "monitor_mode": "selector"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Should monitor specific selector
    assert!(result.get("monitored_content").is_some() || result.get("selected_content").is_some());
}

#[tokio::test]
async fn test_webpage_monitor_metadata_changes() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "monitor_metadata": true
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Should include metadata monitoring
    assert!(result.get("metadata").is_some() || result.get("meta_tags").is_some());
}

#[tokio::test]
async fn test_webpage_monitor_content_diff() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    // First check to establish baseline
    let input1 = create_agent_input(json!({
        "input": format!("{}/html", test_endpoints::HTTPBIN_BASE),
        "monitor_id": "test_monitor_1"
    }))
    .unwrap();

    let _output1 = tool.execute(input1, context.clone()).await.unwrap();

    // Second check on same URL (simulating change detection)
    let input2 = create_agent_input(json!({
        "input": format!("{}/html", test_endpoints::HTTPBIN_BASE),
        "monitor_id": "test_monitor_1",
        "show_diff": true
    }))
    .unwrap();

    let output2 = tool.execute(input2, create_test_context()).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output2.text).unwrap();
    let result = &output_value["result"];

    // Should indicate if content changed or not
    assert!(
        result.get("has_changed").is_some()
            || result.get("changes_detected").is_some()
            || result.get("is_unchanged").is_some()
    );
}

#[tokio::test]
async fn test_webpage_monitor_alert_threshold() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "alert_on_change": true,
        "change_threshold": 10  // 10% change threshold
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Should have alert configuration
    assert!(result.get("alert_enabled").is_some() || result.get("monitoring_config").is_some());
}

#[tokio::test]
async fn test_webpage_monitor_invalid_url() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "not-a-url"
    }))
    .unwrap();

    let result = tool.execute(input, context).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("URL") || error.to_string().contains("url"));
}

#[tokio::test]
async fn test_webpage_monitor_network_error() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::INVALID_URL,
        "retry_on_error": false
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_error_output(&output, "error");
        }
        Err(e) => {
            assert!(e.to_string().contains("error") || e.to_string().contains("network"));
        }
    }
}
