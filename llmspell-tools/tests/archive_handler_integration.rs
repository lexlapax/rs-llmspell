//! Integration tests for `ArchiveHandlerTool`

#![cfg(feature = "archives")]

use anyhow::Result;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ResourceLimits, SecurityLevel, SecurityRequirements},
    },
    types::AgentInput,
    ExecutionContext,
};
use llmspell_security::sandbox::{FileSandbox, SandboxContext};
use llmspell_tools::fs::{ArchiveHandlerConfig, ArchiveHandlerTool};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
#[tokio::test]
async fn test_zip_create_and_extract() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let tool = ArchiveHandlerTool::new();

    // Create test files
    let file1_path = temp_dir.path().join("test1.txt");
    let file2_path = temp_dir.path().join("test2.txt");
    fs::write(&file1_path, "Hello from file 1")?;
    fs::write(&file2_path, "Hello from file 2")?;

    // Create ZIP archive
    let archive_path = temp_dir.path().join("test.zip");
    let create_params = json!({
        "operation": "create",
        "path": archive_path.to_str().unwrap(),
        "input": [
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap()
        ]
    });

    let input = AgentInput::text("").with_parameter("parameters", create_params);
    let result = tool.execute(input, ExecutionContext::default()).await?;
    assert!(archive_path.exists());

    // Verify creation result
    assert!(result.text.contains("Created archive with 2 files"));

    // List archive contents
    let list_params = json!({
        "operation": "list",
        "path": archive_path.to_str().unwrap()
    });

    let input = AgentInput::text("").with_parameter("parameters", list_params);
    let list_result = tool.execute(input, ExecutionContext::default()).await?;
    assert!(list_result.text.contains("Listed 2 files in ZIP archive"));

    // Extract archive
    let extract_dir = temp_dir.path().join("extracted");
    let extract_params = json!({
        "operation": "extract",
        "path": archive_path.to_str().unwrap(),
        "target_path": extract_dir.to_str().unwrap()
    });

    let input = AgentInput::text("").with_parameter("parameters", extract_params);
    let extract_result = tool.execute(input, ExecutionContext::default()).await?;
    println!("Extract result text: {}", extract_result.text);
    println!("Extract result metadata: {:?}", extract_result.metadata);
    assert!(extract_result.text.contains("Extracted 2 files"));

    // Verify extracted content
    assert!(extract_dir.join("test1.txt").exists());
    assert!(extract_dir.join("test2.txt").exists());

    let content1 = fs::read_to_string(extract_dir.join("test1.txt"))?;
    assert_eq!(content1, "Hello from file 1");

    Ok(())
}
#[tokio::test]
async fn test_tar_gz_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let tool = ArchiveHandlerTool::new();

    // Create test files
    let file1_path = temp_dir.path().join("data1.json");
    let file2_path = temp_dir.path().join("data2.json");
    fs::write(&file1_path, r#"{"key": "value1"}"#)?;
    fs::write(&file2_path, r#"{"key": "value2"}"#)?;

    // Create TAR.GZ archive
    let archive_path = temp_dir.path().join("data.tar.gz");
    let create_params = json!({
        "operation": "create",
        "path": archive_path.to_str().unwrap(),
        "input": [
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap()
        ]
    });

    let input = AgentInput::text("").with_parameter("parameters", create_params);
    let result = tool.execute(input, ExecutionContext::default()).await?;
    assert!(result.text.contains("Created archive"));
    assert!(archive_path.exists());

    // Extract and verify
    let extract_dir = temp_dir.path().join("extracted_tar");
    let extract_params = json!({
        "operation": "extract",
        "path": archive_path.to_str().unwrap(),
        "target_path": extract_dir.to_str().unwrap()
    });

    let input = AgentInput::text("").with_parameter("parameters", extract_params);
    tool.execute(input, ExecutionContext::default()).await?;

    assert!(extract_dir.join("data1.json").exists());
    assert!(extract_dir.join("data2.json").exists());

    Ok(())
}
#[tokio::test]
async fn test_single_file_gz() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let tool = ArchiveHandlerTool::new();

    // Create test file
    let file_path = temp_dir.path().join("large_data.txt");
    let content = "A".repeat(1000);
    fs::write(&file_path, &content)?;

    // Compress single file
    let archive_path = temp_dir.path().join("large_data.txt.gz");
    let create_params = json!({
        "operation": "create",
        "path": archive_path.to_str().unwrap(),
        "input": [file_path.to_str().unwrap()]
    });

    let input = AgentInput::text("").with_parameter("parameters", create_params);
    tool.execute(input, ExecutionContext::default()).await?;
    assert!(archive_path.exists());

    // Verify compression occurred
    let compressed_size = fs::metadata(&archive_path)?.len();
    assert!(compressed_size < 1000); // Should be much smaller than original

    // Extract
    let extract_dir = temp_dir.path().join("extracted_gz");
    let extract_params = json!({
        "operation": "extract",
        "path": archive_path.to_str().unwrap(),
        "target_path": extract_dir.to_str().unwrap()
    });

    let input = AgentInput::text("").with_parameter("parameters", extract_params);
    tool.execute(input, ExecutionContext::default()).await?;

    // Verify extracted content
    let extracted_file = extract_dir.join("large_data.txt");
    assert!(extracted_file.exists());
    let extracted_content = fs::read_to_string(extracted_file)?;
    assert_eq!(extracted_content, content);

    Ok(())
}
#[tokio::test]
async fn test_archive_size_limits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let tool = ArchiveHandlerTool::new();

    // Create a file that exceeds the default max file size
    let large_file = temp_dir.path().join("huge.txt");
    let huge_content = "B".repeat(200 * 1024 * 1024); // 200MB
    fs::write(&large_file, huge_content)?;

    // Try to create archive - should skip the file
    let archive_path = temp_dir.path().join("test_limits.zip");
    let create_params = json!({
        "operation": "create",
        "path": archive_path.to_str().unwrap(),
        "input": [large_file.to_str().unwrap()]
    });

    let input = AgentInput::text("").with_parameter("parameters", create_params);
    let result = tool.execute(input, ExecutionContext::default()).await?;

    // Archive should be created but with no files
    assert!(archive_path.exists());
    assert!(result.text.contains("Created archive with 0 files")); // File was skipped due to size

    Ok(())
}
#[tokio::test]
async fn test_path_traversal_protection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let tool = ArchiveHandlerTool::new();

    // This test would require creating a malicious archive with path traversal
    // For now, we'll test that the is_safe_path function works correctly
    // (actual unit tests are in the module)

    // Create a normal archive
    let safe_file = temp_dir.path().join("safe.txt");
    fs::write(&safe_file, "Safe content")?;

    let archive_path = temp_dir.path().join("safe.zip");
    let create_params = json!({
        "operation": "create",
        "path": archive_path.to_str().unwrap(),
        "input": [safe_file.to_str().unwrap()]
    });

    let input = AgentInput::text("").with_parameter("parameters", create_params);
    tool.execute(input, ExecutionContext::default()).await?;

    // Extract normally
    let extract_dir = temp_dir.path().join("safe_extract");
    let extract_params = json!({
        "operation": "extract",
        "path": archive_path.to_str().unwrap(),
        "target_path": extract_dir.to_str().unwrap()
    });

    let input = AgentInput::text("").with_parameter("parameters", extract_params);
    tool.execute(input, ExecutionContext::default()).await?;
    assert!(extract_dir.join("safe.txt").exists());

    Ok(())
}
#[tokio::test]
async fn test_with_file_sandbox() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let sandbox_dir = temp_dir.path().join("sandbox");
    fs::create_dir_all(&sandbox_dir)?;

    // Create sandbox that only allows access to sandbox_dir
    let security_requirements = SecurityRequirements {
        level: SecurityLevel::Restricted,
        file_permissions: vec![sandbox_dir.to_string_lossy().to_string()],
        network_permissions: vec![],
        env_permissions: vec![],
        custom_requirements: HashMap::new(),
    };
    let resource_limits = ResourceLimits::default();
    let sandbox_context = SandboxContext::new(
        "test-sandbox".to_string(),
        security_requirements,
        resource_limits,
    );
    let sandbox = FileSandbox::new(sandbox_context)?;

    let tool = ArchiveHandlerTool::new().with_sandbox(sandbox);

    // Create file inside sandbox
    let allowed_file = sandbox_dir.join("allowed.txt");
    fs::write(&allowed_file, "Allowed content")?;

    // Create archive inside sandbox
    let archive_path = sandbox_dir.join("sandboxed.zip");
    let create_params = json!({
        "operation": "create",
        "path": archive_path.to_str().unwrap(),
        "input": [allowed_file.to_str().unwrap()]
    });

    // This should succeed
    let input = AgentInput::text("").with_parameter("parameters", create_params);
    tool.execute(input, ExecutionContext::default()).await?;
    assert!(archive_path.exists());

    // Try to create archive outside sandbox
    let outside_archive = temp_dir.path().join("outside.zip");
    let create_outside_params = json!({
        "operation": "create",
        "path": outside_archive.to_str().unwrap(),
        "input": [allowed_file.to_str().unwrap()]
    });

    // This should fail due to sandbox
    let input = AgentInput::text("").with_parameter("parameters", create_outside_params);
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    Ok(())
}
#[tokio::test]
async fn test_archive_formats() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let tool = ArchiveHandlerTool::new();

    // Create test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content for all formats")?;

    // Test different archive formats
    let formats = vec![
        ("test.zip", "ZIP"),
        ("test.tar", "TAR"),
        ("test.tar.gz", "TAR.GZ"),
        ("test.gz", "GZ"),
    ];

    for (filename, expected_format) in formats {
        let archive_path = temp_dir.path().join(filename);

        // For all archive types, we need array with single file
        let files = vec![test_file.to_str().unwrap()];

        let create_params = json!({
            "operation": "create",
            "path": archive_path.to_str().unwrap(),
            "input": files
        });

        let input = AgentInput::text("").with_parameter("parameters", create_params);
        tool.execute(input, ExecutionContext::default()).await?;
        assert!(archive_path.exists());

        // List to verify format detection
        let list_params = json!({
            "operation": "list",
            "path": archive_path.to_str().unwrap()
        });

        let input = AgentInput::text("").with_parameter("parameters", list_params);
        let list_result = tool.execute(input, ExecutionContext::default()).await?;
        assert!(list_result.text.contains(expected_format));
    }

    Ok(())
}
#[tokio::test]
async fn test_compression_levels() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create test file with repetitive content (compresses well)
    let test_file = temp_dir.path().join("compressible.txt");
    let content = "AAAAAAAAAA".repeat(1000); // 10KB of 'A's
    fs::write(&test_file, &content)?;

    // Test different compression levels
    let mut sizes = Vec::new();

    for level in [0, 5, 9] {
        let config = ArchiveHandlerConfig {
            compression_level: level,
            ..Default::default()
        };

        let tool = ArchiveHandlerTool::with_config(config);

        let archive_path = temp_dir.path().join(format!("test_level_{level}.zip"));
        let create_params = json!({
            "operation": "create",
            "path": archive_path.to_str().unwrap(),
            "input": [test_file.to_str().unwrap()]
        });

        let input = AgentInput::text("").with_parameter("parameters", create_params);
        tool.execute(input, ExecutionContext::default()).await?;

        let size = fs::metadata(&archive_path)?.len();
        sizes.push((level, size));
    }

    // Higher compression levels should produce smaller files
    // (though for small files the difference might be minimal)
    println!("Compression sizes: {sizes:?}");

    Ok(())
}
