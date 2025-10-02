//! Local provider trait extensions for model management
//!
//! This module defines traits and types for local LLM providers (Ollama and Candle)
//! that extend the base ProviderInstance with model management capabilities.

pub mod ollama_manager;
pub mod ollama_provider;
pub mod candle;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::abstraction::{ProviderConfig, ProviderInstance};
use llmspell_core::error::LLMSpellError;

pub use ollama_manager::OllamaModelManager;
pub use ollama_provider::OllamaProvider;
pub use candle::{CandleProvider, create_candle_provider};

/// Factory function to create an Ollama provider instance
///
/// This function creates an OllamaProvider that uses rig for inference
/// and ollama-rs for model management. It requires:
/// - A rig provider for the actual inference
/// - Base URL for Ollama API (defaults to http://localhost:11434)
///
/// # Arguments
/// * `config` - Provider configuration with model and optional endpoint
///
/// # Returns
/// * `Ok(Box<dyn ProviderInstance>)` - Initialized Ollama provider
/// * `Err(LLMSpellError)` - Configuration or initialization error
///
/// # Examples
///
/// ```no_run
/// # use llmspell_providers::abstraction::ProviderConfig;
/// # use llmspell_providers::local::create_ollama_provider;
/// let mut config = ProviderConfig::new_with_type("ollama", "local", "llama3.1:8b");
/// config.endpoint = Some("http://localhost:11434".to_string());
/// let provider = create_ollama_provider(config).unwrap();
/// ```
pub fn create_ollama_provider(
    config: ProviderConfig,
) -> Result<Box<dyn ProviderInstance>, LLMSpellError> {
    // Base URL for Ollama (from config or default)
    let base_url = config
        .endpoint
        .clone()
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    // Create underlying rig provider for inference
    let rig_config = ProviderConfig {
        name: "rig".to_string(),
        provider_type: "ollama".to_string(),
        model: config.model.clone(),
        endpoint: Some(base_url.clone()),
        api_key: None, // Ollama doesn't require API keys
        timeout_secs: config.timeout_secs,
        max_retries: config.max_retries,
        custom_config: config.custom_config.clone(),
    };

    let rig_provider = crate::rig::create_rig_provider(rig_config)?;

    // Wrap in OllamaProvider for model management
    Ok(Box::new(OllamaProvider::new(rig_provider, base_url)))
}

/// Health status of a local provider backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum HealthStatus {
    /// Backend is healthy and operational
    Healthy {
        /// Number of models available locally
        available_models: usize,
        /// Backend version if available
        version: Option<String>,
    },
    /// Backend is not responding or has errors
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
    },
    /// Backend status is unknown
    Unknown,
}

/// Local model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    /// Model identifier (e.g., "llama3.1:8b")
    pub id: String,
    /// Backend name ("ollama" or "candle")
    pub backend: String,
    /// Model size in bytes
    pub size_bytes: u64,
    /// Quantization format if known (e.g., "Q4_K_M")
    pub quantization: Option<String>,
    /// Last modification time
    pub modified_at: Option<SystemTime>,
}

/// Model download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    /// Model being downloaded
    pub model_id: String,
    /// Current download status
    pub status: DownloadStatus,
    /// Percentage complete (0.0 - 100.0)
    pub percent_complete: f32,
    /// Bytes downloaded so far
    pub bytes_downloaded: u64,
    /// Total bytes to download (if known)
    pub bytes_total: Option<u64>,
}

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum DownloadStatus {
    /// Download is starting
    Starting,
    /// Download in progress
    Downloading,
    /// Verifying downloaded files
    Verifying,
    /// Download complete
    Complete,
    /// Download failed
    Failed {
        /// Error message
        error: String,
    },
}

/// Model specification for downloads
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model name (e.g., "llama3.1")
    pub model: String,
    /// Model variant/size (e.g., "8b")
    pub variant: Option<String>,
    /// Backend to use ("ollama" or "candle")
    pub backend: Option<String>,
}

impl ModelSpec {
    /// Create a new ModelSpec
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            variant: None,
            backend: None,
        }
    }

    /// Create ModelSpec with variant
    pub fn with_variant(model: impl Into<String>, variant: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            variant: Some(variant.into()),
            backend: None,
        }
    }

    /// Parse from model specifier string
    ///
    /// # Examples
    ///
    /// ```
    /// # use llmspell_providers::local::ModelSpec;
    /// let spec = ModelSpec::parse("llama3.1:8b@ollama").unwrap();
    /// assert_eq!(spec.model, "llama3.1");
    /// assert_eq!(spec.variant, Some("8b".to_string()));
    /// assert_eq!(spec.backend, Some("ollama".to_string()));
    /// ```
    pub fn parse(spec: &str) -> Result<Self> {
        // Split on @ to extract backend
        let (model_part, backend) = if let Some(idx) = spec.rfind('@') {
            (&spec[..idx], Some(spec[idx + 1..].to_string()))
        } else {
            (spec, None)
        };

        // Split on : to extract variant
        let (model, variant) = if let Some(idx) = model_part.find(':') {
            (&model_part[..idx], Some(model_part[idx + 1..].to_string()))
        } else {
            (model_part, None)
        };

        Ok(Self {
            model: model.to_string(),
            variant,
            backend,
        })
    }
}

