//! Tests for watch expressions system (Task 9.2.8)
//!
//! These tests verify that:
//! - Watch expressions are stored correctly in `DebugStateCache`
//! - Evaluation happens ONLY in slow path when paused
//! - Results are cached with generation counter for invalidation
//! - Batch evaluation works efficiently
//! - Performance requirements are met (<10ms for 10 expressions)

use llmspell_bridge::{
    condition_evaluator::SharedDebugContext, debug_state_cache::DebugStateCache,
    execution_context::SharedExecutionContext,
    lua::condition_evaluator_impl::LuaConditionEvaluator,
    lua::debug_state_cache_impl::LuaDebugStateCache,
};
use mlua::Lua;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Test basic watch expression storage and retrieval
#[test]
fn test_watch_expression_storage() {
    let cache = LuaDebugStateCache::new();

    // Add some watch expressions
    let watch_id1 = cache.add_watch("x > 10".to_string());
    let watch_id2 = cache.add_watch("y + z".to_string());

    // Check that watch IDs are generated correctly
    assert!(watch_id1.starts_with("watch_"));
    assert!(watch_id2.starts_with("watch_"));
    assert_ne!(watch_id1, watch_id2); // Should be unique

    // Check that expressions are stored
    let expressions = cache.get_watch_expressions();
    assert_eq!(expressions.len(), 2);
    assert!(expressions.contains(&"x > 10".to_string()));
    assert!(expressions.contains(&"y + z".to_string()));

    // Test removal
    assert!(cache.remove_watch("x > 10"));
    assert!(!cache.remove_watch("nonexistent"));

    let expressions = cache.get_watch_expressions();
    assert_eq!(expressions.len(), 1);
    assert!(expressions.contains(&"y + z".to_string()));
}

/// Test watch expression caching with generation counter
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_expression_caching() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up context variables
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("x".to_string(), json!(15));
        ctx.variables.insert("y".to_string(), json!(25));
        drop(ctx); // Explicitly drop to release lock
    }

    // Add watch expressions
    cache.add_watch("x > 10".to_string());
    cache.add_watch("y < 20".to_string());

    let debug_context = SharedDebugContext::new(shared_context.clone());

    // First evaluation - should compute results
    let results1 = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    assert_eq!(results1.len(), 2);
    assert_eq!(results1.get("x > 10"), Some(&"true".to_string()));
    assert_eq!(results1.get("y < 20"), Some(&"false".to_string()));

    // Second evaluation - should use cached results
    let results2 = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    assert_eq!(results2, results1); // Should be identical

    // Verify cached results are available
    assert_eq!(cache.get_watch_result("x > 10"), Some("true".to_string()));
    assert_eq!(cache.get_watch_result("y < 20"), Some("false".to_string()));

    // Invalidate cache and verify results are cleared
    cache.invalidate_condition_cache();
    assert_eq!(cache.get_watch_result("x > 10"), None);
    assert_eq!(cache.get_watch_result("y < 20"), None);
}

/// Test batch evaluation of multiple watch expressions
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_batch_watch_evaluation() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up comprehensive context
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("a".to_string(), json!(5));
        ctx.variables.insert("b".to_string(), json!(10));
        ctx.variables.insert("c".to_string(), json!(15));
        ctx.variables.insert("name".to_string(), json!("test"));
        ctx.variables.insert("enabled".to_string(), json!(true));
        drop(ctx);
    }

    // Add multiple watch expressions with different types and complexity
    let expressions = vec![
        "a > 3",           // Simple comparison
        "b * 2 == 20",     // Arithmetic
        "c < 20",          // Another comparison
        "enabled == true", // Boolean comparison
        "a + b + c",       // Multi-variable arithmetic
    ];

    for expr in &expressions {
        cache.add_watch((*expr).to_string());
    }

    let debug_context = SharedDebugContext::new(shared_context);

    // Evaluate all watches in batch
    let start_time = Instant::now();
    let results = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    let evaluation_time = start_time.elapsed();

    // Verify all expressions were evaluated
    assert_eq!(results.len(), expressions.len());

    // Verify specific results
    assert_eq!(results.get("a > 3"), Some(&"true".to_string()));
    assert_eq!(results.get("b * 2 == 20"), Some(&"true".to_string()));
    assert_eq!(results.get("c < 20"), Some(&"true".to_string()));
    assert_eq!(results.get("enabled == true"), Some(&"true".to_string()));
    // Note: "a + b + c" evaluates to a number, which is truthy, so "true"
    assert_eq!(results.get("a + b + c"), Some(&"true".to_string()));

    // Performance check: should be fast for batch evaluation
    assert!(
        evaluation_time.as_millis() < 50,
        "Batch evaluation too slow: {evaluation_time:?}"
    );

    // Verify all results are cached
    for expr in &expressions {
        assert!(cache.get_watch_result(expr).is_some());
    }
}

