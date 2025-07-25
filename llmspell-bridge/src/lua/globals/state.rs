//! ABOUTME: Lua-specific State global implementation
//! ABOUTME: Provides Lua bindings for persistent state functionality

use crate::globals::{state_global::StateGlobal, GlobalContext};
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use mlua::{Error as LuaError, Lua, Value};

/// Inject State global into Lua environment
pub fn inject_state_global(
    lua: &Lua,
    _context: &GlobalContext,
    state_global: &StateGlobal,
) -> mlua::Result<()> {
    let state_table = lua.create_table()?;

    // Clone references for the closures
    let state_manager = state_global.state_manager.clone();
    let fallback_state = state_global.fallback_state.clone();

    // save(scope, key, value) - Store state value
    let save_state_manager = state_manager.clone();
    let save_fallback_state = fallback_state.clone();
    let save_fn =
        lua.create_function(move |_, (scope_str, key, value): (String, String, Value)| {
            let json_value = lua_value_to_json(value)?;

            if let Some(state_mgr) = &save_state_manager {
                let scope = StateGlobal::parse_scope(&scope_str);
                let runtime = tokio::runtime::Handle::try_current()
                    .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                    .map_err(|e| LuaError::RuntimeError(format!("No tokio runtime: {}", e)))?;

                let result =
                    runtime.block_on(async { state_mgr.set(scope, &key, json_value).await });

                match result {
                    Ok(()) => Ok(()),
                    Err(e) => Err(LuaError::RuntimeError(format!("State save error: {}", e))),
                }
            } else {
                // Fallback to in-memory storage
                let full_key = format!("{}:{}", scope_str, key);
                let mut state = save_fallback_state.write();
                state.insert(full_key, json_value);
                Ok(())
            }
        })?;
    state_table.set("save", save_fn)?;

    // load(scope, key) - Retrieve state value
    let load_state_manager = state_manager.clone();
    let load_fallback_state = fallback_state.clone();
    let load_fn = lua.create_function(move |lua, (scope_str, key): (String, String)| {
        if let Some(state_mgr) = &load_state_manager {
            let scope = StateGlobal::parse_scope(&scope_str);
            let runtime = tokio::runtime::Handle::try_current()
                .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                .map_err(|e| LuaError::RuntimeError(format!("No tokio runtime: {}", e)))?;

            let result = runtime.block_on(async { state_mgr.get(scope, &key).await });

            match result {
                Ok(Some(value)) => {
                    let lua_value = json_to_lua_value(lua, &value)?;
                    Ok(lua_value)
                }
                Ok(None) => Ok(Value::Nil),
                Err(e) => Err(LuaError::RuntimeError(format!("State load error: {}", e))),
            }
        } else {
            // Fallback to in-memory storage
            let full_key = format!("{}:{}", scope_str, key);
            let state = load_fallback_state.read();
            match state.get(&full_key) {
                Some(value) => {
                    let lua_value = json_to_lua_value(lua, value)?;
                    Ok(lua_value)
                }
                None => Ok(Value::Nil),
            }
        }
    })?;
    state_table.set("load", load_fn)?;

    // delete(scope, key) - Remove state value
    let delete_state_manager = state_manager.clone();
    let delete_fallback_state = fallback_state.clone();
    let delete_fn = lua.create_function(move |_, (scope_str, key): (String, String)| {
        if let Some(state_mgr) = &delete_state_manager {
            let scope = StateGlobal::parse_scope(&scope_str);
            let runtime = tokio::runtime::Handle::try_current()
                .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                .map_err(|e| LuaError::RuntimeError(format!("No tokio runtime: {}", e)))?;

            let result = runtime.block_on(async { state_mgr.delete(scope, &key).await });

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(LuaError::RuntimeError(format!("State delete error: {}", e))),
            }
        } else {
            // Fallback to in-memory storage
            let full_key = format!("{}:{}", scope_str, key);
            let mut state = delete_fallback_state.write();
            state.remove(&full_key);
            Ok(())
        }
    })?;
    state_table.set("delete", delete_fn)?;

    // list_keys(scope) - List all keys in a scope
    let list_state_manager = state_manager;
    let list_fallback_state = fallback_state;
    let list_fn = lua.create_function(move |lua, scope_str: String| {
        if let Some(state_mgr) = &list_state_manager {
            let scope = StateGlobal::parse_scope(&scope_str);
            let runtime = tokio::runtime::Handle::try_current()
                .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                .map_err(|e| LuaError::RuntimeError(format!("No tokio runtime: {}", e)))?;

            let result = runtime.block_on(async { state_mgr.list_keys(scope).await });

            match result {
                Ok(keys) => {
                    let table = lua.create_table()?;
                    for (i, key) in keys.iter().enumerate() {
                        table.set(i + 1, key.clone())?;
                    }
                    Ok(table)
                }
                Err(e) => Err(LuaError::RuntimeError(format!("State list error: {}", e))),
            }
        } else {
            // Fallback to in-memory storage
            let state = list_fallback_state.read();
            let prefix = format!("{}:", scope_str);
            let keys: Vec<String> = state
                .keys()
                .filter(|k| k.starts_with(&prefix))
                .map(|k| k.strip_prefix(&prefix).unwrap_or(k).to_string())
                .collect();

            let table = lua.create_table()?;
            for (i, key) in keys.iter().enumerate() {
                table.set(i + 1, key.clone())?;
            }
            Ok(table)
        }
    })?;
    state_table.set("list_keys", list_fn)?;

    // Set the state table as a global
    lua.globals().set("State", state_table)?;

    Ok(())
}
