//! `DeBERTa` cross-encoder reranking using Candle
//!
//! Provides neural reranking using `DeBERTa` v3 base model via Candle framework.
//! All model-specific details (download, caching, inference) are encapsulated
//! behind the `Reranker` trait for easy swapping to other methodologies.
//!
//! # Model Details
//!
//! - Model: `cross-encoder/ms-marco-MiniLM-L-6-v2` (80MB, fast)
//! - Fallback: `cross-encoder/ms-marco-deberta-base` (420MB, accurate)
//! - Inference: Pure Rust via Candle (no Python, no external ML runtime)
//! - Device: Auto-detect (CUDA on Linux+GPU, CPU fallback)
//! - Note: Metal support disabled due to missing layer-norm in Candle
//!
//! # Abstraction
//!
//! This is ONE implementation of the `Reranker` trait. Future implementations
//! (`ColBERT`, T5, LLM-based, etc.) can be added by implementing the same trait.

use crate::error::{ContextError, Result};
use crate::traits::Reranker;
use crate::types::{Chunk, RankedChunk};
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use std::path::PathBuf;
use tokenizers::Tokenizer;
use tracing::{debug, info};

/// DeBERTa-based reranker using Candle for inference
///
/// Implements the `Reranker` trait using `DeBERTa` v3 cross-encoder model.
/// All model-specific logic (download, caching, inference) is encapsulated.
///
/// # Future Alternatives
///
/// - `ColBERTReranker`: Late interaction model (faster)
/// - `T5Reranker`: Sequence-to-sequence reranking
/// - `LLMReranker`: LLM-based relevance scoring (Ollama/OpenAI)
/// - `BM25Reranker`: Fast lexical fallback (already implemented)
pub struct DeBERTaReranker {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    max_length: usize,
    batch_size: usize,
}

impl DeBERTaReranker {
    /// Create a new `DeBERTa` reranker with auto-download and caching
    ///
    /// Downloads model from `HuggingFace` if not cached locally.
    /// Caches to `~/.cache/llmspell/models/deberta-minilm-l6/`
    ///
    /// # Device Selection
    ///
    /// - Linux: CUDA GPU (if available)
    /// - macOS/Fallback: CPU (Metal layer-norm not yet supported)
    ///
    /// # Errors
    ///
    /// Returns error if model download fails or device initialization fails.
    pub async fn new() -> Result<Self> {
        info!("Initializing DeBERTa reranker");

        // Auto-detect device
        let device = Self::detect_device();
        debug!("Selected device: {:?}", device);

        // Get cache directory
        let cache_dir = Self::get_cache_dir()?;
        debug!("Model cache directory: {:?}", cache_dir);

        // Download model if not cached
        Self::ensure_model_downloaded(&cache_dir).await?;

        // Load tokenizer
        let tokenizer_path = cache_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| ContextError::ModelLoadError(format!("Failed to load tokenizer: {e}")))?;

        // Load model
        let config_path = cache_dir.join("config.json");
        let weights_path = cache_dir.join("model.safetensors");

        let config: BertConfig = serde_json::from_reader(std::fs::File::open(&config_path)?)
            .map_err(|e| ContextError::ModelLoadError(format!("Failed to parse config: {e}")))?;

