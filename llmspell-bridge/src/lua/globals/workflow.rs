//! ABOUTME: Lua-specific Workflow global implementation
//! ABOUTME: Provides comprehensive Lua bindings for all workflow patterns

use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use crate::workflows::WorkflowBridge;
use llmspell_core::ComponentId;
use llmspell_workflows::{Condition, ErrorStrategy, StepType, WorkflowStep};
use mlua::{Lua, Table, UserData, UserDataMethods, Value};
use std::sync::Arc;
// use std::time::Duration; // Unused for now
use tracing::{debug, info};

/// Parse step configuration from Lua table
#[allow(dead_code)]
fn _parse_workflow_step(_lua: &Lua, step_table: Table) -> mlua::Result<WorkflowStep> {
    let name: String = step_table.get("name")?;
    let step_type: String = step_table.get("type")?;

    let step = match step_type.as_str() {
        "tool" => {
            let tool_name: String = step_table.get("tool")?;
            let input: Option<Table> = step_table.get("input").ok();

            let parameters = if let Some(input_table) = input {
                lua_value_to_json(Value::Table(input_table))?
            } else {
                serde_json::json!({})
            };

            WorkflowStep::new(
                name,
                StepType::Tool {
                    tool_name,
                    parameters,
                },
            )
        }
        "agent" => {
            let agent_id: String = step_table.get("agent")?;
            let input: String = step_table.get("input").unwrap_or_default();

            WorkflowStep::new(
                name,
                StepType::Agent {
                    agent_id: ComponentId::from_name(&agent_id),
                    input,
                },
            )
        }
        "workflow" => {
            // Workflow steps not supported yet in StepType
            return Err(mlua::Error::RuntimeError(
                "Workflow steps are not yet implemented".to_string(),
            ));
        }
        "custom" => {
            let function_name: String = step_table.get("function")?;
            let parameters: Option<Table> = step_table.get("parameters").ok();

            let params = if let Some(params_table) = parameters {
                lua_value_to_json(Value::Table(params_table))?
            } else {
                serde_json::json!({})
            };

            WorkflowStep::new(
                name,
                StepType::Custom {
                    function_name,
                    parameters: params,
                },
            )
        }
        _ => {
            return Err(mlua::Error::RuntimeError(format!(
                "Unknown step type: {}",
                step_type
            )))
        }
    };

    // Parse optional step configuration
    if let Ok(timeout_ms) = step_table.get::<_, u64>("timeout_ms") {
        debug!("Step timeout requested: {}ms", timeout_ms);
    }

    if let Ok(retry_count) = step_table.get::<_, u32>("retry_count") {
        debug!("Step retry count: {}", retry_count);
    }

    Ok(step)
}

/// Parse error strategy from string
#[allow(dead_code)]
fn _parse_error_strategy(strategy: &str) -> ErrorStrategy {
    match strategy.to_lowercase().as_str() {
        "fail_fast" | "failfast" => ErrorStrategy::FailFast,
        "continue" => ErrorStrategy::Continue,
        "retry" => ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 1000,
        },
        _ => ErrorStrategy::FailFast,
    }
}

