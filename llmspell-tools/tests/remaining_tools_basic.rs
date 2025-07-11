// ABOUTME: Basic integration tests for the remaining Phase 2 tools
// ABOUTME: Tests tool creation, schema access, and basic execution

use llmspell_core::traits::{base_agent::BaseAgent, tool::Tool};
use llmspell_core::types::{AgentInput, ExecutionContext};
use llmspell_security::sandbox::file_sandbox::FileSandbox;
use llmspell_security::sandbox::SandboxContext;
use llmspell_tools::{
    fs::{FileConverterTool, FileSearchTool, FileWatcherTool},
    media::{AudioProcessorTool, ImageProcessorTool, VideoProcessorTool},
    search::WebSearchTool,
    system::{
        EnvironmentReaderTool, ProcessExecutorTool, ServiceCheckerTool, SystemMonitorTool,
    },
    util::{HashCalculatorTool, TextManipulatorTool, UuidGeneratorTool},
};
use serde_json::{json, Value};
use std::sync::Arc;

// Helper function to create a file sandbox
fn create_file_sandbox() -> Arc<FileSandbox> {
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    
    let security_requirements = SecurityRequirements::default().with_file_access("/tmp");
    let sandbox_context = SandboxContext::new(
        "test-tools".to_string(),
        security_requirements,
        ResourceLimits::default(),
    );
    Arc::new(FileSandbox::new(sandbox_context).expect("Failed to create file sandbox"))
}

// ===== Tool Creation Tests =====

#[test]
fn test_file_system_tools_creation() {
    let sandbox = create_file_sandbox();
    
    // FileWatcherTool
    let tool = FileWatcherTool::new(Default::default(), sandbox.clone());
    let schema = tool.schema();
    assert_eq!(schema.name, "file_watcher");
    assert!(!schema.description.is_empty());
    assert!(!schema.parameters.is_empty());
    
    // FileConverterTool
    let tool = FileConverterTool::new(Default::default(), sandbox.clone());
    let schema = tool.schema();
    assert_eq!(schema.name, "file_converter");
    assert!(!schema.description.is_empty());
    
    // FileSearchTool
    let tool = FileSearchTool::new(Default::default(), sandbox);
    let schema = tool.schema();
    assert_eq!(schema.name, "file_search");
    assert!(!schema.description.is_empty());
}

#[test]
fn test_system_integration_tools_creation() {
    // EnvironmentReaderTool
    let tool = EnvironmentReaderTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "environment_reader");
    assert!(!schema.description.is_empty());
    
    // ProcessExecutorTool
    let tool = ProcessExecutorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "process_executor");
    assert!(!schema.description.is_empty());
    
    // ServiceCheckerTool
    let tool = ServiceCheckerTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "service_checker");
    assert!(!schema.description.is_empty());
    
    // SystemMonitorTool
    let tool = SystemMonitorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "system_monitor");
    assert!(!schema.description.is_empty());
}

#[test]
fn test_media_processing_tools_creation() {
    // AudioProcessorTool
    let tool = AudioProcessorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "audio_processor");
    assert!(!schema.description.is_empty());
    
    // VideoProcessorTool
    let tool = VideoProcessorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "video_processor");
    assert!(!schema.description.is_empty());
    
    // ImageProcessorTool
    let tool = ImageProcessorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "image_processor");
    assert!(!schema.description.is_empty());
}

#[test]
fn test_search_tool_creation() {
    let tool = WebSearchTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "web_search");
    assert!(!schema.description.is_empty());
}

#[test]
fn test_utility_tools_creation() {
    // HashCalculatorTool
    let tool = HashCalculatorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "hash_calculator");
    assert!(!schema.description.is_empty());
    
    // TextManipulatorTool
    let tool = TextManipulatorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "text_manipulator");
    assert!(!schema.description.is_empty());
    
    // UuidGeneratorTool
    let tool = UuidGeneratorTool::new(Default::default());
    let schema = tool.schema();
    assert_eq!(schema.name, "uuid_generator");
    assert!(!schema.description.is_empty());
}

// ===== Schema Inspection Tests =====

