//! Integration tests for HttpRequestTool

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_tools::HttpRequestTool;
use serde_json::json;

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_request_tool_creation() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    assert_eq!(tool.metadata().name, "http-request-tool");
    assert_eq!(tool.category().to_string(), "api");
    assert!(matches!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Safe
    ));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_get_request() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    // Using httpbin.org for testing
    let input = AgentInput::text("fetch data").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/get",
            "headers": {
                "X-Test-Header": "test-value"
            }
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;

    match result {
        Ok(output) => {
            // Parse response
            let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

            // Check response structure
            assert_eq!(response["operation"], "http_request");
            assert_eq!(response["success"], true);
            assert!(response["message"]
                .as_str()
                .unwrap()
                .contains("completed with status 200"));

            // Check result data
            let result = &response["result"];
            assert_eq!(result["status_code"], 200);
            assert_eq!(response["metadata"]["method"], "GET");
            assert!(result["headers"].is_object());
        }
        Err(e) => {
            eprintln!("Warning: HTTP GET test failed due to network issue: {}", e);
            eprintln!("This is likely due to httpbin.org being unavailable");
        }
    }
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_post_request() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("post data").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "POST",
            "input": "https://httpbin.org/post",
            "body": {
                "name": "test",
                "value": 123
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check response contains posted data
    assert!(output.text.contains("200"));
    assert!(output.text.contains("test"));
    assert!(output.text.contains("123"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_basic_auth() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("auth request").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/basic-auth/user/pass",
            "auth": {
                "type": "basic",
                "username": "user",
                "password": "pass"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should authenticate successfully
    assert!(output.text.contains("200"));
    assert!(output.text.contains("authenticated"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_bearer_auth() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("bearer auth").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/bearer",
            "auth": {
                "type": "bearer",
                "token": "test-token-123"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should include bearer token
    assert!(output.text.contains("200"));
    assert!(output.text.contains("authenticated"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_custom_headers() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("custom headers").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/headers",
            "headers": {
                "X-Custom-Header": "custom-value",
                "X-Another-Header": "another-value"
            }
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;

    match result {
        Ok(output) => {
            // Response should echo headers
            assert!(output.text.contains("X-Custom-Header"));
            assert!(output.text.contains("custom-value"));
        }
        Err(e) => {
            eprintln!(
                "Warning: HTTP custom headers test failed due to network issue: {}",
                e
            );
            eprintln!("This is likely due to httpbin.org being unavailable");
            // Skip test instead of panicking
        }
    }
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_error_handling() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    // Test 404 error
    let input = AgentInput::text("not found").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/status/404"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(output.text.contains("404"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_retry_logic() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    // Test retry on 503 (service unavailable)
    let input = AgentInput::text("retry test").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/status/503",
            "retry": {
                "max_attempts": 2,
                "initial_delay_ms": 100,
                "retry_on_status": [503]
            }
        }),
    );

    // Should retry but still fail
    let start = std::time::Instant::now();
    let result = tool.execute(input, ExecutionContext::default()).await;

    match result {
        Ok(output) => {
            // Should have retried (takes at least initial_delay_ms)
            assert!(start.elapsed().as_millis() >= 100);
            assert!(output.text.contains("503"));
        }
        Err(e) => {
            eprintln!(
                "Warning: HTTP retry logic test failed due to network issue: {}",
                e
            );
            eprintln!("This is likely due to httpbin.org being unavailable");
            // Skip test instead of panicking
        }
    }
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_json_response_parsing() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("json response").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/json"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should parse JSON response
    assert!(output.text.contains("slideshow"));
    assert!(output.text.contains("json"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "httpbin.org delay endpoint may not respect long delays"]
async fn test_http_timeout() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    // httpbin.org/delay delays response by N seconds
    let input = AgentInput::text("timeout test").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/delay/35" // 35 second delay (more than default 30s timeout)
        }),
    );

    // Execute without wrapping in timeout - the tool itself should timeout
    let result = tool.execute(input, ExecutionContext::default()).await;

    // The request should fail due to timeout
    assert!(result.is_err());
    if let Err(e) = result {
        // Should be a network/tool error due to timeout
        assert!(e.to_string().contains("request") || e.to_string().contains("timeout"));
    }
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_put_request() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("put data").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "PUT",
            "input": "https://httpbin.org/put",
            "body": {
                "updated": true,
                "id": 42
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Parse response
    let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["result"]["status_code"], 200);
    assert_eq!(response["metadata"]["method"], "PUT");
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_http_delete_request() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("delete resource").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "DELETE",
            "input": "https://httpbin.org/delete"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Parse response
    let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["result"]["status_code"], 200);
    assert_eq!(response["metadata"]["method"], "DELETE");
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "httpbin.org intermittent network issues"]
async fn test_http_api_key_auth() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("api key auth").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "https://httpbin.org/headers",
            "auth": {
                "type": "api_key",
                "key": "my-api-key-123",
                "header_name": "X-API-Key"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Headers should include the API key
    assert!(output.text.contains("X-Api-Key")); // httpbin normalizes header names
    assert!(output.text.contains("my-api-key-123"));
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_invalid_url() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("invalid url").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "input": "not-a-valid-url"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_missing_url() {
    let tool = HttpRequestTool::new(Default::default()).unwrap();

    let input = AgentInput::text("no url").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Missing required parameter 'input'"));
    }
}
