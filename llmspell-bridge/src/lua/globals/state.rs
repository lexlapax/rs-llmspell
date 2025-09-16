//! ABOUTME: Lua-specific State global implementation
//! ABOUTME: Provides Lua bindings for persistent state functionality

#![allow(clippy::significant_drop_tightening)]

use crate::globals::{state_global::StateGlobal, GlobalContext};
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use llmspell_core::traits::state::StateAccess;
use mlua::{Error as LuaError, Lua, Value};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument};

/// Create save operation handler
fn create_save_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String, Value)) -> mlua::Result<()> {
    move |_, (scope_str, key, value): (String, String, Value)| {
        let json_value = lua_value_to_json(value)?;
        let full_key = format!("{scope_str}:{key}");

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "state_save",
                async move { state_clone.write(&full_key, json_value).await },
                None,
            );

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            let mut state = fallback_state.write();
            state.insert(full_key, json_value);
            Ok(())
        }
    }
}

/// Create load operation handler
fn create_load_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String)) -> mlua::Result<Value> {
    move |lua, (scope_str, key): (String, String)| {
        // For custom scope with workflow keys, construct the key properly
        // The StateAccess will add its own prefix, so we just pass the key part
        let full_key = format!("{scope_str}:{key}");
        let actual_key = if scope_str == "custom" && key.starts_with(':') {
            // Remove leading colon for custom::workflow keys
            key[1..].to_string()
        } else {
            key.clone()
        };
        info!(
            "Lua State.load: scope='{}', key='{}', actual_key='{}', full_key='{}'",
            scope_str, key, actual_key, full_key
        );

        state_access.as_ref().map_or_else(
            || {
                let state = fallback_state.read();
                state
                    .get(&full_key)
                    .map_or_else(|| Ok(Value::Nil), |value| json_to_lua_value(lua, value))
            },
            |state| {
                // Use block_on_async utility for proper async-to-sync conversion
                // Pass the actual_key which will get the Custom("") prefix from StateManagerAdapter
                let state_clone = state.clone();
                let result = crate::lua::sync_utils::block_on_async(
                    "state_load",
                    async move { state_clone.read(&actual_key).await },
                    None,
                );

                match result {
                    Ok(Some(value)) => json_to_lua_value(lua, &value),
                    Ok(None) => Ok(Value::Nil),
                    Err(e) => Err(e),
                }
            },
        )
    }
}

/// Create delete operation handler
fn create_delete_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String)) -> mlua::Result<()> {
    move |_, (scope_str, key): (String, String)| {
        let full_key = format!("{scope_str}:{key}");

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "state_delete",
                async move { state_clone.delete(&full_key).await },
                None,
            );

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(LuaError::RuntimeError(format!("State delete error: {e}"))),
            }
        } else {
            let mut state = fallback_state.write();
            state.remove(&full_key);
            Ok(())
        }
    }
}

/// Create `list_keys` operation handler
fn create_list_keys_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, String) -> mlua::Result<mlua::Table> {
    move |lua, scope_str: String| {
        if let Some(state) = &state_access {
            let prefix = format!("{scope_str}:");
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let prefix_clone = prefix.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "state_list_keys",
                async move { state_clone.list_keys(&prefix_clone).await },
                None,
            );

            match result {
                Ok(keys) => {
                    let table = lua.create_table()?;
                    for (i, key) in keys.iter().enumerate() {
                        let stripped_key = key.strip_prefix(&prefix).unwrap_or(key);
                        table.set(i + 1, stripped_key.to_string())?;
                    }
                    Ok(table)
                }
                Err(e) => Err(LuaError::RuntimeError(format!("State list error: {e}"))),
            }
        } else {
            let state = fallback_state.read();
            let prefix = format!("{scope_str}:");
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
    }
}

