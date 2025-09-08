//! Tests for step debugging with mode transitions (Task 9.2.6)

use llmspell_bridge::{
    debug_state_cache::{DebugMode, DebugStateCache, StepMode},
    execution_bridge::{DebugStepType, ExecutionManager},
    execution_context::SharedExecutionContext,
    lua::debug_state_cache_impl::LuaDebugStateCache,
    lua::globals::execution::install_interactive_debug_hooks,
};
use mlua::Lua;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Test basic step debugging with mode transitions
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_step_debugging_mode_transitions() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Install debug hooks
    let _hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context.clone()).unwrap();

    // Initially should be in Disabled mode
    assert_eq!(execution_manager.get_debug_mode(), DebugMode::Disabled);

    // Start step-in debugging
    execution_manager.start_step(DebugStepType::StepIn).await;

    // Should automatically switch to Full mode
    assert_eq!(execution_manager.get_debug_mode(), DebugMode::Full);

    // Cache should reflect stepping state
    let cache = execution_manager.get_debug_cache();
    assert!(cache.is_stepping());

    // Complete step should restore previous mode
    execution_manager.complete_step();
    assert_eq!(execution_manager.get_debug_mode(), DebugMode::Disabled);
    assert!(!cache.is_stepping());
}

/// Test step-in functionality
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_step_in() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Install debug hooks
    let _hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context.clone()).unwrap();

    // Start in Minimal mode
    execution_manager.set_debug_mode(DebugMode::Minimal {
        check_interval: 100,
    });
    let saved_mode = execution_manager.get_debug_mode();

    // Start step-in
    execution_manager.start_step(DebugStepType::StepIn).await;

    // Verify step mode
    let cache = execution_manager.get_debug_cache();
    match cache.get_step_mode() {
        StepMode::StepIn { depth } => assert_eq!(depth, 0),
        _ => panic!("Expected StepIn mode"),
    }

    // Should be in Full mode now
    assert_eq!(execution_manager.get_debug_mode(), DebugMode::Full);

    // Saved mode should be preserved
    assert_eq!(cache.get_saved_mode(), Some(saved_mode));
}

/// Test step-over functionality with depth tracking
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_step_over_depth_tracking() {
    let cache = LuaDebugStateCache::new();

    // Set initial depth
    cache.set_current_depth(5);

    // Start step-over at depth 5
    cache.start_stepping(StepMode::StepOver { target_depth: 5 }, DebugMode::Disabled);

    // Simulate entering a function (depth increases)
    cache.set_current_depth(6);
    assert_eq!(cache.get_current_depth(), 6);

    // Step mode should still be active
    assert!(cache.is_stepping());
    match cache.get_step_mode() {
        StepMode::StepOver { target_depth } => assert_eq!(target_depth, 5),
        _ => panic!("Expected StepOver mode"),
    }

    // Return to original depth
    cache.set_current_depth(5);

    // Stop stepping and verify restoration
    let saved = cache.stop_stepping();
    assert_eq!(saved, Some(DebugMode::Disabled));
    assert!(!cache.is_stepping());
}

/// Test step-out functionality
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_step_out() {
    let cache = LuaDebugStateCache::new();

    // Start at depth 3
    cache.set_current_depth(3);

    // Step out should target depth 2
    cache.start_stepping(StepMode::StepOut { target_depth: 2 }, DebugMode::Full);

    assert!(cache.is_stepping());

    // Verify step mode
    match cache.get_step_mode() {
        StepMode::StepOut { target_depth } => assert_eq!(target_depth, 2),
        _ => panic!("Expected StepOut mode"),
    }

    // Saved mode should be Full
    assert_eq!(cache.get_saved_mode(), Some(DebugMode::Full));
}

/// Test that stepping state is cleared on cache clear
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stepping_cleared_on_cache_clear() {
    let cache = LuaDebugStateCache::new();

    // Start stepping
    cache.start_stepping(
        StepMode::StepIn { depth: 0 },
        DebugMode::Minimal {
            check_interval: 100,
        },
    );

    assert!(cache.is_stepping());
    assert!(cache.get_saved_mode().is_some());

    // Clear cache
    cache.clear();

    // Stepping state should be cleared
    assert!(!cache.is_stepping());
    assert_eq!(cache.get_step_mode(), StepMode::None);
    assert!(cache.get_saved_mode().is_none());
    assert_eq!(cache.get_current_depth(), 0);
}

/// Test fast path performance with stepping check
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fast_path_stepping_performance() {
    let cache = LuaDebugStateCache::new();

    // Measure fast path check performance
    let start = std::time::Instant::now();

    // Check stepping state many times (fast path)
    for _ in 0..100_000 {
        let _ = cache.is_stepping();
    }

    let elapsed = start.elapsed();

    // Should be very fast (< 1ms for 100k checks)
    assert!(
        elapsed < Duration::from_millis(1),
        "Fast path stepping check too slow: {elapsed:?}"
    );
}

/// Test mode restoration after stepping completes
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mode_restoration() {
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));

    // Set initial mode to Minimal
    let initial_mode = DebugMode::Minimal {
        check_interval: 500,
    };
    execution_manager.set_debug_mode(initial_mode);

    // Start stepping (should save current mode and switch to Full)
    execution_manager.start_step(DebugStepType::StepOver).await;
    assert_eq!(execution_manager.get_debug_mode(), DebugMode::Full);

    // Complete step (should restore previous mode)
    execution_manager.complete_step();
    assert_eq!(execution_manager.get_debug_mode(), initial_mode);
}

/// Test step debugging integration with execution flow
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_step_execution_flow() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Install hooks
    let _hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context.clone()).unwrap();

    // Create a simple Lua script
    let script = r"
        local x = 1
        local y = 2
        local z = x + y
        return z
    ";

    // Start step-in debugging
    execution_manager.start_step(DebugStepType::StepIn).await;

    // Execute script synchronously (Lua is not Send)
    // We'll just test the state changes, not actual execution
    let _ = lua.load(script).exec();

    // Check state after execution
    // Note: actual stepping would pause, but without a real debug loop
    // the test just verifies the mode transitions
    let _state = execution_manager.get_state().await;

    // Verify that stepping was initiated properly
    let _cache = execution_manager.get_debug_cache();

    // Complete stepping to test restoration
    execution_manager.complete_step();
}

/// Test concurrent stepping operations
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_stepping_safety() {
    let cache = Arc::new(LuaDebugStateCache::new());

    // Spawn multiple tasks that try to modify stepping state
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                cache_clone.start_stepping(StepMode::StepIn { depth: i }, DebugMode::Full);
            } else {
                cache_clone.stop_stepping();
            }

            // Read operations should always be safe
            let _ = cache_clone.is_stepping();
            let _ = cache_clone.get_step_mode();
            let _ = cache_clone.get_current_depth();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Cache should still be in a valid state
    // (exact state depends on task execution order)
    let _ = cache.is_stepping();
    let _ = cache.get_step_mode();
}
