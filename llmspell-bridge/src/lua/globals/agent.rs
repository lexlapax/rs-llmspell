//! ABOUTME: Lua-specific Agent global implementation
//! ABOUTME: Provides Lua bindings for Agent functionality

use crate::agent_bridge::AgentBridge;
use crate::globals::GlobalContext;
use crate::lua::conversion::{agent_output_to_lua_table, lua_table_to_agent_input};
use mlua::{Lua, Table, UserData, UserDataMethods};
use std::sync::Arc;

/// Lua userdata representing an agent instance
struct LuaAgentInstance {
    agent_instance_name: String,
    bridge: Arc<AgentBridge>,
}

impl UserData for LuaAgentInstance {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // invoke method
        methods.add_async_method("invoke", |lua, this, input: Table| async move {
            let agent_input = lua_table_to_agent_input(lua, input)?;

            let result = this
                .bridge
                .execute_agent(&this.agent_instance_name, agent_input, None)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

            agent_output_to_lua_table(lua, result)
        });

        // invokeStream method
        methods.add_async_method(
            "invokeStream",
            |lua, this, (input, callback): (Table, mlua::Function)| async move {
                let agent_input = lua_table_to_agent_input(lua, input)?;

                // Get streaming receiver
                let mut rx = this
                    .bridge
                    .execute_agent_streaming(&this.agent_instance_name, agent_input, None)
                    .await
                    .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

                // Process stream
                while let Some(output) = rx.recv().await {
                    let output_table = agent_output_to_lua_table(lua, output)?;
                    callback.call::<_, ()>(output_table)?;
                }

                Ok(())
            },
        );

        // getState method
        methods.add_async_method("getState", |_, this, ()| async move {
            let state = this
                .bridge
                .get_agent_state(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(format!("{:?}", state))
        });

        // destroy method
        methods.add_async_method("destroy", |_, this, ()| async move {
            this.bridge
                .remove_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });
    }
}

/// Inject Agent global into Lua environment
pub fn inject_agent_global(
    lua: &Lua,
    context: &GlobalContext,
    bridge: Arc<AgentBridge>,
) -> mlua::Result<()> {
    let agent_table = lua.create_table()?;

    // Store bridge reference in context for cross-global access
    context.set_bridge("agent", bridge.clone());

    // Create Agent.create() function
    let bridge_clone = bridge.clone();
    let create_fn = lua.create_async_function(move |_lua, args: Table| {
        let bridge = bridge_clone.clone();

        async move {
            // Extract configuration from Lua table
            let system_prompt: Option<String> = args.get("system_prompt").ok();
            let temperature: Option<f32> = args.get("temperature").ok();
            let max_tokens: Option<usize> = args.get("max_tokens").ok();
            let max_conversation_length: Option<usize> = args.get("max_conversation_length").ok();
            let base_url: Option<String> = args.get("base_url").ok();
            let api_key: Option<String> = args.get("api_key").ok();

            // Get model specification
            let model_str = args
                .get::<_, Option<String>>("model")
                .ok()
                .flatten()
                .ok_or_else(|| {
                    mlua::Error::RuntimeError("Model specification required".to_string())
                })?;

            // Create unique instance name
            let instance_name = format!(
                "agent_{}",
                uuid::Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            );

            // Create agent configuration
            let mut config = std::collections::HashMap::new();
            config.insert("model".to_string(), serde_json::json!(model_str));
            config.insert("base_url".to_string(), serde_json::json!(base_url));
            config.insert("api_key".to_string(), serde_json::json!(api_key));
            config.insert(
                "system_prompt".to_string(),
                serde_json::json!(system_prompt),
            );
            config.insert("temperature".to_string(), serde_json::json!(temperature));
            config.insert("max_tokens".to_string(), serde_json::json!(max_tokens));
            config.insert(
                "max_conversation_length".to_string(),
                serde_json::json!(max_conversation_length),
            );

            // Create the agent with bridge
            bridge
                .create_agent(&instance_name, "llm", config)
                .await
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create agent: {}", e)))?;

            // Create Lua agent instance
            let agent_instance = LuaAgentInstance {
                agent_instance_name: instance_name,
                bridge: bridge.clone(),
            };

            Ok(agent_instance)
        }
    })?;

    // Create Agent.list() function
    let bridge_clone = bridge.clone();
    let list_fn = lua.create_function(move |lua, ()| {
        let agents = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge_clone.list_instances())
        });
        let list_table = lua.create_table()?;
        for (i, name) in agents.into_iter().enumerate() {
            let agent_table = lua.create_table()?;
            agent_table.set("name", name)?;
            list_table.set(i + 1, agent_table)?;
        }
        Ok(list_table)
    })?;

    // Create Agent.discover() function
    let bridge_clone = bridge.clone();
    let discover_fn = lua.create_function(move |lua, ()| {
        let agent_types = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge_clone.list_agent_types())
        });
        let discover_table = lua.create_table()?;
        for (i, agent_type) in agent_types.into_iter().enumerate() {
            let agent_table = lua.create_table()?;
            agent_table.set("type", agent_type)?;
            discover_table.set(i + 1, agent_table)?;
        }
        Ok(discover_table)
    })?;

    // Set functions on Agent table
    agent_table.set("create", create_fn)?;
    agent_table.set("list", list_fn)?;
    agent_table.set("discover", discover_fn)?;

    // Set Agent as global
    lua.globals().set("Agent", agent_table)?;

    Ok(())
}
