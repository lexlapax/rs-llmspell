//! ABOUTME: Integration tests for Memory + Context bridge and global interaction
//! ABOUTME: Verifies E2E workflows, cross-component dependencies, error propagation, API consistency

mod test_helpers;

use llmspell_bridge::lua::globals::context::inject_context_global;
use llmspell_bridge::lua::globals::memory::inject_memory_global;
use llmspell_bridge::{
    globals::types::GlobalContext, ComponentRegistry, ContextBridge, MemoryBridge, MemoryProvider,
    ProviderManager,
};
use llmspell_config::ProviderManagerConfig;
use llmspell_memory::{DefaultMemoryManager, EpisodicEntry, MemoryManager};
use mlua::Lua;
use std::sync::Arc;
use test_helpers::with_runtime_context;
use tracing::{debug, info};

/// Create a minimal `GlobalContext` for testing
fn create_test_context() -> GlobalContext {
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = llmspell_kernel::global_io_runtime()
        .block_on(async { Arc::new(ProviderManager::new(provider_config).await.unwrap()) });
    GlobalContext::new(registry, providers)
}

/// Setup Lua environment with both Memory and Context globals injected
fn setup_integrated_lua_env() -> (Lua, Arc<DefaultMemoryManager>) {
    info!("Setting up integrated Lua environment with Memory + Context globals");

    // Create memory manager
    let memory_manager = llmspell_kernel::global_io_runtime().block_on(async {
        DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager")
    });
    let memory_manager = Arc::new(memory_manager);

    // Create bridges
    let memory_bridge = Arc::new(MemoryBridge::new(MemoryProvider::new_eager(
        memory_manager.clone(),
    )));
    let context_bridge = Arc::new(ContextBridge::new(MemoryProvider::new_eager(
        memory_manager.clone(),
    )));

    // Create Lua runtime
    let lua = Lua::new();

    // Inject both globals
    let context = create_test_context();
    inject_memory_global(&lua, &context, &memory_bridge).expect("Failed to inject Memory global");
    inject_context_global(&lua, &context, &context_bridge)
        .expect("Failed to inject Context global");

    debug!("Integrated Lua environment ready");
    (lua, memory_manager)
}

/// Helper: Verify context assembly result has expected structure
fn verify_context_result(result: &mlua::Table) {
    assert!(
        result.contains_key("chunks").unwrap(),
        "Result should have chunks"
    );
    assert!(
        result.contains_key("token_count").unwrap(),
        "Result should have token_count"
    );
}

/// Helper: Add episodic entry to memory manager (async)
async fn add_episodic_entry(
    memory_manager: &DefaultMemoryManager,
    session_id: &str,
    role: &str,
    content: &str,
) {
    let entry = EpisodicEntry::new(
        session_id.to_string(),
        role.to_string(),
        content.to_string(),
    );
    memory_manager
        .episodic()
        .add(entry)
        .await
        .expect("Failed to add episodic entry");
}

/// Helper: Execute Lua script expecting table result
fn exec_lua_script_table<'a>(lua: &'a Lua, script: &str, desc: &str) -> mlua::Table<'a> {
    lua.load(script)
        .eval::<mlua::Table>()
        .unwrap_or_else(|_| panic!("{desc} should succeed"))
}

/// Helper: Execute Lua script expecting string result
fn exec_lua_script_string(lua: &Lua, script: &str, desc: &str) -> String {
    lua.load(script)
        .eval()
        .unwrap_or_else(|_| panic!("{desc} should succeed"))
}

/// Helper: Add episodic conversation entries for testing
fn add_episodic_conversation(memory_manager: &DefaultMemoryManager, session_id: &str) {
    llmspell_kernel::global_io_runtime().block_on(async {
        add_episodic_entry(memory_manager, session_id, "user", "Hello episodic world").await;
        add_episodic_entry(
            memory_manager,
            session_id,
            "assistant",
            "Episodic response here",
        )
        .await;
    });
}

/// Helper: Test a context assembly strategy
fn test_strategy<'a>(lua: &'a Lua, script: &str, strategy_name: &str) -> mlua::Table<'a> {
    let result = exec_lua_script_table(lua, script, &format!("{strategy_name} strategy"));
    debug!("{} strategy result: {:?}", strategy_name, result);
    result
}

/// Helper: Add memories for multiple sessions
fn add_session_memories(
    memory_manager: &DefaultMemoryManager,
    sessions: &[(&str, &[(&str, &str)])],
) {
    llmspell_kernel::global_io_runtime().block_on(async {
        for (session_id, messages) in sessions {
            for (role, content) in *messages {
                add_episodic_entry(memory_manager, session_id, role, content).await;
            }
        }
    });
}

