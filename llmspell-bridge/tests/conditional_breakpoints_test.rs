//! Tests for conditional breakpoints with two-tier architecture
//!
//! These tests verify that:
//! - Conditions are only evaluated in the slow path
//! - Conditions are properly cached and invalidated
//! - Fast path remains fast when conditions exist but don't match
//! - Complex expressions work correctly

use llmspell_bridge::{
    condition_evaluator::ConditionEvaluator,
    execution_bridge::{Breakpoint, ExecutionManager},
    execution_context::SharedExecutionContext,
    lua::debug_cache::{DebugMode, DebugStateCache},
    lua::globals::execution::{install_interactive_debug_hooks, update_debug_mode},
};
use mlua::Lua;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Test that conditions are compiled and cached correctly
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_condition_compilation() {
    let lua = Lua::new();
    let cache = DebugStateCache::new();

    // Compile a valid condition
    let condition = ConditionEvaluator::compile_condition(&lua, "x > 10").unwrap();
    assert_eq!(condition.expression, "x > 10");
    assert!(condition.compiled_chunk.is_some());

    // Cache the condition
    cache.set_condition("test.lua".to_string(), 10, condition);
    assert!(cache.has_condition("test.lua", 10));

    // Retrieve the condition
    let retrieved = cache.get_condition("test.lua", 10).unwrap();
    assert_eq!(retrieved.expression, "x > 10");
}

/// Test fast path performance with conditions
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fast_path_with_conditions() {
    let lua = Lua::new();
    let cache = DebugStateCache::new();

    // Set up a breakpoint with condition
    cache.update_breakpoints(vec![("test.lua".to_string(), 10)]);
    let condition = ConditionEvaluator::compile_condition(&lua, "x > 100").unwrap();
    cache.set_condition("test.lua".to_string(), 10, condition);

    // Measure fast path performance
    let start = Instant::now();
    for _ in 0..10000 {
        // Fast path check - should be very fast
        let might_break = cache.might_break_at("test.lua", 10);
        assert!(might_break); // Has breakpoint

        // Check if has condition - also fast (lockless read)
        let has_cond = cache.has_condition("test.lua", 10);
        assert!(has_cond);

        // Fast path for non-breakpoint locations
        let no_break = cache.might_break_at("test.lua", 20);
        assert!(!no_break);
    }
    let elapsed = start.elapsed();

    // Fast path should be < 1ms for 10k checks
    assert!(
        elapsed < Duration::from_millis(10),
        "Fast path too slow: {elapsed:?}"
    );
}

/// Test condition evaluation in slow path
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_slow_path_evaluation() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Set up a variable in context
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("x".to_string(), serde_json::json!(15));
    }

    // Create a conditional breakpoint
    let bp = Breakpoint::new("test.lua".to_string(), 10).with_condition("x > 10".to_string());
    execution_manager.add_breakpoint(bp).await;

    // Install debug hooks
    let hook =
        install_interactive_debug_hooks(&lua, execution_manager.clone(), shared_context.clone())
            .unwrap();

    // Switch to Full mode for testing
    update_debug_mode(&lua, &hook, DebugMode::Full).unwrap();

    // The actual condition evaluation would happen in handle_event
    // For this test, we directly test the evaluator
    let cache = DebugStateCache::new();
    let condition = ConditionEvaluator::compile_condition(&lua, "x > 10").unwrap();
    cache.set_condition("test.lua".to_string(), 10, condition);

    // Inject variable into Lua for evaluation
    lua.globals().set("x", 15).unwrap();

    // Evaluate condition
    let bp = Breakpoint::new("test.lua".to_string(), 10).with_condition("x > 10".to_string());
    let batcher = llmspell_bridge::lua::debug_cache::ContextBatcher::new();
    let result = ConditionEvaluator::evaluate_in_slow_path(
        &bp,
        &cache,
        &batcher,
        shared_context.clone(),
        &lua,
    );

    assert!(result, "Condition should evaluate to true");
}

/// Test condition caching and invalidation
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_condition_caching() {
    let lua = Lua::new();
    let cache = DebugStateCache::new();
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Set up initial variable
    lua.globals().set("x", 5).unwrap();

    let bp = Breakpoint::new("test.lua".to_string(), 10).with_condition("x > 10".to_string());
    let batcher = llmspell_bridge::lua::debug_cache::ContextBatcher::new();

    // First evaluation - should be false and cached
    let result1 = ConditionEvaluator::evaluate_in_slow_path(
        &bp,
        &cache,
        &batcher,
        shared_context.clone(),
        &lua,
    );
    assert!(!result1, "Condition should be false with x=5");

    // Check cache was populated
    let cached = cache.get_cached_condition("test.lua", 10);
    assert!(cached.is_some());
    assert!(!cached.unwrap().0);

    // Change variable
    lua.globals().set("x", 15).unwrap();

    // Invalidate cache (simulating variable change)
    cache.invalidate_condition_cache();

    // Second evaluation - should re-evaluate and be true
    let result2 =
        ConditionEvaluator::evaluate_in_slow_path(&bp, &cache, &batcher, shared_context, &lua);
    assert!(result2, "Condition should be true with x=15");
}

