use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::WebError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ConfigItem {
    pub name: String,
    pub description: String,
    pub category: String,
    pub value: Option<String>,
    pub default: Option<String>, // Kept for future use
    pub is_sensitive: bool,
    pub is_overridden: bool,
}

/// Get current configuration
pub async fn get_config(State(state): State<AppState>) -> Result<Json<Vec<ConfigItem>>, WebError> {
    // Read from shared registry
    let registry = state.runtime_config.read().await;

    let mut items = Vec::new();

    if let Ok(vars) = registry.list_vars() {
        for (name, description, category, sensitive) in vars {
            let value = registry.get(&name);

            items.push(ConfigItem {
                name: name.clone(),
                description,
                category: format!("{:?}", category),
                value: if sensitive {
                    Some("***".to_string())
                } else {
                    value
                },
                default: None,
                is_sensitive: sensitive,
                is_overridden: false,
            });
        }
    }

    Ok(Json(items))
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub overrides: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct UpdateConfigResponse {
    pub status: String,
    pub message: String,
    pub overrides: HashMap<String, String>,
}

/// Update configuration overrides
pub async fn update_config(
    State(state): State<AppState>,
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<UpdateConfigResponse>, WebError> {
    // Acquire write lock to ensure we are the only one updating config at this moment
    let registry = state.runtime_config.write().await;

    // Update process environment variables
    for (key, value) in &payload.overrides {
        std::env::set_var(key, value);
    }

    // Update internal registry overrides so subsequent gets reflect changes immediately
    registry
        .with_overrides(payload.overrides.clone())
        .map_err(WebError::Internal)?;

    // Task 14.5.1e: Persist changes to SQLite
    if let Some(storage) = &state.config_store {
        use llmspell_core::traits::storage::StorageBackend;
        for (key, value) in &payload.overrides {
            let storage_key = format!("config:{}", key);
            storage
                .set(&storage_key, value.as_bytes().to_vec())
                .await
                .map_err(|e| WebError::Internal(format!("Failed to persist config: {}", e)))?;
        }
    }

    Ok(Json(UpdateConfigResponse {
        status: "updated".to_string(),
        message: "Configuration updated successfully".to_string(),
        overrides: payload.overrides,
    }))
}
