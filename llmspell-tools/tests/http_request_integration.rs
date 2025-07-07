//! Integration tests for HttpRequestTool

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::{AgentInput, ExecutionContext},
};
use llmspell_tools::HttpRequestTool;
use serde_json::json;

#[tokio::test]
async fn test_http_request_tool_creation() {
    let tool = HttpRequestTool::default();

    assert_eq!(tool.metadata().name, "http-request-tool");
    assert_eq!(tool.category().to_string(), "api");
    assert!(matches!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Privileged
    ));
}

#[tokio::test]
async fn test_http_get_request() {
    let tool = HttpRequestTool::default();

    // Using httpbin.org for testing
    let input = AgentInput::text("fetch data").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/get",
            "headers": {
                "X-Test-Header": "test-value"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check response
    assert!(output.text.contains("200"));
    assert!(output.text.contains("headers"));

    // Check metadata
    let metadata = &output.metadata;
    assert_eq!(metadata.extra["status_code"], 200);
    assert_eq!(metadata.extra["method"], "GET");
}

#[tokio::test]
async fn test_http_post_request() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("post data").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "POST",
            "url": "https://httpbin.org/post",
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

#[tokio::test]
async fn test_http_basic_auth() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("auth request").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/basic-auth/user/pass",
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

#[tokio::test]
async fn test_http_bearer_auth() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("bearer auth").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/bearer",
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

#[tokio::test]
async fn test_http_custom_headers() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("custom headers").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/headers",
            "headers": {
                "X-Custom-Header": "custom-value",
                "X-Another-Header": "another-value"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Response should echo headers
    assert!(output.text.contains("X-Custom-Header"));
    assert!(output.text.contains("custom-value"));
}

#[tokio::test]
async fn test_http_error_handling() {
    let tool = HttpRequestTool::default();

    // Test 404 error
    let input = AgentInput::text("not found").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/status/404"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(output.text.contains("404"));
}

#[tokio::test]
async fn test_http_retry_logic() {
    let tool = HttpRequestTool::default();

    // Test retry on 503 (service unavailable)
    let input = AgentInput::text("retry test").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/status/503",
            "retry": {
                "max_attempts": 2,
                "initial_delay_ms": 100,
                "retry_on_status": [503]
            }
        }),
    );

    // Should retry but still fail
    let start = std::time::Instant::now();
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should have retried (takes at least initial_delay_ms)
    assert!(start.elapsed().as_millis() >= 100);
    assert!(output.text.contains("503"));
}

#[tokio::test]
async fn test_http_json_response_parsing() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("json response").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/json"
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

#[tokio::test]
#[ignore = "httpbin.org delay endpoint may not respect long delays"]
async fn test_http_timeout() {
    let tool = HttpRequestTool::default();

    // httpbin.org/delay delays response by N seconds
    let input = AgentInput::text("timeout test").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/delay/35" // 35 second delay (more than default 30s timeout)
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

#[tokio::test]
async fn test_http_put_request() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("put data").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "PUT",
            "url": "https://httpbin.org/put",
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

    assert!(output.text.contains("200"));
    assert!(output.text.contains("updated"));
    assert_eq!(output.metadata.extra["method"], "PUT");
}

#[tokio::test]
async fn test_http_delete_request() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("delete resource").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "DELETE",
            "url": "https://httpbin.org/delete"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(output.text.contains("200"));
    assert_eq!(output.metadata.extra["method"], "DELETE");
}

#[tokio::test]
async fn test_http_api_key_auth() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("api key auth").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "https://httpbin.org/headers",
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

#[tokio::test]
async fn test_invalid_url() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("invalid url").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET",
            "url": "not-a-valid-url"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_missing_url() {
    let tool = HttpRequestTool::default();

    let input = AgentInput::text("no url").with_parameter(
        "parameters".to_string(),
        json!({
            "method": "GET"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Missing required parameter 'url'"));
    }
}
