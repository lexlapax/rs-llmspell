//! GGUF file loading for Candle provider
//!
//! Loads GGUF model files and extracts metadata using Candle 0.9 API.

use anyhow::{anyhow, Result};
use candle_core::quantized::gguf_file;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use tracing::{debug, info, trace, warn};

/// Metadata extracted from GGUF file
#[derive(Debug, Clone)]
pub struct GGUFMetadata {
    /// Model architecture (e.g., "llama")
    pub architecture: String,
    /// Number of attention heads
    pub attention_head_count: usize,
    /// Number of KV attention heads (for GQA/MQA)
    pub attention_head_count_kv: usize,
    /// Number of transformer layers
    pub block_count: usize,
    /// Hidden dimension size
    pub embedding_length: usize,
    /// RoPE dimension
    pub rope_dimension_count: usize,
    /// RoPE frequency base
    pub rope_freq_base: f32,
    /// RMS norm epsilon
    pub rms_norm_epsilon: f64,
    /// Context length (if specified)
    pub context_length: Option<usize>,
    /// Model name (if specified)
    pub model_name: Option<String>,
    /// Quantization format
    pub quantization: Option<String>,
}

impl GGUFMetadata {
    /// Extract metadata from GGUF content
    fn from_content(content: &gguf_file::Content) -> Result<Self> {
        debug!("Extracting GGUF metadata");

        // Helper to get metadata value
        let md_get = |key: &str| -> Result<&gguf_file::Value> {
            content
                .metadata
                .get(key)
                .ok_or_else(|| anyhow!("Missing required metadata key: {}", key))
        };

        // Extract architecture
        let architecture = md_get("general.architecture")?.to_string()?.clone();

        trace!("GGUF architecture: {}", architecture);

        // Verify it's LLaMA architecture
        if architecture != "llama" {
            return Err(anyhow!(
                "Unsupported architecture: {}. Only 'llama' is currently supported.",
                architecture
            ));
        }

        // Extract required LLaMA metadata
        let attention_head_count = md_get("llama.attention.head_count")?.to_u32()? as usize;

        let attention_head_count_kv = md_get("llama.attention.head_count_kv")?.to_u32()? as usize;

        let block_count = md_get("llama.block_count")?.to_u32()? as usize;

        let embedding_length = md_get("llama.embedding_length")?.to_u32()? as usize;

        let rope_dimension_count = md_get("llama.rope.dimension_count")?.to_u32()? as usize;

        // Optional metadata with defaults
        let rope_freq_base = content
            .metadata
            .get("llama.rope.freq_base")
            .and_then(|v| v.to_f32().ok())
            .unwrap_or(10000.0);

        let rms_norm_epsilon = md_get("llama.attention.layer_norm_rms_epsilon")?.to_f32()? as f64;

        let context_length = content
            .metadata
            .get("llama.context_length")
            .and_then(|v| v.to_u32().ok())
            .map(|v| v as usize);

        let model_name = content
            .metadata
            .get("general.name")
            .and_then(|v| v.to_string().ok())
            .cloned();

        // Detect quantization from tensor info
        let quantization = detect_quantization(&content.tensor_infos);

        info!(
            "GGUF metadata: architecture={}, blocks={}, heads={}/{}, dim={}, quant={:?}",
            architecture,
            block_count,
            attention_head_count,
            attention_head_count_kv,
            embedding_length,
            quantization
        );

        Ok(Self {
            architecture,
            attention_head_count,
            attention_head_count_kv,
            block_count,
            embedding_length,
            rope_dimension_count,
            rope_freq_base,
            rms_norm_epsilon,
            context_length,
            model_name,
            quantization,
        })
    }
}

/// GGUF file loader
#[derive(Debug)]
pub struct GGUFLoader {
    /// Loaded GGUF content
    content: gguf_file::Content,
    /// Extracted metadata
    metadata: GGUFMetadata,
    /// File handle (kept open for tensor loading)
    file: File,
}

impl GGUFLoader {
    /// Load GGUF file from path
    ///
    /// # Arguments
    /// * `path` - Path to .gguf file
    ///
    /// # Returns
    /// * `Ok(GGUFLoader)` - Loaded GGUF file with metadata
    /// * `Err(anyhow::Error)` - File not found, invalid GGUF, or unsupported format
    pub fn load(path: &Path) -> Result<Self> {
        info!("Loading GGUF file: {:?}", path);

        // Verify file exists and has .gguf extension
        if !path.exists() {
            return Err(anyhow!("GGUF file not found: {:?}", path));
        }

        if path.extension().and_then(|s| s.to_str()) != Some("gguf") {
            warn!("File does not have .gguf extension: {:?}", path);
        }

        // Open file
        let mut file = File::open(path)?;

        // Read GGUF content (metadata + tensor info)
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| anyhow!("Failed to read GGUF file {:?}: {}", path, e))?;

        debug!(
            "GGUF content loaded: {} metadata keys, {} tensors",
            content.metadata.len(),
            content.tensor_infos.len()
        );

        // Extract metadata
        let metadata = GGUFMetadata::from_content(&content)?;

        Ok(Self {
            content,
            metadata,
            file,
        })
    }

    /// Get reference to GGUF content
    pub fn content(&self) -> &gguf_file::Content {
        &self.content
    }

    /// Get mutable reference to GGUF content
    pub fn content_mut(&mut self) -> &mut gguf_file::Content {
        &mut self.content
    }

    /// Get reference to file (for tensor loading)
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Get reference to extracted metadata
    pub fn metadata(&self) -> &GGUFMetadata {
        &self.metadata
    }

    /// Consume loader and return content and file for model loading
    pub fn into_parts(self) -> (gguf_file::Content, File, GGUFMetadata) {
        (self.content, self.file, self.metadata)
    }
}

/// Detect quantization type from tensor info
fn detect_quantization(tensor_infos: &HashMap<String, gguf_file::TensorInfo>) -> Option<String> {
    // Sample a few key tensors to determine quantization
    let sample_tensors = ["token_embd.weight", "blk.0.attn_q.weight", "output.weight"];

    for tensor_name in &sample_tensors {
        if let Some(info) = tensor_infos.get(*tensor_name) {
            let quant_str = format!("{:?}", info.ggml_dtype);
            trace!("Tensor {} uses dtype: {}", tensor_name, quant_str);
            return Some(quant_str);
        }
    }

    warn!("Could not detect quantization type from tensor info");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gguf_loader_nonexistent_file() {
        let result = GGUFLoader::load(Path::new("/nonexistent/model.gguf"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // Note: Real GGUF loading tests require a test .gguf file
    // These will be added in Task 11.7.10 (Integration Testing)
}