/// Test watch expression error handling
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_expression_error_handling() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Add expressions that will cause errors
    cache.add_watch("undefined_variable > 10".to_string());
    cache.add_watch("invalid_syntax >>>".to_string());
    cache.add_watch("nil.field".to_string());

    let debug_context = SharedDebugContext::new(shared_context);

    // Evaluate expressions with errors
    let results = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);

    assert_eq!(results.len(), 3);

    // All should have error messages
    for (expr, result) in results {
        assert!(
            result.starts_with("<error:"),
            "Expression '{expr}' should have error result, got '{result}'"
        );
    }
}

/// Test performance requirements for watch expressions
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_expression_performance() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up variables for performance test
    {
        let mut ctx = shared_context.write().await;
        for i in 0..20 {
            ctx.variables.insert(format!("var_{i}"), json!(i * 2));
        }
        drop(ctx);
    }

    // Add 10 watch expressions (requirement: <10ms for 10 expressions)
    for i in 0..10 {
        cache.add_watch(format!("var_{i} > {i}"));
    }

    let debug_context = SharedDebugContext::new(shared_context);

    // Measure evaluation time
    let start_time = Instant::now();
    let results = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    let evaluation_time = start_time.elapsed();

    // Verify results
    assert_eq!(results.len(), 10);

    // Performance requirement: <10ms for 10 watch expressions
    assert!(
        evaluation_time.as_millis() < 10,
        "Watch expression evaluation too slow: {evaluation_time:?} (requirement: <10ms)"
    );

    // Test cached performance (should be much faster)
    let start_time = Instant::now();
    let cached_results = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    let cached_evaluation_time = start_time.elapsed();

    assert_eq!(cached_results, results);
    assert!(
        cached_evaluation_time.as_millis() < 2,
        "Cached evaluation too slow: {cached_evaluation_time:?}"
    );
}

/// Test cache invalidation on context changes
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_invalidation_on_context_change() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Initial context
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("value".to_string(), json!(5));
        drop(ctx);
    }

    cache.add_watch("value > 3".to_string());
    let debug_context = SharedDebugContext::new(shared_context.clone());

    // First evaluation
    let results1 = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    assert_eq!(results1.get("value > 3"), Some(&"true".to_string()));

    // Change context
    {
        let mut ctx = shared_context.write().await;
        ctx.variables.insert("value".to_string(), json!(1));
        drop(ctx);
    }

    // Invalidate cache (simulating what happens on context change)
    cache.invalidate_condition_cache();

    // Second evaluation should give different result
    let debug_context = SharedDebugContext::new(shared_context);
    let results2 = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);
    assert_eq!(results2.get("value > 3"), Some(&"false".to_string()));

    // Results should be different
    assert_ne!(results1, results2);
}

/// Test watch expressions with complex data types
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_complex_watch_expressions() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let evaluator = LuaConditionEvaluator::new();
    let lua = Lua::new();

    // Set up complex variables
    {
        let mut ctx = shared_context.write().await;
        ctx.variables
            .insert("numbers".to_string(), json!([1, 2, 3, 4, 5]));
        ctx.variables.insert(
            "config".to_string(),
            json!({
                "enabled": true,
                "max_count": 100,
                "name": "test_config"
            }),
        );
        ctx.variables
            .insert("text".to_string(), json!("hello world"));
        drop(ctx);
    }

    // Add complex watch expressions (Note: these evaluate as Lua conditions, so they return boolean)
    cache.add_watch("text".to_string()); // String is truthy
    cache.add_watch("numbers".to_string()); // Array is truthy
    cache.add_watch("config".to_string()); // Object is truthy

    let debug_context = SharedDebugContext::new(shared_context);
    let results = cache.evaluate_watches_with_lua(&lua, &debug_context, &evaluator);

    // All complex types should be truthy
    assert_eq!(results.get("text"), Some(&"true".to_string()));
    assert_eq!(results.get("numbers"), Some(&"true".to_string()));
    assert_eq!(results.get("config"), Some(&"true".to_string()));
}

/// Test clearing watch expressions
#[test]
fn test_clear_watch_expressions() {
    let cache = LuaDebugStateCache::new();

    // Add expressions
    cache.add_watch("x > 10".to_string());
    cache.add_watch("y < 5".to_string());

    assert_eq!(cache.get_watch_expressions().len(), 2);

    // Clear should remove all expressions
    cache.clear();
    assert_eq!(cache.get_watch_expressions().len(), 0);
    assert_eq!(cache.get_all_watch_results().len(), 0);
}