/// Parse condition from Lua value
#[allow(dead_code)]
fn _parse_condition(_lua: &Lua, condition_value: Value) -> mlua::Result<Condition> {
    match condition_value {
        Value::String(s) => {
            let condition_str = s.to_str()?;
            match condition_str {
                "always" => Ok(Condition::Always),
                "never" => Ok(Condition::Never),
                _ => {
                    // Check if it's a step output reference
                    if condition_str.starts_with("step:") {
                        let parts: Vec<&str> = condition_str.splitn(3, ':').collect();
                        if parts.len() == 3 {
                            Ok(Condition::StepResultEquals {
                                step_id: ComponentId::from_name(parts[1]),
                                expected_output: parts[2].to_string(),
                            })
                        } else {
                            Err(mlua::Error::RuntimeError(
                                "Invalid step condition format".to_string(),
                            ))
                        }
                    } else {
                        Err(mlua::Error::RuntimeError(format!(
                            "Unknown condition: {}",
                            condition_str
                        )))
                    }
                }
            }
        }
        Value::Table(t) => {
            let condition_type: String = t.get("type")?;

            match condition_type.as_str() {
                "always" => Ok(Condition::Always),
                "never" => Ok(Condition::Never),
                "and" => {
                    let conditions: Table = t.get("conditions")?;
                    let mut and_conditions = Vec::new();

                    for pair in conditions.pairs::<i32, Value>() {
                        let (_, cond_value) = pair?;
                        and_conditions.push(_parse_condition(_lua, cond_value)?);
                    }

                    Ok(Condition::And {
                        conditions: and_conditions,
                    })
                }
                "or" => {
                    let conditions: Table = t.get("conditions")?;
                    let mut or_conditions = Vec::new();

                    for pair in conditions.pairs::<i32, Value>() {
                        let (_, cond_value) = pair?;
                        or_conditions.push(_parse_condition(_lua, cond_value)?);
                    }

                    Ok(Condition::Or {
                        conditions: or_conditions,
                    })
                }
                "not" => {
                    let inner: Value = t.get("condition")?;
                    Ok(Condition::Not {
                        condition: Box::new(_parse_condition(_lua, inner)?),
                    })
                }
                "step_output_equals" | "step_result_equals" => {
                    let step_name: String = t.get("step")?;
                    let expected: String = t.get("expected")?;

                    Ok(Condition::StepResultEquals {
                        step_id: ComponentId::from_name(&step_name),
                        expected_output: expected,
                    })
                }
                "shared_data_equals" => {
                    let key: String = t.get("key")?;
                    let expected: Value = t.get("expected")?;
                    let expected_json = lua_value_to_json(expected)?;

                    Ok(Condition::SharedDataEquals {
                        key,
                        expected_value: expected_json,
                    })
                }
                "custom" => {
                    let expression: String = t.get("expression")?;
                    let description: String = t.get("description").unwrap_or_default();

                    Ok(Condition::Custom {
                        expression,
                        description,
                    })
                }
                _ => Err(mlua::Error::RuntimeError(format!(
                    "Unknown condition type: {}",
                    condition_type
                ))),
            }
        }
        _ => Err(mlua::Error::RuntimeError(
            "Invalid condition value type".to_string(),
        )),
    }
}

/// Workflow instance that wraps WorkflowBridge
struct WorkflowInstance {
    bridge: Arc<WorkflowBridge>,
    workflow_id: String,
    name: String,
    workflow_type: String,
}

impl UserData for WorkflowInstance {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method
        methods.add_async_method("execute", |lua, this, input: Option<Table>| async move {
            let input_json = if let Some(input_table) = input {
                lua_value_to_json(Value::Table(input_table))?
            } else {
                serde_json::json!({})
            };

            info!("Executing {} workflow: {}", this.workflow_type, this.name);

            let result = this
                .bridge
                .execute_workflow(&this.workflow_id, input_json)
                .await
                .map_err(|e| {
                    mlua::Error::RuntimeError(format!("Workflow execution failed: {}", e))
                })?;

            // Convert result to Lua table
            let result_value = json_to_lua_value(lua, &result)?;

            match result_value {
                Value::Table(t) => Ok(t),
                _ => Err(mlua::Error::RuntimeError(
                    "Unexpected result type".to_string(),
                )),
            }
        });

        // getInfo method
        methods.add_method("getInfo", |lua, this, ()| {
            let info_table = lua.create_table()?;
            info_table.set("id", this.workflow_id.clone())?;
            info_table.set("name", this.name.clone())?;
            info_table.set("type", this.workflow_type.clone())?;

            Ok(info_table)
        });

