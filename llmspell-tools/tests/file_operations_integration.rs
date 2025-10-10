//! Integration tests for `FileOperationsTool`

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_testing::tool_helpers::create_test_sandbox_with_temp_dir;
use llmspell_tools::{fs::FileOperationsConfig, FileOperationsTool};
use serde_json::json;
use std::path::PathBuf;

fn create_context() -> ExecutionContext {
    let mut ctx = ExecutionContext::new();
    ctx.session_id = Some("test-session".to_string());
    ctx
}

/// Create a unique test directory under /tmp
fn create_test_dir() -> PathBuf {
    let dir_name = format!("/tmp/llmspell_test_{}", uuid::Uuid::new_v4());
    std::fs::create_dir_all(&dir_name).unwrap();
    PathBuf::from(dir_name)
}

/// Clean up test directory
fn cleanup_test_dir(path: &PathBuf) {
    let _ = std::fs::remove_dir_all(path);
}
#[tokio::test]
async fn test_file_operations_basic() {
    let test_dir = create_test_dir();
    let test_file = test_dir.join("test.txt");
    let test_content = "Hello, FileOperations!";

    // Create tool
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);

    // Test write operation
    let write_input = AgentInput::text("write").with_parameter(
        "parameters",
        json!({
            "operation": "write",
            "path": test_file.to_str().unwrap(),
            "input": test_content
        }),
    );

    let context = create_context();
    let result = tool.execute(write_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Wrote") && result.text.contains("bytes"));

    // Test read operation
    let read_input = AgentInput::text("read").with_parameter(
        "parameters",
        json!({
            "operation": "read",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(read_input, context.clone()).await.unwrap();
    assert!(result.text.contains(test_content));

    // Test exists operation
    let exists_input = AgentInput::text("exists").with_parameter(
        "parameters",
        json!({
            "operation": "exists",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(exists_input, context.clone()).await.unwrap();
    assert!(result.text.contains("exists"));

    // Test metadata operation
    let metadata_input = AgentInput::text("metadata").with_parameter(
        "parameters",
        json!({
            "operation": "metadata",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(metadata_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Retrieved metadata"));

    // Test append operation
    let append_input = AgentInput::text("append").with_parameter(
        "parameters",
        json!({
            "operation": "append",
            "path": test_file.to_str().unwrap(),
            "input": "\nAppended content"
        }),
    );

    let result = tool.execute(append_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Appended") && result.text.contains("bytes"));

    // Read again to verify append
    let read_input = AgentInput::text("read").with_parameter(
        "parameters",
        json!({
            "operation": "read",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(read_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Hello, FileOperations!"));
    assert!(result.text.contains("Appended content"));

    // Test delete operation
    let delete_input = AgentInput::text("delete").with_parameter(
        "parameters",
        json!({
            "operation": "delete",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(delete_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Deleted file"));

    // Verify file is deleted
    let exists_input = AgentInput::text("exists").with_parameter(
        "parameters",
        json!({
            "operation": "exists",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(exists_input, context).await.unwrap();
    assert!(result.text.contains("does not exist"));

    cleanup_test_dir(&test_dir);
}
#[tokio::test]
async fn test_directory_operations() {
    let test_base = create_test_dir();
    let _test_dir = test_base.join("test_dir");
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);
    let context = create_context();

    // Create directory
    let create_dir_input = AgentInput::text("create_dir").with_parameter(
        "parameters",
        json!({
            "operation": "create_dir",
            "path": test_dir.to_str().unwrap()
        }),
    );

    let result = tool
        .execute(create_dir_input, context.clone())
        .await
        .unwrap();
    assert!(result.text.contains("Created directory"));

    // Create some files in the directory
    for i in 1..=3 {
        let file_path = test_dir.join(format!("file{i}.txt"));
        let write_input = AgentInput::text("write").with_parameter(
            "parameters",
            json!({
                "operation": "write",
                "path": file_path.to_str().unwrap(),
                "input": format!("Content {}", i)
            }),
        );
        tool.execute(write_input, context.clone()).await.unwrap();
    }

    // List directory
    let list_input = AgentInput::text("list_dir").with_parameter(
        "parameters",
        json!({
            "operation": "list_dir",
            "path": test_dir.to_str().unwrap()
        }),
    );

    let result = tool.execute(list_input, context).await.unwrap();
    assert!(result.text.contains("Found 3 entries"));

    cleanup_test_dir(&test_base);
}
#[tokio::test]
async fn test_copy_move_operations() {
    let test_dir = create_test_dir();
    let source = test_dir.join("source.txt");
    let copy_dest = test_dir.join("copy.txt");
    let move_dest = test_dir.join("moved.txt");

    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);
    let context = create_context();

    // Create source file
    let write_input = AgentInput::text("write").with_parameter(
        "parameters",
        json!({
            "operation": "write",
            "path": source.to_str().unwrap(),
            "input": "Original content"
        }),
    );
    tool.execute(write_input, context.clone()).await.unwrap();

    // Copy file
    let copy_input = AgentInput::text("copy").with_parameter(
        "parameters",
        json!({
            "operation": "copy",
            "source_path": source.to_str().unwrap(),
            "target_path": copy_dest.to_str().unwrap()
        }),
    );

    let result = tool.execute(copy_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Copied"));

    // Verify both files exist
    assert!(source.exists());
    assert!(copy_dest.exists());

    // Move file
    let move_input = AgentInput::text("move").with_parameter(
        "parameters",
        json!({
            "operation": "move",
            "source_path": source.to_str().unwrap(),
            "target_path": move_dest.to_str().unwrap()
        }),
    );

    let result = tool.execute(move_input, context.clone()).await.unwrap();
    assert!(result.text.contains("Moved"));

    // Verify move
    assert!(!source.exists());
    assert!(move_dest.exists());

    // Read moved file to verify content
    let read_input = AgentInput::text("read").with_parameter(
        "parameters",
        json!({
            "operation": "read",
            "path": move_dest.to_str().unwrap()
        }),
    );

    let result = tool.execute(read_input, context).await.unwrap();
    assert!(result.text.contains("Original content"));

    cleanup_test_dir(&test_dir);
}
#[tokio::test]
async fn test_security_sandbox() {
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);
    let context = create_context();

    // Attempt to access file outside sandbox (should fail)
    let read_input = AgentInput::text("read").with_parameter(
        "parameters",
        json!({
            "operation": "read",
            "path": "/etc/passwd"
        }),
    );

    let result = tool.execute(read_input, context.clone()).await;
    // The sandbox should prevent access to system files
    assert!(result.is_err() || result.unwrap().text.contains("error"));

    // Test path traversal attempt from /tmp
    let traversal_input = AgentInput::text("read").with_parameter(
        "parameters",
        json!({
            "operation": "read",
            "path": "/tmp/../etc/passwd"
        }),
    );

    let result = tool.execute(traversal_input, context).await;
    assert!(result.is_err() || result.unwrap().text.contains("error"));
}
#[tokio::test]
async fn test_tool_metadata() {
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);

    // Test tool category
    assert_eq!(
        tool.category(),
        llmspell_core::traits::tool::ToolCategory::Filesystem
    );

    // Test security level
    assert_eq!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Privileged
    );

    // Test schema
    let schema = tool.schema();
    assert_eq!(schema.name, "file-operations");
    assert!(!schema.parameters.is_empty());

    // Verify required parameters
    let operation_param = schema
        .parameters
        .iter()
        .find(|p| p.name == "operation")
        .expect("operation parameter should exist");
    assert!(operation_param.required);
}
#[tokio::test]
async fn test_error_handling() {
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);
    let context = create_context();

    // Test missing operation parameter
    let invalid_input = AgentInput::text("invalid").with_parameter(
        "parameters",
        json!({
            "path": "/tmp/test.txt"
        }),
    );

    let result = tool.execute(invalid_input, context.clone()).await;
    assert!(result.is_err());

    // Test invalid operation
    let invalid_op_input = AgentInput::text("invalid").with_parameter(
        "parameters",
        json!({
            "operation": "invalid_operation",
            "path": "/tmp/test.txt"
        }),
    );

    let result = tool.execute(invalid_op_input, context.clone()).await;
    assert!(result.is_err());

    // Test missing required parameters for write
    let missing_content_input = AgentInput::text("write").with_parameter(
        "parameters",
        json!({
            "operation": "write",
            "path": "/tmp/test.txt"
        }),
    );

    let result = tool.execute(missing_content_input, context).await;
    assert!(result.is_err());
}
#[tokio::test]
async fn test_recursive_directory_creation() {
    let test_dir = create_test_dir();
    let nested_dir = test_dir.join("a/b/c/d");

    let config = FileOperationsConfig {
        allow_recursive: true,
        ..Default::default()
    };
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(config, sandbox);
    let context = create_context();

    // Create nested directory with recursive flag
    let create_input = AgentInput::text("create_dir").with_parameter(
        "parameters",
        json!({
            "operation": "create_dir",
            "path": nested_dir.to_str().unwrap(),
            "recursive": true
        }),
    );

    let result = tool.execute(create_input, context).await.unwrap();
    assert!(result.text.contains("Created directory"));
    assert!(nested_dir.exists());

    cleanup_test_dir(&test_dir);
}
#[tokio::test]
async fn test_file_size_limits() {
    let test_dir = create_test_dir();
    let test_file = test_dir.join("large.txt");

    // Create a custom config with small file size limit
    let config = FileOperationsConfig {
        max_file_size: 100, // 100 bytes
        ..Default::default()
    };
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(config, sandbox);
    let context = create_context();

    // Try to write content larger than limit
    let large_content = "x".repeat(200); // 200 bytes
    let write_input = AgentInput::text("write").with_parameter(
        "parameters",
        json!({
            "operation": "write",
            "path": test_file.to_str().unwrap(),
            "input": large_content
        }),
    );

    let result = tool.execute(write_input, context).await.unwrap();
    // The tool returns errors as successful responses with error information
    assert!(
        result.text.contains("validation_error")
            || result.text.contains("exceeds maximum allowed size")
    );

    cleanup_test_dir(&test_dir);
}
#[tokio::test]
async fn test_atomic_writes() {
    let test_dir = create_test_dir();
    let test_file = test_dir.join("atomic.txt");

    // Create tool with atomic writes enabled (default)
    let test_dir = create_test_dir();
    let sandbox = create_test_sandbox_with_temp_dir("file_ops_test", &test_dir.to_string_lossy());
    let tool = FileOperationsTool::new(FileOperationsConfig::default(), sandbox);
    let context = create_context();

    // Write initial content
    let write_input = AgentInput::text("write").with_parameter(
        "parameters",
        json!({
            "operation": "write",
            "path": test_file.to_str().unwrap(),
            "input": "Initial content"
        }),
    );

    tool.execute(write_input, context.clone()).await.unwrap();

    // Overwrite with new content
    let write_input = AgentInput::text("write").with_parameter(
        "parameters",
        json!({
            "operation": "write",
            "path": test_file.to_str().unwrap(),
            "input": "Updated content"
        }),
    );

    tool.execute(write_input, context.clone()).await.unwrap();

    // Read and verify
    let read_input = AgentInput::text("read").with_parameter(
        "parameters",
        json!({
            "operation": "read",
            "path": test_file.to_str().unwrap()
        }),
    );

    let result = tool.execute(read_input, context).await.unwrap();
    assert!(result.text.contains("Updated content"));
    assert!(!result.text.contains("Initial content"));

    cleanup_test_dir(&test_dir);
}
