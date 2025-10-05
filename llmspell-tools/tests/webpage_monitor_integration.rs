//! ABOUTME: Integration tests for `WebpageMonitorTool`
//! ABOUTME: Tests webpage monitoring and change detection functionality

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::WebpageMonitorTool;
use serde_json::json;

/// Helper to check if an error is network-related (should skip test, not fail)
fn is_network_error(e: &llmspell_core::error::LLMSpellError) -> bool {
    let err_str = e.to_string();
    err_str.contains("Failed to fetch URL")
        || err_str.contains("HTTP error")
        || err_str.contains("network")
        || err_str.contains("timeout")
        || err_str.contains("connection")
}
#[tokio::test]
async fn test_webpage_monitor_initial_check() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "parameters": {
            "input": test_endpoints::EXAMPLE_WEBSITE,
            "check_interval": 60
        }
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_success_output(&output, &["operation", "result"]);

            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            let result = &output_value["result"];

            // Initial check should return current content (no previous content)
            assert!(result["current_content"].is_string());
            assert_eq!(result["has_changes"], false);
            assert!(result["message"]
                .as_str()
                .unwrap()
                .contains("No previous content"));
        }
        Err(e) => {
            // If it's a network error, skip the test
            if is_network_error(&e) {
                eprintln!("Skipping test due to network error: {e}");
                return;
            }
            // Otherwise, propagate the error
            panic!("Unexpected error: {e}");
        }
    }
}
#[tokio::test]
async fn test_webpage_monitor_with_selector() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "parameters": {
            "input": test_endpoints::EXAMPLE_WEBSITE,
            "selector": "h1",
            "monitor_mode": "selector"
        }
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            let result = &output_value["result"];

            // Should have current content from the selector
            assert!(result["current_content"].is_string());
            // Should indicate the selector was used (if supported)
            assert!(
                result.get("selector_used").is_some()
                    || !result["current_content"].as_str().unwrap().is_empty()
            );
        }
        Err(e) => {
            // If it's a network error, skip the test
            if is_network_error(&e) {
                eprintln!("Skipping test due to network error: {e}");
                return;
            }
            // Otherwise, propagate the error
            panic!("Unexpected error: {e}");
        }
    }
}
#[tokio::test]
async fn test_webpage_monitor_metadata_changes() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "parameters": {
            "input": test_endpoints::EXAMPLE_WEBSITE,
            "monitor_metadata": true
        }
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            let result = &output_value["result"];

            // Should return current content (metadata monitoring not implemented)
            assert!(result["current_content"].is_string());
            assert_eq!(result["has_changes"], false);
        }
        Err(e) => {
            // If it's a network error, skip the test
            if is_network_error(&e) {
                eprintln!("Skipping test due to network error: {e}");
                return;
            }
            // Otherwise, propagate the error
            panic!("Unexpected error: {e}");
        }
    }
}
#[tokio::test]
async fn test_webpage_monitor_content_diff() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    // First check to get baseline content
    let input1 = create_agent_input(json!({
        "parameters": {
            "input": format!("{}/html", test_endpoints::HTTPBIN_BASE)
        }
    }))
    .unwrap();

    match tool.execute(input1, context.clone()).await {
        Ok(output1) => {
            let output1_value: serde_json::Value = serde_json::from_str(&output1.text).unwrap();
            let baseline_content = output1_value["result"]["current_content"].as_str().unwrap();

            // Second check with previous content provided
            let input2 = create_agent_input(json!({
                "parameters": {
                    "input": format!("{}/html", test_endpoints::HTTPBIN_BASE),
                    "previous_content": baseline_content
                }
            }))
            .unwrap();

            match tool.execute(input2, create_test_context()).await {
                Ok(output2) => {
                    let output2_value: serde_json::Value =
                        serde_json::from_str(&output2.text).unwrap();
                    let result = &output2_value["result"];

                    // Should indicate if content changed or not
                    assert!(result.get("has_changes").is_some());
                    assert!(result.get("changes").is_some());
                    assert!(result.get("change_count").is_some());
                }
                Err(e) => {
                    if is_network_error(&e) {
                        eprintln!("Skipping second fetch due to network error: {e}");
                        return;
                    }
                    panic!("Unexpected error on second fetch: {e}");
                }
            }
        }
        Err(e) => {
            // If it's a network error, skip the test
            if is_network_error(&e) {
                eprintln!("Skipping test due to network error: {e}");
                return;
            }
            // Otherwise, propagate the error
            panic!("Unexpected error: {e}");
        }
    }
}
#[tokio::test]
async fn test_webpage_monitor_alert_threshold() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "parameters": {
            "input": test_endpoints::EXAMPLE_WEBSITE,
            "alert_on_change": true,
            "change_threshold": 10  // 10% change threshold
        }
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            let result = &output_value["result"];

            // Should return current content (alert configuration not implemented)
            assert!(result["current_content"].is_string());
            assert_eq!(result["has_changes"], false);
        }
        Err(e) => {
            // If it's a network error, skip the test
            if is_network_error(&e) {
                eprintln!("Skipping test due to network error: {e}");
                return;
            }
            // Otherwise, propagate the error
            panic!("Unexpected error: {e}");
        }
    }
}
#[tokio::test]
async fn test_webpage_monitor_invalid_url() {
    let tool = WebpageMonitorTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "parameters": {
            "input": "not-a-url"
        }
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
        "parameters": {
            "input": test_endpoints::INVALID_URL,
            "retry_on_error": false
        }
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
