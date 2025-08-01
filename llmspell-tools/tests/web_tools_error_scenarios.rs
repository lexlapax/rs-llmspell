//! ABOUTME: Comprehensive error scenario tests for web tools
//! ABOUTME: Tests various failure modes and edge cases for external integration tools

mod common;

use common::*;
use llmspell_core::BaseAgent;
use llmspell_tools::{
    ApiTesterTool, SitemapCrawlerTool, UrlAnalyzerTool, WebScraperTool, WebhookCallerTool,
    WebpageMonitorTool,
};
use serde_json::json;

/// Test network timeouts across all tools
mod timeout_tests {
    use super::*;

    async fn test_tool_timeout<T: BaseAgent>(tool: T, tool_name: &str) {
        let context = create_test_context();

        // Use unreachable endpoint with 1 second timeout
        let input = create_agent_input(json!({
            "input": "http://1.2.3.4:9999/test",
            "timeout": 1
        }))
        .unwrap();

        match tool.execute(input, context).await {
            Ok(output) => {
                println!("{} timeout test: Got response", tool_name);
                // When using unreachable IPs, we might get "error sending request" instead of "timeout"
                let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();
                assert!(!output_value["success"].as_bool().unwrap_or(true));

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
                    // Some tools put error in result.error
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
                        || error_msg.contains("connection")
                        || error_msg.contains("failed to fetch"),
                    "Expected timeout-related error, got: '{}'",
                    error_msg
                );
            }
            Err(e) => {
                println!("{} timeout test: Got error: {}", tool_name, e);
                let err_str = e.to_string().to_lowercase();
                assert!(
                    err_str.contains("timeout")
                        || err_str.contains("elapsed")
                        || err_str.contains("failed to fetch")
                        || err_str.contains("error sending request")
                        || err_str.contains("connection")
                );
            }
        }
    }
    #[tokio::test]
    async fn test_all_tools_timeout() {
        println!("Testing ApiTester timeout...");
        test_tool_timeout(ApiTesterTool::new(), "ApiTester").await;
        println!("Testing WebScraper timeout...");
        test_tool_timeout(WebScraperTool::default(), "WebScraper").await;
        println!("Testing WebhookCaller timeout...");
        test_tool_timeout(WebhookCallerTool::new(), "WebhookCaller").await;
        println!("Testing WebpageMonitor timeout...");
        test_tool_timeout(WebpageMonitorTool::new(), "WebpageMonitor").await;
        println!("Testing SitemapCrawler timeout...");
        test_tool_timeout(SitemapCrawlerTool::new(), "SitemapCrawler").await;
        println!("All timeout tests completed");
    }
}

/// Test invalid URL handling
mod invalid_url_tests {
    use super::*;

    async fn test_tool_invalid_url<T: BaseAgent>(tool: T, tool_name: &str) {
        let context = create_test_context();

        let test_cases = vec![
            ("not-a-url", "plain text"),
            ("ftp://unsupported.com", "unsupported protocol"),
            ("http://", "incomplete URL"),
            ("://broken", "malformed URL"),
            ("", "empty string"),
        ];

        for (url, case_name) in test_cases {
            let input = create_agent_input(json!({
                "input": url
            }))
            .unwrap();

            match tool.execute(input, context.clone()).await {
                Ok(output) => {
                    println!("{} {} test: Got response", tool_name, case_name);
                    // Check if it's an error response
                    let output_value: serde_json::Value =
                        serde_json::from_str(&output.text).unwrap();
                    if !output_value["success"].as_bool().unwrap_or(true) {
                        // It's an error response - check error message
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
                        } else {
                            "".to_string()
                        };

                        assert!(
                            error_msg.contains("url")
                                || error_msg.contains("invalid")
                                || error_msg.contains("request failed")
                                || error_msg.contains("builder error")
                        );
                    } else {
                        panic!("Expected error response, got success: {}", output_value);
                    }
                }
                Err(e) => {
                    println!("{} {} test: Got error: {}", tool_name, case_name, e);
                    let err_str = e.to_string().to_lowercase();
                    assert!(
                        err_str.contains("url")
                            || err_str.contains("invalid")
                            || err_str.contains("request failed")
                            || err_str.contains("builder error")
                    );
                }
            }
        }
    }
    #[tokio::test]
    async fn test_all_tools_invalid_urls() {
        test_tool_invalid_url(ApiTesterTool::new(), "ApiTester").await;
        test_tool_invalid_url(WebScraperTool::default(), "WebScraper").await;
        test_tool_invalid_url(UrlAnalyzerTool::new(), "UrlAnalyzer").await;
        test_tool_invalid_url(WebhookCallerTool::new(), "WebhookCaller").await;
        test_tool_invalid_url(WebpageMonitorTool::new(), "WebpageMonitor").await;
        test_tool_invalid_url(SitemapCrawlerTool::new(), "SitemapCrawler").await;
    }
}

