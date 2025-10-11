//! Model wrapper for Candle models
//!
//! Provides high-level interface for model loading and inference.
//! Supports multiple model architectures: GGUF LLaMA and Safetensors T5.

use anyhow::{anyhow, Result};
use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::{quantized_llama, t5};
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{debug, info};

use super::gguf_loader::{GGUFLoader, GGUFMetadata};
use super::model_type::ModelArchitecture;
use super::tokenizer_loader::TokenizerLoader;

/// Wrapper around Candle models supporting multiple architectures
pub enum ModelWrapper {
    /// LLaMA-family models (GGUF quantized format)
    LLaMA {
        /// Underlying Candle model (boxed to reduce enum size)
        model: Box<quantized_llama::ModelWeights>,
        /// Tokenizer (boxed to reduce enum size)
        tokenizer: Box<TokenizerLoader>,
        /// Model metadata from GGUF
        metadata: GGUFMetadata,
        /// Device model is loaded on
        device: Device,
    },

    /// T5 encoder-decoder models (Safetensors format)
    T5 {
        /// Underlying T5 model (boxed to reduce enum size)
        model: Box<t5::T5ForConditionalGeneration>,
        /// Tokenizer
        tokenizer: Box<Tokenizer>,
        /// Model configuration
        config: t5::Config,
        /// Device model is loaded on
        device: Device,
    },
}

impl ModelWrapper {
    /// Load model from file or directory
    ///
    /// Auto-detects architecture based on file contents:
    /// - GGUF files → LLaMA models
    /// - Safetensors + config.json → T5 models
    ///
    /// # Arguments
    /// * `model_path` - Path to model file or directory
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

        // Detect architecture
        let architecture = ModelArchitecture::detect(model_path)?;
        info!("Detected {} architecture", architecture.name());

