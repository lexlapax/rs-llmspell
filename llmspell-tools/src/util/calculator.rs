// ABOUTME: Mathematical expression calculator with support for arithmetic and scientific functions
// ABOUTME: Provides expression evaluation with variables, validation, and helpful error messages

//! Calculator tool
//!
//! This tool provides mathematical expression evaluation including:
//! - Basic arithmetic operations (+, -, *, /, %, ^)
//! - Scientific functions (trigonometry, logarithms, etc.)
//! - Variable storage and substitution
//! - Expression validation with helpful errors

use crate::{lifecycle::HookableToolExecution, resource_limited::ResourceLimited};
use async_trait::async_trait;
use fasteval::Error as FastevalError;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits as ToolResourceLimits, SecurityLevel,
            SecurityRequirements, Tool, ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{tool_error, validation_error},
    params::{
        extract_optional_object, extract_parameters, extract_required_string,
        extract_string_with_default,
    },
    resource_limits::{ResourceLimits, ResourceTracker},
    response::ResponseBuilder,
    security::{
        EnhancedExpressionAnalyzer, EnhancedExpressionConfig, ExpressionAnalyzer,
        ExpressionComplexityConfig, MemoryTracker,
    },
    timeout::with_timeout,
};
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;

/// Calculator tool for mathematical expressions
#[derive(Debug, Clone)]
pub struct CalculatorTool {
    /// Tool metadata
    metadata: ComponentMetadata,
    /// Basic expression analyzer for DoS protection
    analyzer: ExpressionAnalyzer,
    /// Enhanced analyzer for advanced DoS protection
    enhanced_analyzer: EnhancedExpressionAnalyzer,
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "calculator".to_string(),
                "Mathematical expression calculator with variables and scientific functions"
                    .to_string(),
            ),
            analyzer: ExpressionAnalyzer::with_config(ExpressionComplexityConfig::default()),
            enhanced_analyzer: EnhancedExpressionAnalyzer::with_config(
                EnhancedExpressionConfig::default(),
            ),
        }
    }
}

impl CalculatorTool {
    /// Create a new calculator tool
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert fasteval error to LLMSpellError
    fn convert_error(&self, error: FastevalError) -> LLMSpellError {
        tool_error(error.to_string(), Some(self.metadata.name.clone()))
    }

    /// Evaluate expression with custom functions and variables
    async fn evaluate_expression(
        &self,
        expression: &str,
        variables: &serde_json::Map<String, JsonValue>,
    ) -> Result<f64> {
        // First, analyze expression complexity for DoS protection
        let complexity = self.analyzer.analyze(expression);
        if !complexity.is_safe {
            return Err(validation_error(
                format!(
                    "Expression too complex: {}",
                    complexity.unsafe_reason.unwrap_or_default()
                ),
                Some("input".to_string()),
            ));
        }

        // Then run enhanced analysis for advanced DoS protection
        let enhanced_complexity = self.enhanced_analyzer.analyze(expression);
        if !enhanced_complexity.is_safe {
            return Err(validation_error(
                format!(
                    "Expression failed security check: {}",
                    enhanced_complexity.unsafe_reason.unwrap_or_default()
                ),
                Some("input".to_string()),
            ));
        }

        // Preprocess custom functions
        let processed_expr = self.preprocess_custom_functions(expression);

        // Convert JSON variables to BTreeMap<String, f64>
        let mut ns = BTreeMap::new();
        for (name, value) in variables {
            if let Some(n) = value.as_f64() {
                ns.insert(name.clone(), n);
            }
        }

        // Create memory tracker for this evaluation
        let memory_tracker = MemoryTracker::new(1_000_000); // 1MB limit per evaluation

        // Track initial memory for expression and variables
        let expr_memory = processed_expr.len() * 8 + variables.len() * 64;
        if let Err(e) = memory_tracker.allocate(expr_memory) {
            return Err(validation_error(
                format!("Expression requires too much memory: {}", e),
                Some("input".to_string()),
            ));
        }

        // Evaluate with timeout to prevent DoS
        let max_eval_time = self.analyzer.max_evaluation_time();
        let ns_clone = ns.clone();
        let expr_clone = processed_expr.clone();

        let result = match with_timeout(max_eval_time, async move {
            fasteval::ez_eval(&expr_clone, &mut ns_clone.clone())
        })
        .await
        {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(self.convert_error(e)),
            Err(_) => Err(validation_error(
                format!("Expression evaluation timed out after {:?}", max_eval_time),
                Some("input".to_string()),
            )),
        };

        // Clean up memory tracking
        memory_tracker.reset();

        result
    }

