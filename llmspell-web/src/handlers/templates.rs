use axum::{
    extract::{Path, State},
    Json,
};
use llmspell_templates::{
    core::{TemplateMetadata, TemplateParams},
    registry::global_registry,
    validation::ConfigSchema,
};
use serde::{Deserialize, Serialize};

use crate::error::WebError;
use crate::state::AppState;
use tracing::info;

/// List all available templates
pub async fn list_templates() -> Result<Json<Vec<TemplateMetadata>>, WebError> {
    let registry = global_registry();
    let templates = registry.list_metadata();

    // Sort by name for better UX
    let mut templates = templates;
    templates.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Json(templates))
}

#[derive(Serialize)]
pub struct TemplateDetails {
    pub metadata: TemplateMetadata,
    pub schema: ConfigSchema,
}

/// Get details for a specific template
pub async fn get_template(Path(id): Path<String>) -> Result<Json<TemplateDetails>, WebError> {
    let registry = global_registry();

    match registry.get(&id) {
        Ok(template) => Ok(Json(TemplateDetails {
            metadata: template.metadata().clone(),
            schema: template.config_schema(),
        })),
        Err(_) => Err(WebError::NotFound(format!("Template '{}' not found", id))),
    }
}

/// Request body for launching a template
#[derive(Debug, Deserialize)]
pub struct LaunchTemplateRequest {
    /// Parameters for the template
    pub params: Option<TemplateParams>,
    /// Optional session ID to attach (if not creating new)
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct LaunchResponse {
    pub session_id: String,
    pub template_id: String,
    pub status: String,
}

/// Launch a template (Create Session)
pub async fn launch_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<LaunchTemplateRequest>,
) -> Result<Json<LaunchResponse>, WebError> {
    let registry = global_registry();

    let template = match registry.get(&id) {
        Ok(t) => t,
        Err(_) => return Err(WebError::NotFound(format!("Template '{}' not found", id))),
    };

    let params = payload.params.unwrap_or_default();

    // Validate parameters
    if let Err(e) = template.validate(&params) {
        return Err(WebError::BadRequest(format!("Invalid parameters: {}", e)));
    }

    // Create a real session using the Kernel's SessionManager
    let kernel = state.kernel.lock().await;

    // Build session options based on template
    let session_name = payload
        .session_id
        .unwrap_or_else(|| format!("template-{}", id));

    let options_builder = llmspell_kernel::sessions::CreateSessionOptionsBuilder::default()
        .name(session_name)
        .created_by("web-user".to_string())
        .add_tag("template".to_string())
        .add_tag(id.clone());

    let options = options_builder
        .metadata(params.values.into_iter().collect())
        .build();

    let session_manager = kernel.session_manager();
    let session_id = session_manager
        .create_session(options)
        .await
        .map_err(|e| WebError::Internal(format!("Failed to create session: {}", e)))?;

    info!("Launched template '{}' as session {}", id, session_id);

    Ok(Json(LaunchResponse {
        session_id: session_id.to_string(),
        template_id: id,
        status: "created".to_string(),
    }))
}
