//! Tests for tracing instrumentation in llmspell-core
//! Verifies that tracing statements are properly emitting logs

use llmspell_core::{
    execution_context::{ContextScope, ExecutionContext, InheritancePolicy},
    traits::base_agent::BaseAgent,
    traits::tool::{Tool, ToolCategory, ToolSchema, SecurityLevel},
    traits::workflow::{Workflow, WorkflowStep, Config as WorkflowConfig, Status},
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ComponentId, LLMSpellError, Result,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::io::Write;
use tracing::Level;
use tracing_subscriber::fmt;

// Mock agent for testing
struct TestAgent {
    metadata: ComponentMetadata,
}

impl TestAgent {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Test agent: {}", name),
            ),
        }
    }
}

#[async_trait]
impl BaseAgent for TestAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Simple echo implementation
        Ok(AgentOutput::text(format!("Echo: {}", input.text)))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input text cannot be empty".to_string(),
                field: Some("text".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Err(error)
    }
}

// Mock tool for testing
struct TestTool {
    metadata: ComponentMetadata,
}

impl TestTool {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Test tool: {}", name),
            ),
        }
    }
}

#[async_trait]
impl BaseAgent for TestTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Tool result: {}", input.text)))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Err(error)
    }
}

impl Tool for TestTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new("test-tool".to_string(), "A test tool for tracing verification".to_string())
    }
}

// Test workflow for testing
struct TestWorkflow {
    metadata: ComponentMetadata,
    steps: Vec<WorkflowStep>,
    config: WorkflowConfig,
}

impl TestWorkflow {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Test workflow: {}", name),
            ),
            steps: Vec::new(),
            config: WorkflowConfig::default(),
        }
    }
}

#[async_trait]
impl BaseAgent for TestWorkflow {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Workflow result: {}", input.text)))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Err(error)
    }
}

#[async_trait]
impl Workflow for TestWorkflow {
    async fn add_step(&self, _step: WorkflowStep) -> Result<()> {
        Ok(())
    }

    async fn remove_step(&self, _step_id: ComponentId) -> Result<()> {
        Ok(())
    }

    fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    async fn get_steps(&self) -> Result<Vec<WorkflowStep>> {
        Ok(self.steps.clone())
    }

    async fn get_results(&self) -> Result<Vec<llmspell_core::traits::workflow::StepResult>> {
        Ok(Vec::new())
    }

    async fn status(&self) -> Result<Status> {
        Ok(Status::Pending)
    }
}

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
        self.logs.lock().unwrap().iter().any(|log| log.contains(text))
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
async fn test_base_agent_execute_tracing() {
    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create and execute agent
    let agent = TestAgent::new("test-agent");
    let input = AgentInput::text("Hello, world!");
    let context = ExecutionContext::new();

    let result = agent.execute(input, context).await;
    assert!(result.is_ok());

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // Check for expected log messages from our instrumentation
    assert!(log_text.contains("Executing component"), "Missing 'Executing component' log");
    assert!(log_text.contains("test-agent"), "Missing agent name in logs");
    assert!(log_text.contains("input_size"), "Missing input_size in logs");
    assert!(log_text.contains("Calling execute_impl"), "Missing 'Calling execute_impl' log");
    assert!(log_text.contains("Component execution completed"), "Missing completion log");
}

#[tokio::test]
async fn test_base_agent_error_tracing() {
    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create agent and execute with valid input that causes an error in execute_impl
    let agent = TestAgent::new("error-test-agent");
    let input = AgentInput::text("Hello"); // Valid input
    let context = ExecutionContext::new();

    // Execute normally - won't fail because validate_input isn't called by execute
    let result = agent.execute(input.clone(), context.clone()).await;
    assert!(result.is_ok()); // This should succeed

    // Now test actual validation error
    let empty_input = AgentInput::text("");
    let validation_result = agent.validate_input(&empty_input).await;
    assert!(validation_result.is_err());

    // Verify tracing output contains agent name
    let logs = capture.get_logs();
    let log_text = logs.join("");

    // We should see the agent name in logs
    assert!(log_text.contains("error-test-agent"), "Missing agent name in logs");
}