/// Detailed model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: String,
    /// Backend name
    pub backend: String,
    /// Model size in bytes
    pub size_bytes: u64,
    /// Parameter count as string (e.g., "7B", "13B")
    pub parameter_count: Option<String>,
    /// Quantization format
    pub quantization: Option<String>,
    /// Model file format (e.g., "GGUF", "Safetensors")
    pub format: String,
    /// Whether model is currently loaded in memory
    pub loaded: bool,
}

/// Trait for local LLM providers with model management
///
/// This trait extends ProviderInstance with operations specific to local models:
/// - Health checking
/// - Model listing
/// - Model downloading
/// - Model information queries
/// - Model unloading
#[async_trait]
pub trait LocalProviderInstance: ProviderInstance {
    /// Check if backend is available and healthy
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_providers::local::{LocalProviderInstance, HealthStatus};
    /// # async fn example(provider: &dyn LocalProviderInstance) -> anyhow::Result<()> {
    /// let status = provider.health_check().await?;
    /// match status {
    ///     HealthStatus::Healthy { available_models, .. } => {
    ///         println!("Backend healthy with {} models", available_models);
    ///     }
    ///     HealthStatus::Unhealthy { reason } => {
    ///         println!("Backend unhealthy: {}", reason);
    ///     }
    ///     HealthStatus::Unknown => {
    ///         println!("Backend status unknown");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn health_check(&self) -> Result<HealthStatus>;

    /// List locally available models
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_providers::local::LocalProviderInstance;
    /// # async fn example(provider: &dyn LocalProviderInstance) -> anyhow::Result<()> {
    /// let models = provider.list_local_models().await?;
    /// for model in models {
    ///     println!("{}: {} bytes", model.id, model.size_bytes);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn list_local_models(&self) -> Result<Vec<LocalModel>>;

    /// Pull/download a model
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_providers::local::{LocalProviderInstance, ModelSpec};
    /// # async fn example(provider: &dyn LocalProviderInstance) -> anyhow::Result<()> {
    /// let spec = ModelSpec::parse("llama3.1:8b@ollama")?;
    /// let progress = provider.pull_model(&spec).await?;
    /// println!("Downloaded {} ({}% complete)", progress.model_id, progress.percent_complete);
    /// # Ok(())
    /// # }
    /// ```
    async fn pull_model(&self, model_spec: &ModelSpec) -> Result<PullProgress>;

    /// Get detailed model information
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_providers::local::LocalProviderInstance;
    /// # async fn example(provider: &dyn LocalProviderInstance) -> anyhow::Result<()> {
    /// let info = provider.model_info("llama3.1:8b").await?;
    /// println!("Model: {} ({} format)", info.id, info.format);
    /// # Ok(())
    /// # }
    /// ```
    async fn model_info(&self, model_id: &str) -> Result<ModelInfo>;

    /// Unload model from memory (if applicable)
    ///
    /// Some backends (like Candle) load models into memory. This method
    /// unloads them to free resources. For backends that don't manage
    /// model loading (like Ollama), this is a no-op.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_providers::local::LocalProviderInstance;
    /// # async fn example(provider: &dyn LocalProviderInstance) -> anyhow::Result<()> {
    /// provider.unload_model("llama3.1:8b").await?;
    /// println!("Model unloaded");
    /// # Ok(())
    /// # }
    /// ```
    async fn unload_model(&self, model_id: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_spec_parse() {
        let spec = ModelSpec::parse("llama3.1:8b@ollama").unwrap();
        assert_eq!(spec.model, "llama3.1");
        assert_eq!(spec.variant, Some("8b".to_string()));
        assert_eq!(spec.backend, Some("ollama".to_string()));
    }

    #[test]
    fn test_model_spec_parse_no_backend() {
        let spec = ModelSpec::parse("llama3.1:8b").unwrap();
        assert_eq!(spec.model, "llama3.1");
        assert_eq!(spec.variant, Some("8b".to_string()));
        assert_eq!(spec.backend, None);
    }

    #[test]
    fn test_model_spec_parse_no_variant() {
        let spec = ModelSpec::parse("phi3@candle").unwrap();
        assert_eq!(spec.model, "phi3");
        assert_eq!(spec.variant, None);
        assert_eq!(spec.backend, Some("candle".to_string()));
    }

    #[test]
    fn test_model_spec_parse_minimal() {
        let spec = ModelSpec::parse("mistral").unwrap();
        assert_eq!(spec.model, "mistral");
        assert_eq!(spec.variant, None);
        assert_eq!(spec.backend, None);
    }

    #[test]
    fn test_model_spec_constructors() {
        let spec1 = ModelSpec::new("llama3.1");
        assert_eq!(spec1.model, "llama3.1");
        assert_eq!(spec1.variant, None);

        let spec2 = ModelSpec::with_variant("llama3.1", "8b");
        assert_eq!(spec2.model, "llama3.1");
        assert_eq!(spec2.variant, Some("8b".to_string()));
    }

    #[test]
    fn test_health_status_serde() {
        let healthy = HealthStatus::Healthy {
            available_models: 5,
            version: Some("v1.0".to_string()),
        };
        let json = serde_json::to_string(&healthy).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        matches!(
            deserialized,
            HealthStatus::Healthy {
                available_models: 5,
                ..
            }
        );
    }
}
