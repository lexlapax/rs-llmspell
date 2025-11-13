//! ABOUTME: Tests for Memory global Lua API
//! ABOUTME: Verifies Memory.episodic, Memory.semantic, Memory.consolidate, Memory.stats

#[path = "../test_helpers.rs"]
mod test_helpers;

use llmspell_bridge::lua::globals::memory::inject_memory_global;
use llmspell_bridge::{
    globals::types::GlobalContext, ComponentRegistry, MemoryBridge, ProviderManager,
};
use llmspell_config::ProviderManagerConfig;
use llmspell_memory::DefaultMemoryManager;
use mlua::Lua;
use std::sync::Arc;
use test_helpers::with_runtime_context;

/// Create a minimal `GlobalContext` for testing
fn create_test_context() -> GlobalContext {
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = llmspell_kernel::global_io_runtime()
        .block_on(async { Arc::new(ProviderManager::new(provider_config).await.unwrap()) });
    GlobalContext::new(registry, providers)
}

#[test]
fn test_memory_global_injection() {
    // Create memory manager using global runtime
    let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
        DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
    });

    // Create bridge
    let memory_bridge = Arc::new(MemoryBridge::new(Arc::new(memory_manager)));

    // Create Lua runtime
    let lua = Lua::new();

    // Inject Memory global
    let context = create_test_context();
    inject_memory_global(&lua, &context, &memory_bridge).expect("Failed to inject Memory global");

    // Verify Memory global exists
    lua.load("assert(Memory ~= nil, 'Memory global not injected')")
        .exec()
        .expect("Memory global should exist");

    // Verify Memory.episodic namespace
    lua.load("assert(Memory.episodic ~= nil, 'Memory.episodic not found')")
        .exec()
        .expect("Memory.episodic should exist");

    // Verify Memory.semantic namespace
    lua.load("assert(Memory.semantic ~= nil, 'Memory.semantic not found')")
        .exec()
        .expect("Memory.semantic should exist");

    // Verify Memory.consolidate function
    lua.load("assert(type(Memory.consolidate) == 'function', 'Memory.consolidate not a function')")
        .exec()
        .expect("Memory.consolidate should be a function");

    // Verify Memory.stats function
    lua.load("assert(type(Memory.stats) == 'function', 'Memory.stats not a function')")
        .exec()
        .expect("Memory.stats should be a function");
}

#[test]
fn test_memory_episodic_add() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });
        let memory_bridge = Arc::new(MemoryBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_memory_global(&lua, &context, &memory_bridge)
            .expect("Failed to inject Memory global");

        // Test Memory.episodic.add
        let script = r#"
            local id = Memory.episodic.add("test-session", "user", "Hello world", {topic = "greeting"})
            assert(type(id) == "string", "add should return a string ID")
            assert(#id > 0, "ID should not be empty")
            return id
        "#;

        let id: String = lua
            .load(script)
            .eval()
            .expect("episodic.add should succeed");
        assert!(!id.is_empty(), "ID should not be empty");
    });
}

#[test]
fn test_memory_episodic_search() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });
        let memory_bridge = Arc::new(MemoryBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_memory_global(&lua, &context, &memory_bridge)
            .expect("Failed to inject Memory global");

        // Add an entry then search
        let script = r#"
            Memory.episodic.add("test-session", "user", "Hello world", {})
            local results = Memory.episodic.search("test-session", "hello", 10)
            assert(type(results) == "table", "search should return a table")
            assert(#results > 0, "search should find the added entry")
            return #results
        "#;

        let count: usize = lua
            .load(script)
            .eval()
            .expect("episodic.search should succeed");
        assert!(count > 0, "Should find at least one result");
    });
}

#[test]
fn test_memory_semantic_query() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });
        let memory_bridge = Arc::new(MemoryBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_memory_global(&lua, &context, &memory_bridge)
            .expect("Failed to inject Memory global");

        // Query semantic memory (should be empty initially)
        let script = r#"
            local results = Memory.semantic.query("test query", 10)
            assert(type(results) == "table", "semantic.query should return a table")
            return #results
        "#;

        let count: usize = lua
            .load(script)
            .eval()
            .expect("semantic.query should succeed");
        // Empty is OK for now - semantic memory is not populated in this test
        assert_eq!(count, 0, "Semantic memory should be empty initially");
    });
}

#[test]
fn test_memory_consolidate() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });
        let memory_bridge = Arc::new(MemoryBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_memory_global(&lua, &context, &memory_bridge)
            .expect("Failed to inject Memory global");

        // Test consolidation
        let script = r#"
            local result = Memory.consolidate(nil, false)
            assert(type(result) == "table", "consolidate should return a table")
            assert(type(result.entries_processed) == "number", "should have entries_processed")
            return result.entries_processed
        "#;

        let processed: usize = lua.load(script).eval().expect("consolidate should succeed");
        // Should be 0 since no entries were added
        assert_eq!(processed, 0, "No entries to consolidate");
    });
}

#[test]
fn test_memory_stats() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });
        let memory_bridge = Arc::new(MemoryBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_memory_global(&lua, &context, &memory_bridge)
            .expect("Failed to inject Memory global");

        // Test stats
        let script = r#"
            local stats = Memory.stats()
            assert(type(stats) == "table", "stats should return a table")
            assert(type(stats.episodic_count) == "number", "should have episodic_count")
            assert(type(stats.semantic_count) == "number", "should have semantic_count")
            return stats.episodic_count
        "#;

        let count: usize = lua.load(script).eval().expect("stats should succeed");
        assert_eq!(count, 0, "No episodic entries initially");
    });
}
