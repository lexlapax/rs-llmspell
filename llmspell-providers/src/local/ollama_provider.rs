//! Ollama provider combining rig inference + ollama-rs management

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing::{debug, info};

use crate::abstraction::{ProviderCapabilities, ProviderInstance};
use llmspell_core::types::{AgentInput, AgentOutput, AgentStream};
use llmspell_core::error::LLMSpellError;

use super::{LocalProviderInstance, OllamaModelManager, HealthStatus, LocalModel,
            PullProgress, ModelSpec, ModelInfo};

/// Ollama provider using rig for inference, ollama-rs for management
pub struct OllamaProvider {
    rig_provider: Arc<Box<dyn ProviderInstance>>,  // Rig handles inference
    manager: OllamaModelManager,                    // ollama-rs handles models
}

impl OllamaProvider {
    pub fn new(
        rig_provider: Box<dyn ProviderInstance>,
        base_url: impl Into<String>,
    ) -> Self {
        info!("Creating OllamaProvider with rig + ollama-rs hybrid");
        let manager = OllamaModelManager::new(base_url);
        debug!("OllamaProvider initialized");

        Self {
            rig_provider: Arc::new(rig_provider),
            manager,
        }
    }
}

#[async_trait]
impl ProviderInstance for OllamaProvider {
    async fn complete(&self, input: &AgentInput) -> Result<AgentOutput, LLMSpellError> {
        info!("OllamaProvider delegating completion to rig");
        // Delegate to rig provider
        self.rig_provider.complete(input).await
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        self.rig_provider.capabilities()
    }

    async fn complete_streaming(&self, input: &AgentInput) -> Result<AgentStream, LLMSpellError> {
        info!("OllamaProvider delegating streaming completion to rig");
        self.rig_provider.complete_streaming(input).await
    }

    async fn validate(&self) -> Result<(), LLMSpellError> {
        info!("OllamaProvider validating rig provider");
        self.rig_provider.validate().await
    }

    fn name(&self) -> &str {
        self.rig_provider.name()
    }

    fn model(&self) -> &str {
        self.rig_provider.model()
    }
}

#[async_trait]
impl LocalProviderInstance for OllamaProvider {
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("OllamaProvider health check");
        self.manager.health_check().await
    }

    async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
        debug!("OllamaProvider listing models");
        self.manager.list_local_models().await
    }

    async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress> {
        info!("OllamaProvider pulling model: {:?}", spec);
        self.manager.pull_model(spec).await
    }

    async fn model_info(&self, model_id: &str) -> Result<ModelInfo> {
        debug!("OllamaProvider getting model info: {}", model_id);
        self.manager.model_info(model_id).await
    }

    async fn unload_model(&self, _model_id: &str) -> Result<()> {
        // Ollama manages model loading internally, nothing to do
        Ok(())
    }
}
