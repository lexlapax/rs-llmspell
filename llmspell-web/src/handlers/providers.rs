use crate::error::WebError;
use crate::state::AppState;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use tracing::{debug, instrument};

/// List available providers and their capabilities
#[utoipa::path(
    get,
    path = "/api/providers",
    tag = "providers",
    responses(
        (status = 200, description = "List of configured providers and their status")
    )
)]
#[instrument(skip(state))]
pub async fn list_providers(State(state): State<AppState>) -> Result<Json<Value>, WebError> {
    debug!("Handling list_providers request");

    let mut kernel = state.kernel.lock().await;

    // Construct model_request to get provider list from kernel
    let request_content = json!({
        "command": "list_providers"
    });

    // Send request to kernel via send_model_request
    // This leverages the kernel's ProviderManager to get the actual list
    let response = kernel
        .send_model_request(request_content)
        .await
        .map_err(|e| WebError::Internal(e.to_string()))?;

    Ok(Json(response))
}