/// Create `workflow_get` operation handler
fn create_workflow_get_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String)) -> mlua::Result<Value> {
    move |lua, (workflow_id, step_name): (String, String)| {
        let key = format!("workflow:{workflow_id}:{step_name}");

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "workflow_get",
                async move { state_clone.read(&key).await },
                None,
            );

            match result {
                Ok(Some(value)) => json_to_lua_value(lua, &value),
                Ok(None) => Ok(Value::Nil),
                Err(e) => Err(LuaError::RuntimeError(format!("State read error: {e}"))),
            }
        } else {
            let state = fallback_state.read();
            state
                .get(&key)
                .map_or_else(|| Ok(Value::Nil), |value| json_to_lua_value(lua, value))
        }
    }
}

/// Create `workflow_list` operation handler
fn create_workflow_list_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, String) -> mlua::Result<Value> {
    move |lua, workflow_id: String| {
        let prefix = format!("workflow:{workflow_id}:");

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let prefix_clone = prefix.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "workflow_list",
                async move { state_clone.list_keys(&prefix_clone).await },
                None,
            );

            match result {
                Ok(keys) => {
                    let table = lua.create_table()?;
                    for (i, key) in keys.iter().enumerate() {
                        if let Some(step_name) = key.strip_prefix(&prefix) {
                            table.set(i + 1, step_name)?;
                        }
                    }
                    Ok(Value::Table(table))
                }
                Err(e) => Err(LuaError::RuntimeError(format!("State list error: {e}"))),
            }
        } else {
            let state = fallback_state.read();
            let table = lua.create_table()?;
            let mut index = 1;
            for key in state.keys() {
                if key.starts_with(&prefix) {
                    if let Some(step_name) = key.strip_prefix(&prefix) {
                        table.set(index, step_name)?;
                        index += 1;
                    }
                }
            }
            Ok(Value::Table(table))
        }
    }
}

/// Create `agent_get` operation handler
fn create_agent_get_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String)) -> mlua::Result<Value> {
    move |lua, (agent_id, key): (String, String)| {
        let full_key = format!("agent:{agent_id}:{key}");

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "agent_get",
                async move { state_clone.read(&full_key).await },
                None,
            );

            match result {
                Ok(Some(value)) => json_to_lua_value(lua, &value),
                Ok(None) => Ok(Value::Nil),
                Err(e) => Err(LuaError::RuntimeError(format!("State read error: {e}"))),
            }
        } else {
            let state = fallback_state.read();
            state
                .get(&full_key)
                .map_or_else(|| Ok(Value::Nil), |value| json_to_lua_value(lua, value))
        }
    }
}

/// Create `agent_set` operation handler
fn create_agent_set_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String, Value)) -> mlua::Result<()> {
    move |_, (agent_id, key, value): (String, String, Value)| {
        let full_key = format!("agent:{agent_id}:{key}");
        let json_value = lua_value_to_json(value)?;

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "agent_set",
                async move { state_clone.write(&full_key, json_value).await },
                None,
            );

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(LuaError::RuntimeError(format!("State write error: {e}"))),
            }
        } else {
            let mut state = fallback_state.write();
            state.insert(full_key, json_value);
            Ok(())
        }
    }
}

/// Create `tool_get` operation handler
fn create_tool_get_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String)) -> mlua::Result<Value> {
    move |lua, (tool_id, key): (String, String)| {
        let full_key = format!("tool:{tool_id}:{key}");

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "agent_get",
                async move { state_clone.read(&full_key).await },
                None,
            );

            match result {
                Ok(Some(value)) => json_to_lua_value(lua, &value),
                Ok(None) => Ok(Value::Nil),
                Err(e) => Err(LuaError::RuntimeError(format!("State read error: {e}"))),
            }
        } else {
            let state = fallback_state.read();
            state
                .get(&full_key)
                .map_or_else(|| Ok(Value::Nil), |value| json_to_lua_value(lua, value))
        }
    }
}

