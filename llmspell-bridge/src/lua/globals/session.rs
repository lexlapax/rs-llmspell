//! ABOUTME: Lua-specific Session global implementation
//! ABOUTME: Provides Lua bindings for session management functionality

use crate::globals::GlobalContext;
use crate::lua::conversion::json_to_lua_value;
use crate::lua::sync_utils::block_on_async;
use crate::session_bridge::SessionBridge;
use llmspell_sessions::{
    types::{CreateSessionOptions, SessionQuery},
    SessionId,
};
use mlua::{Error as LuaError, Lua, Table, UserData, UserDataMethods};
use std::str::FromStr;
use std::sync::Arc;

/// SessionBuilder for creating sessions with method chaining
#[derive(Clone)]
struct SessionBuilder {
    bridge: Arc<SessionBridge>,
    name: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
}

impl SessionBuilder {
    fn new(bridge: Arc<SessionBridge>) -> Self {
        Self {
            bridge,
            name: None,
            description: None,
            tags: Vec::new(),
        }
    }
}

impl UserData for SessionBuilder {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Set name
        methods.add_method_mut("name", |_, this, name: String| {
            this.name = Some(name);
            Ok(this.clone())
        });

        // Set description
        methods.add_method_mut("description", |_, this, desc: String| {
            this.description = Some(desc);
            Ok(this.clone())
        });

        // Add single tag
        methods.add_method_mut("tag", |_, this, tag: String| {
            this.tags.push(tag);
            Ok(this.clone())
        });

        // Add multiple tags
        methods.add_method_mut("tags", |_, this, tags: Vec<String>| {
            this.tags.extend(tags);
            Ok(this.clone())
        });

        // Build method
        methods.add_method("build", |_lua, this, ()| {
            // Create session options
            let mut builder = CreateSessionOptions::builder().tags(this.tags.clone());

            if let Some(n) = &this.name {
                builder = builder.name(n.clone());
            }
            if let Some(d) = &this.description {
                builder = builder.description(d.clone());
            }

            let options = builder.build();

            // Create session using bridge
            let bridge = this.bridge.clone();
            let result = block_on_async(
                "session_builder_create",
                async move { bridge.create_session(options).await },
                None,
            )?;

            // Convert SessionId to string for Lua
            Ok(result.to_string())
        });
    }
}