        // getState method (integrates with State global)
        methods.add_method("getState", |lua, this, key: String| {
            // Access State global if available
            if let Ok(globals) = lua.globals().get::<_, Table>("State") {
                if let Ok(get_fn) = globals.get::<_, mlua::Function>("get") {
                    let state_key = format!("workflow:{}:{}", this.workflow_id, key);
                    return get_fn.call::<_, mlua::Value>(state_key);
                }
            }

            // Fallback if State global not available
            let state_table = lua.create_table()?;
            state_table.set("workflow_id", this.workflow_id.clone())?;
            state_table.set("key", key)?;
            state_table.set("value", mlua::Value::Nil)?;
            Ok(mlua::Value::Table(state_table))
        });

        // setState method (integrates with State global)
        methods.add_method(
            "setState",
            |lua, this, (key, value): (String, mlua::Value)| {
                // Access State global if available
                if let Ok(globals) = lua.globals().get::<_, Table>("State") {
                    if let Ok(set_fn) = globals.get::<_, mlua::Function>("set") {
                        let state_key = format!("workflow:{}:{}", this.workflow_id, key);
                        set_fn.call::<_, ()>((state_key, value))?;
                        return Ok(true);
                    }
                }

                // Return false if State global not available
                Ok(false)
            },
        );

        // onBeforeExecute method (Hook integration)
        methods.add_method("onBeforeExecute", |lua, this, _callback: mlua::Function| {
            // Store callback for future use in Phase 4
            // For now, just acknowledge the registration
            let result_table = lua.create_table()?;
            result_table.set(
                "message",
                "Hook registered (Phase 4 implementation pending)",
            )?;
            result_table.set("workflow_id", this.workflow_id.clone())?;
            result_table.set("hook_type", "before_execute")?;
            Ok(result_table)
        });

        // onAfterExecute method (Hook integration)
        methods.add_method("onAfterExecute", |lua, this, _callback: mlua::Function| {
            let result_table = lua.create_table()?;
            result_table.set(
                "message",
                "Hook registered (Phase 4 implementation pending)",
            )?;
            result_table.set("workflow_id", this.workflow_id.clone())?;
            result_table.set("hook_type", "after_execute")?;
            Ok(result_table)
        });

        // onError method (Hook integration)
        methods.add_method("onError", |lua, this, _callback: mlua::Function| {
            let result_table = lua.create_table()?;
            result_table.set(
                "message",
                "Hook registered (Phase 4 implementation pending)",
            )?;
            result_table.set("workflow_id", this.workflow_id.clone())?;
            result_table.set("hook_type", "on_error")?;
            Ok(result_table)
        });

        // emit method (Event integration)
        methods.add_method(
            "emit",
            |lua, this, (event_name, data): (String, Option<Table>)| {
                // Try to access Event global if available
                if let Ok(globals) = lua.globals().get::<_, Table>("Event") {
                    if let Ok(emit_fn) = globals.get::<_, mlua::Function>("emit") {
                        let event_data = lua.create_table()?;
                        event_data.set("workflow_id", this.workflow_id.clone())?;
                        event_data.set("workflow_name", this.name.clone())?;
                        event_data.set("workflow_type", this.workflow_type.clone())?;
                        if let Some(user_data) = data {
                            event_data.set("data", user_data)?;
                        }

                        emit_fn.call::<_, mlua::Value>((event_name.clone(), event_data))?;
                    }
                }

                // Return acknowledgment
                let result_table = lua.create_table()?;
                result_table.set("success", true)?;
                result_table.set("workflow_id", this.workflow_id.clone())?;
                result_table.set("event_name", event_name)?;
                Ok(result_table)
            },
        );

