//! ABOUTME: Integration tests for WebSearchTool public API
//! ABOUTME: Tests search functionality, parameter handling, and error cases

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::AgentInput,
    ExecutionContext, LLMSpellError,
};
use llmspell_tools::search::{WebSearchConfig, WebSearchTool};
use serde_json::Value;

#[tokio::test]
async fn test_basic_web_search() {
    let config = WebSearchConfig::default();
    let tool = WebSearchTool::new(config).unwrap();

    let input = AgentInput::text("search for rust programming").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "rust programming language"
        }),
    );

    let context = ExecutionContext::default();
    let result = tool.execute(input, context).await;

    if let Err(e) = &result {
        eprintln!("Test failed with error: {}", e);
    }
    assert!(result.is_ok());
    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["operation"], "search");
}

#[tokio::test]
async fn test_parameter_validation() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();

    // Test missing input parameter
    let input = AgentInput::text("search");
    let result = tool.validate_input(&input).await;
    assert!(result.is_err());

    // Test with valid parameters
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia"
        }),
    );
    let result = tool.validate_input(&input).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_with_options() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    // Test with max_results
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Python programming",
            "max_results": 5
        }),
    );

    let result = tool.execute(input, context.clone()).await;
    if let Err(e) = &result {
        eprintln!("test_search_with_options (max_results) failed: {}", e);
    }
    assert!(result.is_ok());

    // Test with safe_search
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "science education",
            "safe_search": true
        }),
    );

    let result = tool.execute(input, context.clone()).await;
    if let Err(e) = &result {
        eprintln!("test_search_with_options (safe_search) failed: {}", e);
    }
    assert!(result.is_ok());

    // Test with language (using English to ensure results)
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia",
            "language": "en"
        }),
    );

    let result = tool.execute(input, context).await;
    if let Err(e) = &result {
        eprintln!("test_search_with_options (language) failed: {}", e);
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_types() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    // Test web search (DuckDuckGo supports this)
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia",
            "search_type": "web"
        }),
    );

    let result = tool.execute(input, context.clone()).await;
    if let Err(e) = &result {
        eprintln!("test_search_types (web) failed: {}", e);
    }
    assert!(result.is_ok());

    // Test that DuckDuckGo properly rejects non-web search types
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia news",
            "search_type": "news",
            "provider": "duckduckgo"
        }),
    );

    let result = tool.execute(input, context.clone()).await;
    // This should fail with a validation error
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e
            .to_string()
            .contains("DuckDuckGo only supports web search"));
    }

    // Test that invalid search type defaults to web
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia",
            "search_type": "invalid_type"
        }),
    );

    let result = tool.execute(input, context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_duckduckgo_provider() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    // DuckDuckGo should always be available
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "open source software",
            "provider": "duckduckgo"
        }),
    );

    let result = tool.execute(input, context).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    assert_eq!(response["success"], true);
    if let Some(result_data) = response.get("result") {
        assert_eq!(result_data["provider"], "duckduckgo");
    }
}

#[tokio::test]
async fn test_error_handling() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();

    let error = LLMSpellError::Network {
        message: "Test network error".to_string(),
        source: None,
    };

    let result = tool.handle_error(error).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    assert_eq!(response["success"], false);
    assert_eq!(response["operation"], "search");
}

#[tokio::test]
async fn test_tool_metadata() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();

    // Test category
    use llmspell_core::traits::tool::ToolCategory;
    assert!(matches!(tool.category(), ToolCategory::Web));

    // Test security level
    assert!(matches!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Safe
    ));

    // Test schema
    let schema = tool.schema();
    assert_eq!(schema.name, "web_search");
    assert!(!schema.parameters.is_empty());

    // Verify required parameter
    let input_param = schema
        .parameters
        .iter()
        .find(|p| p.name == "input")
        .expect("input parameter should exist");
    assert!(input_param.required);
}

#[tokio::test]
async fn test_response_format() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia"
        }),
    );

    let result = tool.execute(input, context).await;

    if let Err(e) = &result {
        eprintln!("test_response_format failed with error: {}", e);
    }
    assert!(result.is_ok());

    let output = result.unwrap();
    let response: Value = serde_json::from_str(&output.text).unwrap();

    // Verify response structure
    assert!(response["success"].is_boolean());
    assert!(response["operation"].is_string());
    assert!(response["message"].is_string());
    assert!(response["result"].is_object());

    let result_data = &response["result"];
    assert!(result_data["query"].is_string());
    assert!(result_data["provider"].is_string());
    assert!(result_data["count"].is_number());
    assert!(result_data["results"].is_array());
}

#[tokio::test]
async fn test_provider_fallback() {
    // Test fallback behavior: if no valid provider is found, it should use the default
    let config = WebSearchConfig {
        default_provider: "duckduckgo".to_string(),
        fallback_chain: vec![
            "duckduckgo".to_string(), // Ensure DuckDuckGo is in fallback chain
        ],
        ..Default::default()
    };

    let tool = WebSearchTool::new(config).unwrap();
    let context = ExecutionContext::default();

    // Test with a provider that's not in the config - should fallback to default
    let input = AgentInput::text("search").with_parameter(
        "parameters",
        serde_json::json!({
            "input": "Wikipedia",
            "provider": "nonexistent_provider"
        }),
    );

    let result = tool.execute(input, context).await;

    // Should succeed using fallback to DuckDuckGo
    if let Err(e) = &result {
        eprintln!("test_provider_fallback failed: {}", e);
    }
    assert!(result.is_ok());

    // Verify the response came from DuckDuckGo
    if let Ok(output) = result {
        let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();
        if response["success"] == true {
            // Should use DuckDuckGo as fallback
            assert_eq!(response["result"]["provider"], "duckduckgo");
        }
    }
}

// Rate limiting is tested in the real integration tests with actual API calls
