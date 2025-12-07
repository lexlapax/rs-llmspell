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
pub async fn get_config_schema() -> impl IntoResponse {
    let schema = schema_for!(LLMSpellConfig);
    Json(schema)
}
