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
use evalexpr::{
    eval_with_context_mut, ContextWithMutableVariables, EvalexprError, HashMapContext,
    IterateVariablesContext, Value,
};
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
use std::collections::HashMap;

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

    /// Convert evalexpr error to LLMSpellError
    fn convert_error(&self, error: EvalexprError) -> LLMSpellError {
        let message = match &error {
            EvalexprError::WrongOperatorArgumentAmount { expected, actual } => {
                format!("Wrong number of arguments: expected {expected}, got {actual}")
            }
            EvalexprError::WrongFunctionArgumentAmount {
                expected, actual, ..
            } => {
                format!("Wrong number of function arguments: expected {expected:?}, got {actual}")
            }
            EvalexprError::ExpectedString { actual } => {
                format!("Expected string but got: {actual:?}")
            }
            EvalexprError::ExpectedInt { actual } => {
                format!("Expected integer but got: {actual:?}")
            }
            EvalexprError::ExpectedFloat { actual } => {
                format!("Expected number but got: {actual:?}")
            }
            EvalexprError::ExpectedNumber { actual } => {
                format!("Expected number but got: {actual:?}")
            }
            EvalexprError::ExpectedBoolean { actual } => {
                format!("Expected boolean but got: {actual:?}")
            }
            EvalexprError::DivisionError { dividend, divisor } => {
                format!("Division error: {dividend} / {divisor}")
            }
            EvalexprError::ModulationError { dividend, divisor } => {
                format!("Modulation error: {dividend} % {divisor}")
            }
            EvalexprError::InvalidRegex { regex, message } => {
                format!("Invalid regex '{regex}': {message}")
            }
            EvalexprError::ContextNotMutable => "Context is not mutable".to_string(),
            EvalexprError::VariableIdentifierNotFound(name) => {
                format!("Variable '{name}' not found")
            }
            EvalexprError::FunctionIdentifierNotFound(name) => {
                format!("Function '{name}' not found")
            }
            _ => error.to_string(),
        };

        tool_error(message, Some(self.metadata.name.clone()))
    }

    /// Convert evalexpr Value to JSON
    #[allow(clippy::only_used_in_recursion)]
    fn value_to_json(&self, value: &Value) -> JsonValue {
        match value {
            Value::String(s) => json!(s),
            Value::Float(f) => json!(f),
            Value::Int(i) => json!(i),
            Value::Boolean(b) => json!(b),
            Value::Tuple(values) => {
                json!(values
                    .iter()
                    .map(|v| self.value_to_json(v))
                    .collect::<Vec<_>>())
            }
            Value::Empty => json!(null),
        }
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

                // Create context with variables
                let mut context = HashMapContext::new();
                for (name, value) in variables {
                    match value {
                        JsonValue::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                context
                                    .set_value(name.clone(), Value::Float(f))
                                    .map_err(|e| self.convert_error(e))?;
                            } else if let Some(i) = n.as_i64() {
                                context
                                    .set_value(name.clone(), Value::Int(i))
                                    .map_err(|e| self.convert_error(e))?;
                            }
                        }
                        JsonValue::Bool(b) => {
                            context
                                .set_value(name.clone(), Value::Boolean(b))
                                .map_err(|e| self.convert_error(e))?;
                        }
                        JsonValue::String(s) => {
                            context
                                .set_value(name.clone(), Value::String(s.clone()))
                                .map_err(|e| self.convert_error(e))?;
                        }
                        _ => {}
                    }
                }

                // Evaluate expression
                let result = eval_with_context_mut(expression, &mut context)
                    .map_err(|e| self.convert_error(e))?;

                // Get all variables after evaluation (in case expression defined new ones)
                let mut final_variables = HashMap::new();
                for (name, value) in context.iter_variables() {
                    final_variables.insert(name.clone(), self.value_to_json(&value));
                }

                let response = ResponseBuilder::success("evaluate")
                    .with_message("Expression evaluated successfully")
                    .with_result(json!({
                        "expression": expression,
                        "result": self.value_to_json(&result),
                        "result_type": match &result {
                            Value::String(_) => "string",
                            Value::Float(_) => "float",
                            Value::Int(_) => "integer",
                            Value::Boolean(_) => "boolean",
                            Value::Tuple(_) => "tuple",
                            Value::Empty => "empty",
                        },
                        "variables": final_variables,
                    }))
                    .build();
                Ok(response)
            }
            "validate" => {
                let expression = extract_required_string(params, "expression")?;

                // Try to parse the expression
                match evalexpr::build_operator_tree(expression) {
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
                                "error": self.convert_error(e).to_string()
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
                        "note": "Mathematical functions can be implemented as custom functions or variables",
                        "string": ["len", "str::regex_matches", "str::regex_replace", "str::to_lowercase", "str::to_uppercase", "str::trim"],
                        "type_checking": ["is_nan", "is_finite", "is_infinite", "is_normal"],
                        "examples": {
                            "basic": "2 + 3 * 4",
                            "variables": "x^2 + y^2 where x=3, y=4",
                            "functions": "pow(2, 3) or 2^3 for exponentiation",
                            "complex": "sqrt(x^2 + y^2) * exp(-t)"
                        }
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
        // Debug logging
        eprintln!(
            "DEBUG calculator: input.parameters = {:?}",
            input.parameters
        );

        // Get parameters using shared utility
        let params = extract_parameters(&input)?;
        eprintln!("DEBUG calculator: extracted params = {:?}", params);

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
        assert_eq!(output["result"]["result"], 14);
        assert_eq!(output["result"]["result_type"], "integer");
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
        assert_eq!(output["result"]["result"], 2);
        assert_eq!(output["result"]["result_type"], "integer");
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

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division"));
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
}
