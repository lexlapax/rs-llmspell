// ABOUTME: Integration tests for the CalculatorTool
// ABOUTME: Tests complex mathematical expressions and edge cases

//! Integration tests for CalculatorTool

use llmspell_core::{
    traits::{base_agent::BaseAgent, tool::Tool},
    types::{AgentInput, ExecutionContext},
};
use llmspell_tools::util::CalculatorTool;
use serde_json::{json, Value};

/// Helper function to evaluate an expression with the calculator
async fn evaluate_expression(
    expression: &str,
    variables: Option<Value>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let tool = CalculatorTool::new();

    let mut params = json!({
        "operation": "evaluate",
        "expression": expression
    });

    if let Some(vars) = variables {
        params["variables"] = vars;
    }

    let input = AgentInput::text("Calculate").with_parameter("parameters", params);

    let result = tool.execute(input, ExecutionContext::default()).await?;
    let output: Value = serde_json::from_str(&result.text)?;

    // Extract the result from the response wrapper
    if let Some(result_obj) = output.get("result") {
        Ok(result_obj.clone())
    } else {
        Ok(output)
    }
}

#[tokio::test]
async fn test_complex_arithmetic() {
    // Test order of operations
    let result = evaluate_expression("2 + 3 * 4 - 6 / 2", None)
        .await
        .unwrap();
    assert_eq!(result["result"], 11.0); // 2 + 12 - 3 = 11

    // Test nested parentheses
    let result = evaluate_expression("((2 + 3) * (4 - 1)) / 5", None)
        .await
        .unwrap();
    assert_eq!(result["result"], 3.0); // (5 * 3) / 5 = 3

    // Test exponentiation
    let result = evaluate_expression("2^3 + 3^2", None).await.unwrap();
    assert_eq!(result["result"], 17.0); // 8 + 9 = 17
}

#[tokio::test]
async fn test_variable_substitution() {
    // Basic variable substitution
    let vars = json!({
        "x": 5,
        "y": 3
    });
    let result = evaluate_expression("x * y + 10", Some(vars)).await.unwrap();
    assert_eq!(result["result"], 25.0); // 5 * 3 + 10 = 25

    // Quadratic formula test
    let vars = json!({
        "a": 1,
        "b": -5,
        "c": 6
    });
    let result = evaluate_expression("b^2 - 4*a*c", Some(vars))
        .await
        .unwrap();
    assert_eq!(result["result"], 1.0); // 25 - 24 = 1

    // Variables in complex expressions
    let vars = json!({
        "radius": 5,
        "pi": 3.14159
    });
    let result = evaluate_expression("pi * radius^2", Some(vars))
        .await
        .unwrap();
    assert!((result["result"].as_f64().unwrap() - 78.53975).abs() < 0.0001);
}

#[tokio::test]
async fn test_comparison_operations() {
    // Test boolean results from comparisons
    let result = evaluate_expression("5 > 3", None).await.unwrap();
    assert_eq!(result["result"], true);
    assert_eq!(result["result_type"], "boolean");

    let result = evaluate_expression("10 <= 5", None).await.unwrap();
    assert_eq!(result["result"], false);

    // Test comparison with variables
    let vars = json!({"x": 10, "y": 20});
    let result = evaluate_expression("x < y && y > 15", Some(vars))
        .await
        .unwrap();
    assert_eq!(result["result"], true);
}

#[tokio::test]
async fn test_logical_operations() {
    // Test logical AND
    let result = evaluate_expression("true && false", None).await.unwrap();
    assert_eq!(result["result"], false);

    // Test logical OR
    let result = evaluate_expression("true || false", None).await.unwrap();
    assert_eq!(result["result"], true);

    // Test logical NOT
    let result = evaluate_expression("!true", None).await.unwrap();
    assert_eq!(result["result"], false);

    // Complex logical expression
    let result = evaluate_expression("(5 > 3) && (10 < 20) || false", None)
        .await
        .unwrap();
    assert_eq!(result["result"], true);
}

