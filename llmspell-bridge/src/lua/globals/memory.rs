//! ABOUTME: Lua-specific Memory global implementation
//! ABOUTME: Provides Lua bindings for memory management functionality

use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use crate::memory_bridge::MemoryBridge;
use mlua::{Error as LuaError, Lua, Table, Value};
use std::sync::Arc;
use tracing::{debug, error, info};

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
pub fn inject_memory_global(
    lua: &Lua,
    _context: &GlobalContext,
    memory_bridge: Arc<MemoryBridge>,
) -> mlua::Result<()> {
    info!("Injecting Memory global API");
    let memory_table = lua.create_table()?;

    // Memory.episodic namespace
    let episodic_table = lua.create_table()?;

    // Memory.episodic.add(session_id, role, content, metadata)
    let add_bridge = memory_bridge.clone();
    episodic_table.set(
        "add",
        lua.create_function(
            move |_lua, (session_id, role, content, metadata): (String, String, String, Option<Table>)| {
                debug!("Memory.episodic.add called for session={}", session_id);
                let metadata_json = if let Some(meta) = metadata {
                    lua_value_to_json(Value::Table(meta))?
                } else {
                    serde_json::json!({})
                };

                add_bridge
                    .episodic_add(session_id, role, content, metadata_json)
                    .map_err(|e| {
                        error!("Memory.episodic.add failed: {}", e);
                        LuaError::RuntimeError(e)
                    })
            },
        )?,
    )?;

    // Memory.episodic.search(session_id, query, limit)
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

                let results = search_bridge
                    .episodic_search(&session_id, &query, limit)
                    .map_err(|e| {
                        error!("Memory.episodic.search failed: {}", e);
                        LuaError::RuntimeError(e)
                    })?;

                json_to_lua_value(lua, &results)
            },
        )?,
    )?;

    memory_table.set("episodic", episodic_table)?;

    // Memory.semantic namespace
    let semantic_table = lua.create_table()?;

    // Memory.semantic.query(query, limit)
    let query_bridge = memory_bridge.clone();
    semantic_table.set(
        "query",
        lua.create_function(move |lua, (query, limit): (String, Option<usize>)| {
            debug!("Memory.semantic.query called with query='{}'", query);
            let limit = limit.unwrap_or(10);

            let results = query_bridge.semantic_query(&query, limit).map_err(|e| {
                error!("Memory.semantic.query failed: {}", e);
                LuaError::RuntimeError(e)
            })?;

            json_to_lua_value(lua, &results)
        })?,
    )?;

    memory_table.set("semantic", semantic_table)?;

    // Memory.consolidate(session_id, force)
    let consolidate_bridge = memory_bridge.clone();
    memory_table.set(
        "consolidate",
        lua.create_function(
            move |lua, (session_id, force): (Option<String>, Option<bool>)| {
                debug!("Memory.consolidate called");
                let force = force.unwrap_or(false);
                let session_id_ref = session_id.as_deref();

                let result = consolidate_bridge
                    .consolidate(session_id_ref, force)
                    .map_err(|e| {
                        error!("Memory.consolidate failed: {}", e);
                        LuaError::RuntimeError(e)
                    })?;

                json_to_lua_value(lua, &result)
            },
        )?,
    )?;

    // Memory.stats()
    let stats_bridge = memory_bridge.clone();
    memory_table.set(
        "stats",
        lua.create_function(move |lua, ()| {
            debug!("Memory.stats called");
            let stats = stats_bridge.stats().map_err(|e| {
                error!("Memory.stats failed: {}", e);
                LuaError::RuntimeError(e)
            })?;

            json_to_lua_value(lua, &stats)
        })?,
    )?;

    // Inject Memory global
    lua.globals().set("Memory", memory_table)?;
    info!("Memory global injected successfully");
    Ok(())
}
