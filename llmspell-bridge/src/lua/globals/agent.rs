//! ABOUTME: Lua-specific Agent global implementation
//! ABOUTME: Provides Lua bindings for Agent functionality

#![allow(clippy::significant_drop_tightening)]

use crate::agent_bridge::AgentBridge;
use crate::globals::GlobalContext;
use crate::lua::conversion::{
    agent_output_to_lua_table, json_to_lua_value, lua_table_to_agent_input, lua_value_to_json,
};
use crate::lua::sync_utils::block_on_async;
use llmspell_agents::{AgentConfig, ModelConfig, ResourceLimits};
use llmspell_core::execution_context::{ContextScope, ExecutionContextBuilder};
use llmspell_core::types::ComponentId;
use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument};

/// Parse `ModelConfig` from Lua table
fn parse_model_config(table: &Table) -> mlua::Result<Option<ModelConfig>> {
    // Check if model field exists
    let model_value: Option<Value> = table.get("model").ok();

    if let Some(Value::Table(model_table)) = model_value {
        let provider: String = model_table.get("provider")?;
        let model_id: String = model_table.get("model_id")?;
        let temperature: Option<f32> = model_table.get("temperature").ok();
        let max_tokens: Option<u32> = model_table.get("max_tokens").ok();

        // Parse settings as JSON map
        let settings_value: Option<Value> = model_table.get("settings").ok();
        let settings = if let Some(Value::Table(settings_table)) = settings_value {
            match lua_value_to_json(Value::Table(settings_table))? {
                serde_json::Value::Object(map) => map,
                _ => serde_json::Map::new(),
            }
        } else {
            serde_json::Map::new()
        };

        Ok(Some(ModelConfig {
            provider,
            model_id,
            temperature,
            max_tokens,
            settings,
        }))
    } else {
        Ok(None)
    }
}

/// Parse `ResourceLimits` from Lua table
fn parse_resource_limits(table: &Table) -> ResourceLimits {
    // Check if resource_limits field exists
    let limits_value: Option<Value> = table.get("resource_limits").ok();

    if let Some(Value::Table(limits_table)) = limits_value {
        ResourceLimits {
            max_execution_time_secs: limits_table.get("max_execution_time_secs").unwrap_or(300),
            max_memory_mb: limits_table.get("max_memory_mb").unwrap_or(512),
            max_tool_calls: limits_table.get("max_tool_calls").unwrap_or(100),
            max_recursion_depth: limits_table.get("max_recursion_depth").unwrap_or(10),
        }
    } else {
        // Use defaults if not specified
        ResourceLimits::default()
    }
}

/// Parse `AgentConfig` from Lua table
///
/// Expected Lua table structure:
/// ```lua
/// {
///     name = "my-agent",
///     description = "Agent description",
///     agent_type = "llm", -- or "type" (both supported)
///     model = {
///         provider = "openai",
///         model_id = "gpt-3.5-turbo",
///         temperature = 0.7,
///         max_tokens = 150,
///         settings = {}
///     },
///     allowed_tools = {"tool1", "tool2"},
///     custom_config = {
///         system_prompt = "You are...",
///         ...
///     },
///     resource_limits = {
///         max_execution_time_secs = 300,
///         max_memory_mb = 512,
///         max_tool_calls = 100,
///         max_recursion_depth = 10
///     }
/// }
/// ```
fn parse_agent_config(table: &Table) -> mlua::Result<AgentConfig> {
    // Extract name (required)
    let name: String = table.get("name").unwrap_or_else(|_| {
        // Generate UUID-based name if not provided
        format!(
            "agent_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        )
    });

    // Extract description (default empty)
    let description: String = table.get("description").unwrap_or_default();

    // Extract agent_type - support both "agent_type" and "type" for compatibility
    let agent_type: String = table
        .get("agent_type")
        .or_else(|_| table.get("type"))
        .unwrap_or_else(|_| "llm".to_string());

    // Parse model config (optional)
    let model = parse_model_config(table)?;

    // Parse allowed_tools (default empty array)
    let allowed_tools_value: Option<Value> = table.get("allowed_tools").ok();
    let allowed_tools = if let Some(Value::Table(tools_table)) = allowed_tools_value {
        let mut tools = Vec::new();
        for i in 1..=tools_table.raw_len() {
            if let Ok(tool_name) = tools_table.get::<_, String>(i) {
                tools.push(tool_name);
            }
        }
        tools
    } else {
        Vec::new()
    };

    // Parse custom_config (default empty map)
    let custom_config_value: Option<Value> = table.get("custom_config").ok();
    let custom_config = if let Some(Value::Table(config_table)) = custom_config_value {
        match lua_value_to_json(Value::Table(config_table))? {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        }
    } else {
        serde_json::Map::new()
    };

    // Parse resource_limits (defaults if not provided)
    let resource_limits = parse_resource_limits(table);

    Ok(AgentConfig {
        name,
        description,
        agent_type,
        model,
        allowed_tools,
        custom_config,
        resource_limits,
    })
}

/// Parse `ContextScope` from a Lua value (string or table)
///
/// Supports both simple string format and full table:
/// - Simple: "global"
/// - Full: { type = "session", id = "`sess_123`" }
fn parse_context_scope(
    value: &Value,
) -> mlua::Result<llmspell_core::execution_context::ContextScope> {
    use llmspell_core::execution_context::ContextScope;
    use llmspell_core::types::ComponentId;

    match value {
        Value::String(s) => {
            let scope_str = s.to_str()?;
            match scope_str {
                "global" => Ok(ContextScope::Global),
                _ => Err(mlua::Error::FromLuaConversionError {
                    from: "string",
                    to: "ContextScope",
                    message: Some(format!("Invalid simple scope: {scope_str}. Use table for session/workflow/agent/user scopes")),
                }),
            }
        }
        Value::Table(table) => {
            let scope_type: String = table.get("type")?;
            match scope_type.as_str() {
                "global" => Ok(ContextScope::Global),
                "session" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::Session(id))
                }
                "workflow" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::Workflow(id))
                }
                "agent" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::Agent(ComponentId::from_name(&id)))
                }
                "user" => {
                    let id: String = table.get("id")?;
                    Ok(ContextScope::User(id))
                }
                _ => Err(mlua::Error::FromLuaConversionError {
                    from: "table",
                    to: "ContextScope",
                    message: Some(format!("Invalid scope type: {scope_type}")),
                }),
            }
        }
        _ => Err(mlua::Error::FromLuaConversionError {
            from: value.type_name(),
            to: "ContextScope",
            message: Some("Expected string or table for scope".to_string()),
        }),
    }
}

