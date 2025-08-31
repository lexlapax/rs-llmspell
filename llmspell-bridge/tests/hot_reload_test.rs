//! Hot reload functionality tests
//!
//! Tests the hot reload system integration with `DiagnosticsBridge`

use llmspell_bridge::diagnostics_bridge::DiagnosticsBridge;
use llmspell_bridge::execution_context::SharedExecutionContext;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_hot_reload_enable_disable() {
    let bridge = DiagnosticsBridge::new();

    // Create a temporary test file
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test_script.lua");
    fs::write(&test_file, "print('Hello, World!')")
        .await
        .unwrap();

    // Enable hot reload for the file
    let result = bridge.enable_hot_reload(vec![test_file.clone()]).await;
    assert!(result.is_ok());

    // Disable hot reload
    bridge.disable_hot_reload();
}

#[tokio::test]
async fn test_hot_reload_validation() {
    let bridge = DiagnosticsBridge::new();

    // Test empty content validation
    let validation_result = bridge.validate_script("", "lua");
    assert!(!validation_result.valid);
    assert_eq!(validation_result.errors.len(), 1);
    assert_eq!(
        validation_result.errors[0].message,
        "Script content is empty"
    );

    // Test valid content
    let validation_result = bridge.validate_script("print('hello')", "lua");
    assert!(validation_result.valid);
    assert_eq!(validation_result.errors.len(), 0);

    // Test oversized content
    let large_content = "a".repeat(2_000_000);
    let validation_result = bridge.validate_script(&large_content, "lua");
    assert!(!validation_result.valid);
    assert_eq!(validation_result.errors.len(), 1);
    assert_eq!(
        validation_result.errors[0].message,
        "Script content too large (>1MB)"
    );
}

#[tokio::test]
async fn test_shared_execution_context_async_boundary() {
    let mut context = SharedExecutionContext::new().with_async_support();

    // Create a snapshot
    let snapshot = context.preserve_across_async_boundary();

    // Verify correlation ID is preserved
    assert!(snapshot.correlation_id.is_some());

    // Simulate some async work changing the context
    context.clear();

    // Restore from snapshot
    context.restore_from_async_boundary(snapshot);

    // Verify correlation ID was restored
    assert!(context.correlation_id.is_some());
}

#[tokio::test]
async fn test_hot_reload_file_does_not_exist() {
    let bridge = DiagnosticsBridge::new();

    let non_existent_file = PathBuf::from("/non/existent/file.lua");

    // Should succeed but not actually watch the file
    let result = bridge.enable_hot_reload(vec![non_existent_file]).await;
    assert!(result.is_ok());

    bridge.disable_hot_reload();
}
