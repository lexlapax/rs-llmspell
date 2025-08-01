//! ABOUTME: Tool composition patterns for chaining and combining tool operations
//! ABOUTME: Enables complex workflows through tool orchestration and dependency management

use llmspell_core::{ExecutionContext, LLMSpellError, Result};
use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A composition of tools that can be executed as a workflow
///
/// This allows multiple tools to be chained together, with output from
/// one tool feeding into the input of subsequent tools.
///
/// # Examples
///
/// ```
/// use llmspell_agents::composition::tool_composition::{ToolComposition, CompositionStep, DataFlow};
/// use serde_json::json;
///
/// # async fn example() -> llmspell_core::Result<()> {
/// let mut composition = ToolComposition::new("file-processing-pipeline");
///
/// // Add steps to the composition
/// composition.add_step(CompositionStep::new("step1", "file_reader")
///     .with_input_mapping("file_path", DataFlow::Parameter("input_file".to_string())));
///
/// composition.add_step(CompositionStep::new("step2", "text_processor")
///     .with_input_mapping("text", DataFlow::StepOutput("step1".to_string(), "content".to_string())));
///
/// composition.add_step(CompositionStep::new("step3", "file_writer")
///     .with_input_mapping("content", DataFlow::StepOutput("step2".to_string(), "processed_text".to_string()))
///     .with_input_mapping("output_path", DataFlow::Parameter("output_file".to_string())));
///
/// // Execute the composition
/// let initial_params = json!({"input_file": "/path/to/input.txt", "output_file": "/path/to/output.txt"});
/// // let result = composition.execute(tool_registry, initial_params, context).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ToolComposition {
    /// Unique identifier for this composition
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this composition does
    pub description: String,
    /// Steps in the composition
    pub steps: Vec<CompositionStep>,
    /// Global error handling strategy
    pub error_strategy: CompositionErrorStrategy,
    /// Maximum execution time for the entire composition
    pub max_execution_time: Option<Duration>,
    /// Whether to run steps in parallel where possible
    pub parallel_execution: bool,
    /// Shared data that persists across all steps
    pub shared_context: HashMap<String, JsonValue>,
}

/// A single step in a tool composition
#[derive(Debug, Clone)]
pub struct CompositionStep {
    /// Unique identifier for this step
    pub id: String,
    /// Name of the tool to execute
    pub tool_name: String,
    /// Input parameter mappings for this step
    pub input_mappings: HashMap<String, DataFlow>,
    /// Output transformations for this step
    pub output_transformations: HashMap<String, OutputTransform>,
    /// Error handling strategy for this step
    pub error_strategy: StepErrorStrategy,
    /// Whether this step is optional
    pub optional: bool,
    /// Conditions that must be met to execute this step
    pub conditions: Vec<ExecutionCondition>,
    /// Maximum execution time for this step
    pub max_execution_time: Option<Duration>,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
}

/// Describes how data flows between steps
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataFlow {
    /// Use a parameter from the initial input
    Parameter(String),
    /// Use output from a previous step
    StepOutput(String, String), // step_id, output_field
    /// Use a constant value
    Constant(JsonValue),
    /// Use shared context value
    SharedContext(String),
    /// Transform data from another source
    Transform {
        source: Box<DataFlow>,
        transform: DataTransform,
    },
}

/// Data transformation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTransform {
    /// Extract a field from an object
    ExtractField(String),
    /// Apply a JSON path expression
    JsonPath(String),
    /// Convert to string
    ToString,
    /// Parse as number
    ToNumber,
    /// Apply a custom transformation function
    Custom(String),
}

/// Output transformation applied to step results
#[derive(Debug, Clone)]
pub struct OutputTransform {
    /// Source field in the step output
    pub source_field: String,
    /// Target field name
    pub target_field: String,
    /// Transformation to apply
    pub transform: DataTransform,
    /// Whether to keep the original field
    pub keep_original: bool,
}

