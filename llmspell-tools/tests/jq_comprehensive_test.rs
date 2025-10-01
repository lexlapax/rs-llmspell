//! Comprehensive tests for jq syntax support

#![cfg(feature = "json-query")]

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::JsonProcessorTool;
use serde_json::json;
// Helper function to create test data
fn create_test_data() -> serde_json::Value {
    json!({
        "users": [
            {"name": "Alice", "age": 30, "skills": ["rust", "python"]},
            {"name": "Bob", "age": 25, "skills": ["javascript", "go"]},
            {"name": "Charlie", "age": 35, "skills": ["rust", "go", "java"]}
        ],
        "projects": [
            {"name": "Project A", "lead": "Alice", "status": "active"},
            {"name": "Project B", "lead": "Bob", "status": "completed"},
            {"name": "Project C", "lead": "Charlie", "status": "active"}
        ]
    })
}

#[tokio::test]
async fn test_jq_pipes_and_sort() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test pipes").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".users | map(.name) | sort"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<String> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result, vec!["Alice", "Bob", "Charlie"]);
}

#[tokio::test]
async fn test_jq_map_operations() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test map").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".projects | map({project: .name, manager: .lead})"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<serde_json::Value> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result[0]["project"], "Project A");
    assert_eq!(result[0]["manager"], "Alice");
}

#[tokio::test]
async fn test_jq_select_filtering() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test select").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".users[] | select(.age > 26) | .name"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<String> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result, vec!["Alice", "Charlie"]);
}

#[tokio::test]
async fn test_jq_reduce_operations() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test reduce").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".users | map(.age) | add / length"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: f64 = serde_json::from_str(&output.text).unwrap();
    assert!((result - 30.0).abs() < f64::EPSILON); // Average age
}

#[tokio::test]
async fn test_jq_group_by() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test group_by").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".projects | group_by(.status) | map({status: .[0].status, count: length})"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<serde_json::Value> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result.len(), 2); // active and completed
}

#[tokio::test]
async fn test_jq_complex_nested_operations() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test complex").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".users | map(select(.skills | contains([\"rust\"]))) | map({name, skill_count: (.skills | length)})"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<serde_json::Value> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result.len(), 2); // Alice and Charlie have rust
    assert!(result
        .iter()
        .any(|u| u["name"] == "Alice" && u["skill_count"] == 2));
    assert!(result
        .iter()
        .any(|u| u["name"] == "Charlie" && u["skill_count"] == 3));
}

#[tokio::test]
async fn test_jq_object_construction() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test object construction").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": "{total_users: (.users | length), avg_age: (.users | map(.age) | add / length), active_projects: (.projects | map(select(.status == \"active\")) | length)}"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result["total_users"], 3);
    assert_eq!(result["avg_age"], 30.0);
    assert_eq!(result["active_projects"], 2);
}

#[tokio::test]
async fn test_jq_array_slicing_and_indexing() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test array ops").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".users[1:] | map(.name)"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<String> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result, vec!["Bob", "Charlie"]);
}

#[tokio::test]
async fn test_jq_string_operations() {
    let tool = JsonProcessorTool::default();

    let input = AgentInput::text("test string ops").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": json!({"text": "hello world"}),
            "query": ".text | ascii_upcase"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: String = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result, "HELLO WORLD");
}

#[tokio::test]
async fn test_jq_type_operations() {
    let tool = JsonProcessorTool::default();
    let data = create_test_data();

    let input = AgentInput::text("test type ops").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "query",
            "input": data,
            "query": ".users | map({name, is_adult: (.age >= 18)})"
        }),
    );
    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<serde_json::Value> = serde_json::from_str(&output.text).unwrap();
    assert!(result.iter().all(|u| u["is_adult"] == true));
}
#[tokio::test]
async fn test_streaming_json_lines() {
    let tool = JsonProcessorTool::default();

    let json_lines = r#"{"id": 1, "name": "Item 1", "value": 100}
{"id": 2, "name": "Item 2", "value": 200}
{"id": 3, "name": "Item 3", "value": 150}"#;

    let input = AgentInput::text("test streaming").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "stream",
            "input": json_lines,
            "query": "select(.value > 120)"
        }),
    );

    let output = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result: Vec<serde_json::Value> = serde_json::from_str(&output.text).unwrap();
    assert_eq!(result.len(), 2);
    assert!(result
        .iter()
        .all(|item| item["value"].as_u64().unwrap() > 120));
}
