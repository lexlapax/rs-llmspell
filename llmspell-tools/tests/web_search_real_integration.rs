//! ABOUTME: Real integration tests for `WebSearchTool` with actual API calls
//! ABOUTME: Tests `DuckDuckGo` and other providers with network connectivity

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_tools::search::{web_search::WebSearchConfig, WebSearchTool};
use serde_json::Value;

/// Test basic DuckDuckGo search functionality with real API call
#[tokio::test]
#[ignore = "external,integration"]
async fn test_duckduckgo_real_search() {
    // Create config with only DuckDuckGo
    let config = WebSearchConfig {
        default_provider: "duckduckgo".to_string(),
        ..Default::default()
    };

    let tool = WebSearchTool::new(config).expect("Failed to create WebSearchTool");

    // Search for something that should definitely return results
    let input = AgentInput::text("search test").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia",
            "provider": "duckduckgo"
        }),
    );

    let context = ExecutionContext::default();
    let result = tool.execute(input, context).await;

    // Debug output
    match &result {
        Ok(output) => {
            println!("Search succeeded!");
            println!("Output: {}", output.text);
            let response: Value = serde_json::from_str(&output.text).unwrap();
            println!("Parsed response: {response:#?}");
        }
        Err(e) => {
            println!("Search failed with error: {e}");
            panic!("DuckDuckGo search should work without API key");
        }
    }

    // Verify the result
    assert!(result.is_ok(), "DuckDuckGo search should succeed");
    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["operation"], "search");
    assert!(response["result"]["results"].is_array());
}

/// Test provider fallback mechanism
#[tokio::test]
#[ignore = "external,integration"]
async fn test_provider_fallback() {
    let config = WebSearchConfig {
        // Set a non-existent provider as default
        default_provider: "nonexistent".to_string(),
        // Ensure DuckDuckGo is in the fallback chain
        fallback_chain: vec!["nonexistent".to_string(), "duckduckgo".to_string()],
        ..Default::default()
    };

    let tool = WebSearchTool::new(config).expect("Failed to create WebSearchTool");

    // Use a query that's more likely to return results from DuckDuckGo
    let input = AgentInput::text("search test").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia"
        }),
    );

    let context = ExecutionContext::default();
    let result = tool.execute(input, context).await;

    // Debug output for failing case
    if let Err(e) = &result {
        eprintln!("Fallback test failed with error: {e}");
    }

    // Should succeed using fallback
    assert!(result.is_ok(), "Should fallback to DuckDuckGo");

    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    // Check the response structure
    assert_eq!(response["success"], true);
    assert_eq!(response["operation"], "search");

    // If we got results, verify they came from DuckDuckGo
    if let Some(results) = response["result"]["results"].as_array() {
        if !results.is_empty() {
            assert_eq!(response["result"]["provider"], "duckduckgo");
        }
    }
}

/// Test search with specific parameters
#[tokio::test]
#[ignore = "external,integration"]
async fn test_search_with_parameters() {
    let config = WebSearchConfig::default();
    let tool = WebSearchTool::new(config).expect("Failed to create WebSearchTool");

    let input = AgentInput::text("search test").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Rust programming language",
            "max_results": 5,
            "safe_search": true
        }),
    );

    let context = ExecutionContext::default();
    let result = tool.execute(input, context).await;

    if let Err(e) = &result {
        eprintln!("Search error: {e}");
    }

    assert!(result.is_ok());
    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    // Check results count
    if let Some(results) = response["result"]["results"].as_array() {
        assert!(results.len() <= 5, "Should respect max_results parameter");

        // Verify result structure
        for result in results {
            assert!(result["title"].is_string());
            assert!(result["url"].is_string());
            assert!(result["snippet"].is_string());
        }
    }
}