    /// Preprocess expression to replace custom functions with their implementations
    fn preprocess_custom_functions(&self, expression: &str) -> String {
        use regex::Regex;
        use std::sync::OnceLock;

        // Use static regex for better performance
        static SQRT_RE: OnceLock<Regex> = OnceLock::new();
        static EXP_RE: OnceLock<Regex> = OnceLock::new();
        static LN_RE: OnceLock<Regex> = OnceLock::new();

        let mut result = expression.to_string();

        // Replace sqrt(x) with (x)^0.5
        let sqrt_re = SQRT_RE.get_or_init(|| Regex::new(r"sqrt\s*\(([^)]+)\)").unwrap());
        result = sqrt_re.replace_all(&result, "($1)^0.5").to_string();

        // Replace exp(x) with e()^(x)
        let exp_re = EXP_RE.get_or_init(|| Regex::new(r"exp\s*\(([^)]+)\)").unwrap());
        result = exp_re.replace_all(&result, "e()^($1)").to_string();

        // Replace ln(x) with log(e(), x)
        let ln_re = LN_RE.get_or_init(|| Regex::new(r"ln\s*\(([^)]+)\)").unwrap());
        result = ln_re.replace_all(&result, "log(e(), $1)").to_string();

        result
    }

    /// Process calculator operation
    async fn process_operation(&self, params: &JsonValue) -> Result<JsonValue> {
        let operation = extract_string_with_default(params, "operation", "evaluate");

        match operation {
            "evaluate" => {
                let expression = extract_required_string(params, "input")?;

                // Get variables if provided
                let variables = extract_optional_object(params, "variables")
                    .cloned()
                    .unwrap_or_default();

                // Use our custom evaluation method
                let result = self.evaluate_expression(expression, &variables).await?;

                // Handle special float values (infinity, NaN) that don't serialize well to JSON
                let result_value = if result.is_infinite() {
                    if result.is_sign_positive() {
                        json!("Infinity")
                    } else {
                        json!("-Infinity")
                    }
                } else if result.is_nan() {
                    json!("NaN")
                } else {
                    json!(result)
                };

                let response = ResponseBuilder::success("evaluate")
                    .with_message("Expression evaluated successfully")
                    .with_result(json!({
                        "input": expression,
                        "result": result_value,
                        "result_type": if result.is_finite() { "float" } else { "special" },
                        "variables": variables,
                    }))
                    .build();
                Ok(response)
            }
            "validate" => {
                let expression = extract_required_string(params, "input")?;

                // Try to evaluate the expression with empty variables to validate syntax
                let empty_vars = serde_json::Map::new();
                match self.evaluate_expression(expression, &empty_vars).await {
                    Ok(_) => {
                        let response = ResponseBuilder::success("validate")
                            .with_message("Expression is valid")
                            .with_result(json!({
                                "input": expression,
                                "valid": true,
                            }))
                            .build();
                        Ok(response)
                    }
                    Err(e) => {
                        let response = ResponseBuilder::success("validate")
                            .with_message("Expression validation failed")
                            .with_result(json!({
                                "input": expression,
                                "valid": false,
                                "error": e.to_string()
                            }))
                            .build();
                        Ok(response)
                    }
                }
            }
            "functions" => {
                // List available functions
                let response = ResponseBuilder::success("functions")
                    .with_message("Available functions and operators")
                    .with_result(json!({
                        "arithmetic": ["+", "-", "*", "/", "%", "^"],
                        "comparison": ["==", "!=", "<", ">", "<=", ">="],
                        "logical": ["&&", "||", "!"],
                        "trigonometric": ["sin", "cos", "tan", "asin", "acos", "atan"],
                        "hyperbolic": ["sinh", "cosh", "tanh", "asinh", "acosh", "atanh"],
                        "mathematical": ["sqrt", "exp", "ln", "log", "abs", "sign"],
                        "rounding": ["int", "ceil", "floor", "round"],
                        "constants": ["pi()", "e()"],
                        "utility": ["min", "max"],
                        "examples": {
                            "basic": "2 + 3 * 4",
                            "variables": "x^2 + y^2 where x=3, y=4",
                            "trigonometry": "sin(pi()/2) + cos(0)",
                            "complex": "sqrt(x^2 + y^2) * exp(-t)",
                            "logarithms": "log(10, 100) or ln(e())"
                        },
                        "note": "All trigonometric functions work in radians. Use deg/360*2*pi() to convert degrees."
                    }))
                    .build();
                Ok(response)
            }
            _ => Err(validation_error(
                format!("Unknown operation: {operation}"),
                Some("operation".to_string()),
            )),
        }
    }
}

