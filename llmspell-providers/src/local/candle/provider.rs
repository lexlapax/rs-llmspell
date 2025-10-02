//! Candle provider implementation for GGUF model inference
//!
//! Provides local LLM inference using Candle framework with GGUF quantized models.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use candle_core::{Device, IndexOp, Tensor};

use crate::abstraction::{ProviderCapabilities, ProviderInstance};
use llmspell_core::error::LLMSpellError;
use llmspell_core::types::{AgentInput, AgentOutput, AgentStream};

use super::super::{
    DownloadStatus, HealthStatus, LocalModel, LocalProviderInstance, ModelInfo, ModelSpec,
    PullProgress,
};

use super::hf_downloader::{HFDownloader, HFModelRepo};
use super::model_wrapper::ModelWrapper;
use super::sampling::{sample_token, SamplingConfig};

/// Candle provider for embedded GGUF inference
pub struct CandleProvider {
    default_model: String,
    device: Device,
    model_directory: PathBuf,
    capabilities: ProviderCapabilities,
}

impl CandleProvider {
    /// Create new Candle provider
    ///
    /// # Arguments
    /// * `default_model` - Default model identifier (e.g., "llama3.1:8b")
    /// * `model_directory` - Optional directory for model files
    /// * `device_str` - Device selection: "auto", "cpu", "cuda", or "metal"
    pub fn new(
        default_model: String,
        model_directory: Option<PathBuf>,
        device_str: String,
    ) -> Result<Self> {
        info!(
            "Initializing Candle provider: default_model={}, device={}",
            default_model, device_str
        );

        // Device selection
        let device = match device_str.as_str() {
            "cuda" => {
                info!("Using CUDA device for Candle inference");
                Device::cuda_if_available(0).map_err(|e| {
                    error!("CUDA device requested but not available: {}", e);
                    anyhow!("CUDA not available: {}", e)
                })?
            }
            "metal" => {
                info!("Using Metal device for Candle inference");
                Device::new_metal(0).map_err(|e| {
                    error!("Metal device requested but not available: {}", e);
                    anyhow!("Metal not available: {}", e)
                })?
            }
            "cpu" => {
                info!("Using CPU device for Candle inference");
                Device::Cpu
            }
            "auto" => {
                // Try CUDA first, then Metal, then CPU
                if let Ok(cuda) = Device::cuda_if_available(0) {
                    info!("Auto-detected CUDA device for Candle");
                    cuda
                } else if let Ok(metal) = Device::new_metal(0) {
                    info!("Auto-detected Metal device for Candle");
                    metal
                } else {
                    info!("Auto-detected CPU device for Candle (no GPU available)");
                    Device::Cpu
                }
            }
            _ => {
                warn!("Unknown device '{}', defaulting to CPU", device_str);
                Device::Cpu
            }
        };

        debug!("Candle provider using device: {:?}", device);

        // Model directory
        let model_directory = model_directory.unwrap_or_else(|| {
            let home = dirs::home_dir().expect("Could not determine home directory");
            home.join(".llmspell").join("models").join("candle")
        });

        info!("Candle model directory: {:?}", model_directory);

        // Ensure model directory exists
        std::fs::create_dir_all(&model_directory)
            .map_err(|e| anyhow!("Failed to create model directory: {}", e))?;

        let capabilities = ProviderCapabilities {
            supports_streaming: false,
            supports_multimodal: false,
            max_context_tokens: Some(4096),
            max_output_tokens: Some(2048),
            available_models: vec![],
            custom_features: HashMap::new(),
        };

        Ok(Self {
            default_model,
            device,
            model_directory,
            capabilities,
        })
    }

    /// Find GGUF file in model directory
    fn find_gguf_file(&self, model_path: &PathBuf) -> Result<PathBuf> {
        for entry in std::fs::read_dir(model_path)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "gguf" {
                    debug!("Found GGUF file: {:?}", path);
                    return Ok(path);
                }
            }
        }
        Err(anyhow!("No GGUF file found in {:?}", model_path))
    }
}

impl CandleProvider {
    /// Create sampling configuration from agent input
    fn create_sampling_config(&self, input: &AgentInput) -> SamplingConfig {
        let mut config = SamplingConfig::default();

        // Extract parameters
        if let Some(temp) = input.parameters.get("temperature").and_then(|v| v.as_f64()) {
            config.temperature = temp;
        }
        if let Some(top_p) = input.parameters.get("top_p").and_then(|v| v.as_f64()) {
            config.top_p = Some(top_p);
        }
        if let Some(top_k) = input.parameters.get("top_k").and_then(|v| v.as_u64()) {
            config.top_k = Some(top_k as usize);
        }

        config
    }

