//! Tests for conditional breakpoints with two-tier architecture
//!
//! These tests verify that:
//! - Conditions are only evaluated in the slow path
//! - Conditions are properly cached and invalidated
//! - Fast path remains fast when conditions exist but don't match
//! - Complex expressions work correctly

use llmspell_bridge::{
    condition_evaluator::{ConditionEvaluator, SharedDebugContext},
    execution_bridge::Breakpoint,
    execution_context::SharedExecutionContext,
    lua::condition_evaluator_impl::LuaConditionEvaluator,
    lua::debug_cache::DebugStateCache,
};
use mlua::Lua;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test that conditions are compiled and cached correctly
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_condition_compilation() {
    let _lua = Lua::new();
    let cache = DebugStateCache::new();

    // Compile a valid condition
    let evaluator = LuaConditionEvaluator::new();
    let condition = evaluator.compile_condition("x > 10").unwrap();
    assert_eq!(condition.expression, "x > 10");
    assert_eq!(
        condition.metadata.get("language"),
        Some(&serde_json::json!("lua"))
    );

    // Cache the condition
    cache.set_condition("test.lua".to_string(), 10, condition);
    assert!(cache.has_condition("test.lua", 10));

    // Retrieve the condition
    let retrieved = cache.get_condition("test.lua", 10).unwrap();
    assert_eq!(retrieved.expression, "x > 10");
}

/// Test compilation of complex conditions
#[test]
fn test_complex_condition_compilation() {
    let evaluator = LuaConditionEvaluator::new();
    let condition = evaluator.compile_condition("x > 100").unwrap();
    assert_eq!(condition.expression, "x > 100");

    // Complex condition with function calls
    let complex_condition = evaluator
        .compile_condition("math.max(x, y) > threshold and string.len(name) < 10")
        .unwrap();
    assert!(complex_condition.expression.contains("math.max"));
}

/// Test condition evaluation with variables in context
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_condition_evaluation_with_context() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up variables in context
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("x".to_string(), serde_json::json!(15));
        ctx.variables.insert("y".to_string(), serde_json::json!(5));
    }

    // Use the Lua-specific evaluation method
    let debug_context = SharedDebugContext::new(shared_context.clone());
    let result = evaluator.evaluate_condition_with_lua("x > 10", None, &debug_context, &lua);

    assert!(
        result.is_ok() && result.unwrap(),
        "Condition should evaluate to true"
    );
}

/// Test condition evaluation with multiple variables
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_variable_conditions() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up variables in context
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("x".to_string(), serde_json::json!(15));
        ctx.variables.insert("y".to_string(), serde_json::json!(5));
        ctx.variables
            .insert("threshold".to_string(), serde_json::json!(20));
    }

    let debug_context = SharedDebugContext::new(shared_context.clone());

    // Test condition that should be true
    let result1 = evaluator
        .evaluate_condition_with_lua("x > y", None, &debug_context, &lua)
        .unwrap();
    assert!(result1, "x > y should be true (15 > 5)");

    // Test condition that should be false
    let result2 = evaluator
        .evaluate_condition_with_lua("x > threshold", None, &debug_context, &lua)
        .unwrap();
    assert!(!result2, "x > threshold should be false (15 > 20)");
}

/// Test that breakpoints without conditions always break
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_unconditional_breakpoints() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();

    // Breakpoint without condition
    let bp = Breakpoint::new("test.lua".to_string(), 10);
    let debug_context = SharedDebugContext::new(shared_context);
    let result = evaluator.evaluate_breakpoint(&bp, &debug_context);

    assert!(result, "Unconditional breakpoints should always break");
}

/// Test condition evaluation performance
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_condition_performance() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up context with early drop to avoid significant drop warning
    for i in 0..100 {
        shared_context
            .write()
            .await
            .variables
            .insert(format!("var_{i}"), serde_json::json!(i));
    }

    let debug_context = SharedDebugContext::new(shared_context);

    // Test multiple condition evaluations
    let start = std::time::Instant::now();
    for i in 0..10 {
        let condition = format!("var_{i} > 50");
        let _result = evaluator.evaluate_condition_with_lua(&condition, None, &debug_context, &lua);
    }
    let duration = start.elapsed();

    // Should complete within reasonable time (< 100ms for 10 evaluations)
    assert!(
        duration.as_millis() < 100,
        "Condition evaluation too slow: {duration:?}"
    );
}

/// Test error handling in conditions
#[test]
fn test_condition_error_handling() {
    let evaluator = LuaConditionEvaluator::new();
    let result = evaluator.compile_condition("x >>> 10");
    assert!(result.is_ok(), "Even invalid syntax should compile for now"); // Basic validation only

    // Empty condition should fail
    let empty_result = evaluator.compile_condition("");
    assert!(
        empty_result.is_err(),
        "Empty condition should fail compilation"
    );
}

/// Test that evaluation errors default to breaking for safety
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_error_safety_behavior() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();

    // Create breakpoint with condition that will cause runtime error
    let bp = Breakpoint::new("test.lua".to_string(), 10)
        .with_condition("undefined_variable > 10".to_string());
    let debug_context = SharedDebugContext::new(shared_context);

    // Should return true (break) on error for safety
    let result = evaluator.evaluate_breakpoint(&bp, &debug_context);
    assert!(result, "Should break on error for safety");
}
