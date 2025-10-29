//! Document ingestion and processing flow

use anyhow::Result;
use llmspell_core::state::StateScope;
use llmspell_storage::vector_storage::{VectorEntry, VectorStorage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    chunking::{
        tokenizer::TokenCounterFactory, ChunkingStrategy, DocumentChunk, SlidingWindowChunker,
    },
    embeddings::{EmbeddingCache, EmbeddingFactory},
};

use super::config::IngestionConfig;

/// Document processor that handles chunking and embedding generation
pub struct DocumentProcessor {
    /// Ingestion configuration
    config: IngestionConfig,

    /// Embedding factory for generating vectors
    embedding_factory: Arc<EmbeddingFactory>,

    /// Chunking strategy
    chunker: Box<dyn ChunkingStrategy>,
}

impl std::fmt::Debug for DocumentProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentProcessor")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl DocumentProcessor {
    /// Create a new document processor
    #[must_use]
    pub fn new(config: IngestionConfig, embedding_factory: Arc<EmbeddingFactory>) -> Self {
        // Create chunker with tokenizer for the embedding model
        let tokenizer = TokenCounterFactory::for_model(&config.embedding.model);
        let chunker = Box::new(SlidingWindowChunker::new().with_tokenizer(tokenizer));

        Self {
            config,
            embedding_factory,
            chunker,
        }
    }

    /// Process a document into chunks with embeddings
    ///
    /// # Errors
    ///
    /// Returns an error if chunking or embedding generation fails
    pub async fn process_document(
        &self,
        document_id: String,
        content: String,
        metadata: Option<serde_json::Value>,
        scope: Option<StateScope>,
    ) -> Result<ProcessedDocument> {
        debug!(
            "Processing document: {} ({} chars)",
            document_id,
            content.len()
        );

        // Extract metadata
        let doc_metadata = self.extract_metadata(&content, metadata.as_ref());

        // Chunk the document
        let chunks = self.chunker.chunk(&content, &self.config.chunking).await?;
        info!(
            "Document {} chunked into {} pieces",
            document_id,
            chunks.len()
        );

        // Deduplicate if configured
        let chunks = if self.config.deduplicate {
            Self::deduplicate_chunks(chunks)
        } else {
            chunks
        };

        // Generate embeddings
        let model = self.embedding_factory.create_model()?;
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = model.embed(&chunk_texts).await?;

        // Combine chunks with their embeddings
        let mut processed_chunks = Vec::with_capacity(chunks.len());
        for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
            let mut chunk_metadata = doc_metadata.clone();

            // Add chunk-specific metadata
            chunk_metadata.insert(
                "document_id".to_string(),
                serde_json::Value::String(document_id.clone()),
            );
            chunk_metadata.insert(
                "chunk_index".to_string(),
                serde_json::Value::Number(chunk.chunk_index.into()),
            );
            chunk_metadata.insert(
                "byte_offset".to_string(),
                serde_json::Value::Number(chunk.byte_offset.into()),
            );
            chunk_metadata.insert(
                "token_count".to_string(),
                serde_json::Value::Number(chunk.token_count.into()),
            );

            if self.config.store_text {
                chunk_metadata.insert(
                    "text".to_string(),
                    serde_json::Value::String(chunk.content.clone()),
                );
            }

            // Merge with any existing chunk metadata
            if let serde_json::Value::Object(chunk_meta) = chunk.metadata {
                for (key, value) in chunk_meta {
                    chunk_metadata.insert(key, value);
                }
            }

            processed_chunks.push(ProcessedChunk {
                id: Uuid::new_v4().to_string(),
                content: chunk.content,
                embedding,
                metadata: chunk_metadata,
                scope: scope.clone(),
            });
        }

        Ok(ProcessedDocument {
            id: document_id,
            chunks: processed_chunks,
            metadata: doc_metadata,
            scope,
        })
    }

    /// Extract metadata from document content and user-provided metadata
    fn extract_metadata(
        &self,
        content: &str,
        user_metadata: Option<&serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();

        // Basic content statistics
        metadata.insert(
            "content_length".to_string(),
            serde_json::Value::Number(content.len().into()),
        );
        metadata.insert(
            "estimated_tokens".to_string(),
            serde_json::Value::Number(self.chunker.estimate_tokens(content).into()),
        );
        metadata.insert(
            "ingested_at".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );

        // Line and paragraph counts
        let line_count = content.lines().count();
        let paragraph_count = content.split("\n\n").count();
        metadata.insert(
            "line_count".to_string(),
            serde_json::Value::Number(line_count.into()),
        );
        metadata.insert(
            "paragraph_count".to_string(),
            serde_json::Value::Number(paragraph_count.into()),
        );

        // Merge user-provided metadata
        if let Some(serde_json::Value::Object(user_meta)) = user_metadata {
            for (key, value) in user_meta {
                metadata.insert(key.clone(), value.clone());
            }
        }

        // TODO: Run additional metadata extractors if configured
        // for extractor in &self.config.metadata_extractors {
        //     match extractor.as_str() {
        //         "language" => { /* detect language */ }
        //         "sentiment" => { /* analyze sentiment */ }
        //         "entities" => { /* extract entities */ }
        //         _ => warn!("Unknown metadata extractor: {}", extractor),
        //     }
        // }

        metadata
    }

    /// Deduplicate chunks by content hash
    fn deduplicate_chunks(chunks: Vec<DocumentChunk>) -> Vec<DocumentChunk> {
        use std::collections::HashSet;

        let mut seen_hashes = HashSet::new();
        let mut unique_chunks = Vec::new();

        let original_len = chunks.len();
        for chunk in chunks {
            let hash = Self::hash_content(&chunk.content);
            if seen_hashes.insert(hash) {
                unique_chunks.push(chunk);
            } else {
                debug!(
                    "Skipping duplicate chunk with {} chars",
                    chunk.content.len()
                );
            }
        }

        if unique_chunks.len() < original_len {
            info!(
                "Deduplicated {} chunks down to {}",
                original_len,
                unique_chunks.len()
            );
        }

        unique_chunks
    }

    /// Generate content hash for deduplication
    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