/// Create `tool_set` operation handler
fn create_tool_set_handler(
    state_access: Option<Arc<dyn StateAccess>>,
    fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
) -> impl Fn(&Lua, (String, String, Value)) -> mlua::Result<()> {
    move |_, (tool_id, key, value): (String, String, Value)| {
        let full_key = format!("tool:{tool_id}:{key}");
        let json_value = lua_value_to_json(value)?;

        if let Some(state) = &state_access {
            // Use block_on_async utility for proper async-to-sync conversion
            let state_clone = state.clone();
            let result = crate::lua::sync_utils::block_on_async(
                "agent_set",
                async move { state_clone.write(&full_key, json_value).await },
                None,
            );

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(LuaError::RuntimeError(format!("State write error: {e}"))),
            }
        } else {
            let mut state = fallback_state.write();
            state.insert(full_key, json_value);
            Ok(())
        }
    }
}

/// Set up migration methods on state table
#[allow(clippy::too_many_lines)]
fn setup_migration_methods(
    lua: &Lua,
    state_table: &mlua::Table,
    migration_engine: &Arc<llmspell_state_persistence::migration::MigrationEngine>,
    schema_registry: &Arc<llmspell_state_persistence::schema::SchemaRegistry>,
) -> mlua::Result<()> {
    // migrate(target_version) - Trigger migration to target schema version
    let migrate_engine = migration_engine.clone();
    let migrate_registry = schema_registry.clone();
    let migrate_fn = lua.create_function(move |lua, target_version: String| {
        // Parse semantic version
        let target_ver: llmspell_state_persistence::schema::SemanticVersion =
            target_version.parse().map_err(|e| {
                mlua::Error::RuntimeError(format!("Invalid version '{target_version}': {e}"))
            })?;

        // Get current version
        let current_schema = migrate_registry
            .get_current_schema()
            .ok_or_else(|| mlua::Error::RuntimeError("No current schema found".to_string()))?;
        let current_ver = current_schema.version.clone();

        if current_ver == target_ver {
            let result_table = lua.create_table()?;
            result_table.set("success", true)?;
            result_table.set("status", "already_current")?;
            result_table.set("current_version", current_ver.to_string())?;
            result_table.set("target_version", target_ver.to_string())?;
            return Ok(result_table);
        }

        // Execute migration
        let migration_config = llmspell_state_persistence::migration::MigrationConfig {
            dry_run: false,
            create_backup: true,
            batch_size: 100,
            timeout: std::time::Duration::from_secs(300),
            max_concurrent_migrations: 1,
            validation_level: llmspell_state_persistence::migration::ValidationLevel::Strict,
            rollback_on_error: true,
        };

        let engine_clone = migrate_engine.clone();
        let current_ver_clone = current_ver.clone();
        let target_ver_clone = target_ver.clone();
        let result = crate::lua::sync_utils::block_on_async(
            "state_migrate",
            async move {
                engine_clone
                    .migrate(&current_ver_clone, &target_ver_clone, migration_config)
                    .await
            },
            Some(std::time::Duration::from_secs(300)),
        );

        match result {
            Ok(migration_result) => {
                let result_table = lua.create_table()?;
                result_table.set("success", true)?;
                result_table.set(
                    "status",
                    format!("{:?}", migration_result.status).to_lowercase(),
                )?;
                result_table.set("from_version", migration_result.from_version.to_string())?;
                result_table.set("to_version", migration_result.to_version.to_string())?;
                result_table.set("items_migrated", migration_result.items_migrated)?;
                result_table.set(
                    "duration_ms",
                    u64::try_from(migration_result.duration.as_millis()).unwrap_or(u64::MAX),
                )?;

                if !migration_result.warnings.is_empty() {
                    let warnings_table = lua.create_table()?;
                    for (i, warning) in migration_result.warnings.iter().enumerate() {
                        warnings_table.set(i + 1, warning.clone())?;
                    }
                    result_table.set("warnings", warnings_table)?;
                }

                if !migration_result.errors.is_empty() {
                    let errors_table = lua.create_table()?;
                    for (i, error) in migration_result.errors.iter().enumerate() {
                        errors_table.set(i + 1, error.clone())?;
                    }
                    result_table.set("errors", errors_table)?;
                }

                Ok(result_table)
            }
            Err(e) => {
                let result_table = lua.create_table()?;
                result_table.set("success", false)?;
                result_table.set("error", format!("Migration failed: {e}"))?;
                result_table.set("from_version", current_ver.to_string())?;
                result_table.set("target_version", target_ver.to_string())?;
                Ok(result_table)
            }
        }
    })?;
    state_table.set("migrate", migrate_fn)?;

    // get_migration_status() - Get migration status information
    let status_registry = schema_registry.clone();
    let status_fn = lua.create_function(move |lua, (): ()| {
        let status_table = lua.create_table()?;

        if let Some(current_schema) = status_registry.get_current_schema() {
            status_table.set("current_version", current_schema.version.to_string())?;
            status_table.set("migration_available", true)?;

            let stats = status_registry.get_stats();
            status_table.set("total_schemas", stats.total_schemas)?;

            if let Some(latest) = stats.latest_version {
                status_table.set("latest_version", latest.to_string())?;
                status_table.set("is_latest", current_schema.version == latest)?;
            }

            // Get migration candidates
            let candidates = status_registry.find_migration_candidates(&current_schema.version);
            let candidates_table = lua.create_table()?;
            for (i, version) in candidates.iter().enumerate() {
                candidates_table.set(i + 1, version.to_string())?;
            }
            status_table.set("migration_targets", candidates_table)?;
        } else {
            status_table.set("current_version", "unknown")?;
            status_table.set("migration_available", false)?;
            status_table.set("error", "No current schema found")?;
        }

        Ok(status_table)
    })?;
    state_table.set("get_migration_status", status_fn)?;

    // get_schema_versions() - List all available schema versions
    let versions_registry = schema_registry.clone();
    let versions_fn = lua.create_function(move |lua, (): ()| {
        let versions = versions_registry.list_versions();
        let versions_table = lua.create_table()?;

        for (i, version) in versions.iter().enumerate() {
            versions_table.set(i + 1, version.to_string())?;
        }

        Ok(versions_table)
    })?;
    state_table.set("get_schema_versions", versions_fn)?;

    Ok(())
}

