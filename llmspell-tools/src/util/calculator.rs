// ABOUTME: Mathematical expression calculator with support for arithmetic and scientific functions
// ABOUTME: Provides expression evaluation with variables, validation, and helpful error messages

//! Calculator tool
//!
//! This tool provides mathematical expression evaluation including:
//! - Basic arithmetic operations (+, -, *, /, %, ^)
//! - Scientific functions (trigonometry, logarithms, etc.)
//! - Variable storage and substitution
//! - Expression validation with helpful errors

use async_trait::async_trait;
use fasteval::Error as FastevalError;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{tool_error, validation_error},
    params::{
        extract_optional_object, extract_parameters, extract_required_string,
        extract_string_with_default,
    },
    response::ResponseBuilder,
};
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;

/// Calculator tool for mathematical expressions
#[derive(Debug, Clone)]
pub struct CalculatorTool {
    /// Tool metadata
    metadata: ComponentMetadata,
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "calculator".to_string(),
                "Mathematical expression calculator with variables and scientific functions"
                    .to_string(),
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
    fn evaluate_expression(
        &self,
        expression: &str,
        variables: &serde_json::Map<String, JsonValue>,
    ) -> Result<f64> {
        // Preprocess custom functions
        let processed_expr = self.preprocess_custom_functions(expression);

        // Convert JSON variables to BTreeMap<String, f64>
        let mut ns = BTreeMap::new();
        for (name, value) in variables {
            if let Some(n) = value.as_f64() {
                ns.insert(name.clone(), n);
            }
        }

        // Evaluate using fasteval
        fasteval::ez_eval(&processed_expr, &mut ns).map_err(|e| self.convert_error(e))
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
                let expression = extract_required_string(params, "expression")?;

                // Get variables if provided
                let variables = extract_optional_object(params, "variables")
                    .cloned()
                    .unwrap_or_default();

                // Use our custom evaluation method
                let result = self.evaluate_expression(expression, &variables)?;

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
                        "expression": expression,
                        "result": result_value,
                        "result_type": if result.is_finite() { "float" } else { "special" },
                        "variables": variables,
                    }))
                    .build();
                Ok(response)
            }
            "validate" => {
                let expression = extract_required_string(params, "expression")?;

                // Try to evaluate the expression with empty variables to validate syntax
                let empty_vars = serde_json::Map::new();
                match self.evaluate_expression(expression, &empty_vars) {
                    Ok(_) => {
                        let response = ResponseBuilder::success("validate")
                            .with_message("Expression is valid")
                            .with_result(json!({
                                "expression": expression,
                                "valid": true,
                            }))
                            .build();
                        Ok(response)
                    }
                    Err(e) => {
                        let response = ResponseBuilder::success("validate")
                            .with_message("Expression validation failed")
                            .with_result(json!({
                                "expression": expression,
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
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Process the operation
        let result = self.process_operation(params).await?;

        // Return the result as JSON formatted text
        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&result).unwrap(),
        ))
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
            name: "expression".to_string(),
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

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::strict()
            .with_memory_limit(10 * 1024 * 1024) // 10MB
            .with_cpu_limit(1000) // 1 second
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
                "expression": "2 + 3 * 4"
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
                "expression": "x^2 + y^2",
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
                "expression": "2^3"
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
                "expression": "17 % 5"
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
                "expression": "2 + 3 * 4"
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
                "expression": "(2 + 3"
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
                "expression": "1 / 0"
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
                "expression": "sin(pi()/2)"
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
                "expression": "sqrt(16)"
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
                "expression": "log(10, 100)"
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
}
