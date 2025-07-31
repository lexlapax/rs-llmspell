//! ABOUTME: Lua-specific Artifact global implementation
//! ABOUTME: Provides Lua bindings for artifact management functionality

use crate::artifact_bridge::ArtifactBridge;
use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use crate::lua::sync_utils::block_on_async;
use crate::session_bridge::SessionBridge;
use llmspell_sessions::{artifact::ArtifactId, SessionId};
use mlua::{Error as LuaError, Lua, Table, Value};
use std::str::FromStr;
use std::sync::Arc;

/// Inject Artifact global into Lua environment
pub fn inject_artifact_global(
    lua: &Lua,
    _context: &GlobalContext,
    artifact_bridge: Arc<ArtifactBridge>,
) -> mlua::Result<()> {
    // Create Artifact table
    let artifact_table = lua.create_table()?;

    // Store method - store an artifact
    let store_bridge = artifact_bridge.clone();
    let store_fn = lua.create_function(
        move |lua, args: (String, String, String, Value, Option<Table>)| {
            let (session_id_str, type_str, name, content, metadata) = args;

            // Parse session ID
            let session_id = SessionId::from_str(&session_id_str)
                .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {}", e)))?;

            // Parse artifact type
            let artifact_type =
                llmspell_sessions::bridge::conversions::parse_artifact_type(&type_str)
                    .map_err(mlua::Error::RuntimeError)?;

            // Convert content to bytes
            let content_bytes = match content {
                Value::String(s) => s.as_bytes().to_vec(),
                Value::UserData(ud) => {
                    // Assume userdata is a byte array
                    ud.borrow::<Vec<u8>>()?.clone()
                }
                _ => {
                    return Err(LuaError::RuntimeError(
                        "Content must be string or byte array".to_string(),
                    ))
                }
            };

            // Convert metadata table to HashMap
            let metadata_map = if let Some(table) = metadata {
                let mut map = std::collections::HashMap::new();
                for pair in table.pairs::<String, Value>() {
                    let (k, v) = pair?;
                    let json_value = lua_value_to_json(v)?;
                    map.insert(k, json_value);
                }
                Some(map)
            } else {
                None
            };

            let bridge = store_bridge.clone();
            let result = block_on_async(
                "artifact_store",
                async move {
                    bridge
                        .store_artifact(
                            &session_id,
                            artifact_type,
                            name,
                            content_bytes,
                            metadata_map,
                        )
                        .await
                },
                None,
            )?;

            // Return artifact ID as table with fields
            let id_table = lua.create_table()?;
            id_table.set("content_hash", result.content_hash)?;
            id_table.set("session_id", result.session_id.to_string())?;
            id_table.set("sequence", result.sequence)?;
            Ok(id_table)
        },
    )?;
    artifact_table.set("store", store_fn)?;

    // Get method - get artifact content and metadata
    let get_bridge = artifact_bridge.clone();
    let get_fn = lua.create_function(move |lua, args: (String, Table)| {
        let (session_id_str, artifact_id_table) = args;

        // Parse session ID
        let session_id = SessionId::from_str(&session_id_str)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {}", e)))?;

        // Parse artifact ID from table
        let content_hash: String = artifact_id_table.get("content_hash")?;
        let artifact_session_id_str: String = artifact_id_table.get("session_id")?;
        let sequence: u64 = artifact_id_table.get("sequence")?;

        let artifact_session_id = SessionId::from_str(&artifact_session_id_str)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid artifact session ID: {}", e)))?;

        let artifact_id = ArtifactId::new(content_hash, artifact_session_id, sequence);

        let bridge = get_bridge.clone();
        let result = block_on_async(
            "artifact_get",
            async move { bridge.get_artifact(&session_id, &artifact_id).await },
            None,
        )?;

        // Convert to Lua table
        let result_table = lua.create_table()?;

        // Convert metadata to JSON then to Lua
        let metadata_json =
            llmspell_sessions::bridge::conversions::artifact_metadata_to_json(&result.metadata);
        let metadata_lua = json_to_lua_value(lua, &metadata_json)?;
        result_table.set("metadata", metadata_lua)?;

        // Content - always return as Lua string (which can hold binary data)
        let content = result
            .get_content()
            .map_err(|e| LuaError::RuntimeError(format!("Failed to get content: {}", e)))?;

        // Convert Vec<u8> to Lua string (which can hold binary data)
        let content_str = lua.create_string(&content)?;
        result_table.set("content", content_str)?;

        Ok(result_table)
    })?;
    artifact_table.set("get", get_fn)?;

    // List method - list artifacts for a session
    let list_bridge = artifact_bridge.clone();
    let list_fn = lua.create_function(move |lua, session_id_str: String| {
        // Use current session if not specified
        let session_id = if session_id_str.is_empty() {
            SessionBridge::get_current_session()
                .ok_or_else(|| LuaError::RuntimeError("No current session set".to_string()))?
        } else {
            SessionId::from_str(&session_id_str)
                .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {}", e)))?
        };

        let bridge = list_bridge.clone();
        let result = block_on_async(
            "artifact_list",
            async move { bridge.list_artifacts(&session_id).await },
            None,
        )?;

        // Convert Vec<ArtifactMetadata> to Lua table
        let lua_table = lua.create_table()?;
        for (i, metadata) in result.iter().enumerate() {
            let json_value =
                llmspell_sessions::bridge::conversions::artifact_metadata_to_json(metadata);
            let lua_value = json_to_lua_value(lua, &json_value)?;
            lua_table.set(i + 1, lua_value)?;
        }
        Ok(lua_table)
    })?;
    artifact_table.set("list", list_fn)?;

    // Delete method - delete an artifact
    let delete_bridge = artifact_bridge.clone();
    let delete_fn = lua.create_function(move |_lua, args: (String, Table)| {
        let (session_id_str, artifact_id_table) = args;

        // Parse session ID
        let session_id = SessionId::from_str(&session_id_str)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {}", e)))?;

        // Parse artifact ID from table
        let content_hash: String = artifact_id_table.get("content_hash")?;
        let artifact_session_id_str: String = artifact_id_table.get("session_id")?;
        let sequence: u64 = artifact_id_table.get("sequence")?;

        let artifact_session_id = SessionId::from_str(&artifact_session_id_str)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid artifact session ID: {}", e)))?;

        let artifact_id = ArtifactId::new(content_hash, artifact_session_id, sequence);

        let bridge = delete_bridge.clone();
        block_on_async(
            "artifact_delete",
            async move { bridge.delete_artifact(&session_id, &artifact_id).await },
            None,
        )?;

        Ok(())
    })?;
    artifact_table.set("delete", delete_fn)?;

    // storeFile method - store a file as an artifact
    let store_file_bridge = artifact_bridge.clone();
    let store_file_fn =
        lua.create_function(move |lua, args: (String, String, String, Option<Table>)| {
            let (session_id_str, file_path, type_str, metadata) = args;

            // Parse session ID
            let session_id = SessionId::from_str(&session_id_str)
                .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {}", e)))?;

            // Parse artifact type
            let artifact_type =
                llmspell_sessions::bridge::conversions::parse_artifact_type(&type_str)
                    .map_err(mlua::Error::RuntimeError)?;

            // Convert metadata table to HashMap
            let metadata_map = if let Some(table) = metadata {
                let mut map = std::collections::HashMap::new();
                for pair in table.pairs::<String, Value>() {
                    let (k, v) = pair?;
                    let json_value = lua_value_to_json(v)?;
                    map.insert(k, json_value);
                }
                Some(map)
            } else {
                None
            };

            let bridge = store_file_bridge.clone();
            let result = block_on_async(
                "artifact_store_file",
                async move {
                    bridge
                        .store_file_artifact(
                            &session_id,
                            std::path::Path::new(&file_path),
                            artifact_type,
                            metadata_map,
                        )
                        .await
                },
                None,
            )?;

            // Return artifact ID as table
            let id_table = lua.create_table()?;
            id_table.set("content_hash", result.content_hash)?;
            id_table.set("session_id", result.session_id.to_string())?;
            id_table.set("sequence", result.sequence)?;
            Ok(id_table)
        })?;
    artifact_table.set("storeFile", store_file_fn)?;

    // Set the Artifact table as a global
    lua.globals().set("Artifact", artifact_table)?;

    Ok(())
}
