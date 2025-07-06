//! Integration tests for JsonProcessorTool

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::{AgentInput, ExecutionContext},
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
            "operation": "transform",
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
            "operation": "transform",
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
    let metadata = &output.metadata;
    let result = metadata.extra.get("result").unwrap();
    let validation = result.get("validation").unwrap();
    assert_eq!(validation.get("is_valid").unwrap(), true);

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
    let metadata = &output.metadata;
    let result = metadata.extra.get("result").unwrap();
    let validation = result.get("validation").unwrap();
    assert_eq!(validation.get("is_valid").unwrap(), false);

    let errors = validation.get("errors").unwrap().as_array().unwrap();
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
            "operation": "filter",
            "input": products,
            "query": ".category == \"electronics\""
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
            "operation": "merge",
            "input": configs
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let merged: serde_json::Value = serde_json::from_str(&output.text).unwrap();

    // Check merged structure - note that merge is shallow, so later objects overwrite earlier ones
    // The last "database" object only has username/password, so it overwrites the entire database object
    assert_eq!(merged["database"]["username"], "admin");
    assert_eq!(merged["database"]["password"], "secret");
    assert!(merged["database"]["host"].is_null()); // host was overwritten

    // The last "cache" object only has ttl, so it overwrites the entire cache object
    assert_eq!(merged["cache"]["ttl"], 3600);
    assert!(merged["cache"]["enabled"].is_null()); // enabled was overwritten

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
            "operation": "transform",
            "input": {"test": "value"},
            "query": ".nonexistent.field"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}
