use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::state::AppState;
use llmspell_core::traits::tool::Tool;

#[derive(Serialize)]
pub struct ToolResponse {
    pub name: String,
    pub description: String,
    pub category: String,
    pub schema: Value,
}

#[derive(Deserialize)]
pub struct ExecuteToolRequest {
    pub parameters: Value,
}

#[derive(Serialize)]
pub struct ExecuteToolResponse {
    pub output: String,
}

pub async fn list_tools(
    State(state): State<AppState>,
) -> Result<Json<Vec<ToolResponse>>, String> {
    let kernel = state.kernel.lock().await;
    
    let registry = kernel
        .component_registry()
        .ok_or_else(|| "Component registry not available".to_string())?;

    let tool_names = registry.list_tools().await;
    let mut tools = Vec::new();

    for name in tool_names {
        if let Some(tool) = registry.get_tool(&name).await {
            let metadata = tool.metadata();
            let schema = tool.schema().to_json_schema();
            
            tools.push(ToolResponse {
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                category: tool.category().to_string(),
                schema,
            });
        }
    }

    Ok(Json(tools))
}

pub async fn execute_tool(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ExecuteToolRequest>,
) -> Result<Json<ExecuteToolResponse>, String> {
    let kernel = state.kernel.lock().await;
    
    let registry = kernel
        .component_registry()
        .ok_or_else(|| "Component registry not available".to_string())?;

    let tool = registry
        .get_tool(&id)
        .await
        .ok_or_else(|| format!("Tool '{}' not found", id))?;

    // Create execution context
    let context = llmspell_core::ExecutionContext::new();
    
    // Create input with parameters
    // Tool execution typically expects parameters in the input
    // We use a dummy prompt since tools are usually invoked with params
    let mut input = llmspell_core::types::AgentInput::text("tool_execution");
    
    // Add parameters to input
    if let Value::Object(params) = payload.parameters {
        input = input.with_parameter("parameters", Value::Object(params));
    } else {
        return Err("Parameters must be a JSON object".to_string());
    }

    // Execute tool
    let output = tool
        .execute(input, context)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ExecuteToolResponse {
        output: output.text,
    }))
}
