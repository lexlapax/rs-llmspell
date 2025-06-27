//! ABOUTME: Lua Workflow API implementation providing workflow constructors
//! ABOUTME: Bridges between Lua scripts and Rust Workflow implementations

use crate::engine::types::WorkflowApiDefinition;
use crate::ComponentRegistry;
use llmspell_core::error::LLMSpellError;
use mlua::Lua;
use std::sync::Arc;

/// Inject the Workflow API into the Lua environment
pub fn inject_workflow_api(
    lua: &Lua,
    api_def: &WorkflowApiDefinition,
    registry: Arc<ComponentRegistry>,
) -> Result<(), LLMSpellError> {
    // Create the Workflow global table
    let workflow_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Workflow table: {}", e),
        source: None,
    })?;

    // Clone registry for closures
    let _registry_clone = registry.clone();

    // Implement Workflow.sequential() constructor
    let sequential_fn = lua
        .create_function(|lua, steps: mlua::Table| -> mlua::Result<mlua::Table> {
            let workflow = lua.create_table()?;
            workflow.set("type", "sequential")?;
            workflow.set("steps", steps)?;

            // Add execute method
            workflow.set(
                "execute",
                lua.create_async_function(|lua, _args: mlua::Table| async move {
                    // Mock execution - run steps in sequence
                    let result = lua.create_table()?;
                    result.set("success", true)?;
                    result.set("message", "Sequential workflow executed")?;
                    Ok(result)
                })?,
            )?;

            Ok(workflow)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.sequential: {}", e),
            source: None,
        })?;

    // Implement Workflow.parallel() constructor
    let parallel_fn = lua
        .create_function(|lua, steps: mlua::Table| -> mlua::Result<mlua::Table> {
            let workflow = lua.create_table()?;
            workflow.set("type", "parallel")?;
            workflow.set("steps", steps)?;

            // Add execute method
            workflow.set(
                "execute",
                lua.create_async_function(|lua, _args: mlua::Table| async move {
                    // Mock execution - would run steps in parallel
                    let result = lua.create_table()?;
                    result.set("success", true)?;
                    result.set("message", "Parallel workflow executed")?;
                    Ok(result)
                })?,
            )?;

            Ok(workflow)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Workflow.parallel: {}", e),
            source: None,
        })?;

    // Add constructors to Workflow table
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

    // Add placeholder for conditional and loop
    let placeholder = lua
        .create_function(|_lua, _: mlua::Value| -> mlua::Result<mlua::Table> {
            Err(mlua::Error::RuntimeError(
                "Workflow type not yet implemented".to_string(),
            ))
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create placeholder: {}", e),
            source: None,
        })?;

    workflow_table
        .set(&api_def.constructors.conditional[..], placeholder.clone())
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.conditional: {}", e),
            source: None,
        })?;

    workflow_table
        .set(&api_def.constructors.loop_workflow[..], placeholder)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Workflow.loop: {}", e),
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
