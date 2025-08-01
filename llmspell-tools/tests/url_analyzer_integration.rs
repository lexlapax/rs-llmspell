//! ABOUTME: Integration tests for UrlAnalyzerTool
//! ABOUTME: Tests URL parsing, validation, and analysis functionality

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::UrlAnalyzerTool;
use serde_json::json;

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_basic() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://example.com/path/to/page?param=value&foo=bar#section"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Check URL components
    assert_eq!(result["scheme"], "https");
    assert_eq!(result["host"], "example.com");
    assert_eq!(result["path"], "/path/to/page");
    assert!(result["query_params"].is_object());
    assert_eq!(result["query_params"]["param"], "value");
    assert_eq!(result["query_params"]["foo"], "bar");
    assert_eq!(result["fragment"], "section");
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_simple_url() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://example.com"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    assert_eq!(result["scheme"], "https");
    assert_eq!(result["host"], "example.com");
    assert_eq!(result["path"], "/");
    assert!(result["query_params"].as_object().unwrap().is_empty());
    assert!(result["fragment"].is_null());
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_with_port() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "http://localhost:8080/api/v1/users"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    assert_eq!(result["scheme"], "http");
    assert_eq!(result["host"], "localhost");
    assert_eq!(result["port"], 8080);
    assert_eq!(result["path"], "/api/v1/users");
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_with_auth() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://user:pass@example.com/secure"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    assert_eq!(result["host"], "example.com");
    assert_eq!(result["path"], "/secure");
    // Check if auth info is present (implementation dependent)
    assert!(result.get("username").is_some() || result.get("has_auth").is_some());
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_decode_params() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://example.com/search?q=hello%20world&category=books%2Fmagazines",
        "decode_params": true
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Check decoded parameters
    assert_eq!(result["query_params"]["q"], "hello world");
    assert_eq!(result["query_params"]["category"], "books/magazines");
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_invalid_url() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "not a valid url at all"
    }))
    .unwrap();

    let result = tool.execute(input, context).await;

    // Should return an error for invalid URL
    match result {
        Ok(output) => {
            assert_error_output(&output, "invalid");
        }
        Err(e) => {
            assert!(
                e.to_string().to_lowercase().contains("invalid")
                    || e.to_string().to_lowercase().contains("url")
            );
        }
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_relative_url() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "/path/to/resource"
    }))
    .unwrap();

    let result = tool.execute(input, context).await;

    // Relative URLs might be rejected or handled specially
    match result {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            if output_value["success"].as_bool().unwrap() {
                // If accepted, check it's identified as relative
                let result = &output_value["result"];
                assert!(
                    result
                        .get("is_relative")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        || result["scheme"].is_null()
                );
            } else {
                assert_error_output(&output, "absolute");
            }
        }
        Err(e) => {
            // It's okay to reject relative URLs
            assert!(e.to_string().contains("absolute") || e.to_string().contains("scheme"));
        }
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_url_analyzer_special_characters() {
    let tool = UrlAnalyzerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://example.com/path?key=value&special=!@$%^&*()"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let result = &output_value["result"];

    // Should handle special characters in query params
    assert!(result["query_params"].is_object());
    assert!(result["query_params"].get("special").is_some());
}
