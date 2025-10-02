//! Ollama model management using ollama-rs
//!
//! This module handles model operations (list, pull, info) via direct
//! ollama-rs client, while inference goes through rig.

use anyhow::{anyhow, Result};
use ollama_rs::Ollama;
use tracing::{debug, error, info, trace, warn};

use super::{DownloadStatus, HealthStatus, LocalModel, ModelInfo, ModelSpec, PullProgress};

/// Manager for Ollama model operations (not inference)
pub struct OllamaModelManager {
    client: Ollama,
    base_url: String,
}

impl OllamaModelManager {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        info!("Initializing OllamaModelManager: {}", base_url);

        // Parse base URL to extract host and port
        let url = url::Url::parse(&base_url)
            .unwrap_or_else(|_| url::Url::parse("http://localhost:11434").unwrap());

        let host = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or(11434);

        let client = Ollama::new(host, port);
        debug!("Ollama client created");

        Self { client, base_url }
    }

    pub async fn health_check(&self) -> Result<HealthStatus> {
        info!("Checking Ollama server health");
        trace!("Sending health check request to {}", self.base_url);

        match self.client.list_local_models().await {
            Ok(models) => {
                let count = models.len();
                debug!("Ollama healthy: {} models available", count);

                // Try to get version if available
                let version = None; // ollama-rs doesn't expose version yet

                Ok(HealthStatus::Healthy {
                    available_models: count,
                    version,
                })
            }
            Err(e) => {
                warn!("Ollama health check failed: {}", e);
                Ok(HealthStatus::Unhealthy {
                    reason: format!("Server not responding: {}", e),
                })
            }
        }
    }

    pub async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
        info!("Listing Ollama local models");
        trace!("Querying Ollama API for model list");

        let models = self.client.list_local_models().await.map_err(|e| {
            error!("Failed to list Ollama models: {}", e);
            anyhow!("Ollama list failed: {}", e)
        })?;

        debug!("Found {} Ollama models", models.len());

        let local_models = models
            .into_iter()
            .map(|m| {
                trace!("Processing model: {}", m.name);
                // Parse modified_at string to SystemTime if possible
                let modified_at = chrono::DateTime::parse_from_rfc3339(&m.modified_at)
                    .ok()
                    .map(std::time::SystemTime::from);

                LocalModel {
                    id: m.name.clone(),
                    backend: "ollama".to_string(),
                    size_bytes: m.size,
                    quantization: None, // Ollama doesn't expose this
                    modified_at,
                }
            })
            .collect();

        info!("Ollama model list complete");
        Ok(local_models)
    }

    pub async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress> {
        let model_name = format!(
            "{}:{}",
            spec.model,
            spec.variant.as_deref().unwrap_or("latest")
        );

        info!("Pulling Ollama model: {}", model_name);
        debug!("Model spec: {:?}", spec);

        // Start pull (ollama-rs provides progress streaming)
        trace!("Initiating Ollama pull request");

        self.client
            .pull_model(model_name.clone(), false)
            .await
            .map_err(|e| {
                error!("Ollama pull failed for {}: {}", model_name, e);
                anyhow!("Pull failed: {}", e)
            })?;

        info!("Ollama model pull complete: {}", model_name);

        Ok(PullProgress {
            model_id: model_name,
            status: DownloadStatus::Complete,
            percent_complete: 100.0,
            bytes_downloaded: 0, // ollama-rs doesn't provide this
            bytes_total: None,
        })
    }

    pub async fn model_info(&self, model_id: &str) -> Result<ModelInfo> {
        info!("Getting Ollama model info: {}", model_id);
        trace!("Querying Ollama API for model details");

        let info = self
            .client
            .show_model_info(model_id.to_string())
            .await
            .map_err(|e| {
                error!("Failed to get model info for {}: {}", model_id, e);
                anyhow!("Model info failed: {}", e)
            })?;

        debug!("Model info retrieved for {}", model_id);

        // Extract size from model_info map if available
        let size_bytes = info
            .model_info
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(ModelInfo {
            id: model_id.to_string(),
            backend: "ollama".to_string(),
            size_bytes,
            parameter_count: Some(info.parameters.clone()),
            quantization: None,
            format: "Ollama".to_string(),
            loaded: false, // Ollama manages this internally
        })
    }
}
