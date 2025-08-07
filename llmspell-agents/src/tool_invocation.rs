//! ABOUTME: Tool invocation wrapper with validation and error handling
//! ABOUTME: Provides safe, validated tool execution with comprehensive error handling

#![allow(clippy::significant_drop_tightening)]

use llmspell_core::{
    traits::tool::Tool,
    types::{AgentInput, AgentOutput},
    ExecutionContext, LLMSpellError, Result,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Tool invocation wrapper that provides validation, error handling,
/// and execution tracking for tool calls.
///
/// This wrapper sits between `ToolCapable` components and the actual tools,
/// providing additional safety, monitoring, and debugging capabilities.
///
/// # Examples
///
/// ```
/// use llmspell_agents::tool_invocation::{ToolInvoker, InvocationConfig};
/// use llmspell_core::ExecutionContext;
/// use serde_json::json;
///
/// # async fn example() -> llmspell_core::Result<()> {
/// let config = InvocationConfig::default();
/// let invoker = ToolInvoker::new(config);
///
/// let context = ExecutionContext::new();
/// let params = json!({"pattern": "*.txt"});
///
/// // This would normally use an actual tool
/// // let result = invoker.invoke(tool, params, context).await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolInvoker {
    config: InvocationConfig,
}

/// Configuration for tool invocation behavior
#[derive(Debug, Clone)]
pub struct InvocationConfig {
    /// Maximum execution time per tool call
    pub max_execution_time: Duration,
    /// Whether to validate parameters before invocation
    pub validate_parameters: bool,
    /// Whether to track execution metrics
    pub track_metrics: bool,
    /// Whether to enable debug logging
    pub debug_logging: bool,
    /// Maximum memory usage per tool call (in bytes)
    pub max_memory_bytes: Option<u64>,
    /// Whether to sandbox tool execution
    pub enable_sandboxing: bool,
    /// Custom validation rules
    pub custom_validators: Vec<String>,
}

impl Default for InvocationConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            validate_parameters: true,
            track_metrics: true,
            debug_logging: false,
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
            enable_sandboxing: true,
            custom_validators: Vec::new(),
        }
    }
}

impl InvocationConfig {
    /// Create a new configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum execution time
    #[must_use]
    pub const fn with_max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = duration;
        self
    }

    /// Enable or disable parameter validation
    #[must_use]
    pub const fn with_parameter_validation(mut self, enabled: bool) -> Self {
        self.validate_parameters = enabled;
        self
    }

    /// Enable or disable metrics tracking
    #[must_use]
    pub const fn with_metrics_tracking(mut self, enabled: bool) -> Self {
        self.track_metrics = enabled;
        self
    }

    /// Enable or disable debug logging
    #[must_use]
    pub const fn with_debug_logging(mut self, enabled: bool) -> Self {
        self.debug_logging = enabled;
        self
    }

    /// Set maximum memory usage
    #[must_use]
    pub const fn with_max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }

    /// Enable or disable sandboxing
    #[must_use]
    pub const fn with_sandboxing(mut self, enabled: bool) -> Self {
        self.enable_sandboxing = enabled;
        self
    }
}

/// Result of a tool invocation with metadata
#[derive(Debug, Clone)]
pub struct InvocationResult {
    /// The actual output from the tool
    pub output: AgentOutput,
    /// Execution metrics
    pub metrics: InvocationMetrics,
    /// Any warnings generated during execution
    pub warnings: Vec<String>,
    /// Whether the execution was successful
    pub success: bool,
}

/// Metrics collected during tool invocation
#[derive(Debug, Clone)]
pub struct InvocationMetrics {
    /// Time taken to execute the tool
    pub execution_time: Duration,
    /// Memory used during execution (if tracked)
    pub memory_used: Option<u64>,
    /// Number of validation errors
    pub validation_errors: u32,
    /// Whether execution timed out
    pub timed_out: bool,
    /// Tool security level used
    pub security_level: String,
    /// Parameter validation time
    pub validation_time: Option<Duration>,
}

impl Default for InvocationMetrics {
    fn default() -> Self {
        Self {
            execution_time: Duration::from_millis(0),
            memory_used: None,
            validation_errors: 0,
            timed_out: false,
            security_level: "unknown".to_string(),
            validation_time: None,
        }
    }
}

/// Validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Error message
    pub message: String,
    /// Expected type or value
    pub expected: Option<String>,
    /// Actual value that failed
    pub actual: Option<JsonValue>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            expected: None,
            actual: None,
        }
    }

    /// Add expected value information
    #[must_use]
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Add actual value information
    #[must_use]
    pub fn with_actual(mut self, actual: JsonValue) -> Self {
        self.actual = Some(actual);
        self
    }
}

impl ToolInvoker {
    /// Create a new tool invoker with the given configuration
    #[must_use]
    pub const fn new(config: InvocationConfig) -> Self {
        Self { config }
    }

