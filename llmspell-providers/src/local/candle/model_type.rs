//! Model architecture detection and classification
//!
//! Supports multiple model architectures with different file formats.

use anyhow::{anyhow, Result};
use std::path::Path;

/// Supported model architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelArchitecture {
    /// LLaMA-family models (quantized GGUF format)
    /// Includes: LLaMA, TinyLlama, Mistral, Phi, Gemma, Qwen
    /// Normalization: RMS-norm (Metal support: BLOCKED)
    LLaMA,

    /// T5 encoder-decoder models (safetensors format)
    /// Includes: T5, FLAN-T5, UL2, MADLAD400
    /// Normalization: LayerNorm (Metal support: WORKING)
    T5,
}

impl ModelArchitecture {
    /// Detect architecture from model directory/file
    ///
    /// Detection logic:
    /// - GGUF file present → LLaMA
    /// - Safetensors + config.json → T5 (check config for architecture)
    ///
    /// # Arguments
    /// * `model_path` - Path to model file or directory
    ///
    /// # Returns
    /// * `Ok(ModelArchitecture)` - Detected architecture
    /// * `Err(anyhow::Error)` - Could not detect or unsupported
    pub fn detect(model_path: &Path) -> Result<Self> {
        // Check for GGUF file
        if Self::has_gguf_file(model_path)? {
            return Ok(ModelArchitecture::LLaMA);
        }

        // Check for safetensors + config.json
        if Self::has_safetensors_and_config(model_path)? {
            // Verify architecture from config.json
            return Self::detect_from_config(model_path);
        }

        Err(anyhow!(
            "Could not detect model architecture from: {:?}\n\
            Expected either:\n\
            - GGUF file (*.gguf) for LLaMA models\n\
            - Safetensors (*.safetensors) + config.json for T5 models",
            model_path
        ))
    }

    fn has_gguf_file(path: &Path) -> Result<bool> {
        let search_path = if path.is_file() {
            path.parent().ok_or_else(|| anyhow!("No parent dir"))?
        } else {
            path
        };

        for entry in std::fs::read_dir(search_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("gguf") {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn has_safetensors_and_config(path: &Path) -> Result<bool> {
        let search_path = if path.is_file() {
            path.parent().ok_or_else(|| anyhow!("No parent dir"))?
        } else {
            path
        };

        let mut has_safetensors = false;
        let mut has_config = false;

        for entry in std::fs::read_dir(search_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().unwrap_or("");

            if file_name_str.ends_with(".safetensors") {
                has_safetensors = true;
            }
            if file_name_str == "config.json" {
                has_config = true;
            }
        }

        Ok(has_safetensors && has_config)
    }

    fn detect_from_config(model_path: &Path) -> Result<Self> {
        let config_path = if model_path.is_dir() {
            model_path.join("config.json")
        } else {
            model_path
                .parent()
                .ok_or_else(|| anyhow!("No parent dir"))?
                .join("config.json")
        };

        let config_str = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&config_str)?;

        let model_type = config
            .get("model_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No model_type in config.json"))?;

        match model_type {
            "t5" => Ok(ModelArchitecture::T5),
            other => Err(anyhow!(
                "Unsupported model architecture: '{}'\n\
                Currently supported: llama (GGUF), t5 (safetensors)",
                other
            )),
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ModelArchitecture::LLaMA => "LLaMA",
            ModelArchitecture::T5 => "T5",
        }
    }

    /// Check if architecture supports Metal GPU
    pub fn supports_metal(&self) -> bool {
        match self {
            ModelArchitecture::LLaMA => false, // Blocked by missing RMS-norm
            ModelArchitecture::T5 => true,     // LayerNorm fully implemented
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_names() {
        assert_eq!(ModelArchitecture::LLaMA.name(), "LLaMA");
        assert_eq!(ModelArchitecture::T5.name(), "T5");
    }

    #[test]
    fn test_metal_support() {
        assert!(!ModelArchitecture::LLaMA.supports_metal());
        assert!(ModelArchitecture::T5.supports_metal());
    }
}
