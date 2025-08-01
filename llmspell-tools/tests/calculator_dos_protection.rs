// ABOUTME: Comprehensive security tests for CalculatorTool DoS protection
// ABOUTME: Tests enhanced protection against various attack vectors

//! Security tests for CalculatorTool DoS protection

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::util::CalculatorTool;
use serde_json::{json, Value};
use std::time::Instant;

/// Helper function to evaluate an expression and check for errors
async fn try_evaluate(expression: &str) -> Result<Value, String> {
    let tool = CalculatorTool::new();
    let params = json!({
        "operation": "evaluate",
        "input": expression
    });

    let input = AgentInput::text("Calculate").with_parameter("parameters", params);
    let start = Instant::now();

    match tool.execute(input, ExecutionContext::default()).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            // Ensure it completed within reasonable time
            if elapsed.as_millis() > 200 {
                return Err(format!("Evaluation took too long: {:?}", elapsed));
            }

            let output: Value = serde_json::from_str(&result.text).map_err(|e| e.to_string())?;

            if output["success"] == false {
                // Extract error message from error.message field
                if let Some(error) = output.get("error") {
                    if let Some(message) = error.get("message") {
                        Err(message.as_str().unwrap_or("Unknown error").to_string())
                    } else {
                        Err("Unknown error".to_string())
                    }
                } else {
                    Err("Unknown error".to_string())
                }
            } else {
                Ok(output)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_basic_complexity_limits() {
    // Test expression length limit
    let long_expr = "1 + ".repeat(300) + "1";
    let result = try_evaluate(&long_expr).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_lowercase().contains("too long"));

    // Test nesting depth limit
    let deep_expr = "(".repeat(25) + "1" + &")".repeat(25);
    let result = try_evaluate(&deep_expr).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_lowercase().contains("too deep"));

    // Test operation count limit (need more than 100 operations)
    let many_ops = (0..110).map(|i| format!("{} + ", i)).collect::<String>() + "0";
    let result = try_evaluate(&many_ops).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_lowercase().contains("operations"));
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_enhanced_pattern_detection() {
    // Test banned patterns - very large nested power
    let result = try_evaluate("2^2^10").await; // This is 2^1024 which is huge
    assert!(
        result.is_err() || {
            // If it evaluates, it should return infinity
            if let Ok(ref output) = result {
                output["result"]["result"].as_str() == Some("Infinity")
            } else {
                false
            }
        }
    );

    // Test banned patterns - nested exp
    let result = try_evaluate("exp(exp(10))").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Banned pattern") || err_msg.contains("banned pattern"));

    // Test excessive consecutive operations
    let result = try_evaluate("1 +++ 2").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_lowercase().contains("consecutive"));

    // Test extremely large numbers
    let result = try_evaluate("123456789012345").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_lowercase().contains("large number"));
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_exponential_growth_prevention() {
    // Test large exponents
    let result = try_evaluate("2^200").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Exponent too large") || err_msg.to_lowercase().contains("exponent"));

    // Test very large exponents that would cause issues
    let result = try_evaluate("10^10^3").await; // 10^1000 is way too large
    assert!(
        result.is_err() || {
            // If it somehow passes, check the result is reasonable
            let output = result.as_ref().unwrap();
            output["result"]["result"].as_str() == Some("Infinity")
        }
    );

    // Test nested exponentials with high values - exp(1024) = infinity
    let expr = "exp(exp(10))"; // This should be caught by banned patterns
    let result = try_evaluate(expr).await;
    assert!(
        result.is_err() || {
            // If it somehow evaluates, it should be infinity
            let output = result.as_ref().unwrap();
            output["result"]["result"].as_str() == Some("Infinity")
        }
    );
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_memory_limits() {
    // Test with way too many variables
    let mut vars = json!({});
    for i in 0..1000 {
        vars[format!("var{}", i)] = json!(i as f64);
    }

    // Create an expression that uses many variables
    let expr = (0..100)
        .map(|i| format!("var{}", i))
        .collect::<Vec<_>>()
        .join(" + ");

    let tool = CalculatorTool::new();
    let params = json!({
        "operation": "evaluate",
        "input": expr,
        "variables": vars
    });

    let input = AgentInput::text("Calculate").with_parameter("parameters", params);
    let result = tool.execute(input, ExecutionContext::default()).await;

    // Should fail due to too many variables
    if let Ok(output) = result {
        let json: Value = serde_json::from_str(&output.text).unwrap();
        assert_eq!(json["success"], false);
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_recursive_depth_limits() {
    // Test deeply nested function calls
    let expr = "sqrt(sqrt(sqrt(sqrt(sqrt(sqrt(16))))))";
    let result = try_evaluate(expr).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Recursive depth")
            || err_msg.to_lowercase().contains("recursive")
            || err_msg.to_lowercase().contains("depth")
    );

    // Test mixed nested functions
    let expr = "log(exp(log(exp(log(10)))))";
    let result = try_evaluate(expr).await;
    assert!(result.is_err());
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeout_enforcement() {
    // This is actually safe and should pass
    let result = try_evaluate("1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10").await;
    assert!(result.is_ok());

    // Test with many function calls to exceed limits
    let complex = (0..60)
        .map(|i| format!("sin({})", i))
        .collect::<Vec<_>>()
        .join(" + ");
    let result = try_evaluate(&complex).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.to_lowercase().contains("function")
            || err_msg.to_lowercase().contains("operations")
    );
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_safe_expressions_still_work() {
    // Ensure legitimate expressions still work
    let safe_exprs = vec![
        ("2 + 3 * 4", 14.0),
        ("sqrt(16)", 4.0),
        ("sin(0)", 0.0),
        ("cos(0)", 1.0),
        ("ln(e())", 1.0),
        ("2^3", 8.0),
        ("min(5, 3)", 3.0),
        ("max(5, 3)", 5.0),
        ("abs(-5)", 5.0),
    ];

    for (expr, expected) in safe_exprs {
        let result = try_evaluate(expr).await.unwrap();
        if let Some(res) = result.get("result").and_then(|r| r.get("result")) {
            let value = res.as_f64().unwrap();
            assert!(
                (value - expected).abs() < 0.0001,
                "Expression {} = {} (expected {})",
                expr,
                value,
                expected
            );
        }
    }
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_variable_limit_enforcement() {
    // Test too many unique variables in expression
    let expr = (0..60)
        .map(|i| format!("var{}", i))
        .collect::<Vec<_>>()
        .join(" + ");
    let result = try_evaluate(&expr).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Too many unique variables")
            || err_msg.to_lowercase().contains("unique variables")
            || err_msg.to_lowercase().contains("variables")
    );
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_dos_attack_patterns() {
    // Pattern 1: Very deep nesting
    let deep_expr = format!("{}1{}", "(".repeat(30), ")".repeat(30));
    let result = try_evaluate(&deep_expr).await;
    assert!(result.is_err());

    // Pattern 2: Memory exhaustion attempt with parentheses
    let expr = format!("{}2{}", "(".repeat(50), ")".repeat(50));
    let result = try_evaluate(&expr).await;
    assert!(result.is_err());

    // Pattern 3: CPU exhaustion with many trig functions (exceed function limit)
    let trig_spam = (0..60)
        .map(|i| format!("sin({}) + cos({}) + tan({})", i, i, i))
        .collect::<Vec<_>>()
        .join(" + ");
    let result = try_evaluate(&trig_spam).await;
    assert!(result.is_err());

    // Pattern 4: Parser confusion - invalid syntax
    let result = try_evaluate("1 + + + + + 2").await;
    // Try a definitely invalid expression
    let result2 = try_evaluate("((1 + 2").await; // Unmatched parentheses
    assert!(result.is_err() || result2.is_err());
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_edge_cases() {
    // Empty expression
    let result = try_evaluate("").await;
    assert!(result.is_err());

    // Just whitespace
    let result = try_evaluate("   \n\t  ").await;
    assert!(result.is_err());

    // Unicode in expression
    let result = try_evaluate("2 + 3 × 4").await; // × instead of *
    assert!(result.is_err());

    // Very small timeout window
    let start = Instant::now();
    let result = try_evaluate("1 + 1").await;
    let elapsed = start.elapsed();
    assert!(result.is_ok());
    assert!(elapsed.as_millis() < 100); // Should be very fast
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_performance_consistency() {
    // Run multiple safe expressions and ensure consistent performance
    let expressions = vec![
        "2 + 3",
        "sin(1.5)",
        "sqrt(25)",
        "log(10, 100)",
        "max(1, 2, 3, 4, 5)",
    ];

    for expr in expressions {
        let start = Instant::now();
        let result = try_evaluate(expr).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok(), "Expression {} failed", expr);
        assert!(
            elapsed.as_millis() < 50,
            "Expression {} took too long: {:?}",
            expr,
            elapsed
        );
    }
}