/// Test error handling for invalid queries
#[tokio::test]
#[ignore = "external,integration"]
async fn test_error_handling() {
    let config = WebSearchConfig::default();
    let tool = WebSearchTool::new(config).expect("Failed to create WebSearchTool");

    // Test with empty query
    let input = AgentInput::text("search test").with_parameter(
        "parameters",
        serde_json::json!({
            "input": ""
        }),
    );

    let context = ExecutionContext::default();
    let result = tool.execute(input, context).await;

    // Empty query might still succeed but return no results
    // or might fail - either is acceptable
    if result.is_ok() {
        let output = result.unwrap();
        let response: Value = serde_json::from_str(&output.text).unwrap();

        if response["success"] == true {
            // If it succeeded, results should be empty or minimal
            let results = response["result"]["results"].as_array().unwrap();
            println!("Empty query returned {} results", results.len());
        }
    }
}

/// Test that validates the WebSearchTool meets Phase 3.0 standards
#[tokio::test]
#[ignore = "external,integration"]
async fn test_phase_30_compliance() {
    let config = WebSearchConfig::default();
    let tool = WebSearchTool::new(config).expect("Failed to create WebSearchTool");

    // 1. Test parameter standardization - uses "input" as primary parameter
    let schema = tool.schema();
    let input_param = schema
        .parameters
        .iter()
        .find(|p| p.name == "input")
        .expect("Must have 'input' parameter");
    assert!(input_param.required);

    // 2. Test ResponseBuilder pattern
    let input = AgentInput::text("search test").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "test query"
        }),
    );

    let context = ExecutionContext::default();
    let result = tool.execute(input, context).await;

    if result.is_ok() {
        let output = result.unwrap();
        let response: Value = serde_json::from_str(&output.text).unwrap();

        // Verify ResponseBuilder structure
        assert!(response["operation"].is_string());
        assert!(response["success"].is_boolean());
        assert!(response["message"].is_string());
        assert!(response["result"].is_object());
    }
}

/// Debug test to see raw DuckDuckGo API response
#[tokio::test]
#[ignore = "external,debug"] // Run with --ignored to see debug output
async fn debug_duckduckgo_api() {
    use reqwest::Client;

    let client = Client::new();
    let url = "https://api.duckduckgo.com/";
    let params = vec![
        ("q", "Wikipedia"),
        ("format", "json"),
        ("no_html", "1"),
        ("skip_disambig", "1"),
    ];

    println!("Making request to DuckDuckGo API...");
    let response = client
        .get(url)
        .query(&params)
        .send()
        .await
        .expect("Failed to make request");

    println!("Status: {}", response.status());
    let text = response.text().await.expect("Failed to get response text");
    println!("Response length: {} bytes", text.len());

    // Try to parse as JSON
    match serde_json::from_str::<Value>(&text) {
        Ok(json) => {
            println!("Parsed JSON response:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            println!("Failed to parse JSON: {e}");
            println!("Raw response: {text}");
        }
    }
}

/// Test with environment variables for API keys
#[tokio::test]
#[ignore = "external,api_keys"] // Only run if API keys are configured
async fn test_with_api_keys() {
    // This test requires environment variables to be set:
    // WEBSEARCH_GOOGLE_API_KEY
    // WEBSEARCH_GOOGLE_SEARCH_ENGINE_ID
    // WEBSEARCH_BRAVE_API_KEY
    // etc.

    let config = WebSearchConfig::from_env();
    let tool = WebSearchTool::new(config).expect("Failed to create WebSearchTool");

    // Test each configured provider
    let providers = vec!["google", "brave", "serpapi", "serperdev"];

    for provider in providers {
        println!("Testing provider: {provider}");

        let input = AgentInput::text("search test").with_parameter(
            "parameters",
            serde_json::json!({
                "input": "artificial intelligence",
                "provider": provider
            }),
        );

        let context = ExecutionContext::default();
        let result = tool.execute(input, context).await;

        match result {
            Ok(output) => {
                let response: Value = serde_json::from_str(&output.text).unwrap();
                if response["success"] == true {
                    println!("✓ {provider} provider working");
                } else {
                    println!("✗ {provider} provider returned success=false");
                }
            }
            Err(e) => {
                println!("✗ {provider} provider failed: {e}");
            }
        }
    }
}
