//! ABOUTME: Lua-specific Workflow global implementation
//! ABOUTME: Provides Lua bindings for Workflow functionality

use crate::globals::GlobalContext;
use crate::lua::conversion::lua_value_to_json;
use crate::ComponentRegistry;
use llmspell_core::ComponentId;
use llmspell_workflows::{
    ConditionalWorkflow, ConditionalWorkflowBuilder, LoopWorkflow, LoopWorkflowBuilder,
    ParallelWorkflow, ParallelWorkflowBuilder, SequentialWorkflow, SequentialWorkflowBuilder,
    StepType, WorkflowStep,
};
use mlua::{Lua, Table, UserData, UserDataMethods};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Enum to hold different workflow types
enum WorkflowType {
    Sequential(SequentialWorkflow),
    Conditional(ConditionalWorkflow),
    Loop(LoopWorkflow),
    Parallel(ParallelWorkflow),
}

/// Lua userdata representing a workflow instance
struct LuaWorkflowInstance {
    workflow: Arc<RwLock<WorkflowType>>,
    name: String,
    description: String,
}

impl UserData for LuaWorkflowInstance {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // execute method
        methods.add_async_method("execute", |lua, this, _input: Option<Table>| async move {
            let workflow = this.workflow.read().await;

            // Execute workflow and convert result to Lua table
            let result_table = lua.create_table()?;

            match &*workflow {
                WorkflowType::Sequential(wf) => {
                    let result = wf.execute().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Sequential workflow failed: {}", e))
                    })?;

                    result_table.set("workflow_name", result.workflow_name)?;
                    result_table.set("success", result.success)?;
                    result_table.set("duration_ms", result.duration.as_millis() as u64)?;
                    if let Some(error) = result.error_message {
                        result_table.set("error", error)?;
                    }

                    // Add successful steps
                    let successful_steps = lua.create_table()?;
                    for (i, step) in result.successful_steps.iter().enumerate() {
                        let step_table = lua.create_table()?;
                        step_table.set("id", step.step_id.to_string())?;
                        step_table.set("name", step.step_name.clone())?;
                        step_table.set("output", step.output.clone())?;
                        step_table.set("duration_ms", step.duration.as_millis() as u64)?;
                        successful_steps.set(i + 1, step_table)?;
                    }
                    result_table.set("successful_steps", successful_steps)?;

                    // Add failed steps
                    let failed_steps = lua.create_table()?;
                    for (i, step) in result.failed_steps.iter().enumerate() {
                        let step_table = lua.create_table()?;
                        step_table.set("id", step.step_id.to_string())?;
                        step_table.set("name", step.step_name.clone())?;
                        if let Some(error) = &step.error {
                            step_table.set("error", error.clone())?;
                        }
                        step_table.set("duration_ms", step.duration.as_millis() as u64)?;
                        failed_steps.set(i + 1, step_table)?;
                    }
                    result_table.set("failed_steps", failed_steps)?;
                }
                WorkflowType::Conditional(wf) => {
                    let result = wf.execute().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Conditional workflow failed: {}", e))
                    })?;

