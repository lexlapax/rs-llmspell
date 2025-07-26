//! ABOUTME: State global object providing persistent state management
//! ABOUTME: Integrates with StateManager for full persistent state functionality

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::error::LLMSpellError;
use llmspell_state_persistence::{
    migration::MigrationEngine,
    schema::{SchemaRegistry, SemanticVersion},
    StateManager, StateScope,
};
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
    /// Migration engine for schema transitions (optional)
    pub migration_engine: Option<Arc<MigrationEngine>>,
    /// Schema registry for migration planning (optional)
    pub schema_registry: Option<Arc<SchemaRegistry>>,
}

impl StateGlobal {
    /// Create a new State global without StateManager (fallback mode)
    pub fn new() -> Self {
        Self {
            state_manager: None,
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: None,
            schema_registry: None,
        }
    }

    /// Create a new State global with StateManager integration
    pub fn with_state_manager(state_manager: Arc<StateManager>) -> Self {
        Self {
            state_manager: Some(state_manager),
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: None,
            schema_registry: None,
        }
    }

    /// Create a new State global with full migration support
    pub fn with_migration_support(
        state_manager: Arc<StateManager>,
        migration_engine: Arc<MigrationEngine>,
        schema_registry: Arc<SchemaRegistry>,
    ) -> Self {
        Self {
            state_manager: Some(state_manager),
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: Some(migration_engine),
            schema_registry: Some(schema_registry),
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
                    // Use sync_utils to bridge async operation
                    use crate::lua::sync_utils::block_on_async;

                    let result = block_on_async(
                        "state_get",
                        async move { state_mgr.get(scope, &key).await },
                        None,
                    );

                    match result {
                        Ok(Some(value)) => {
                            let lua_value = crate::lua::conversion::json_to_lua_value(lua, &value)?;
                            Ok(lua_value)
                        }
                        Ok(None) => Ok(mlua::Value::Nil),
                        Err(e) => Err(e),
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
                        // Use sync_utils to bridge async operation
                        use crate::lua::sync_utils::block_on_async;

                        let result = block_on_async(
                            "state_set",
                            async move { state_mgr.set(scope, &key, json_value).await },
                            None
                        );

                        match result {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e)
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
                    // Use sync_utils to bridge async operation
                    use crate::lua::sync_utils::block_on_async;

                    let result = block_on_async(
                        "state_delete",
                        async move { state_mgr.delete(scope, &key).await },
                        None
                    );

                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e)
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
                    // Use sync_utils to bridge async operation
                    use crate::lua::sync_utils::block_on_async;

                    let result = block_on_async(
                        "state_list_keys",
                        async move { state_mgr.list_keys(scope).await },
                        None,
                    );

                    match result {
                        Ok(keys) => {
                            let table = lua.create_table()?;
                            for (i, key) in keys.iter().enumerate() {
                                table.set(i + 1, key.clone())?;
                            }
                            Ok(table)
                        }
                        Err(e) => Err(e),
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

        // Migration methods (only available when migration support is enabled)
        if let (Some(migration_engine), Some(schema_registry)) =
            (&self.migration_engine, &self.schema_registry)
        {
            // migrate_to_version(target_version) - Trigger migration to target schema version
            let migrate_engine = migration_engine.clone();
            let migrate_registry = schema_registry.clone();
            let migrate_fn = lua
                .create_function(move |lua, target_version: String| {
                    // Use sync_utils for async operations
                    use crate::lua::sync_utils::block_on_async;

                    // Parse target version
                    let target_ver: SemanticVersion = target_version.parse().map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Invalid version format '{}': {}",
                            target_version, e
                        ))
                    })?;

                    // Get current schema version from registry
                    let current_schema =
                        migrate_registry.get_current_schema().ok_or_else(|| {
                            mlua::Error::RuntimeError("No current schema found".to_string())
                        })?;
                    let current_ver = current_schema.version.clone();

                    if current_ver == target_ver {
                        // Already at target version - return success result
                        let result_table = lua.create_table()?;
                        result_table.set("success", true)?;
                        result_table.set("status", "completed")?;
                        result_table.set("from_version", current_ver.to_string())?;
                        result_table.set("to_version", target_ver.to_string())?;
                        result_table.set("message", "Already at target version")?;
                        return Ok(result_table);
                    }

                    // Create migration config with default settings
                    let migration_config = llmspell_state_persistence::migration::MigrationConfig {
                        dry_run: false,
                        create_backup: true,
                        batch_size: 100,
                        timeout: std::time::Duration::from_secs(300),
                        max_concurrent_migrations: 1,
                        validation_level:
                            llmspell_state_persistence::migration::ValidationLevel::Strict,
                        rollback_on_error: true,
                    };

                    // Execute migration
                    let migrate_engine_clone = migrate_engine.clone();
                    let current_ver_clone = current_ver.clone();
                    let target_ver_clone = target_ver.clone();

                    let result = block_on_async(
                        "state_migrate",
                        async move {
                            migrate_engine_clone
                                .migrate(&current_ver_clone, &target_ver_clone, migration_config)
                                .await
                        },
                        None,
                    );

                    match result {
                        Ok(migration_result) => {
                            let result_table = lua.create_table()?;
                            result_table.set("success", true)?;
                            result_table.set("status", format!("{:?}", migration_result.status))?;
                            result_table
                                .set("from_version", migration_result.from_version.to_string())?;
                            result_table
                                .set("to_version", migration_result.to_version.to_string())?;
                            result_table.set("items_migrated", migration_result.items_migrated)?;
                            result_table
                                .set("duration_ms", migration_result.duration.as_millis() as u64)?;

                            // Add warnings if any
                            if !migration_result.warnings.is_empty() {
                                let warnings_table = lua.create_table()?;
                                for (i, warning) in migration_result.warnings.iter().enumerate() {
                                    warnings_table.set(i + 1, warning.clone())?;
                                }
                                result_table.set("warnings", warnings_table)?;
                            }

                            Ok(result_table)
                        }
                        Err(e) => {
                            let result_table = lua.create_table()?;
                            result_table.set("success", false)?;
                            result_table.set("error", format!("Migration failed: {}", e))?;
                            result_table.set("from_version", current_ver.to_string())?;
                            result_table.set("to_version", target_ver.to_string())?;
                            Ok(result_table)
                        }
                    }
                })
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to create State.migrate_to_version: {}", e),
                    source: None,
                })?;

            state_table
                .set("migrate_to_version", migrate_fn)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to set State.migrate_to_version: {}", e),
                    source: None,
                })?;

            // get_migration_status() - Get current migration information
            let status_registry = schema_registry.clone();
            let status_fn = lua
                .create_function(move |lua, _: ()| {
                    let status_table = lua.create_table()?;

                    if let Some(current_schema) = status_registry.get_current_schema() {
                        status_table.set("current_version", current_schema.version.to_string())?;

                        let stats = status_registry.get_stats();
                        status_table.set("total_schemas", stats.total_schemas)?;
                        status_table.set("major_versions_count", stats.major_versions_count)?;

                        if let Some(latest) = stats.latest_version {
                            status_table.set("latest_version", latest.to_string())?;
                        }
                        if let Some(oldest) = stats.oldest_version {
                            status_table.set("oldest_version", oldest.to_string())?;
                        }

                        // Get available migration targets
                        let compatible_versions =
                            status_registry.find_compatible_schemas(&current_schema.version);
                        let migration_candidates =
                            status_registry.find_migration_candidates(&current_schema.version);

                        let compatible_table = lua.create_table()?;
                        for (i, version) in compatible_versions.iter().enumerate() {
                            compatible_table.set(i + 1, version.to_string())?;
                        }
                        status_table.set("compatible_versions", compatible_table)?;

                        let candidates_table = lua.create_table()?;
                        for (i, version) in migration_candidates.iter().enumerate() {
                            candidates_table.set(i + 1, version.to_string())?;
                        }
                        status_table.set("migration_candidates", candidates_table)?;

                        status_table.set("migration_available", true)?;
                    } else {
                        status_table.set("current_version", "unknown")?;
                        status_table.set("migration_available", false)?;
                        status_table.set("error", "No current schema found")?;
                    }

                    Ok(status_table)
                })
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to create State.get_migration_status: {}", e),
                    source: None,
                })?;

            state_table
                .set("get_migration_status", status_fn)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to set State.get_migration_status: {}", e),
                    source: None,
                })?;

            // list_schema_versions() - List all available schema versions
            let list_registry = schema_registry.clone();
            let list_versions_fn = lua
                .create_function(move |lua, _: ()| {
                    let versions = list_registry.list_versions();
                    let versions_table = lua.create_table()?;

                    for (i, version) in versions.iter().enumerate() {
                        versions_table.set(i + 1, version.to_string())?;
                    }

                    Ok(versions_table)
                })
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to create State.list_schema_versions: {}", e),
                    source: None,
                })?;

            state_table
                .set("list_schema_versions", list_versions_fn)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to set State.list_schema_versions: {}", e),
                    source: None,
                })?;
        }

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
