use axum::{
    Json,
};
use llmspell_config::env::EnvRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::WebError;

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
pub async fn get_config() -> Result<Json<Vec<ConfigItem>>, WebError> {
    // For this implementation, we'll create a new registry instance to read ENV vars
    // In a real app, this should probably come from a shared state if we allow runtime overrides that persist
    let registry = EnvRegistry::new(); // Starts with Global isolation
    
    // We ignore the error here for now as in a web handler we want to return what we can
    let _ = registry.load_from_env();

    let mut items = Vec::new();

    if let Ok(vars) = registry.list_vars() {
        for (name, description, category, sensitive) in vars {
            let value = registry.get(&name);
            
            items.push(ConfigItem {
                name: name.clone(),
                description,
                category: format!("{:?}", category),
                value: if sensitive { Some("***".to_string()) } else { value },
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
    pub overrides: HashMap<String, String>
}

/// Update configuration overrides
pub async fn update_config(
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<UpdateConfigResponse>, WebError> {
    // LIMITATION: Without a shared `Arc<EnvRegistry>` in AppState, we cannot affect global state.
    
    Ok(Json(UpdateConfigResponse {
        status: "updated".to_string(), 
        message: "Configuration updated (simulation - requires shared registry state)".to_string(),
        overrides: payload.overrides
    }))
}