    /// Generate text using loaded model
    fn generate_with_model(
        &self,
        model_wrapper: &mut ModelWrapper,
        prompt: &str,
        sampling_config: &SamplingConfig,
        max_tokens: usize,
    ) -> Result<String> {
        debug!("Starting generation: max_tokens={}", max_tokens);
        let gen_start = std::time::Instant::now();

        // Tokenize prompt
        let tokenize_start = std::time::Instant::now();
        let prompt_tokens = model_wrapper.tokenizer().encode(prompt, true)?;
        let tokenize_duration = tokenize_start.elapsed();
        info!(
            "Prompt tokenized: {} tokens in {:.2}ms",
            prompt_tokens.len(),
            tokenize_duration.as_secs_f64() * 1000.0
        );

        // Get EOS token
        let eos_token_id = model_wrapper.tokenizer().eos_token_id().unwrap_or(2);

        // Process prompt (index_pos = 0) - FIRST TOKEN LATENCY
        let first_token_start = std::time::Instant::now();
        let prompt_tensor =
            Tensor::new(prompt_tokens.as_slice(), model_wrapper.device())?.unsqueeze(0)?;

        let mut logits = model_wrapper.model().forward(&prompt_tensor, 0)?;
        logits = logits.i((0, prompt_tokens.len() - 1))?;

        let first_token_duration = first_token_start.elapsed();
        info!(
            "First token latency: {:.2}ms ({} prompt tokens)",
            first_token_duration.as_secs_f64() * 1000.0,
            prompt_tokens.len()
        );

        // Generation loop
        let mut generated_tokens = Vec::new();
        let mut all_tokens = prompt_tokens.clone();
        let generation_start = std::time::Instant::now();

        for index_pos in 1..=max_tokens {
            // Sample next token
            let next_token = sample_token(&logits, sampling_config, &all_tokens)?;

            // Check for EOS
            if next_token == eos_token_id {
                debug!("EOS token encountered at position {}", index_pos);
                break;
            }

            generated_tokens.push(next_token);
            all_tokens.push(next_token);

            // Forward pass with single token
            let input_tensor = Tensor::new(&[next_token], model_wrapper.device())?.unsqueeze(0)?;

            logits = model_wrapper.model().forward(&input_tensor, index_pos)?;
            logits = logits.i((0, 0))?;
        }

        let generation_duration = generation_start.elapsed();
        let tokens_per_second = if generation_duration.as_secs_f64() > 0.0 {
            generated_tokens.len() as f64 / generation_duration.as_secs_f64()
        } else {
            0.0
        };

        info!(
            "Generated {} tokens in {:.2}ms ({:.2} tokens/sec)",
            generated_tokens.len(),
            generation_duration.as_secs_f64() * 1000.0,
            tokens_per_second
        );

        // Decode generated tokens
        let decode_start = std::time::Instant::now();
        let generated_text = model_wrapper.tokenizer().decode(&generated_tokens, true)?;
        let decode_duration = decode_start.elapsed();

        let total_duration = gen_start.elapsed();
        info!(
            "Total generation: {:.2}ms (tokenize: {:.2}ms, first token: {:.2}ms, generation: {:.2}ms, decode: {:.2}ms)",
            total_duration.as_secs_f64() * 1000.0,
            tokenize_duration.as_secs_f64() * 1000.0,
            first_token_duration.as_secs_f64() * 1000.0,
            generation_duration.as_secs_f64() * 1000.0,
            decode_duration.as_secs_f64() * 1000.0
        );

        Ok(generated_text)
    }
}

#[async_trait]
impl ProviderInstance for CandleProvider {
    async fn complete(&self, input: &AgentInput) -> Result<AgentOutput, LLMSpellError> {
        info!(
            "CandleProvider completion request for model: {}",
            self.default_model
        );

        // Determine model path
        let model_path = self.model_directory.join(&self.default_model);
        if !model_path.exists() {
            return Err(LLMSpellError::Component {
                message: format!(
                    "Model '{}' not found in directory: {:?}. \
                    Use 'llmspell model pull {}@candle' to download.",
                    self.default_model, self.model_directory, self.default_model
                ),
                source: None,
            });
        }

        // Load model with timing
        let load_start = std::time::Instant::now();
        let mut model_wrapper =
            ModelWrapper::load(&model_path, self.device.clone()).map_err(|e| {
                LLMSpellError::Component {
                    message: format!("Failed to load model: {}", e),
                    source: None,
                }
            })?;
        let load_duration = load_start.elapsed();
        info!("Model loaded in {:.2}s", load_duration.as_secs_f64());

        // Extract prompt from input
        let prompt = &input.text;
        info!(
            "Generating completion for prompt (length: {} chars)",
            prompt.len()
        );

        // Configure sampling from input parameters
        let sampling_config = self.create_sampling_config(input);
        debug!("Using sampling config: {:?}", sampling_config);

        // Get max_tokens from parameters or use default
        let max_tokens = input
            .parameters
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(512);

        // Generate completion
        let generated_text = self
            .generate_with_model(&mut model_wrapper, prompt, &sampling_config, max_tokens)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Generation failed: {}", e),
                source: None,
            })?;

        // Build output
        Ok(AgentOutput::text(generated_text))
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    async fn complete_streaming(&self, _input: &AgentInput) -> Result<AgentStream, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Candle streaming not yet implemented".to_string(),
            source: None,
        })
    }

    async fn validate(&self) -> Result<(), LLMSpellError> {
        info!("CandleProvider validation: checking model directory");
        if !self.model_directory.exists() {
            return Err(LLMSpellError::Configuration {
                message: format!("Model directory does not exist: {:?}", self.model_directory),
                source: None,
            });
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "candle"
    }

    fn model(&self) -> &str {
        &self.default_model
    }

    fn as_local(&self) -> Option<&dyn LocalProviderInstance> {
        Some(self)
    }
}