/// Parse `InheritancePolicy` from a string
fn parse_inheritance_policy(value: &str) -> llmspell_core::execution_context::InheritancePolicy {
    use llmspell_core::execution_context::InheritancePolicy;

    match value {
        "isolate" => InheritancePolicy::Isolate,
        "copy" => InheritancePolicy::Copy,
        "share" => InheritancePolicy::Share,
        _ => InheritancePolicy::Inherit,
    }
}

/// Parse `ExecutionContextConfig` from a Lua table
fn parse_execution_context_config(
    table: &Table,
) -> mlua::Result<crate::agent_bridge::ExecutionContextConfig> {
    use crate::agent_bridge::{ExecutionContextConfig, SecurityContextConfig};

    // Parse optional string fields
    let conversation_id: Option<String> = table.get("conversation_id").ok();
    let user_id: Option<String> = table.get("user_id").ok();
    let session_id: Option<String> = table.get("session_id").ok();

    // Parse scope
    let scope = table
        .get::<_, Value>("scope")
        .ok()
        .map(|v| parse_context_scope(&v))
        .transpose()?;

    // Parse inheritance
    let inheritance = table
        .get::<_, String>("inheritance")
        .ok()
        .map(|s| parse_inheritance_policy(&s));

    // Parse data map
    let data = table.get::<_, Value>("data").ok().and_then(|v| {
        if let Value::Table(data_table) = v {
            match lua_value_to_json(Value::Table(data_table)) {
                Ok(serde_json::Value::Object(map)) => Some(map.into_iter().collect()),
                _ => None,
            }
        } else {
            None
        }
    });

    // Parse security
    let security = table.get::<_, Value>("security").ok().and_then(|v| {
        if let Value::Table(sec_table) = v {
            let permissions_val: Option<Value> = sec_table.get("permissions").ok();
            let permissions = if let Some(Value::Table(perms_table)) = permissions_val {
                let mut perms = Vec::new();
                for i in 1..=perms_table.raw_len() {
                    if let Ok(perm) = perms_table.get::<_, String>(i) {
                        perms.push(perm);
                    }
                }
                perms
            } else {
                Vec::new()
            };

            let level: String = sec_table
                .get("level")
                .unwrap_or_else(|_| "default".to_string());

            Some(SecurityContextConfig { permissions, level })
        } else {
            None
        }
    });

    Ok(ExecutionContextConfig {
        conversation_id,
        user_id,
        session_id,
        scope,
        inheritance,
        data,
        security,
    })
}

/// Parse `ChildContextConfig` from scope and inheritance values
fn parse_child_context_config(
    scope_value: &Value,
    inheritance_str: &str,
) -> mlua::Result<crate::agent_bridge::ChildContextConfig> {
    use crate::agent_bridge::ChildContextConfig;

    let scope = parse_context_scope(scope_value)?;
    let inheritance = parse_inheritance_policy(inheritance_str);

    Ok(ChildContextConfig { scope, inheritance })
}

/// Parse `RoutingConfig` from a Lua table or simple string
///
/// Supports both simple string format and full config table:
/// - Simple: "sequential", "parallel", "vote"
/// - Full: { strategy = "sequential", `fallback_agent` = "default", `timeout_ms` = 5000 }
fn parse_routing_config(value: &Value) -> mlua::Result<crate::agent_bridge::RoutingConfig> {
    use crate::agent_bridge::{RoutingConfig, RoutingStrategy};

    match value {
        // Simple string format - just strategy name
        Value::String(s) => {
            let strategy_str = s.to_str()?;
            let strategy = match strategy_str {
                "sequential" => RoutingStrategy::Sequential,
                "parallel" => RoutingStrategy::Parallel,
                "vote" => RoutingStrategy::Vote { threshold: None },
                custom => RoutingStrategy::Custom {
                    name: custom.to_string(),
                },
            };
            Ok(RoutingConfig {
                strategy,
                fallback_agent: None,
                timeout_ms: None,
            })
        }
        // Full config table
        Value::Table(table) => {
            // Parse strategy (required)
            let strategy = table.get::<_, String>("strategy").map_or(
                RoutingStrategy::Sequential,
                |strategy_str| match strategy_str.as_str() {
                    "sequential" => RoutingStrategy::Sequential,
                    "parallel" => RoutingStrategy::Parallel,
                    "vote" => {
                        let threshold: Option<usize> = table.get("threshold").ok();
                        RoutingStrategy::Vote { threshold }
                    }
                    custom => RoutingStrategy::Custom {
                        name: custom.to_string(),
                    },
                },
            );

            // Parse optional fields
            let fallback_agent: Option<String> = table.get("fallback_agent").ok();
            let timeout_ms: Option<u64> = table.get("timeout_ms").ok();

            Ok(RoutingConfig {
                strategy,
                fallback_agent,
                timeout_ms,
            })
        }
        _ => Err(mlua::Error::FromLuaConversionError {
            from: value.type_name(),
            to: "RoutingConfig",
            message: Some("Expected string or table for routing config".to_string()),
        }),
    }
}

/// Parse `ToolWrapperConfig` from a Lua table
///
/// Supports both minimal and full configuration:
/// - Minimal: { `tool_name` = `"my_tool"` }
/// - Full: { `tool_name` = `"my_tool"`, category = "api", `security_level` = "restricted" }
fn parse_tool_wrapper_config(table: &Table) -> crate::agent_bridge::ToolWrapperConfig {
    use crate::agent_bridge::ToolWrapperConfig;
    use llmspell_core::traits::tool::{SecurityLevel, ToolCategory};

    let tool_name: String = table.get("tool_name").unwrap_or_else(|_| String::new());

    // Parse category (optional)
    let category = table
        .get::<_, Option<String>>("category")
        .unwrap_or(None)
        .map(|cat_str| match cat_str.as_str() {
            "filesystem" => ToolCategory::Filesystem,
            "web" => ToolCategory::Web,
            "api" => ToolCategory::Api,
            "analysis" => ToolCategory::Analysis,
            "data" => ToolCategory::Data,
            "system" => ToolCategory::System,
            "media" => ToolCategory::Media,
            "utility" => ToolCategory::Utility,
            custom => ToolCategory::Custom(custom.to_string()),
        });

    // Parse security_level (optional)
    let security_level = table
        .get::<_, Option<String>>("security_level")
        .unwrap_or(None)
        .and_then(|level_str| match level_str.as_str() {
            "safe" => Some(SecurityLevel::Safe),
            "restricted" => Some(SecurityLevel::Restricted),
            "privileged" => Some(SecurityLevel::Privileged),
            _ => None,
        });

    ToolWrapperConfig {
        tool_name,
        category,
        security_level,
    }
}

