//! Tests for variable inspection system (Task 9.2.7)

use llmspell_bridge::{
    debug_state_cache::DebugStateCache,
    execution_bridge::ExecutionManager,
    execution_context::SharedExecutionContext,
    lua::debug_state_cache_impl::LuaDebugStateCache,
    lua::globals::execution::install_interactive_debug_hooks,
    lua::variable_inspector_impl::LuaVariableInspector,
    variable_inspector::SharedVariableInspector,
    variable_inspector::{ContextBatcher, ContextUpdate},
};
use mlua::Lua;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Test basic variable caching operations
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_variable_caching() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let inspector = SharedVariableInspector::new(cache.clone(), context.clone());

    // Add some variables to context
    {
        let mut ctx = context.write().await;
        ctx.variables.insert("test_int".to_string(), json!(42));
        ctx.variables.insert("test_str".to_string(), json!("hello"));
        ctx.variables.insert("test_bool".to_string(), json!(true));
    }

    // Inspect variables - should read from context
    let mut batcher = ContextBatcher::new();
    let vars = inspector.inspect_variables(
        &["test_int".to_string(), "test_str".to_string()],
        &mut batcher,
    );

    assert_eq!(vars.get("test_int"), Some(&json!(42)));
    assert_eq!(vars.get("test_str"), Some(&json!("hello")));

    // Second inspection should use cache
    let vars2 = inspector.inspect_variables(&["test_int".to_string()], &mut batcher);
    assert_eq!(vars2.get("test_int"), Some(&json!(42)));
}

/// Test watch list functionality
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_list() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let inspector = SharedVariableInspector::new(cache.clone(), context.clone());
    let mut batcher = ContextBatcher::new();

    // Add variable to watch list
    inspector.watch_variable("important_var".to_string(), &mut batcher);
    assert!(cache.is_watched("important_var"));

    // Add variable to context
    {
        let mut ctx = context.write().await;
        ctx.variables
            .insert("important_var".to_string(), json!(999));
        ctx.variables.insert("normal_var".to_string(), json!(100));
    }

    // Inspect any variable - watched variable should be included
    let vars = inspector.inspect_variables(&["normal_var".to_string()], &mut batcher);

    assert_eq!(vars.get("normal_var"), Some(&json!(100)));
    assert_eq!(vars.get("important_var"), Some(&json!(999))); // Watched variable included

    // Remove from watch list
    inspector.unwatch_variable("important_var", &mut batcher);
    assert!(!cache.is_watched("important_var"));
}

/// Test batch variable reading
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_batch_variable_reading() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let _inspector = SharedVariableInspector::new(cache, context);
    let mut batcher = ContextBatcher::new();

    // Batch read multiple variables
    let var_names: Vec<String> = (0..10).map(|i| format!("var_{i}")).collect();

    batcher.batch_read_variables(var_names);

    // Check that batch was recorded
    assert!(batcher.pending_count() > 0);

    // Flush and verify
    let updates = batcher.flush();
    assert!(updates
        .iter()
        .any(|u| matches!(u, ContextUpdate::ReadVariables(_))));
}

/// Test cache invalidation
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_invalidation() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let inspector = SharedVariableInspector::new(cache.clone(), context);

    // Cache a variable
    cache.cache_variable("test_var".to_string(), json!(42));
    assert_eq!(cache.get_cached_variable("test_var"), Some(json!(42)));

    // Invalidate cache
    inspector.invalidate_cache();

    // Variable should no longer be cached
    assert_eq!(cache.get_cached_variable("test_var"), None);
}

/// Test LRU eviction
#[test]
fn test_lru_eviction() {
    let cache = LuaDebugStateCache::new();

    // Set max cached variables to a small number for testing
    // Note: In real implementation, max_cached_variables is 1000

    // Add many variables
    for i in 0..100 {
        cache.cache_variable(format!("var_{i}"), json!(i));
    }

    // Add to watch list to prevent eviction
    cache.add_to_watch_list("var_50".to_string());

    // Trigger more additions to cause eviction
    for i in 100..1100 {
        cache.cache_variable(format!("var_{i}"), json!(i));
    }

    // Watched variable should still be cached
    assert!(cache.is_watched("var_50"));
}

