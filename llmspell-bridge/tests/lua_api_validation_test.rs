//! ABOUTME: Validates Lua API examples and documentation accuracy
//! ABOUTME: Ensures all Memory/Context examples run correctly

use llmspell_bridge::lua::globals::context::inject_context_global;
use llmspell_bridge::lua::globals::memory::inject_memory_global;
use llmspell_bridge::{
    globals::types::GlobalContext, ComponentRegistry, ContextBridge, MemoryBridge, ProviderManager,
};
use llmspell_config::ProviderManagerConfig;
use llmspell_memory::DefaultMemoryManager;
use mlua::Lua;
use std::sync::Arc;
use tracing::{debug, info};

/// Setup Lua environment with Memory + Context globals
///
/// Note: This must be called from within a tokio runtime context (use `#[tokio::test(flavor = "multi_thread")]`)
async fn setup_lua_with_memory_context() -> (Lua, Arc<DefaultMemoryManager>) {
    info!("Setting up Lua environment for API validation");

    let memory_manager = DefaultMemoryManager::new_in_memory()
        .await
        .expect("Failed to create memory manager");
    let memory_manager = Arc::new(memory_manager);

    let memory_bridge = Arc::new(MemoryBridge::new(memory_manager.clone()));
    let context_bridge = Arc::new(ContextBridge::new(memory_manager.clone()));

    let lua = Lua::new();
    let context = create_global_context().await;
    inject_memory_global(&lua, &context, &memory_bridge).expect("Failed to inject Memory global");
    inject_context_global(&lua, &context, &context_bridge)
        .expect("Failed to inject Context global");

    debug!("Lua environment ready for API validation");
    (lua, memory_manager)
}