/// Helper: Verify chunk count from context result
fn verify_chunk_count(result: &mlua::Table, min_expected: usize, desc: &str) -> usize {
    let chunks: mlua::Table = result.get("chunks").unwrap();
    let chunk_count =
        usize::try_from(chunks.len().unwrap()).expect("chunk count should fit in usize");
    debug!("{desc}: {chunk_count} chunks");
    assert!(
        chunk_count >= min_expected,
        "{desc} should have at least {min_expected} chunk(s)"
    );
    chunk_count
}

/// Helper: Test Lua error propagation
fn test_lua_error(lua: &Lua, script: &str, expected_msg: &str, desc: &str) {
    let err_msg = exec_lua_script_string(lua, script, desc);
    debug!("Captured error: {}", err_msg);
    assert!(
        err_msg.contains(expected_msg),
        "{desc} error should contain '{expected_msg}'"
    );
}

/// Helper: Search all episodic entries
async fn search_all_entries(memory_manager: &DefaultMemoryManager) -> Vec<EpisodicEntry> {
    memory_manager
        .episodic()
        .search("", 1000)
        .await
        .expect("Should retrieve all entries")
}

/// Helper: Find entry by `session_id`
fn find_entry_by_session<'a>(
    entries: &'a [EpisodicEntry],
    session_id: &str,
) -> Option<&'a EpisodicEntry> {
    entries.iter().find(|e| e.session_id == session_id)
}

/// Helper: Verify entries exist in memory by `session_id`
fn verify_entries_exist(memory_manager: &DefaultMemoryManager, session_ids: &[&str]) {
    llmspell_kernel::global_io_runtime().block_on(async {
        let all_entries = search_all_entries(memory_manager).await;
        verify_entry_count(&all_entries, session_ids.len());
        verify_sessions_present(&all_entries, session_ids);
    });
}

/// Helper: Verify entry count
fn verify_entry_count(entries: &[EpisodicEntry], min_count: usize) {
    debug!("Total entries in memory: {}", entries.len());
    assert!(
        entries.len() >= min_count,
        "Should have at least {min_count} entries"
    );
}

/// Helper: Verify session IDs are present in entries
fn verify_sessions_present(entries: &[EpisodicEntry], session_ids: &[&str]) {
    for session_id in session_ids {
        let entry = find_entry_by_session(entries, session_id);
        assert!(entry.is_some(), "Entry for {session_id} should exist");
        debug!("{} entry: {:?}", session_id, entry.unwrap());
    }
}

/// Helper: Verify episodic count via Lua stats
fn verify_lua_stats(lua: &Lua, min_count: usize) {
    let script = r#"
        local stats = Memory.stats()
        assert(stats.episodic_count >= 2, "Should have at least 2 episodic entries")
        return stats
    "#;
    let stats = exec_lua_script_table(lua, script, "Memory stats");
    let episodic_count: usize = stats.get("episodic_count").unwrap();
    debug!("Episodic count from Lua: {}", episodic_count);
    assert!(
        episodic_count >= min_count,
        "Should have at least {min_count} episodic entries"
    );
}

/// Helper: Execute E2E Memory+Context workflow in Lua
fn execute_e2e_workflow(lua: &Lua) -> mlua::Table<'_> {
    let script = r#"
        Memory.episodic.add("session-123", "user", "What is Rust?", {topic = "programming"})
        Memory.episodic.add("session-123", "assistant", "Rust is a systems programming language", {topic = "programming"})
        Memory.episodic.add("session-123", "user", "Tell me about ownership", {topic = "rust"})
        local result = Context.assemble("ownership in Rust", "episodic", 2000, "session-123")
        assert(result.chunks ~= nil, "Should return chunks")
        assert(type(result.chunks) == "table", "Chunks should be a table")
        assert(result.token_count ~= nil, "Should have token_count")
        assert(type(result.token_count) == "number", "token_count should be a number")
        return result
    "#;
    exec_lua_script_table(lua, script, "E2E script")
}

/// Helper: Verify E2E result has chunks
fn verify_e2e_result_has_chunks(result: &mlua::Table) {
    debug!("E2E test result: {:?}", result);
    verify_context_result(result);
    let chunks: mlua::Table = result.get("chunks").unwrap();
    let chunk_count = chunks.len().unwrap();
    debug!("Retrieved {} chunks from context assembly", chunk_count);
    assert!(chunk_count > 0, "Should retrieve at least 1 chunk");
}

#[test]
fn test_e2e_lua_memory_context_workflow() {
    with_runtime_context(|| {
        info!("Starting E2E Lua Memory+Context integration test");
        let (lua, _memory_manager) = setup_integrated_lua_env();
        let result = execute_e2e_workflow(&lua);
        verify_e2e_result_has_chunks(&result);
        info!("E2E workflow test completed successfully");
    });
}