/// Error handling strategies for compositions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionErrorStrategy {
    /// Stop execution on first error
    FailFast,
    /// Continue execution, collecting errors
    ContinueOnError,
    /// Skip failed steps and continue
    SkipErrors,
    /// Retry failed steps with exponential backoff
    RetryWithBackoff {
        max_attempts: u32,
        base_delay: Duration,
    },
}

/// Error handling strategies for individual steps
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepErrorStrategy {
    /// Inherit from composition strategy
    Inherit,
    /// Stop execution on error
    Stop,
    /// Continue to next step
    Continue,
    /// Skip this step
    Skip,
    /// Retry this step
    Retry { max_attempts: u32, delay: Duration },
}

/// Execution condition for steps
#[derive(Debug, Clone)]
pub struct ExecutionCondition {
    /// Field to check
    pub field: DataFlow,
    /// Condition type
    pub condition_type: ConditionType,
    /// Value to compare against
    pub value: JsonValue,
}

/// Types of execution conditions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionType {
    /// Field equals value
    Equals,
    /// Field does not equal value
    NotEquals,
    /// Field exists and is not null
    Exists,
    /// Field does not exist or is null
    NotExists,
    /// Field matches regex (for strings)
    Matches,
    /// Field is greater than value
    GreaterThan,
    /// Field is less than value
    LessThan,
}

/// Retry configuration for steps
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Maximum delay between retries
    pub max_delay: Option<Duration>,
}

/// Result of executing a tool composition
#[derive(Debug, Clone)]
pub struct CompositionResult {
    /// Whether the composition succeeded overall
    pub success: bool,
    /// Results from each step
    pub step_results: HashMap<String, StepResult>,
    /// Final output of the composition
    pub output: JsonValue,
    /// Any errors that occurred
    pub errors: Vec<CompositionError>,
    /// Execution metrics
    pub metrics: CompositionMetrics,
}

/// Result of executing a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Whether the step succeeded
    pub success: bool,
    /// Output from the step
    pub output: JsonValue,
    /// Error if step failed
    pub error: Option<String>,
    /// Step execution metrics
    pub metrics: StepMetrics,
}

/// Error that occurred during composition execution
#[derive(Debug, Clone)]
pub struct CompositionError {
    /// Step ID where error occurred
    pub step_id: String,
    /// Error message
    pub message: String,
    /// Whether this error stopped execution
    pub fatal: bool,
}

/// Metrics for composition execution
#[derive(Debug, Clone)]
pub struct CompositionMetrics {
    /// Total execution time
    pub total_execution_time: Duration,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Number of steps that failed
    pub steps_failed: usize,
    /// Number of retries performed
    pub total_retries: u32,
}

/// Metrics for step execution
#[derive(Debug, Clone)]
pub struct StepMetrics {
    /// Step execution time
    pub execution_time: Duration,
    /// Number of retry attempts
    pub retry_attempts: u32,
    /// Memory used during execution
    pub memory_used: Option<u64>,
}

