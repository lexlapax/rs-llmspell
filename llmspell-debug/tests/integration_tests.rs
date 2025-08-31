//! Integration tests for llmspell-debug with llmspell-bridge architecture
//!
//! Tests the complete interactive debugging pipeline using Phase 9.1 architecture

use llmspell_bridge::lua::debug_state_cache_impl::LuaDebugStateCache;
use llmspell_debug::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_bridge_kernel_interactive_debugging_integration() {
    // Create ExecutionManager from bridge (Phase 9.1 architecture)
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));

    // Create SharedExecutionContext
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Create InteractiveDebugger using bridge architecture
    let debugger = InteractiveDebugger::new(execution_manager.clone(), shared_context.clone());

    // Test basic debugging workflow

    // 1. Set a breakpoint
    let bp_id = debugger
        .set_breakpoint("test.lua".to_string(), 10)
        .await
        .unwrap();
    assert!(!bp_id.is_empty());

    // 2. Verify breakpoint is stored in ExecutionManager
    let breakpoints = execution_manager.get_breakpoints().await;
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0].source, "test.lua");
    assert_eq!(breakpoints[0].line, 10);

    // 3. Test conditional breakpoint
    let conditional_bp_id = debugger
        .set_conditional_breakpoint("test.lua".to_string(), 20, "x > 5".to_string())
        .await
        .unwrap();
    assert!(!conditional_bp_id.is_empty());

    // 4. Verify conditional breakpoint
    let breakpoints = execution_manager.get_breakpoints().await;
    assert_eq!(breakpoints.len(), 2);

    // Find the conditional breakpoint
    let conditional_bp = breakpoints
        .iter()
        .find(|bp| bp.line == 20)
        .expect("Conditional breakpoint should exist");
    assert_eq!(conditional_bp.condition, Some("x > 5".to_string()));

    // 5. Test debug session management
    let session_id = debugger
        .create_debug_session("client1".to_string())
        .await
        .unwrap();
    assert!(!session_id.is_empty());

    // 6. Test debug commands
    debugger.continue_execution().await.unwrap();
    let state = debugger.get_debug_state().await;
    assert_eq!(state, DebugState::Running);

    // Step into configures stepping mode but stays Running until code executes
    debugger.step_into().await.unwrap();
    let state = debugger.get_debug_state().await;
    assert_eq!(state, DebugState::Running); // Correctly in Running state, ready to step

    // 7. Test cleanup
    let removed = debugger.remove_breakpoint(&bp_id).await.unwrap();
    assert!(removed);

    let breakpoints = execution_manager.get_breakpoints().await;
    assert_eq!(breakpoints.len(), 1); // Only conditional breakpoint remains
}

#[tokio::test]
async fn test_lua_hooks_installation() {
    // Create ExecutionManager
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Create InteractiveDebugger
    let debugger = InteractiveDebugger::new(execution_manager, shared_context);

    // Create Lua context
    let lua = mlua::Lua::new();

    // Test hook installation
    let result = debugger.install_lua_hooks(&lua);
    assert!(result.is_ok(), "Hook installation should succeed");
}

#[tokio::test]
async fn test_debug_session_workflow() {
    let debugger = InteractiveDebugger::default();

    // Create debug session
    let session_id = debugger
        .create_debug_session("test_client".to_string())
        .await
        .unwrap();

    // Test various debug commands through session
    debugger
        .handle_session_debug_command(&session_id, DebugCommand::Continue)
        .await
        .unwrap();
    debugger
        .handle_session_debug_command(&session_id, DebugCommand::StepInto)
        .await
        .unwrap();
    debugger
        .handle_session_debug_command(&session_id, DebugCommand::StepOver)
        .await
        .unwrap();
    debugger
        .handle_session_debug_command(&session_id, DebugCommand::Pause)
        .await
        .unwrap();

    // Verify final state
    let state = debugger.get_debug_state().await;
    if let DebugState::Paused { reason, .. } = state {
        assert_eq!(reason, PauseReason::Pause);
    } else {
        panic!("Expected paused state after pause command");
    }
}

#[tokio::test]
async fn test_shared_execution_context_integration() {
    let execution_manager = Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    let debugger = InteractiveDebugger::new(execution_manager, shared_context.clone());

    // Update shared context with test data
    {
        let mut context = shared_context.write().await;
        context.variables.insert(
            "test_var".to_string(),
            serde_json::Value::String("test_value".to_string()),
        );
        context.set_location(llmspell_bridge::execution_context::SourceLocation {
            source: "test.lua".to_string(),
            line: 42,
            column: Some(10),
        });
    }

    // Retrieve and verify context
    let retrieved_context = debugger.get_shared_context().await;
    assert!(retrieved_context.variables.contains_key("test_var"));

    let location = retrieved_context.location.as_ref().unwrap();
    assert_eq!(location.source, "test.lua");
    assert_eq!(location.line, 42);
}