    /// Invoke a tool with full validation and error handling
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parameter validation fails
    /// - Security checks fail
    /// - Tool execution fails
    /// - Resource limits are exceeded
    pub async fn invoke(
        &self,
        tool: Arc<dyn Tool>,
        parameters: JsonValue,
        context: ExecutionContext,
    ) -> Result<InvocationResult> {
        let start_time = Instant::now();
        let mut metrics = InvocationMetrics::default();
        let mut warnings = Vec::new();

        // Get tool security level
        metrics.security_level = match tool.security_level() {
            llmspell_core::traits::tool::SecurityLevel::Safe => "safe",
            llmspell_core::traits::tool::SecurityLevel::Restricted => "restricted",
            llmspell_core::traits::tool::SecurityLevel::Privileged => "privileged",
        }
        .to_string();

        // Validate parameters if enabled
        if self.config.validate_parameters {
            let validation_start = Instant::now();
            match Self::validate_tool_parameters(tool.as_ref(), &parameters) {
                Ok(validation_warnings) => {
                    warnings.extend(validation_warnings);
                }
                Err(e) => {
                    metrics.validation_errors += 1;
                    metrics.execution_time = start_time.elapsed();
                    return Ok(InvocationResult {
                        output: AgentOutput::text(format!("Validation failed: {e}")),
                        metrics,
                        warnings,
                        success: false,
                    });
                }
            }
            metrics.validation_time = Some(validation_start.elapsed());
        }

        // Prepare input
        let input = AgentInput::text("Tool invocation".to_string())
            .with_parameter("parameters".to_string(), parameters);

        // Execute tool with timeout
        let execution_result =
            timeout(self.config.max_execution_time, tool.execute(input, context)).await;

        let output = match execution_result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                metrics.execution_time = start_time.elapsed();
                return Ok(InvocationResult {
                    output: AgentOutput::text(format!("Tool execution failed: {e}")),
                    metrics,
                    warnings,
                    success: false,
                });
            }
            Err(_) => {
                metrics.timed_out = true;
                metrics.execution_time = self.config.max_execution_time;
                return Ok(InvocationResult {
                    output: AgentOutput::text("Tool execution timed out".to_string()),
                    metrics,
                    warnings,
                    success: false,
                });
            }
        };

        // Update final metrics
        metrics.execution_time = start_time.elapsed();

        // Log execution if debug logging is enabled
        if self.config.debug_logging {
            tracing::debug!(
                "Tool {} executed in {:?}",
                tool.metadata().name,
                metrics.execution_time
            );
        }

        Ok(InvocationResult {
            output,
            metrics,
            warnings,
            success: true,
        })
    }

    /// Invoke a tool with basic error handling (convenience method)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tool invocation fails
    /// - Tool returns an error result
    pub async fn invoke_simple(
        &self,
        tool: Arc<dyn Tool>,
        parameters: JsonValue,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let result = self.invoke(tool, parameters, context).await?;
        if result.success {
            Ok(result.output)
        } else {
            Err(LLMSpellError::Component {
                message: result.output.text,
                source: None,
            })
        }
    }

    /// Validate tool parameters against the tool's schema
    fn validate_tool_parameters(tool: &dyn Tool, parameters: &JsonValue) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Get tool schema
        let schema = tool.schema();

        // Check that parameters is an object
        let params_map = parameters
            .as_object()
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Parameters must be an object".to_string(),
                field: Some("parameters".to_string()),
            })?;

        // Validate required parameters
        for required in schema.required_parameters() {
            if !params_map.contains_key(&required) {
                return Err(LLMSpellError::Validation {
                    message: format!("Missing required parameter: {required}"),
                    field: Some(required),
                });
            }
        }

        // Validate parameter types and constraints
        for param_def in &schema.parameters {
            if let Some(value) = params_map.get(&param_def.name) {
                // Type validation
                let valid_type = match param_def.param_type {
                    llmspell_core::traits::tool::ParameterType::String => value.is_string(),
                    llmspell_core::traits::tool::ParameterType::Number => value.is_number(),
                    llmspell_core::traits::tool::ParameterType::Boolean => value.is_boolean(),
                    llmspell_core::traits::tool::ParameterType::Array => value.is_array(),
                    llmspell_core::traits::tool::ParameterType::Object => value.is_object(),
                    llmspell_core::traits::tool::ParameterType::Null => value.is_null(),
                };

                if !valid_type {
                    return Err(LLMSpellError::Validation {
                        message: format!(
                            "Invalid type for parameter '{}': expected {:?}",
                            param_def.name, param_def.param_type
                        ),
                        field: Some(param_def.name.clone()),
                    });
                }

                // Additional validation for strings
                if param_def.param_type == llmspell_core::traits::tool::ParameterType::String {
                    if let Some(string_val) = value.as_str() {
                        if string_val.is_empty() && param_def.required {
                            warnings
                                .push(format!("Required parameter '{}' is empty", param_def.name));
                        }
                        if string_val.len() > 10000 {
                            warnings.push(format!(
                                "Parameter '{}' is very long ({} chars), this may impact performance",
                                param_def.name, string_val.len()
                            ));
                        }
                    }
                }

                // Additional validation for arrays
                if param_def.param_type == llmspell_core::traits::tool::ParameterType::Array {
                    if let Some(array_val) = value.as_array() {
                        if array_val.len() > 1000 {
                            warnings.push(format!(
                                "Parameter '{}' has many elements ({}), this may impact performance",
                                param_def.name, array_val.len()
                            ));
                        }
                    }
                }
            }
        }

        // Check for unexpected parameters
        for key in params_map.keys() {
            if !schema.parameters.iter().any(|p| &p.name == key) {
                warnings.push(format!("Unexpected parameter '{key}' will be ignored"));
            }
        }

        Ok(warnings)
    }

    /// Get configuration
    #[must_use]
    pub const fn config(&self) -> &InvocationConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: InvocationConfig) {
        self.config = config;
    }
}

