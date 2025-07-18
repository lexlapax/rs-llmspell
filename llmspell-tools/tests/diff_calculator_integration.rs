// ABOUTME: Integration tests for the diff calculator tool
// ABOUTME: Tests text diff, JSON diff, file comparison, and various output formats

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::util::DiffCalculatorTool;
use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;

/// Helper to extract result from response wrapper
fn extract_result(response_text: &str) -> Value {
    let output: Value = serde_json::from_str(response_text).unwrap();
    assert!(output["success"].as_bool().unwrap_or(false));
    output["result"].clone()
}

#[tokio::test]
async fn test_text_diff_formats() {
    let tool = DiffCalculatorTool::new();

    let old_text = "The quick brown fox\njumps over the lazy dog.\nThe end.";
    let new_text = "The quick brown fox\njumps over the lazy cat.\nA new line.\nThe end.";

    // Test unified format
    let input = AgentInput::text("diff unified").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": old_text,
            "new_text": new_text,
            "format": "unified"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["type"], "text");
    assert_eq!(output["format"], "unified");
    let diff = output["diff"].as_str().unwrap();
    assert!(diff.contains("@@"));
    assert!(diff.contains("-jumps over the lazy dog."));
    assert!(diff.contains("+jumps over the lazy cat."));
    assert!(diff.contains("+A new line."));

    // Test context format
    let input = AgentInput::text("diff context").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": old_text,
            "new_text": new_text,
            "format": "context"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["format"], "context");
    let diff = output["diff"].as_str().unwrap();
    assert!(diff.contains("***"));
    assert!(diff.contains("---"));

    // Test inline format
    let input = AgentInput::text("diff inline").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": old_text,
            "new_text": new_text,
            "format": "inline"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["format"], "inline");
    let diff = output["diff"].as_str().unwrap();
    // The inline format shows line-by-line changes with +/- prefixes
    assert!(diff.contains("-jumps over the lazy dog."));
    assert!(diff.contains("+jumps over the lazy cat."));
    assert!(diff.contains("+A new line."));

    // Test simple format
    let input = AgentInput::text("diff simple").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": old_text,
            "new_text": new_text,
            "format": "simple"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["format"], "simple");
    let diff = output["diff"].as_str().unwrap();
    assert!(diff.contains("Total changes:"));
    // The simple format shows "Replaced" for modified lines
    assert!(diff.contains("Replaced at lines"));
}

