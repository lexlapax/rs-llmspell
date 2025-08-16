//! ABOUTME: Lua-specific Workflow global implementation
//! ABOUTME: Provides comprehensive Lua bindings for all workflow patterns

#![allow(clippy::significant_drop_tightening)]

use crate::globals::GlobalContext;
use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use crate::lua::sync_utils::block_on_async;
use crate::workflows::{WorkflowBridge, WorkflowInfo};
use llmspell_core::{ComponentId, LLMSpellError};
use llmspell_workflows::{Condition, ErrorStrategy, StepType, WorkflowStep};
use mlua::{AnyUserDataExt, Lua, Table, UserData, UserDataMethods, Value};
use std::sync::Arc;
// use std::time::Duration; // Unused for now
use tracing::{debug, info};
use uuid;

/// Parse step configuration from Lua table
fn parse_workflow_step(_lua: &Lua, step_table: &Table) -> mlua::Result<WorkflowStep> {
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
            let workflow: mlua::AnyUserData = step_table.get("workflow")?;

            // Get workflow info from the workflow userdata
            let workflow_info: Table = workflow.call_method("get_info", ())?;
            let workflow_id: String = workflow_info.get("id")?;

            let input: Option<Table> = step_table.get("input").ok();
            let input_value = if let Some(input_table) = input {
                lua_value_to_json(Value::Table(input_table))?
            } else {
                serde_json::json!({})
            };

            WorkflowStep::new(
                name,
                StepType::Workflow {
                    workflow_id: ComponentId::from_name(&workflow_id),
                    input: input_value,
                },
            )
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
                "Unknown step type: {step_type}"
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
#[allow(clippy::only_used_in_recursion)]
fn parse_condition(lua: &Lua, condition_value: Value) -> mlua::Result<Condition> {
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
                            "Unknown condition: {condition_str}"
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
                        and_conditions.push(parse_condition(lua, cond_value)?);
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
                        or_conditions.push(parse_condition(lua, cond_value)?);
                    }

                    Ok(Condition::Or {
                        conditions: or_conditions,
                    })
                }
                "not" => {
                    let inner: Value = t.get("condition")?;
                    Ok(Condition::Not {
                        condition: Box::new(parse_condition(lua, inner)?),
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
                    "Unknown condition type: {condition_type}"
                ))),
            }
        }
        _ => Err(mlua::Error::RuntimeError(
            "Invalid condition value type".to_string(),
        )),
    }
}

/// Workflow instance that wraps `WorkflowBridge`
struct WorkflowInstance {
    bridge: Arc<WorkflowBridge>,
    workflow_id: String,
    name: String,
    workflow_type: String,
}

