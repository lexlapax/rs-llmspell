//! HuggingFace model downloader for GGUF and Safetensors models
//!
//! Downloads GGUF models (LLaMA) and Safetensors models (T5) from HuggingFace Hub.

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
        let dest_tokenizer = dest_dir.join("tokenizer.json");

        if let Ok(tokenizer_path) = repo.get("tokenizer.json") {
            // GGUF repo has tokenizer (rare but possible)
            std::fs::copy(&tokenizer_path, &dest_tokenizer)?;
            info!("Tokenizer downloaded from GGUF repo: {:?}", dest_tokenizer);
        } else {
            // GGUF repo lacks tokenizer - try original model repo
            info!(
                "tokenizer.json not in GGUF repo {}, trying original repo",
                repo_id
            );

            // Extract model name and get original repo
            if let Some(model_name) = HFModelRepo::extract_model_name(repo_id) {
                if let Some(original_repo) = HFModelRepo::get_original_repo(model_name) {
                    info!(
                        "Downloading tokenizer from original repo: {}",
                        original_repo
                    );

                    // Download tokenizer directly via HTTP
                    // NOTE: Using direct HTTP instead of hf-hub API due to API state issues when
                    // calling model() multiple times in the same function
                    let tokenizer_url = format!(
                        "https://huggingface.co/{}/resolve/main/tokenizer.json",
                        original_repo
                    );
                    debug!("Tokenizer URL: {}", tokenizer_url);

                    let response = ureq::get(&tokenizer_url).call().map_err(|e| {
                        anyhow!("Failed to download tokenizer from {}: {}", tokenizer_url, e)
                    })?;

                    let mut file = std::fs::File::create(&dest_tokenizer)?;
                    std::io::copy(&mut response.into_reader(), &mut file)?;

                    info!(
                        "Tokenizer downloaded from original repo: {:?}",
                        dest_tokenizer
                    );
                } else {
                    warn!(
                        "No original repo mapping for model {}, tokenizer must be provided manually",
                        model_name
                    );
                }
            } else {
                warn!(
                    "Cannot extract model name from GGUF repo {}, tokenizer must be provided manually",
                    repo_id
                );
            }
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

    /// Download Safetensors model from HuggingFace
    ///
    /// Downloads all required files for a Safetensors model:
    /// - model.safetensors (or model-*.safetensors for sharded models)
    /// - config.json
    /// - tokenizer.json (optional but recommended)
    ///
    /// # Arguments
    /// * `repo_id` - HuggingFace repository ID (e.g., "google/flan-t5-small")
    /// * `dest_dir` - Destination directory for downloaded files
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to destination directory
    /// * `Err(anyhow::Error)` - Download failed
    pub fn download_safetensors_model(&self, repo_id: &str, dest_dir: &Path) -> Result<PathBuf> {
        info!("Downloading safetensors model from HF: repo={}", repo_id);

        // Create destination directory
        std::fs::create_dir_all(dest_dir)?;

        // Get repository
        let repo = self.api.model(repo_id.to_string());

        // Download config.json (required)
        info!("Downloading config.json");
        let config_path = repo
            .get("config.json")
            .map_err(|e| anyhow!("Failed to download config.json: {}", e))?;
        let dest_config = dest_dir.join("config.json");
        std::fs::copy(&config_path, &dest_config)?;
        info!("config.json copied to: {:?}", dest_config);

        // Download tokenizer.json (highly recommended)
        if let Ok(tokenizer_path) = repo.get("tokenizer.json") {
            let dest_tokenizer = dest_dir.join("tokenizer.json");
            std::fs::copy(&tokenizer_path, &dest_tokenizer)?;
            info!("tokenizer.json copied to: {:?}", dest_tokenizer);
        } else {
            warn!("tokenizer.json not found in repo {}", repo_id);

            // Try tokenizer_config.json as fallback
            if let Ok(tokenizer_config_path) = repo.get("tokenizer_config.json") {
                let dest_tokenizer_config = dest_dir.join("tokenizer_config.json");
                std::fs::copy(&tokenizer_config_path, &dest_tokenizer_config)?;
                info!(
                    "tokenizer_config.json copied to: {:?}",
                    dest_tokenizer_config
                );
            }
        }

        // Download safetensors files
        // Try single-file model first (model.safetensors)
        if let Ok(model_path) = repo.get("model.safetensors") {
            let dest_model = dest_dir.join("model.safetensors");
            std::fs::copy(&model_path, &dest_model)?;
            info!("model.safetensors copied to: {:?}", dest_model);
        } else {
            // Try sharded model (model-00001-of-*.safetensors, etc.)
            info!("Single model.safetensors not found, trying sharded model files");

            // Try to download model index file first
            let mut shard_index = 1;
            let mut downloaded_shards = Vec::new();

            loop {
                // Try direct download with common naming patterns for sharded models
                let common_names = vec![
                    format!("model-{:05}-of-00002.safetensors", shard_index),
                    format!("model-{:05}-of-00003.safetensors", shard_index),
                    format!("model-{:05}-of-00004.safetensors", shard_index),
                    format!("model_{:05}.safetensors", shard_index),
                    format!("pytorch_model-{:05}-of-00002.safetensors", shard_index),
                ];

                let mut shard_found = false;

                for name in &common_names {
                    if let Ok(shard_path) = repo.get(name) {
                        let dest_shard = dest_dir.join(name);
                        std::fs::copy(&shard_path, &dest_shard)?;
                        info!("Downloaded shard: {:?}", dest_shard);
                        downloaded_shards.push(dest_shard);
                        shard_found = true;
                        break;
                    }
                }

                if !shard_found {
                    // No more shards found
                    break;
                }

                shard_index += 1;
            }

            if downloaded_shards.is_empty() {
                return Err(anyhow!(
                    "No safetensors files found in repo {}. Expected model.safetensors or sharded model files",
                    repo_id
                ));
            }

            info!("Downloaded {} shard(s)", downloaded_shards.len());
        }

        info!("Safetensors model download complete: {:?}", dest_dir);
        Ok(dest_dir.to_path_buf())
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

    /// Get original model repo for tokenizer download
    ///
    /// GGUF repos (e.g., TheBloke/*-GGUF) often lack tokenizer.json.
    /// This maps to the original model repo which has all required files.
    ///
    /// # Arguments
    /// * `model_name` - Simple model name (e.g., "tinyllama", "phi-2")
    ///
    /// # Returns
    /// * `Some(repo_id)` - Original HuggingFace repo with tokenizer
    /// * `None` - Unknown model or no mapping
    pub fn get_original_repo(model_name: &str) -> Option<&'static str> {
        match model_name.to_lowercase().as_str() {
            "tinyllama" | "tinyllama-1.1b" => Some("TinyLlama/TinyLlama-1.1B-Chat-v1.0"),
            "phi-2" => Some("microsoft/phi-2"),
            "qwen2-0.5b" => Some("Qwen/Qwen2-0.5B-Instruct"),
            _ => {
                debug!("No original repo mapping for: {}", model_name);
                None
            }
        }
    }

    /// Extract model name from GGUF repo ID
    ///
    /// # Arguments
    /// * `gguf_repo_id` - GGUF repository ID (e.g., "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF")
    ///
    /// # Returns
    /// * `Some(model_name)` - Extracted model name for original repo lookup
    /// * `None` - Cannot extract model name
    pub fn extract_model_name(gguf_repo_id: &str) -> Option<&'static str> {
        // Map GGUF repo IDs to model names
        match gguf_repo_id {
            "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF" => Some("tinyllama"),
            "TheBloke/phi-2-GGUF" => Some("phi-2"),
            "Qwen/Qwen2-0.5B-Instruct-GGUF" => Some("qwen2-0.5b"),
            _ => {
                debug!("Cannot extract model name from GGUF repo: {}", gguf_repo_id);
                None
            }
        }
    }

    /// Get HuggingFace repo for T5 Safetensors models
    ///
    /// Maps simple model names to their HuggingFace repository IDs for T5 models.
    /// T5 models use safetensors format and don't have quantization variants.
    ///
    /// # Arguments
    /// * `model_name` - Simple model name (e.g., "flan-t5-small", "t5-base")
    ///
    /// # Returns
    /// * `Some(repo_id)` - HuggingFace repository ID
    /// * `None` - Unknown model
    pub fn get_t5_repo_info(model_name: &str) -> Option<&'static str> {
        match model_name.to_lowercase().as_str() {
            "flan-t5-small" => Some("google/flan-t5-small"),
            "flan-t5-base" => Some("google/flan-t5-base"),
            "flan-t5-large" => Some("google/flan-t5-large"),
            "t5-small" => Some("t5-small"),
            "t5-base" => Some("t5-base"),
            "t5-large" => Some("t5-large"),
            _ => {
                debug!("Unknown T5 model name: {}", model_name);
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

    #[test]
    fn test_get_original_repo_tinyllama() {
        let repo = HFModelRepo::get_original_repo("tinyllama").unwrap();
        assert_eq!(repo, "TinyLlama/TinyLlama-1.1B-Chat-v1.0");
    }

    #[test]
    fn test_get_original_repo_phi2() {
        let repo = HFModelRepo::get_original_repo("phi-2").unwrap();
        assert_eq!(repo, "microsoft/phi-2");
    }

    #[test]
    fn test_get_original_repo_unknown() {
        let result = HFModelRepo::get_original_repo("unknown-model");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_model_name_tinyllama() {
        let name =
            HFModelRepo::extract_model_name("TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF").unwrap();
        assert_eq!(name, "tinyllama");
    }

    #[test]
    fn test_extract_model_name_phi2() {
        let name = HFModelRepo::extract_model_name("TheBloke/phi-2-GGUF").unwrap();
        assert_eq!(name, "phi-2");
    }

    #[test]
    fn test_extract_model_name_unknown() {
        let result = HFModelRepo::extract_model_name("SomeRepo/UnknownModel-GGUF");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_t5_repo_info_flan_t5_small() {
        let repo = HFModelRepo::get_t5_repo_info("flan-t5-small").unwrap();
        assert_eq!(repo, "google/flan-t5-small");
    }

    #[test]
    fn test_get_t5_repo_info_flan_t5_base() {
        let repo = HFModelRepo::get_t5_repo_info("flan-t5-base").unwrap();
        assert_eq!(repo, "google/flan-t5-base");
    }

    #[test]
    fn test_get_t5_repo_info_t5_small() {
        let repo = HFModelRepo::get_t5_repo_info("t5-small").unwrap();
        assert_eq!(repo, "t5-small");
    }

    #[test]
    fn test_get_t5_repo_info_case_insensitive() {
        let repo = HFModelRepo::get_t5_repo_info("FLAN-T5-SMALL").unwrap();
        assert_eq!(repo, "google/flan-t5-small");
    }

    #[test]
    fn test_get_t5_repo_info_unknown() {
        let result = HFModelRepo::get_t5_repo_info("unknown-t5-model");
        assert!(result.is_none());
    }

    // Note: Actual download tests require network and are in integration tests
}