#[tokio::test]
async fn test_json_diff_simple() {
    let tool = DiffCalculatorTool::new();

    let old_json = json!({
        "name": "Alice",
        "age": 25,
        "email": "alice@example.com"
    });

    let new_json = json!({
        "name": "Alice",
        "age": 26,
        "phone": "+1234567890"
    });

    let input = AgentInput::text("diff json").with_parameter(
        "parameters",
        json!({
            "type": "json",
            "old_json": old_json,
            "new_json": new_json
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["type"], "json");

    let diff = &output["diff"];
    assert_eq!(diff["removed"]["email"], "alice@example.com");
    assert_eq!(diff["added"]["phone"], "+1234567890");
    assert_eq!(diff["modified"]["age"]["old"], 25);
    assert_eq!(diff["modified"]["age"]["new"], 26);

    let summary = &output["summary"];
    assert_eq!(summary["added"], 1);
    assert_eq!(summary["removed"], 1);
    assert_eq!(summary["modified"], 1);
    assert_eq!(summary["unchanged"], 1); // "name" is unchanged
}

#[tokio::test]
async fn test_json_diff_nested() {
    let tool = DiffCalculatorTool::new();

    let old_json = json!({
        "user": {
            "name": "Bob",
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        },
        "data": [1, 2, 3]
    });

    let new_json = json!({
        "user": {
            "name": "Bob",
            "settings": {
                "theme": "light",
                "notifications": true,
                "sound": false
            }
        },
        "data": [1, 2, 3, 4]
    });

    let input = AgentInput::text("diff nested json").with_parameter(
        "parameters",
        json!({
            "type": "json",
            "old_json": old_json,
            "new_json": new_json
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    let diff = &output["diff"];
    assert_eq!(diff["added"]["user.settings.sound"], false);
    assert_eq!(diff["modified"]["user.settings.theme"]["old"], "dark");
    assert_eq!(diff["modified"]["user.settings.theme"]["new"], "light");
    // Arrays are handled as a single modified entity with "data" key (not "data[]")
    assert!(diff["modified"]["data"]["old"].is_array());
    assert!(diff["modified"]["data"]["new"].is_array());
    assert_eq!(diff["modified"]["data"]["old"], json!([1, 2, 3]));
    assert_eq!(diff["modified"]["data"]["new"], json!([1, 2, 3, 4]));
}

#[tokio::test]
async fn test_file_diff() {
    let tool = DiffCalculatorTool::new();
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    let old_content = "Line 1\nLine 2\nLine 3\n";
    let new_content = "Line 1\nLine 2 modified\nLine 3\nLine 4\n";

    let old_file = temp_dir.path().join("old.txt");
    let new_file = temp_dir.path().join("new.txt");

    fs::write(&old_file, old_content).unwrap();
    fs::write(&new_file, new_content).unwrap();

    // Test text file diff
    let input = AgentInput::text("diff files").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_file": old_file.to_str().unwrap(),
            "new_file": new_file.to_str().unwrap(),
            "format": "unified"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["type"], "text");
    let diff = output["diff"].as_str().unwrap();
    assert!(diff.contains("-Line 2"));
    assert!(diff.contains("+Line 2 modified"));
    assert!(diff.contains("+Line 4"));
}

#[tokio::test]
async fn test_json_file_diff() {
    let tool = DiffCalculatorTool::new();
    let temp_dir = TempDir::new().unwrap();

    // Create JSON files
    let old_json = json!({
        "version": "1.0.0",
        "features": ["feature1", "feature2"]
    });
    let new_json = json!({
        "version": "1.1.0",
        "features": ["feature1", "feature2", "feature3"],
        "author": "Test"
    });

    let old_file = temp_dir.path().join("old.json");
    let new_file = temp_dir.path().join("new.json");

    fs::write(&old_file, serde_json::to_string_pretty(&old_json).unwrap()).unwrap();
    fs::write(&new_file, serde_json::to_string_pretty(&new_json).unwrap()).unwrap();

    // Test JSON file diff
    let input = AgentInput::text("diff json files").with_parameter(
        "parameters",
        json!({
            "type": "json",
            "old_file": old_file.to_str().unwrap(),
            "new_file": new_file.to_str().unwrap()
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["type"], "json");
    let diff = &output["diff"];
    assert_eq!(diff["added"]["author"], "Test");
    assert_eq!(diff["modified"]["version"]["old"], "1.0.0");
    assert_eq!(diff["modified"]["version"]["new"], "1.1.0");
}

#[tokio::test]
async fn test_empty_diff() {
    let tool = DiffCalculatorTool::new();

    let same_text = "This is the same text";

    let input = AgentInput::text("diff same").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": same_text,
            "new_text": same_text,
            "format": "simple"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    let diff = output["diff"].as_str().unwrap();
    assert!(diff.contains("Total changes: 0"));
}

#[tokio::test]
async fn test_large_text_diff() {
    let tool = DiffCalculatorTool::new();

    // Generate large texts
    let mut old_lines = vec![];
    let mut new_lines = vec![];

    for i in 0..1000 {
        old_lines.push(format!("Line {}", i));
        if i % 10 == 0 {
            new_lines.push(format!("Modified line {}", i));
        } else {
            new_lines.push(format!("Line {}", i));
        }
    }

    let old_text = old_lines.join("\n");
    let new_text = new_lines.join("\n");

    let input = AgentInput::text("diff large").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": old_text,
            "new_text": new_text,
            "format": "simple"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);

    assert_eq!(output["stats"]["old_lines"], 1000);
    assert_eq!(output["stats"]["new_lines"], 1000);

    let diff = output["diff"].as_str().unwrap();
    assert!(diff.contains("Total changes: 100")); // 100 modifications
}

#[tokio::test]
async fn test_error_handling() {
    let tool = DiffCalculatorTool::new();

    // Test missing inputs
    let input = AgentInput::text("diff missing").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "format": "unified"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test invalid format
    let input = AgentInput::text("diff invalid format").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_text": "test",
            "new_text": "test2",
            "format": "invalid_format"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test invalid diff type
    let input = AgentInput::text("diff invalid type").with_parameter(
        "parameters",
        json!({
            "type": "xml",
            "old_text": "test",
            "new_text": "test2"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test non-existent files
    let input = AgentInput::text("diff nonexistent").with_parameter(
        "parameters",
        json!({
            "type": "text",
            "old_file": "/non/existent/old.txt",
            "new_file": "/non/existent/new.txt"
        }),
    );

    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_metadata() {
    use llmspell_core::traits::tool::{SecurityLevel, Tool, ToolCategory};

    let tool = DiffCalculatorTool::new();

    assert_eq!(tool.metadata().name, "diff-calculator");
    assert!(tool.metadata().description.contains("differences"));
    assert_eq!(tool.category(), ToolCategory::Utility);
    assert_eq!(tool.security_level(), SecurityLevel::Safe);

    // Verify schema
    let schema = tool.schema();
    assert_eq!(schema.name, "diff_calculator");
    assert!(schema.description.contains("differences"));
}
