use axum::{
    extract::Path,
    Json,
};
use llmspell_templates::{
    core::{TemplateMetadata, TemplateParams},
    registry::global_registry,
    validation::ConfigSchema,
};
use serde::{Deserialize, Serialize};

use crate::error::WebError;

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
        Ok(template) => {
            Ok(Json(TemplateDetails {
                metadata: template.metadata().clone(),
                schema: template.config_schema(),
            }))
        },
        Err(_) => {
            Err(WebError::NotFound(format!("Template '{}' not found", id)))
        }
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

    // TODO: Phase 14.4 - Integrate with SessionManager to actually create a functional session
    let mock_session_id = uuid::Uuid::new_v4().to_string();

    tracing::info!("Launched template '{}' as session {}", id, mock_session_id);

    Ok(Json(LaunchResponse {
        session_id: mock_session_id,
        template_id: id,
        status: "created".to_string(),
    }))
}