impl ToolComposition {
    /// Create a new tool composition
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            description: format!("Tool composition: {}", name),
            steps: Vec::new(),
            error_strategy: CompositionErrorStrategy::FailFast,
            max_execution_time: Some(Duration::from_secs(300)), // 5 minutes default
            parallel_execution: false,
            shared_context: HashMap::new(),
        }
    }

    /// Add a step to the composition
    pub fn add_step(&mut self, step: CompositionStep) {
        self.steps.push(step);
    }

    /// Set error strategy for the composition
    pub fn with_error_strategy(mut self, strategy: CompositionErrorStrategy) -> Self {
        self.error_strategy = strategy;
        self
    }

    /// Set maximum execution time
    pub fn with_max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = Some(duration);
        self
    }

    /// Enable parallel execution where possible
    pub fn with_parallel_execution(mut self, enabled: bool) -> Self {
        self.parallel_execution = enabled;
        self
    }

    /// Add shared context data
    pub fn with_shared_context(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.shared_context.insert(key.into(), value);
        self
    }

    /// Execute the composition with the given tool provider
    pub async fn execute<T>(
        &self,
        tool_provider: &T,
        initial_parameters: JsonValue,
        context: ExecutionContext,
    ) -> Result<CompositionResult>
    where
        T: ToolProvider,
    {
        let start_time = Instant::now();
        let mut step_results = HashMap::new();
        let mut errors = Vec::new();
        let mut execution_context =
            CompositionExecutionContext::new(initial_parameters, &self.shared_context);

        let mut steps_executed = 0;
        let mut steps_failed = 0;
        let mut total_retries = 0;

        for step in &self.steps {
            // Check if we should continue based on error strategy
            if self.should_stop_execution(&errors) {
                break;
            }

            // Check step conditions
            if !self.check_step_conditions(step, &execution_context)? {
                continue;
            }

            let step_start_time = Instant::now();
            let mut retry_attempts = 0;

            let step_result = loop {
                steps_executed += 1;

                // Prepare step input
                let step_input = self.prepare_step_input(step, &execution_context)?;

                // Execute the step
                match self
                    .execute_step(tool_provider, step, step_input, context.clone())
                    .await
                {
                    Ok(output) => {
                        let step_metrics = StepMetrics {
                            execution_time: step_start_time.elapsed(),
                            retry_attempts,
                            memory_used: None,
                        };

                        let result = StepResult {
                            success: true,
                            output: output.clone(),
                            error: None,
                            metrics: step_metrics,
                        };

                        // Update execution context with step output
                        execution_context.set_step_output(&step.id, output);

                        break result;
                    }
                    Err(e) => {
                        steps_failed += 1;
                        let error_msg = e.to_string();

                        // Check if we should retry
                        if let Some(retry_config) = &step.retry_config {
                            if retry_attempts < retry_config.max_attempts {
                                retry_attempts += 1;
                                total_retries += 1;

                                let delay =
                                    self.calculate_retry_delay(retry_config, retry_attempts);
                                tokio::time::sleep(delay).await;
                                continue;
                            }
                        }

                        let step_metrics = StepMetrics {
                            execution_time: step_start_time.elapsed(),
                            retry_attempts,
                            memory_used: None,
                        };

                        let result = StepResult {
                            success: false,
                            output: JsonValue::Null,
                            error: Some(error_msg.clone()),
                            metrics: step_metrics,
                        };

                        // Handle step error based on strategy
                        let fatal = self.handle_step_error(step, &e);
                        errors.push(CompositionError {
                            step_id: step.id.clone(),
                            message: error_msg,
                            fatal,
                        });

                        break result;
                    }
                }
            };

            step_results.insert(step.id.clone(), step_result);
        }

        // Determine final output
        let output = if let Some(last_step) = self.steps.last() {
            step_results
                .get(&last_step.id)
                .map(|r| r.output.clone())
                .unwrap_or(JsonValue::Null)
        } else {
            JsonValue::Null
        };

        let metrics = CompositionMetrics {
            total_execution_time: start_time.elapsed(),
            steps_executed,
            steps_failed,
            total_retries,
        };

        Ok(CompositionResult {
            success: errors.iter().all(|e| !e.fatal),
            step_results,
            output,
            errors,
            metrics,
        })
    }

    /// Check if execution should stop based on error strategy
    fn should_stop_execution(&self, errors: &[CompositionError]) -> bool {
        match self.error_strategy {
            CompositionErrorStrategy::FailFast => errors.iter().any(|e| e.fatal),
            _ => false,
        }
    }

    /// Check if step conditions are met
    fn check_step_conditions(
        &self,
        step: &CompositionStep,
        context: &CompositionExecutionContext,
    ) -> Result<bool> {
        for condition in &step.conditions {
            let field_value = self.resolve_data_flow(&condition.field, context)?;

            let condition_met = match condition.condition_type {
                ConditionType::Equals => field_value == condition.value,
                ConditionType::NotEquals => field_value != condition.value,
                ConditionType::Exists => !field_value.is_null(),
                ConditionType::NotExists => field_value.is_null(),
                ConditionType::Matches => {
                    if let (JsonValue::String(field_str), JsonValue::String(pattern)) =
                        (&field_value, &condition.value)
                    {
                        // Simple pattern matching - could be enhanced with regex
                        field_str.contains(pattern)
                    } else {
                        false
                    }
                }
                ConditionType::GreaterThan => match (&field_value, &condition.value) {
                    (JsonValue::Number(a), JsonValue::Number(b)) => {
                        a.as_f64().unwrap_or(0.0) > b.as_f64().unwrap_or(0.0)
                    }
                    _ => false,
                },
                ConditionType::LessThan => match (&field_value, &condition.value) {
                    (JsonValue::Number(a), JsonValue::Number(b)) => {
                        a.as_f64().unwrap_or(0.0) < b.as_f64().unwrap_or(0.0)
                    }
                    _ => false,
                },
            };

            if !condition_met {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Prepare input for a step
    fn prepare_step_input(
        &self,
        step: &CompositionStep,
        context: &CompositionExecutionContext,
    ) -> Result<JsonValue> {
        let mut input_params = Map::new();

        for (param_name, data_flow) in &step.input_mappings {
            let value = self.resolve_data_flow(data_flow, context)?;
            input_params.insert(param_name.clone(), value);
        }

        Ok(JsonValue::Object(input_params))
    }

    /// Resolve a data flow to a concrete value
    fn resolve_data_flow(
        &self,
        data_flow: &DataFlow,
        context: &CompositionExecutionContext,
    ) -> Result<JsonValue> {
        match data_flow {
            DataFlow::Parameter(param_name) => {
                Ok(context.get_parameter(param_name).unwrap_or(JsonValue::Null))
            }
            DataFlow::StepOutput(step_id, field_name) => {
                if let Some(step_output) = context.get_step_output(step_id) {
                    if field_name == "*" {
                        Ok(step_output.clone())
                    } else if let JsonValue::Object(obj) = step_output {
                        Ok(obj.get(field_name).cloned().unwrap_or(JsonValue::Null))
                    } else {
                        Ok(JsonValue::Null)
                    }
                } else {
                    Ok(JsonValue::Null)
                }
            }
            DataFlow::Constant(value) => Ok(value.clone()),
            DataFlow::SharedContext(key) => {
                Ok(context.get_shared_context(key).unwrap_or(JsonValue::Null))
            }
            DataFlow::Transform { source, transform } => {
                let source_value = self.resolve_data_flow(source, context)?;
                self.apply_data_transform(&source_value, transform)
            }
        }
    }

    /// Apply a data transformation
    fn apply_data_transform(
        &self,
        value: &JsonValue,
        transform: &DataTransform,
    ) -> Result<JsonValue> {
        match transform {
            DataTransform::ExtractField(field_name) => {
                if let JsonValue::Object(obj) = value {
                    Ok(obj.get(field_name).cloned().unwrap_or(JsonValue::Null))
                } else {
                    Ok(JsonValue::Null)
                }
            }
            DataTransform::JsonPath(_path) => {
                // Simple JSON path implementation would go here
                // For now, just return the original value
                Ok(value.clone())
            }
            DataTransform::ToString => {
                let string_value = match value {
                    JsonValue::String(s) => s.clone(),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    JsonValue::Null => "null".to_string(),
                    _ => serde_json::to_string(value).map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to convert to string: {}", e),
                        source: Some(Box::new(e)),
                    })?,
                };
                Ok(JsonValue::String(string_value))
            }
            DataTransform::ToNumber => match value {
                JsonValue::Number(n) => Ok(JsonValue::Number(n.clone())),
                JsonValue::String(s) => {
                    if let Ok(n) = s.parse::<f64>() {
                        Ok(JsonValue::Number(
                            serde_json::Number::from_f64(n)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ))
                    } else {
                        Ok(JsonValue::Number(serde_json::Number::from(0)))
                    }
                }
                _ => Ok(JsonValue::Number(serde_json::Number::from(0))),
            },
            DataTransform::Custom(function_name) => {
                // Custom transformations would be implemented here
                tracing::warn!("Custom transform '{}' not implemented", function_name);
                Ok(value.clone())
            }
        }
    }

    /// Execute a single step
    async fn execute_step<T>(
        &self,
        tool_provider: &T,
        step: &CompositionStep,
        input: JsonValue,
        context: ExecutionContext,
    ) -> Result<JsonValue>
    where
        T: ToolProvider,
    {
        tool_provider
            .execute_tool(&step.tool_name, input, context)
            .await
    }

    /// Handle step error based on strategy
    fn handle_step_error(&self, step: &CompositionStep, _error: &LLMSpellError) -> bool {
        match step.error_strategy {
            StepErrorStrategy::Inherit => {
                matches!(self.error_strategy, CompositionErrorStrategy::FailFast)
            }
            StepErrorStrategy::Stop => true,
            StepErrorStrategy::Continue | StepErrorStrategy::Skip => false,
            StepErrorStrategy::Retry { .. } => false, // Retries are handled separately
        }
    }

    /// Calculate retry delay
    fn calculate_retry_delay(&self, retry_config: &RetryConfig, attempt: u32) -> Duration {
        let base_delay = retry_config.base_delay;

        if retry_config.exponential_backoff {
            let multiplier = 2_u32.pow(attempt.saturating_sub(1));
            let delay = base_delay * multiplier;

            if let Some(max_delay) = retry_config.max_delay {
                delay.min(max_delay)
            } else {
                delay
            }
        } else {
            base_delay
        }
    }
}

