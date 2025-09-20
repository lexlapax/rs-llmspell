//! Tests for tracing instrumentation in llmspell-tools
//! Verifies that tracing statements are properly emitting logs in tools

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::registry::ToolRegistry;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing::Level;

// Helper to capture log output
#[derive(Clone)]
struct LogCapture {
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogCapture {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }

    #[allow(dead_code)]
    fn contains(&self, text: &str) -> bool {
        self.logs
            .lock()
            .unwrap()
            .iter()
            .any(|log| log.contains(text))
    }
}

impl Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let log = String::from_utf8_lossy(buf).to_string();
        self.logs.lock().unwrap().push(log);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_tool_registry_tracing() {
    use llmspell_tools::util::calculator::CalculatorTool;

    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create tool registry
    let registry = ToolRegistry::new();

    // Test tool registration logging - use a built-in tool
    let tool = CalculatorTool::new();
    let tool_name = "test_calculator".to_string();

    registry.register(tool_name.clone(), tool).await.unwrap();

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Check for expected log messages
    assert!(
        log_text.contains("Creating new tool registry"),
        "Missing registry creation log"
    );
    assert!(
        log_text.contains("Registering tool"),
        "Missing tool registration log"
    );
    assert!(log_text.contains(&tool_name), "Missing tool name in logs");

    // Test tool lookup logging
    let _tool = registry.get_tool(&tool_name).await;
    let logs = capture.get_logs();
    let log_text = logs.join("");
    assert!(
        log_text.contains("Looking up tool"),
        "Missing tool lookup log"
    );

    // Test tool list logging
    let _tools = registry.list_tools().await;
    let logs = capture.get_logs();
    let log_text = logs.join("");
    assert!(
        log_text.contains("Listing all registered tools"),
        "Missing tool list log"
    );
}

#[tokio::test]
async fn test_file_operations_tool_tracing() {
    use llmspell_security::sandbox::{FileSandbox, SandboxContext};
    use llmspell_tools::fs::file_operations::{FileOperationsConfig, FileOperationsTool};
    use std::sync::Arc;
    use tempfile::TempDir;

    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create file operations tool
    let temp_dir = TempDir::new().unwrap();
    let security_requirements = llmspell_core::traits::tool::SecurityRequirements::default()
        .with_file_access(temp_dir.path().to_str().unwrap());
    let resource_limits = llmspell_core::traits::tool::ResourceLimits::default();

    let context = SandboxContext::new("test".to_string(), security_requirements, resource_limits);
    let sandbox = Arc::new(FileSandbox::new(context).unwrap());

    let config = FileOperationsConfig::default();
    let tool = FileOperationsTool::new(config, sandbox);

    // Execute a file exists operation
    let input = AgentInput::text("Check file existence").with_parameter(
        "parameters",
        serde_json::json!({
            "operation": "exists",
            "path": temp_dir.path().join("test.txt").to_str().unwrap()
        }),
    );

    let _result = tool.execute(input, ExecutionContext::default()).await;

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Check for expected log messages
    assert!(
        log_text.contains("Creating FileOperationsTool"),
        "Missing tool creation log"
    );
    assert!(
        log_text.contains("Executing file operation"),
        "Missing operation execution log"
    );
    assert!(
        log_text.contains("Checking if file exists"),
        "Missing file exists check log"
    );
    assert!(
        log_text.contains("File operation completed"),
        "Missing completion log"
    );
}

#[tokio::test]
async fn test_file_search_tool_tracing() {
    use llmspell_security::sandbox::{FileSandbox, SandboxContext};
    use llmspell_tools::fs::file_search::{FileSearchConfig, FileSearchTool};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::fs;

    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create file search tool
    let temp_dir = TempDir::new().unwrap();

    // Create a test file to search
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test file with search pattern")
        .await
        .unwrap();

    let security_requirements =
        llmspell_core::traits::tool::SecurityRequirements::default().with_file_access("*");
    let resource_limits = llmspell_core::traits::tool::ResourceLimits::default();

    let context = SandboxContext::new("test".to_string(), security_requirements, resource_limits);
    let sandbox = Arc::new(FileSandbox::new(context).unwrap());

    let config = FileSearchConfig::default();
    let tool = FileSearchTool::new(config, sandbox);

    // Execute a file search
    let input = AgentInput::text("Search for pattern").with_parameter(
        "parameters",
        serde_json::json!({
            "pattern": "pattern",
            "path": test_file.to_str().unwrap()
        }),
    );

    let _result = tool.execute(input, ExecutionContext::default()).await;

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Check for expected log messages
    assert!(
        log_text.contains("Creating FileSearchTool"),
        "Missing tool creation log"
    );
    assert!(
        log_text.contains("Executing file search"),
        "Missing search execution log"
    );
    assert!(
        log_text.contains("File search completed"),
        "Missing search completion log"
    );
    assert!(
        log_text.contains("duration_ms"),
        "Missing duration tracking"
    );
}

#[tokio::test]
async fn test_http_request_tool_tracing() {
    use llmspell_tools::api::http_request::{HttpRequestConfig, HttpRequestTool};

    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create HTTP request tool
    let config = HttpRequestConfig::default();
    let _tool = HttpRequestTool::new(config).unwrap();

    // Note: We won't actually execute an HTTP request since it requires network
    // Just verify the tool creation was logged

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Check for expected log messages
    assert!(
        log_text.contains("Creating HttpRequestTool"),
        "Missing tool creation log"
    );
}

#[tokio::test]
async fn test_tool_execution_with_hooks_tracing() {
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_tools::lifecycle::ToolLifecycleConfig;
    use llmspell_tools::util::calculator::CalculatorTool;

    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create registry with hooks enabled
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let hook_config = ToolLifecycleConfig::default();

    let registry = ToolRegistry::with_hooks(Some(hook_executor), Some(hook_registry), hook_config);

    // Register a tool
    let tool = CalculatorTool::new();
    registry
        .register("calculator".to_string(), tool)
        .await
        .unwrap();

    // Execute tool through registry
    let input = AgentInput::text("Calculate 2 + 2").with_parameter(
        "parameters",
        serde_json::json!({
            "operation": "add",
            "a": 2,
            "b": 2
        }),
    );

    let _result =
        Box::pin(registry.execute_tool("calculator", input, ExecutionContext::default())).await;

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Check for expected log messages
    assert!(
        log_text.contains("Creating new tool registry with hook configuration"),
        "Missing hook registry creation log"
    );
    assert!(
        log_text.contains("Registering tool"),
        "Missing tool registration log"
    );
    assert!(
        log_text.contains("Executing tool from registry"),
        "Missing tool execution log"
    );
}

#[tokio::test]
async fn test_tracing_levels() {
    // Test that different log levels work correctly
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    // Set to INFO level - should not see TRACE or DEBUG messages
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    let registry = ToolRegistry::new();

    // This should emit DEBUG level logs (not visible at INFO level)
    let _tool = registry.get_tool("nonexistent").await;

    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Should NOT contain debug-level "Looking up tool" message
    assert!(
        !log_text.contains("Looking up tool"),
        "DEBUG log should not appear at INFO level"
    );
}
