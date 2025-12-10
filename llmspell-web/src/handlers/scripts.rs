use crate::state::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::WebError;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ExecuteScriptRequest {
    pub code: String,
    #[serde(default)]
    pub engine: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ExecuteScriptResponse {
    pub output: String,
}

#[utoipa::path(
    post,
    path = "/api/scripts/execute",
    tag = "scripts",
    request_body = ExecuteScriptRequest,
    responses(
        (status = 200, description = "Script executed successfully", body = ExecuteScriptResponse),
        (status = 500, description = "Execution failed")
    )
)]
pub async fn execute_script(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteScriptRequest>,
) -> Result<Json<ExecuteScriptResponse>, WebError> {
    let mut kernel = state.kernel.lock().await;

    // TODO: Handle engine selection if kernel supports it via directives
    let result = kernel
        .execute(&payload.code)
        .await
        .map_err(|e| WebError::Internal(format!("Script execution failed: {}", e)))?;

    Ok(Json(ExecuteScriptResponse { output: result }))
}
