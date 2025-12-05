use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ExecuteScriptRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct ScriptExecutionResponse {
    pub result: String,
}

use crate::error::WebError;

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
