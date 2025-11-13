//! ABOUTME: Tests for Context global Lua API
//! ABOUTME: Verifies `Context.assemble`, `Context.test`, `Context.strategy_stats`

mod test_helpers;

use llmspell_bridge::lua::globals::context::inject_context_global;
use llmspell_bridge::{
    globals::types::GlobalContext, ComponentRegistry, ContextBridge, ProviderManager,
};
use llmspell_config::ProviderManagerConfig;
use llmspell_memory::{DefaultMemoryManager, MemoryManager};
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
fn test_context_global_injection() {
    // Create memory manager using global runtime
    let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
        DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
    });

    // Create bridge
    let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));

    // Create Lua runtime
    let lua = Lua::new();

    // Inject Context global
    let context = create_test_context();
    inject_context_global(&lua, &context, &context_bridge)
        .expect("Failed to inject Context global");

    // Verify Context global exists
    lua.load("assert(Context ~= nil, 'Context global not injected')")
        .exec()
        .expect("Context global should exist");

    // Verify Context.assemble function
    lua.load("assert(type(Context.assemble) == 'function', 'Context.assemble not a function')")
        .exec()
        .expect("Context.assemble should be a function");

    // Verify Context.test function
    lua.load("assert(type(Context.test) == 'function', 'Context.test not a function')")
        .exec()
        .expect("Context.test should be a function");

    // Verify Context.strategy_stats function
    lua.load(
        "assert(type(Context.strategy_stats) == 'function', 'Context.strategy_stats not a function')",
    )
    .exec()
    .expect("Context.strategy_stats should be a function");
}

#[test]
fn test_context_assemble_episodic() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        // Add some episodic memory entries first
        llmspell_kernel::global_io_runtime().block_on(async {
            let entry = llmspell_memory::EpisodicEntry::new(
                "test-session".to_string(),
                "user".to_string(),
                "Hello world".to_string(),
            );
            memory_manager
                .episodic()
                .add(entry)
                .await
                .expect("Failed to add entry");
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.assemble with episodic strategy
        let script = r#"
            local result = Context.assemble("hello", "episodic", 1000, "test-session")
            assert(type(result) == "table", "assemble should return a table")
            assert(type(result.chunks) == "table", "result should have chunks")
            return result
        "#;

        let result: mlua::Table = lua.load(script).eval().expect("assemble should succeed");
        assert!(result.contains_key("chunks").unwrap());
    });
}

#[test]
fn test_context_assemble_semantic() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.assemble with semantic strategy (empty memory OK)
        let script = r#"
        local result = Context.assemble("test query", "semantic", 1000, nil)
        assert(type(result) == "table", "assemble should return a table")
        assert(type(result.chunks) == "table", "result should have chunks")
        return result
    "#;

        let result: mlua::Table = lua.load(script).eval().expect("assemble should succeed");
        assert!(result.contains_key("chunks").unwrap());
    });
}

#[test]
fn test_context_assemble_hybrid() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.assemble with hybrid strategy
        let script = r#"
        local result = Context.assemble("test query", "hybrid", 2000, nil)
        assert(type(result) == "table", "assemble should return a table")
        assert(type(result.chunks) == "table", "result should have chunks")
        return result
    "#;

        let result: mlua::Table = lua.load(script).eval().expect("assemble should succeed");
        assert!(result.contains_key("chunks").unwrap());
    });
}

#[test]
fn test_context_strategy_validation() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test invalid strategy error
        let script = r#"
        local success, err = pcall(function()
            Context.assemble("test", "invalid_strategy", 1000, nil)
        end)
        assert(not success, "Invalid strategy should error")
        local err_str = tostring(err)
        assert(string.find(err_str, "Unknown strategy"), "Error should mention unknown strategy")
        return err_str
    "#;

        let _err: String = lua.load(script).eval().expect("Should capture error");
    });
}

#[test]
fn test_context_token_budget_validation() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test token budget < 100 error
        let script = r#"
        local success, err = pcall(function()
            Context.assemble("test", "episodic", 50, nil)
        end)
        assert(not success, "Token budget < 100 should error")
        local err_str = tostring(err)
        assert(string.find(err_str, "must be >=100"), "Error should mention minimum budget")
        return err_str
    "#;

        let _err: String = lua
            .load(script)
            .eval()
            .expect("Should capture token budget error");
    });
}