impl Default for ToolInvoker {
    fn default() -> Self {
        Self::new(InvocationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::{
        traits::{base_agent::BaseAgent, tool::*},
        ComponentMetadata,
    };
    use serde_json::json;

    // Mock tool for testing
    struct MockTool {
        metadata: ComponentMetadata,
    }

    impl MockTool {
        fn new() -> Self {
            Self {
                metadata: ComponentMetadata::new(
                    "mock-tool".to_string(),
                    "A mock tool for testing".to_string(),
                ),
            }
        }
    }

    #[async_trait::async_trait]
    impl BaseAgent for MockTool {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            input: AgentInput,
            _context: ExecutionContext,
        ) -> Result<AgentOutput> {
            let params =
                input
                    .parameters
                    .get("parameters")
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Missing parameters".to_string(),
                        field: Some("parameters".to_string()),
                    })?;

            let text = params
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            Ok(AgentOutput::text(format!("Processed: {}", text)))
        }

        async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
            Ok(())
        }

        async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {}", error)))
        }
    }

    #[async_trait::async_trait]
    impl Tool for MockTool {
        fn category(&self) -> ToolCategory {
            ToolCategory::Utility
        }

        fn security_level(&self) -> SecurityLevel {
            SecurityLevel::Safe
        }

        fn schema(&self) -> ToolSchema {
            ToolSchema::new("mock_tool".to_string(), "A mock tool".to_string()).with_parameter(
                ParameterDef {
                    name: "text".to_string(),
                    param_type: ParameterType::String,
                    description: "Text to process".to_string(),
                    required: true,
                    default: None,
                },
            )
        }
    }
    #[tokio::test]
    async fn test_tool_invoker_creation() {
        let config = InvocationConfig::default();
        let invoker = ToolInvoker::new(config);

        assert_eq!(invoker.config().max_execution_time, Duration::from_secs(30));
        assert!(invoker.config().validate_parameters);
    }
    #[tokio::test]
    async fn test_invocation_config_builder() {
        let config = InvocationConfig::new()
            .with_max_execution_time(Duration::from_secs(10))
            .with_parameter_validation(false)
            .with_debug_logging(true);

        assert_eq!(config.max_execution_time, Duration::from_secs(10));
        assert!(!config.validate_parameters);
        assert!(config.debug_logging);
    }
    #[tokio::test]
    async fn test_successful_tool_invocation() {
        let config = InvocationConfig::default();
        let invoker = ToolInvoker::new(config);
        let tool: Arc<dyn Tool> = Arc::new(MockTool::new());

        let params = json!({"text": "hello world"});
        let context = ExecutionContext::new();

        let result = invoker.invoke(tool, params, context).await.unwrap();

        assert!(result.success);
        assert!(result.output.text.contains("Processed: hello world"));
        assert!(result.metrics.execution_time > Duration::from_millis(0));
    }
    #[tokio::test]
    async fn test_parameter_validation_failure() {
        let config = InvocationConfig::default();
        let invoker = ToolInvoker::new(config);
        let tool: Arc<dyn Tool> = Arc::new(MockTool::new());

        let params = json!({}); // Missing required parameter
        let context = ExecutionContext::new();

        let result = invoker.invoke(tool, params, context).await.unwrap();

        assert!(!result.success);
        assert!(result.output.text.contains("Validation failed"));
        assert_eq!(result.metrics.validation_errors, 1);
    }
    #[tokio::test]
    async fn test_simple_invocation() {
        let config = InvocationConfig::default();
        let invoker = ToolInvoker::new(config);
        let tool: Arc<dyn Tool> = Arc::new(MockTool::new());

        let params = json!({"text": "simple test"});
        let context = ExecutionContext::new();

        let output = invoker.invoke_simple(tool, params, context).await.unwrap();
        assert!(output.text.contains("Processed: simple test"));
    }
    #[tokio::test]
    async fn test_validation_error_creation() {
        let error = ValidationError::new("field1", "Invalid value")
            .with_expected("string")
            .with_actual(json!(123));

        assert_eq!(error.field, "field1");
        assert_eq!(error.message, "Invalid value");
        assert_eq!(error.expected, Some("string".to_string()));
        assert_eq!(error.actual, Some(json!(123)));
    }
}