#[test]
fn test_strategy_routing() {
    with_runtime_context(|| {
        info!("Testing strategy routing (episodic vs semantic vs hybrid)");
        let (lua, memory_manager) = setup_integrated_lua_env();

        add_episodic_conversation(&memory_manager, "session-abc");

        let episodic_script = r#"
        local result = Context.assemble("episodic", "episodic", 1000, "session-abc")
        assert(result.chunks ~= nil, "Should return chunks")
        return result
    "#;
        test_strategy(&lua, episodic_script, "Episodic");

        let semantic_script = r#"
        local result = Context.assemble("semantic", "semantic", 1000, nil)
        assert(result.chunks ~= nil, "Should return chunks (even if empty)")
        return result
    "#;
        test_strategy(&lua, semantic_script, "Semantic");

        let hybrid_script = r#"
        local result = Context.assemble("hybrid query", "hybrid", 2000, nil)
        assert(result.chunks ~= nil, "Should return chunks")
        return result
    "#;
        test_strategy(&lua, hybrid_script, "Hybrid");

        info!("Strategy routing tests completed successfully");
    });
}

#[test]
fn test_session_filtering() {
    with_runtime_context(|| {
        info!("Testing session_id filtering in episodic retrieval");
        let (lua, memory_manager) = setup_integrated_lua_env();

        let sessions = &[
            (
                "session-A",
                &[
                    ("user", "Message in session A"),
                    ("assistant", "Response in session A"),
                ][..],
            ),
            (
                "session-B",
                &[
                    ("user", "Message in session B"),
                    ("assistant", "Response in session B"),
                ][..],
            ),
        ];
        add_session_memories(&memory_manager, sessions);

        let script = r#"
        local result = Context.assemble("session", "episodic", 2000, "session-A")
        assert(result.chunks ~= nil, "Should return chunks")
        local chunks = result.chunks
        assert(#chunks > 0, "Should have chunks from session-A")
        return result
    "#;

        let result = exec_lua_script_table(&lua, script, "Session filtering");
        debug!("Session filtering result: {:?}", result);
        verify_chunk_count(&result, 1, "session-A");
        info!("Session filtering test completed successfully");
    });
}

#[test]
fn test_error_propagation() {
    with_runtime_context(|| {
        info!("Testing Rust error → Lua RuntimeError propagation");
        let (lua, _memory_manager) = setup_integrated_lua_env();

        let invalid_strategy_script = r#"
        local success, err = pcall(function()
            Context.assemble("test", "invalid_strategy", 1000, nil)
        end)
        assert(not success, "Invalid strategy should error")
        local err_str = tostring(err)
        assert(string.find(err_str, "Unknown strategy"), "Error should mention unknown strategy")
        return err_str
    "#;
        test_lua_error(
            &lua,
            invalid_strategy_script,
            "Unknown strategy",
            "Invalid strategy",
        );

        let token_budget_script = r#"
        local success, err = pcall(function()
            Context.assemble("test", "episodic", 50, nil)
        end)
        assert(not success, "Token budget < 100 should error")
        local err_str = tostring(err)
        assert(string.find(err_str, "must be >=100"), "Error should mention minimum budget")
        return err_str
    "#;
        test_lua_error(&lua, token_budget_script, "must be >=100", "Token budget");

        info!("Error propagation test completed successfully");
    });
}

/// Helper: Add entry via Rust `MemoryBridge`
fn add_via_rust_bridge(memory_manager: Arc<DefaultMemoryManager>, session_id: &str, message: &str) {
    let memory_bridge = MemoryBridge::new(MemoryProvider::new_eager(memory_manager));
    llmspell_kernel::global_io_runtime().block_on(async {
        memory_bridge
            .episodic_add(
                session_id.to_string(),
                "user".to_string(),
                message.to_string(),
                serde_json::json!({"source": "rust"}),
            )
            .await
            .expect("Rust bridge add should succeed");
    });
    debug!("Added entry via Rust MemoryBridge");
}

/// Helper: Add entry via Lua Memory global
fn add_via_lua_global(lua: &Lua, session_id: &str, message: &str) {
    let script =
        format!(r#"Memory.episodic.add("{session_id}", "user", "{message}", {{source = "lua"}})"#);
    lua.load(&script)
        .exec()
        .expect("Lua Memory.episodic.add should succeed");
    debug!("Added entry via Lua Memory.episodic.add()");
}

#[test]
fn test_bridge_global_api_consistency() {
    with_runtime_context(|| {
        info!("Testing MemoryBridge methods match Memory.* Lua API");
        let (lua, memory_manager) = setup_integrated_lua_env();

        add_via_rust_bridge(
            memory_manager.clone(),
            "session-rust",
            "Message added via Rust bridge",
        );
        add_via_lua_global(&lua, "session-lua", "Message added via Lua");

        verify_entries_exist(&memory_manager, &["session-rust", "session-lua"]);
        verify_lua_stats(&lua, 2);

        info!("Bridge-Global API consistency test completed successfully");
    });
}
