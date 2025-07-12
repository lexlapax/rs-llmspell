//! ABOUTME: Integration tests for WebSearchTool public API
//! ABOUTME: Tests search functionality, parameter handling, and error cases

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::{AgentInput, ExecutionContext},
    LLMSpellError,
};
use llmspell_tools::search::{WebSearchConfig, WebSearchTool};
use serde_json::Value;

#[tokio::test]
async fn test_basic_web_search() {
    let config = WebSearchConfig::default();
    let tool = WebSearchTool::new(config).unwrap();

    let input = AgentInput::text("search for rust programming")
        .with_parameter("input", "rust programming language");

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
    let input = AgentInput::text("search").with_parameter("input", "test query");
    let result = tool.validate_input(&input).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_with_options() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    // Test with max_results
    let input = AgentInput::text("search")
        .with_parameter("input", "machine learning")
        .with_parameter("max_results", 5);

    let result = tool.execute(input, context.clone()).await;
    assert!(result.is_ok());

    // Test with safe_search
    let input = AgentInput::text("search")
        .with_parameter("input", "educational content")
        .with_parameter("safe_search", true);

    let result = tool.execute(input, context.clone()).await;
    assert!(result.is_ok());

    // Test with language
    let input = AgentInput::text("search")
        .with_parameter("input", "bonjour")
        .with_parameter("language", "fr");

    let result = tool.execute(input, context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_types() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    // Test web search
    let input = AgentInput::text("search")
        .with_parameter("input", "technology news")
        .with_parameter("search_type", "web");

    let result = tool.execute(input, context.clone()).await;
    assert!(result.is_ok());

    // Test news search
    let input = AgentInput::text("search")
        .with_parameter("input", "latest technology")
        .with_parameter("search_type", "news");

    let result = tool.execute(input, context.clone()).await;
    assert!(result.is_ok());

    // Test image search
    let input = AgentInput::text("search")
        .with_parameter("input", "nature photography")
        .with_parameter("search_type", "images");

    let result = tool.execute(input, context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_duckduckgo_provider() {
    let tool = WebSearchTool::new(WebSearchConfig::default()).unwrap();
    let context = ExecutionContext::default();

    // DuckDuckGo should always be available
    let input = AgentInput::text("search")
        .with_parameter("input", "open source software")
        .with_parameter("provider", "duckduckgo");

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

    let input = AgentInput::text("search").with_parameter("input", "test query");

    let result = tool.execute(input, context).await;
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
    let mut config = WebSearchConfig::default();
    // Set up fallback chain with nonexistent provider first
    config.fallback_chain = vec![
        "nonexistent_provider".to_string(),
        "duckduckgo".to_string(), // This should work
    ];

    let tool = WebSearchTool::new(config).unwrap();
    let context = ExecutionContext::default();

    let input = AgentInput::text("search")
        .with_parameter("input", "fallback test")
        .with_parameter("provider", "nonexistent_provider");

    let result = tool.execute(input, context).await;

    // Should succeed using fallback
    assert!(result.is_ok());
}

#[cfg(feature = "rate_limiting")]
#[tokio::test]
async fn test_rate_limiting() {
    let config = WebSearchConfig::default();
    let tool = WebSearchTool::new(config).unwrap();
    let context = ExecutionContext::default();

    // This test would need specific setup to test rate limiting
    // For now, just verify the tool handles searches correctly
    let input = AgentInput::text("search")
        .with_parameter("input", "rate limit test")
        .with_parameter("provider", "duckduckgo");

    let result = tool.execute(input, context).await;
    assert!(result.is_ok());
}
