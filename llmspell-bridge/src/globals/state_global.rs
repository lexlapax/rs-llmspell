//! ABOUTME: State global object providing state management (placeholder for Phase 5)
//! ABOUTME: Minimal in-memory implementation preparing for persistent state in Phase 5

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::error::LLMSpellError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// State global object providing state management
///
/// NOTE: This is a placeholder implementation with in-memory storage only.
/// Full persistent state with sled/rocksdb, migrations, and backup/restore
/// will be implemented in Phase 5.
pub struct StateGlobal {
    /// In-memory state storage (placeholder for Phase 5 persistent storage)
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl StateGlobal {
    /// Create a new State global
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl GlobalObject for StateGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "State".to_string(),
            version: "0.1.0".to_string(), // Placeholder version
            description: "State management system (in-memory placeholder)".to_string(),
            dependencies: vec![],
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, _context: &GlobalContext) -> Result<(), LLMSpellError> {
        let state_clone = self.state.clone();

        // Create State table
        let state_table = lua.create_table().map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create State table: {}", e),
            source: None,
        })?;

        // Get method - retrieve state value
        let state_get = state_clone.clone();
        let get_fn = lua
            .create_function(move |lua, key: String| {
                let state = state_get.read();
                match state.get(&key) {
                    Some(value) => {
                        let lua_value = crate::lua::conversion::json_to_lua_value(lua, value)?;
                        Ok(lua_value)
                    }
                    None => Ok(mlua::Value::Nil),
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

        // Set method - store state value
        let state_set = state_clone.clone();
        let set_fn = lua
            .create_function(move |_, (key, value): (String, mlua::Value)| {
                let json_value = crate::lua::conversion::lua_value_to_json(value)?;
                let mut state = state_set.write();
                state.insert(key, json_value);
                // TODO: Phase 5 - Persist to storage backend
                Ok(())
            })
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

        // Delete method
        let state_delete = state_clone.clone();
        let delete_fn = lua
            .create_function(move |_, key: String| {
                let mut state = state_delete.write();
                state.remove(&key);
                // TODO: Phase 5 - Remove from storage backend
                Ok(())
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

        // List method - get all keys
        let state_list = state_clone;
        let list_fn = lua
            .create_function(move |lua, ()| {
                let state = state_list.read();
                let keys: Vec<String> = state.keys().cloned().collect();

                let table = lua.create_table()?;
                for (i, key) in keys.iter().enumerate() {
                    table.set(i + 1, key.clone())?;
                }
                Ok(table)
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
        assert_eq!(metadata.version, "0.1.0"); // Placeholder version
    }

    #[test]
    fn test_state_in_memory_storage() {
        let global = StateGlobal::new();

        // Test basic storage operations
        {
            let mut state = global.state.write();
            state.insert("test_key".to_string(), serde_json::json!("test_value"));
        }

        {
            let state = global.state.read();
            assert_eq!(
                state.get("test_key"),
                Some(&serde_json::json!("test_value"))
            );
        }
    }
}