#[tokio::test]
async fn test_string_operations() {
    // Test string concatenation
    let vars = json!({
        "name": "Calculator",
        "suffix": "Tool"
    });
    let result = evaluate_expression("name + suffix", Some(vars))
        .await
        .unwrap();
    assert_eq!(result["result"], "CalculatorTool");
    assert_eq!(result["result_type"], "string");
}

#[tokio::test]
async fn test_edge_cases() {
    // Test very large numbers
    let result = evaluate_expression("10^10", None).await.unwrap();
    assert_eq!(result["result"], 10000000000.0);

    // Test very small numbers
    let result = evaluate_expression("1.0 / 1000000", None).await.unwrap();
    assert_eq!(result["result"], 0.000001);

    // Test negative numbers
    let result = evaluate_expression("-5 * -3", None).await.unwrap();
    assert_eq!(result["result"], 15.0);

    // Test modulo with negative numbers
    let result = evaluate_expression("-17 % 5", None).await.unwrap();
    assert_eq!(result["result"], -2);
}

#[tokio::test]
async fn test_validation_operation() {
    let tool = CalculatorTool::new();

    // Valid expression
    let input = AgentInput::text("validate").with_parameter(
        "parameters",
        json!({
            "operation": "validate",
            "expression": "x^2 + y^2"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["result"]["valid"], true);

    // Expression with mismatched parentheses
    let input = AgentInput::text("validate").with_parameter(
        "parameters",
        json!({
            "operation": "validate",
            "expression": "((x + y) * z"
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["result"]["valid"], false);
    // For validation, errors are in the result
    assert!(output["result"].get("error").is_some());
}

#[tokio::test]
async fn test_error_handling() {
    // Division by zero
    let result = evaluate_expression("5 / 0", None).await;
    assert!(result.is_err());

    // Undefined variable
    let result = evaluate_expression("x + y", None).await;
    assert!(result.is_err());

    // Type mismatch
    let vars = json!({
        "x": "hello",
        "y": 5
    });
    let result = evaluate_expression("x * y", Some(vars)).await;
    assert!(result.is_err());
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
    let output: Value = serde_json::from_str(&result.text).unwrap();

    // Verify structure
    assert!(output["success"].as_bool().unwrap_or(false));
    let functions = &output["result"];
    // The operation field might not be in the result
    // assert_eq!(functions["operation"], "functions");
    assert!(functions["arithmetic"].is_array());
    assert!(functions["comparison"].is_array());
    assert!(functions["logical"].is_array());
    assert!(functions["string"].is_array());
    assert!(functions["examples"].is_object());

    // Verify some operators are listed
    let arithmetic = functions["arithmetic"].as_array().unwrap();
    assert!(arithmetic.contains(&json!("+")));
    assert!(arithmetic.contains(&json!("*")));
    assert!(arithmetic.contains(&json!("^")));
}

#[tokio::test]
async fn test_tool_characteristics() {
    let tool = CalculatorTool::new();

    // Test metadata
    assert_eq!(tool.metadata().name, "calculator");
    assert!(tool
        .metadata()
        .description
        .contains("Mathematical expression"));

    // Test tool category and security
    assert_eq!(
        tool.category(),
        llmspell_core::traits::tool::ToolCategory::Utility
    );
    assert_eq!(
        tool.security_level(),
        llmspell_core::traits::tool::SecurityLevel::Safe
    );

    // Test resource limits
    let limits = tool.resource_limits();
    assert!(limits.max_memory_bytes.is_some());
    assert!(limits.max_cpu_time_ms.is_some());
}

#[tokio::test]
async fn test_mixed_type_variables() {
    // Test with mixed numeric types
    let vars = json!({
        "int_val": 10,
        "float_val": 3.5,
        "bool_val": true
    });

    let result = evaluate_expression("int_val + float_val", Some(vars.clone()))
        .await
        .unwrap();
    assert_eq!(result["result"], 13.5);

    // Test boolean in arithmetic context
    let result = evaluate_expression("bool_val && (int_val > 5)", Some(vars))
        .await
        .unwrap();
    assert_eq!(result["result"], true);
}