                    result_table.set("workflow_name", result.workflow_name)?;
                    result_table.set("success", result.success)?;
                    result_table.set("duration_ms", result.duration.as_millis() as u64)?;
                    result_table.set("matched_branches", result.matched_branches)?;
                    result_table.set("total_branches", result.total_branches)?;
                    if let Some(error) = result.error_message {
                        result_table.set("error", error)?;
                    }
                }
                WorkflowType::Loop(wf) => {
                    let result = wf.execute().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Loop workflow failed: {}", e))
                    })?;

                    result_table.set("workflow_name", result.workflow_name)?;
                    result_table.set("success", result.success)?;
                    result_table.set("total_iterations", result.total_iterations)?;
                    result_table.set("completed_iterations", result.completed_iterations)?;
                    result_table.set("break_reason", result.break_reason)?;
                }
                WorkflowType::Parallel(wf) => {
                    let result = wf.execute().await.map_err(|e| {
                        mlua::Error::RuntimeError(format!("Parallel workflow failed: {}", e))
                    })?;

                    result_table.set("workflow_name", result.workflow_name)?;
                    result_table.set("success", result.success)?;
                    result_table.set("duration_ms", result.duration.as_millis() as u64)?;
                    result_table.set("successful_branches", result.successful_branches)?;
                    result_table.set("failed_branches", result.failed_branches)?;
                }
            }

            Ok(result_table)
        });

        // getInfo method
        methods.add_method("getInfo", |lua, this, ()| {
            let info_table = lua.create_table()?;
            info_table.set("name", this.name.clone())?;
            info_table.set("description", this.description.clone())?;

            let workflow = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(this.workflow.read())
            });

            match &*workflow {
                WorkflowType::Sequential(_) => {
                    info_table.set("type", "sequential")?;
                }
                WorkflowType::Conditional(_) => {
                    info_table.set("type", "conditional")?;
                }
                WorkflowType::Loop(_) => {
                    info_table.set("type", "loop")?;
                }
                WorkflowType::Parallel(_) => {
                    info_table.set("type", "parallel")?;
                }
            }
            Ok(info_table)
        });

        // addStep method (for sequential workflows)
        methods.add_async_method(
            "addStep",
            |_lua, this, (name, step_type, config): (String, String, Table)| async move {
                let config_json = lua_value_to_json(mlua::Value::Table(config))?;

                let mut workflow = this.workflow.write().await;
                match &mut *workflow {
                    WorkflowType::Sequential(wf) => {
                        // Create a workflow step based on type
                        let step = match step_type.as_str() {
                            "agent" => {
                                let agent_id = config_json
                                    .get("agent_id")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        mlua::Error::RuntimeError(
                                            "Agent step requires 'agent_id' field".to_string(),
                                        )
                                    })?;
                                let input = config_json
                                    .get("input")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");

                                WorkflowStep::new(
                                    name,
                                    StepType::Agent {
                                        agent_id: ComponentId::from_name(agent_id),
                                        input: input.to_string(),
                                    },
                                )
                            }
                            "tool" => {
                                let tool_name = config_json
                                    .get("tool")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        mlua::Error::RuntimeError(
                                            "Tool step requires 'tool' field".to_string(),
                                        )
                                    })?;

                                WorkflowStep::new(
                                    name,
                                    StepType::Tool {
                                        tool_name: tool_name.to_string(),
                                        parameters: config_json
                                            .get("parameters")
                                            .cloned()
                                            .unwrap_or(serde_json::json!({})),
                                    },
                                )
                            }
                            "custom" => {
                                let function_name = config_json
                                    .get("function")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        mlua::Error::RuntimeError(
                                            "Custom step requires 'function' field".to_string(),
                                        )
                                    })?;

                                WorkflowStep::new(
                                    name,
                                    StepType::Custom {
                                        function_name: function_name.to_string(),
                                        parameters: config_json
                                            .get("parameters")
                                            .cloned()
                                            .unwrap_or(serde_json::json!({})),
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

                        wf.add_step(step);
                        Ok(())
                    }
                    _ => Err(mlua::Error::RuntimeError(
                        "addStep is only supported for sequential workflows".to_string(),
                    )),
                }
            },
        );
    }
}

