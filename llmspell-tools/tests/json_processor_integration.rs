//! Integration tests for JsonProcessorTool

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_tools::JsonProcessorTool;
use serde_json::json;

#[tokio::test]
async fn test_json_processor_complex_workflow() {
    let tool = JsonProcessorTool::default();

    // Test 1: Complex nested JSON transformation
    let complex_json = json!({
        "company": {
            "name": "TechCorp",
            "employees": [
                {"name": "Alice", "department": "Engineering", "salary": 120000},
                {"name": "Bob", "department": "Sales", "salary": 90000},
                {"name": "Charlie", "department": "Engineering", "salary": 110000}
            ],
            "departments": {
                "Engineering": {"budget": 5000000, "head": "Alice"},
                "Sales": {"budget": 2000000, "head": "David"}
            }
        }
    });

    // Extract company name
    let input = AgentInput::text("extract company name").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": complex_json.clone(),
            "query": ".company.name"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    assert_eq!(output.text.trim(), "\"TechCorp\"");

    // Extract all employee names
    let input = AgentInput::text("extract employees").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": complex_json.clone(),
            "query": ".company.employees"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let employees: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(employees.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_json_processor_schema_validation_complex() {
    let tool = JsonProcessorTool::default();

    // Complex schema with nested objects and arrays
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "product": {
                "type": "object",
                "properties": {
                    "id": {"type": "integer", "minimum": 1},
                    "name": {"type": "string", "minLength": 1},
                    "price": {"type": "number", "minimum": 0},
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "minItems": 1
                    }
                },
                "required": ["id", "name", "price"]
            }
        },
        "required": ["product"]
    });

    // Valid product
    let valid_product = json!({
        "product": {
            "id": 123,
            "name": "Laptop",
            "price": 999.99,
            "tags": ["electronics", "computers"]
        }
    });

    let input = AgentInput::text("validate product").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "validate",
            "input": valid_product,
            "schema": schema.clone()
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let validation_result: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(validation_result["is_valid"], true);

    // Invalid product (missing required field)
    let invalid_product = json!({
        "product": {
            "id": 123,
            "name": "Laptop"
            // missing price
        }
    });

    let input = AgentInput::text("validate invalid product").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "validate",
            "input": invalid_product,
            "schema": schema
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let validation_result: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(validation_result["is_valid"], false);

    let errors = validation_result["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn test_json_processor_array_filtering() {
    let tool = JsonProcessorTool::default();

    let products = json!([
        {"name": "Laptop", "price": 999, "category": "electronics"},
        {"name": "Book", "price": 15, "category": "books"},
        {"name": "Phone", "price": 699, "category": "electronics"},
        {"name": "Pen", "price": 2, "category": "stationery"}
    ]);

    // Filter by category
    let input = AgentInput::text("filter electronics").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": products,
            "query": ".[] | select(.category == \"electronics\")"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let filtered: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    let items = filtered.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| item["category"] == "electronics"));
}

#[tokio::test]
async fn test_json_processor_merge_complex() {
    let tool = JsonProcessorTool::default();

    let configs = json!([
        {
            "database": {
                "host": "localhost",
                "port": 5432
            },
            "cache": {
                "enabled": true
            }
        },
        {
            "database": {
                "username": "admin",
                "password": "secret"
            },
            "api": {
                "version": "v2"
            }
        },
        {
            "cache": {
                "ttl": 3600
            }
        }
    ]);

    let input = AgentInput::text("merge configs").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": configs,
            "query": "reduce .[] as $item ({}; . * $item)"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let merged: serde_json::Value = serde_json::from_str(&output.text).unwrap();

    // Check merged structure - the * operator does a deep merge
    assert_eq!(merged["database"]["host"], "localhost");
    assert_eq!(merged["database"]["port"], 5432);
    assert_eq!(merged["database"]["username"], "admin");
    assert_eq!(merged["database"]["password"], "secret");

    assert_eq!(merged["cache"]["enabled"], true);
    assert_eq!(merged["cache"]["ttl"], 3600);

    assert_eq!(merged["api"]["version"], "v2");
}

#[tokio::test]
async fn test_json_processor_tool_metadata() {
    let tool = JsonProcessorTool::default();

    // Test tool metadata
    assert_eq!(
        tool.category(),
        llmspell_core::traits::tool::ToolCategory::Data
    );
    assert_eq!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Safe
    );

    let schema = tool.schema();
    assert_eq!(schema.name, "json_processor");
    assert!(schema.description.contains("JSON"));

    // Check parameters
    let params = &schema.parameters;
    assert!(params.iter().any(|p| p.name == "operation"));
    assert!(params.iter().any(|p| p.name == "input"));
    assert!(params.iter().any(|p| p.name == "query"));
    assert!(params.iter().any(|p| p.name == "schema"));
}