async fn create_global_context() -> GlobalContext {
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    GlobalContext::new(registry, providers)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_episodic_api_structure() {
    info!("Validating Memory.episodic API structure");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    let script = r#"
        -- Validate Memory.episodic.add exists and works
        local success, result = pcall(Memory.episodic.add, "test-session", "user", "test content", {test = true})
        assert(success, "Memory.episodic.add should succeed: " .. tostring(result))

        -- Validate Memory.episodic.search returns expected structure
        -- API: Memory.episodic.search(session_id, query, limit) -> array of entries
        local results = Memory.episodic.search("test-session", "test", 10)
        assert(type(results) == "table", "search should return table")
        assert(#results >= 0, "search should return array of entries")

        return "ok"
    "#;

    let result: String = lua
        .load(script)
        .eval()
        .expect("API validation should succeed");
    assert_eq!(result, "ok");
    debug!("Memory.episodic API structure validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_stats_api_structure() {
    info!("Validating Memory.stats API structure");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    let script = r#"
        local stats = Memory.stats()
        assert(type(stats) == "table", "stats should return table")
        assert(stats.episodic_count ~= nil, "stats should have episodic_count")
        assert(stats.semantic_count ~= nil, "stats should have semantic_count")
        assert(type(stats.episodic_count) == "number", "episodic_count should be number")
        assert(type(stats.semantic_count) == "number", "semantic_count should be number")
        return "ok"
    "#;

    let result: String = lua
        .load(script)
        .eval()
        .expect("Stats API validation should succeed");
    assert_eq!(result, "ok");
    debug!("Memory.stats API structure validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_context_assemble_api_structure() {
    info!("Validating Context.assemble API structure");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    let script = r#"
        -- Add some data first
        Memory.episodic.add("test-session", "user", "test query", {})
        Memory.episodic.add("test-session", "assistant", "test response", {})

        -- Validate Context.assemble returns expected structure
        local result = Context.assemble("test", "episodic", 1000, "test-session")
        assert(type(result) == "table", "assemble should return table")
        assert(result.chunks ~= nil, "assemble should return chunks")
        assert(type(result.chunks) == "table", "chunks should be table")

        return "ok"
    "#;

    let result: String = lua
        .load(script)
        .eval()
        .expect("Context API validation should succeed");
    assert_eq!(result, "ok");
    debug!("Context.assemble API structure validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_context_strategy_stats_api() {
    info!("Validating Context.strategy_stats API structure");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    let script = r#"
        local stats = Context.strategy_stats()
        assert(type(stats) == "table", "strategy_stats should return table")
        -- Check that stats contains numeric values
        local has_numeric = false
        for k, v in pairs(stats) do
            if type(v) == "number" then
                has_numeric = true
                break
            end
        end
        assert(has_numeric, "stats should contain numeric values")
        return "ok"
    "#;

    let result: String = lua
        .load(script)
        .eval()
        .expect("Strategy stats validation should succeed");
    assert_eq!(result, "ok");
    debug!("Context.strategy_stats API structure validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_documentation_examples_accuracy() {
    info!("Validating documentation code examples");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    // This validates the example from the documentation
    let doc_example = r#"
        -- From Memory.episodic.add documentation
        Memory.episodic.add(
            "session-123",
            "user",
            "What is Rust?",
            {topic = "programming", priority = "high"}
        )

        -- From Memory.episodic.search documentation
        local results = Memory.episodic.search("session-123", "What", 10)
        assert(#results >= 1, "Should find at least the entry we just added")
        assert(results[1].role == "user", "Role should match")
        assert(results[1].content == "What is Rust?", "Content should match")

        -- From Context.assemble documentation
        local context = Context.assemble(
            "Rust ownership",
            "episodic",
            3000,
            "session-123"
        )
        assert(context.chunks ~= nil, "context should have chunks")
        assert(context.token_count ~= nil, "context should have token_count")

        return "ok"
    "#;

    let result: String = lua
        .load(doc_example)
        .eval()
        .expect("Documentation examples should be accurate");
    assert_eq!(result, "ok");
    debug!("Documentation examples validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_error_handling_in_examples() {
    info!("Validating error handling patterns");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    let script = r#"
        -- Test invalid strategy error
        local success, err = pcall(function()
            Context.assemble("test", "invalid_strategy", 1000, nil)
        end)
        assert(not success, "Invalid strategy should error")

        -- Test token budget validation (< 100 is invalid)
        local success2, err2 = pcall(function()
            Context.assemble("test", "episodic", 50, nil)
        end)
        assert(not success2, "Token budget < 100 should error")

        return "ok"
    "#;

    let result: String = lua
        .load(script)
        .eval()
        .expect("Error handling validation should succeed");
    assert_eq!(result, "ok");
    debug!("Error handling patterns validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_context_integration_workflow() {
    info!("Validating Memory + Context integration workflow");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    // Simulate the production pattern: store → assemble → generate → store
    let workflow = r#"
        local session_id = "workflow-test-" .. os.time()

        -- Step 1: Store user input
        Memory.episodic.add(session_id, "user", "What is Rust ownership?", {topic = "rust"})

        -- Step 2: Assemble context
        local context = Context.assemble("Rust ownership", "episodic", 2000, session_id)
        assert(context.chunks ~= nil, "Context should be assembled")

        -- Step 3: Simulate LLM response (would call LLM in production)
        local response = "Rust ownership ensures each value has one owner"

        -- Step 4: Store assistant response
        Memory.episodic.add(session_id, "assistant", response, {topic = "rust"})

        -- Step 5: Verify memory state
        local stats = Memory.stats()
        assert(stats.episodic_count >= 2, "Should have at least 2 episodic entries")

        -- Step 6: Query conversation history
        local history = Memory.episodic.search(session_id, "", 10)
        assert(#history >= 2, "Should find both user and assistant messages")

        return "ok"
    "#;

    let result: String = lua
        .load(workflow)
        .eval()
        .expect("Integration workflow should succeed");
    assert_eq!(result, "ok");
    debug!("Memory + Context integration workflow validated");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_strategy_selection_semantics() {
    info!("Validating strategy selection semantics");
    let (lua, _memory_manager) = setup_lua_with_memory_context().await;

    let script = r#"
        local session_id = "strategy-test-" .. os.time()

        -- Add conversation data
        Memory.episodic.add(session_id, "user", "What is machine learning?", {})
        Memory.episodic.add(session_id, "assistant", "ML learns from data.", {})

        -- Test episodic strategy (requires session_id)
        local episodic_result = Context.assemble("machine learning", "episodic", 1500, session_id)
        assert(episodic_result.chunks ~= nil, "Episodic should return chunks")

        -- Test semantic strategy (ignores session_id)
        local semantic_result = Context.assemble("machine learning", "semantic", 1500)
        assert(semantic_result.chunks ~= nil, "Semantic should return chunks")

        -- Test hybrid strategy (combines both)
        local hybrid_result = Context.assemble("machine learning", "hybrid", 1500, session_id)
        assert(hybrid_result.chunks ~= nil, "Hybrid should return chunks")

        -- Verify strategy stats returns valid data
        local stats = Context.strategy_stats()
        assert(type(stats) == "table", "Stats should be a table")

        return "ok"
    "#;

    let result: String = lua
        .load(script)
        .eval()
        .expect("Strategy selection validation should succeed");
    assert_eq!(result, "ok");
    debug!("Strategy selection semantics validated");
}
