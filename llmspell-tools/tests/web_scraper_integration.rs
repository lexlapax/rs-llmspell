//! ABOUTME: Integration tests for WebScraperTool
//! ABOUTME: Tests web scraping functionality with real websites

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::WebScraperTool;
use serde_json::json;

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "external,tool,integration"]
async fn test_web_scraper_basic() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "extract_links": true,
        "extract_images": true
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let content = &output_value["result"]["content"];

    // Example.com should have these basic elements
    assert!(content["title"].as_str().unwrap().contains("Example"));
    assert!(!content["text"].as_str().unwrap().is_empty());
    assert!(content["links"].is_array());
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "external,tool,integration"]
async fn test_web_scraper_selectors() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "selector": "h1"
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let content = &output_value["result"]["content"];

    // Should extract h1 content
    assert!(content["selected_content"].is_array());
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "external,tool,integration"]
async fn test_web_scraper_metadata() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "extract_meta": true
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let content = &output_value["result"]["content"];

    // Should have metadata
    assert!(content["metadata"].is_object() || content["meta_tags"].is_object());
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "external,tool,integration"]
async fn test_web_scraper_httpbin_html() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": format!("{}/html", test_endpoints::HTTPBIN_BASE),
        "extract_links": true
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    assert_success_output(&output, &["operation", "result"]);

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let content = &output_value["result"]["content"];

    // httpbin.org/html returns a sample HTML page
    assert!(content["text"]
        .as_str()
        .unwrap()
        .contains("Herman Melville"));
    assert!(content["links"].is_array());
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "external,tool,integration"]
async fn test_web_scraper_invalid_url() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "not-a-valid-url"
    }))
    .unwrap();

    let result = tool.execute(input, context).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid URL"));
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "tool,integration"]
async fn test_web_scraper_network_error() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": test_endpoints::INVALID_URL
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            // Some implementations might return success with error in result
            assert_error_output(&output, "error");
        }
        Err(e) => {
            // Others might return an error directly
            assert!(e.to_string().contains("error") || e.to_string().contains("network"));
        }
    }
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
#[ignore = "external,tool,integration,slow"]
async fn test_web_scraper_timeout() {
    let tool = WebScraperTool::default();
    let context = create_test_context();

    // Request with 1 second timeout to an unreachable endpoint
    let input = create_agent_input(json!({
        "input": "http://1.2.3.4:9999/test",
        "timeout": 1
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_error_output(&output, "timeout");
        }
        Err(e) => {
            let error_str = e.to_string();
            assert!(
                error_str.contains("timeout")
                    || error_str.contains("elapsed")
                    || error_str.contains("Failed to fetch URL")
                    || error_str.contains("error sending request")
            );
        }
    }
}