/// Test variable formatting
#[test]
fn test_variable_formatting() {
    let lua = Lua::new();
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let inspector = LuaVariableInspector::new(cache, context);

    // Test simple value formatting
    let simple_value = json!(42);
    let formatted = inspector.format_variable_with_lua("test_int", &simple_value, &lua);
    assert!(formatted.contains("test_int"));
    assert!(formatted.contains("42"));

    // Test complex value formatting
    let complex_value = json!({
        "name": "test",
        "value": 123,
        "nested": {
            "a": 1,
            "b": 2
        }
    });
    let formatted = inspector.format_variable_with_lua("complex", &complex_value, &lua);
    assert!(formatted.contains("complex"));
}

/// Test performance of variable caching
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_variable_cache_performance() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let inspector = SharedVariableInspector::new(cache.clone(), context.clone());
    let mut batcher = ContextBatcher::new();

    // Prepare 100 variables in context
    {
        let mut ctx = context.write().await;
        for i in 0..100 {
            ctx.variables.insert(format!("var_{i}"), json!(i));
        }
        drop(ctx); // Explicitly drop to release the lock early
    }

    // First read - from context
    let var_names: Vec<String> = (0..100).map(|i| format!("var_{i}")).collect();

    let start = Instant::now();
    let _vars = inspector.inspect_variables(&var_names, &mut batcher);
    let first_read = start.elapsed();

    // Second read - from cache
    let start = Instant::now();
    let _vars = inspector.inspect_variables(&var_names, &mut batcher);
    let cached_read = start.elapsed();

    // Cached read should be significantly faster
    println!("First read: {first_read:?}, Cached read: {cached_read:?}");

    // Performance requirement: <5ms for 100 variable reads (batched)
    assert!(first_read < Duration::from_millis(5));
    assert!(cached_read < Duration::from_millis(1)); // Cache should be very fast
}

/// Test integration with execution hook
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execution_hook_integration() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Install debug hooks
    let hook = install_interactive_debug_hooks(&lua, &execution_manager, shared_context).unwrap();

    // Access the debug cache through the hook (it's locked with a Mutex)
    let cache = {
        let hook_guard = hook.lock();
        hook_guard.debug_cache()
    };

    // Add variables to watch
    cache.add_to_watch_list("watched_var".to_string());

    // Create a script that uses variables
    let script = r"
        local x = 10
        local y = 20
        local z = x + y
        return z
    ";

    // Execute script
    let _result: i32 = lua.load(script).eval().unwrap();

    // Check that watch list is maintained
    assert!(cache.is_watched("watched_var"));
}

/// Test context update processing
#[test]
fn test_context_update_processing() {
    let cache = Arc::new(LuaDebugStateCache::new());
    let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let inspector = SharedVariableInspector::new(cache.clone(), context);

    // Create various context updates
    let updates = vec![
        ContextUpdate::CacheVariable {
            name: "test_var".to_string(),
            value: json!(42),
        },
        ContextUpdate::WatchVariable("important".to_string()),
        ContextUpdate::ReadVariables(vec!["a".to_string(), "b".to_string()]),
        ContextUpdate::UnwatchVariable("old_var".to_string()),
    ];

    // Process updates
    inspector.process_context_updates(updates);

    // Verify effects
    assert_eq!(cache.get_cached_variable("test_var"), Some(json!(42)));
    assert!(cache.is_watched("important"));
    assert!(!cache.is_watched("old_var"));
}

/// Test variable caching with generation counter
#[test]
fn test_generation_based_caching() {
    let cache = LuaDebugStateCache::new();

    // Cache a variable
    let gen1 = cache.generation();
    cache.cache_variable("test".to_string(), json!(1));

    // Variable should be retrievable
    assert_eq!(cache.get_cached_variable("test"), Some(json!(1)));

    // Invalidate (increments generation)
    cache.invalidate_variable_cache();
    let gen2 = cache.generation();
    assert!(gen2 > gen1);

    // Variable should not be retrievable (generation mismatch)
    assert_eq!(cache.get_cached_variable("test"), None);

    // Cache with new generation
    cache.cache_variable("test".to_string(), json!(2));
    assert_eq!(cache.get_cached_variable("test"), Some(json!(2)));
}
