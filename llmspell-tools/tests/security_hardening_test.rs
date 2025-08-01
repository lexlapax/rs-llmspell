// ABOUTME: Security hardening tests for Phase 3.0.12 critical security fixes
// ABOUTME: Tests DoS protection for calculator tool

//! Security hardening integration tests
//!
//! This test suite validates the security hardening measures implemented
//! in Phase 3.0.12, specifically for Calculator DoS protection

use llmspell_core::{types::AgentInput, BaseAgent, ExecutionContext};
use llmspell_tools::util::calculator::CalculatorTool;
use serde_json::{json, Value as JsonValue};

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_calculator_dos_protection_comprehensive() {
    let tool = CalculatorTool::new();

    // Test 1: Expression length limit
    let long_expr = "1 + 2 * 3 - 4 / 5 + ".repeat(100) + "6";
    let input = AgentInput::text("test").with_parameter(
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

    // Test 2: Nesting depth limit
    let nested = "sin(cos(tan(".repeat(10) + "1" + &")))".repeat(10);
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": nested
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: JsonValue = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], false);

    // Test 3: Operation count limit
    let many_ops = (0..150)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" + ");
    let input = AgentInput::text("test").with_parameter(
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
    println!(
        "Error message for many operations: {}",
        output["error"]["message"]
    );
    assert!(output["error"]["message"]
        .as_str()
        .unwrap()
        .contains("operation"));

    // Test 4: Function count limit
    let many_funcs = (0..60)
        .map(|i| format!("sin({})", i))
        .collect::<Vec<_>>()
        .join(" + ");
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": many_funcs
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
        .contains("function"));

    // Test 5: Dangerous patterns
    let patterns = vec![
        "1 +++ 2",
        "x --- y",
        "a *** b",
        "c /// d",
        "(((((((((((x",
        "12345678901234567890123456789012345678901234567890",
    ];

    for pattern in patterns {
        let input = AgentInput::text("test").with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": pattern
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();
        assert_eq!(
            output["success"], false,
            "Pattern '{}' should fail",
            pattern
        );
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_calculator_safe_expressions_still_work() {
    let tool = CalculatorTool::new();

    // Test that normal expressions still work
    let safe_expressions = vec![
        ("2 + 2", 4.0),
        ("10 * 5 - 3", 47.0),
        ("sin(0)", 0.0),
        ("sqrt(16)", 4.0),
        ("2^3", 8.0),
        ("log(10, 100)", 2.0),
    ];

    for (expr, expected) in safe_expressions {
        let input = AgentInput::text("test").with_parameter(
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
            output["success"], true,
            "Expression '{}' should succeed",
            expr
        );
        let result_val = output["result"]["result"].as_f64().unwrap();
        assert!(
            (result_val - expected).abs() < 0.0001,
            "Expression '{}' = {} (expected {})",
            expr,
            result_val,
            expected
        );
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_calculator_timeout_protection() {
    let tool = CalculatorTool::new();

    // Test expressions that might take too long
    let potentially_slow = vec![
        "2^2^2^2^2^2^2^2",                         // Nested exponentials
        "(((((((((1+1)*2)*2)*2)*2)*2)*2)*2)*2)*2", // Deep but valid
    ];

    for expr in potentially_slow {
        let input = AgentInput::text("test").with_parameter(
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

        // Should either succeed quickly or timeout
        if !output["success"].as_bool().unwrap_or(false) {
            let error_msg = output["error"]["message"].as_str().unwrap();
            assert!(
                error_msg.contains("timeout")
                    || error_msg.contains("complex")
                    || error_msg.contains("operations"),
                "Expression '{}' failed with: {}",
                expr,
                error_msg
            );
        }
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
async fn test_expression_validation_operation() {
    let tool = CalculatorTool::new();

    // Test the validate operation
    let long_expr = "1234567890".to_string() + &"1234567890".repeat(100);
    let test_cases = vec![
        ("2 + 2", true),
        ("sin(0) + cos(0)", true), // Use concrete values instead of variables
        ("((2 + 3) * 4)", true),
        ("2 + + 3", true), // fasteval treats this as 2 + (+3)
        ("(2 + 3", false),
        ("2 +++ 3", false), // Should fail complexity check
    ];

    // Test the long expression separately
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "validate",
            "input": &long_expr
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: JsonValue = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(
        !output["result"]["valid"].as_bool().unwrap(),
        "Long expression should be invalid"
    );

    for (expr, should_be_valid) in test_cases {
        let input = AgentInput::text("test").with_parameter(
            "parameters",
            json!({
                "operation": "validate",
                "input": expr
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: JsonValue = serde_json::from_str(&result.text).unwrap();

        assert_eq!(
            output["success"], true,
            "Validate operation should always return success"
        );
        let is_valid = output["result"]["valid"].as_bool().unwrap();
        if is_valid != should_be_valid {
            println!(
                "Expression '{}' validation mismatch. Expected: {}, Got: {}",
                expr, should_be_valid, is_valid
            );
            if let Some(error) = output["result"].get("error") {
                println!("  Error: {}", error);
            }
        }
        assert_eq!(
            is_valid, should_be_valid,
            "Expression '{}' validation result incorrect",
            expr
        );
    }
}