/// Parse `AlertConditionConfig` from a Lua table
fn parse_alert_condition(table: &Table) -> mlua::Result<crate::agent_bridge::AlertConditionConfig> {
    use crate::agent_bridge::AlertConditionConfig;

    let condition_type: String = table.get("type")?;

    match condition_type.as_str() {
        "metric_threshold" => {
            let metric_name: String = table.get("metric_name")?;
            let operator: String = table.get("operator")?;
            let threshold: f64 = table.get("threshold")?;
            let duration_seconds: u64 = table.get("duration_seconds").unwrap_or(60);

            Ok(AlertConditionConfig::MetricThreshold {
                metric_name,
                operator,
                threshold,
                duration_seconds,
            })
        }
        "health_status" => {
            let status: String = table.get("status")?;
            let duration_seconds: u64 = table.get("duration_seconds").unwrap_or(60);

            Ok(AlertConditionConfig::HealthStatus {
                status,
                duration_seconds,
            })
        }
        "error_rate" => {
            let rate_percent: f64 = table.get("rate_percent")?;
            let duration_seconds: u64 = table.get("duration_seconds").unwrap_or(60);

            Ok(AlertConditionConfig::ErrorRate {
                rate_percent,
                duration_seconds,
            })
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown alert condition type: {condition_type}"
        ))),
    }
}

/// Parse `AlertConfig` from a Lua table
///
/// Expected fields:
/// - name: string (required)
/// - severity: string (optional, defaults to "warning")
/// - condition: table with type field (required)
/// - `cooldown_seconds`: number (optional)
/// - enabled: boolean (optional, defaults to true)
fn parse_alert_config(table: &Table) -> mlua::Result<crate::agent_bridge::BridgeAlertConfig> {
    use crate::agent_bridge::BridgeAlertConfig;

    let name: String = table.get("name")?;
    let severity: String = table
        .get("severity")
        .unwrap_or_else(|_| "warning".to_string());

    let condition_table: Table = table.get("condition")?;
    let condition = parse_alert_condition(&condition_table)?;

    let cooldown_seconds: Option<u64> = table.get("cooldown_seconds").ok();
    let enabled: bool = table.get("enabled").unwrap_or(true);

    Ok(BridgeAlertConfig {
        name,
        severity,
        condition,
        cooldown_seconds,
        enabled,
    })
}

/// Lua userdata representing an agent instance
struct LuaAgentInstance {
    agent_instance_name: String,
    bridge: Arc<AgentBridge>,
    global_context: Arc<GlobalContext>,
}

impl UserData for LuaAgentInstance {
    #[allow(clippy::too_many_lines)]
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method - synchronous wrapper
        methods.add_method("execute", |lua, this, input: Table| {
            let agent_input = lua_table_to_agent_input(lua, &input)?;
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();
            let global_context = this.global_context.clone();

            // Create ExecutionContext with state if available
            let context = global_context.state_access.as_ref().map(|state_access| {
                ExecutionContextBuilder::new()
                    .scope(ContextScope::Agent(ComponentId::from_name(&agent_name)))
                    .state(state_access.clone())
                    .build()
            });

            // Use shared sync utility to execute async code
            let result = block_on_async(
                "agent_execute",
                async move {
                    bridge
                        .execute_agent(&agent_name, agent_input, context)
                        .await
                },
                None,
            )?;

            agent_output_to_lua_table(lua, &result)
        });

