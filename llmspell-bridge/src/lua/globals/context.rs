//! ABOUTME: Lua-specific Context global implementation
//! ABOUTME: Provides Lua bindings for context assembly functionality

use crate::context_bridge::ContextBridge;
use crate::globals::GlobalContext;
use crate::lua::conversion::json_to_lua_value;
use mlua::{Error as LuaError, Lua};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Inject Context global API into Lua
///
/// Provides the `Context` namespace with assembly, test, and stats operations.
///
/// # API Surface
///
/// ```lua
/// -- Context assembly operations
/// Context.assemble(query, strategy, max_tokens, session_id) -> result
/// Context.test(query, session_id) -> result
/// Context.strategy_stats() -> {episodic_count, semantic_count, strategies}
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function injection into Lua fails
/// - Global injection fails
pub fn inject_context_global(
    lua: &Lua,
    _context: &GlobalContext,
    context_bridge: &Arc<ContextBridge>,
) -> mlua::Result<()> {
    info!("Injecting Context global API");
    let context_table = lua.create_table()?;

    // Context.assemble(query, strategy, max_tokens, session_id)
    let assemble_bridge = context_bridge.clone();
    context_table.set(
        "assemble",
        lua.create_function(
            move |lua,
                  (query, strategy, max_tokens, session_id): (
                String,
                String,
                Option<usize>,
                Option<String>,
            )| {
                debug!(
                    "Context.assemble called with query='{}', strategy='{}'",
                    query, strategy
                );
                let max_tokens = max_tokens.unwrap_or(8192);
                let session_id_ref = session_id.as_deref();

                let result = assemble_bridge
                    .assemble(&query, &strategy, max_tokens, session_id_ref)
                    .map_err(|e| {
                        error!("Context.assemble failed: {}", e);
                        LuaError::RuntimeError(e)
                    })?;

                json_to_lua_value(lua, &result)
            },
        )?,
    )?;

    // Context.test(query, session_id)
    let test_bridge = context_bridge.clone();
    context_table.set(
        "test",
        lua.create_function(move |lua, (query, session_id): (String, Option<String>)| {
            debug!("Context.test called with query='{}'", query);
            let session_id_ref = session_id.as_deref();

            let result = test_bridge
                .test_query(&query, session_id_ref)
                .map_err(|e| {
                    error!("Context.test failed: {}", e);
                    LuaError::RuntimeError(e)
                })?;

            json_to_lua_value(lua, &result)
        })?,
    )?;

    // Context.strategy_stats()
    let stats_bridge = context_bridge.clone();
    context_table.set(
        "strategy_stats",
        lua.create_function(move |lua, ()| {
            debug!("Context.strategy_stats called");

            let stats = stats_bridge.get_strategy_stats().map_err(|e| {
                error!("Context.strategy_stats failed: {}", e);
                LuaError::RuntimeError(e)
            })?;

            json_to_lua_value(lua, &stats)
        })?,
    )?;

    // Inject Context global
    lua.globals().set("Context", context_table)?;
    info!("Context global injected successfully");
    Ok(())
}