        // debug method - Get workflow debug information
        methods.add_method("debug", |lua, this, ()| {
            let debug_table = lua.create_table()?;
            debug_table.set("workflow_id", this.workflow_id.clone())?;
            debug_table.set("name", this.name.clone())?;
            debug_table.set("type", this.workflow_type.clone())?;

            // Add runtime info if available
            let runtime_info = lua.create_table()?;
            runtime_info.set("created_at", chrono::Utc::now().to_rfc3339())?;
            runtime_info.set("state_keys", lua.create_table()?)?; // Would list actual keys in full impl
            debug_table.set("runtime", runtime_info)?;

            Ok(debug_table)
        });

        // validate method - Validate workflow configuration
        methods.add_method("validate", |lua, this, ()| {
            let validation_result = lua.create_table()?;
            validation_result.set("valid", true)?;
            validation_result.set("workflow_id", this.workflow_id.clone())?;

            let warnings = lua.create_table()?;
            let errors = lua.create_table()?;

            // Basic validation checks (extend in full implementation)
            if this.name.is_empty() {
                errors.set(errors.len()? + 1, "Workflow name is empty")?;
                validation_result.set("valid", false)?;
            }

            validation_result.set("warnings", warnings)?;
            validation_result.set("errors", errors)?;

            Ok(validation_result)
        });

        // getMetrics method - Performance and execution metrics
        methods.add_method("getMetrics", |lua, this, ()| {
            let metrics_table = lua.create_table()?;
            metrics_table.set("workflow_id", this.workflow_id.clone())?;

            // Placeholder metrics - would be populated from actual execution
            let execution_metrics = lua.create_table()?;
            execution_metrics.set("total_executions", 0)?;
            execution_metrics.set("successful_executions", 0)?;
            execution_metrics.set("failed_executions", 0)?;
            execution_metrics.set("average_duration_ms", 0)?;
            execution_metrics.set("last_execution_time", mlua::Value::Nil)?;

            metrics_table.set("execution", execution_metrics)?;

            Ok(metrics_table)
        });
    }
}

