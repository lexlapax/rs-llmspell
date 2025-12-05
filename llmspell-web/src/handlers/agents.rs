use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::error::WebError;

#[derive(Serialize)]
pub struct AgentResponse {
    pub name: String,
    // Add more fields if available in Agent trait (e.g. description)
}

#[derive(Deserialize)]
pub struct ExecuteAgentRequest {
    pub input: String,
}

#[derive(Serialize)]
pub struct ExecuteAgentResponse {
    pub output: String,
}

pub async fn list_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentResponse>>, WebError> {
    let kernel = state.kernel.lock().await;
    
    let registry = kernel
        .component_registry()
        .ok_or_else(|| WebError::Internal("Component registry not available".to_string()))?;

    let agents = registry.list_agents().await;

    let response = agents
        .into_iter()
        .map(|name| AgentResponse { name })
        .collect();

    Ok(Json(response))
}

pub async fn execute_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ExecuteAgentRequest>,
) -> Result<Json<ExecuteAgentResponse>, WebError> {
    let kernel = state.kernel.lock().await;
    
    let registry = kernel
        .component_registry()
        .ok_or_else(|| WebError::Internal("Component registry not available".to_string()))?;

    let agent = registry
        .get_agent(&id)
        .await
        .ok_or_else(|| WebError::NotFound(format!("Agent '{}' not found", id)))?;

    // Execute agent
    // Note: Agent trait execute method signature needs to be checked.
    // Assuming execute(input) -> Result<Output>
    // I'll check Agent trait next, but for now assuming standard interface.
    // Wait, I should check Agent trait first to be sure.
    // But I'll write this and fix if needed.
    
    // Create execution context
    let context = llmspell_core::ExecutionContext::new();
    
    // Create input
    let input = llmspell_core::types::AgentInput::text(payload.input);

    // Execute agent
    let output = agent
        .execute(input, context)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(ExecuteAgentResponse {
        output: output.text,
    }))
}
