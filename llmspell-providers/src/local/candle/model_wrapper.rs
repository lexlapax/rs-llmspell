//! Model wrapper for Candle quantized LLaMA models
//!
//! Provides high-level interface for GGUF model loading and inference.

use anyhow::{anyhow, Result};
use candle_core::Device;
use candle_transformers::models::quantized_llama;
use std::path::Path;
use tracing::{debug, info};

use super::gguf_loader::{GGUFLoader, GGUFMetadata};
use super::tokenizer_loader::TokenizerLoader;

/// Wrapper around Candle's quantized LLaMA model
pub struct ModelWrapper {
    /// Underlying Candle model
    model: quantized_llama::ModelWeights,
    /// Tokenizer
    tokenizer: TokenizerLoader,
    /// Model metadata
    metadata: GGUFMetadata,
    /// Device model is loaded on
    device: Device,
}

impl ModelWrapper {
    /// Load model from GGUF file
    ///
    /// # Arguments
    /// * `model_path` - Path to .gguf file or model directory
    /// * `device` - Device to load model on (CPU/CUDA/Metal)
    ///
    /// # Returns
    /// * `Ok(ModelWrapper)` - Loaded model ready for inference
    /// * `Err(anyhow::Error)` - Model loading failed
    pub fn load(model_path: &Path, device: Device) -> Result<Self> {
        info!(
            "Loading model from: {:?} on device: {:?}",
            model_path, device
        );

        // Determine GGUF file path
        let gguf_path = if model_path.is_file() {
            model_path.to_path_buf()
        } else if model_path.is_dir() {
            // Find .gguf file in directory
            Self::find_gguf_in_dir(model_path)?
        } else {
            return Err(anyhow!("Model path does not exist: {:?}", model_path));
        };

        info!("Loading GGUF file: {:?}", gguf_path);

        // Load GGUF file
        let gguf_loader = GGUFLoader::load(&gguf_path)?;
        let metadata = gguf_loader.metadata().clone();

        // Load tokenizer (searches same directory)
        let tokenizer = TokenizerLoader::load(&gguf_path)?;

        info!("Initializing quantized LLaMA model on {:?}", device);

        // Load model weights using Candle's quantized_llama
        let (content, mut file, _) = gguf_loader.into_parts();
        let model = quantized_llama::ModelWeights::from_gguf(content, &mut file, &device)
            .map_err(|e| anyhow!("Failed to load model weights: {}", e))?;

        info!(
            "Model loaded successfully: {} blocks, {} params",
            metadata.block_count,
            Self::estimate_param_count(&metadata)
        );

        Ok(Self {
            model,
            tokenizer,
            metadata,
            device,
        })
    }

    /// Find GGUF file in directory
    fn find_gguf_in_dir(dir: &Path) -> Result<std::path::PathBuf> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                debug!("Found GGUF file: {:?}", path);
                return Ok(path);
            }
        }
        Err(anyhow!("No GGUF file found in directory: {:?}", dir))
    }

    /// Get reference to model
    pub fn model(&mut self) -> &mut quantized_llama::ModelWeights {
        &mut self.model
    }

    /// Get reference to tokenizer
    pub fn tokenizer(&self) -> &TokenizerLoader {
        &self.tokenizer
    }

    /// Get reference to metadata
    pub fn metadata(&self) -> &GGUFMetadata {
        &self.metadata
    }

    /// Get device model is loaded on
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Estimate parameter count from metadata
    fn estimate_param_count(metadata: &GGUFMetadata) -> String {
        // Rough estimation: embedding_dim * layers * heads
        // LLaMA 7B: 4096 * 32 layers ≈ 7B
        // LLaMA 13B: 5120 * 40 layers ≈ 13B
        let approx_params = metadata.embedding_length * metadata.block_count * 1_000_000;

        if approx_params > 1_000_000_000 {
            format!("~{}B", approx_params / 1_000_000_000)
        } else {
            format!("~{}M", approx_params / 1_000_000)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_wrapper_nonexistent_path() {
        let device = Device::Cpu;
        let result = ModelWrapper::load(Path::new("/nonexistent/model.gguf"), device);
        assert!(result.is_err());
    }

    #[test]
    fn test_estimate_param_count() {
        let metadata = GGUFMetadata {
            architecture: "llama".to_string(),
            attention_head_count: 32,
            attention_head_count_kv: 8,
            block_count: 32,
            embedding_length: 4096,
            rope_dimension_count: 128,
            rope_freq_base: 10000.0,
            rms_norm_epsilon: 1e-5,
            context_length: Some(2048),
            model_name: Some("test-model".to_string()),
            quantization: Some("Q4_K_M".to_string()),
        };

        let count = ModelWrapper::estimate_param_count(&metadata);
        assert!(count.contains("B") || count.contains("M"));
    }

    // Note: Real model loading tests require a test .gguf file
    // These will be added in Task 11.7.10 (Integration Testing)
}