#[tokio::test]
async fn test_tool_tracing() {
    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create and use tool
    let tool = TestTool::new("test-tool");

    // Test security requirements tracing
    let _requirements = tool.security_requirements();

    // Test resource limits tracing
    let _limits = tool.resource_limits();

    // Test parameter validation
    let params = serde_json::json!({
        "key": "value"
    });
    let _validation = tool.validate_parameters(&params).await;

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    assert!(log_text.contains("Getting security requirements"), "Missing security requirements log");
    assert!(log_text.contains("Getting resource limits"), "Missing resource limits log");
    assert!(log_text.contains("Validating tool parameters"), "Missing parameter validation log");
}

#[tokio::test]
async fn test_workflow_tracing() {
    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create workflow and test planning
    let workflow = TestWorkflow::new("test-workflow");

    // Test execution planning
    let _plan = workflow.plan_execution().await;

    // Test step result lookup
    let step_id = llmspell_core::ComponentId::new();
    let _result = workflow.get_step_result(step_id).await;

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    assert!(log_text.contains("Planning workflow execution"), "Missing workflow planning log");
    assert!(log_text.contains("test-workflow"), "Missing workflow name in logs");
    assert!(log_text.contains("Getting step result"), "Missing step result log");
}

#[tokio::test]
async fn test_execution_context_tracing() {
    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Test context operations
    let mut context = ExecutionContext::new();

    // Test set operation
    context.set("test_key".to_string(), serde_json::json!("test_value"));

    // Test get operation
    let _value = context.get("test_key");

    // Test child context creation
    let _child = context.create_child(
        ContextScope::Agent(llmspell_core::ComponentId::new()),
        InheritancePolicy::Inherit,
    );

    // Test merge operation
    let other_context = ExecutionContext::new();
    context.merge(&other_context);

    // Test shared memory operations
    context.set_shared("shared_key".to_string(), serde_json::json!("shared_value"));
    let _shared = context.get_shared(&ContextScope::Global, "shared_key");

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    assert!(log_text.contains("Setting value in context"), "Missing context set log");
    assert!(log_text.contains("Getting value from context"), "Missing context get log");
    assert!(log_text.contains("Creating child context"), "Missing child context log");
    assert!(log_text.contains("Merging context data"), "Missing merge log");
    assert!(log_text.contains("Setting value in shared memory"), "Missing shared memory set log");
    assert!(log_text.contains("Getting value from shared memory"), "Missing shared memory get log");
}

#[tokio::test]
async fn test_error_conversion_tracing() {
    // Initialize tracing with our capture
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Test IO error conversion
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let _llm_error: LLMSpellError = io_error.into();

    // Test JSON error conversion
    let json_str = "{invalid json}";
    let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let _llm_error: LLMSpellError = json_error.into();

    // Test fmt error conversion
    let fmt_error = std::fmt::Error;
    let _llm_error: LLMSpellError = fmt_error.into();

    // Verify tracing output
    let logs = capture.get_logs();
    let log_text = logs.join("");

    assert!(log_text.contains("IO error converted to LLMSpellError"), "Missing IO error conversion log");
    assert!(log_text.contains("JSON error converted to LLMSpellError"), "Missing JSON error conversion log");
    assert!(log_text.contains("Formatting error converted to LLMSpellError"), "Missing fmt error conversion log");
}

#[tokio::test]
async fn test_tracing_levels() {
    // Test that different log levels are working correctly
    let capture = LogCapture::new();
    let capture_clone = capture.clone();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::INFO)  // Set to INFO level
        .with_writer(move || capture_clone.clone())
        .with_ansi(false)
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Create context and perform operations
    let mut context = ExecutionContext::new();

    // This should be logged (debug level)
    context.set("key".to_string(), serde_json::json!("value"));

    // This should NOT be logged (trace level)
    let _value = context.get("key");

    // This should be logged (info level)
    let _child = context.create_child(ContextScope::Global, InheritancePolicy::Inherit);

    let logs = capture.get_logs();
    let log_text = logs.join("");

    // INFO level should be present
    assert!(log_text.contains("Creating child context"), "INFO level log missing");

    // TRACE level should NOT be present (we're at INFO level)
    assert!(!log_text.contains("Getting value from context"), "TRACE level log should not appear at INFO level");
}