impl UserData for WorkflowInstance {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("type", |_, this| Ok(this.workflow_type.clone()));
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("id", |_, this| Ok(this.workflow_id.clone()));
    }

    #[allow(clippy::too_many_lines)]
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method
        methods.add_method("execute", |lua, this, input: Option<Table>| {
            let input_json = if let Some(input_table) = input {
                lua_value_to_json(Value::Table(input_table))?
            } else {
                serde_json::json!({})
            };

            info!("Executing {} workflow: {}", this.workflow_type, this.name);

            // Use shared sync utility for async operation
            let result = block_on_async::<_, serde_json::Value, LLMSpellError>(
                &format!("workflow_execute_{}", this.workflow_id),
                async move {
                    this.bridge
                        .execute_workflow(&this.workflow_id, input_json)
                        .await
                },
                None,
            )?;

            // Convert result to Lua table
            let result_value = json_to_lua_value(lua, &result)?;

            match result_value {
                Value::Table(t) => Ok(t),
                _ => Err(mlua::Error::RuntimeError(
                    "Unexpected result type".to_string(),
                )),
            }
        });

        // get_info method
        methods.add_method("get_info", |lua, this, ()| {
            let info_table = lua.create_table()?;
            info_table.set("id", this.workflow_id.clone())?;
            info_table.set("name", this.name.clone())?;
            info_table.set("type", this.workflow_type.clone())?;

            Ok(info_table)
        });

        // get_state method (integrates with State global)
        methods.add_method("get_state", |lua, this, key: String| {
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

        // set_state method (integrates with State global)
        methods.add_method(
            "set_state",
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

        // on_before_execute method (Hook integration)
        methods.add_method(
            "on_before_execute",
            |lua, this, _callback: mlua::Function| {
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
            },
        );

        // on_after_execute method (Hook integration)
        methods.add_method(
            "on_after_execute",
            |lua, this, _callback: mlua::Function| {
                let result_table = lua.create_table()?;
                result_table.set(
                    "message",
                    "Hook registered (Phase 4 implementation pending)",
                )?;
                result_table.set("workflow_id", this.workflow_id.clone())?;
                result_table.set("hook_type", "after_execute")?;
                Ok(result_table)
            },
        );

        // on_error method (Hook integration)
        methods.add_method("on_error", |lua, this, _callback: mlua::Function| {
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

        // get_metrics method - Performance and execution metrics
        methods.add_method("get_metrics", |lua, this, ()| {
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

/// Type alias for workflow condition functions
type WorkflowCondition = Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>;

/// `WorkflowBuilder` for creating workflows with method chaining
struct WorkflowBuilder {
    bridge: Arc<WorkflowBridge>,
    workflow_type: Option<String>,
    name: Option<String>,
    description: Option<String>,
    steps: Vec<WorkflowStep>,
    error_strategy: Option<String>,
    timeout_ms: Option<u64>,
    // Conditional workflow fields
    condition: Option<WorkflowCondition>,
    then_steps: Vec<WorkflowStep>,
    else_steps: Vec<WorkflowStep>,
    // Loop workflow fields
    loop_condition: Option<WorkflowCondition>,
    max_iterations: Option<usize>,
    // Parallel workflow fields
    max_concurrency: Option<usize>,
}

impl WorkflowBuilder {
    fn new(bridge: Arc<WorkflowBridge>) -> Self {
        Self {
            bridge,
            workflow_type: None,
            name: None,
            description: None,
            steps: Vec::new(),
            error_strategy: None,
            timeout_ms: None,
            condition: None,
            then_steps: Vec::new(),
            else_steps: Vec::new(),
            loop_condition: None,
            max_iterations: None,
            max_concurrency: None,
        }
    }
}

#[allow(clippy::too_many_lines)]
impl UserData for WorkflowBuilder {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Set workflow type
        methods.add_method_mut("sequential", |_, this, ()| {
            this.workflow_type = Some("sequential".to_string());
            Ok(this.clone())
        });

        methods.add_method_mut("conditional", |_, this, ()| {
            this.workflow_type = Some("conditional".to_string());
            Ok(this.clone())
        });

        methods.add_method_mut("loop_workflow", |_, this, ()| {
            this.workflow_type = Some("loop".to_string());
            Ok(this.clone())
        });

        methods.add_method_mut("parallel", |_, this, ()| {
            this.workflow_type = Some("parallel".to_string());
            Ok(this.clone())
        });

        // Common configuration methods
        methods.add_method_mut("name", |_, this, name: String| {
            this.name = Some(name);
            Ok(this.clone())
        });

        methods.add_method_mut("description", |_, this, desc: String| {
            this.description = Some(desc);
            Ok(this.clone())
        });

        methods.add_method_mut("error_strategy", |_, this, strategy: String| {
            this.error_strategy = Some(strategy);
            Ok(this.clone())
        });

        methods.add_method_mut("timeout_ms", |_, this, timeout: u64| {
            this.timeout_ms = Some(timeout);
            Ok(this.clone())
        });

        // Add step (for sequential, loop, and parallel workflows)
        methods.add_method_mut("add_step", |lua, this, step_table: Table| {
            let step = parse_workflow_step(lua, &step_table)?;
            this.steps.push(step);
            Ok(this.clone())
        });

        // Conditional workflow specific methods
        methods.add_method_mut("condition", |_lua, this, _func: mlua::Function| {
            // Store Lua function for condition evaluation
            // Note: This is a simplified version - in production, you'd need to handle
            // Lua function storage and evaluation properly
            this.condition = Some(Box::new(move |_value| {
                // In a real implementation, this would evaluate the Lua function
                true
            }));
            Ok(this.clone())
        });

        methods.add_method_mut("add_then_step", |lua, this, step_table: Table| {
            let step = parse_workflow_step(lua, &step_table)?;
            this.then_steps.push(step);
            Ok(this.clone())
        });

        methods.add_method_mut("add_else_step", |lua, this, step_table: Table| {
            let step = parse_workflow_step(lua, &step_table)?;
            this.else_steps.push(step);
            Ok(this.clone())
        });

        // Loop workflow specific methods
        methods.add_method_mut("loop_condition", |_lua, this, _func: mlua::Function| {
            // Store Lua function for loop condition
            this.loop_condition = Some(Box::new(move |_value| {
                // In a real implementation, this would evaluate the Lua function
                true
            }));
            Ok(this.clone())
        });

        methods.add_method_mut("max_iterations", |_, this, max: usize| {
            this.max_iterations = Some(max);
            Ok(this.clone())
        });

        // Parallel workflow specific methods
        methods.add_method_mut("max_concurrency", |_, this, max: usize| {
            this.max_concurrency = Some(max);
            Ok(this.clone())
        });

        // Build method
        methods.add_method("build", |_lua, this, ()| {
            let workflow_type = this.workflow_type.as_ref().ok_or_else(|| {
                mlua::Error::RuntimeError("Workflow type not specified".to_string())
            })?;

            let name = this.name.clone().unwrap_or_else(|| {
                format!(
                    "workflow_{}",
                    uuid::Uuid::new_v4()
                        .to_string()
                        .chars()
                        .take(8)
                        .collect::<String>()
                )
            });

            // Create configuration based on workflow type
            let mut config = serde_json::json!({
                "name": &name,
                "description": this.description.as_deref().unwrap_or("Workflow created via builder")
            });

            if let Some(strategy) = &this.error_strategy {
                config["error_strategy"] = serde_json::json!(strategy);
            }

            if let Some(timeout) = this.timeout_ms {
                config["timeout_ms"] = serde_json::json!(timeout);
            }

            // Convert steps to JSON matching WorkflowStep structure
            let steps_json: Vec<serde_json::Value> = this
                .steps
                .iter()
                .map(|step| {
                    // Generate a unique ID for each step (just the UUID without prefix)
                    let step_id = uuid::Uuid::new_v4().to_string();

                    // Build the step_type field based on the step type
                    let step_type = match &step.step_type {
                        StepType::Tool {
                            tool_name,
                            parameters,
                        } => {
                            serde_json::json!({
                                "Tool": {
                                    "tool_name": tool_name,
                                    "parameters": parameters
                                }
                            })
                        }
                        StepType::Agent { agent_id, input } => {
                            serde_json::json!({
                                "Agent": {
                                    "agent_id": agent_id.to_string(),
                                    "input": input
                                }
                            })
                        }
                        StepType::Custom {
                            function_name,
                            parameters,
                        } => {
                            serde_json::json!({
                                "Custom": {
                                    "function_name": function_name,
                                    "parameters": parameters
                                }
                            })
                        }
                        StepType::Workflow { workflow_id, input } => {
                            serde_json::json!({
                                "Workflow": {
                                    "workflow_id": workflow_id.to_string(),
                                    "input": input
                                }
                            })
                        }
                    };

                    // Build the complete WorkflowStep JSON
                    serde_json::json!({
                        "id": step_id,
                        "name": &step.name,
                        "step_type": step_type,
                        "timeout": null,
                        "retry_attempts": 0
                    })
                })
                .collect();

            config["steps"] = serde_json::json!(steps_json);

            // Add type-specific configuration
            match workflow_type.as_str() {
                "conditional" => {
                    // For conditional workflows, add then/else steps
                    let then_steps_json: Vec<serde_json::Value> = this
                        .then_steps
                        .iter()
                        .map(|step| {
                            serde_json::json!({
                                "name": &step.name,
                                "type": "tool", // Simplified for now
                                "tool": "placeholder"
                            })
                        })
                        .collect();

                    let else_steps_json: Vec<serde_json::Value> = this
                        .else_steps
                        .iter()
                        .map(|step| {
                            serde_json::json!({
                                "name": &step.name,
                                "type": "tool", // Simplified for now
                                "tool": "placeholder"
                            })
                        })
                        .collect();

                    config["then_steps"] = serde_json::json!(then_steps_json);
                    config["else_steps"] = serde_json::json!(else_steps_json);
                }
                "loop" => {
                    if let Some(max_iter) = this.max_iterations {
                        config["max_iterations"] = serde_json::json!(max_iter);
                    }
                }
                "parallel" => {
                    if let Some(max_conc) = this.max_concurrency {
                        config["max_concurrency"] = serde_json::json!(max_conc);
                    }
                }
                _ => {}
            }

            // Create workflow using bridge
            let bridge = this.bridge.clone();
            let workflow_name = name;

            let result = block_on_async(
                "workflow_builder_create",
                async move {
                    let workflow_name_clone = workflow_name.clone();
                    let workflow_type_clone = workflow_type.clone();
                    bridge
                        .create_workflow(workflow_type, config)
                        .await
                        .map(|workflow_id| WorkflowInstance {
                            workflow_id,
                            bridge: bridge.clone(),
                            name: workflow_name_clone,
                            workflow_type: workflow_type_clone,
                        })
                },
                None,
            )?;

            Ok(result)
        });
    }
}

// Implement Clone for WorkflowBuilder
impl Clone for WorkflowBuilder {
    fn clone(&self) -> Self {
        Self {
            bridge: self.bridge.clone(),
            workflow_type: self.workflow_type.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            steps: self.steps.clone(),
            error_strategy: self.error_strategy.clone(),
            timeout_ms: self.timeout_ms,
            condition: None, // Can't clone closures, will be set fresh
            then_steps: self.then_steps.clone(),
            else_steps: self.else_steps.clone(),
            loop_condition: None, // Can't clone closures, will be set fresh
            max_iterations: self.max_iterations,
            max_concurrency: self.max_concurrency,
        }
    }
}

/// Inject Workflow global into Lua environment
///
/// # Errors
///
/// Returns an error if Lua table creation or function binding fails
///
/// # Panics
///
/// Panics if the Lua table creation or function binding fails
#[allow(clippy::too_many_lines)]
pub fn inject_workflow_global(
    lua: &Lua,
    _context: &GlobalContext,
    workflow_bridge: Arc<WorkflowBridge>,
) -> mlua::Result<()> {
    let workflow_table = lua.create_table()?;

    // Workflow.sequential() - accepts full configuration
    let bridge_clone = workflow_bridge.clone();
    let sequential_fn = lua.create_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();

        // Use shared sync utility for async operation
        let result = block_on_async::<_, WorkflowInstance, LLMSpellError>(
            "workflow_create_sequential",
            async move {
                let name: String = config.get("name").map_err(|e| LLMSpellError::Script {
                    message: format!("Failed to get workflow name: {e}"),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })?;
                let description: Option<String> = config.get("description").ok();
                let steps: Table = config.get("steps").map_err(|e| LLMSpellError::Script {
                    message: format!("Failed to get workflow steps: {e}"),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })?;
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
                    let (_, step_table) = pair.map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to iterate workflow steps: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    })?;
                    let step_json = lua_value_to_json(Value::Table(step_table)).map_err(|e| {
                        LLMSpellError::Script {
                            message: format!("Failed to convert step to JSON: {e}"),
                            language: Some("lua".to_string()),
                            line: None,
                            source: None,
                        }
                    })?;
                    steps_array.push(step_json);
                }

                // Register with workflow bridge
                let workflow_id = bridge.create_workflow("sequential", params).await?;

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
            },
            None,
        )?;

        Ok(result)
    })?;

    // Workflow.conditional() - accepts branches and conditions
    let bridge_clone = workflow_bridge.clone();
    let conditional_fn = lua.create_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();

        // Use shared sync utility for async operation
        let result = block_on_async::<_, WorkflowInstance, LLMSpellError>(
            "workflow_create_conditional",
            async move {
                let name: String = config.get("name").map_err(|e| LLMSpellError::Script {
                    message: format!("Failed to get workflow name: {e}"),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })?;
                let description: Option<String> = config.get("description").ok();
                let branches: Table =
                    config.get("branches").map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to get workflow branches: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    })?;
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
                    let (_, branch_table) = pair.map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to iterate workflow branches: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    })?;
                    let branch_json =
                        lua_value_to_json(Value::Table(branch_table)).map_err(|e| {
                            LLMSpellError::Script {
                                message: format!("Failed to convert branch to JSON: {e}"),
                                language: Some("lua".to_string()),
                                line: None,
                                source: None,
                            }
                        })?;
                    branches_array.push(branch_json);
                }

                // Add default branch if provided
                if let Some(default_table) = default_branch {
                    let default_json =
                        lua_value_to_json(Value::Table(default_table)).map_err(|e| {
                            LLMSpellError::Script {
                                message: format!("Failed to convert default branch to JSON: {e}"),
                                language: Some("lua".to_string()),
                                line: None,
                                source: None,
                            }
                        })?;
                    params["default_branch"] = default_json;
                }

                // Register with workflow bridge
                let workflow_id = bridge.create_workflow("conditional", params).await?;

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
            },
            None,
        )?;

        Ok(result)
    })?;

    // Workflow.loop() - accepts iterators and body
    let bridge_clone = workflow_bridge.clone();
    let loop_fn = lua.create_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();

        // Use shared sync utility for async operation
        let result = block_on_async::<_, WorkflowInstance, LLMSpellError>(
            "workflow_create_loop",
            async move {
                let name: String = config.get("name").map_err(|e| LLMSpellError::Script {
                    message: format!("Failed to get workflow name: {e}"),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })?;
                let description: Option<String> = config.get("description").ok();
                let iterator_table: Table =
                    config.get("iterator").map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to get iterator configuration: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    })?;
                let body: Table = config.get("body").map_err(|e| LLMSpellError::Script {
                    message: format!("Failed to get loop body: {e}"),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })?;
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

                // Parse iterator configuration from the table
                let iterator_obj = if let Ok(range) = iterator_table.get::<_, Table>("range") {
                    // Range iterator
                    let start = range.get::<_, i32>("start").unwrap_or(0);
                    let end = range
                        .get::<_, i32>("end")
                        .map_err(|e| LLMSpellError::Script {
                            message: format!("Failed to get range end: {e}"),
                            language: Some("lua".to_string()),
                            line: None,
                            source: None,
                        })?;
                    let step = range.get::<_, i32>("step").unwrap_or(1);

                    serde_json::json!({
                        "type": "range",
                        "start": start,
                        "end": end,
                        "step": step
                    })
                } else if let Ok(collection) = iterator_table.get::<_, Table>("collection") {
                    // Collection iterator
                    let mut collection_vec = Vec::new();
                    for pair in collection.pairs::<i32, Value>() {
                        let (_, value) = pair.map_err(|e| LLMSpellError::Script {
                            message: format!("Failed to iterate collection: {e}"),
                            language: Some("lua".to_string()),
                            line: None,
                            source: None,
                        })?;
                        collection_vec.push(lua_value_to_json(value).map_err(|e| {
                            LLMSpellError::Script {
                                message: format!("Failed to convert collection item to JSON: {e}"),
                                language: Some("lua".to_string()),
                                line: None,
                                source: None,
                            }
                        })?);
                    }

                    serde_json::json!({
                        "type": "collection",
                        "items": collection_vec
                    })
                } else if let Ok(condition_str) = iterator_table.get::<_, String>("while_condition")
                {
                    // While iterator
                    let max_iterations = iterator_table
                        .get::<_, u64>("max_iterations")
                        .unwrap_or(100);

                    serde_json::json!({
                        "type": "while",
                        "condition": condition_str,
                        "max_iterations": max_iterations
                    })
                } else {
                    return Err(LLMSpellError::Script {
                        message:
                            "Iterator must contain 'range', 'collection', or 'while_condition'"
                                .to_string(),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    });
                };

                params["iterator"] = iterator_obj;

                // Add body steps
                let body_array = params["body"].as_array_mut().unwrap();
                for pair in body.pairs::<i32, Table>() {
                    let (_, step_table) = pair.map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to iterate body steps: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    })?;
                    let step_json = lua_value_to_json(Value::Table(step_table)).map_err(|e| {
                        LLMSpellError::Script {
                            message: format!("Failed to convert body step to JSON: {e}"),
                            language: Some("lua".to_string()),
                            line: None,
                            source: None,
                        }
                    })?;
                    body_array.push(step_json);
                }

                // Add break conditions
                if let Some(conditions_table) = break_conditions {
                    let mut break_conditions_array = Vec::new();
                    for pair in conditions_table.pairs::<i32, Value>() {
                        let (_, condition_value) = pair.map_err(|e| LLMSpellError::Script {
                            message: format!("Failed to iterate break conditions: {e}"),
                            language: Some("lua".to_string()),
                            line: None,
                            source: None,
                        })?;
                        let condition_json = lua_value_to_json(condition_value).map_err(|e| {
                            LLMSpellError::Script {
                                message: format!("Failed to convert break condition to JSON: {e}"),
                                language: Some("lua".to_string()),
                                line: None,
                                source: None,
                            }
                        })?;
                        break_conditions_array.push(condition_json);
                    }
                    params["break_conditions"] = serde_json::json!(break_conditions_array);
                }

                // Register with workflow bridge
                let workflow_id = bridge.create_workflow("loop", params).await?;

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
            },
            None,
        )?;

        Ok(result)
    })?;

    // Workflow.parallel() - accepts branches configuration
    let bridge_clone = workflow_bridge.clone();
    let parallel_fn = lua.create_function(move |_lua, config: Table| {
        let bridge = bridge_clone.clone();

        // Use shared sync utility for async operation
        let result = block_on_async::<_, WorkflowInstance, LLMSpellError>(
            "workflow_create_parallel",
            async move {
                let name: String = config.get("name").map_err(|e| LLMSpellError::Script {
                    message: format!("Failed to get workflow name: {e}"),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })?;
                let description: Option<String> = config.get("description").ok();

                // Accept both "branches" and "steps" for API consistency
                let branches: Table = if let Ok(branches) = config.get::<&str, Table>("branches") {
                    branches
                } else if let Ok(steps) = config.get::<&str, Table>("steps") {
                    steps
                } else {
                    return Err(LLMSpellError::Script {
                        message: "Parallel workflow requires either 'branches' or 'steps' field"
                            .to_string(),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    });
                };
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
                    let (_, branch_table) = pair.map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to iterate workflow branches: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: None,
                    })?;
                    let branch_json =
                        lua_value_to_json(Value::Table(branch_table)).map_err(|e| {
                            LLMSpellError::Script {
                                message: format!("Failed to convert branch to JSON: {e}"),
                                language: Some("lua".to_string()),
                                line: None,
                                source: None,
                            }
                        })?;
                    branches_array.push(branch_json);
                }

                // Register with workflow bridge
                let workflow_id = bridge.create_workflow("parallel", params).await?;

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
            },
            None,
        )?;

        Ok(result)
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
        lua.create_function(move |lua, ()| {
            let bridge = bridge_clone.clone();

            // Use shared sync utility for async operation
            let workflows = block_on_async::<_, Vec<(String, WorkflowInfo)>, LLMSpellError>(
                "workflow_list",
                async move {
                    Ok::<Vec<(String, WorkflowInfo)>, LLMSpellError>(bridge.get_all_workflow_info())
                },
                None,
            )?;

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
        })?,
    )?;

    // Workflow.get() - get a specific workflow by ID
    workflow_table.set(
        "get",
        lua.create_function({
            let bridge_clone = workflow_bridge.clone();
            move |_, workflow_id: String| {
                let bridge = bridge_clone.clone();
                let wf_id = workflow_id.clone();
                let info = block_on_async::<_, Option<WorkflowInfo>, LLMSpellError>(
                    "workflow_get_info",
                    async move {
                        let workflows = bridge.get_all_workflow_info();
                        Ok::<Option<WorkflowInfo>, LLMSpellError>(
                            workflows
                                .into_iter()
                                .find(|(id, _)| id == &wf_id)
                                .map(|(_, info)| info),
                        )
                    },
                    None,
                )?;

                if let Some(workflow_info) = info {
                    Ok(WorkflowInstance {
                        bridge: bridge_clone.clone(),
                        workflow_id: workflow_id.clone(),
                        name: workflow_id, // Use workflow_id as name for now
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
        lua.create_function(move |_, workflow_id: String| {
            let bridge = bridge_clone.clone();

            // Use shared sync utility for async operation
            block_on_async::<_, (), LLMSpellError>(
                &format!("workflow_remove_{workflow_id}"),
                async move { bridge.remove_workflow(&workflow_id).await },
                None,
            )?;

            Ok(())
        })?,
    )?;

    // Workflow.register() - register a workflow (alias for create_workflow)
    let bridge_clone = workflow_bridge.clone();
    workflow_table.set(
        "register",
        lua.create_function(move |_lua, (workflow_type, params): (String, Table)| {
            let bridge = bridge_clone.clone();

            // Convert Lua table to JSON
            let params_json = lua_value_to_json(Value::Table(params))
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to convert params: {e}")))?;

            // Use shared sync utility for async operation
            let workflow_id = block_on_async::<_, String, LLMSpellError>(
                &format!("workflow_register_{workflow_type}"),
                async move { bridge.create_workflow(&workflow_type, params_json).await },
                None,
            )?;

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
            let bridge_for_list = bridge.clone();
            let workflows = block_on_async::<_, Vec<(String, WorkflowInfo)>, LLMSpellError>(
                "workflow_clear_list",
                async move {
                    Ok::<Vec<(String, WorkflowInfo)>, LLMSpellError>(
                        bridge_for_list.get_all_workflow_info(),
                    )
                },
                None,
            )?;

            // Remove each workflow
            for (workflow_id, _) in workflows {
                let bridge = bridge.clone();
                let _ = block_on_async::<_, (), LLMSpellError>(
                    &format!("workflow_clear_remove_{workflow_id}"),
                    async move { bridge.remove_workflow(&workflow_id).await },
                    None,
                );
            }

            Ok(())
        })?,
    )?;

    // Note: executeAsync helper removed - all methods now use synchronous API

    // Add Workflow.builder() method
    let bridge_for_builder = workflow_bridge;
    let builder_fn =
        lua.create_function(move |_lua, ()| Ok(WorkflowBuilder::new(bridge_for_builder.clone())))?;
    workflow_table.set("builder", builder_fn)?;

    // Set Workflow as global
    lua.globals().set("Workflow", workflow_table)?;

    Ok(())
}