#[test]
fn test_context_test() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.test (uses hybrid, 2000 tokens)
        let script = r#"
        local result = Context.test("test query", nil)
        assert(type(result) == "table", "test should return a table")
        assert(type(result.chunks) == "table", "result should have chunks")
        return result
    "#;

        let result: mlua::Table = lua.load(script).eval().expect("test should succeed");
        assert!(result.contains_key("chunks").unwrap());
    });
}

#[test]
fn test_context_strategy_stats() {
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory()
                .await
                .expect("Failed to create memory manager")
        });

        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.strategy_stats
        let script = r#"
        local stats = Context.strategy_stats()
        assert(type(stats) == "table", "strategy_stats should return a table")
        assert(type(stats.episodic_count) == "number", "should have episodic_count")
        assert(type(stats.semantic_count) == "number", "should have semantic_count")
        assert(type(stats.strategies) == "table", "should have strategies array")
        assert(#stats.strategies == 4, "should have 4 strategies (episodic, semantic, hybrid, rag)")
        return stats
    "#;

        let stats: mlua::Table = lua
            .load(script)
            .eval()
            .expect("strategy_stats should succeed");
        let episodic_count: usize = stats.get("episodic_count").unwrap();
        assert_eq!(episodic_count, 0, "No episodic entries initially");
    });
}

#[test]
fn test_context_assemble_rag_without_pipeline() {
    // Test RAG strategy without RAG pipeline configured - should fallback to hybrid
    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });

        // Create ContextBridge WITHOUT RAG pipeline
        let context_bridge = Arc::new(ContextBridge::new(Arc::new(memory_manager)));
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.assemble with rag strategy - should fallback to hybrid
        let script = r#"
        local result = Context.assemble("test query", "rag", 2000, nil)
        assert(type(result) == "table", "assemble should return a table")
        assert(type(result.chunks) == "table", "result should have chunks")
        return result
    "#;

        let result: mlua::Table = lua.load(script).eval().expect("assemble should succeed");
        assert!(result.contains_key("chunks").unwrap());
    });
}

#[test]
fn test_context_assemble_rag_with_pipeline() {
    // Test RAG strategy with RAG pipeline configured
    // Note: This test creates a mock RAG pipeline for testing
    use async_trait::async_trait;
    use llmspell_core::state::StateScope;
    use llmspell_rag::pipeline::{RAGResult, RAGRetriever};
    use std::collections::HashMap;

    // Mock RAG pipeline for testing
    struct MockRAGRetriever;

    #[async_trait]
    impl RAGRetriever for MockRAGRetriever {
        async fn retrieve(
            &self,
            _query: &str,
            k: usize,
            _scope: Option<StateScope>,
        ) -> anyhow::Result<Vec<RAGResult>> {
            // Return mock results (up to 3)
            let now = chrono::Utc::now();
            let mut results = Vec::new();

            if k >= 1 {
                results.push(RAGResult {
                    id: "rag-0".to_string(),
                    content: "RAG result 0".to_string(),
                    score: 0.9,
                    metadata: HashMap::default(),
                    timestamp: now,
                });
            }
            if k >= 2 {
                results.push(RAGResult {
                    id: "rag-1".to_string(),
                    content: "RAG result 1".to_string(),
                    score: 0.8,
                    metadata: HashMap::default(),
                    timestamp: now,
                });
            }
            if k >= 3 {
                results.push(RAGResult {
                    id: "rag-2".to_string(),
                    content: "RAG result 2".to_string(),
                    score: 0.7,
                    metadata: HashMap::default(),
                    timestamp: now,
                });
            }

            Ok(results)
        }
    }

    with_runtime_context(|| {
        let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager")
        });

        // Create ContextBridge WITH mock RAG pipeline
        let context_bridge = Arc::new(
            ContextBridge::new(Arc::new(memory_manager))
                .with_rag_pipeline(Arc::new(MockRAGRetriever)),
        );
        let lua = Lua::new();
        let context = create_test_context();

        inject_context_global(&lua, &context, &context_bridge)
            .expect("Failed to inject Context global");

        // Test Context.assemble with rag strategy - should use RAG+Memory hybrid
        let script = r#"
        local result = Context.assemble("test query", "rag", 2000, "test-session")
        assert(type(result) == "table", "assemble should return a table")
        assert(type(result.chunks) == "table", "result should have chunks")
        return result
    "#;

        let result: mlua::Table = lua.load(script).eval().expect("assemble should succeed");
        assert!(result.contains_key("chunks").unwrap());
    });
}