/// Set up backup methods on state table
fn setup_backup_methods(
    lua: &Lua,
    state_table: &mlua::Table,
    _backup_mgr: &Arc<llmspell_state_persistence::backup::BackupManager>,
) -> mlua::Result<()> {
    // create_backup(incremental) - Create a backup (full or incremental)
    let create_backup_fn = lua.create_function(move |lua, incremental: Option<bool>| {
        let incremental = incremental.unwrap_or(false);

        // TODO: Implement actual backup creation when BackupManager is integrated
        let result_table = lua.create_table()?;
        result_table.set("success", false)?;
        result_table.set("error", "Backup functionality not yet implemented")?;
        result_table.set("incremental", incremental)?;

        Ok(result_table)
    })?;
    state_table.set("create_backup", create_backup_fn)?;

    // list_backups() - List available backups
    let list_backups_fn = lua.create_function(move |lua, (): ()| {
        // TODO: Implement actual backup listing
        let backups_table = lua.create_table()?;

        // Return empty list for now
        Ok(backups_table)
    })?;
    state_table.set("list_backups", list_backups_fn)?;

    // restore_backup(backup_id) - Restore from a specific backup
    let restore_backup_fn = lua.create_function(move |lua, backup_id: String| {
        // TODO: Implement actual backup restoration
        let result_table = lua.create_table()?;
        result_table.set("success", false)?;
        result_table.set("error", "Restore functionality not yet implemented")?;
        result_table.set("backup_id", backup_id)?;

        Ok(result_table)
    })?;
    state_table.set("restore_backup", restore_backup_fn)?;

    // validate_backup(backup_id) - Validate a backup
    let validate_backup_fn = lua.create_function(move |lua, backup_id: String| {
        // TODO: Implement actual backup validation
        let result_table = lua.create_table()?;
        result_table.set("is_valid", false)?;
        result_table.set("error", "Validation functionality not yet implemented")?;
        result_table.set("backup_id", backup_id)?;

        Ok(result_table)
    })?;
    state_table.set("validate_backup", validate_backup_fn)?;

    // get_storage_usage() - Get backup storage usage information
    let get_storage_usage_fn = lua.create_function(move |lua, (): ()| {
        // TODO: Implement actual storage usage calculation
        let usage_table = lua.create_table()?;
        usage_table.set("total_backups", 0)?;
        usage_table.set("total_size", 0)?;
        usage_table.set("full_backups", 0)?;
        usage_table.set("incremental_backups", 0)?;
        usage_table.set("oldest_backup", mlua::Value::Nil)?;
        usage_table.set("newest_backup", mlua::Value::Nil)?;

        Ok(usage_table)
    })?;
    state_table.set("get_storage_usage", get_storage_usage_fn)?;

    // cleanup_backups(dry_run) - Apply retention policies and cleanup old backups
    let cleanup_backups_fn = lua.create_function(move |lua, dry_run: Option<bool>| {
        let dry_run = dry_run.unwrap_or(false);

        // TODO: Implement actual cleanup when BackupManager is integrated
        let result_table = lua.create_table()?;
        result_table.set("success", false)?;
        result_table.set("error", "Cleanup functionality not yet implemented")?;
        result_table.set("dry_run", dry_run)?;
        result_table.set("deleted_count", 0)?;
        result_table.set("retained_count", 0)?;
        result_table.set("space_freed", 0)?;

        Ok(result_table)
    })?;
    state_table.set("cleanup_backups", cleanup_backups_fn)?;

    Ok(())
}

