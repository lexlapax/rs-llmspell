//! ABOUTME: Integration tests for WebhookCallerTool
//! ABOUTME: Tests webhook calling functionality with various HTTP methods and payloads

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::WebhookCallerTool;
use serde_json::json;
#[tokio::test]
async fn test_webhook_caller_post() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    let webhook_data = json!({
        "event": "test_event",
        "data": {
            "user_id": 123,
            "action": "test_action"
        }
    });

    let input = create_agent_input(json!({
        "input": test_endpoints::HTTPBIN_POST,
        "method": "POST",
        "payload": webhook_data
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // httpbin should echo our webhook data
    assert_eq!(result["status_code"], 200);
    let response_body = result["response"]["json"]["json"].as_object().unwrap();
    assert_eq!(response_body["event"], "test_event");
}
#[tokio::test]
async fn test_webhook_caller_with_headers() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::HTTPBIN_POST,
        "headers": {
            "X-Webhook-Secret": "test-secret",
            "X-Event-Type": "user.created"
        },
        "payload": {
            "user_id": 456
        }
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    assert_eq!(result["status_code"], 200);
    // httpbin should echo our headers in the request headers section
    let request_headers = result["response"]["json"]["headers"].as_object().unwrap();
    assert_eq!(request_headers["X-Webhook-Secret"], "test-secret");
    assert_eq!(request_headers["X-Event-Type"], "user.created");
}
#[tokio::test]
async fn test_webhook_caller_get_method() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::HTTPBIN_GET,
        "method": "GET"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    assert_eq!(result["status_code"], 200);
    assert!(result["response_time_ms"].as_f64().unwrap() > 0.0);
}
#[tokio::test]
async fn test_webhook_caller_retry_on_failure() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    // Test with a 500 status code endpoint
    let input = create_agent_input(json!({
        "input": format!("{}/500", test_endpoints::HTTPBIN_STATUS),
        "retry_count": 2,
        "retry_delay": 100
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Should still get 500 but with retry attempts logged
    assert_eq!(result["status_code"], 500);
    assert!(result.get("retries_attempted").is_some() || result.get("retry_count").is_some());
}
#[tokio::test]
async fn test_webhook_caller_timeout() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": format!("{}/delay/10", test_endpoints::HTTPBIN_BASE),
        "timeout": 2
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            // Check if it's an error response
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            assert!(
                !output_value["success"].as_bool().unwrap_or(true),
                "Expected error response, got success: {}",
                output_value
            );

            // Extract error message from various possible locations
            let error_msg = if let Some(error_str) = output_value["error"].as_str() {
                error_str.to_lowercase()
            } else if let Some(error_obj) = output_value["error"].as_object() {
                if let Some(msg) = error_obj.get("message").and_then(|m| m.as_str()) {
                    msg.to_lowercase()
                } else {
                    serde_json::to_string(error_obj)
                        .unwrap_or_default()
                        .to_lowercase()
                }
            } else if let Some(result) = output_value.get("result") {
                if let Some(err) = result.get("error").and_then(|e| e.as_str()) {
                    err.to_lowercase()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            assert!(
                error_msg.contains("timeout")
                    || error_msg.contains("elapsed")
                    || error_msg.contains("error sending request")
                    || error_msg.contains("timed out"),
                "Expected timeout-related error, got: '{}'",
                error_msg
            );
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            assert!(
                err_str.contains("timeout")
                    || err_str.contains("elapsed")
                    || err_str.contains("error sending request")
                    || err_str.contains("timed out"),
                "Expected timeout-related error, got: '{}'",
                err_str
            );
        }
    }
}
#[tokio::test]
async fn test_webhook_caller_invalid_url() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "not-a-url",
        "payload": {"test": "data"}
    }))
    .unwrap();

    let result = tool.execute(input, context).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("URL") || error.to_string().contains("url"));
}
#[tokio::test]
async fn test_webhook_caller_custom_method() {
    let tool = WebhookCallerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": format!("{}/put", test_endpoints::HTTPBIN_BASE),
        "method": "PUT",
        "payload": {"updated": true}
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    let status_code = result["status_code"].as_u64().unwrap_or(0);
    assert!(
        status_code == 200 || status_code == 405 || status_code == 404,
        "Expected status 200, 404, or 405, got: {}",
        status_code
    );
}
