use crate::state::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::WebError;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ExecuteScriptRequest {
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct ScriptExecutionResponse {
    pub result: String,
}

#[utoipa::path(
    post,
    path = "/api/scripts/execute",
    tag = "scripts",
    request_body = ExecuteScriptRequest,
    responses(
        (status = 200, description = "Script executed successfully", body = ScriptExecutionResponse),
        (status = 500, description = "Execution failed")
    )
)]
pub async fn execute_script(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteScriptRequest>,
) -> Result<Json<ScriptExecutionResponse>, WebError> {
    let mut kernel = state.kernel.lock().await;
    let result = kernel
        .execute(&payload.code)
        .await
        .map_err(|e| WebError::Internal(e.to_string()))?;
    Ok(Json(ScriptExecutionResponse { result }))
}