        // invokeStream method - synchronous wrapper
        methods.add_method(
            "invokeStream",
            |lua, this, (input, callback): (Table, mlua::Function)| {
                let agent_input = lua_table_to_agent_input(lua, &input)?;
                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();
                let global_context = this.global_context.clone();

                // Create ExecutionContext with state if available
                let context = global_context.state_access.as_ref().map(|state_access| {
                    ExecutionContextBuilder::new()
                        .scope(ContextScope::Agent(ComponentId::from_name(&agent_name)))
                        .state(state_access.clone())
                        .build()
                });

                // Use block_on to handle the streaming operation
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        // Get streaming receiver
                        let mut rx = bridge
                            .execute_agent_streaming(&agent_name, agent_input, context)
                            .await
                            .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

                        // Process stream
                        let mut chunk_count = 0;
                        while let Some(output) = rx.recv().await {
                            let output_table = agent_output_to_lua_table(lua, &output)?;
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

        // get_state method - synchronous wrapper
        methods.add_method("get_state", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let state = block_on_async(
                "agent_getState",
                async move { bridge.get_agent_state(&agent_name).await },
                None,
            )?;
            Ok(format!("{state:?}"))
        });

        // getConfig method
        methods.add_method("get_config", |lua, _this, ()| {
            // TODO: Get agent configuration from bridge when API is available
            // For now, return empty config table
            let config_table = lua.create_table()?;
            Ok(config_table)
        });

        // set_state method
        methods.add_method("set_state", |_lua, _this, _state: Table| {
            // TODO: Implement state setting when bridge supports it
            Ok(())
        });

        // save_state method - synchronous wrapper
        methods.add_method("save_state", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_saveState",
                async move { bridge.save_agent_state(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // load_state method - synchronous wrapper
        methods.add_method("load_state", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let loaded = block_on_async(
                "agent_loadState",
                async move { bridge.load_agent_state(&agent_name).await },
                None,
            )?;
            Ok(loaded)
        });

        // delete_state method - synchronous wrapper
        methods.add_method("delete_state", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_deleteState",
                async move { bridge.delete_agent_state(&agent_name).await },
                None,
            )?;
            Ok(())
        });

        // Tool Integration Methods

        // discover_tools method
        methods.add_method("discover_tools", |lua, this, ()| {
            let tools = this.bridge.list_tools();
            let tools_table = lua.create_table()?;
            for (i, tool_name) in tools.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let lua_index = (i + 1) as i32;
                tools_table.set(lua_index, tool_name.clone())?;
            }
            Ok(tools_table)
        });

        // get_tool_metadata method
        methods.add_method("get_tool_metadata", |lua, this, tool_name: String| {
            if let Some(metadata) = this.bridge.get_tool_metadata(&tool_name) {
                // Convert JSON to Lua table
                let metadata_table = lua.create_table()?;
                if let Some(name) = metadata.get("name").and_then(serde_json::Value::as_str) {
                    metadata_table.set("name", name)?;
                }
                if let Some(desc) = metadata
                    .get("description")
                    .and_then(serde_json::Value::as_str)
                {
                    metadata_table.set("description", desc)?;
                }
                if let Some(version) = metadata.get("version").and_then(serde_json::Value::as_str) {
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

                let mut builder = llmspell_core::types::AgentInput::builder()
                    .text(format!("Invoking tool: {tool_name}"));

                // Add parameters
                for (key, value) in wrapped_params {
                    builder = builder.parameter(key, value);
                }

                let agent_input = builder.build();

                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();
                let global_context = this.global_context.clone();

                // Create ExecutionContext with state if available
                let context = global_context.state_access.as_ref().map(|state_access| {
                    ExecutionContextBuilder::new()
                        .scope(ContextScope::Agent(ComponentId::from_name(&agent_name)))
                        .state(state_access.clone())
                        .build()
                });

                // Invoke the tool through the bridge
                let result = block_on_async(
                    "agent_invokeTool",
                    async move {
                        bridge
                            .invoke_tool_for_agent(&agent_name, &tool_name, agent_input, context)
                            .await
                    },
                    None,
                )?;

                // Convert AgentOutput to Lua table
                agent_output_to_lua_table(lua, &result)
            },
        );

        // has_tool method
        methods.add_method("has_tool", |_lua, this, tool_name: String| {
            Ok(this.bridge.has_tool(&tool_name))
        });

        // get_all_tool_metadata method
        methods.add_method("get_all_tool_metadata", |lua, this, ()| {
            let all_metadata = this.bridge.get_all_tool_metadata();
            let metadata_table = lua.create_table()?;

            for (tool_name, metadata) in all_metadata {
                let tool_metadata_table = lua.create_table()?;
                if let Some(name) = metadata.get("name").and_then(serde_json::Value::as_str) {
                    tool_metadata_table.set("name", name)?;
                }
                if let Some(desc) = metadata
                    .get("description")
                    .and_then(serde_json::Value::as_str)
                {
                    tool_metadata_table.set("description", desc)?;
                }
                if let Some(version) = metadata.get("version").and_then(serde_json::Value::as_str) {
                    tool_metadata_table.set("version", version)?;
                }
                metadata_table.set(tool_name, tool_metadata_table)?;
            }

            Ok(metadata_table)
        });

        // Monitoring & Lifecycle Methods

        // get_metrics method - synchronous wrapper
        methods.add_method("get_metrics", |lua, this, ()| {
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
                    #[allow(clippy::cast_precision_loss)]
                    let requests_total = metrics.requests_total.get() as f64;
                    #[allow(clippy::cast_precision_loss)]
                    let requests_failed = metrics.requests_failed.get() as f64;
                    #[allow(clippy::cast_precision_loss)]
                    let tool_invocations = metrics.tool_invocations.get() as f64;
                    metrics_table.set("requests_total", requests_total)?;
                    metrics_table.set("requests_failed", requests_failed)?;
                    metrics_table.set("requests_active", metrics.requests_active.get())?;
                    metrics_table.set("tool_invocations", tool_invocations)?;
                    metrics_table.set("memory_bytes", metrics.memory_bytes.get())?;
                    metrics_table.set("cpu_percent", metrics.cpu_percent.get())?;
                    Ok(Some(metrics_table))
                }
                Err(_) => Ok(None),
            }
        });

        // get_health method - synchronous wrapper
        methods.add_method("get_health", |lua, this, ()| {
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
                    if let Some(status) = health_json
                        .get("status")
                        .and_then(serde_json::Value::as_str)
                    {
                        health_table.set("status", status)?;
                    }
                    if let Some(message) = health_json
                        .get("message")
                        .and_then(serde_json::Value::as_str)
                    {
                        health_table.set("message", message)?;
                    }
                    if let Some(timestamp) = health_json
                        .get("timestamp")
                        .and_then(serde_json::Value::as_str)
                    {
                        health_table.set("timestamp", timestamp)?;
                    }
                    Ok(Some(health_table))
                }
                Err(_) => Ok(None),
            }
        });

        // get_performance method - synchronous wrapper
        methods.add_method("get_performance", |lua, this, ()| {
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
                    if let Some(total_executions) = perf_json
                        .get("total_executions")
                        .and_then(serde_json::Value::as_u64)
                    {
                        #[allow(clippy::cast_precision_loss)]
                        let total_exec_f64 = total_executions as f64;
                        perf_table.set("total_executions", total_exec_f64)?;
                    }
                    if let Some(avg_time) = perf_json
                        .get("avg_execution_time_ms")
                        .and_then(serde_json::Value::as_f64)
                    {
                        perf_table.set("avg_execution_time_ms", avg_time)?;
                    }
                    if let Some(success_rate) = perf_json
                        .get("success_rate")
                        .and_then(serde_json::Value::as_f64)
                    {
                        perf_table.set("success_rate", success_rate)?;
                    }
                    if let Some(error_rate) = perf_json
                        .get("error_rate")
                        .and_then(serde_json::Value::as_f64)
                    {
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

        // configure_alerts method - synchronous wrapper
        methods.add_method("configure_alerts", |_lua, this, config_table: Table| {
            // Parse Lua table to typed alert config
            let config_typed = parse_alert_config(&config_table)?;
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            block_on_async(
                "agent_configureAlerts",
                async move {
                    bridge
                        .configure_agent_alerts(&agent_name, config_typed)
                        .await
                },
                None,
            )?;
            Ok(())
        });

        // get_alerts method - synchronous wrapper
        methods.add_method("get_alerts", |lua, this, ()| {
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
                        let alert_item = lua.create_table()?;
                        if let Some(severity) =
                            alert.get("severity").and_then(serde_json::Value::as_str)
                        {
                            alert_item.set("severity", severity)?;
                        }
                        if let Some(message) =
                            alert.get("message").and_then(serde_json::Value::as_str)
                        {
                            alert_item.set("message", message)?;
                        }
                        if let Some(timestamp) =
                            alert.get("timestamp").and_then(serde_json::Value::as_str)
                        {
                            alert_item.set("timestamp", timestamp)?;
                        }
                        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                        let lua_index = (i + 1) as i32;
                        alerts_table.set(lua_index, alert_item)?;
                    }
                    Ok(alerts_table)
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // get_bridge_metrics method (static bridge-wide metrics)
        methods.add_method("get_bridge_metrics", |lua, this, ()| {
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

        // get_agent_state method - Get current agent state with full details - synchronous wrapper
        methods.add_method("get_agent_state", |_, this, ()| {
            let bridge = this.bridge.clone();
            let agent_name = this.agent_instance_name.clone();

            let state = block_on_async(
                "agent_getAgentState",
                async move { bridge.get_agent_state(&agent_name).await },
                None,
            )?;
            Ok(format!("{state:?}"))
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

        // set_error method - synchronous wrapper
        methods.add_method("set_error", |_, this, error_message: String| {
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

        // get_state_history method - synchronous wrapper
        methods.add_method("get_state_history", |lua, this, ()| {
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
                        if let Some(from) =
                            transition.get("from").and_then(serde_json::Value::as_str)
                        {
                            transition_table.set("from", from)?;
                        }
                        if let Some(to) = transition.get("to").and_then(serde_json::Value::as_str) {
                            transition_table.set("to", to)?;
                        }
                        if let Some(timestamp) = transition
                            .get("timestamp")
                            .and_then(serde_json::Value::as_str)
                        {
                            transition_table.set("timestamp", timestamp)?;
                        }
                        if let Some(elapsed) = transition
                            .get("elapsed")
                            .and_then(serde_json::Value::as_f64)
                        {
                            transition_table.set("elapsed", elapsed)?;
                        }
                        if let Some(reason) =
                            transition.get("reason").and_then(serde_json::Value::as_str)
                        {
                            transition_table.set("reason", reason)?;
                        }
                        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                        let lua_index = (i + 1) as i32;
                        history_table.set(lua_index, transition_table)?;
                    }
                    Ok(history_table)
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        });

        // get_last_error method - synchronous wrapper
        methods.add_method("get_last_error", |_, this, ()| {
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

        // get_recovery_attempts method - synchronous wrapper
        methods.add_method("get_recovery_attempts", |_, this, ()| {
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

        // is_healthy method - synchronous wrapper
        methods.add_method("is_healthy", |_, this, ()| {
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

        // get_state_metrics method - synchronous wrapper
        methods.add_method("get_state_metrics", |lua, this, ()| {
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
                    if let Some(state) = metrics_json
                        .get("current_state")
                        .and_then(serde_json::Value::as_str)
                    {
                        metrics_table.set("current_state", state)?;
                    }
                    if let Some(transitions) = metrics_json
                        .get("total_transitions")
                        .and_then(serde_json::Value::as_u64)
                    {
                        #[allow(clippy::cast_precision_loss)]
                        let transitions_f64 = transitions as f64;
                        metrics_table.set("total_transitions", transitions_f64)?;
                    }
                    if let Some(errors) = metrics_json
                        .get("error_count")
                        .and_then(serde_json::Value::as_u64)
                    {
                        #[allow(clippy::cast_precision_loss)]
                        let errors_f64 = errors as f64;
                        metrics_table.set("error_count", errors_f64)?;
                    }
                    if let Some(attempts) = metrics_json
                        .get("recovery_attempts")
                        .and_then(serde_json::Value::as_u64)
                    {
                        #[allow(clippy::cast_precision_loss)]
                        let attempts_f64 = attempts as f64;
                        metrics_table.set("recovery_attempts", attempts_f64)?;
                    }
                    if let Some(uptime) = metrics_json
                        .get("uptime")
                        .and_then(serde_json::Value::as_f64)
                    {
                        metrics_table.set("uptime", uptime)?;
                    }
                    if let Some(last_transition) = metrics_json
                        .get("last_transition")
                        .and_then(serde_json::Value::as_str)
                    {
                        metrics_table.set("last_transition", last_transition)?;
                    }
                    if let Some(state_dist) = metrics_json
                        .get("state_time_distribution")
                        .and_then(serde_json::Value::as_object)
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

        // execute_with_context method - synchronous wrapper
        methods.add_method(
            "execute_with_context",
            |lua, this, (input, context_id): (Table, String)| {
                let agent_input = lua_table_to_agent_input(lua, &input)?;
                let bridge = this.bridge.clone();
                let agent_name = this.agent_instance_name.clone();

                // Use shared sync utility to execute async code
                let result = block_on_async(
                    "agent_execute_with_context",
                    async move {
                        bridge
                            .execute_agent_with_context(&agent_name, agent_input, &context_id)
                            .await
                    },
                    None,
                )?;

                agent_output_to_lua_table(lua, &result)
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

/// Lua userdata representing an agent builder
#[derive(Clone)]
struct AgentBuilder {
    bridge: Arc<AgentBridge>,
    global_context: Arc<GlobalContext>,
    name: Option<String>,
    description: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    system_prompt: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    max_conversation_length: Option<usize>,
    base_url: Option<String>,
    api_key: Option<String>,
    allowed_tools: Vec<String>,
    max_execution_time_secs: Option<u64>,
    max_memory_mb: Option<u32>,
    max_tool_calls: Option<u32>,
    max_recursion_depth: Option<u32>,
}

impl AgentBuilder {
    const fn new(bridge: Arc<AgentBridge>, global_context: Arc<GlobalContext>) -> Self {
        Self {
            bridge,
            global_context,
            name: None,
            description: None,
            model: None,
            provider: None,
            system_prompt: None,
            temperature: None,
            max_tokens: None,
            max_conversation_length: None,
            base_url: None,
            api_key: None,
            allowed_tools: Vec::new(),
            max_execution_time_secs: None,
            max_memory_mb: None,
            max_tool_calls: None,
            max_recursion_depth: None,
        }
    }
}

#[allow(clippy::too_many_lines)]
impl UserData for AgentBuilder {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // name method
        methods.add_method_mut("name", |_, this, name: String| {
            this.name = Some(name);
            Ok(this.clone())
        });

        // description method
        methods.add_method_mut("description", |_, this, description: String| {
            this.description = Some(description);
            Ok(this.clone())
        });

        // model method - supports "provider/model" syntax
        methods.add_method_mut("model", |_, this, model: String| {
            if model.contains('/') {
                let parts: Vec<&str> = model.splitn(2, '/').collect();
                this.provider = Some(parts[0].to_string());
                this.model = Some(parts[1].to_string());
            } else {
                this.model = Some(model);
                if this.provider.is_none() {
                    this.provider = Some("openai".to_string());
                }
            }
            Ok(this.clone())
        });

        // provider method
        methods.add_method_mut("provider", |_, this, provider: String| {
            this.provider = Some(provider);
            Ok(this.clone())
        });

        // system_prompt method
        methods.add_method_mut("system_prompt", |_, this, prompt: String| {
            this.system_prompt = Some(prompt);
            Ok(this.clone())
        });

        // temperature method
        methods.add_method_mut("temperature", |_, this, temp: f32| {
            this.temperature = Some(temp);
            Ok(this.clone())
        });

        // max_tokens method
        methods.add_method_mut("max_tokens", |_, this, tokens: u32| {
            this.max_tokens = Some(tokens);
            Ok(this.clone())
        });

        // max_conversation_length method
        methods.add_method_mut("max_conversation_length", |_, this, length: usize| {
            this.max_conversation_length = Some(length);
            Ok(this.clone())
        });

        // base_url method
        methods.add_method_mut("base_url", |_, this, url: String| {
            this.base_url = Some(url);
            Ok(this.clone())
        });

        // api_key method
        methods.add_method_mut("api_key", |_, this, key: String| {
            this.api_key = Some(key);
            Ok(this.clone())
        });

        // allowed_tools method - can be called multiple times
        methods.add_method_mut("allow_tool", |_, this, tool: String| {
            this.allowed_tools.push(tool);
            Ok(this.clone())
        });

        // allowed_tools method - set all at once
        methods.add_method_mut("allowed_tools", |_, this, tools: Vec<String>| {
            this.allowed_tools = tools;
            Ok(this.clone())
        });

        // Resource limit methods
        methods.add_method_mut("max_execution_time", |_, this, secs: u64| {
            this.max_execution_time_secs = Some(secs);
            Ok(this.clone())
        });

        methods.add_method_mut("max_memory_mb", |_, this, mb: u32| {
            this.max_memory_mb = Some(mb);
            Ok(this.clone())
        });

        methods.add_method_mut("max_tool_calls", |_, this, calls: u32| {
            this.max_tool_calls = Some(calls);
            Ok(this.clone())
        });

        methods.add_method_mut("max_recursion_depth", |_, this, depth: u32| {
            this.max_recursion_depth = Some(depth);
            Ok(this.clone())
        });

        // type method - for compatibility (always returns "llm" for now)
        methods.add_method_mut("type", |_, this, _agent_type: String| {
            // Currently only LLM agents are supported, so we ignore the input
            // This method exists for API compatibility
            Ok(this.clone())
        });

        // custom_config method - convenience method for setting multiple configs
        methods.add_method_mut("custom_config", |_, this, config: Table| {
            // Extract system_prompt if present
            if let Ok(prompt) = config.get::<_, String>("system_prompt") {
                this.system_prompt = Some(prompt);
            }

            // Extract max_conversation_length if present
            if let Ok(len) = config.get::<_, usize>("max_conversation_length") {
                this.max_conversation_length = Some(len);
            }

            Ok(this.clone())
        });

        // resource_limits method - convenience method for setting all resource limits
        methods.add_method_mut("resource_limits", |_, this, limits: Table| {
            if let Ok(secs) = limits.get::<_, u64>("max_execution_time_secs") {
                this.max_execution_time_secs = Some(secs);
            }

            if let Ok(mb) = limits.get::<_, u32>("max_memory_mb") {
                this.max_memory_mb = Some(mb);
            }

            if let Ok(calls) = limits.get::<_, u32>("max_tool_calls") {
                this.max_tool_calls = Some(calls);
            }

            if let Ok(depth) = limits.get::<_, u32>("max_recursion_depth") {
                this.max_recursion_depth = Some(depth);
            }

            Ok(this.clone())
        });

        // build method - creates the agent
        methods.add_method("build", |_lua, this, ()| {
            // Validate required fields
            let model = this
                .model
                .as_ref()
                .ok_or_else(|| mlua::Error::RuntimeError("Model is required".to_string()))?;

            let provider = this
                .provider
                .as_ref()
                .ok_or_else(|| mlua::Error::RuntimeError("Provider is required".to_string()))?;

            // Generate name if not provided
            let name = this.name.clone().unwrap_or_else(|| {
                format!(
                    "agent_{}",
                    uuid::Uuid::new_v4()
                        .to_string()
                        .chars()
                        .take(8)
                        .collect::<String>()
                )
            });

            // Create model configuration struct
            let mut settings = serde_json::Map::new();
            if let Some(base_url) = &this.base_url {
                settings.insert("base_url".to_string(), serde_json::json!(base_url));
            }
            if let Some(api_key) = &this.api_key {
                settings.insert("api_key".to_string(), serde_json::json!(api_key));
            }

            let model_config = ModelConfig {
                provider: provider.clone(),
                model_id: model.clone(),
                temperature: this.temperature,
                max_tokens: this.max_tokens,
                settings,
            };

            // Create custom config
            let mut custom_config = serde_json::Map::new();
            if let Some(prompt) = &this.system_prompt {
                custom_config.insert("system_prompt".to_string(), serde_json::json!(prompt));
            }
            if let Some(len) = this.max_conversation_length {
                custom_config.insert(
                    "max_conversation_length".to_string(),
                    serde_json::json!(len),
                );
            }

            // Create resource limits struct
            #[allow(clippy::cast_possible_truncation)]
            let resource_limits = ResourceLimits {
                max_execution_time_secs: this.max_execution_time_secs.unwrap_or(300),
                max_memory_mb: this.max_memory_mb.unwrap_or(512).into(),
                max_tool_calls: this.max_tool_calls.unwrap_or(100),
                max_recursion_depth: this.max_recursion_depth.unwrap_or(10) as u8,
            };

            // Create typed AgentConfig struct
            let agent_config = AgentConfig {
                name: name.clone(),
                description: this
                    .description
                    .clone()
                    .unwrap_or_else(|| "LLM-powered agent".to_string()),
                agent_type: "llm".to_string(),
                model: Some(model_config),
                allowed_tools: this.allowed_tools.clone(),
                custom_config,
                resource_limits,
            };

            // Create agent using bridge with typed config
            let bridge = this.bridge.clone();
            let agent_name_clone = name;

            block_on_async(
                "agent_builder_create",
                bridge.create_agent(agent_config),
                None,
            )?;

            // Return agent instance
            Ok(LuaAgentInstance {
                agent_instance_name: agent_name_clone,
                bridge: this.bridge.clone(),
                global_context: this.global_context.clone(),
            })
        });
    }
}

/// Inject Agent global into Lua environment
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function binding fails
#[allow(clippy::too_many_lines)]
#[instrument(
    level = "info",
    skip(lua, context, bridge),
    fields(global_name = "Agent", agent_count = 0)
)]
pub fn inject_agent_global(
    lua: &Lua,
    context: &GlobalContext,
    bridge: Arc<AgentBridge>,
) -> mlua::Result<()> {
    info!("Injecting Agent global API");
    let agent_table = lua.create_table()?;

    // Store bridge reference in context for cross-global access
    context.set_bridge("agent", bridge.clone());

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
        let agent_types = bridge_clone.list_agent_types();
        let discover_table = lua.create_table()?;
        for (i, agent_type) in agent_types.into_iter().enumerate() {
            let agent_table = lua.create_table()?;
            agent_table.set("type", agent_type)?;
            discover_table.set(i + 1, agent_table)?;
        }
        Ok(discover_table)
    })?;

    // Create Agent.wrap_as_tool() function
    let bridge_clone = bridge.clone();
    let wrap_as_tool_fn = lua.create_function(move |_lua, args: (String, Table)| {
        let (agent_name, config) = args;
        let bridge = bridge_clone.clone();

        // Parse Lua table to typed config
        let config_typed = parse_tool_wrapper_config(&config);

        // Use sync wrapper to call async method
        let tool_name = block_on_async(
            "agent_wrapAsTool",
            bridge.wrap_agent_as_tool(&agent_name, config_typed),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to wrap agent as tool: {e}")))?;

        Ok(tool_name)
    })?;

    // Create Agent.get_info() function
    let bridge_clone = bridge.clone();
    let get_info_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let agent_info = bridge
            .get_agent_info(&agent_name)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get agent info: {e}")))?;

        // Convert AgentInfo to JSON, then to Lua table
        let info_json = serde_json::to_value(&agent_info).map_err(|e| {
            mlua::Error::RuntimeError(format!("Failed to serialize agent info: {e}"))
        })?;
        let info_table = json_to_lua_value(lua, &info_json)?;
        match info_table {
            Value::Table(table) => Ok(table),
            _ => Err(mlua::Error::RuntimeError(
                "Invalid agent info format".to_string(),
            )),
        }
    })?;

    // Create Agent.list_capabilities() function
    let bridge_clone = bridge.clone();
    let list_capabilities_fn = lua.create_function(move |lua, ()| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let capabilities_json = block_on_async(
            "agent_listCapabilities",
            bridge.list_agent_capabilities(),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to list capabilities: {e}")))?;

        // Convert JSON to Lua table
        let capabilities_table = json_to_lua_value(lua, &capabilities_json)?;
        match capabilities_table {
            Value::Table(table) => Ok(table),
            _ => Err(mlua::Error::RuntimeError(
                "Invalid capabilities format".to_string(),
            )),
        }
    })?;

    // Create Agent.create_composite() function
    let bridge_clone = bridge.clone();
    let create_composite_fn = lua.create_function(move |_lua, args: (String, Table, Value)| {
        let (name, agent_list, routing_value) = args;
        let bridge = bridge_clone.clone();

        // Convert agents table to Vec<String>
        let mut agents = Vec::new();
        for pair in agent_list.pairs::<mlua::Integer, String>() {
            let (_, agent_name) = pair?;
            agents.push(agent_name);
        }

        // Parse routing config using typed parser
        let routing_config = parse_routing_config(&routing_value).map_err(|e| {
            mlua::Error::RuntimeError(format!("Failed to parse routing config: {e}"))
        })?;

        // Use sync wrapper to call async method
        block_on_async(
            "agent_create_composite",
            bridge.create_composite_agent(name, agents, routing_config),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create composite: {e}")))?;

        Ok(())
    })?;

    // Create Agent.discover_by_capability() function
    let bridge_clone = bridge.clone();
    let discover_by_capability_fn = lua.create_function(move |lua, capability: String| {
        let bridge = bridge_clone.clone();

        // Use sync wrapper to call async method
        let agents = block_on_async(
            "agent_discoverByCapability",
            bridge.discover_agents_by_capability(&capability),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to discover agents: {e}")))?;

        // Convert to Lua table
        let agent_results = lua.create_table()?;
        for (i, agent_name) in agents.into_iter().enumerate() {
            agent_results.set(i + 1, agent_name)?;
        }
        Ok(agent_results)
    })?;

    // Create Agent.register() function - alias for create
    let bridge_clone = bridge.clone();
    let register_fn = lua.create_function(move |_lua, args: Table| {
        let bridge = bridge_clone.clone();

        // Parse Lua table into typed AgentConfig struct
        let agent_config = parse_agent_config(&args)
            .map_err(|e| mlua::Error::RuntimeError(format!("Invalid agent configuration: {e}")))?;

        // Capture name for return value
        let name = agent_config.name.clone();

        // Use sync wrapper to call async method with typed config
        block_on_async("agent_register", bridge.create_agent(agent_config), None)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to register agent: {e}")))?;

        // Return the agent name
        Ok(name)
    })?;

    // Create Agent.get() function
    let bridge_clone = bridge.clone();
    let context_clone = Arc::new(context.clone());
    let get_fn = lua.create_function(move |_lua, agent_name: String| {
        let bridge = bridge_clone.clone();
        let name = agent_name.clone();
        let global_context = context_clone.clone();

        // Use sync wrapper to call async method
        let agent_exists = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(bridge.get_agent(&name))
        })
        .is_some();

        if agent_exists {
            // Create Lua agent instance
            let agent_instance = LuaAgentInstance {
                agent_instance_name: agent_name,
                bridge,
                global_context,
            };
            Ok(Some(agent_instance))
        } else {
            Ok(None)
        }
    })?;

    // Create Agent.list_templates() function
    let bridge_clone = bridge.clone();
    let list_templates_fn = lua.create_function(move |lua, ()| {
        let templates = bridge_clone.list_templates();
        let list_table = lua.create_table()?;
        for (i, template) in templates.into_iter().enumerate() {
            list_table.set(i + 1, template)?;
        }
        Ok(list_table)
    })?;

    // Create Agent.create_from_template() function
    let bridge_clone = bridge.clone();
    let context_clone2 = Arc::new(context.clone());
    let create_from_template_fn =
        lua.create_function(move |_lua, args: (String, String, Table)| {
            let (instance_name, template_name, params) = args;
            let bridge = bridge_clone.clone();
            let global_context = context_clone2.clone();

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
                mlua::Error::RuntimeError(format!("Failed to create from template: {e}"))
            })?;

            // Return the created agent instance
            let agent_instance = LuaAgentInstance {
                agent_instance_name: instance_name,
                bridge,
                global_context,
            };
            Ok(agent_instance)
        })?;

    // Create Agent.list_instances() function (alias for list)
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

    // Create Agent.create_context() function
    let bridge_clone = bridge.clone();
    let create_context_fn = lua.create_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();

        // Parse typed config
        let context_config = parse_execution_context_config(&config).map_err(|e| {
            mlua::Error::RuntimeError(format!("Failed to parse context config: {e}"))
        })?;

        let context_id = block_on_async(
            "agent_createContext",
            bridge.create_context(context_config),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create context: {e}")))?;

        Ok(context_id)
    })?;

    // Create Agent.create_child_context() function
    let bridge_clone = bridge.clone();
    let create_child_context_fn =
        lua.create_function(move |_lua, args: (String, Value, String)| {
            let (parent_id, scope_value, inheritance) = args;
            let bridge = bridge_clone.clone();

            // Parse typed config
            let child_config =
                parse_child_context_config(&scope_value, &inheritance).map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to parse child context config: {e}"))
                })?;

            let child_id = block_on_async(
                "agent_createChildContext",
                bridge.create_child_context(&parent_id, child_config),
                None,
            )
            .map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to create child context: {e}"))
            })?;

            Ok(child_id)
        })?;

    // Create Agent.update_context() function
    let bridge_clone = bridge.clone();
    let update_context_fn = lua.create_function(move |_lua, args: (String, String, Value)| {
        let (context_id, key, value) = args;
        let bridge = bridge_clone.clone();
        let value_json = crate::lua::conversion::lua_value_to_json(value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert value: {e}")))?;

        block_on_async(
            "agent_updateContext",
            bridge.update_context(&context_id, key, value_json),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to update context: {e}")))?;

        Ok(())
    })?;

    // Create Agent.get_context_data() function
    let bridge_clone = bridge.clone();
    let get_context_data_fn = lua.create_function(move |lua, args: (String, String)| {
        let (context_id, key) = args;
        let bridge = bridge_clone.clone();

        let result = block_on_async(
            "agent_getContextData",
            bridge.get_context_data(&context_id, &key),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get context data: {e}")))?;

        result.map_or_else(|| Ok(Value::Nil), |value| json_to_lua_value(lua, &value))
    })?;

    // Create Agent.remove_context() function
    let bridge_clone = bridge.clone();
    let remove_context_fn = lua.create_function(move |_lua, context_id: String| {
        let bridge = bridge_clone.clone();

        block_on_async(
            "agent_removeContext",
            bridge.remove_context(&context_id),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to remove context: {e}")))?;

        Ok(())
    })?;

    // Shared Memory Functions

    // Create Agent.set_shared_memory() function
    let bridge_clone = bridge.clone();
    let set_shared_memory_fn = lua.create_function(move |_lua, args: (Value, String, Value)| {
        let (scope_value, key, value) = args;
        let bridge = bridge_clone.clone();

        // Parse ContextScope using typed parser
        let scope = parse_context_scope(&scope_value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to parse scope: {e}")))?;

        let value_json = crate::lua::conversion::lua_value_to_json(value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert value: {e}")))?;

        bridge.set_shared_memory(&scope, key, value_json);
        Ok(())
    })?;

    // Create Agent.get_shared_memory() function
    let bridge_clone = bridge.clone();
    let get_shared_memory_fn = lua.create_function(move |lua, args: (Value, String)| {
        let (scope_value, key) = args;
        let bridge = bridge_clone.clone();

        // Parse ContextScope using typed parser
        let scope = parse_context_scope(&scope_value)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to parse scope: {e}")))?;

        let result = bridge.get_shared_memory(&scope, &key);
        result.map_or_else(|| Ok(Value::Nil), |value| json_to_lua_value(lua, &value))
    })?;

    // Create Agent.get_hierarchy() function
    let bridge_clone = bridge.clone();
    let get_hierarchy_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        let hierarchy = block_on_async(
            "agent_getHierarchy",
            bridge.get_composition_hierarchy(&agent_name),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get hierarchy: {e}")))?;

        json_to_lua_value(lua, &hierarchy)
    })?;

    // Create Agent.get_details() function (alias for get_info with different return format)
    let bridge_clone = bridge.clone();
    let get_details_fn = lua.create_function(move |lua, agent_name: String| {
        let bridge = bridge_clone.clone();

        let details = block_on_async(
            "agent_getDetails",
            bridge.get_agent_details(&agent_name),
            None,
        )
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to get agent details: {e}")))?;

        json_to_lua_value(lua, &details)
    })?;

    // Add builder() method
    let bridge_for_builder = bridge;
    let context_for_builder = Arc::new(context.clone());
    let builder_fn = lua.create_function(move |_lua, ()| {
        Ok(AgentBuilder::new(
            bridge_for_builder.clone(),
            context_for_builder.clone(),
        ))
    })?;
    agent_table.set("builder", builder_fn)?;

    // Replace create() with deprecation notice
    let create_deprecated_fn = lua.create_function(|_, _: Value| {
        Err::<Value, _>(mlua::Error::RuntimeError(
            "Agent.create() is deprecated. Use Agent.builder() instead:\n\
             local agent = Agent.builder()\n\
                 :name('my_agent')\n\
                 :model('openai/gpt-4')\n\
                 :temperature(0.7)\n\
                 :build()"
                .to_string(),
        ))
    })?;

    // Set functions on Agent table
    agent_table.set("create", create_deprecated_fn)?;
    agent_table.set("list", list_fn)?;
    agent_table.set("discover", discover_fn)?;
    agent_table.set("wrap_as_tool", wrap_as_tool_fn)?;
    agent_table.set("get_info", get_info_fn)?;
    agent_table.set("list_capabilities", list_capabilities_fn)?;
    agent_table.set("create_composite", create_composite_fn)?;
    agent_table.set("discover_by_capability", discover_by_capability_fn)?;
    agent_table.set("register", register_fn)?;
    agent_table.set("get", get_fn)?;
    agent_table.set("list_templates", list_templates_fn)?;
    agent_table.set("create_from_template", create_from_template_fn)?;
    agent_table.set("list_instances", list_instances_fn)?;
    agent_table.set("create_context", create_context_fn)?;
    agent_table.set("create_child_context", create_child_context_fn)?;
    agent_table.set("update_context", update_context_fn)?;
    agent_table.set("get_context_data", get_context_data_fn)?;
    agent_table.set("remove_context", remove_context_fn)?;
    agent_table.set("set_shared_memory", set_shared_memory_fn)?;
    agent_table.set("get_shared_memory", get_shared_memory_fn)?;
    agent_table.set("get_hierarchy", get_hierarchy_fn)?;
    agent_table.set("get_details", get_details_fn)?;

    // Set Agent as global
    lua.globals().set("Agent", agent_table)?;

    Ok(())
}