/// Inject Session global into Lua environment
pub fn inject_session_global(
    lua: &Lua,
    _context: &GlobalContext,
    session_bridge: Arc<SessionBridge>,
) -> mlua::Result<()> {
    // Create Session table
    let session_table = lua.create_table()?;

    // Create method - create a new session
    let create_bridge = session_bridge.clone();
    let create_fn = lua.create_function(move |_lua, options: Option<Table>| {
        let create_options = if let Some(opts) = options {
            // Convert Lua table to CreateSessionOptions
            let name = opts.get::<_, Option<String>>("name")?;
            let description = opts.get::<_, Option<String>>("description")?;
            let tags = opts
                .get::<_, Option<Vec<String>>>("tags")?
                .unwrap_or_default();

            let mut builder = CreateSessionOptions::builder().tags(tags);

            if let Some(n) = name {
                builder = builder.name(n);
            }
            if let Some(d) = description {
                builder = builder.description(d);
            }

            builder.build()
        } else {
            CreateSessionOptions::default()
        };

        let bridge = create_bridge.clone();
        let result = block_on_async(
            "session_create",
            async move { bridge.create_session(create_options).await },
            None,
        )?;

        // Convert SessionId to string for Lua
        Ok(result.to_string())
    })?;
    session_table.set("create", create_fn)?;

    // Get method - get session metadata
    let get_bridge = session_bridge.clone();
    let get_fn = lua.create_function(move |lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = get_bridge.clone();
        let result = block_on_async(
            "session_get_metadata",
            async move { bridge.get_session_metadata(&session_id).await },
            None,
        )?;

        // Convert JSON to Lua value
        json_to_lua_value(lua, &result)
    })?;
    session_table.set("get", get_fn)?;

    // List method - list sessions with optional query
    let list_bridge = session_bridge.clone();
    let list_fn = lua.create_function(move |lua, query: Option<Table>| {
        let session_query = if let Some(q) = query {
            // Convert Lua table to SessionQuery
            let mut query = SessionQuery::default();

            if let Ok(Some(_status)) = q.get::<_, Option<String>>("status") {
                // Parse status - implementation would need the actual status parsing
            }

            if let Ok(Some(tags)) = q.get::<_, Option<Vec<String>>>("tags") {
                query.tags = tags;
            }

            if let Ok(Some(limit)) = q.get::<_, Option<usize>>("limit") {
                query.limit = Some(limit);
            }

            query
        } else {
            SessionQuery::default()
        };

        let bridge = list_bridge.clone();
        let result = block_on_async(
            "session_list",
            async move { bridge.list_sessions(session_query).await },
            None,
        )?;

        // Convert Vec<SessionMetadata> to Lua table
        let lua_table = lua.create_table()?;
        for (i, metadata) in result.iter().enumerate() {
            let json_value =
                llmspell_sessions::bridge::conversions::session_metadata_to_json(metadata);
            let lua_value = json_to_lua_value(lua, &json_value)?;
            lua_table.set(i + 1, lua_value)?;
        }
        Ok(lua_table)
    })?;
    session_table.set("list", list_fn)?;

    // Save method - save a session
    let save_bridge = session_bridge.clone();
    let save_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        // First get the session, then save it
        let bridge = save_bridge.clone();
        block_on_async(
            "session_save",
            async move {
                let session = bridge.get_session(&session_id).await?;
                bridge.save_session(&session).await
            },
            None,
        )?;

        Ok(())
    })?;
    session_table.set("save", save_fn)?;

    // Load method - load a session
    let load_bridge = session_bridge.clone();
    let load_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = load_bridge.clone();
        block_on_async(
            "session_load",
            async move { bridge.load_session(&session_id).await },
            None,
        )?;

        Ok(session_id.to_string())
    })?;
    session_table.set("load", load_fn)?;

    // Complete method - complete a session
    let complete_bridge = session_bridge.clone();
    let complete_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = complete_bridge.clone();
        block_on_async(
            "session_complete",
            async move { bridge.complete_session(&session_id).await },
            None,
        )?;

        Ok(())
    })?;
    session_table.set("complete", complete_fn)?;

    // Suspend method - suspend a session
    let suspend_bridge = session_bridge.clone();
    let suspend_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = suspend_bridge.clone();
        block_on_async(
            "session_suspend",
            async move { bridge.suspend_session(&session_id).await },
            None,
        )?;

        Ok(())
    })?;
    session_table.set("suspend", suspend_fn)?;

    // Resume method - resume a session
    let resume_bridge = session_bridge.clone();
    let resume_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = resume_bridge.clone();
        block_on_async(
            "session_resume",
            async move { bridge.resume_session(&session_id).await },
            None,
        )?;

        Ok(())
    })?;
    session_table.set("resume", resume_fn)?;

    // Delete method - delete a session
    let delete_bridge = session_bridge.clone();
    let delete_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = delete_bridge.clone();
        block_on_async(
            "session_delete",
            async move { bridge.delete_session(&session_id).await },
            None,
        )?;

        Ok(())
    })?;
    session_table.set("delete", delete_fn)?;

    // get_current method - get current session from thread-local context
    let get_current_fn = lua.create_function(|_lua, ()| {
        SessionBridge::get_current_session()
            .map_or_else(|| Ok(None), |session_id| Ok(Some(session_id.to_string())))
    })?;
    session_table.set("get_current", get_current_fn)?;

    // set_current method - set current session in thread-local context
    let set_current_fn = lua.create_function(|_lua, session_id: Option<String>| {
        let session_id = match session_id {
            Some(id) => Some(
                SessionId::from_str(&id)
                    .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?,
            ),
            None => None,
        };
        SessionBridge::set_current_session(session_id);
        Ok(())
    })?;
    session_table.set("set_current", set_current_fn)?;

    // Replay methods - session replay functionality

    // can_replay method - check if a session can be replayed
    let can_replay_bridge = session_bridge.clone();
    let can_replay_fn = lua.create_function(move |_lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = can_replay_bridge.clone();
        block_on_async(
            "session_can_replay",
            async move { bridge.can_replay_session(&session_id).await },
            None,
        )
    })?;
    session_table.set("can_replay", can_replay_fn)?;

    // replay method - replay a session
    let replay_bridge = session_bridge.clone();
    let replay_fn = lua.create_function(move |lua, args: (String, Option<Table>)| {
        let (session_id_str, config_table) = args;
        let session_id = SessionId::from_str(&session_id_str)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        // Convert config table to JSON (SessionBridge handles the conversion)
        let config_json = if let Some(config) = config_table {
            serde_json::json!({
                "start_from": config.get::<_, Option<String>>("start_from")?,
                "end_at": config.get::<_, Option<String>>("end_at")?,
                "hook_filter": config.get::<_, Option<String>>("hook_filter")?,
                "max_duration_seconds": config.get::<_, Option<u64>>("max_duration_seconds")?,
                "include_failed": config.get::<_, Option<bool>>("include_failed")?.unwrap_or(false),
                "progress_callback": config.get::<_, Option<bool>>("progress_callback")?.unwrap_or(false)
            })
        } else {
            serde_json::json!({})
        };

        let bridge = replay_bridge.clone();
        let result = block_on_async(
            "session_replay",
            async move { bridge.replay_session(&session_id, config_json).await },
            None,
        )?;

        // Convert JSON result back to Lua
        json_to_lua_value(lua, &result)
    })?;
    session_table.set("replay", replay_fn)?;

    // get_replay_metadata method - get replay metadata for a session
    let metadata_bridge = session_bridge.clone();
    let metadata_fn = lua.create_function(move |lua, session_id: String| {
        let session_id = SessionId::from_str(&session_id)
            .map_err(|e| LuaError::RuntimeError(format!("Invalid session ID: {e}")))?;

        let bridge = metadata_bridge.clone();
        let result = block_on_async(
            "session_replay_metadata",
            async move { bridge.get_session_replay_metadata(&session_id).await },
            None,
        )?;

        // Convert JSON to Lua value
        json_to_lua_value(lua, &result)
    })?;
    session_table.set("get_replay_metadata", metadata_fn)?;

    // list_replayable method - list all sessions that can be replayed
    let list_replayable_bridge = session_bridge.clone();
    let list_replayable_fn = lua.create_function(move |lua, ()| {
        let bridge = list_replayable_bridge.clone();
        let result = block_on_async(
            "session_list_replayable",
            async move { bridge.list_replayable_sessions().await },
            None,
        )?;

        // Convert Vec<SessionId> to Lua table
        let lua_table = lua.create_table()?;
        for (i, session_id) in result.iter().enumerate() {
            lua_table.set(i + 1, session_id.to_string())?;
        }
        Ok(lua_table)
    })?;
    session_table.set("list_replayable", list_replayable_fn)?;

    // Set the Session table as a global
    // Add Session.builder() method
    let bridge_for_builder = session_bridge.clone();
    let builder_fn =
        lua.create_function(move |_lua, ()| Ok(SessionBuilder::new(bridge_for_builder.clone())))?;
    session_table.set("builder", builder_fn)?;

    // Note: Session.create() remains available but builder pattern is preferred

    lua.globals().set("Session", session_table)?;

    Ok(())
}
