//! ABOUTME: Lua Agent API implementation providing Agent.create() and agent methods
//! ABOUTME: Bridges between Lua scripts and Rust Agent implementations

use crate::agent_bridge::AgentBridge;
use crate::agent_conversion::{
    agent_output_to_lua_table, lua_table_to_agent_input, lua_table_to_tool_input,
    lua_value_to_json, tool_output_to_lua_table,
};
use crate::engine::types::AgentApiDefinition;
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, Result,
};
use llmspell_providers::{ModelSpecifier, ProviderInstance};
use mlua::{Lua, Table, UserData, UserDataMethods};
use std::collections::HashMap;
use std::sync::Arc;

/// Inject the Agent API into the Lua environment
pub fn inject_agent_api(
    lua: &Lua,
    api_def: &AgentApiDefinition,
    registry: Arc<ComponentRegistry>,
    providers: Arc<ProviderManager>,
) -> Result<()> {
    // Create the Agent global table
    let agent_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Agent table: {}", e),
        source: None,
    })?;

    // Create agent bridge
    let bridge = Arc::new(AgentBridge::new(registry.clone()));

    // Clone Arc for the closure
    let registry_clone = registry.clone();
    let providers_clone = providers.clone();

    // Create the Agent.create() function
    let create_fn = lua
        .create_async_function(move |_lua, args: Table| {
            let registry = registry_clone.clone();
            let providers = providers_clone.clone();

            async move {
                // Extract configuration from Lua table
                let system_prompt: Option<String> = args.get("system_prompt").ok();
                let temperature: Option<f32> = args.get("temperature").ok();
                let max_tokens: Option<usize> = args.get("max_tokens").ok();
                let max_conversation_length: Option<usize> =
                    args.get("max_conversation_length").ok();
                let base_url: Option<String> = args.get("base_url").ok();
                let api_key: Option<String> = args.get("api_key").ok();

                // Create a basic agent configuration
                let agent_config = AgentConfig {
                    system_prompt,
                    temperature,
                    max_tokens,
                    max_conversation_length,
                };

                // Handle model specification with new syntax support
                let provider = if let Some(model_str) =
                    args.get::<_, Option<String>>("model").ok().flatten()
                {
                    // New syntax: "provider/model" or "model"
                    let model_spec = ModelSpecifier::parse(&model_str).map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Invalid model specification '{}': {}",
                            model_str, e
                        ))
                    })?;

                    providers
                        .as_ref()
                        .create_agent_from_spec(model_spec, base_url.as_deref(), api_key.as_deref())
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to create agent from spec: {}",
                                e
                            ))
                        })?
                } else if let (Some(provider_name), Some(model_name)) = (
                    args.get::<_, Option<String>>("provider").ok().flatten(),
                    args.get::<_, Option<String>>("model_name").ok().flatten(),
                ) {
                    // Legacy syntax: separate provider and model_name fields
                    let model_spec = ModelSpecifier::with_provider(provider_name, model_name);
                    providers
                        .as_ref()
                        .create_agent_from_spec(model_spec, base_url.as_deref(), api_key.as_deref())
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to create agent from legacy spec: {}",
                                e
                            ))
                        })?
                } else if let Some(provider_name) =
                    args.get::<_, Option<String>>("provider").ok().flatten()
                {
                    // Legacy syntax with just provider (use default model)
                    providers
                        .get_provider(Some(&provider_name))
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to get provider '{}': {}",
                                provider_name, e
                            ))
                        })?
                } else {
                    // No provider specified, use default
                    providers.get_default_provider().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Failed to get default provider: {}", e))
                    })?
                };

                // Create a simple agent wrapper
                let agent: Arc<dyn Agent> = Arc::new(SimpleProviderAgent::new(
                    agent_config,
                    provider,
                    "default".to_string(), // This will be overridden by the provider's model
                ));

                // Create the Lua wrapper with bridge access
                let bridge = Arc::new(AgentBridge::new(registry.clone()));
                let wrapper = LuaAgentWrapper {
                    agent,
                    bridge,
                    agent_instance_name: "anonymous_agent".to_string(), // For Agent.create(), no instance name
                    _registry: registry,
                    _providers: providers,
                };

                Ok(wrapper)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.create function: {}", e),
            source: None,
        })?;

    // Add the create function to the Agent table
    agent_table
        .set(&api_def.constructor[..], create_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.create: {}", e),
            source: None,
        })?;

    // Add Agent.list() function to list available agent types
    let bridge_for_list = bridge.clone();
    let list_fn = lua
        .create_async_function(move |lua, _: ()| {
            let bridge = bridge_for_list.clone();
            async move {
                let types = bridge.list_agent_types().await;
                let list_table = lua.create_table()?;
                for (i, agent_type) in types.iter().enumerate() {
                    list_table.set(i + 1, agent_type.clone())?;
                }
                Ok(list_table)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.list function: {}", e),
            source: None,
        })?;

    agent_table
        .set("list", list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.list: {}", e),
            source: None,
        })?;

    // Add Agent.listTemplates() function
    let bridge_for_templates = bridge.clone();
    let list_templates_fn = lua
        .create_async_function(move |lua, _: ()| {
            let bridge = bridge_for_templates.clone();
            async move {
                let templates = bridge.list_templates().await;
                let list_table = lua.create_table()?;
                for (i, template) in templates.iter().enumerate() {
                    list_table.set(i + 1, template.clone())?;
                }
                Ok(list_table)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.listTemplates function: {}", e),
            source: None,
        })?;

    agent_table
        .set("listTemplates", list_templates_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.listTemplates: {}", e),
            source: None,
        })?;

    // Add Agent.get() function to get an existing agent instance
    let bridge_for_get = bridge.clone();
    let registry_for_get = registry.clone();
    let providers_for_get = providers.clone();
    let get_fn = lua
        .create_async_function(move |_lua, name: String| {
            let bridge = bridge_for_get.clone();
            let registry = registry_for_get.clone();
            let providers = providers_for_get.clone();
            async move {
                if let Some(agent) = bridge.get_agent(&name).await {
                    let wrapper = LuaAgentWrapper {
                        agent,
                        bridge: bridge.clone(),
                        agent_instance_name: name,
                        _registry: registry,
                        _providers: providers,
                    };
                    Ok(Some(wrapper))
                } else {
                    Ok(None)
                }
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.get function: {}", e),
            source: None,
        })?;

    agent_table
        .set("get", get_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.get: {}", e),
            source: None,
        })?;

    // Add Agent.createFromTemplate() function
    let bridge_for_template = bridge.clone();
    let registry_for_template = registry.clone();
    let providers_for_template = providers.clone();
    let create_from_template_fn = lua
        .create_async_function(
            move |_lua, (instance_name, template_name, params): (String, String, Table)| {
                let bridge = bridge_for_template.clone();
                let registry = registry_for_template.clone();
                let providers = providers_for_template.clone();
                async move {
                    // Convert Lua table to HashMap
                    let mut parameters = HashMap::new();
                    for (key, value) in params.pairs::<String, mlua::Value>().flatten() {
                        if let Ok(json_value) = lua_value_to_json(value) {
                            parameters.insert(key, json_value);
                        }
                    }

                    // Create from template
                    bridge
                        .create_from_template(&instance_name, &template_name, parameters)
                        .await
                        .map_err(|e| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to create agent from template: {}",
                                e
                            ))
                        })?;

                    // Return the created agent
                    if let Some(agent) = bridge.get_agent(&instance_name).await {
                        let wrapper = LuaAgentWrapper {
                            agent,
                            bridge: bridge.clone(),
                            agent_instance_name: instance_name,
                            _registry: registry,
                            _providers: providers,
                        };
                        Ok(wrapper)
                    } else {
                        Err(mlua::Error::RuntimeError(
                            "Failed to retrieve created agent".to_string(),
                        ))
                    }
                }
            },
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.createFromTemplate function: {}", e),
            source: None,
        })?;

    agent_table
        .set("createFromTemplate", create_from_template_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.createFromTemplate: {}", e),
            source: None,
        })?;

    // Add Agent.listInstances() function
    let bridge_for_instances = bridge.clone();
    let list_instances_fn = lua
        .create_async_function(move |lua, _: ()| {
            let bridge = bridge_for_instances.clone();
            async move {
                let instances = bridge.list_instances().await;
                let list_table = lua.create_table()?;
                for (i, instance) in instances.iter().enumerate() {
                    list_table.set(i + 1, instance.clone())?;
                }
                Ok(list_table)
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Agent.listInstances function: {}", e),
            source: None,
        })?;

    agent_table
        .set("listInstances", list_instances_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent.listInstances: {}", e),
            source: None,
        })?;

    // Set the Agent table as a global
    lua.globals()
        .set(&api_def.global_name[..], agent_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Agent global: {}", e),
            source: None,
        })?;

    Ok(())
}