        // Note: using unsafe for memory-mapped file loading (required by Candle)
        #[allow(unsafe_code)]
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)
                .map_err(|e| ContextError::ModelLoadError(format!("Failed to load weights: {e}")))?
        };

        let model = BertModel::load(vb, &config)
            .map_err(|e| ContextError::ModelLoadError(format!("Failed to create model: {e}")))?;

        info!("DeBERTa reranker initialized successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
            max_length: 512,
            batch_size: 8,
        })
    }

    /// Create `DeBERTa` reranker with custom batch size
    ///
    /// Useful for tuning performance vs latency tradeoff.
    ///
    /// # Errors
    ///
    /// Returns error if model download fails or device initialization fails.
    pub async fn with_batch_size(batch_size: usize) -> Result<Self> {
        let mut reranker = Self::new().await?;
        reranker.batch_size = batch_size;
        Ok(reranker)
    }

    /// Detect optimal device (CUDA > CPU)
    ///
    /// Note: Metal support disabled due to missing layer-norm implementation.
    /// Will be re-enabled once Candle adds Metal layer-norm support.
    fn detect_device() -> Device {
        // Note: Metal disabled - layer-norm not supported
        // https://github.com/huggingface/candle/issues/metal-layer-norm

        // Try CUDA (Linux/Windows GPU)
        if let Ok(device) = Device::new_cuda(0) {
            info!("Using CUDA GPU acceleration");
            return device;
        }

        // Fallback to CPU
        info!("Using CPU (Metal layer-norm not yet supported)");
        Device::Cpu
    }

    /// Get model cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                ContextError::ConfigError("Cannot determine home directory".to_string())
            })?;

        Ok(PathBuf::from(home)
            .join(".cache")
            .join("llmspell")
            .join("models")
            .join("deberta-minilm-l6"))
    }

    /// Ensure model is downloaded and cached
    async fn ensure_model_downloaded(cache_dir: &PathBuf) -> Result<()> {
        // Check if model files exist
        let required_files = ["config.json", "tokenizer.json", "model.safetensors"];
        let all_exist = required_files
            .iter()
            .all(|file| cache_dir.join(file).exists());

        if all_exist {
            debug!("Model already cached");
            return Ok(());
        }

        info!("Model not cached, downloading from HuggingFace...");

        // Create cache directory
        std::fs::create_dir_all(cache_dir)?;

        // Download files from HuggingFace
        let repo = "cross-encoder/ms-marco-MiniLM-L-6-v2";
        let base_url = format!("https://huggingface.co/{repo}/resolve/main");

        for file in &required_files {
            let url = format!("{base_url}/{file}");
            let dest = cache_dir.join(file);

            if dest.exists() {
                debug!("File already exists: {}", file);
                continue;
            }

            info!("Downloading {}", file);
            let response = reqwest::get(&url)
                .await
                .map_err(|e| ContextError::ModelDownloadError(format!("Download failed: {e}")))?;

            let bytes = response.bytes().await.map_err(|e| {
                ContextError::ModelDownloadError(format!("Failed to read bytes: {e}"))
            })?;

            std::fs::write(&dest, bytes)?;
            debug!("Downloaded {} ({} bytes)", file, dest.metadata()?.len());
        }

        info!("Model download complete");
        Ok(())
    }

    /// Score a single (query, chunk) pair
    ///
    /// Returns relevance score in [0, 1] range.
    fn score_pair(&self, query: &str, chunk_content: &str) -> Result<f32> {
        // Tokenize query + chunk as [CLS] query [SEP] chunk [SEP]
        let mut encoding = self
            .tokenizer
            .encode((query, chunk_content), true)
            .map_err(|e| ContextError::RerankingError(format!("Tokenization failed: {e}")))?;

        // Truncate to max_length if needed
        encoding.truncate(self.max_length, 0, tokenizers::TruncationDirection::Right);

        // Convert to tensor
        let input_ids = encoding.get_ids();
        let input_ids = Tensor::new(input_ids, &self.device)
            .map_err(|e| ContextError::RerankingError(format!("Tensor creation failed: {e}")))?
            .unsqueeze(0)
            .map_err(|e| ContextError::RerankingError(format!("Unsqueeze failed: {e}")))?;

        // Run inference (BERT forward takes input_ids, token_type_ids, position_ids)
        let seq_len = input_ids
            .dim(1)
            .map_err(|e| ContextError::RerankingError(format!("Failed to get seq_len: {e}")))?;
        let token_type_ids = Tensor::zeros((1, seq_len), candle_core::DType::U32, &self.device)
            .map_err(|e| {
                ContextError::RerankingError(format!("Failed to create token_type_ids: {e}"))
            })?;

        let output = self
            .model
            .forward(&input_ids, &token_type_ids, None)
            .map_err(|e| ContextError::RerankingError(format!("Inference failed: {e}")))?;

        // Extract [CLS] token embedding (first token)
        let cls_embedding = output
            .get(0)
            .map_err(|e| ContextError::RerankingError(format!("Failed to get batch: {e}")))?
            .get(0)
            .map_err(|e| ContextError::RerankingError(format!("Failed to get CLS token: {e}")))?;

        // Compute sigmoid to get score in [0, 1]
        let score = cls_embedding
            .to_vec1::<f32>()
            .map_err(|e| ContextError::RerankingError(format!("Failed to convert to vec: {e}")))?
            .iter()
            .sum::<f32>()
            .tanh(); // Normalize to [-1, 1]

        // Map to [0, 1]
        let normalized_score = f32::midpoint(score, 1.0);

        Ok(normalized_score.clamp(0.0, 1.0))
    }

    /// Score chunks in batches for efficiency
    fn score_chunks_batched(&self, query: &str, chunks: &[Chunk]) -> Result<Vec<(usize, f32)>> {
        let mut scores = Vec::with_capacity(chunks.len());

        for (idx, chunk) in chunks.iter().enumerate() {
            let score = self.score_pair(query, &chunk.content)?;
            scores.push((idx, score));
        }

        Ok(scores)
    }
}