#[async_trait]
impl BaseAgent for CalculatorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Create resource tracker for this execution
        let limits = ResourceLimits {
            max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
            max_cpu_time_ms: Some(5_000),             // 5 seconds
            max_operations: Some(10_000),             // 10K operations
            operation_timeout_ms: Some(5_000),        // 5 seconds
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Track the operation
        tracker.track_operation()?;

        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Process the operation with resource tracking
        let result = tracker
            .with_timeout(async { self.process_operation(params).await })
            .await;

        // Format the result
        match result {
            Ok(Ok(response)) => {
                // Add resource metrics to the response
                let mut response_with_metrics = response;
                if let Some(obj) = response_with_metrics.as_object_mut() {
                    let metrics = tracker.get_metrics();
                    obj.insert(
                        "resource_usage".to_string(),
                        json!({
                            "memory_bytes": metrics.memory_bytes,
                            "cpu_time_ms": metrics.cpu_time_ms,
                            "operations_count": metrics.operations_count,
                        }),
                    );
                }

                // Return the result as JSON formatted text
                Ok(AgentOutput::text(
                    serde_json::to_string_pretty(&response_with_metrics).unwrap(),
                ))
            }
            Ok(Err(e)) => {
                // Return error as a response with success=false
                let error_response = ResponseBuilder::error("evaluate", e.to_string()).build();
                Ok(AgentOutput::text(
                    serde_json::to_string_pretty(&error_response).unwrap(),
                ))
            }
            Err(e) => {
                // Timeout error
                let error_response = ResponseBuilder::error("evaluate", e.to_string()).build();
                Ok(AgentOutput::text(
                    serde_json::to_string_pretty(&error_response).unwrap(),
                ))
            }
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(validation_error(
                "Input prompt cannot be empty",
                Some("prompt".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Calculator error: {error}")))
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "calculator".to_string(),
            "Mathematical expression calculator with variables and scientific functions"
                .to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: evaluate, validate, functions".to_string(),
            required: false,
            default: Some(json!("evaluate")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Mathematical expression to evaluate or validate".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "variables".to_string(),
            param_type: ParameterType::Object,
            description: "Variables to use in the expression (e.g., {\"x\": 5, \"y\": 3})"
                .to_string(),
            required: false,
            default: Some(json!({})),
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ToolResourceLimits {
        ToolResourceLimits::strict()
            .with_memory_limit(10 * 1024 * 1024) // 10MB
            .with_cpu_limit(1000) // 1 second
    }
}

impl ResourceLimited for CalculatorTool {
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits {
            max_memory_bytes: Some(10 * 1024 * 1024), // 10MB
            max_cpu_time_ms: Some(5_000),             // 5 seconds
            max_operations: Some(10_000),             // 10K operations
            operation_timeout_ms: Some(5_000),        // 5 seconds
            ..Default::default()
        }
    }
}

// Demonstrate that CalculatorTool automatically implements HookableToolExecution
// This is provided by the blanket implementation in the lifecycle module
impl CalculatorTool {
    /// Helper method to show hook integration capabilities
    pub fn supports_hooks(&self) -> bool {
        true // All tools that implement Tool automatically support hooks
    }

    /// Get hook integration metadata for this tool
    pub fn hook_metadata(&self) -> serde_json::Value {
        json!({
            "tool_name": self.metadata().name,
            "hook_points_supported": [
                "parameter_validation",
                "security_check",
                "resource_allocation",
                "pre_execution",
                "post_execution",
                "error_handling",
                "resource_cleanup",
                "timeout"
            ],
            "security_level": self.security_level(),
            "resource_limits": {
                "memory_mb": 10,
                "cpu_time_ms": 5000,
                "operations": 10000
            },
            "hook_integration_benefits": [
                "Security validation before expression evaluation",
                "Resource monitoring during calculation",
                "Error logging and recovery",
                "Performance metrics collection",
                "Timeout handling",
                "Audit trail for mathematical operations"
            ]
        })
    }

    /// Demonstrate hook-aware execution
    /// This method showcases how the calculator works with the hook system
    pub async fn demonstrate_hook_integration(
        &self,
        tool_executor: &crate::lifecycle::ToolExecutor,
        expression: &str,
        variables: Option<serde_json::Map<String, JsonValue>>,
    ) -> Result<AgentOutput> {
        let input = AgentInput::text("Hook demonstration").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": expression,
                "variables": variables.unwrap_or_default(),
                "hook_integration": true  // Flag to indicate this is a hook demo
            }),
        );

        let context = ExecutionContext::default();

        // Execute with hooks using the HookableToolExecution trait
        self.execute_with_hooks(input, context, tool_executor).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_arithmetic() {
        let tool = CalculatorTool::new();

        let input = AgentInput::text("calculate").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "2 + 3 * 4"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["result"], 14.0);
        assert_eq!(output["result"]["result_type"], "float");
    }

    #[tokio::test]
    async fn test_variables() {
        let tool = CalculatorTool::new();

        let input = AgentInput::text("calculate with vars").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "x^2 + y^2",
                "variables": {
                    "x": 3,
                    "y": 4
                }
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["result"], 25.0);
        assert_eq!(output["result"]["variables"]["x"], 3.0);
        assert_eq!(output["result"]["variables"]["y"], 4.0);
    }

    #[tokio::test]
    async fn test_power_operations() {
        let tool = CalculatorTool::new();

        // Test power operation
        let input = AgentInput::text("power").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "2^3"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["result"], 8.0);
        assert_eq!(output["result"]["result_type"], "float");

        // Test modulo
        let input = AgentInput::text("modulo").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "17 % 5"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["result"], 2.0);
        assert_eq!(output["result"]["result_type"], "float");
    }

    #[tokio::test]
    async fn test_expression_validation() {
        let tool = CalculatorTool::new();

        // Valid expression
        let input = AgentInput::text("validate").with_parameter(
            "parameters",
            json!({
                "operation": "validate",
                "input": "2 + 3 * 4"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["valid"], true);

        // Invalid expression
        let input = AgentInput::text("validate invalid").with_parameter(
            "parameters",
            json!({
                "operation": "validate",
                "input": "(2 + 3"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["valid"], false);
        // The error message should indicate an issue with the expression
        assert!(output["result"].get("error").is_some());
    }

    #[tokio::test]
    async fn test_division_by_zero() {
        let tool = CalculatorTool::new();

        let input = AgentInput::text("divide by zero").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "1 / 0"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        // Fasteval returns infinity for division by zero
        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["result"], "Infinity");
        assert_eq!(output["result"]["result_type"], "special");
    }

    #[tokio::test]
    async fn test_functions_list() {
        let tool = CalculatorTool::new();

        let input = AgentInput::text("list functions").with_parameter(
            "parameters",
            json!({
                "operation": "functions"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert!(output["result"]["arithmetic"].is_array());
        assert!(output["result"]["logical"].is_array());
        assert!(output["result"]["examples"].is_object());
    }

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = CalculatorTool::new();

        assert_eq!(tool.metadata().name, "calculator");
        assert!(tool
            .metadata()
            .description
            .contains("Mathematical expression"));
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
    }

    #[tokio::test]
    async fn test_mathematical_functions() {
        let tool = CalculatorTool::new();

        // Test trigonometric functions
        let input = AgentInput::text("trig").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "sin(pi()/2)"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert!((output["result"]["result"].as_f64().unwrap() - 1.0).abs() < 0.0001);

        // Test sqrt
        let input = AgentInput::text("sqrt").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "sqrt(16)"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["result"], 4.0);

        // Test logarithm
        let input = AgentInput::text("log").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "log(10, 100)"
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert!(output["success"].as_bool().unwrap_or(false));
        assert!((output["result"]["result"].as_f64().unwrap() - 2.0).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_dos_protection_long_expression() {
        let tool = CalculatorTool::new();

        // Create an expression that's too long
        let long_expr = "1 + ".repeat(300) + "1";

        let input = AgentInput::text("long expr").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": long_expr
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert_eq!(output["success"], false);
        assert!(output["error"]["message"]
            .as_str()
            .unwrap()
            .contains("too long"));
    }

    #[tokio::test]
    async fn test_dos_protection_deep_nesting() {
        let tool = CalculatorTool::new();

        // Create deeply nested expression
        let nested_expr = "(".repeat(25) + "1" + &")".repeat(25);

        let input = AgentInput::text("nested expr").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": nested_expr
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert_eq!(output["success"], false);
        assert!(output["error"]["message"]
            .as_str()
            .unwrap()
            .contains("too deep"));
    }

    #[tokio::test]
    async fn test_dos_protection_too_many_operations() {
        let tool = CalculatorTool::new();

        // Create expression with too many operations
        let many_ops = (0..150)
            .map(|i| format!("{}", i))
            .collect::<Vec<_>>()
            .join(" + ");

        let input = AgentInput::text("many ops").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": many_ops
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert_eq!(output["success"], false);
        assert!(output["error"]["message"]
            .as_str()
            .unwrap()
            .contains("operations"));
    }

    #[tokio::test]
    async fn test_dos_protection_timeout() {
        let tool = CalculatorTool::new();

        // This expression would take a long time to evaluate if not for the timeout
        // Using nested exponentials that would be very slow
        let slow_expr = "2^2^2^2^2^2^2^2^2^2";

        let input = AgentInput::text("slow expr").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": slow_expr
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        // This should either timeout or be caught by complexity analysis
        if output["success"].as_bool().unwrap_or(true) {
            // If it succeeded, the result should be reasonable
            assert!(
                output["result"]["result"].is_number() || output["result"]["result"].is_string()
            );
        } else {
            // If it failed, it should be due to timeout or complexity
            let error_msg = output["error"]["message"].as_str().unwrap();
            assert!(error_msg.contains("timeout") || error_msg.contains("operations"));
        }
    }

    #[tokio::test]
    async fn test_dos_protection_large_numbers() {
        let tool = CalculatorTool::new();

        // Expression with extremely large numbers
        let large_num_expr = "123456789012345678901234567890 + 1";

        let input = AgentInput::text("large num").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": large_num_expr
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert_eq!(output["success"], false);
        assert!(output["error"]["message"]
            .as_str()
            .unwrap()
            .contains("large number"));
    }

    #[tokio::test]
    async fn test_dos_protection_dangerous_patterns() {
        let tool = CalculatorTool::new();

        // Expression with dangerous patterns
        let dangerous_exprs = vec!["1 +++ 2", "((((((((x", "x *** y /// z"];

        for expr in dangerous_exprs {
            let input = AgentInput::text("dangerous").with_parameter(
                "parameters",
                json!({
                    "operation": "evaluate",
                    "input": expr
                }),
            );

            let result = tool
                .execute(input, ExecutionContext::default())
                .await
                .unwrap();
            let output: JsonValue = serde_json::from_str(&result.text).unwrap();

            assert_eq!(
                output["success"], false,
                "Expression '{}' should fail",
                expr
            );
            let error_msg = output["error"]["message"].as_str().unwrap();
            assert!(
                error_msg.contains("consecutive")
                    || error_msg.contains("stack overflow")
                    || error_msg.contains("complex"),
                "Expression '{}' should have appropriate error message",
                expr
            );
        }
    }

    #[tokio::test]
    async fn test_hook_integration_capability() {
        let tool = CalculatorTool::new();

        // Test that the tool supports hooks
        assert!(tool.supports_hooks());

        // Test hook metadata
        let metadata = tool.hook_metadata();
        assert_eq!(metadata["tool_name"], "calculator");
        assert!(metadata["hook_points_supported"].is_array());
        assert_eq!(
            metadata["hook_points_supported"].as_array().unwrap().len(),
            8
        );
        assert!(metadata["hook_integration_benefits"].is_array());
    }

    #[tokio::test]
    async fn test_calculator_with_hook_executor() {
        use crate::lifecycle::{ToolExecutor, ToolLifecycleConfig};

        let tool = CalculatorTool::new();

        // Create a ToolExecutor for testing (without actual hooks for this demo)
        let config = ToolLifecycleConfig {
            enable_hooks: false, // Disable for testing to avoid hook dependencies
            ..Default::default()
        };
        let tool_executor = ToolExecutor::new(config, None, None);

        // Demonstrate hook integration
        let result = tool
            .demonstrate_hook_integration(&tool_executor, "2 + 3", None)
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        let parsed: JsonValue = serde_json::from_str(&output.text).unwrap();
        assert!(parsed["success"].as_bool().unwrap_or(false));
        assert_eq!(parsed["result"]["result"], 5.0);
    }

    #[tokio::test]
    async fn test_calculator_hook_integration_with_variables() {
        use crate::lifecycle::{ToolExecutor, ToolLifecycleConfig};

        let tool = CalculatorTool::new();

        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        // Test with variables
        let mut variables = serde_json::Map::new();
        variables.insert("x".to_string(), json!(4));
        variables.insert("y".to_string(), json!(3));

        let result = tool
            .demonstrate_hook_integration(&tool_executor, "x^2 + y^2", Some(variables))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        let parsed: JsonValue = serde_json::from_str(&output.text).unwrap();
        assert!(parsed["success"].as_bool().unwrap_or(false));
        assert_eq!(parsed["result"]["result"], 25.0);
    }

    #[tokio::test]
    async fn test_hookable_tool_execution_trait() {
        use crate::lifecycle::{HookableToolExecution, ToolExecutor, ToolLifecycleConfig};

        let tool = CalculatorTool::new();

        // Verify the tool implements HookableToolExecution
        // This is automatic via the blanket implementation
        let config = ToolLifecycleConfig::default();
        let tool_executor = ToolExecutor::new(config, None, None);

        let input = AgentInput::text("test hook trait").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "10 / 2"
            }),
        );

        let context = ExecutionContext::default();

        // Call the trait method directly
        let result = tool
            .execute_with_hooks(input, context, &tool_executor)
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        let parsed: JsonValue = serde_json::from_str(&output.text).unwrap();
        assert!(parsed["success"].as_bool().unwrap_or(false));
        assert_eq!(parsed["result"]["result"], 5.0);
    }
}
