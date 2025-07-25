//! ABOUTME: State global object providing persistent state management
//! ABOUTME: Integrates with StateManager for full persistent state functionality

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::error::LLMSpellError;
use llmspell_state_persistence::{StateManager, StateScope};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// State global object providing persistent state management
///
/// Integrates with StateManager for full persistent state functionality.
/// Falls back to in-memory storage when StateManager is not available.
pub struct StateGlobal {
    /// StateManager for persistent storage (optional for backward compatibility)
    pub state_manager: Option<Arc<StateManager>>,
    /// Fallback in-memory state storage (when StateManager is not available)
    pub fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl StateGlobal {
    /// Create a new State global without StateManager (fallback mode)
    pub fn new() -> Self {
        Self {
            state_manager: None,
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new State global with StateManager integration
    pub fn with_state_manager(state_manager: Arc<StateManager>) -> Self {
        Self {
            state_manager: Some(state_manager),
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Helper method to parse scope string to StateScope enum
    pub fn parse_scope(scope_str: &str) -> StateScope {
        if scope_str.starts_with("agent:") {
            StateScope::Agent(
                scope_str
                    .strip_prefix("agent:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str.starts_with("workflow:") {
            StateScope::Workflow(
                scope_str
                    .strip_prefix("workflow:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str.starts_with("session:") {
            StateScope::Session(
                scope_str
                    .strip_prefix("session:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str.starts_with("custom:") {
            StateScope::Custom(
                scope_str
                    .strip_prefix("custom:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str == "global" {
            StateScope::Global
        } else {
            StateScope::Custom(scope_str.to_string())
        }
    }
}

impl GlobalObject for StateGlobal {
    fn metadata(&self) -> GlobalMetadata {
        let description = if self.state_manager.is_some() {
            "State management system with persistent storage".to_string()
        } else {
            "State management system (in-memory fallback)".to_string()
        };

        GlobalMetadata {
            name: "State".to_string(),
            version: "1.0.0".to_string(), // Phase 5 version with StateManager
            description,
            dependencies: vec![], // StateManager is optional - gracefully falls back to in-memory
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, _context: &GlobalContext) -> Result<(), LLMSpellError> {
        let state_manager = self.state_manager.clone();
        let fallback_state = self.fallback_state.clone();

        // Create State table
        let state_table = lua.create_table().map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create State table: {}", e),
            source: None,
        })?;

        // Get method - retrieve state value (scope, key)
        let get_state_manager = state_manager.clone();
        let get_fallback_state = fallback_state.clone();
        let get_fn = lua
            .create_function(move |lua, args: mlua::Variadic<String>| {
                let (scope_str, key) = match args.len() {
                    1 => ("Global".to_string(), args[0].clone()),
                    2 => (args[0].clone(), args[1].clone()),
                    _ => {
                        return Err(mlua::Error::RuntimeError(
                            "State.get() expects 1 or 2 arguments: get(key) or get(scope, key)"
                                .to_string(),
                        ))
                    }
                };
                // Use async wrapper for StateManager operations
                if let Some(state_mgr) = &get_state_manager {
                    let scope = Self::parse_scope(&scope_str);
                    let runtime = tokio::runtime::Handle::try_current()
                        .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!("No tokio runtime: {}", e))
                        })?;

                    let result = runtime.block_on(async { state_mgr.get(scope, &key).await });

                    match result {
                        Ok(Some(value)) => {
                            let lua_value = crate::lua::conversion::json_to_lua_value(lua, &value)?;
                            Ok(lua_value)
                        }
                        Ok(None) => Ok(mlua::Value::Nil),
                        Err(e) => Err(mlua::Error::RuntimeError(format!("State get error: {}", e))),
                    }
                } else {
                    // Fallback to in-memory storage
                    let full_key = format!("{}:{}", scope_str, key);
                    let state = get_fallback_state.read();
                    match state.get(&full_key) {
                        Some(value) => {
                            let lua_value = crate::lua::conversion::json_to_lua_value(lua, value)?;
                            Ok(lua_value)
                        }
                        None => Ok(mlua::Value::Nil),
                    }
                }
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create State.get: {}", e),
                source: None,
            })?;

        state_table
            .set("get", get_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set State.get: {}", e),
                source: None,
            })?;

        // Set method - store state value (scope, key, value)
        let set_state_manager = state_manager.clone();
        let set_fallback_state = fallback_state.clone();
        let set_fn = lua
            .create_function(
                move |_, args: mlua::Variadic<mlua::Value>| {
                    let (scope_str, key, value) = match args.len() {
                        2 => {
                            let key = args[0].as_str().ok_or_else(|| {
                                mlua::Error::RuntimeError("First argument must be a string (key)".to_string())
                            })?.to_string();
                            ("Global".to_string(), key, args[1].clone())
                        },
                        3 => {
                            let scope = args[0].as_str().ok_or_else(|| {
                                mlua::Error::RuntimeError("First argument must be a string (scope)".to_string())
                            })?.to_string();
                            let key = args[1].as_str().ok_or_else(|| {
                                mlua::Error::RuntimeError("Second argument must be a string (key)".to_string())
                            })?.to_string();
                            (scope, key, args[2].clone())
                        },
                        _ => return Err(mlua::Error::RuntimeError(
                            "State.set() expects 2 or 3 arguments: set(key, value) or set(scope, key, value)".to_string()
                        )),
                    };
                    let json_value = crate::lua::conversion::lua_value_to_json(value)?;

                    if let Some(state_mgr) = &set_state_manager {
                        let scope = Self::parse_scope(&scope_str);
                        let runtime = tokio::runtime::Handle::try_current()
                            .or_else(|_| {
                                tokio::runtime::Runtime::new().map(|rt| rt.handle().clone())
                            })
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!("No tokio runtime: {}", e))
                            })?;

                        let result = runtime
                            .block_on(async { state_mgr.set(scope, &key, json_value).await });

                        match result {
                            Ok(()) => Ok(()),
                            Err(e) => {
                                Err(mlua::Error::RuntimeError(format!("State set error: {}", e)))
                            }
                        }
                    } else {
                        // Fallback to in-memory storage
                        let full_key = format!("{}:{}", scope_str, key);
                        let mut state = set_fallback_state.write();
                        state.insert(full_key, json_value);
                        Ok(())
                    }
                },
            )
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create State.set: {}", e),
                source: None,
            })?;

        state_table
            .set("set", set_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set State.set: {}", e),
                source: None,
            })?;

        // Delete method - remove state value (scope, key)
        let delete_state_manager = state_manager.clone();
        let delete_fallback_state = fallback_state.clone();
        let delete_fn = lua
            .create_function(move |_, args: mlua::Variadic<String>| {
                let (scope_str, key) = match args.len() {
                    1 => ("Global".to_string(), args[0].clone()),
                    2 => (args[0].clone(), args[1].clone()),
                    _ => return Err(mlua::Error::RuntimeError(
                        "State.delete() expects 1 or 2 arguments: delete(key) or delete(scope, key)".to_string()
                    )),
                };
                if let Some(state_mgr) = &delete_state_manager {
                    let scope = Self::parse_scope(&scope_str);
                    let runtime = tokio::runtime::Handle::try_current()
                        .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!("No tokio runtime: {}", e))
                        })?;

                    let result = runtime.block_on(async { state_mgr.delete(scope, &key).await });

                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(mlua::Error::RuntimeError(format!(
                            "State delete error: {}",
                            e
                        ))),
                    }
                } else {
                    // Fallback to in-memory storage
                    let full_key = format!("{}:{}", scope_str, key);
                    let mut state = delete_fallback_state.write();
                    state.remove(&full_key);
                    Ok(())
                }
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create State.delete: {}", e),
                source: None,
            })?;

        state_table
            .set("delete", delete_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set State.delete: {}", e),
                source: None,
            })?;

        // List method - get all keys for a scope
        let list_state_manager = state_manager.clone();
        let list_fallback_state = fallback_state.clone();
        let list_fn = lua
            .create_function(move |lua, scope_str: Option<String>| {
                let scope_str = scope_str.unwrap_or_else(|| "Global".to_string());
                if let Some(state_mgr) = &list_state_manager {
                    let scope = Self::parse_scope(&scope_str);
                    let runtime = tokio::runtime::Handle::try_current()
                        .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!("No tokio runtime: {}", e))
                        })?;

                    let result = runtime.block_on(async { state_mgr.list_keys(scope).await });

                    match result {
                        Ok(keys) => {
                            let table = lua.create_table()?;
                            for (i, key) in keys.iter().enumerate() {
                                table.set(i + 1, key.clone())?;
                            }
                            Ok(table)
                        }
                        Err(e) => Err(mlua::Error::RuntimeError(format!(
                            "State list error: {}",
                            e
                        ))),
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
            })
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create State.list: {}", e),
                source: None,
            })?;

        state_table
            .set("list", list_fn)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set State.list: {}", e),
                source: None,
            })?;

        lua.globals()
            .set("State", state_table)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to set State global: {}", e),
                source: None,
            })?;

        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO (Phase 5): JavaScript State implementation - stub for now
        Ok(())
    }
}

impl Default for StateGlobal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_global_metadata() {
        let global = StateGlobal::new();
        let metadata = global.metadata();
        assert_eq!(metadata.name, "State");
        assert_eq!(metadata.version, "1.0.0"); // Phase 5 version with StateManager
    }

    #[test]
    fn test_state_in_memory_storage() {
        let global = StateGlobal::new();

        // Test basic storage operations (fallback mode)
        {
            let mut state = global.fallback_state.write();
            state.insert("test_key".to_string(), serde_json::json!("test_value"));
        }

        {
            let state = global.fallback_state.read();
            assert_eq!(
                state.get("test_key"),
                Some(&serde_json::json!("test_value"))
            );
        }
    }
}