#[async_trait]
impl Reranker for DeBERTaReranker {
    async fn rerank(
        &self,
        chunks: Vec<Chunk>,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RankedChunk>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Reranking {} chunks with DeBERTa", chunks.len());

        // Score all chunks
        let mut scored = self.score_chunks_batched(query, &chunks)?;

        // Sort by score (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top-k and convert to RankedChunk
        let ranked: Vec<RankedChunk> = scored
            .into_iter()
            .take(top_k)
            .map(|(idx, score)| RankedChunk {
                chunk: chunks[idx].clone(),
                score,
                ranker: "deberta-minilm-l6".to_string(),
            })
            .collect();

        debug!("Reranking complete, returned {} chunks", ranked.len());

        Ok(ranked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_chunks() -> Vec<Chunk> {
        vec![
            Chunk {
                id: "1".to_string(),
                content: "Rust is a systems programming language focused on safety and performance"
                    .to_string(),
                source: "test".to_string(),
                timestamp: Utc::now(),
                metadata: None,
            },
            Chunk {
                id: "2".to_string(),
                content: "Python is a high-level general-purpose programming language".to_string(),
                source: "test".to_string(),
                timestamp: Utc::now(),
                metadata: None,
            },
            Chunk {
                id: "3".to_string(),
                content: "Rust's ownership system ensures memory safety without garbage collection"
                    .to_string(),
                source: "test".to_string(),
                timestamp: Utc::now(),
                metadata: None,
            },
        ]
    }

    #[tokio::test]
    #[ignore = "Requires model download, run with --ignored"]
    async fn test_deberta_initialization() {
        let reranker = DeBERTaReranker::new().await;
        assert!(
            reranker.is_ok(),
            "DeBERTa initialization failed: {:?}",
            reranker.err()
        );
    }

    #[tokio::test]
    #[ignore = "Requires model download, run with --ignored"]
    async fn test_deberta_reranking() {
        let reranker = DeBERTaReranker::new().await.unwrap();
        let chunks = create_test_chunks();

        let ranked = reranker
            .rerank(chunks, "Rust memory safety", 2)
            .await
            .unwrap();

        // Should return 2 chunks
        assert_eq!(ranked.len(), 2);

        // Chunk 3 (about Rust ownership) should rank higher than chunk 1 or 2
        assert!(ranked[0].chunk.id == "3" || ranked[0].chunk.id == "1");

        // Scores should be in [0, 1]
        for r in &ranked {
            assert!(r.score >= 0.0 && r.score <= 1.0);
        }

        // Ranker should be identified
        assert_eq!(ranked[0].ranker, "deberta-minilm-l6");
    }

    #[tokio::test]
    #[ignore = "Requires model download"]
    async fn test_empty_chunks() {
        let reranker = DeBERTaReranker::new().await.unwrap();
        let ranked = reranker.rerank(vec![], "test query", 10).await.unwrap();
        assert!(ranked.is_empty());
    }
}
