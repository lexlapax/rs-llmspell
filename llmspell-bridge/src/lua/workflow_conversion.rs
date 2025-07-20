//! ABOUTME: Lua-specific workflow parameter conversion
//! ABOUTME: Handles bidirectional conversion of Lua tables to workflow inputs/outputs

use llmspell_core::{LLMSpellError, Result};
use mlua::{Lua, Table, Value as LuaValue};
use serde_json::Value;

// Helper to convert mlua errors
fn lua_error_to_llmspell(e: mlua::Error) -> LLMSpellError {
    LLMSpellError::Script {
        message: format!("Lua error: {}", e),
        language: Some("lua".to_string()),
        line: None,
        source: None,
    }
}

/// Convert a Lua table to workflow creation parameters
pub fn lua_table_to_workflow_params(lua: &Lua, table: Table) -> Result<serde_json::Value> {
    let mut params = serde_json::Map::new();

    // Convert basic fields
    if let Ok(name) = table.get::<_, String>("name") {
        params.insert("name".to_string(), Value::String(name));
    }

    if let Ok(timeout) = table.get::<_, f64>("timeout") {
        params.insert(
            "timeout".to_string(),
            Value::Number(serde_json::Number::from_f64(timeout).unwrap()),
        );
    }

    // Convert workflow-specific fields
    if let Ok(workflow_type) = table.get::<_, String>("type") {
        match workflow_type.as_str() {
            "sequential" => convert_sequential_params(lua, &table, &mut params)?,
            "conditional" => convert_conditional_params(lua, &table, &mut params)?,
            "loop" => convert_loop_params(lua, &table, &mut params)?,
            "parallel" => convert_parallel_params(lua, &table, &mut params)?,
            _ => {}
        }
    }

    Ok(Value::Object(params))
}

/// Convert workflow output to Lua table
pub fn workflow_output_to_lua_table(lua: &Lua, output: serde_json::Value) -> Result<Table> {
    match json_to_lua_value(lua, output)? {
        LuaValue::Table(table) => Ok(table),
        _ => Err(LLMSpellError::Script {
            message: "Workflow output is not an object".to_string(),
            language: Some("lua".to_string()),
            line: None,
            source: None,
        }),
    }
}

// Helper functions for specific workflow types

fn convert_sequential_params(
    lua: &Lua,
    table: &Table,
    params: &mut serde_json::Map<String, Value>,
) -> Result<()> {
    // Convert steps array
    if let Ok(steps_table) = table.get::<_, Table>("steps") {
        let mut steps = Vec::new();

        for pair in steps_table.pairs::<LuaValue, Table>() {
            let (_, step_table) = pair.map_err(lua_error_to_llmspell)?;

            let step_json = convert_workflow_step(lua, step_table)?;
            steps.push(step_json);
        }

        params.insert("steps".to_string(), Value::Array(steps));
    }

    // Convert error strategy
    if let Ok(error_strategy) = table.get::<_, String>("error_strategy") {
        params.insert("error_strategy".to_string(), Value::String(error_strategy));
    }

    Ok(())
}

fn convert_conditional_params(
    lua: &Lua,
    table: &Table,
    params: &mut serde_json::Map<String, Value>,
) -> Result<()> {
    // Convert condition
    if let Ok(condition) = table.get::<_, LuaValue>("condition") {
        let condition_json = lua_value_to_json(lua, condition)?;
        params.insert("condition".to_string(), condition_json);
    }

    // Convert branches
    if let Ok(branches_table) = table.get::<_, Table>("branches") {
        let mut branches = serde_json::Map::new();

        for pair in branches_table.pairs::<String, Table>() {
            let (branch_name, branch_table) = pair.map_err(lua_error_to_llmspell)?;

            let mut branch_data = serde_json::Map::new();

            // Convert branch steps
            if let Ok(steps_table) = branch_table.get::<_, Table>("steps") {
                let mut steps = Vec::new();

                for pair in steps_table.pairs::<LuaValue, Table>() {
                    let (_, step_table) = pair.map_err(lua_error_to_llmspell)?;

                    let step_json = convert_workflow_step(lua, step_table)?;
                    steps.push(step_json);
                }

                branch_data.insert("steps".to_string(), Value::Array(steps));
            }

            branches.insert(branch_name, Value::Object(branch_data));
        }

        params.insert("branches".to_string(), Value::Object(branches));
    }

    // Convert default branch
    if let Ok(default_branch) = table.get::<_, String>("default_branch") {
        params.insert("default_branch".to_string(), Value::String(default_branch));
    }

    Ok(())
}