/// Test network failure scenarios
mod network_failure_tests {
    use super::*;
    #[tokio::test]
    async fn test_dns_resolution_failure() {
        let tools: Vec<(&str, Box<dyn BaseAgent>)> = vec![
            ("ApiTester", Box::new(ApiTesterTool::new())),
            ("WebScraper", Box::new(WebScraperTool::default())),
            ("WebhookCaller", Box::new(WebhookCallerTool::new())),
        ];

        for (name, tool) in tools {
            let context = create_test_context();
            let input = create_agent_input(json!({
                "input": test_endpoints::INVALID_URL
            }))
            .unwrap();

            match tool.execute(input, context).await {
                Ok(output) => {
                    assert_error_output(&output, "error");
                }
                Err(e) => {
                    println!("{} DNS failure: {}", name, e);
                    let err_str = e.to_string().to_lowercase();
                    assert!(
                        err_str.contains("error")
                            || err_str.contains("network")
                            || err_str.contains("resolve")
                            || err_str.contains("failed")
                            || err_str.contains("dns")
                            || err_str.contains("connection")
                    );
                }
            }
        }
    }
    #[tokio::test]
    async fn test_connection_refused() {
        // Local port that's likely not in use
        let refused_url = "http://localhost:59999";

        let tool = ApiTesterTool::new();
        let context = create_test_context();
        let input = create_agent_input(json!({
            "input": refused_url
        }))
        .unwrap();

        match tool.execute(input, context).await {
            Ok(output) => {
                assert_error_output(&output, "error");
            }
            Err(e) => {
                let err_str = e.to_string().to_lowercase();
                assert!(
                    err_str.contains("connection")
                        || err_str.contains("refused")
                        || err_str.contains("error")
                );
            }
        }
    }
}

/// Test HTTP error status codes
mod http_status_tests {
    use super::*;
    #[tokio::test]
    async fn test_http_error_statuses() {
        let statuses = vec![400, 401, 403, 404, 500, 502, 503];
        let tool = ApiTesterTool::new();

        for status in statuses {
            let context = create_test_context();
            let input = create_agent_input(json!({
                "input": format!("{}/{}", test_endpoints::HTTPBIN_STATUS, status)
            }))
            .unwrap();

            let output = match tool.execute(input, context).await {
                Ok(output) => output,
                Err(e) => {
                    // If we get a network error, skip this status code test
                    if e.to_string().contains("error sending request")
                        || e.to_string().contains("Connection refused")
                    {
                        eprintln!("Warning: Network error testing status {}: {}", status, e);
                        eprintln!("This indicates httpbin.org is temporarily unavailable - skipping this status code");
                        continue;
                    }
                    // If it's not a network error, still fail the test
                    panic!("Unexpected error testing status {}: {}", status, e);
                }
            };

            let output_value: serde_json::Value = serde_json::from_str(&output.text).unwrap();

            // Should still succeed but return the error status
            assert!(output_value["success"].as_bool().unwrap());

            let actual_status = output_value["result"]["response"]["status_code"]
                .as_u64()
                .unwrap();

            // If we get 503, it means httpbin.org is down/overloaded - skip the specific assertion
            if actual_status == 503 {
                eprintln!("Warning: httpbin.org returned 503 (Service Unavailable) instead of expected {}", status);
                eprintln!("This indicates the external service is temporarily unavailable - skipping specific status assertion");
                continue; // Skip to next status code
            }

            assert_eq!(actual_status, status as u64);
        }
    }
}

/// Test rate limiting integration
mod rate_limit_tests {
    use super::*;
    #[tokio::test]
    async fn test_rapid_requests() {
        let _tool = ApiTesterTool::new();

        // Send 5 rapid requests
        let mut handles = vec![];

        for i in 0..5 {
            let context = create_test_context();
            let input = create_agent_input(json!({
                "input": format!("{}/get?request={}", test_endpoints::HTTPBIN_BASE, i)
            }))
            .unwrap();

            let tool_clone = ApiTesterTool::new();
            let handle = tokio::spawn(async move { tool_clone.execute(input, context).await });

            handles.push(handle);
        }

        // All should complete without rate limiting errors
        let mut success_count = 0;
        for handle in handles {
            if let Ok(result) = handle.await {
                if result.is_ok() {
                    success_count += 1;
                }
            }
        }

        // At least some should succeed
        assert!(success_count > 0);
    }
}

/// Test input validation edge cases
mod input_validation_tests {
    use super::*;
    #[tokio::test]
    async fn test_missing_required_parameters() {
        let tool = ApiTesterTool::new();
        let context = create_test_context();

        // Missing "input" parameter
        let input = create_agent_input(json!({
            "method": "GET"
        }))
        .unwrap();

        let result = tool.execute(input, context).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_invalid_parameter_types() {
        let tool = ApiTesterTool::new();
        let context = create_test_context();

        // Invalid timeout type (string instead of number)
        let input = create_agent_input(json!({
            "input": test_endpoints::HTTPBIN_GET,
            "timeout": "not a number"
        }))
        .unwrap();

        // Should either handle gracefully or error
        let _ = tool.execute(input, context).await;
    }
    #[tokio::test]
    async fn test_extremely_long_urls() {
        let tool = UrlAnalyzerTool::new();
        let context = create_test_context();

        // Create a very long URL
        let long_param = "x".repeat(10000);
        let input = create_agent_input(json!({
            "input": format!("https://example.com/?param={}", long_param)
        }))
        .unwrap();

        // Should handle without panic
        let _ = tool.execute(input, context).await;
    }
}
