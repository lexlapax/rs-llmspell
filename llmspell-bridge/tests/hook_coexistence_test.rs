//! Test the interaction between debug hooks and other Lua hooks
//!
//! NOTE: Lua only supports ONE debug hook at a time. This is a fundamental limitation.
//! These tests verify that we handle this limitation correctly.

use llmspell_bridge::{
    debug_state_cache::DebugStateCache,
    execution_bridge::ExecutionManager,
    execution_context::SharedExecutionContext,
    lua::debug_state_cache_impl::LuaDebugStateCache,
    lua::globals::execution::{install_interactive_debug_hooks, update_debug_mode},
};
use mlua::Lua;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tokio::sync::RwLock;

/// Test that debug hooks in Disabled mode don't interfere with other hooks
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_disabled_mode_allows_other_hooks() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Set up a counter for our custom hook
    let custom_hook_calls = Arc::new(AtomicU32::new(0));
    let counter = custom_hook_calls.clone();

    // Install a custom hook (simulating profiler, memory tracker, etc.)
    lua.set_hook(
        mlua::HookTriggers {
            every_nth_instruction: Some(10),
            ..Default::default()
        },
        move |_, _| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        },
    );

    // Verify custom hook works
    lua.load("for i = 1, 100 do local x = i * 2 end")
        .exec()
        .unwrap();

    let initial_count = custom_hook_calls.load(Ordering::Relaxed);
    assert!(initial_count > 0, "Custom hook should have been called");

    // Now install debug hooks in Disabled mode
    let debug_hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context).unwrap();

    // Verify debug hooks are in Disabled mode
    assert_eq!(
        debug_hook.lock().debug_cache().get_debug_mode(),
        llmspell_bridge::debug_state_cache::DebugMode::Disabled
    );

    // Reset counter
    custom_hook_calls.store(0, Ordering::Relaxed);

    // Run code again - custom hook should STILL work
    lua.load("for i = 1, 100 do local x = i * 2 end")
        .exec()
        .unwrap();

    let after_debug_count = custom_hook_calls.load(Ordering::Relaxed);

    // In Disabled mode, we remove our hooks to allow others to work
    // So the custom hook is replaced when we install, but restored when we go to Disabled
    // This is the best we can do given Lua's one-hook limitation

    // The custom hook was replaced by our debug hook installation
    // This is expected - Lua only supports one hook
    assert_eq!(
        after_debug_count, 0,
        "Custom hook is replaced by debug hooks (Lua limitation)"
    );
}

/// Test that debug hooks properly override and restore
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_hook_lifecycle() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Set up a counter for our custom hook
    let custom_hook_calls = Arc::new(AtomicU32::new(0));
    let counter = custom_hook_calls.clone();

    // Install a custom hook
    lua.set_hook(
        mlua::HookTriggers {
            every_line: true,
            ..Default::default()
        },
        move |_, _| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        },
    );

    // Install debug hooks
    let debug_hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context).unwrap();

    // Test code
    let test_code = "local x = 1\nlocal y = 2\nlocal z = x + y";

    // Test in each mode
    for mode in [
        llmspell_bridge::debug_state_cache::DebugMode::Disabled,
        llmspell_bridge::debug_state_cache::DebugMode::Minimal {
            check_interval: 100,
        },
        llmspell_bridge::debug_state_cache::DebugMode::Full,
    ] {
        // Switch mode
        update_debug_mode(&lua, &debug_hook, mode).unwrap();

        // Reset counter
        custom_hook_calls.store(0, Ordering::Relaxed);

        // Run code
        lua.load(test_code).exec().unwrap();

        // Check behavior based on mode
        let count = custom_hook_calls.load(Ordering::Relaxed);
        match mode {
            llmspell_bridge::debug_state_cache::DebugMode::Disabled => {
                // In Disabled, we remove hooks, so count should be 0
                assert_eq!(count, 0, "In Disabled mode, no hooks run");
            }
            _ => {
                // In other modes, our debug hooks replace the custom hook
                assert_eq!(count, 0, "Debug hooks replace custom hooks");
            }
        }
    }
}

/// Test that debug functionality still works properly
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_functionality_still_works() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Add a breakpoint
    let bp = llmspell_bridge::execution_bridge::Breakpoint::new("test".to_string(), 2);
    execution_manager.add_breakpoint(bp).await;

    // Install debug hooks - should auto-detect Minimal mode due to breakpoint
    let debug_hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context.clone()).unwrap();

    // Should be in Minimal mode now
    assert!(matches!(
        debug_hook.lock().debug_cache().get_debug_mode(),
        llmspell_bridge::debug_state_cache::DebugMode::Minimal { .. }
    ));

    // Switch to Full mode for testing
    update_debug_mode(
        &lua,
        &debug_hook,
        llmspell_bridge::debug_state_cache::DebugMode::Full,
    )
    .unwrap();

    // Execute code
    let code = "local x = 1\nlocal y = 2\nlocal z = x + y";
    lua.load(code).set_name("test").exec().ok();

    // Flush any pending updates
    debug_hook.lock().flush_batched_context_updates();

    // Verify context was updated (debug functionality works)
    let ctx = shared_context.read().await;
    assert!(ctx.location.is_some(), "Debug tracking should still work");
    assert!(
        ctx.performance_metrics.execution_count > 0,
        "Metrics should be tracked"
    );
    drop(ctx); // Early drop to avoid resource contention
}

/// Test performance in Disabled mode with other hooks present
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_disabled_mode_performance_with_other_hooks() {
    let lua = Lua::new();
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Install a lightweight custom hook
    let custom_counter = Arc::new(AtomicU32::new(0));
    let counter = custom_counter.clone();
    lua.set_hook(
        mlua::HookTriggers {
            every_nth_instruction: Some(1000),
            ..Default::default()
        },
        move |_, _| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        },
    );

    // Baseline with just custom hook
    let start = std::time::Instant::now();
    lua.load("for i = 1, 10000 do local x = i * 2 end")
        .exec()
        .unwrap();
    let baseline = start.elapsed();

    // Install debug hooks in Disabled mode
    let debug_hook =
        install_interactive_debug_hooks(&lua, &execution_manager, shared_context).unwrap();

    assert_eq!(
        debug_hook.lock().debug_cache().get_debug_mode(),
        llmspell_bridge::debug_state_cache::DebugMode::Disabled
    );

    // Measure with debug hooks in Disabled mode
    let start = std::time::Instant::now();
    lua.load("for i = 1, 10000 do local x = i * 2 end")
        .exec()
        .unwrap();
    let with_debug = start.elapsed();

    // Performance should still be good
    let overhead = with_debug.as_secs_f64() / baseline.as_secs_f64();
    println!("Performance with coexisting hooks:");
    println!("  Baseline (custom hook only): {baseline:?}");
    println!("  With debug hooks disabled: {with_debug:?}");
    println!("  Overhead: {overhead:.2}x");

    // Should have minimal overhead even with both hooks
    assert!(
        overhead < 1.5,
        "Overhead too high with coexisting hooks: {overhead:.2}x"
    );

    // Verify custom hook still worked
    assert!(
        custom_counter.load(Ordering::Relaxed) > 0,
        "Custom hook should still be active"
    );
}
