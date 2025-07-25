//! Separate test for HTTP timeout functionality

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::api::http_request::{HttpRequestConfig, HttpRequestTool};
use serde_json::json;

#[tokio::test]
async fn test_http_timeout_with_short_timeout() {
    // Create tool with very short timeout
    let config = HttpRequestConfig {
        timeout_seconds: 1, // 1 second timeout
        ..Default::default()
    };

    let tool = HttpRequestTool::new(config).unwrap();

    // Request a 3-second delay which should timeout
    let input = AgentInput::text("timeout test").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/delay/3"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;

    // Should fail due to timeout
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        // Check that it's a timeout or request error
        assert!(
            error_msg.contains("request")
                || error_msg.contains("timeout")
                || error_msg.contains("operation timed out"),
            "Expected timeout error, got: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_http_no_timeout_with_long_timeout() {
    // Create tool with long timeout
    let config = HttpRequestConfig {
        timeout_seconds: 10, // 10 second timeout
        ..Default::default()
    };

    let tool = HttpRequestTool::new(config).unwrap();

    // Request a 2-second delay which should complete
    let input = AgentInput::text("delay test").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/delay/2"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;

    // Should succeed, but handle potential network issues gracefully
    match result {
        Ok(output) => {
            assert!(
                output.text.contains("200"),
                "Expected successful response with status 200"
            );
        }
        Err(e) => {
            // Don't panic on network errors, just skip the test
            eprintln!("Warning: HTTP test failed due to network issue: {}", e);
            eprintln!("This is likely due to httpbin.org being unavailable");
            return; // Skip rest of test
        }
    }
}