#[tokio::test]
async fn test_json_processor_error_handling() {
    let tool = JsonProcessorTool::default();

    // Test invalid operation
    let input = AgentInput::text("invalid op").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "invalid_operation",
            "input": {"test": "value"}
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test missing input
    let input = AgentInput::text("no input").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "format"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test invalid query for transform
    let input = AgentInput::text("bad query").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": {"test": "value"},
            "query": ".nonexistent.field"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_enhanced_jq_complex_workflow() {
    let tool = JsonProcessorTool::default();

    // Complex data processing workflow
    let users_data = json!([
        {"name": "Alice", "age": 30, "active": true, "score": 85},
        {"name": "Bob", "age": 25, "active": false, "score": 92},
        {"name": "Charlie", "age": 35, "active": true, "score": 78},
        {"name": "David", "age": 28, "active": true, "score": 95},
        {"name": "Eve", "age": 22, "active": false, "score": 88}
    ]);

    // Test 1: Extract all ages and sort them
    let input = AgentInput::text("extract ages").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": users_data.clone(),
            "query": "map(.age) | sort"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    // Should contain all ages sorted: 22, 25, 28, 30, 35
    assert!(output.text.contains("22"));
    assert!(output.text.contains("25"));
    assert!(output.text.contains("28"));
    assert!(output.text.contains("30"));
    assert!(output.text.contains("35"));

    // Test 2: Get top 3 scores
    let input = AgentInput::text("top scores").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": users_data.clone(),
            "query": "map(.score) | sort | .[2:]"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    // Should contain top 3 scores: 88, 92, 95
    assert!(output.text.contains("88"));
    assert!(output.text.contains("92"));
    assert!(output.text.contains("95"));

    // Test 3: Complex nested object processing
    let nested_data = json!({
        "company": {
            "name": "TechCorp",
            "departments": {
                "engineering": {
                    "employees": ["Alice", "Bob", "Charlie"],
                    "budget": 1000000
                },
                "sales": {
                    "employees": ["David", "Eve"],
                    "budget": 500000
                }
            }
        }
    });

    let input = AgentInput::text("get departments").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": nested_data,
            "query": ".company.departments | keys"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    assert!(output.text.contains("\"engineering\""));
    assert!(output.text.contains("\"sales\""));
}

#[tokio::test]
async fn test_streaming_json_lines() {
    let tool = JsonProcessorTool::default();

    // Test streaming with transformation
    let log_lines = r#"{"timestamp": "2024-01-01T10:00:00Z", "level": "INFO", "message": "Server started"}
{"timestamp": "2024-01-01T10:00:05Z", "level": "ERROR", "message": "Connection failed"}
{"timestamp": "2024-01-01T10:00:10Z", "level": "INFO", "message": "Request processed"}
{"timestamp": "2024-01-01T10:00:15Z", "level": "WARN", "message": "High memory usage"}
{"timestamp": "2024-01-01T10:00:20Z", "level": "ERROR", "message": "Database timeout"}"#;

    // Extract all levels to verify streaming works
    let input = AgentInput::text("extract levels").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "stream",
            "input": log_lines,
            "query": ".level"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    // Should have all 5 levels
    assert!(output.text.contains("\"INFO\""));
    assert!(output.text.contains("\"ERROR\""));
    assert!(output.text.contains("\"WARN\""));

    // Count by level
    let input = AgentInput::text("count levels").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "stream",
            "input": log_lines,
            "query": ".level"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    // Should have extracted all levels
    assert!(output.text.contains("INFO"));
    assert!(output.text.contains("ERROR"));
    assert!(output.text.contains("WARN"));
}

#[tokio::test]
async fn test_advanced_array_operations() {
    let tool = JsonProcessorTool::default();

    // Test array slicing with edge cases
    let array_data = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    // Test various slice operations - check content not exact format
    let test_cases = vec![
        (
            ".[]",
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
        ), // All elements
        (".[2:10]", vec!["2", "3", "4", "5", "6", "7", "8", "9"]), // From index 2 to end
        (".[0:5]", vec!["0", "1", "2", "3", "4"]),                 // First 5 elements
        (".[2:5]", vec!["2", "3", "4"]),                           // Range
    ];

    for (query, expected_values) in test_cases {
        let input = AgentInput::text("slice array").with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "query",
                "input": array_data.clone(),
                "query": query
            }),
        );

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        for value in expected_values {
            assert!(
                output.text.contains(value),
                "Query {} failed - missing value {}",
                query,
                value
            );
        }
    }
}