/// Inject Workflow global into Lua environment
pub fn inject_workflow_global(
    lua: &Lua,
    _context: &GlobalContext,
    workflow_bridge: Arc<WorkflowBridge>,
) -> mlua::Result<()> {
    let workflow_table = lua.create_table()?;

    // Workflow.sequential() - accepts full configuration
    let bridge_clone = workflow_bridge.clone();
    let sequential_fn = lua.create_async_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();
        async move {
            let name: String = config.get("name")?;
            let description: Option<String> = config.get("description").ok();
            let steps: Table = config.get("steps")?;
            let error_strategy: Option<String> = config.get("error_strategy").ok();
            let timeout_ms: Option<u64> = config.get("timeout_ms").ok();

            // Convert to JSON parameters for WorkflowFactory
            let mut params = serde_json::json!({
                "name": name.clone(),
                "steps": []
            });

            if let Some(desc) = &description {
                params["description"] = serde_json::json!(desc);
            }

            if let Some(strategy) = &error_strategy {
                params["error_strategy"] = serde_json::json!(strategy);
            }

            if let Some(ms) = timeout_ms {
                params["timeout_ms"] = serde_json::json!(ms);
            }

            // Add steps to params
            let steps_array = params["steps"].as_array_mut().unwrap();
            for pair in steps.pairs::<i32, Table>() {
                let (_, step_table) = pair?;
                let step_json = lua_value_to_json(Value::Table(step_table))?;
                steps_array.push(step_json);
            }

            // Register with workflow bridge
            let workflow_id = bridge
                .create_workflow("sequential", params)
                .await
                .map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to register workflow: {}", e))
                })?;

            debug!(
                "Created and registered sequential workflow: {} ({})",
                name, workflow_id
            );

            Ok(WorkflowInstance {
                bridge: bridge.clone(),
                workflow_id,
                name,
                workflow_type: "sequential".to_string(),
            })
        }
    })?;

    // Workflow.conditional() - accepts branches and conditions
    let bridge_clone = workflow_bridge.clone();
    let conditional_fn = lua.create_async_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();
        async move {
            let name: String = config.get("name")?;
            let description: Option<String> = config.get("description").ok();
            let branches: Table = config.get("branches")?;
            let default_branch: Option<Table> = config.get("default_branch").ok();
            let error_strategy: Option<String> = config.get("error_strategy").ok();

            // Convert to JSON parameters for WorkflowFactory
            let mut params = serde_json::json!({
                "name": name.clone(),
                "branches": []
            });

            if let Some(desc) = &description {
                params["description"] = serde_json::json!(desc);
            }

            if let Some(strategy) = &error_strategy {
                params["error_strategy"] = serde_json::json!(strategy);
            }

            // Add branches to params
            let branches_array = params["branches"].as_array_mut().unwrap();
            for pair in branches.pairs::<i32, Table>() {
                let (_, branch_table) = pair?;
                let branch_json = lua_value_to_json(Value::Table(branch_table))?;
                branches_array.push(branch_json);
            }

            // Add default branch if provided
            if let Some(default_table) = default_branch {
                let default_json = lua_value_to_json(Value::Table(default_table))?;
                params["default_branch"] = default_json;
            }

            // Register with workflow bridge
            let workflow_id = bridge
                .create_workflow("conditional", params)
                .await
                .map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to register workflow: {}", e))
                })?;

            debug!(
                "Created and registered conditional workflow: {} ({})",
                name, workflow_id
            );

            Ok(WorkflowInstance {
                bridge: bridge.clone(),
                workflow_id,
                name,
                workflow_type: "conditional".to_string(),
            })
        }
    })?;

    // Workflow.loop() - accepts iterators and body
    let bridge_clone = workflow_bridge.clone();
    let loop_fn = lua.create_async_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();
        async move {
            let name: String = config.get("name")?;
            let description: Option<String> = config.get("description").ok();
            let iterator_type: String = config.get("iterator")?;
            let body: Table = config.get("body")?;
            let break_conditions: Option<Table> = config.get("break_conditions").ok();
            let error_strategy: Option<String> = config.get("error_strategy").ok();

            // Convert to JSON parameters for WorkflowFactory
            let mut params = serde_json::json!({
                "name": name.clone(),
                "body": []
            });

            if let Some(desc) = &description {
                params["description"] = serde_json::json!(desc);
            }

            if let Some(strategy) = &error_strategy {
                params["error_strategy"] = serde_json::json!(strategy);
            }

            // Add iterator-specific parameters
            // The bridge expects iterator to be an object with specific structure
            let iterator_obj = match iterator_type.as_str() {
                "range" => {
                    serde_json::json!({
                        "range": {
                            "start": config.get::<_, i32>("start").unwrap_or(0),
                            "end": config.get::<_, i32>("end")?,
                            "step": config.get::<_, i32>("step").unwrap_or(1)
                        }
                    })
                }
                "collection" => {
                    let items: Table = config.get("items")?;
                    let mut collection = Vec::new();

                    for pair in items.pairs::<i32, Value>() {
                        let (_, value) = pair?;
                        collection.push(lua_value_to_json(value)?);
                    }

                    serde_json::json!({
                        "collection": collection
                    })
                }
                "while" => {
                    let condition_value: Value = config.get("condition")?;
                    let condition_str = match condition_value {
                        Value::String(s) => s.to_str()?.to_string(),
                        _ => {
                            return Err(mlua::Error::RuntimeError(
                                "while condition must be a string".to_string(),
                            ))
                        }
                    };
                    let max_iterations = config.get::<_, u64>("max_iterations").unwrap_or(100);

                    serde_json::json!({
                        "while_condition": condition_str,
                        "max_iterations": max_iterations
                    })
                }
                _ => {
                    return Err(mlua::Error::RuntimeError(format!(
                        "Unknown iterator type: {}",
                        iterator_type
                    )))
                }
            };

            params["iterator"] = iterator_obj;

            // Add body steps
            let body_array = params["body"].as_array_mut().unwrap();
            for pair in body.pairs::<i32, Table>() {
                let (_, step_table) = pair?;
                let step_json = lua_value_to_json(Value::Table(step_table))?;
                body_array.push(step_json);
            }

            // Add break conditions
            if let Some(conditions_table) = break_conditions {
                let mut break_conditions_array = Vec::new();
                for pair in conditions_table.pairs::<i32, Value>() {
                    let (_, condition_value) = pair?;
                    let condition_json = lua_value_to_json(condition_value)?;
                    break_conditions_array.push(condition_json);
                }
                params["break_conditions"] = serde_json::json!(break_conditions_array);
            }

            // Register with workflow bridge
            let workflow_id = bridge.create_workflow("loop", params).await.map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to register workflow: {}", e))
            })?;

            debug!(
                "Created and registered loop workflow: {} ({})",
                name, workflow_id
            );

            Ok(WorkflowInstance {
                bridge: bridge.clone(),
                workflow_id,
                name,
                workflow_type: "loop".to_string(),
            })
        }
    })?;

    // Workflow.parallel() - accepts branches configuration
    let bridge_clone = workflow_bridge.clone();
    let parallel_fn = lua.create_async_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();
        async move {
            let name: String = config.get("name")?;
            let description: Option<String> = config.get("description").ok();
            let branches: Table = config.get("branches")?;
            let max_concurrency: Option<usize> = config.get("max_concurrency").ok();
            let error_strategy: Option<String> = config.get("error_strategy").ok();
            let timeout_ms: Option<u64> = config.get("timeout_ms").ok();

            // Convert to JSON parameters for WorkflowFactory
            let mut params = serde_json::json!({
                "name": name.clone(),
                "branches": []
            });

            if let Some(desc) = &description {
                params["description"] = serde_json::json!(desc);
            }

            if let Some(strategy) = &error_strategy {
                params["error_strategy"] = serde_json::json!(strategy);
            }

            if let Some(max) = max_concurrency {
                params["max_concurrency"] = serde_json::json!(max);
            }

            if let Some(ms) = timeout_ms {
                params["timeout_ms"] = serde_json::json!(ms);
            }

            // Add branches to params
            let branches_array = params["branches"].as_array_mut().unwrap();
            for pair in branches.pairs::<i32, Table>() {
                let (_, branch_table) = pair?;
                let branch_json = lua_value_to_json(Value::Table(branch_table))?;
                branches_array.push(branch_json);
            }

            // Register with workflow bridge
            let workflow_id = bridge
                .create_workflow("parallel", params)
                .await
                .map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to register workflow: {}", e))
                })?;

            debug!(
                "Created and registered parallel workflow: {} ({})",
                name, workflow_id
            );

            Ok(WorkflowInstance {
                bridge: bridge.clone(),
                workflow_id,
                name,
                workflow_type: "parallel".to_string(),
            })
        }
    })?;

    // Set workflow constructors on table
    workflow_table.set("sequential", sequential_fn)?;
    workflow_table.set("conditional", conditional_fn)?;
    workflow_table.set("loop", loop_fn)?;
    workflow_table.set("parallel", parallel_fn)?;

    // Add workflow discovery methods
    workflow_table.set(
        "types",
        lua.create_function(|lua, ()| {
            let types = lua.create_table()?;
            types.set(1, "sequential")?;
            types.set(2, "conditional")?;
            types.set(3, "loop")?;
            types.set(4, "parallel")?;
            Ok(types)
        })?,
    )?;

    // Add error handling utilities
    workflow_table.set(
        "setDefaultErrorHandler",
        lua.create_function(|lua, handler: mlua::Function| {
            // Store in registry for future use
            lua.set_named_registry_value("workflow_default_error_handler", handler)?;
            Ok(())
        })?,
    )?;

    // Add debugging utilities
    workflow_table.set(
        "enableDebug",
        lua.create_function(|lua, enabled: bool| {
            lua.set_named_registry_value("workflow_debug_enabled", enabled)?;
            Ok(())
        })?,
    )?;

    workflow_table.set(
        "isDebugEnabled",
        lua.create_function(|lua, ()| {
            let enabled = lua
                .named_registry_value::<bool>("workflow_debug_enabled")
                .unwrap_or(false);
            Ok(enabled)
        })?,
    )?;

    // Add registry methods

    // Workflow.list() - list all registered workflows
    let bridge_clone = workflow_bridge.clone();
    workflow_table.set(
        "list",
        lua.create_async_function(move |lua, ()| {
            let bridge = bridge_clone.clone();
            async move {
                let workflows = bridge.list_workflows().await;

                let list_table = lua.create_table()?;
                for (i, (id, info)) in workflows.iter().enumerate() {
                    let workflow_table = lua.create_table()?;
                    workflow_table.set("id", id.clone())?;
                    workflow_table.set("type", info.workflow_type.clone())?;
                    workflow_table.set("description", info.description.clone())?;
                    workflow_table.set("features", info.features.clone())?;
                    list_table.set(i + 1, workflow_table)?;
                }

                Ok(list_table)
            }
        })?,
    )?;

    // Workflow.get() - get a specific workflow by ID
    let _bridge_clone = workflow_bridge.clone();
    workflow_table.set(
        "get",
        lua.create_function({
            let bridge_clone = workflow_bridge.clone();
            move |_, workflow_id: String| {
                let bridge = bridge_clone.clone();
                let wf_id = workflow_id.clone();
                let info = tokio::task::block_in_place(move || {
                    tokio::runtime::Handle::current().block_on(async move {
                        let workflows = bridge.list_workflows().await;
                        workflows
                            .into_iter()
                            .find(|(id, _)| id == &wf_id)
                            .map(|(_, info)| info)
                    })
                });

                if let Some(workflow_info) = info {
                    Ok(WorkflowInstance {
                        bridge: bridge_clone.clone(),
                        workflow_id: workflow_id.clone(),
                        name: workflow_id.clone(), // Use workflow_id as name for now
                        workflow_type: workflow_info.workflow_type,
                    })
                } else {
                    Err(mlua::Error::RuntimeError("Workflow not found".to_string()))
                }
            }
        })?,
    )?;

    // Workflow.remove() - remove a workflow
    let bridge_clone = workflow_bridge.clone();
    workflow_table.set(
        "remove",
        lua.create_async_function(move |_, workflow_id: String| {
            let bridge = bridge_clone.clone();
            async move {
                bridge.remove_workflow(&workflow_id).await.map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to remove workflow: {}", e))
                })?;
                Ok(())
            }
        })?,
    )?;

    // Workflow.register() - register a workflow (alias for create_workflow)
    let bridge_clone = workflow_bridge.clone();
    workflow_table.set(
        "register",
        lua.create_function(move |_lua, (workflow_type, params): (String, Table)| {
            let bridge = bridge_clone.clone();

            // Convert Lua table to JSON
            let params_json = lua_value_to_json(Value::Table(params)).map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to convert params: {}", e))
            })?;

            // Use sync wrapper to call async method
            let workflow_id = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(bridge.create_workflow(&workflow_type, params_json))
            })
            .map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to register workflow: {}", e))
            })?;

            Ok(workflow_id)
        })?,
    )?;

    // Workflow.clear() - remove all workflows
    let bridge_clone = workflow_bridge.clone();
    workflow_table.set(
        "clear",
        lua.create_function(move |_lua, ()| {
            let bridge = bridge_clone.clone();

            // Get all workflow IDs first
            let workflows = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(bridge.list_workflows())
            });

            // Remove each workflow
            for (workflow_id, _) in workflows {
                let _ = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(bridge.remove_workflow(&workflow_id))
                });
            }

            Ok(())
        })?,
    )?;

    // Set Workflow as global
    lua.globals().set("Workflow", workflow_table)?;

    Ok(())
}