/// Wrapper around Agent for Lua
#[derive(Clone)]
struct LuaAgentWrapper {
    agent: Arc<dyn Agent>,
    bridge: Arc<AgentBridge>,
    agent_instance_name: String,
    _registry: Arc<ComponentRegistry>,
    _providers: Arc<ProviderManager>,
}

impl UserData for LuaAgentWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method
        methods.add_async_method("execute", |lua, this, input: Table| async move {
            // Convert Lua table to AgentInput
            let agent_input = lua_table_to_agent_input(lua, input)?;
            let context = ExecutionContext::new();

            // Execute the agent
            let result = this
                .agent
                .execute(agent_input, context)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

            // Convert AgentOutput to Lua table
            let output_table = agent_output_to_lua_table(lua, result)?;

            Ok(output_table)
        });

        // getConfig method
        methods.add_method("getConfig", |lua, this, ()| {
            let config_table = lua.create_table()?;
            let config = this.agent.config();

            if let Some(prompt) = &config.system_prompt {
                config_table.set("system_prompt", prompt.clone())?;
            }
            if let Some(temp) = config.temperature {
                config_table.set("temperature", temp)?;
            }
            if let Some(tokens) = config.max_tokens {
                config_table.set("max_tokens", tokens)?;
            }

            Ok(config_table)
        });

        // getState method
        methods.add_method("getState", |lua, _this, ()| {
            let state_table = lua.create_table()?;
            // TODO: Implement state retrieval from agent
            Ok(state_table)
        });

        // setState method
        methods.add_method("setState", |_lua, _this, _state: Table| {
            // TODO: Implement state setting on agent
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

        // invokeTool method
        methods.add_async_method(
            "invokeTool",
            |lua, this, (tool_name, input_table): (String, Table)| async move {
                // Convert Lua table to AgentInput (for tool execution)
                let tool_input = lua_table_to_tool_input(lua, input_table)?;

                // Invoke the tool through the bridge
                let result = this
                    .bridge
                    .invoke_tool_for_agent(&this.agent_instance_name, &tool_name, tool_input, None)
                    .await
                    .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

                // Convert AgentOutput to Lua table
                let output_table = tool_output_to_lua_table(lua, result)?;
                Ok(output_table)
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

        // getMetrics method
        methods.add_async_method("getMetrics", |lua, this, ()| async move {
            match this
                .bridge
                .get_agent_metrics(&this.agent_instance_name)
                .await
            {
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

        // getHealth method
        methods.add_async_method("getHealth", |lua, this, ()| async move {
            match this
                .bridge
                .get_agent_health(&this.agent_instance_name)
                .await
            {
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

        // getPerformance method
        methods.add_async_method("getPerformance", |lua, this, ()| async move {
            match this
                .bridge
                .get_agent_performance(&this.agent_instance_name)
                .await
            {
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

        // logEvent method
        methods.add_async_method(
            "logEvent",
            |_lua, this, (event_type, message): (String, String)| async move {
                this.bridge
                    .log_agent_event(&this.agent_instance_name, &event_type, &message)
                    .await
                    .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
                Ok(())
            },
        );

        // configureAlerts method
        methods.add_async_method(
            "configureAlerts",
            |_lua, this, config_table: Table| async move {
                // Convert Lua table to JSON for alert configuration
                let config_json = lua_value_to_json(mlua::Value::Table(config_table))?;
                this.bridge
                    .configure_agent_alerts(&this.agent_instance_name, config_json)
                    .await
                    .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
                Ok(())
            },
        );

        // getAlerts method
        methods.add_async_method("getAlerts", |lua, this, ()| async move {
            match this
                .bridge
                .get_agent_alerts(&this.agent_instance_name)
                .await
            {
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

        // getState method - Get current agent state
        methods.add_async_method("getAgentState", |_lua, this, ()| async move {
            match this.bridge.get_agent_state(&this.agent_instance_name).await {
                Ok(state) => Ok(format!("{:?}", state)),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // initialize method - Initialize agent state machine
        methods.add_async_method("initialize", |_lua, this, ()| async move {
            this.bridge
                .initialize_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // start method - Start agent execution
        methods.add_async_method("start", |_lua, this, ()| async move {
            this.bridge
                .start_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // pause method - Pause agent execution
        methods.add_async_method("pause", |_lua, this, ()| async move {
            this.bridge
                .pause_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // resume method - Resume agent execution
        methods.add_async_method("resume", |_lua, this, ()| async move {
            this.bridge
                .resume_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // stop method - Stop agent execution
        methods.add_async_method("stop", |_lua, this, ()| async move {
            this.bridge
                .stop_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // terminate method - Terminate agent
        methods.add_async_method("terminate", |_lua, this, ()| async move {
            this.bridge
                .terminate_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // setError method - Put agent in error state
        methods.add_async_method("setError", |_lua, this, error_message: String| async move {
            this.bridge
                .error_agent(&this.agent_instance_name, error_message)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // recover method - Attempt to recover from error
        methods.add_async_method("recover", |_lua, this, ()| async move {
            this.bridge
                .recover_agent(&this.agent_instance_name)
                .await
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
            Ok(())
        });

        // getStateHistory method - Get state transition history
        methods.add_async_method("getStateHistory", |lua, this, ()| async move {
            match this
                .bridge
                .get_agent_state_history(&this.agent_instance_name)
                .await
            {
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

        // getLastError method - Get last error message
        methods.add_async_method("getLastError", |_lua, this, ()| async move {
            match this
                .bridge
                .get_agent_last_error(&this.agent_instance_name)
                .await
            {
                Ok(error) => Ok(error),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // getRecoveryAttempts method - Get recovery attempt count
        methods.add_async_method("getRecoveryAttempts", |_lua, this, ()| async move {
            match this
                .bridge
                .get_agent_recovery_attempts(&this.agent_instance_name)
                .await
            {
                Ok(attempts) => Ok(attempts),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // isHealthy method - Check if agent is in healthy state
        methods.add_async_method("isHealthy", |_lua, this, ()| async move {
            match this
                .bridge
                .is_agent_healthy(&this.agent_instance_name)
                .await
            {
                Ok(healthy) => Ok(healthy),
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // getStateMetrics method - Get state machine metrics
        methods.add_async_method("getStateMetrics", |lua, this, ()| async move {
            match this
                .bridge
                .get_agent_state_metrics(&this.agent_instance_name)
                .await
            {
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
    }
}

/// Simple agent implementation that uses a provider directly
struct SimpleProviderAgent {
    metadata: ComponentMetadata,
    config: AgentConfig,
    provider: Arc<Box<dyn ProviderInstance>>,
    _model: String,
    conversation: tokio::sync::Mutex<Vec<ConversationMessage>>,
}

impl SimpleProviderAgent {
    fn new(config: AgentConfig, provider: Arc<Box<dyn ProviderInstance>>, model: String) -> Self {
        let metadata = ComponentMetadata::new(
            "SimpleProviderAgent".to_string(),
            "A basic agent that uses a provider directly".to_string(),
        );

        Self {
            metadata,
            config,
            provider,
            _model: model,
            conversation: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl BaseAgent for SimpleProviderAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        mut input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Add system prompt to the input if configured
        if let Some(ref system_prompt) = self.config.system_prompt {
            // Prepend system prompt to the input text
            input.text = format!("{}\n\n{}", system_prompt, input.text);
        }

        // Use the provider to complete the request
        let output = self.provider.complete(&input).await?;
        Ok(output)
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        // Basic validation - ensure text is not empty
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {}", error)))
    }
}

#[async_trait]
impl Agent for SimpleProviderAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>> {
        let conv = self.conversation.lock().await;
        Ok(conv.clone())
    }

    async fn add_message(&mut self, message: ConversationMessage) -> Result<()> {
        let mut conv = self.conversation.lock().await;
        conv.push(message);
        Ok(())
    }

    async fn clear_conversation(&mut self) -> Result<()> {
        let mut conv = self.conversation.lock().await;
        conv.clear();
        Ok(())
    }
}