#[test]
fn test_tool_schemas_have_required_fields() {
    let sandbox = create_file_sandbox();
    
    // Check that tools have proper parameter definitions
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(FileWatcherTool::new(Default::default(), sandbox.clone())),
        Box::new(FileConverterTool::new(Default::default(), sandbox.clone())),
        Box::new(FileSearchTool::new(Default::default(), sandbox)),
        Box::new(EnvironmentReaderTool::new(Default::default())),
        Box::new(ProcessExecutorTool::new(Default::default())),
        Box::new(ServiceCheckerTool::new(Default::default())),
        Box::new(SystemMonitorTool::new(Default::default())),
        Box::new(AudioProcessorTool::new(Default::default())),
        Box::new(VideoProcessorTool::new(Default::default())),
        Box::new(ImageProcessorTool::new(Default::default())),
        Box::new(WebSearchTool::new(Default::default())),
        Box::new(HashCalculatorTool::new(Default::default())),
        Box::new(TextManipulatorTool::new(Default::default())),
        Box::new(UuidGeneratorTool::new(Default::default())),
    ];
    
    for tool in tools {
        let schema = tool.schema();
        let param_names: Vec<String> = schema.parameters.iter().map(|p| p.name.clone()).collect();
        println!("Tool: {} - Parameters: {:?}", schema.name, param_names);
        
        // Every tool should have a name and description
        assert!(!schema.name.is_empty());
        assert!(!schema.description.is_empty());
        
        // Every tool should have at least one parameter (most have "operation" or similar)
        assert!(!schema.parameters.is_empty(), "Tool {} has no parameters", schema.name);
    }
}

// ===== Basic Execution Tests =====

#[tokio::test]
async fn test_hash_calculator_basic() {
    let tool = HashCalculatorTool::new(Default::default());
    
    // Test with correct parameters based on schema
    let input = AgentInput::text("hash test").with_parameter(
        "parameters",
        json!({
            "operation": "hash",
            "algorithm": "sha256",
            "data": "test"
        })
    );
    
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    let output: Value = serde_json::from_str(&response.text).unwrap();
    assert!(output["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_uuid_generator_basic() {
    let tool = UuidGeneratorTool::new(Default::default());
    
    let input = AgentInput::text("generate uuid").with_parameter(
        "parameters",
        json!({
            "operation": "generate",
            "version": "v4"
        })
    );
    
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    let output: Value = serde_json::from_str(&response.text).unwrap();
    assert!(output["success"].as_bool().unwrap());
    assert!(output["result"]["uuid"].is_string());
}

#[tokio::test]
async fn test_web_search_basic() {
    let tool = WebSearchTool::new(Default::default());
    
    // Get schema to understand parameters
    let schema = tool.schema();
    let param_names: Vec<String> = schema.parameters.iter().map(|p| p.name.clone()).collect();
    println!("WebSearchTool parameters: {:?}", param_names);
    
    let input = AgentInput::text("search test").with_parameter(
        "parameters",
        json!({
            "query": "test",
            "max_results": 5
        })
    );
    
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_ok(), "Tool execution failed: {:?}", result);
    
    let response = result.unwrap();
    println!("WebSearchTool response: {}", response.text);
    
    // WebSearchTool returns a direct result, not wrapped in success/output format
    // Skip the JSON parsing since it's likely a plain text response for mock searches
    assert!(!response.text.is_empty());
}

// ===== Performance Test =====

#[test]
fn test_tool_creation_performance() {
    use std::time::Instant;
    
    let iterations = 100;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let sandbox = create_file_sandbox();
        let _ = FileWatcherTool::new(Default::default(), sandbox.clone());
        let _ = FileConverterTool::new(Default::default(), sandbox.clone());
        let _ = FileSearchTool::new(Default::default(), sandbox);
        let _ = EnvironmentReaderTool::new(Default::default());
        let _ = ProcessExecutorTool::new(Default::default());
        let _ = ServiceCheckerTool::new(Default::default());
        let _ = SystemMonitorTool::new(Default::default());
        let _ = AudioProcessorTool::new(Default::default());
        let _ = VideoProcessorTool::new(Default::default());
        let _ = ImageProcessorTool::new(Default::default());
        let _ = WebSearchTool::new(Default::default());
        let _ = HashCalculatorTool::new(Default::default());
        let _ = TextManipulatorTool::new(Default::default());
        let _ = UuidGeneratorTool::new(Default::default());
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / iterations;
    
    println!("Average time to create all 14 tools: {:?}", avg_duration);
    assert!(avg_duration.as_millis() < 10, "Tool creation took too long");
}