//! Tokenizer loading for Candle provider
//!
//! Loads HuggingFace tokenizers from tokenizer.json files.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};

/// Tokenizer loader for GGUF models
pub struct TokenizerLoader {
    tokenizer: Tokenizer,
}

impl TokenizerLoader {
    /// Load tokenizer from file
    ///
    /// Searches for tokenizer.json in the following locations:
    /// 1. Exact path if provided
    /// 2. Same directory as model file
    /// 3. Model directory root
    ///
    /// # Arguments
    /// * `model_path` - Path to model directory or GGUF file
    ///
    /// # Returns
    /// * `Ok(TokenizerLoader)` - Loaded tokenizer
    /// * `Err(anyhow::Error)` - Tokenizer file not found or invalid
    pub fn load(model_path: &Path) -> Result<Self> {
        info!("Loading tokenizer for model: {:?}", model_path);

        // Determine search paths
        let search_paths = Self::find_tokenizer_paths(model_path);

        // Try each path
        for path in &search_paths {
            debug!("Searching for tokenizer at: {:?}", path);
            if path.exists() {
                info!("Found tokenizer at: {:?}", path);
                let tokenizer = Tokenizer::from_file(path)
                    .map_err(|e| anyhow!("Failed to load tokenizer from {:?}: {}", path, e))?;
                return Ok(Self { tokenizer });
            }
        }

        // No tokenizer found
        Err(anyhow!(
            "Tokenizer file not found. Searched paths: {:?}",
            search_paths
        ))
    }

    /// Find potential tokenizer.json locations
    fn find_tokenizer_paths(model_path: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Determine if path looks like a file or directory
        // Check if it has an extension (file-like) or ends with .gguf
        let has_file_extension = model_path.extension().is_some();
        let is_gguf_file = model_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "gguf")
            .unwrap_or(false);

        // If model_path looks like a file (has extension), check same directory
        if has_file_extension {
            if let Some(parent) = model_path.parent() {
                paths.push(parent.join("tokenizer.json"));
            }
        }

        // If model_path looks like a directory (no extension), check inside
        if !has_file_extension {
            paths.push(model_path.join("tokenizer.json"));
        }

        // If model_path is a .gguf file nested in subdirectory, also check grandparent
        if is_gguf_file {
            if let Some(parent) = model_path.parent() {
                if let Some(grandparent) = parent.parent() {
                    // Add grandparent path if not already added
                    let gp_path = grandparent.join("tokenizer.json");
                    if !paths.contains(&gp_path) {
                        paths.push(gp_path);
                    }
                }
            }
        }

        paths
    }

    /// Get reference to underlying tokenizer
    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    /// Encode text to token IDs
    ///
    /// # Arguments
    /// * `text` - Input text to tokenize
    /// * `add_special_tokens` - Whether to add BOS/EOS tokens
    ///
    /// # Returns
    /// * `Ok(Vec<u32>)` - Token IDs
    /// * `Err(anyhow::Error)` - Encoding error
    pub fn encode(&self, text: &str, add_special_tokens: bool) -> Result<Vec<u32>> {
        let encoding = self
            .tokenizer
            .encode(text, add_special_tokens)
            .map_err(|e| anyhow!("Failed to encode text: {}", e))?;
        Ok(encoding.get_ids().to_vec())
    }

    /// Decode token IDs to text
    ///
    /// # Arguments
    /// * `ids` - Token IDs to decode
    /// * `skip_special_tokens` - Whether to skip BOS/EOS tokens
    ///
    /// # Returns
    /// * `Ok(String)` - Decoded text
    /// * `Err(anyhow::Error)` - Decoding error
    pub fn decode(&self, ids: &[u32], skip_special_tokens: bool) -> Result<String> {
        self.tokenizer
            .decode(ids, skip_special_tokens)
            .map_err(|e| anyhow!("Failed to decode tokens: {}", e))
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    /// Get EOS token ID if available
    pub fn eos_token_id(&self) -> Option<u32> {
        // Try to get EOS token ID from tokenizer
        // Different models use different EOS token strings
        let eos_candidates = ["</s>", "<|endoftext|>", "<|im_end|>", "</|endoftext|>"];

        for candidate in &eos_candidates {
            if let Some(id) = self.tokenizer.token_to_id(candidate) {
                debug!("Found EOS token '{}' with ID: {}", candidate, id);
                return Some(id);
            }
        }

        warn!("Could not determine EOS token ID, using default heuristic");
        // Common LLaMA EOS token ID
        Some(2)
    }

    /// Get BOS token ID if available
    pub fn bos_token_id(&self) -> Option<u32> {
        // Try to get BOS token ID from tokenizer
        let bos_candidates = ["<s>", "<|startoftext|>", "<|im_start|>"];

        for candidate in &bos_candidates {
            if let Some(id) = self.tokenizer.token_to_id(candidate) {
                debug!("Found BOS token '{}' with ID: {}", candidate, id);
                return Some(id);
            }
        }

        warn!("Could not determine BOS token ID");
        Some(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_loader_nonexistent() {
        let result = TokenizerLoader::load(Path::new("/nonexistent/model"));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_tokenizer_paths_for_file() {
        let model_path = PathBuf::from("/path/to/model/model.gguf");
        let paths = TokenizerLoader::find_tokenizer_paths(&model_path);
        assert!(paths.contains(&PathBuf::from("/path/to/model/tokenizer.json")));
    }

    #[test]
    fn test_find_tokenizer_paths_for_dir() {
        let model_path = PathBuf::from("/path/to/model");
        let paths = TokenizerLoader::find_tokenizer_paths(&model_path);
        assert!(paths.contains(&PathBuf::from("/path/to/model/tokenizer.json")));
    }

    // Note: Real tokenizer tests require a test tokenizer.json file
    // These will be added in Task 11.7.10 (Integration Testing)
}