#[async_trait]
impl LocalProviderInstance for CandleProvider {
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("CandleProvider health check");

        // Check if model directory exists
        if !self.model_directory.exists() {
            return Ok(HealthStatus::Unhealthy {
                reason: format!("Model directory does not exist: {:?}", self.model_directory),
            });
        }

        Ok(HealthStatus::Healthy {
            available_models: 0, // TODO: Count models in directory
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        })
    }

    async fn list_local_models(&self) -> Result<Vec<LocalModel>> {
        debug!("CandleProvider listing models");
        let mut models = Vec::new();

        // Scan model directory
        if !self.model_directory.exists() {
            warn!("Model directory does not exist: {:?}", self.model_directory);
            return Ok(models);
        }

        for entry in std::fs::read_dir(&self.model_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let model_id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow!("Invalid model directory name"))?
                    .to_string();

                // Check for GGUF file
                if self.find_gguf_file(&path).is_ok() {
                    // Get total size of all files in directory
                    let mut total_size = 0u64;
                    for file_entry in std::fs::read_dir(&path)?.flatten() {
                        if let Ok(metadata) = file_entry.metadata() {
                            total_size += metadata.len();
                        }
                    }

                    let metadata = std::fs::metadata(&path)?;

                    models.push(LocalModel {
                        id: model_id,
                        backend: "candle".to_string(),
                        size_bytes: total_size,
                        quantization: Some("Q4_K_M".to_string()), // TODO: Detect from GGUF metadata
                        modified_at: metadata.modified().ok(),
                    });
                }
            }
        }

        info!("Found {} Candle models", models.len());
        Ok(models)
    }

    async fn pull_model(&self, spec: &ModelSpec) -> Result<PullProgress> {
        info!("CandleProvider pulling model: {:?}", spec);

        let model_name = &spec.model;
        let variant = spec.variant.as_deref().unwrap_or("Q4_K_M");

        // Create model directory
        let model_id = format!("{}:{}", model_name, variant);
        let model_dir = self.model_directory.join(&model_id);

        if model_dir.exists() {
            info!("Model {} already exists", model_id);
            // Get actual size
            let mut total_size = 0u64;
            for entry in std::fs::read_dir(&model_dir)?.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
            return Ok(PullProgress {
                model_id: model_id.clone(),
                status: DownloadStatus::Complete,
                percent_complete: 100.0,
                bytes_downloaded: total_size,
                bytes_total: Some(total_size),
            });
        }

        // Try to map to known HuggingFace repositories
        if let Some((repo_id, filename)) = HFModelRepo::get_repo_info(model_name, variant) {
            info!(
                "Downloading from HuggingFace: repo={}, file={}",
                repo_id, filename
            );

            // Create downloader
            let downloader = HFDownloader::new()?;

            // Download with progress
            let progress =
                downloader.download_with_progress(repo_id, &filename, &model_dir, &model_id)?;

            info!("Model {} downloaded successfully", model_id);
            Ok(progress)
        } else {
            // Unknown model - provide instructions for manual download
            Err(anyhow!(
                "Model '{}' not found in known repositories.\n\
                \n\
                For known models, use:\n\
                - tinyllama (TinyLlama-1.1B-Chat)\n\
                - phi-2 (Phi-2)\n\
                - qwen2-0.5b (Qwen2-0.5B-Instruct)\n\
                \n\
                For custom models, download manually:\n\
                1. Download GGUF model from HuggingFace\n\
                2. Place files in: {:?}\n\
                3. Ensure both .gguf and tokenizer.json are present\n\
                \n\
                Alternative: Use Ollama backend:\n\
                llmspell model pull {}@ollama",
                model_name,
                model_dir,
                model_name
            ))
        }
    }

    async fn model_info(&self, model_id: &str) -> Result<ModelInfo> {
        debug!("CandleProvider getting model info: {}", model_id);

        let model_path = self.model_directory.join(model_id);
        if !model_path.exists() {
            return Err(anyhow!("Model {} not found", model_id));
        }

        // Get total size
        let mut total_size = 0u64;
        for entry in std::fs::read_dir(&model_path)?.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }

        Ok(ModelInfo {
            id: model_id.to_string(),
            backend: "candle".to_string(),
            size_bytes: total_size,
            parameter_count: None, // TODO: Extract from GGUF metadata
            quantization: Some("Q4_K_M".to_string()), // TODO: Detect from GGUF
            format: "GGUF".to_string(),
            loaded: false, // Models not kept in memory in current stub
        })
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        debug!("Candle model unload requested for: {}", model_id);
        // No-op in current implementation as models aren't kept loaded
        Ok(())
    }
}
