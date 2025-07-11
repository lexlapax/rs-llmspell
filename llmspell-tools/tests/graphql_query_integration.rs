//! Integration tests for GraphQLQueryTool

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::{AgentInput, ExecutionContext},
};
use llmspell_tools::GraphQLQueryTool;
use serde_json::json;

#[tokio::test]
async fn test_graphql_tool_creation() {
    let tool = GraphQLQueryTool::default();

    assert_eq!(tool.metadata().name, "graphql-query-tool");
    assert_eq!(tool.category().to_string(), "api");
    assert!(matches!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Privileged
    ));
}

#[tokio::test]
async fn test_graphql_introspection() {
    let tool = GraphQLQueryTool::default();

    // Using countries.trevorblades.com public GraphQL API
    let input = AgentInput::text("introspect schema").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "introspection",
            "endpoint": "https://countries.trevorblades.com/"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check response contains schema
    assert!(output.text.contains("__schema"));
    assert!(output.text.contains("queryType"));
    assert!(output.text.contains("Country"));
    assert!(output.text.contains("cached"));
}

#[tokio::test]
async fn test_graphql_query() {
    let tool = GraphQLQueryTool::default();

    // Query for a specific country
    let input = AgentInput::text("query country").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "endpoint": "https://countries.trevorblades.com/",
            "query": r#"
                query GetCountry($code: ID!) {
                    country(code: $code) {
                        name
                        capital
                        currency
                        emoji
                    }
                }
            "#,
            "variables": {
                "code": "US"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check response contains expected data
    assert!(output.text.contains("United States"));
    assert!(output.text.contains("Washington"));
    assert!(output.text.contains("USD"));
    assert!(output.text.contains("🇺🇸"));

    // Check metadata
    let metadata = &output.metadata;
    assert_eq!(metadata.extra["operation"], "query");
}

#[tokio::test]
async fn test_graphql_query_without_variables() {
    let tool = GraphQLQueryTool::default();

    // Query all continents
    let input = AgentInput::text("query continents").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "https://countries.trevorblades.com/",
            "query": r#"
                {
                    continents {
                        code
                        name
                    }
                }
            "#
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check response contains continents
    assert!(output.text.contains("Africa"));
    assert!(output.text.contains("Europe"));
    assert!(output.text.contains("Asia"));
    assert!(output.text.contains("North America"));
}

#[tokio::test]
async fn test_graphql_with_custom_headers() {
    let tool = GraphQLQueryTool::default();

    let input = AgentInput::text("query with headers").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "https://countries.trevorblades.com/",
            "query": "{ continents { code } }",
            "headers": {
                "X-Custom-Header": "test-value"
            }
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should still work with custom headers
    assert!(output.text.contains("continents"));
}

#[tokio::test]
async fn test_graphql_error_handling() {
    let tool = GraphQLQueryTool::default();

    // Invalid query
    let input = AgentInput::text("invalid query").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "https://countries.trevorblades.com/",
            "query": "{ invalidField }"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;

    // Query with invalid field should fail
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("invalidField") || e.to_string().contains("errors"));
    }
}

#[tokio::test]
async fn test_graphql_depth_limit() {
    let tool = GraphQLQueryTool::default();

    // Very deeply nested query
    let deep_query = r#"
        {
            continents {
                countries {
                    states {
                        cities {
                            districts {
                                streets {
                                    buildings {
                                        floors {
                                            rooms {
                                                furniture {
                                                    items {
                                                        name
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let input = AgentInput::text("deep query").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "https://countries.trevorblades.com/",
            "query": deep_query
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;

    // Should fail due to depth limit
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("depth"));
    }
}

#[tokio::test]
async fn test_invalid_endpoint() {
    let tool = GraphQLQueryTool::default();

    let input = AgentInput::text("invalid endpoint").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "not-a-valid-url",
            "query": "{ test }"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_missing_endpoint() {
    let tool = GraphQLQueryTool::default();

    let input = AgentInput::text("no endpoint").with_parameter(
        "parameters".to_string(),
        json!({
            "query": "{ test }"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e
            .to_string()
            .contains("Missing required parameter 'endpoint'"));
    }
}

#[tokio::test]
async fn test_missing_query_for_query_operation() {
    let tool = GraphQLQueryTool::default();

    let input = AgentInput::text("no query").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "https://countries.trevorblades.com/",
            "operation": "query"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Missing required parameter 'query'"));
    }
}

#[tokio::test]
async fn test_subscription_not_supported() {
    let tool = GraphQLQueryTool::default();

    let input = AgentInput::text("subscription").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "subscription",
            "endpoint": "https://countries.trevorblades.com/",
            "query": "subscription { test }"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("subscriptions not yet supported"));
    }
}

#[tokio::test]
async fn test_graphql_with_operation_name() {
    let tool = GraphQLQueryTool::default();

    // Note: countries.trevorblades.com doesn't support operation_name parameter
    // So we'll just test that the query works without it
    let input = AgentInput::text("named operation").with_parameter(
        "parameters".to_string(),
        json!({
            "endpoint": "https://countries.trevorblades.com/",
            "query": r#"
                query GetUS {
                    country(code: "US") {
                        name
                        capital
                    }
                }
            "#
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should execute the GetUS operation
    assert!(output.text.contains("United States"));
    assert!(output.text.contains("Washington"));
}

#[tokio::test]
async fn test_graphql_schema_caching() {
    let tool = GraphQLQueryTool::default();

    // First introspection
    let input1 = AgentInput::text("introspect 1").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "introspection",
            "endpoint": "https://countries.trevorblades.com/"
        }),
    );

    let start1 = std::time::Instant::now();
    let output1 = tool
        .execute(input1, ExecutionContext::default())
        .await
        .unwrap();
    let duration1 = start1.elapsed();

    // Second introspection (should be cached)
    let input2 = AgentInput::text("introspect 2").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "introspection",
            "endpoint": "https://countries.trevorblades.com/"
        }),
    );

    let start2 = std::time::Instant::now();
    let output2 = tool
        .execute(input2, ExecutionContext::default())
        .await
        .unwrap();
    let duration2 = start2.elapsed();

    // Both should have the same schema
    assert_eq!(output1.text, output2.text);

    // Second request should be significantly faster (cached)
    // Note: This is a heuristic test and might be flaky
    assert!(duration2 < duration1 / 2 || duration2.as_millis() < 50);
}