        // Dispatch to appropriate loader
        match architecture {
            ModelArchitecture::LLaMA => Self::load_llama(model_path, device),
            ModelArchitecture::T5 => Self::load_t5(model_path, device),
        }
    }

    /// Load LLaMA model from GGUF file
    fn load_llama(model_path: &Path, device: Device) -> Result<Self> {
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
            Self::estimate_param_count_llama(&metadata)
        );

        Ok(ModelWrapper::LLaMA {
            model: Box::new(model),
            tokenizer: Box::new(tokenizer),
            metadata,
            device,
        })
    }

    /// Load T5 model from Safetensors files
    fn load_t5(model_path: &Path, device: Device) -> Result<Self> {
        // Determine model directory
        let model_dir = if model_path.is_dir() {
            model_path
        } else {
            model_path
                .parent()
                .ok_or_else(|| anyhow!("No parent directory for model file"))?
        };

        info!("Loading T5 model from: {:?}", model_dir);

        // Find safetensors files
        let safetensors_files = Self::find_safetensors_files(model_dir)?;
        info!("Found {} safetensors file(s)", safetensors_files.len());

        // Load config.json
        let config_path = model_dir.join("config.json");
        if !config_path.exists() {
            return Err(anyhow!("config.json not found in {:?}", model_dir));
        }

        info!("Loading config from: {:?}", config_path);
        let config_str = std::fs::read_to_string(&config_path)?;
        let mut config: t5::Config = serde_json::from_str(&config_str)
            .map_err(|e| anyhow!("Failed to parse config.json: {}", e))?;

        // Enable KV cache for better performance
        config.use_cache = true;

        info!(
            "T5 config loaded: vocab_size={}, d_model={}, layers={}",
            config.vocab_size, config.d_model, config.num_layers
        );

        // Create VarBuilder from safetensors
        // Use memory-mapped loading for efficiency
        let dtype = DType::F32; // T5 uses F32 by default
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&safetensors_files, dtype, &device)? };

        info!("Initializing T5ForConditionalGeneration on {:?}", device);

        // Load model weights
        let model = t5::T5ForConditionalGeneration::load(vb, &config)
            .map_err(|e| anyhow!("Failed to load T5 model weights: {}", e))?;

        info!("T5 model loaded successfully");

        // Load tokenizer
        let tokenizer = Self::load_t5_tokenizer(model_dir)?;

        Ok(ModelWrapper::T5 {
            model: Box::new(model),
            tokenizer: Box::new(tokenizer),
            config,
            device,
        })
    }

    /// Find all safetensors files in directory
    fn find_safetensors_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "safetensors")
                .unwrap_or(false)
            {
                debug!("Found safetensors file: {:?}", path);
                files.push(path);
            }
        }

        if files.is_empty() {
            return Err(anyhow!(
                "No safetensors files found in directory: {:?}",
                dir
            ));
        }

        // Sort for deterministic loading order
        files.sort();

        Ok(files)
    }

    /// Load T5 tokenizer
    fn load_t5_tokenizer(model_dir: &Path) -> Result<Tokenizer> {
        // Try tokenizer.json first (standard HuggingFace format)
        let tokenizer_path = model_dir.join("tokenizer.json");
        if tokenizer_path.exists() {
            info!("Loading tokenizer from: {:?}", tokenizer_path);
            return Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| anyhow!("Failed to load tokenizer: {}", e));
        }

        // Try spiece.model (SentencePiece format, used by some T5 models)
        let spiece_path = model_dir.join("spiece.model");
        if spiece_path.exists() {
            info!("Loading SentencePiece tokenizer from: {:?}", spiece_path);
            // Note: Would need to use tokenizers::models::unigram::Unigram
            // For now, return error with helpful message
            return Err(anyhow!(
                "SentencePiece tokenizer format not yet supported.\n\
                Please ensure model has tokenizer.json file.\n\
                Found: {:?}",
                spiece_path
            ));
        }

        Err(anyhow!(
            "No tokenizer file found in {:?}\n\
            Expected: tokenizer.json or spiece.model",
            model_dir
        ))
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

    /// Get reference to LLaMA model
    ///
    /// # Panics
    /// Panics if called on non-LLaMA model variant
    pub fn llama_model(&mut self) -> &mut quantized_llama::ModelWeights {
        match self {
            ModelWrapper::LLaMA { model, .. } => model,
            ModelWrapper::T5 { .. } => panic!("llama_model() called on T5 model"),
        }
    }

    /// Get reference to LLaMA tokenizer
    ///
    /// # Panics
    /// Panics if called on non-LLaMA model variant
    pub fn tokenizer(&self) -> &TokenizerLoader {
        match self {
            ModelWrapper::LLaMA { tokenizer, .. } => tokenizer,
            ModelWrapper::T5 { .. } => panic!("tokenizer() called on T5 model - use t5_tokenizer()"),
        }
    }

    /// Get reference to T5 model
    ///
    /// # Panics
    /// Panics if called on non-T5 model variant
    pub fn t5_model(&mut self) -> &mut t5::T5ForConditionalGeneration {
        match self {
            ModelWrapper::T5 { model, .. } => model,
            ModelWrapper::LLaMA { .. } => panic!("t5_model() called on LLaMA model"),
        }
    }

    /// Get reference to T5 tokenizer
    ///
    /// # Panics
    /// Panics if called on non-T5 model variant
    pub fn t5_tokenizer(&self) -> &Tokenizer {
        match self {
            ModelWrapper::T5 { tokenizer, .. } => tokenizer,
            ModelWrapper::LLaMA { .. } => panic!("t5_tokenizer() called on LLaMA model"),
        }
    }

    /// Get reference to T5 config
    ///
    /// # Panics
    /// Panics if called on non-T5 model variant
    pub fn t5_config(&self) -> &t5::Config {
        match self {
            ModelWrapper::T5 { config, .. } => config,
            ModelWrapper::LLaMA { .. } => panic!("t5_config() called on LLaMA model"),
        }
    }

    /// Get reference to GGUF metadata (LLaMA only)
    ///
    /// # Panics
    /// Panics if called on non-LLaMA model variant
    pub fn metadata(&self) -> &GGUFMetadata {
        match self {
            ModelWrapper::LLaMA { metadata, .. } => metadata,
            ModelWrapper::T5 { .. } => panic!("metadata() called on T5 model"),
        }
    }

    /// Get device model is loaded on
    pub fn device(&self) -> &Device {
        match self {
            ModelWrapper::LLaMA { device, .. } => device,
            ModelWrapper::T5 { device, .. } => device,
        }
    }

    /// Get model architecture type
    pub fn architecture(&self) -> ModelArchitecture {
        match self {
            ModelWrapper::LLaMA { .. } => ModelArchitecture::LLaMA,
            ModelWrapper::T5 { .. } => ModelArchitecture::T5,
        }
    }

    /// Estimate parameter count from LLaMA metadata
    fn estimate_param_count_llama(metadata: &GGUFMetadata) -> String {
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

        let count = ModelWrapper::estimate_param_count_llama(&metadata);
        assert!(count.contains("B") || count.contains("M"));
    }

    #[test]
    fn test_architecture_detection() {
        // Test would require actual model files
        // Will be added in integration tests
    }

    // Note: Real model loading tests require test model files
    // These will be added in Task 11.7.10 (Integration Testing)
}