impl CompositionStep {
    /// Create a new composition step
    pub fn new(id: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            tool_name: tool_name.into(),
            input_mappings: HashMap::new(),
            output_transformations: HashMap::new(),
            error_strategy: StepErrorStrategy::Inherit,
            optional: false,
            conditions: Vec::new(),
            max_execution_time: None,
            retry_config: None,
        }
    }

    /// Add an input mapping
    pub fn with_input_mapping(
        mut self,
        param_name: impl Into<String>,
        data_flow: DataFlow,
    ) -> Self {
        self.input_mappings.insert(param_name.into(), data_flow);
        self
    }

    /// Set error strategy for this step
    pub fn with_error_strategy(mut self, strategy: StepErrorStrategy) -> Self {
        self.error_strategy = strategy;
        self
    }

    /// Mark step as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Add execution condition
    pub fn with_condition(mut self, condition: ExecutionCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set retry configuration
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }
}

/// Execution context for a composition
struct CompositionExecutionContext {
    /// Initial parameters
    parameters: Map<String, JsonValue>,
    /// Output from each step
    step_outputs: HashMap<String, JsonValue>,
    /// Shared context data
    shared_context: HashMap<String, JsonValue>,
}

impl CompositionExecutionContext {
    fn new(parameters: JsonValue, shared_context: &HashMap<String, JsonValue>) -> Self {
        let parameters = parameters.as_object().cloned().unwrap_or_default();
        Self {
            parameters,
            step_outputs: HashMap::new(),
            shared_context: shared_context.clone(),
        }
    }

