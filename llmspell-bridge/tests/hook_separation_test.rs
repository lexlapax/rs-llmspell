//! Test that llmspell-hooks (Hook global) work independently of debug hooks
//!
//! llmspell-hooks use normal Lua functions, NOT debug hooks, so they should
//! work regardless of debug hook state.

use llmspell_bridge::{
    execution_bridge::ExecutionManager,
    execution_context::SharedExecutionContext,
    globals::GlobalContext,
    hook_bridge::HookBridge,
    lua::globals::{execution::install_interactive_debug_hooks, hook::inject_hook_global},
    ComponentRegistry, ProviderManager,
};
use llmspell_config::providers::ProviderManagerConfig;
use mlua::Lua;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test that Hook.register works with debug hooks active
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_llmspell_hooks_work_with_debug_hooks() {
    let lua = Lua::new();

    // Set up llmspell-hooks system
    let context = Arc::new(GlobalContext::new(
        Arc::new(ComponentRegistry::new()),
        Arc::new(
            ProviderManager::new(ProviderManagerConfig::default())
                .await
                .unwrap(),
        ),
    ));
    let hook_bridge = Arc::new(HookBridge::new(context.clone()).unwrap());

    // Inject Hook global
    inject_hook_global(&lua, &context, hook_bridge).unwrap();

    // Register a hook using Hook.register
    let _hook_called = Arc::new(std::sync::atomic::AtomicBool::new(false));

    lua.load(
        r#"
        -- Register a hook for system startup
        local handle = Hook.register("SystemStartup", function(context)
            -- Mark that hook was called
            _G.hook_was_called = true
            return "continue"
        end, "normal")
        
        -- Store handle globally for cleanup
        _G.hook_handle = handle
    "#,
    )
    .exec()
    .unwrap();

    // Now install debug hooks
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    let _debug_hook =
        install_interactive_debug_hooks(&lua, execution_manager, shared_context).unwrap();

    // llmspell hooks should still work - they don't use debug hooks
    lua.load(
        r#"
        -- This should work regardless of debug hooks
        local hooks = Hook.list("SystemStartup")
        assert(#hooks > 0, "Hook should be registered")
        
        -- Unregister
        Hook.unregister(_G.hook_handle)
        
        -- Verify unregistered
        hooks = Hook.list("SystemStartup")
        assert(#hooks == 0, "Hook should be unregistered")
    "#,
    )
    .exec()
    .unwrap();
}

/// Test that both hook systems can be used in the same script
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_both_hook_systems_in_same_script() {
    let lua = Lua::new();

    // Set up llmspell-hooks
    let context = Arc::new(GlobalContext::new(
        Arc::new(ComponentRegistry::new()),
        Arc::new(
            ProviderManager::new(ProviderManagerConfig::default())
                .await
                .unwrap(),
        ),
    ));
    let hook_bridge = Arc::new(HookBridge::new(context.clone()).unwrap());
    inject_hook_global(&lua, &context, hook_bridge).unwrap();

    // Set up debug hooks
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Add a breakpoint for testing
    let bp = llmspell_bridge::execution_bridge::Breakpoint::new("test.lua".to_string(), 5);
    execution_manager.add_breakpoint(bp).await;

    let debug_hook =
        install_interactive_debug_hooks(&lua, execution_manager.clone(), shared_context.clone())
            .unwrap();

    // Switch to Full mode
    llmspell_bridge::lua::globals::execution::update_debug_mode(
        &lua,
        &debug_hook,
        llmspell_bridge::lua::debug_cache::DebugMode::Full,
    )
    .unwrap();

    // Both systems should work in the same script
    lua.load(
        r#"
        -- llmspell-hooks work
        local handle = Hook.register("BeforeToolExecution", function(ctx)
            return "continue"
        end)
        
        -- Code that would trigger debug hooks
        local x = 1
        local y = 2
        local z = x + y
        
        -- llmspell-hooks still work
        local hooks = Hook.list()
        assert(#hooks > 0, "llmspell hooks should exist")
        
        -- Clean up
        Hook.unregister(handle)
    "#,
    )
    .set_name("test.lua")
    .exec()
    .unwrap();

    // Verify debug hooks tracked execution
    debug_hook.lock().flush_batched_context_updates();
    {
        let ctx = shared_context.read().await;
        assert!(
            ctx.location.is_some(),
            "Debug hooks should have tracked execution"
        );
        drop(ctx); // Early drop to avoid resource contention
    }
}

/// Test that llmspell-hooks performance is not affected by debug hooks
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_llmspell_hooks_performance() {
    let lua = Lua::new();

    // Set up llmspell-hooks
    let context = Arc::new(GlobalContext::new(
        Arc::new(ComponentRegistry::new()),
        Arc::new(
            ProviderManager::new(ProviderManagerConfig::default())
                .await
                .unwrap(),
        ),
    ));
    let hook_bridge = Arc::new(HookBridge::new(context.clone()).unwrap());
    inject_hook_global(&lua, &context, hook_bridge).unwrap();

    // Measure hook registration performance without debug hooks
    let start = std::time::Instant::now();
    lua.load(
        r#"
        for i = 1, 100 do
            local h = Hook.register("SystemStartup", function() return "continue" end)
            Hook.unregister(h)
        end
    "#,
    )
    .exec()
    .unwrap();
    let without_debug = start.elapsed();

    // Install debug hooks
    let execution_manager = Arc::new(ExecutionManager::new());
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let _debug_hook =
        install_interactive_debug_hooks(&lua, execution_manager, shared_context).unwrap();

    // Measure again with debug hooks
    let start = std::time::Instant::now();
    lua.load(
        r#"
        for i = 1, 100 do
            local h = Hook.register("SystemStartup", function() return "continue" end)
            Hook.unregister(h)
        end
    "#,
    )
    .exec()
    .unwrap();
    let with_debug = start.elapsed();

    println!("llmspell-hooks performance:");
    println!("  Without debug hooks: {without_debug:?}");
    println!("  With debug hooks: {with_debug:?}");

    // Performance should be similar - they're independent systems
    let overhead = with_debug.as_secs_f64() / without_debug.as_secs_f64();
    assert!(
        overhead < 2.0,
        "llmspell-hooks should not be significantly affected by debug hooks: {overhead:.2}x"
    );
}
