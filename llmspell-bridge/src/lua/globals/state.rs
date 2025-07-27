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
    let list_state_manager = state_manager.clone();
    let list_fallback_state = fallback_state.clone();
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

    // Migration methods (available when migration support is present in state_global)
    if let (Some(migration_engine), Some(schema_registry)) = (
        &state_global.migration_engine,
        &state_global.schema_registry,
    ) {
        // migrate(target_version) - Trigger migration to target schema version
        let migrate_engine = migration_engine.clone();
        let migrate_registry = schema_registry.clone();
        let migrate_fn = lua.create_function(move |lua, target_version: String| {
            let runtime = tokio::runtime::Handle::try_current()
                .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
                .map_err(|e| mlua::Error::RuntimeError(format!("No tokio runtime: {}", e)))?;

            // Parse semantic version
            let target_ver: llmspell_state_persistence::schema::SemanticVersion =
                target_version.parse().map_err(|e| {
                    mlua::Error::RuntimeError(format!(
                        "Invalid version '{}': {}",
                        target_version, e
                    ))
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

            let result = runtime.block_on(async {
                migrate_engine
                    .migrate(&current_ver, &target_ver, migration_config)
                    .await
            });

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
                    result_table
                        .set("duration_ms", migration_result.duration.as_millis() as u64)?;

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
                    result_table.set("error", format!("Migration failed: {}", e))?;
                    result_table.set("from_version", current_ver.to_string())?;
                    result_table.set("target_version", target_ver.to_string())?;
                    Ok(result_table)
                }
            }
        })?;
        state_table.set("migrate", migrate_fn)?;

        // migration_status() - Get migration status information
        let status_registry = schema_registry.clone();
        let status_fn = lua.create_function(move |lua, _: ()| {
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
        state_table.set("migration_status", status_fn)?;

        // schema_versions() - List all available schema versions
        let versions_registry = schema_registry.clone();
        let versions_fn = lua.create_function(move |lua, _: ()| {
            let versions = versions_registry.list_versions();
            let versions_table = lua.create_table()?;

            for (i, version) in versions.iter().enumerate() {
                versions_table.set(i + 1, version.to_string())?;
            }

            Ok(versions_table)
        })?;
        state_table.set("schema_versions", versions_fn)?;
    }

    // Backup methods (available when backup manager is configured)
    if let Some(ref backup_mgr) = state_global.backup_manager {
        // create_backup(incremental) - Create a backup (full or incremental)
        let _backup_mgr_clone = backup_mgr.clone();
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
        let list_backups_fn = lua.create_function(move |lua, _: ()| {
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
        let get_storage_usage_fn = lua.create_function(move |lua, _: ()| {
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
    }

    // Set the state table as a global
    lua.globals().set("State", state_table)?;

    Ok(())
}