    fn get_parameter(&self, name: &str) -> Option<JsonValue> {
        self.parameters.get(name).cloned()
    }

    fn get_step_output(&self, step_id: &str) -> Option<&JsonValue> {
        self.step_outputs.get(step_id)
    }

    fn set_step_output(&mut self, step_id: &str, output: JsonValue) {
        self.step_outputs.insert(step_id.to_string(), output);
    }

    fn get_shared_context(&self, key: &str) -> Option<JsonValue> {
        self.shared_context.get(key).cloned()
    }
}

/// Trait for providing tools to compositions
#[async_trait::async_trait]
pub trait ToolProvider {
    /// Execute a tool by name
    async fn execute_tool(
        &self,
        tool_name: &str,
        input: JsonValue,
        context: ExecutionContext,
    ) -> Result<JsonValue>;

    /// Check if a tool is available
    async fn has_tool(&self, tool_name: &str) -> bool;
}

#[cfg(test)]
#[cfg_attr(test_category = "agent")]
mod tests {
    use super::*;
    use serde_json::json;

    struct MockToolProvider {
        tools: HashMap<String, JsonValue>,
    }

    impl MockToolProvider {
        fn new() -> Self {
            let mut tools = HashMap::new();
            tools.insert("echo".to_string(), json!({"type": "echo"}));
            tools.insert("transform".to_string(), json!({"type": "transform"}));

            Self { tools }
        }
    }

