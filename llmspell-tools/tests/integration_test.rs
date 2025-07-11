//! Integration tests for llmspell-tools

use llmspell_core::{
    traits::tool::ToolCategory,
    types::{AgentInput, ExecutionContext},
};
use llmspell_tools::{search::web_search::WebSearchConfig, ToolRegistry, WebSearchTool};
use std::collections::HashMap;

#[tokio::test]
async fn test_web_search_tool_registration() {
    // Create registry
    let registry = ToolRegistry::new();

    // Create and register web search tool
    let config = WebSearchConfig::default();
    let search_tool = WebSearchTool::new(config).unwrap();

    registry
        .register("web_search".to_string(), search_tool)
        .await
        .unwrap();

    // Verify tool is registered
    assert!(registry.contains_tool("web_search").await);

    // Get tool info
    let tool_info = registry.get_tool_info("web_search").await.unwrap();
    assert_eq!(tool_info.name, "web_search");
    assert_eq!(tool_info.category, ToolCategory::Web);

    // Test discovery by category
    let web_tools = registry.get_tools_by_category(&ToolCategory::Web).await;
    assert!(web_tools.contains(&"web_search".to_string()));
}

#[tokio::test]
async fn test_web_search_tool_execution_through_registry() {
    // Create registry
    let registry = ToolRegistry::new();

    // Register web search tool
    let config = WebSearchConfig::default();
    let search_tool = WebSearchTool::new(config).unwrap();
    registry
        .register("web_search".to_string(), search_tool)
        .await
        .unwrap();

    // Get tool from registry
    let tool = registry.get_tool("web_search").await.unwrap();

    // Execute search
    let input = AgentInput {
        text: "search for rust".to_string(),
        media: vec![],
        context: None,
        parameters: {
            let mut map = HashMap::new();
            map.insert(
                "parameters".to_string(),
                serde_json::json!({
                    "input": "rust programming"
                }),
            );
            map
        },
        output_modalities: vec![],
    };

    let context = ExecutionContext::with_conversation("test".to_string());
    let result = tool.execute(input, context).await.unwrap();

    // Verify result contains the expected search results
    assert!(result.text.contains("rust programming"));
    assert!(result.text.contains("DuckDuckGo"));

    // For web search tools, metadata might not always be populated
    // The important thing is that the tool executed successfully
}
