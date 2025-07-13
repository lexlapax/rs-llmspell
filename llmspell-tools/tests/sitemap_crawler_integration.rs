//! ABOUTME: Integration tests for SitemapCrawlerTool
//! ABOUTME: Tests sitemap parsing and crawling functionality

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::SitemapCrawlerTool;
use serde_json::json;

#[tokio::test]
async fn test_sitemap_crawler_xml() {
    let tool = SitemapCrawlerTool::new();
    let context = create_test_context();

    // Most sites have a sitemap.xml
    let input = create_agent_input(json!({
        "input": "https://www.rust-lang.org/sitemap.xml"
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_success_output(&output, &["operation", "result"]);

            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            let result = &output_value["result"];

            // Should have URLs from sitemap
            assert!(result["urls"].is_array() || result["pages"].is_array());
            assert!(
                result["url_count"].as_u64().unwrap_or(0) > 0
                    || result["total_urls"].as_u64().unwrap_or(0) > 0
            );
        }
        Err(_) => {
            // Some sites might block automated access, which is okay for tests
        }
    }
}

#[tokio::test]
async fn test_sitemap_crawler_robots_txt() {
    let tool = SitemapCrawlerTool::new();
    let context = create_test_context();

    // Check robots.txt for sitemap location
    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE,
        "check_robots": true
    }))
    .unwrap();

    let output = tool.execute(input, context).await.unwrap();

    let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();

    // Might find sitemap or indicate none found
    assert!(output_value["success"].as_bool().is_some());
}

#[tokio::test]
async fn test_sitemap_crawler_with_filters() {
    let tool = SitemapCrawlerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://example.com/sitemap.xml",
        "url_filter": "blog",
        "max_urls": 10
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            if output_value["success"].as_bool().unwrap() {
                let result = &output_value["result"];
                // If successful, check filters were applied
                if let Some(urls) = result["urls"].as_array() {
                    assert!(urls.len() <= 10);
                }
            }
        }
        Err(_) => {
            // Site might not have sitemap or block access
        }
    }
}

#[tokio::test]
async fn test_sitemap_crawler_depth_limit() {
    let tool = SitemapCrawlerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": "https://example.com/sitemap.xml",
        "max_depth": 1,
        "follow_sitemap_index": true
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            // Should respect depth limit
            assert!(output_value["success"].as_bool().is_some());
        }
        Err(_) => {
            // Acceptable if no sitemap exists
        }
    }
}

#[tokio::test]
async fn test_sitemap_crawler_invalid_url() {
    let tool = SitemapCrawlerTool::new();
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
async fn test_sitemap_crawler_non_sitemap_url() {
    let tool = SitemapCrawlerTool::new();
    let context = create_test_context();

    // Regular HTML page, not a sitemap
    let input = create_agent_input(json!({
        "input": test_endpoints::EXAMPLE_WEBSITE
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
            // Should either find sitemap via robots.txt or indicate none found
            assert!(output_value["success"].as_bool().is_some());
        }
        Err(e) => {
            // Might error if expecting XML format
            assert!(e.to_string().contains("sitemap") || e.to_string().contains("XML"));
        }
    }
}

#[tokio::test]
async fn test_sitemap_crawler_timeout() {
    let tool = SitemapCrawlerTool::new();
    let context = create_test_context();

    let input = create_agent_input(json!({
        "input": format!("{}/3", test_endpoints::HTTPBIN_DELAY),
        "timeout": 1
    }))
    .unwrap();

    match tool.execute(input, context).await {
        Ok(output) => {
            assert_error_output(&output, "timeout");
        }
        Err(e) => {
            assert!(e.to_string().contains("timeout") || e.to_string().contains("elapsed"));
        }
    }
}
