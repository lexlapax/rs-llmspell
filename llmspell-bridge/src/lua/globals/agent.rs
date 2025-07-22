//! ABOUTME: Lua-specific Agent global implementation
//! ABOUTME: Provides Lua bindings for Agent functionality

use crate::agent_bridge::AgentBridge;
use crate::globals::GlobalContext;
use crate::lua::conversion::{
    agent_output_to_lua_table, json_to_lua_value, lua_table_to_agent_input, lua_table_to_json,
};
use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use std::collections::HashMap;
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
            let name: String = args.get("name").unwrap_or_else(|_| {
                format!(
                    "agent_{}",
                    uuid::Uuid::new_v4()
                        .to_string()
                        .chars()
                        .take(8)
                        .collect::<String>()
                )
            });
            let description: String = args
                .get("description")
                .unwrap_or_else(|_| "LLM-powered agent".to_string());
            let system_prompt: Option<String> = args.get("system_prompt").ok();
            let temperature: Option<f32> = args.get("temperature").ok();
            let max_tokens: Option<u32> = args.get("max_tokens").ok().map(|v: usize| v as u32);
            let max_conversation_length: Option<usize> = args.get("max_conversation_length").ok();
            let base_url: Option<String> = args.get("base_url").ok();
            let api_key: Option<String> = args.get("api_key").ok();

            // Get model specification - support both "model" and "provider_model" fields
            let model_str = args
                .get::<_, Option<String>>("model")
                .ok()
                .flatten()
                .or_else(|| {
                    args.get::<_, Option<String>>("provider_model")
                        .ok()
                        .flatten()
                })
                .ok_or_else(|| {
                    mlua::Error::RuntimeError(
                        "Model specification required (use 'model' field)".to_string(),
                    )
                })?;

            // Parse provider/model syntax (e.g., "openai/gpt-4")
            let (provider, model_id) = if model_str.contains('/') {
                let parts: Vec<&str> = model_str.splitn(2, '/').collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                // Default to openai if no provider specified
                ("openai".to_string(), model_str)
            };

            // Create model configuration
            let model_config = serde_json::json!({
                "provider": provider,
                "model_id": model_id,
                "temperature": temperature,
                "max_tokens": max_tokens,
                "settings": {
                    "base_url": base_url,
                    "api_key": api_key
                }
            });

            // Create custom config for agent
            let mut custom_config = serde_json::Map::new();
            if let Some(prompt) = system_prompt {
                custom_config.insert("system_prompt".to_string(), serde_json::json!(prompt));
            }
            if let Some(len) = max_conversation_length {
                custom_config.insert(
                    "max_conversation_length".to_string(),
                    serde_json::json!(len),
                );
            }

            // Create full agent configuration
            let agent_config = serde_json::json!({
                "name": name.clone(),
                "description": description,
                "agent_type": "llm",  // Default to LLM agent type
                "model": model_config,
                "allowed_tools": [],  // Can be extended later
                "custom_config": custom_config,
                "resource_limits": {
                    "max_execution_time_secs": 300,
                    "max_memory_mb": 512,
                    "max_tool_calls": 100,
                    "max_recursion_depth": 10
                }
            });

            // Convert JSON value to HashMap for bridge
            let config_map: HashMap<String, serde_json::Value> = match agent_config {
                serde_json::Value::Object(map) => map.into_iter().collect(),
                _ => {
                    return Err(mlua::Error::RuntimeError(
                        "Invalid agent configuration format".to_string(),
                    ))
                }
            };

            // Create the agent through discovery
            bridge
                .create_agent(&name, "llm", config_map)
                .await
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create agent: {}", e)))?;

            // Create Lua agent instance
            let agent_instance = LuaAgentInstance {
                agent_instance_name: name,
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

    // Create Agent.wrapAsTool() function
    let bridge_clone = bridge.clone();
    let wrap_as_tool_fn = lua.create_function(move |_lua, args: (String, Table)| {
        let (agent_name, config) = args;
        let bridge = bridge_clone.clone();

        // Convert Lua table to JSON
        let config_value = lua_table_to_json(config)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert config: {}", e)))?;

        // Use sync wrapper to call async method
        let tool_name = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(bridge.wrap_agent_as_tool(&agent_name, config_value))
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to wrap agent as tool: {}", e)))?;

        Ok(tool_name)
    })?;

    // Create Agent.getInfo() function
    let bridge_clone = bridge.clone();
    let get_info_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let agent_info = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge.get_agent_info(&agent_name))
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get agent info: {}", e)))?;

        // Convert AgentInfo to JSON, then to Lua table
        let info_json = serde_json::to_value(&agent_info).map_err(|e| {
            mlua::Error::RuntimeError(format!("Failed to serialize agent info: {}", e))
        })?;
        let info_table = json_to_lua_value(lua, &info_json)?;
        match info_table {
            Value::Table(table) => Ok(table),
            _ => Err(mlua::Error::RuntimeError(
                "Invalid agent info format".to_string(),
            )),
        }
    })?;

    // Create Agent.listCapabilities() function
    let bridge_clone = bridge.clone();
    let list_capabilities_fn = lua.create_function(move |lua, ()| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let capabilities_json = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge.list_agent_capabilities())
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to list capabilities: {}", e)))?;

        // Convert JSON to Lua table
        let capabilities_table = json_to_lua_value(lua, &capabilities_json)?;
        match capabilities_table {
            Value::Table(table) => Ok(table),
            _ => Err(mlua::Error::RuntimeError(
                "Invalid capabilities format".to_string(),
            )),
        }
    })?;

    // Create Agent.createComposite() function
    let bridge_clone = bridge.clone();
    let create_composite_fn = lua.create_function(move |_lua, args: (String, Table, Table)| {
        let (name, agents_table, config) = args;
        let bridge = bridge_clone.clone();

        // Convert agents table to Vec<String>
        let mut agents = Vec::new();
        for pair in agents_table.pairs::<mlua::Integer, String>() {
            let (_, agent_name) = pair?;
            agents.push(agent_name);
        }

        // Convert config to JSON
        let config_json = lua_table_to_json(config)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert config: {}", e)))?;

        // Use sync wrapper to call async method
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge.create_composite_agent(
                name,
                agents,
                config_json,
            ))
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create composite: {}", e)))?;

        Ok(())
    })?;

    // Create Agent.discoverByCapability() function
    let bridge_clone = bridge.clone();
    let discover_by_capability_fn = lua.create_function(move |lua, capability: String| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let agents = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(bridge.discover_agents_by_capability(&capability))
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to discover agents: {}", e)))?;

        // Convert to Lua table
        let agents_table = lua.create_table()?;
        for (i, agent_name) in agents.into_iter().enumerate() {
            agents_table.set(i + 1, agent_name)?;
        }
        Ok(agents_table)
    })?;

    // Create Agent.register() function - alias for create
    let bridge_clone = bridge.clone();
    let register_fn = lua.create_function(move |_lua, args: Table| {
        let bridge = bridge_clone.clone();

        // Extract configuration from Lua table
        let name: String = args.get("name").unwrap_or_else(|_| {
            format!(
                "agent_{}",
                uuid::Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            )
        });

        // Get agent type - default to "llm"
        let agent_type: String = args.get("agent_type").unwrap_or_else(|_| "llm".to_string());

        // Convert entire args table to JSON for config
        let config_json = lua_table_to_json(args)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert config: {}", e)))?;

        // Convert JSON to HashMap for bridge
        let config_map: HashMap<String, serde_json::Value> = match config_json {
            serde_json::Value::Object(map) => map.into_iter().collect(),
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "Invalid agent configuration format".to_string(),
                ))
            }
        };

        // Use sync wrapper to call async method
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge.create_agent(
                &name,
                &agent_type,
                config_map,
            ))
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to register agent: {}", e)))?;

        // Return the agent name
        Ok(name)
    })?;

    // Create Agent.get() function
    let bridge_clone = bridge.clone();
    let get_fn = lua.create_function(move |_lua, agent_name: String| {
        let bridge = bridge_clone.clone();
        let name = agent_name.clone();

        // Use sync wrapper to call async method
        let agent_exists = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge.get_agent(&name))
        })
        .is_some();

        if agent_exists {
            // Create Lua agent instance
            let agent_instance = LuaAgentInstance {
                agent_instance_name: agent_name,
                bridge: bridge.clone(),
            };
            Ok(Some(agent_instance))
        } else {
            Ok(None)
        }
    })?;

    // Set functions on Agent table
    agent_table.set("create", create_fn)?;
    agent_table.set("list", list_fn)?;
    agent_table.set("discover", discover_fn)?;
    agent_table.set("wrapAsTool", wrap_as_tool_fn)?;
    agent_table.set("getInfo", get_info_fn)?;
    agent_table.set("listCapabilities", list_capabilities_fn)?;
    agent_table.set("createComposite", create_composite_fn)?;
    agent_table.set("discoverByCapability", discover_by_capability_fn)?;
    agent_table.set("register", register_fn)?;
    agent_table.set("get", get_fn)?;

    // Add coroutine wrapper helper for async Agent.create
    let create_async_code = r#"
        -- Helper to create agents within a coroutine context
        function(config)
            -- Create coroutine for async execution
            local co = coroutine.create(function()
                return Agent.create(config)
            end)
            
            -- Execute the coroutine
            local success, result = coroutine.resume(co)
            
            -- Handle async operations that yield
            while success and coroutine.status(co) ~= "dead" do
                success, result = coroutine.resume(co, result)
            end
            
            if not success then
                error(tostring(result))
            end
            
            return result
        end
    "#;

    let create_async_fn = lua
        .load(create_async_code)
        .eval::<mlua::Function>()
        .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

    agent_table.set("createAsync", create_async_fn)?;

    // Set Agent as global
    lua.globals().set("Agent", agent_table)?;

    Ok(())
}