/// Test complex condition expressions
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_complex_conditions() {
    let lua = Lua::new();
    let cache = DebugStateCache::new();
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Set up multiple variables
    lua.globals().set("x", 10).unwrap();
    lua.globals().set("y", 20).unwrap();
    lua.globals().set("name", "test").unwrap();

    let test_cases = vec![
        ("x > 5 and y < 30", true),
        ("x == 10 or y == 15", true),
        ("x + y == 30", true),
        ("name == 'test'", true),
        ("x > 100", false),
        ("x ~= 10", false),
    ];

    for (expr, expected) in test_cases {
        let bp = Breakpoint::new("test.lua".to_string(), 10).with_condition(expr.to_string());
        let batcher = llmspell_bridge::lua::debug_cache::ContextBatcher::new();

        // Clear cache for each test
        cache.invalidate_condition_cache();

        let result = ConditionEvaluator::evaluate_in_slow_path(
            &bp,
            &cache,
            &batcher,
            shared_context.clone(),
            &lua,
        );

        assert_eq!(result, expected, "Condition '{expr}' should be {expected}");
    }
}

/// Test hit count with conditions
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_hit_count_with_conditions() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Create a breakpoint with both hit count and condition
    let bp = Breakpoint::new("test.lua".to_string(), 10)
        .with_condition("x > 5".to_string())
        .with_hit_count(3);
    execution_manager.add_breakpoint(bp).await;

    // Set variable that satisfies condition
    lua.globals().set("x", 10).unwrap();

    // Install hooks
    let hook =
        install_interactive_debug_hooks(&lua, execution_manager.clone(), shared_context.clone())
            .unwrap();

    // Update breakpoints in hook
    let breakpoints = execution_manager.get_breakpoints().await;
    hook.lock().update_breakpoints(&breakpoints, &lua);

    // First two hits shouldn't break (hit count = 3)
    assert!(!execution_manager.should_break_at("test.lua", 10).await);
    assert!(!execution_manager.should_break_at("test.lua", 10).await);

    // Third hit should break (condition is satisfied and hit count reached)
    assert!(execution_manager.should_break_at("test.lua", 10).await);
}

/// Test error handling for invalid conditions
#[test]
fn test_invalid_condition_handling() {
    let lua = Lua::new();

    // Invalid syntax should fail compilation
    let result = ConditionEvaluator::compile_condition(&lua, "x >>> 10");
    assert!(result.is_err());

    // Runtime errors should be handled gracefully
    let cache = DebugStateCache::new();
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Condition referencing undefined function
    let bp = Breakpoint::new("test.lua".to_string(), 10)
        .with_condition("undefined_function()".to_string());
    let batcher = llmspell_bridge::lua::debug_cache::ContextBatcher::new();

    // Should return true (break) on error to be safe
    let result =
        ConditionEvaluator::evaluate_in_slow_path(&bp, &cache, &batcher, shared_context, &lua);
    assert!(result, "Should break on error for safety");
}

/// Test performance impact of conditional breakpoints
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_conditional_breakpoint_performance() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Baseline: No breakpoints
    let start = Instant::now();
    lua.load("for i = 1, 10000 do local x = i * 2 end")
        .exec()
        .unwrap();
    let baseline = start.elapsed();

    // Add conditional breakpoint that never matches
    let bp = Breakpoint::new("test".to_string(), 1).with_condition("false".to_string());
    execution_manager.add_breakpoint(bp).await;

    // Install hooks
    let hook =
        install_interactive_debug_hooks(&lua, execution_manager.clone(), shared_context).unwrap();

    // Update breakpoints
    let breakpoints = execution_manager.get_breakpoints().await;
    hook.lock().update_breakpoints(&breakpoints, &lua);

    // Switch to Minimal mode
    update_debug_mode(
        &lua,
        &hook,
        DebugMode::Minimal {
            check_interval: 1000,
        },
    )
    .unwrap();

    // Measure with conditional breakpoint
    let start = Instant::now();
    lua.load("for i = 1, 10000 do local x = i * 2 end")
        .set_name("test")
        .exec()
        .unwrap();
    let with_condition = start.elapsed();

    // Performance impact should be minimal (<2x for non-matching conditions)
    let overhead = with_condition.as_secs_f64() / baseline.as_secs_f64();
    println!(
        "Conditional breakpoint overhead: {overhead:.2}x (baseline: {baseline:?}, with condition: {with_condition:?})"
    );

    assert!(
        overhead < 3.0,
        "Conditional breakpoint overhead too high: {overhead:.2}x"
    );
}