fn convert_loop_params(
    lua: &Lua,
    table: &Table,
    params: &mut serde_json::Map<String, Value>,
) -> Result<()> {
    // Convert iterator configuration
    if let Ok(iterator_table) = table.get::<_, Table>("iterator") {
        let iterator_json = lua_table_to_json(lua, iterator_table)?;
        params.insert("iterator".to_string(), iterator_json);
    }

    // Convert body steps
    if let Ok(body_table) = table.get::<_, Table>("body") {
        let mut body_steps = Vec::new();

        for pair in body_table.pairs::<LuaValue, Table>() {
            let (_, step_table) = pair.map_err(lua_error_to_llmspell)?;

            let step_json = convert_workflow_step(lua, step_table)?;
            body_steps.push(step_json);
        }

        params.insert("body".to_string(), Value::Array(body_steps));
    }

    // Convert other loop parameters
    if let Ok(max_iterations) = table.get::<_, i64>("max_iterations") {
        params.insert(
            "max_iterations".to_string(),
            Value::Number(max_iterations.into()),
        );
    }

    if let Ok(break_condition) = table.get::<_, String>("break_condition") {
        params.insert(
            "break_condition".to_string(),
            Value::String(break_condition),
        );
    }

    if let Ok(aggregation) = table.get::<_, String>("aggregation") {
        params.insert("aggregation".to_string(), Value::String(aggregation));
    }

    Ok(())
}

fn convert_parallel_params(
    lua: &Lua,
    table: &Table,
    params: &mut serde_json::Map<String, Value>,
) -> Result<()> {
    // Convert branches array
    if let Ok(branches_table) = table.get::<_, Table>("branches") {
        let mut branches = Vec::new();

        for pair in branches_table.pairs::<LuaValue, Table>() {
            let (_, branch_table) = pair.map_err(lua_error_to_llmspell)?;

            let mut branch = serde_json::Map::new();

            // Branch name
            if let Ok(name) = branch_table.get::<_, String>("name") {
                branch.insert("name".to_string(), Value::String(name));
            }

            // Branch description
            if let Ok(desc) = branch_table.get::<_, String>("description") {
                branch.insert("description".to_string(), Value::String(desc));
            }

            // Optional flag
            if let Ok(optional) = branch_table.get::<_, bool>("optional") {
                branch.insert("optional".to_string(), Value::Bool(optional));
            }

            // Branch steps
            if let Ok(steps_table) = branch_table.get::<_, Table>("steps") {
                let mut steps = Vec::new();

                for pair in steps_table.pairs::<LuaValue, Table>() {
                    let (_, step_table) = pair.map_err(lua_error_to_llmspell)?;

                    let step_json = convert_workflow_step(lua, step_table)?;
                    steps.push(step_json);
                }

                branch.insert("steps".to_string(), Value::Array(steps));
            }

            branches.push(Value::Object(branch));
        }

        params.insert("branches".to_string(), Value::Array(branches));
    }

    // Convert concurrency settings
    if let Ok(max_concurrency) = table.get::<_, i64>("max_concurrency") {
        params.insert(
            "max_concurrency".to_string(),
            Value::Number(max_concurrency.into()),
        );
    }

    if let Ok(fail_fast) = table.get::<_, bool>("fail_fast") {
        params.insert("fail_fast".to_string(), Value::Bool(fail_fast));
    }

    Ok(())
}