/// Ingestion flow that stores processed documents
pub struct IngestionFlow {
    /// Vector storage backend
    storage: Arc<dyn VectorStorage>,

    /// Embedding cache for potential cache warming
    cache: Arc<EmbeddingCache>,

    /// Configuration
    config: IngestionConfig,
}

impl std::fmt::Debug for IngestionFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestionFlow")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl IngestionFlow {
    /// Create new ingestion flow
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        cache: Arc<EmbeddingCache>,
        config: IngestionConfig,
    ) -> Self {
        Self {
            storage,
            cache,
            config,
        }
    }

    /// Ingest a processed document into storage
    ///
    /// # Errors
    ///
    /// Returns an error if storage operations fail
    pub async fn ingest_processed_document(
        &self,
        document: ProcessedDocument,
    ) -> Result<IngestionResult> {
        debug!(
            "Ingesting processed document: {} with {} chunks",
            document.id,
            document.chunks.len()
        );

        // Convert processed chunks to vector entries
        let mut vector_entries = Vec::with_capacity(document.chunks.len());
        for chunk in &document.chunks {
            let entry = VectorEntry::new(chunk.id.clone(), chunk.embedding.clone())
                .with_metadata(chunk.metadata.clone())
                .with_scope(chunk.scope.clone().unwrap_or(StateScope::Global));

            vector_entries.push(entry);

            // Cache the embedding for potential future use
            let cache_key = EmbeddingCache::generate_key(&chunk.content);
            if let Err(e) = self.cache.put(cache_key, chunk.embedding.clone()) {
                warn!("Failed to cache embedding: {}", e);
            }
        }

        // Store in vector database
        let stored_ids = self.storage.insert(vector_entries).await?;

        info!(
            "Successfully stored {} vector entries for document {}",
            stored_ids.len(),
            document.id
        );

        Ok(IngestionResult {
            document_id: document.id,
            chunks_stored: stored_ids.len(),
            vector_ids: stored_ids,
            total_tokens: document
                .chunks
                .iter()
                .map(|c| {
                    c.metadata
                        .get("token_count")
                        .and_then(serde_json::Value::as_u64)
                        .and_then(|v| v.try_into().ok())
                        .unwrap_or(0)
                })
                .sum(),
        })
    }
}

/// A document that has been processed (chunked and embedded)
#[derive(Debug, Clone)]
pub struct ProcessedDocument {
    /// Document ID
    pub id: String,

    /// Processed chunks with embeddings
    pub chunks: Vec<ProcessedChunk>,

    /// Document-level metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Optional scope for multi-tenancy
    pub scope: Option<StateScope>,
}

/// A processed chunk with its embedding
#[derive(Debug, Clone)]
pub struct ProcessedChunk {
    /// Unique chunk ID
    pub id: String,

    /// Chunk text content
    pub content: String,

    /// Generated embedding vector
    pub embedding: Vec<f32>,

    /// Chunk metadata (includes document info)
    pub metadata: HashMap<String, serde_json::Value>,

    /// Optional scope for multi-tenancy
    pub scope: Option<StateScope>,
}

/// Result of document ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionResult {
    /// Original document ID
    pub document_id: String,

    /// Number of chunks successfully stored
    pub chunks_stored: usize,

    /// Vector database IDs of stored chunks
    pub vector_ids: Vec<String>,

    /// Total token count across all chunks
    pub total_tokens: usize,
}

/// Document metadata extracted during processing
pub type DocumentMetadata = HashMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::{EmbeddingProviderConfig, EmbeddingProviderType};

    fn create_test_processor() -> DocumentProcessor {
        let config = IngestionConfig {
            embedding: EmbeddingProviderConfig {
                provider_type: EmbeddingProviderType::HuggingFace,
                model: "test-model".to_string(),
                dimensions: Some(384),
                ..Default::default()
            },
            ..Default::default()
        };

        let embedding_factory = Arc::new(EmbeddingFactory::new(config.embedding.clone()));

        DocumentProcessor::new(config, embedding_factory)
    }

    #[tokio::test]
    async fn test_document_processing() {
        let processor = create_test_processor();

        let result = processor
            .process_document(
                "test-doc".to_string(),
                "This is a test document with some content.".to_string(),
                Some(serde_json::json!({"source": "test"})),
                Some(StateScope::Global),
            )
            .await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.id, "test-doc");
        assert!(!doc.chunks.is_empty());
    }

    #[test]
    fn test_metadata_extraction() {
        let processor = create_test_processor();
        let content = "Line 1\nLine 2\n\nParagraph 2";

        let metadata = processor.extract_metadata(content, None);

        assert!(metadata.contains_key("content_length"));
        assert!(metadata.contains_key("line_count"));
        assert!(metadata.contains_key("paragraph_count"));
    }

    #[test]
    fn test_deduplication() {
        let _processor = create_test_processor();

        let chunks = vec![
            DocumentChunk::new("same content".to_string(), 0, 2, 0),
            DocumentChunk::new("same content".to_string(), 10, 2, 1),
            DocumentChunk::new("different content".to_string(), 20, 2, 2),
        ];

        let unique = DocumentProcessor::deduplicate_chunks(chunks);
        assert_eq!(unique.len(), 2); // Should deduplicate the first two
    }
}