/// Inject State global into Lua environment
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function binding fails
#[instrument(level = "info", skip(lua, _context, state_global), fields(
    global_name = "State",
    has_state_backend = state_global.state_access.is_some()
))]
pub fn inject_state_global(
    lua: &Lua,
    _context: &GlobalContext,
    state_global: &StateGlobal,
) -> mlua::Result<()> {
    info!("Injecting State global API");
    let state_table = lua.create_table()?;

    // Clone references for the closures
    let state_access = state_global.state_access.clone();
    let _state_manager = state_global.state_manager.clone(); // Keep for migration/backup features
    let fallback_state = state_global.fallback_state.clone();

    info!(
        "inject_state_global: state_access is_some: {}",
        state_access.is_some()
    );

    // Basic operations
    state_table.set(
        "save",
        lua.create_function(create_save_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;
    state_table.set(
        "load",
        lua.create_function(create_load_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;
    state_table.set(
        "delete",
        lua.create_function(create_delete_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;
    state_table.set(
        "list_keys",
        lua.create_function(create_list_keys_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;

    // Workflow helpers
    state_table.set(
        "workflow_get",
        lua.create_function(create_workflow_get_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;
    state_table.set(
        "workflow_list",
        lua.create_function(create_workflow_list_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;

    // Agent helpers
    state_table.set(
        "agent_get",
        lua.create_function(create_agent_get_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;
    state_table.set(
        "agent_set",
        lua.create_function(create_agent_set_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;

    // Tool helpers
    state_table.set(
        "tool_get",
        lua.create_function(create_tool_get_handler(
            state_access.clone(),
            fallback_state.clone(),
        ))?,
    )?;
    state_table.set(
        "tool_set",
        lua.create_function(create_tool_set_handler(
            state_access.clone(),
            fallback_state,
        ))?,
    )?;

    // Migration methods (available when migration support is present in state_global)
    if let (Some(migration_engine), Some(schema_registry)) = (
        &state_global.migration_engine,
        &state_global.schema_registry,
    ) {
        setup_migration_methods(lua, &state_table, migration_engine, schema_registry)?;
    }

    // Backup methods (available when backup manager is configured)
    if let Some(ref backup_mgr) = state_global.backup_manager {
        setup_backup_methods(lua, &state_table, backup_mgr)?;
    }

    // Set the state table as a global
    lua.globals().set("State", state_table)?;

    Ok(())
}