fn convert_workflow_step(lua: &Lua, step_table: Table) -> Result<serde_json::Value> {
    let mut step = serde_json::Map::new();

    // Step name
    if let Ok(name) = step_table.get::<_, String>("name") {
        step.insert("name".to_string(), Value::String(name));
    }

    // Determine step type
    if let Ok(tool_name) = step_table.get::<_, String>("tool") {
        step.insert("tool".to_string(), Value::String(tool_name));

        // Tool parameters
        if let Ok(params) = step_table.get::<_, LuaValue>("parameters") {
            let params_json = lua_value_to_json(lua, params)?;
            step.insert("parameters".to_string(), params_json);
        }
    } else if let Ok(agent_id) = step_table.get::<_, String>("agent") {
        step.insert("agent".to_string(), Value::String(agent_id));

        // Agent input
        if let Ok(input) = step_table.get::<_, LuaValue>("input") {
            let input_json = lua_value_to_json(lua, input)?;
            step.insert("input".to_string(), input_json);
        }
    } else if let Ok(func_name) = step_table.get::<_, String>("function") {
        step.insert("function".to_string(), Value::String(func_name));

        // Function parameters
        if let Ok(params) = step_table.get::<_, LuaValue>("parameters") {
            let params_json = lua_value_to_json(lua, params)?;
            step.insert("parameters".to_string(), params_json);
        }
    }

    // Optional fields
    if let Ok(timeout) = step_table.get::<_, f64>("timeout") {
        step.insert(
            "timeout".to_string(),
            Value::Number(serde_json::Number::from_f64(timeout).unwrap()),
        );
    }

    if let Ok(description) = step_table.get::<_, String>("description") {
        step.insert("description".to_string(), Value::String(description));
    }

    Ok(Value::Object(step))
}

// JSON <-> Lua conversion helpers

pub fn lua_value_to_json(lua: &Lua, value: LuaValue) -> Result<serde_json::Value> {
    match value {
        LuaValue::Nil => Ok(Value::Null),
        LuaValue::Boolean(b) => Ok(Value::Bool(b)),
        LuaValue::Integer(i) => Ok(Value::Number(i.into())),
        LuaValue::Number(n) => Ok(Value::Number(serde_json::Number::from_f64(n).ok_or_else(
            || LLMSpellError::Script {
                message: format!("Invalid number: {}", n),
                language: Some("lua".to_string()),
                line: None,
                source: None,
            },
        )?)),
        LuaValue::String(s) => Ok(Value::String(
            s.to_str().map_err(lua_error_to_llmspell)?.to_string(),
        )),
        LuaValue::Table(table) => lua_table_to_json(lua, table),
        _ => Err(LLMSpellError::Script {
            message: format!("Cannot convert Lua value of type {:?} to JSON", value),
            language: Some("lua".to_string()),
            line: None,
            source: None,
        }),
    }
}

fn lua_table_to_json(lua: &Lua, table: Table) -> Result<serde_json::Value> {
    // Check if it's an array (sequential integer keys starting from 1)
    let mut is_array = true;
    let mut max_index = 0;

    for pair in table.clone().pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair.map_err(lua_error_to_llmspell)?;

        match key {
            LuaValue::Integer(i) if i > 0 => {
                max_index = max_index.max(i as usize);
            }
            _ => {
                is_array = false;
                break;
            }
        }
    }

    if is_array && max_index > 0 {
        // Convert as array
        let mut array = Vec::new();
        for i in 1..=max_index {
            let value = table.get::<_, LuaValue>(i).unwrap_or(LuaValue::Nil);
            array.push(lua_value_to_json(lua, value)?);
        }
        Ok(Value::Array(array))
    } else {
        // Convert as object
        let mut object = serde_json::Map::new();
        for pair in table.pairs::<String, LuaValue>() {
            let (key, value) = pair.map_err(lua_error_to_llmspell)?;
            object.insert(key, lua_value_to_json(lua, value)?);
        }
        Ok(Value::Object(object))
    }
}

pub fn json_to_lua_value(lua: &Lua, value: serde_json::Value) -> Result<LuaValue> {
    match value {
        Value::Null => Ok(LuaValue::Nil),
        Value::Bool(b) => Ok(LuaValue::Boolean(b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Err(LLMSpellError::Script {
                    message: format!("Invalid number: {}", n),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                })
            }
        }
        Value::String(s) => Ok(LuaValue::String(
            lua.create_string(&s).map_err(lua_error_to_llmspell)?,
        )),
        Value::Array(arr) => {
            let table = lua.create_table().map_err(lua_error_to_llmspell)?;
            for (i, val) in arr.into_iter().enumerate() {
                table
                    .set(i + 1, json_to_lua_value(lua, val)?)
                    .map_err(lua_error_to_llmspell)?;
            }
            Ok(LuaValue::Table(table))
        }
        Value::Object(obj) => {
            let table = lua.create_table().map_err(lua_error_to_llmspell)?;
            for (key, val) in obj {
                table
                    .set(key, json_to_lua_value(lua, val)?)
                    .map_err(lua_error_to_llmspell)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    // Add Lua-specific tests here
}
