//! ABOUTME: Lua-specific Agent global implementation
//! ABOUTME: Provides Lua bindings for Agent functionality

use crate::agent_bridge::AgentBridge;
use crate::globals::GlobalContext;
use crate::lua::conversion::{
    agent_output_to_lua_table, json_to_lua_value, lua_table_to_agent_input, lua_table_to_json,
};
use crate::lua::sync_utils::block_on_async;
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
        // invoke method (same as execute in API) - synchronous wrapper
        methods.add_method("invoke", |lua, this, input: Table| {
            let agent_input = lua_table_to_agent_input(lua, input)?;
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            // Use shared sync utility to execute async code
            let result = block_on_async(
                "agent_invoke",
                async move { bridge.execute_agent(&agent_name, agent_input, None).await },
                None,
            )?;

            agent_output_to_lua_table(lua, result)
        });

        // execute method (alias for invoke) - synchronous wrapper
        methods.add_method("execute", |lua, this, input: Table| {
            let agent_input = lua_table_to_agent_input(lua, input)?;
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            // Use shared sync utility to execute async code
            let result = block_on_async(
                "agent_execute",
                async move { bridge.execute_agent(&agent_name, agent_input, None).await },
                None,
            )?;

            agent_output_to_lua_table(lua, result)
        });

        // invokeStream method - synchronous wrapper
        methods.add_method(
            "invokeStream",
            |lua, this, (input, callback): (Table, mlua::Function)| {
                let agent_input = lua_table_to_agent_input(lua, input)?;
                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();

                // Use block_on to handle the streaming operation
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        // Get streaming receiver
                        let mut rx = bridge
                            .execute_agent_streaming(&agent_name, agent_input, None)
                            .await
                            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

                        // Process stream
                        let mut chunk_count = 0;
                        while let Some(output) = rx.recv().await {
                            let output_table = agent_output_to_lua_table(lua, output)?;
                            callback.call::<_, ()>(output_table)?;
                            chunk_count += 1;
                        }

                        // Return a table with streaming results
                        let result_table = lua.create_table()?;
                        result_table.set("success", true)?;
                        result_table.set("chunks_received", chunk_count)?;
                        Ok(result_table)
                    })
                })
            },
        );

        // getState method - synchronous wrapper
        methods.add_method("getState", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let state = block_on_async(
                "agent_getState",
                async move { bridge.get_agent_state(&agent_name).await },
                None,
            )?;
            Ok(format!("{:?}", state))
        });

        // getConfig method
        methods.add_method("getConfig", |lua, _this, ()| {
            // TODO: Get agent configuration from bridge when API is available
            // For now, return empty config table
            let config_table = lua.create_table()?;
            Ok(config_table)
        });

        // setState method
        methods.add_method("setState", |_lua, _this, _state: Table| {
            // TODO: Implement state setting when bridge supports it
            Ok(())
        });

        // Tool Integration Methods

        // discoverTools method
        methods.add_method("discoverTools", |lua, this, ()| {
            let tools = this.bridge.list_tools();
            let tools_table = lua.create_table()?;
            for (i, tool_name) in tools.iter().enumerate() {
                tools_table.set(i + 1, tool_name.clone())?;
            }
            Ok(tools_table)
        });

        // getToolMetadata method
        methods.add_method("getToolMetadata", |lua, this, tool_name: String| {
            if let Some(metadata) = this.bridge.get_tool_metadata(&tool_name) {
                // Convert JSON to Lua table
                let metadata_table = lua.create_table()?;
                if let Some(name) = metadata.get("name").and_then(|v| v.as_str()) {
                    metadata_table.set("name", name)?;
                }
                if let Some(desc) = metadata.get("description").and_then(|v| v.as_str()) {
                    metadata_table.set("description", desc)?;
                }
                if let Some(version) = metadata.get("version").and_then(|v| v.as_str()) {
                    metadata_table.set("version", version)?;
                }
                Ok(Some(metadata_table))
            } else {
                Ok(None)
            }
        });

        // invokeTool method - synchronous wrapper
        methods.add_method(
            "invokeTool",
            |lua, this, (tool_name, input_table): (String, Table)| {
                // Convert Lua table to tool input
                let tool_input_json = crate::lua::conversion::lua_table_to_json(input_table)?;

                // Wrap the parameters in a "parameters" key as expected by extract_parameters
                let mut wrapped_params = std::collections::HashMap::new();
                wrapped_params.insert("parameters".to_string(), tool_input_json);

                let agent_input = llmspell_core::types::AgentInput {
                    text: format!("Invoking tool: {}", tool_name),
                    media: vec![],
                    context: None,
                    parameters: wrapped_params,
                    output_modalities: vec![],
                };

                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();

                // Invoke the tool through the bridge
                let result = block_on_async(
                    "agent_invokeTool",
                    async move {
                        bridge
                            .invoke_tool_for_agent(&agent_name, &tool_name, agent_input, None)
                            .await
                    },
                    None,
                )?;

                // Convert AgentOutput to Lua table
                agent_output_to_lua_table(lua, result)
            },
        );

        // hasTool method
        methods.add_method("hasTool", |_lua, this, tool_name: String| {
            Ok(this.bridge.has_tool(&tool_name))
        });

        // getAllToolMetadata method
        methods.add_method("getAllToolMetadata", |lua, this, ()| {
            let all_metadata = this.bridge.get_all_tool_metadata();
            let metadata_table = lua.create_table()?;

            for (tool_name, metadata) in all_metadata {
                let tool_metadata_table = lua.create_table()?;
                if let Some(name) = metadata.get("name").and_then(|v| v.as_str()) {
                    tool_metadata_table.set("name", name)?;
                }
                if let Some(desc) = metadata.get("description").and_then(|v| v.as_str()) {
                    tool_metadata_table.set("description", desc)?;
                }
                if let Some(version) = metadata.get("version").and_then(|v| v.as_str()) {
                    tool_metadata_table.set("version", version)?;
                }
                metadata_table.set(tool_name, tool_metadata_table)?;
            }

            Ok(metadata_table)
        });

        // Monitoring & Lifecycle Methods

        // getMetrics method - synchronous wrapper
        methods.add_method("getMetrics", |lua, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let metrics_result = block_on_async(
                "agent_getMetrics",
                async move { bridge.get_agent_metrics(&agent_name).await },
                None,
            );

            match metrics_result {
                Ok(metrics) => {
                    let metrics_table = lua.create_table()?;
                    metrics_table.set("agent_id", metrics.agent_id.clone())?;
                    metrics_table.set("requests_total", metrics.requests_total.get() as f64)?;
                    metrics_table.set("requests_failed", metrics.requests_failed.get() as f64)?;
                    metrics_table.set("requests_active", metrics.requests_active.get())?;
                    metrics_table.set("tool_invocations", metrics.tool_invocations.get() as f64)?;
                    metrics_table.set("memory_bytes", metrics.memory_bytes.get())?;
                    metrics_table.set("cpu_percent", metrics.cpu_percent.get())?;
                    Ok(Some(metrics_table))
                }
                Err(_) => Ok(None),
            }
        });

        // getHealth method - synchronous wrapper
        methods.add_method("getHealth", |lua, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let health_result = block_on_async(
                "agent_getHealth",
                async move { bridge.get_agent_health(&agent_name).await },
                None,
            );

            match health_result {
                Ok(health_json) => {
                    // Convert JSON to Lua table
                    let health_table = lua.create_table()?;
                    if let Some(status) = health_json.get("status").and_then(|v| v.as_str()) {
                        health_table.set("status", status)?;
                    }
                    if let Some(message) = health_json.get("message").and_then(|v| v.as_str()) {
                        health_table.set("message", message)?;
                    }
                    if let Some(timestamp) = health_json.get("timestamp").and_then(|v| v.as_str()) {
                        health_table.set("timestamp", timestamp)?;
                    }
                    Ok(Some(health_table))
                }
                Err(_) => Ok(None),
            }
        });

        // getPerformance method - synchronous wrapper
        methods.add_method("getPerformance", |lua, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let perf_result = block_on_async(
                "agent_getPerformance",
                async move { bridge.get_agent_performance(&agent_name).await },
                None,
            );

            match perf_result {
                Ok(perf_json) => {
                    let perf_table = lua.create_table()?;
                    if let Some(total_executions) =
                        perf_json.get("total_executions").and_then(|v| v.as_u64())
                    {
                        perf_table.set("total_executions", total_executions as f64)?;
                    }
                    if let Some(avg_time) = perf_json
                        .get("avg_execution_time_ms")
                        .and_then(|v| v.as_f64())
                    {
                        perf_table.set("avg_execution_time_ms", avg_time)?;
                    }
                    if let Some(success_rate) =
                        perf_json.get("success_rate").and_then(|v| v.as_f64())
                    {
                        perf_table.set("success_rate", success_rate)?;
                    }
                    if let Some(error_rate) = perf_json.get("error_rate").and_then(|v| v.as_f64()) {
                        perf_table.set("error_rate", error_rate)?;
                    }
                    Ok(Some(perf_table))
                }
                Err(_) => Ok(None),
            }
        });

        // logEvent method - synchronous wrapper
        methods.add_method(
            "logEvent",
            |_, this, (event_type, message): (String, String)| {
                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();

                block_on_async(
                    "agent_logEvent",
                    async move {
                        bridge
                            .log_agent_event(&agent_name, &event_type, &message)
                            .await
                    },
                    None,
                )?;
                Ok(())
            },
        );

        // configureAlerts method - synchronous wrapper
        methods.add_method("configureAlerts", |_lua, this, config_table: Table| {
            // Convert Lua table to JSON for alert configuration
            let config_json = lua_table_to_json(config_table)?;
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_configureAlerts",
                async move {
                    bridge
                        .configure_agent_alerts(&agent_name, config_json)
                        .await
                },
                None,
            )?;
            Ok(())
        });

        // getAlerts method - synchronous wrapper
        methods.add_method("getAlerts", |lua, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let alerts_result = block_on_async(
                "agent_getAlerts",
                async move { bridge.get_agent_alerts(&agent_name).await },
                None,
            );

            match alerts_result {
                Ok(alerts) => {
                    let alerts_table = lua.create_table()?;
                    for (i, alert) in alerts.iter().enumerate() {
                        let alert_table = lua.create_table()?;
                        if let Some(severity) = alert.get("severity").and_then(|v| v.as_str()) {
                            alert_table.set("severity", severity)?;
                        }
                        if let Some(message) = alert.get("message").and_then(|v| v.as_str()) {
                            alert_table.set("message", message)?;
                        }
                        if let Some(timestamp) = alert.get("timestamp").and_then(|v| v.as_str()) {
                            alert_table.set("timestamp", timestamp)?;
                        }
                        alerts_table.set(i + 1, alert_table)?;
                    }
                    Ok(alerts_table)
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // getBridgeMetrics method (static bridge-wide metrics)
        methods.add_method("getBridgeMetrics", |lua, this, ()| {
            let bridge_metrics = this.bridge.get_bridge_metrics();
            let metrics_table = lua.create_table()?;

            for (key, value) in bridge_metrics {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            metrics_table.set(key, f)?;
                        }
                    }
                    serde_json::Value::String(s) => {
                        metrics_table.set(key, s)?;
                    }
                    serde_json::Value::Object(obj) => {
                        let obj_table = lua.create_table()?;
                        for (obj_key, obj_value) in obj {
                            if let Some(num) = obj_value.as_f64() {
                                obj_table.set(obj_key, num)?;
                            } else if let Some(str_val) = obj_value.as_str() {
                                obj_table.set(obj_key, str_val)?;
                            }
                        }
                        metrics_table.set(key, obj_table)?;
                    }
                    _ => {} // Skip other types
                }
            }

            Ok(metrics_table)
        });

        // State Machine Methods

        // getAgentState method - Get current agent state with full details - synchronous wrapper
        methods.add_method("getAgentState", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let state = block_on_async(
                "agent_getAgentState",
                async move { bridge.get_agent_state(&agent_name).await },
                None,
            )?;
            Ok(format!("{:?}", state))
        });

        // initialize method - synchronous wrapper
        methods.add_method("initialize", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_initialize",
                async move { bridge.initialize_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // start method - synchronous wrapper
        methods.add_method("start", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_start",
                async move { bridge.start_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // pause method - synchronous wrapper
        methods.add_method("pause", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_pause",
                async move { bridge.pause_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // resume method - synchronous wrapper
        methods.add_method("resume", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_resume",
                async move { bridge.resume_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // stop method - synchronous wrapper
        methods.add_method("stop", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_stop",
                async move { bridge.stop_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // terminate method - synchronous wrapper
        methods.add_method("terminate", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_terminate",
                async move { bridge.terminate_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // setError method - synchronous wrapper
        methods.add_method("setError", |_, this, error_message: String| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_setError",
                async move { bridge.error_agent(&agent_name, error_message).await },
                None,
            )?;
            Ok(())
        });

        // recover method - synchronous wrapper
        methods.add_method("recover", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_recover",
                async move { bridge.recover_agent(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // getStateHistory method - synchronous wrapper
        methods.add_method("getStateHistory", |lua, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let history_result = block_on_async(
                "agent_getStateHistory",
                async move { bridge.get_agent_state_history(&agent_name).await },
                None,
            );

            match history_result {
                Ok(history) => {
                    let history_table = lua.create_table()?;
                    for (i, transition) in history.iter().enumerate() {
                        let transition_table = lua.create_table()?;
                        if let Some(from) = transition.get("from").and_then(|v| v.as_str()) {
                            transition_table.set("from", from)?;
                        }
                        if let Some(to) = transition.get("to").and_then(|v| v.as_str()) {
                            transition_table.set("to", to)?;
                        }
                        if let Some(timestamp) =
                            transition.get("timestamp").and_then(|v| v.as_str())
                        {
                            transition_table.set("timestamp", timestamp)?;
                        }
                        if let Some(elapsed) = transition.get("elapsed").and_then(|v| v.as_f64()) {
                            transition_table.set("elapsed", elapsed)?;
                        }
                        if let Some(reason) = transition.get("reason").and_then(|v| v.as_str()) {
                            transition_table.set("reason", reason)?;
                        }
                        history_table.set(i + 1, transition_table)?;
                    }
                    Ok(history_table)
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // getLastError method - synchronous wrapper
        methods.add_method("getLastError", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let error_result = block_on_async(
                "agent_getLastError",
                async move { bridge.get_agent_last_error(&agent_name).await },
                None,
            );

            match error_result {
                Ok(error) => Ok(error),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // getRecoveryAttempts method - synchronous wrapper
        methods.add_method("getRecoveryAttempts", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let attempts_result = block_on_async(
                "agent_getRecoveryAttempts",
                async move { bridge.get_agent_recovery_attempts(&agent_name).await },
                None,
            );

            match attempts_result {
                Ok(attempts) => Ok(attempts),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // isHealthy method - synchronous wrapper
        methods.add_method("isHealthy", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let healthy_result = block_on_async(
                "agent_isHealthy",
                async move { bridge.is_agent_healthy(&agent_name).await },
                None,
            );

            match healthy_result {
                Ok(healthy) => Ok(healthy),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // getStateMetrics method - synchronous wrapper
        methods.add_method("getStateMetrics", |lua, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let metrics_result = block_on_async(
                "agent_getStateMetrics",
                async move { bridge.get_agent_state_metrics(&agent_name).await },
                None,
            );

            match metrics_result {
                Ok(metrics_json) => {
                    let metrics_table = lua.create_table()?;
                    if let Some(state) = metrics_json.get("current_state").and_then(|v| v.as_str())
                    {
                        metrics_table.set("current_state", state)?;
                    }
                    if let Some(transitions) = metrics_json
                        .get("total_transitions")
                        .and_then(|v| v.as_u64())
                    {
                        metrics_table.set("total_transitions", transitions as f64)?;
                    }
                    if let Some(errors) = metrics_json.get("error_count").and_then(|v| v.as_u64()) {
                        metrics_table.set("error_count", errors as f64)?;
                    }
                    if let Some(attempts) = metrics_json
                        .get("recovery_attempts")
                        .and_then(|v| v.as_u64())
                    {
                        metrics_table.set("recovery_attempts", attempts as f64)?;
                    }
                    if let Some(uptime) = metrics_json.get("uptime").and_then(|v| v.as_f64()) {
                        metrics_table.set("uptime", uptime)?;
                    }
                    if let Some(last_transition) =
                        metrics_json.get("last_transition").and_then(|v| v.as_str())
                    {
                        metrics_table.set("last_transition", last_transition)?;
                    }
                    if let Some(state_dist) = metrics_json
                        .get("state_time_distribution")
                        .and_then(|v| v.as_object())
                    {
                        let dist_table = lua.create_table()?;
                        for (state, time) in state_dist {
                            if let Some(time_val) = time.as_f64() {
                                dist_table.set(state.as_str(), time_val)?;
                            }
                        }
                        metrics_table.set("state_time_distribution", dist_table)?;
                    }
                    Ok(metrics_table)
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // Context & Communication Methods

        // executeWithContext method - synchronous wrapper
        methods.add_method(
            "executeWithContext",
            |lua, this, (input, context_id): (Table, String)| {
                let agent_input = lua_table_to_agent_input(lua, input)?;
                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();

                // Use shared sync utility to execute async code
                let result = block_on_async(
                    "agent_executeWithContext",
                    async move {
                        bridge
                            .execute_agent_with_context(&agent_name, agent_input, &context_id)
                            .await
                    },
                    None,
                )?;

                agent_output_to_lua_table(lua, result)
            },
        );

        // destroy method
        methods.add_method("destroy", |_, this, ()| {
            let agent_name = this.agent_instance_name.clone();
            let bridge = this.bridge.clone();

            block_on_async(
                "agent_destroy",
                async move { bridge.remove_agent(&agent_name).await },
                None,
            )
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

    // Create Agent.create() function (synchronous wrapper)
    let bridge_clone = bridge.clone();
    let create_fn = lua.create_function(move |_lua, args: Table| {
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

        // Use block_on to execute async code synchronously
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create the agent through discovery
                bridge
                    .create_agent(&name, "llm", config_map)
                    .await
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to create agent: {}", e))
                    })?;

                // Create Lua agent instance
                let agent_instance = LuaAgentInstance {
                    agent_instance_name: name,
                    bridge: bridge.clone(),
                };

                Ok(agent_instance)
            })
        })
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
        let tool_name = block_on_async(
            "agent_wrapAsTool",
            bridge.wrap_agent_as_tool(&agent_name, config_value),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to wrap agent as tool: {}", e)))?;

        Ok(tool_name)
    })?;

    // Create Agent.getInfo() function
    let bridge_clone = bridge.clone();
    let get_info_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let agent_info = block_on_async("agent_getInfo", bridge.get_agent_info(&agent_name), None)
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
        let capabilities_json = block_on_async(
            "agent_listCapabilities",
            bridge.list_agent_capabilities(),
            None,
        )
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
        block_on_async(
            "agent_createComposite",
            bridge.create_composite_agent(name, agents, config_json),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create composite: {}", e)))?;

        Ok(())
    })?;

    // Create Agent.discoverByCapability() function
    let bridge_clone = bridge.clone();
    let discover_by_capability_fn = lua.create_function(move |lua, capability: String| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let agents = block_on_async(
            "agent_discoverByCapability",
            bridge.discover_agents_by_capability(&capability),
            None,
        )
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
        let mut config_json = lua_table_to_json(args)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert config: {}", e)))?;

        // Fix empty objects that should be arrays
        if let serde_json::Value::Object(ref mut map) = config_json {
            // Fix allowed_tools if it's an empty object
            if let Some(serde_json::Value::Object(allowed_tools)) = map.get("allowed_tools") {
                if allowed_tools.is_empty() {
                    map.insert(
                        "allowed_tools".to_string(),
                        serde_json::Value::Array(vec![]),
                    );
                }
            }
        }

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
        block_on_async(
            "agent_register",
            bridge.create_agent(&name, &agent_type, config_map),
            None,
        )
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

    // Create Agent.listTemplates() function
    let bridge_clone = bridge.clone();
    let list_templates_fn = lua.create_function(move |lua, ()| {
        let templates = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge_clone.list_templates())
        });
        let list_table = lua.create_table()?;
        for (i, template) in templates.into_iter().enumerate() {
            list_table.set(i + 1, template)?;
        }
        Ok(list_table)
    })?;

    // Create Agent.createFromTemplate() function
    let bridge_clone = bridge.clone();
    let create_from_template_fn =
        lua.create_function(move |_lua, args: (String, String, Table)| {
            let (instance_name, template_name, params) = args;
            let bridge = bridge_clone.clone();

            // Convert Lua table to HashMap
            let mut parameters = HashMap::new();
            for pair in params.pairs::<String, Value>() {
                let (key, value) = pair?;
                if let Ok(json_value) = crate::lua::conversion::lua_value_to_json(value) {
                    parameters.insert(key, json_value);
                }
            }

            // Use sync wrapper to call async method
            block_on_async(
                "agent_createFromTemplate",
                bridge.create_from_template(&instance_name, &template_name, parameters),
                None,
            )
            .map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to create from template: {}", e))
            })?;

            // Return the created agent instance
            let agent_instance = LuaAgentInstance {
                agent_instance_name: instance_name.clone(),
                bridge: bridge.clone(),
            };
            Ok(agent_instance)
        })?;

    // Create Agent.listInstances() function (alias for list)
    let bridge_clone = bridge.clone();
    let list_instances_fn = lua.create_function(move |lua, ()| {
        let instances = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge_clone.list_instances())
        });
        let list_table = lua.create_table()?;
        for (i, name) in instances.into_iter().enumerate() {
            list_table.set(i + 1, name)?;
        }
        Ok(list_table)
    })?;

    // Context Management Functions

    // Create Agent.createContext() function
    let bridge_clone = bridge.clone();
    let create_context_fn = lua.create_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();
        let config_json = lua_table_to_json(config)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert config: {}", e)))?;

        let context_id = block_on_async(
            "agent_createContext",
            bridge.create_context(config_json),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create context: {}", e)))?;

        Ok(context_id)
    })?;

    // Create Agent.createChildContext() function
    let bridge_clone = bridge.clone();
    let create_child_context_fn =
        lua.create_function(move |_lua, args: (String, Table, String)| {
            let (parent_id, scope, inheritance) = args;
            let bridge = bridge_clone.clone();
            let scope_json = lua_table_to_json(scope).map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to convert scope: {}", e))
            })?;

            let child_id = block_on_async(
                "agent_createChildContext",
                bridge.create_child_context(&parent_id, scope_json, &inheritance),
                None,
            )
            .map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to create child context: {}", e))
            })?;

            Ok(child_id)
        })?;

    // Create Agent.updateContext() function
    let bridge_clone = bridge.clone();
    let update_context_fn = lua.create_function(move |_lua, args: (String, String, Value)| {
        let (context_id, key, value) = args;
        let bridge = bridge_clone.clone();
        let value_json = crate::lua::conversion::lua_value_to_json(value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert value: {}", e)))?;

        block_on_async(
            "agent_updateContext",
            bridge.update_context(&context_id, key, value_json),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to update context: {}", e)))?;

        Ok(())
    })?;

    // Create Agent.getContextData() function
    let bridge_clone = bridge.clone();
    let get_context_data_fn = lua.create_function(move |lua, args: (String, String)| {
        let (context_id, key) = args;
        let bridge = bridge_clone.clone();

        let result = block_on_async(
            "agent_getContextData",
            bridge.get_context_data(&context_id, &key),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get context data: {}", e)))?;

        match result {
            Some(value) => json_to_lua_value(lua, &value),
            None => Ok(Value::Nil),
        }
    })?;

    // Create Agent.removeContext() function
    let bridge_clone = bridge.clone();
    let remove_context_fn = lua.create_function(move |_lua, context_id: String| {
        let bridge = bridge_clone.clone();

        block_on_async(
            "agent_removeContext",
            bridge.remove_context(&context_id),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to remove context: {}", e)))?;

        Ok(())
    })?;

    // Shared Memory Functions

    // Create Agent.setSharedMemory() function
    let bridge_clone = bridge.clone();
    let set_shared_memory_fn = lua.create_function(move |_lua, args: (Table, String, Value)| {
        let (scope, key, value) = args;
        let bridge = bridge_clone.clone();
        let scope_json = lua_table_to_json(scope)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert scope: {}", e)))?;
        let value_json = crate::lua::conversion::lua_value_to_json(value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert value: {}", e)))?;

        block_on_async(
            "agent_setSharedMemory",
            bridge.set_shared_memory(scope_json, key, value_json),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to set shared memory: {}", e)))?;

        Ok(())
    })?;

    // Create Agent.getSharedMemory() function
    let bridge_clone = bridge.clone();
    let get_shared_memory_fn = lua.create_function(move |lua, args: (Table, String)| {
        let (scope, key) = args;
        let bridge = bridge_clone.clone();
        let scope_json = lua_table_to_json(scope)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert scope: {}", e)))?;

        let result = block_on_async(
            "agent_getSharedMemory",
            bridge.get_shared_memory(scope_json, &key),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get shared memory: {}", e)))?;

        match result {
            Some(value) => json_to_lua_value(lua, &value),
            None => Ok(Value::Nil),
        }
    })?;

    // Create Agent.getHierarchy() function
    let bridge_clone = bridge.clone();
    let get_hierarchy_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        let hierarchy = block_on_async(
            "agent_getHierarchy",
            bridge.get_composition_hierarchy(&agent_name),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get hierarchy: {}", e)))?;

        json_to_lua_value(lua, &hierarchy)
    })?;

    // Create Agent.getDetails() function (alias for getInfo with different return format)
    let bridge_clone = bridge.clone();
    let get_details_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        let details = block_on_async(
            "agent_getDetails",
            bridge.get_agent_details(&agent_name),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get agent details: {}", e)))?;

        json_to_lua_value(lua, &details)
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
    agent_table.set("listTemplates", list_templates_fn)?;
    agent_table.set("createFromTemplate", create_from_template_fn)?;
    agent_table.set("listInstances", list_instances_fn)?;
    agent_table.set("createContext", create_context_fn)?;
    agent_table.set("createChildContext", create_child_context_fn)?;
    agent_table.set("updateContext", update_context_fn)?;
    agent_table.set("getContextData", get_context_data_fn)?;
    agent_table.set("removeContext", remove_context_fn)?;
    agent_table.set("setSharedMemory", set_shared_memory_fn)?;
    agent_table.set("getSharedMemory", get_shared_memory_fn)?;
    agent_table.set("getHierarchy", get_hierarchy_fn)?;
    agent_table.set("getDetails", get_details_fn)?;

    // Set Agent as global
    lua.globals().set("Agent", agent_table)?;

    Ok(())
}
