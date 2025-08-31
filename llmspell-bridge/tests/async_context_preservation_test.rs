//! Tests for `SharedExecutionContext` async preservation (Task 9.2.10)
//!
//! These tests verify that:
//! - Context is preserved across async boundaries
//! - Correlation IDs are maintained
//! - Debug context works with async operations
//! - Multi-threaded runtime compatibility

#![allow(clippy::significant_drop_tightening)]

use llmspell_bridge::{
    engine::factory::LuaConfig,
    execution_bridge::{Breakpoint, ExecutionManager},
    execution_context::{SharedExecutionContext, SourceLocation},
    lua::debug_state_cache_impl::LuaDebugStateCache,
    lua::engine::LuaEngine,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test basic async preservation functionality
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_context_preservation() {
    let context = SharedExecutionContext::new();

    // Create context with async support
    let enhanced = context.with_async_support();
    assert!(enhanced.correlation_id.is_some());

    // Preserve across async boundary
    let snapshot = enhanced.preserve_across_async_boundary();
    assert_eq!(snapshot.correlation_id, enhanced.correlation_id);

    // Modify context
    let mut modified = SharedExecutionContext::new();
    modified.set_location(SourceLocation {
        source: "test.lua".to_string(),
        line: 10,
        column: Some(5),
    });

    // Restore from snapshot
    modified.restore_from_async_boundary(snapshot);
    assert_eq!(modified.correlation_id, enhanced.correlation_id);
}

/// Test correlation ID tracking across async operations
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_correlation_id_tracking() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // First async operation
    let correlation_id = {
        let mut ctx = shared_context.write().await;
        let enhanced = ctx.clone().with_async_support();
        *ctx = enhanced;
        ctx.correlation_id
    };

    assert!(correlation_id.is_some());
    let first_id = correlation_id.unwrap();

    // Simulate async boundary crossing
    let snapshot = shared_context.read().await.preserve_across_async_boundary();

    // Another async operation
    tokio::spawn({
        let shared_context = shared_context.clone();
        async move {
            {
                let mut ctx = shared_context.write().await;
                ctx.restore_from_async_boundary(snapshot);
                assert_eq!(ctx.correlation_id, Some(first_id));
            } // Explicitly drop the lock before await point
        }
    })
    .await
    .unwrap();
}

/// Test `LuaEngine` with debug context and async preservation
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_lua_engine_with_async_context() {
    let config = LuaConfig::default();
    let engine = LuaEngine::new(&config).expect("Failed to create Lua engine");

    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Execute with debug context
    let script = r"
        local x = 42
        return x
    ";

    let result = engine
        .execute_with_debug_context(script, shared_context.clone())
        .await;
    assert!(result.is_ok());

    // Verify context has correlation ID
    {
        let ctx = shared_context.read().await;
        assert!(ctx.correlation_id.is_some());
    } // Explicitly drop the lock
}

/// Test context preservation during debugging operations
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_operations_with_async_context() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    let debug_cache = Arc::new(LuaDebugStateCache::new());
    let execution_manager = ExecutionManager::new(debug_cache);

    // Add a breakpoint
    let bp = Breakpoint::new("test.lua".to_string(), 10);
    let _bp_id = execution_manager.add_breakpoint(bp).await;

    // Simulate debug event with context preservation
    let snapshot = {
        let mut ctx = shared_context.write().await;
        let enhanced = ctx.clone().with_async_support();
        *ctx = enhanced;
        ctx.preserve_across_async_boundary()
    };

    // Process debug event in another task
    let context_clone = shared_context.clone();
    let handle = tokio::spawn(async move {
        let mut ctx = context_clone.write().await;
        ctx.restore_from_async_boundary(snapshot);

        // Set debug location
        ctx.set_location(SourceLocation {
            source: "test.lua".to_string(),
            line: 10,
            column: None,
        });

        ctx.correlation_id
    });

    let correlation_id = handle.await.unwrap();
    assert!(correlation_id.is_some());
}

/// Test nested async operations with context preservation
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_nested_async_operations() {
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Initial correlation ID
    let initial_id = {
        let mut ctx = shared_context.write().await;
        let enhanced = ctx.clone().with_async_support();
        *ctx = enhanced;
        ctx.correlation_id.unwrap()
    };

    // First level async operation
    let context_clone = shared_context.clone();
    let handle1 = tokio::spawn(async move {
        let snapshot = context_clone.read().await.preserve_across_async_boundary();

        // Nested async operation
        let context_nested = context_clone.clone();
        let handle2 = tokio::spawn(async move {
            let mut ctx = context_nested.write().await;
            ctx.restore_from_async_boundary(snapshot);
            ctx.correlation_id
        });

        handle2.await.unwrap()
    });

    let final_id = handle1.await.unwrap();
    assert_eq!(final_id, Some(initial_id));
}

/// Test performance of context preservation operations
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_context_preservation_performance() {
    use std::time::Instant;

    let shared_context = Arc::new(RwLock::new(
        SharedExecutionContext::new().with_async_support(),
    ));

    let start = Instant::now();

    // Perform many preserve/restore operations
    for _ in 0..1000 {
        let snapshot = shared_context.read().await.preserve_across_async_boundary();

        shared_context
            .write()
            .await
            .restore_from_async_boundary(snapshot);
    }

    let elapsed = start.elapsed();

    // Should be fast (< 100ms for 1000 operations)
    println!("Context preservation time for 1000 operations: {elapsed:?}");
    assert!(elapsed.as_millis() < 100);
}