    #[async_trait::async_trait]
    impl ToolProvider for MockToolProvider {
        async fn execute_tool(
            &self,
            tool_name: &str,
            input: JsonValue,
            _context: ExecutionContext,
        ) -> Result<JsonValue> {
            match tool_name {
                "echo" => Ok(input),
                "transform" => {
                    if let Some(text) = input.get("text").and_then(|v| v.as_str()) {
                        Ok(json!({"result": text.to_uppercase()}))
                    } else {
                        Ok(json!({"result": "NO_TEXT"}))
                    }
                }
                _ => Err(LLMSpellError::Component {
                    message: format!("Tool not found: {}", tool_name),
                    source: None,
                }),
            }
        }

        async fn has_tool(&self, tool_name: &str) -> bool {
            self.tools.contains_key(tool_name)
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_simple_composition() {
        let provider = MockToolProvider::new();
        let mut composition = ToolComposition::new("test-composition");

        composition.add_step(
            CompositionStep::new("step1", "echo")
                .with_input_mapping("text", DataFlow::Parameter("input_text".to_string())),
        );

        let params = json!({"input_text": "hello world"});
        let context = ExecutionContext::new();

        let result = composition
            .execute(&provider, params, context)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.step_results.len(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_chained_composition() {
        let provider = MockToolProvider::new();
        let mut composition = ToolComposition::new("chained-test");

        composition.add_step(
            CompositionStep::new("step1", "echo")
                .with_input_mapping("text", DataFlow::Parameter("input_text".to_string())),
        );

        composition.add_step(
            CompositionStep::new("step2", "transform").with_input_mapping(
                "text",
                DataFlow::StepOutput("step1".to_string(), "text".to_string()),
            ),
        );

        let params = json!({"input_text": "hello world"});
        let context = ExecutionContext::new();

        let result = composition
            .execute(&provider, params, context)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.step_results.len(), 2);

        // Check that the second step got the output from the first
        let step2_result = result.step_results.get("step2").unwrap();
        assert!(step2_result.success);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_data_flow_transforms() {
        let composition = ToolComposition::new("test");
        let context = CompositionExecutionContext::new(
            json!({"nested": {"field": "value"}}),
            &HashMap::new(),
        );

        // Test parameter resolution
        let param_flow = DataFlow::Parameter("nested".to_string());
        let result = composition
            .resolve_data_flow(&param_flow, &context)
            .unwrap();
        assert_eq!(result, json!({"field": "value"}));

        // Test constant resolution
        let const_flow = DataFlow::Constant(json!("constant_value"));
        let result = composition
            .resolve_data_flow(&const_flow, &context)
            .unwrap();
        assert_eq!(result, json!("constant_value"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_data_transforms() {
        let composition = ToolComposition::new("test");

        // Test ToString transform
        let result = composition
            .apply_data_transform(&json!(123), &DataTransform::ToString)
            .unwrap();
        assert_eq!(result, json!("123"));

        // Test ExtractField transform
        let result = composition
            .apply_data_transform(
                &json!({"field1": "value1", "field2": "value2"}),
                &DataTransform::ExtractField("field1".to_string()),
            )
            .unwrap();
        assert_eq!(result, json!("value1"));

        // Test ToNumber transform
        let result = composition
            .apply_data_transform(&json!("42"), &DataTransform::ToNumber)
            .unwrap();
        assert_eq!(result.as_f64(), Some(42.0));
    }
}