/// Inject Workflow global into Lua environment
pub fn inject_workflow_global(
    lua: &Lua,
    _context: &GlobalContext,
    _registry: Arc<ComponentRegistry>,
) -> mlua::Result<()> {
    let workflow_table = lua.create_table()?;

    // Create Workflow.sequential() function
    let sequential_fn =
        lua.create_function(move |_lua, (name, description): (String, Option<String>)| {
            let builder = SequentialWorkflowBuilder::new(name.clone());
            let workflow = builder.build();

            let workflow_instance = LuaWorkflowInstance {
                workflow: Arc::new(RwLock::new(WorkflowType::Sequential(workflow))),
                name,
                description: description.unwrap_or_default(),
            };

            Ok(workflow_instance)
        })?;

    // Create Workflow.conditional() function
    let conditional_fn =
        lua.create_function(move |_lua, (name, description): (String, Option<String>)| {
            let builder = ConditionalWorkflowBuilder::new(name.clone());
            let workflow = builder.build();

            let workflow_instance = LuaWorkflowInstance {
                workflow: Arc::new(RwLock::new(WorkflowType::Conditional(workflow))),
                name,
                description: description.unwrap_or_default(),
            };

            Ok(workflow_instance)
        })?;

    // Create Workflow.loop() function
    let loop_fn = lua.create_function(
        move |_lua,
              (name, _max_iterations, _condition, description): (
            String,
            Option<usize>,
            Option<String>,
            Option<String>,
        )| {
            let builder = LoopWorkflowBuilder::new(name.clone());

            let workflow = builder.build().map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to build loop workflow: {}", e))
            })?;

            let workflow_instance = LuaWorkflowInstance {
                workflow: Arc::new(RwLock::new(WorkflowType::Loop(workflow))),
                name,
                description: description.unwrap_or_default(),
            };

            Ok(workflow_instance)
        },
    )?;

    // Create Workflow.parallel() function
    let parallel_fn =
        lua.create_function(move |_lua, (name, description): (String, Option<String>)| {
            let builder = ParallelWorkflowBuilder::new(name.clone());
            let workflow = builder.build().map_err(|e| {
                mlua::Error::RuntimeError(format!("Failed to build parallel workflow: {}", e))
            })?;

            let workflow_instance = LuaWorkflowInstance {
                workflow: Arc::new(RwLock::new(WorkflowType::Parallel(workflow))),
                name,
                description: description.unwrap_or_default(),
            };

            Ok(workflow_instance)
        })?;

    // Create Workflow.create() function for backward compatibility
    let create_fn = lua.create_function(move |_lua, args: Table| {
        let workflow_type: String = args.get("type")?;
        let name: String = args.get("name")?;
        let description: Option<String> = args.get("description").ok();

        match workflow_type.as_str() {
            "sequential" => {
                let builder = SequentialWorkflowBuilder::new(name.clone());
                let workflow = builder.build();

                let workflow_instance = LuaWorkflowInstance {
                    workflow: Arc::new(RwLock::new(WorkflowType::Sequential(workflow))),
                    name,
                    description: description.unwrap_or_default(),
                };

                Ok(workflow_instance)
            }
            "conditional" => {
                let builder = ConditionalWorkflowBuilder::new(name.clone());
                let workflow = builder.build();

                let workflow_instance = LuaWorkflowInstance {
                    workflow: Arc::new(RwLock::new(WorkflowType::Conditional(workflow))),
                    name,
                    description: description.unwrap_or_default(),
                };

                Ok(workflow_instance)
            }
            "loop" => {
                let builder = LoopWorkflowBuilder::new(name.clone());
                let workflow = builder.build().map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to build loop workflow: {}", e))
                })?;

                let workflow_instance = LuaWorkflowInstance {
                    workflow: Arc::new(RwLock::new(WorkflowType::Loop(workflow))),
                    name,
                    description: description.unwrap_or_default(),
                };

                Ok(workflow_instance)
            }
            "parallel" => {
                let builder = ParallelWorkflowBuilder::new(name.clone());
                let workflow = builder.build().map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to build parallel workflow: {}", e))
                })?;

                let workflow_instance = LuaWorkflowInstance {
                    workflow: Arc::new(RwLock::new(WorkflowType::Parallel(workflow))),
                    name,
                    description: description.unwrap_or_default(),
                };

                Ok(workflow_instance)
            }
            _ => Err(mlua::Error::RuntimeError(format!(
                "Unknown workflow type: {}",
                workflow_type
            ))),
        }
    })?;

    // Set functions on Workflow table
    workflow_table.set("create", create_fn)?;
    workflow_table.set("sequential", sequential_fn)?;
    workflow_table.set("conditional", conditional_fn)?;
    workflow_table.set("loop", loop_fn)?;
    workflow_table.set("parallel", parallel_fn)?;

    // Set Workflow as global
    lua.globals().set("Workflow", workflow_table)?;

    Ok(())
}
