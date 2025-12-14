use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use llmspell_config::LLMSpellConfig;
use schemars::schema_for;
use std::fs;

/// Get the raw configuration source (TOML)
#[utoipa::path(
    get,
    path = "/api/config/source",
    tag = "config",
    responses(
        (status = 200, description = "Get configuration TOML source", body = String, content_type = "text/plain"),
        (status = 404, description = "No static configuration file loaded")
    )
)]
pub async fn get_config_source(State(state): State<AppState>) -> impl IntoResponse {
    match &state.static_config_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(content) => (StatusCode::OK, content).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config file: {}", e),
            )
                .into_response(),
        },
        None => (StatusCode::NOT_FOUND, "No static configuration file loaded").into_response(),
    }
}

/// Update the raw configuration source (TOML)
/// Note: This does not reload the configuration in-memory, but persists it to disk.
/// A server restart is required to apply changes.
#[utoipa::path(
    put,
    path = "/api/config/source",
    tag = "config",
    request_body(content = String, content_type = "text/plain"),
    responses(
        (status = 200, description = "Configuration saved"),
        (status = 400, description = "Invalid TOML configuration"),
        (status = 500, description = "Failed to write config file")
    )
)]
pub async fn update_config_source(
    State(state): State<AppState>,
    body: String,
) -> impl IntoResponse {
    match &state.static_config_path {
        Some(path) => {
            // Validate that string is valid TOML for LLMSpellConfig
            match toml::from_str::<LLMSpellConfig>(&body) {
                Ok(_) => {
                    // It's valid, write to disk
                    match fs::write(path, body) {
                        Ok(_) => (StatusCode::OK, "Configuration saved").into_response(),
                        Err(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to write config file: {}", e),
                        )
                            .into_response(),
                    }
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid TOML configuration: {}", e),
                )
                    .into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, "No static configuration file loaded").into_response(),
    }
}

/// Get the JSON Schema for the configuration
#[utoipa::path(
    get,
    path = "/api/config/schema",
    tag = "config",
    responses(
        (status = 200, description = "Get configuration JSON Schema", body = serde_json::Value)
    )
)]
pub async fn get_config_schema() -> impl IntoResponse {
    let schema = schema_for!(LLMSpellConfig);
    Json(schema)
}

/// List available builtin configuration profiles
#[utoipa::path(
    get,
    path = "/api/config/profiles",
    tag = "config",
    responses(
        (status = 200, description = "List builtin configuration profiles", body = Vec<String>)
    )
)]
pub async fn get_profiles() -> Json<Vec<&'static str>> {
    Json(llmspell_config::LLMSpellConfig::list_builtin_profiles())
}

/// Restart the server to apply static configuration changes
///
/// This triggers a process exit, relying on the process manager (systemd, docker, etc.)
/// to restart the service.
#[utoipa::path(
    post,
    path = "/api/config/restart",
    tag = "config",
    responses(
        (status = 200, description = "Server restarting")
    )
)]
pub async fn restart_server() -> impl IntoResponse {
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });
    Json(serde_json::json!({ "status": "restarting" }))
}
