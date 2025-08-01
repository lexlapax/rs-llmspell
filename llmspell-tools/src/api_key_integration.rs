//! API Key Integration for Tools
//!
//! This module provides integration between tools and the API key management system.

use chrono::Utc;
use llmspell_utils::api_key_manager::{ApiKeyManager, ApiKeyMetadata};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

/// Global API key manager instance
static API_KEY_MANAGER: Lazy<Arc<RwLock<ApiKeyManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(ApiKeyManager::new())));

use std::sync::Once;

/// One-time initialization flag
static INIT: Once = Once::new();

/// Initialize the API key manager (loads from environment)
fn ensure_initialized() {
    INIT.call_once(|| {
        // Don't load from environment during initialization to avoid deadlock
        // The manager is already initialized with new()
    });
}

/// Get the global API key manager
pub fn get_api_key_manager() -> Arc<RwLock<ApiKeyManager>> {
    ensure_initialized();
    Arc::clone(&API_KEY_MANAGER)
}

/// Get an API key for a service
pub fn get_api_key(service: &str) -> Option<String> {
    ensure_initialized();
    let manager = API_KEY_MANAGER.read();
    match manager.get_key(service) {
        Ok(Some(key)) => Some(key),
        Ok(None) => {
            // Also check environment variable directly as fallback
            std::env::var(format!("LLMSPELL_API_KEY_{}", service.to_uppercase())).ok()
        }
        Err(e) => {
            tracing::warn!("Error retrieving API key for service '{}': {}", service, e);
            None
        }
    }
}

/// Add an API key programmatically
pub fn add_api_key(service: &str, key: &str) -> Result<(), String> {
    ensure_initialized();
    let manager = API_KEY_MANAGER.read();
    let metadata = ApiKeyMetadata {
        key_id: format!("tool_{}", service),
        service: service.to_string(),
        created_at: Utc::now(),
        last_used: None,
        expires_at: None,
        is_active: true,
        usage_count: 0,
    };

    manager.add_key(&metadata.key_id, key, metadata.clone())
}

/// Configuration helper for tools that need API keys
pub struct ApiKeyConfig {
    /// Service name (e.g., "google_search", "sendgrid")
    pub service: String,
    /// Whether the API key is required
    pub required: bool,
    /// Default value if not found (for testing)
    pub default: Option<String>,
}

impl ApiKeyConfig {
    /// Create a new API key configuration
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            required: true,
            default: None,
        }
    }

    /// Set whether the key is required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set a default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Get the API key
    pub fn get_key(&self) -> Result<Option<String>, String> {
        if let Some(key) = get_api_key(&self.service) {
            Ok(Some(key))
        } else if let Some(default) = &self.default {
            Ok(Some(default.clone()))
        } else if self.required {
            Err(format!(
                "API key for service '{}' is required but not found. \
                Set the environment variable LLMSPELL_API_KEY_{} or configure it using the CLI.",
                self.service,
                self.service.to_uppercase()
            ))
        } else {
            Ok(None)
        }
    }
}

/// Trait for tools that require API keys
pub trait RequiresApiKey {
    /// Get the list of API key configurations this tool needs
    fn api_key_configs(&self) -> Vec<ApiKeyConfig>;

    /// Validate that all required API keys are available
    fn validate_api_keys(&self) -> Result<(), String> {
        for config in self.api_key_configs() {
            config.get_key()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_api_key_config() {
        let config = ApiKeyConfig::new("test_service")
            .required(false)
            .with_default("test_key");

        let key = config.get_key().unwrap();
        assert_eq!(key, Some("test_key".to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_add_and_get_key() {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Create a local manager instance for testing to avoid global state issues
        let manager = ApiKeyManager::new();

        // Use a unique service name to avoid conflicts with other tests
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let service = format!("test_tool_{}_{}", std::process::id(), timestamp);
        let key_id = format!("tool_{}", service);

        let metadata = ApiKeyMetadata {
            key_id: key_id.clone(),
            service: service.clone(),
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: true,
            usage_count: 0,
        };

        // Add key directly to the local manager
        manager.add_key(&key_id, "secret123", metadata).unwrap();

        // Get key from the local manager
        let retrieved_key = manager.get_key(&service).unwrap();
        assert_eq!(retrieved_key, Some("secret123".to_string()));
    }
}
