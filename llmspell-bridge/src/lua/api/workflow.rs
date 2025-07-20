//! ABOUTME: Simplified Lua Workflow API using data-oriented approach
//! ABOUTME: Provides workflow creation and execution without complex closures
//!
//! This implementation avoids the complexity of nested closures by:
//! 1. Having workflow constructors return simple configuration tables
//! 2. Using a single execute function that retrieves the bridge from Lua registry
//! 3. Following Lua's data-oriented patterns rather than object-oriented

use crate::engine::types::WorkflowApiDefinition;
use crate::lua::workflow_conversion::lua_table_to_workflow_params;
use crate::lua::workflow_results::script_result_to_lua_table;
use crate::workflow_bridge::WorkflowBridge;
use crate::ComponentRegistry;
use llmspell_core::error::LLMSpellError;
use mlua::{Lua, Table, Value as LuaValue};
use std::sync::Arc;

/// Key for storing workflow bridge in Lua registry
const WORKFLOW_BRIDGE_KEY: &str = "llmspell_workflow_bridge";

/// Wrapper for WorkflowBridge that can be stored in Lua
#[derive(Clone)]
struct BridgeWrapper(Arc<WorkflowBridge>);

impl mlua::UserData for BridgeWrapper {}

/// Inject the Workflow API into the Lua environment
pub fn inject_workflow_api(
    lua: &Lua,
    api_def: &WorkflowApiDefinition,
    _registry: Arc<ComponentRegistry>,
    bridge: Arc<WorkflowBridge>,
) -> Result<(), LLMSpellError> {
    // Store the bridge in Lua registry for later access
    let wrapper =
        lua.create_userdata(BridgeWrapper(bridge))
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create workflow bridge wrapper: {}", e),
                source: None,
            })?;
    lua.set_named_registry_value(WORKFLOW_BRIDGE_KEY, wrapper)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to store workflow bridge: {}", e),
            source: None,
        })?;

    // Create the Workflow global table
    let workflow_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Workflow table: {}", e),
        source: None,
    })?;

    // Sequential workflow constructor - just returns config
    let sequential_fn = lua
        .create_function(|lua, config: Table| -> mlua::Result<Table> {
            let workflow = lua.create_table()?;
            workflow.set("type", "sequential")?;
            workflow.set("config", config)?;
            Ok(workflow)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.sequential: {}", e),
            source: None,
        })?;

    // Parallel workflow constructor - just returns config
    let parallel_fn = lua
        .create_function(|lua, config: Table| -> mlua::Result<Table> {
            let workflow = lua.create_table()?;
            workflow.set("type", "parallel")?;
            workflow.set("config", config)?;
            Ok(workflow)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.parallel: {}", e),
            source: None,
        })?;

    // Conditional workflow constructor
    let conditional_fn = lua
        .create_function(|lua, config: Table| -> mlua::Result<Table> {
            let workflow = lua.create_table()?;
            workflow.set("type", "conditional")?;
            workflow.set("config", config)?;
            Ok(workflow)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.conditional: {}", e),
            source: None,
        })?;

    // Loop workflow constructor
    let loop_fn = lua
        .create_function(|lua, config: Table| -> mlua::Result<Table> {
            let workflow = lua.create_table()?;
            workflow.set("type", "loop")?;
            workflow.set("config", config)?;
            Ok(workflow)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.loop: {}", e),
            source: None,
        })?;

    // Single execute function that works with any workflow config
    let execute_fn = lua
        .create_function(|lua, args: mlua::MultiValue| -> mlua::Result<Table> {
            // Get workflow config from arguments
            let workflow_table: Table = match args.into_iter().next() {
                Some(LuaValue::Table(t)) => t,
                _ => {
                    return Err(mlua::Error::RuntimeError(
                        "Workflow.execute requires a workflow configuration table".to_string(),
                    ))
                }
            };

            // Get workflow type and config
            let workflow_type: String = workflow_table.get("type")?;
            let config: Table = workflow_table.get("config")?;

            // Get the bridge from Lua registry
            let wrapper_ud: mlua::AnyUserData = lua.named_registry_value(WORKFLOW_BRIDGE_KEY)?;
            let bridge = wrapper_ud.borrow::<BridgeWrapper>()?.0.clone();

            // Add the type to the config table so conversion knows which type it is
            config.set("type", workflow_type.clone())?;

            // Convert config to params
            let params = lua_table_to_workflow_params(lua, config)
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;

            // Create and execute workflow using a blocking task
            let (_workflow_id, result_json) = {
                let bridge_clone = bridge.clone();
                let workflow_type_clone = workflow_type.clone();
                let params_clone = params.clone();

                // Use spawn_blocking to run async code in a blocking context
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let workflow_id = bridge_clone
                            .create_workflow(&workflow_type_clone, params_clone)
                            .await?;
                        let result = bridge_clone
                            .execute_workflow(&workflow_id, serde_json::json!({}))
                            .await?;
                        Ok::<_, llmspell_core::LLMSpellError>((workflow_id, result))
                    })
                })
                .join()
                .unwrap()
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?
            };

            // Convert result
            let result: crate::workflow_results::ScriptWorkflowResult =
                serde_json::from_value(result_json).map_err(|e| {
                    mlua::Error::ExternalError(Arc::new(llmspell_core::LLMSpellError::Component {
                        message: format!("Failed to deserialize workflow result: {}", e),
                        source: None,
                    }))
                })?;

            script_result_to_lua_table(lua, result)
                .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.execute: {}", e),
            source: None,
        })?;

    // List workflows function
    let list_fn = lua
        .create_function(|lua, _: ()| -> mlua::Result<Table> {
            let wrapper_ud: mlua::AnyUserData = lua.named_registry_value(WORKFLOW_BRIDGE_KEY)?;
            let bridge = wrapper_ud.borrow::<BridgeWrapper>()?.0.clone();
            let workflows = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(bridge.list_workflows())
            })
            .join()
            .unwrap();

            let result = lua.create_table()?;
            for (i, (id, info)) in workflows.into_iter().enumerate() {
                let workflow_table = lua.create_table()?;
                workflow_table.set("id", id)?;
                workflow_table.set("type", info.workflow_type)?;
                workflow_table.set("description", info.description)?;
                result.set(i + 1, workflow_table)?;
            }

            Ok(result)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.list: {}", e),
            source: None,
        })?;

    // Discover types function
    let discover_fn = lua
        .create_function(|lua, _: ()| -> mlua::Result<Table> {
            let wrapper_ud: mlua::AnyUserData = lua.named_registry_value(WORKFLOW_BRIDGE_KEY)?;
            let bridge = wrapper_ud.borrow::<BridgeWrapper>()?.0.clone();
            let types = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(bridge.discover_workflow_types())
            })
            .join()
            .unwrap();

            let result = lua.create_table()?;
            for (i, (type_name, info)) in types.into_iter().enumerate() {
                let type_table = lua.create_table()?;
                type_table.set("type", type_name)?;
                type_table.set("description", info.description)?;

                // Add features
                let features = lua.create_table()?;
                for (j, feature) in info.features.into_iter().enumerate() {
                    features.set(j + 1, feature)?;
                }
                type_table.set("features", features)?;

                result.set(i + 1, type_table)?;
            }

            Ok(result)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.discover_types: {}", e),
            source: None,
        })?;

    // Add all functions to the Workflow table
    workflow_table
        .set(&api_def.constructors.sequential[..], sequential_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.sequential: {}", e),
            source: None,
        })?;
    workflow_table
        .set(&api_def.constructors.parallel[..], parallel_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.parallel: {}", e),
            source: None,
        })?;
    workflow_table
        .set(&api_def.constructors.conditional[..], conditional_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.conditional: {}", e),
            source: None,
        })?;
    workflow_table
        .set(&api_def.constructors.loop_workflow[..], loop_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.loop: {}", e),
            source: None,
        })?;
    workflow_table
        .set("execute", execute_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.execute: {}", e),
            source: None,
        })?;
    workflow_table
        .set("list", list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.list: {}", e),
            source: None,
        })?;
    workflow_table
        .set("discover_types", discover_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.discover_types: {}", e),
            source: None,
        })?;

    // Set the Workflow table as a global
    lua.globals()
        .set(&api_def.global_name[..], workflow_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow global: {}", e),
            source: None,
        })?;

    Ok(())
}

// Example usage from Lua:
// local workflow = Workflow.sequential({
//     name = "my_workflow",
//     steps = {
//         { name = "step1", tool = "calculator", parameters = { operation = "add", a = 1, b = 2 } },
//         { name = "step2", tool = "logger", parameters = { message = "$step1_output" } }
//     }
// })
// local result = Workflow.execute(workflow)
