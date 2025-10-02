//! HuggingFace model downloader for GGUF models
//!
//! Downloads GGUF models and tokenizers from HuggingFace Hub.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use super::super::{DownloadStatus, PullProgress};

/// HuggingFace model downloader
pub struct HFDownloader {
    /// HuggingFace API client
    api: hf_hub::api::sync::Api,
}

impl HFDownloader {
    /// Create new HuggingFace downloader
    ///
    /// Reads HFHUB_API_KEY from environment if available.
    pub fn new() -> Result<Self> {
        info!("Initializing HuggingFace downloader");

        // Try to get API key from environment
        let api = if let Ok(token) = std::env::var("HFHUB_API_KEY") {
            info!("Using HuggingFace API key from environment");
            hf_hub::api::sync::ApiBuilder::new()
                .with_token(Some(token))
                .build()
                .map_err(|e| anyhow!("Failed to create HF API client: {}", e))?
        } else {
            warn!("No HFHUB_API_KEY found, using anonymous access");
            hf_hub::api::sync::Api::new()
                .map_err(|e| anyhow!("Failed to create HF API client: {}", e))?
        };

        Ok(Self { api })
    }

    /// Download GGUF model from HuggingFace
    ///
    /// # Arguments
    /// * `repo_id` - HuggingFace repository ID (e.g., "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF")
    /// * `filename` - GGUF filename in the repo (e.g., "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf")
    /// * `dest_dir` - Destination directory for downloaded files
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to downloaded GGUF file
    /// * `Err(anyhow::Error)` - Download failed
    pub fn download_model(
        &self,
        repo_id: &str,
        filename: &str,
        dest_dir: &Path,
    ) -> Result<PathBuf> {
        info!(
            "Downloading model from HF: repo={}, file={}",
            repo_id, filename
        );

        // Create destination directory
        std::fs::create_dir_all(dest_dir)?;

        // Get repository
        let repo = self.api.model(repo_id.to_string());

        // Download GGUF file
        info!("Downloading GGUF file: {}", filename);
        let gguf_path = repo
            .get(filename)
            .map_err(|e| anyhow!("Failed to download GGUF file '{}': {}", filename, e))?;

        debug!("GGUF downloaded to cache: {:?}", gguf_path);

        // Copy to destination directory
        let dest_gguf = dest_dir.join(filename);
        std::fs::copy(&gguf_path, &dest_gguf)?;
        info!("GGUF copied to: {:?}", dest_gguf);

        // Try to download tokenizer.json (common filename)
        if let Ok(tokenizer_path) = repo.get("tokenizer.json") {
            let dest_tokenizer = dest_dir.join("tokenizer.json");
            std::fs::copy(&tokenizer_path, &dest_tokenizer)?;
            info!("Tokenizer downloaded to: {:?}", dest_tokenizer);
        } else {
            warn!(
                "tokenizer.json not found in repo {}, you may need to provide it manually",
                repo_id
            );
        }

        Ok(dest_gguf)
    }

    /// Download model with progress tracking
    ///
    /// # Arguments
    /// * `repo_id` - HuggingFace repository ID
    /// * `filename` - GGUF filename in the repo
    /// * `dest_dir` - Destination directory
    /// * `model_id` - Model identifier for progress tracking
    ///
    /// # Returns
    /// * `Ok(PullProgress)` - Download progress
    /// * `Err(anyhow::Error)` - Download failed
    pub fn download_with_progress(
        &self,
        repo_id: &str,
        filename: &str,
        dest_dir: &Path,
        model_id: &str,
    ) -> Result<PullProgress> {
        // Start download
        info!("Starting download with progress: {}", model_id);

        // Download model
        let _path = self.download_model(repo_id, filename, dest_dir)?;

        // Get file size
        let dest_gguf = dest_dir.join(filename);
        let metadata = std::fs::metadata(&dest_gguf)?;
        let file_size = metadata.len();

        // Return completion progress
        Ok(PullProgress {
            model_id: model_id.to_string(),
            status: DownloadStatus::Complete,
            percent_complete: 100.0,
            bytes_downloaded: file_size,
            bytes_total: Some(file_size),
        })
    }
}

/// Common HuggingFace repository mappings for popular models
pub struct HFModelRepo;

impl HFModelRepo {
    /// Get HuggingFace repo and filename for common model names
    ///
    /// # Arguments
    /// * `model_name` - Simple model name (e.g., "tinyllama", "phi-2")
    /// * `quantization` - Quantization format (e.g., "Q4_K_M", "Q5_K_M")
    ///
    /// # Returns
    /// * `Some((repo_id, filename))` - HF repo and GGUF filename
    /// * `None` - Unknown model
    pub fn get_repo_info(model_name: &str, quantization: &str) -> Option<(&'static str, String)> {
        match model_name.to_lowercase().as_str() {
            "tinyllama" | "tinyllama-1.1b" => Some((
                "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
                format!("tinyllama-1.1b-chat-v1.0.{}.gguf", quantization),
            )),
            "phi-2" => Some((
                "TheBloke/phi-2-GGUF",
                format!("phi-2.{}.gguf", quantization),
            )),
            "qwen2-0.5b" => Some((
                "Qwen/Qwen2-0.5B-Instruct-GGUF",
                format!("qwen2-0_5b-instruct-{}.gguf", quantization.to_lowercase()),
            )),
            _ => {
                debug!("Unknown model name: {}", model_name);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hf_model_repo_tinyllama() {
        let (repo, filename) = HFModelRepo::get_repo_info("tinyllama", "Q4_K_M").unwrap();
        assert_eq!(repo, "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF");
        assert!(filename.contains("tinyllama"));
        assert!(filename.contains("Q4_K_M"));
    }

    #[test]
    fn test_hf_model_repo_phi2() {
        let (repo, filename) = HFModelRepo::get_repo_info("phi-2", "Q5_K_M").unwrap();
        assert_eq!(repo, "TheBloke/phi-2-GGUF");
        assert!(filename.contains("phi-2"));
    }

    #[test]
    fn test_hf_model_repo_unknown() {
        let result = HFModelRepo::get_repo_info("unknown-model", "Q4_K_M");
        assert!(result.is_none());
    }

    // Note: Actual download tests require network and are in integration tests
}
