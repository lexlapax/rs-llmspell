//! ABOUTME: Lua-specific Memory global implementation
//! ABOUTME: Provides Lua bindings for memory management functionality

use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use crate::lua::sync_utils::block_on_async;
use crate::memory_bridge::MemoryBridge;
use mlua::{Lua, Table, Value};
use std::sync::Arc;
use tracing::{debug, error, info};

// Wrapper to make String errors compatible with StdError trait bound
#[derive(Debug)]
struct StringError(String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

/// Create episodic namespace with `add()` and `search()` functions
fn create_episodic_namespace<'lua>(
    lua: &'lua Lua,
    memory_bridge: &Arc<MemoryBridge>,
) -> mlua::Result<Table<'lua>> {
    let episodic_table = lua.create_table()?;

    // Memory.episodic.add
    let add_bridge = memory_bridge.clone();
    episodic_table.set(
        "add",
        lua.create_function(
            move |_lua,
                  (session_id, role, content, metadata): (
                String,
                String,
                String,
                Option<Table>,
            )| {
                debug!("Memory.episodic.add called for session={}", session_id);
                let metadata_json = metadata
                    .map(|m| lua_value_to_json(Value::Table(m)))
                    .transpose()?
                    .unwrap_or_else(|| serde_json::json!({}));

                let bridge = add_bridge.clone();
                block_on_async(
                    "memory_episodic_add",
                    async move {
                        bridge
                            .episodic_add(session_id, role, content, metadata_json)
                            .await
                            .map_err(StringError)
                    },
                    None,
                )
                .map_err(|e| {
                    error!("Memory.episodic.add failed: {}", e);
                    e
                })
            },
        )?,
    )?;

    // Memory.episodic.search
    let search_bridge = memory_bridge.clone();
    episodic_table.set(
        "search",
        lua.create_function(
            move |lua, (session_id, query, limit): (String, String, Option<usize>)| {
                debug!(
                    "Memory.episodic.search called for session={}, query='{}'",
                    session_id, query
                );
                let limit = limit.unwrap_or(10);

                let bridge = search_bridge.clone();
                let results = block_on_async(
                    "memory_episodic_search",
                    async move {
                        bridge
                            .episodic_search(&session_id, &query, limit)
                            .await
                            .map_err(StringError)
                    },
                    None,
                )?;

                json_to_lua_value(lua, &results)
            },
        )?,
    )?;

    Ok(episodic_table)
}

/// Create semantic namespace with `query()` function
fn create_semantic_namespace<'lua>(
    lua: &'lua Lua,
    memory_bridge: &Arc<MemoryBridge>,
) -> mlua::Result<Table<'lua>> {
    let semantic_table = lua.create_table()?;

    // Memory.semantic.query
    let query_bridge = memory_bridge.clone();
    semantic_table.set(
        "query",
        lua.create_function(move |lua, (query, limit): (String, Option<usize>)| {
            debug!("Memory.semantic.query called with query='{}'", query);
            let limit = limit.unwrap_or(10);

            let bridge = query_bridge.clone();
            let results = block_on_async(
                "memory_semantic_query",
                async move {
                    bridge
                        .semantic_query(&query, limit)
                        .await
                        .map_err(StringError)
                },
                None,
            )?;

            json_to_lua_value(lua, &results)
        })?,
    )?;

    Ok(semantic_table)
}

/// Inject Memory global API into Lua
///
/// Provides the `Memory` namespace with episodic, semantic, consolidation, and stats operations.
///
/// # API Surface
///
/// ```lua
/// -- Episodic memory operations
/// Memory.episodic.add(session_id, role, content, metadata) -> id
/// Memory.episodic.search(session_id, query, limit) -> results
///
/// -- Semantic memory operations
/// Memory.semantic.query(query, limit) -> results
///
/// -- Consolidation
/// Memory.consolidate(session_id, force) -> stats
///
/// -- Stats
/// Memory.stats() -> {episodic_count, semantic_count, ...}
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function injection into Lua fails
/// - Global injection fails
pub fn inject_memory_global(
    lua: &Lua,
    _context: &GlobalContext,
    memory_bridge: &Arc<MemoryBridge>,
) -> mlua::Result<()> {
    info!("Injecting Memory global API");
    let memory_table = lua.create_table()?;

    // Create namespaces
    memory_table.set("episodic", create_episodic_namespace(lua, memory_bridge)?)?;
    memory_table.set("semantic", create_semantic_namespace(lua, memory_bridge)?)?;

    // Memory.consolidate
    let consolidate_bridge = memory_bridge.clone();
    memory_table.set(
        "consolidate",
        lua.create_function(
            move |lua, (session_id, force): (Option<String>, Option<bool>)| {
                debug!("Memory.consolidate called");
                let force = force.unwrap_or(false);
                let session_id_ref = session_id.as_deref();

                let bridge = consolidate_bridge.clone();
                let result = block_on_async(
                    "memory_consolidate",
                    async move {
                        bridge
                            .consolidate(session_id_ref, force)
                            .await
                            .map_err(StringError)
                    },
                    None,
                )?;

                json_to_lua_value(lua, &result)
            },
        )?,
    )?;

    // Memory.stats
    let stats_bridge = memory_bridge.clone();
    memory_table.set(
        "stats",
        lua.create_function(move |lua, ()| {
            debug!("Memory.stats called");
            let bridge = stats_bridge.clone();
            let stats = block_on_async(
                "memory_stats",
                async move { bridge.stats().await.map_err(StringError) },
                None,
            )?;

            json_to_lua_value(lua, &stats)
        })?,
    )?;

    // Inject Memory global
    lua.globals().set("Memory", memory_table)?;
    info!("Memory global injected successfully");
    Ok(())
}
