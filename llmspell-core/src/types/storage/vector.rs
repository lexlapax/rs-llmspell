//! Vector storage types
//!
//! Domain types for vector storage operations including:
//! - `VectorEntry`: Multi-tenant vector with bi-temporal support
//! - `VectorQuery`: Query parameters with temporal filters
//! - `VectorResult`: Search result with similarity score
//! - `StorageStats`: Overall storage metrics
//! - `ScopedStats`: Per-tenant statistics
//! - `DistanceMetric`: Similarity calculation methods
//! - `HNSWConfig`: HNSW algorithm configuration
//! - `NamespaceStats`: Namespace-specific metrics
//!
//! Migrated from llmspell-storage/src/vector_storage.rs as part of Phase 13c.3.

use crate::state::StateScope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::SystemTime;

// ============================================================================
// Core Vector Types
// ============================================================================

/// Multi-tenant aware vector entry with bi-temporal support
///
/// Represents a single vector embedding with associated metadata, scope, and temporal
/// tracking. Supports bi-temporal queries through separate `created_at` (ingestion time)
/// and `event_time` (when the event actually occurred) timestamps.
///
/// # Builder Pattern
///
/// Uses fluent builder methods for ergonomic construction:
/// ```
/// # use llmspell_core::types::storage::vector::VectorEntry;
/// # use llmspell_core::state::StateScope;
/// # use std::collections::HashMap;
/// let entry = VectorEntry::new("doc1".to_string(), vec![0.1, 0.2, 0.3])
///     .with_scope(StateScope::User("alice".to_string()))
///     .with_ttl(3600);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique identifier
    pub id: String,

    /// Embedding vector
    pub embedding: Vec<f32>,

    /// Metadata for filtering and context
    pub metadata: HashMap<String, Value>,

    /// Tenant/user/session binding
    pub scope: StateScope,

    /// Creation timestamp (ingestion time)
    pub created_at: SystemTime,

    /// Last update timestamp
    pub updated_at: SystemTime,

    /// Event time - when the event actually occurred (bi-temporal support)
    /// Different from created_at which is when it was ingested
    pub event_time: Option<SystemTime>,

    /// Optional expiration time for session vectors
    pub expires_at: Option<SystemTime>,

    /// TTL duration in seconds (alternative to expires_at)
    /// If set, expires_at will be calculated as created_at + ttl
    pub ttl_seconds: Option<u64>,

    /// Explicit tenant ID for billing
    pub tenant_id: Option<String>,
}

impl VectorEntry {
    /// Create a new vector entry with default values
    #[must_use]
    pub fn new(id: String, embedding: Vec<f32>) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            embedding,
            metadata: HashMap::new(),
            scope: StateScope::Global,
            created_at: now,
            updated_at: now,
            event_time: None,
            expires_at: None,
            ttl_seconds: None,
            tenant_id: None,
        }
    }

    /// Set the scope for this vector
    #[must_use]
    pub fn with_scope(mut self, scope: StateScope) -> Self {
        self.scope = scope;
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set expiration time
    #[must_use]
    pub const fn with_expiration(mut self, expires_at: SystemTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set event time (when the event actually occurred)
    #[must_use]
    pub const fn with_event_time(mut self, event_time: SystemTime) -> Self {
        self.event_time = Some(event_time);
        self
    }

    /// Set TTL in seconds
    #[must_use]
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        // Calculate expires_at from TTL
        let duration = std::time::Duration::from_secs(ttl_seconds);
        self.expires_at = Some(self.created_at + duration);
        self
    }

    /// Check if the entry has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }

    /// Update the entry (updates the updated_at timestamp)
    pub fn update(&mut self) {
        self.updated_at = SystemTime::now();
    }
}

/// Query parameters for vector search with temporal support
///
/// Configures vector similarity search including the query vector, result count,
/// filters, and temporal constraints. Supports bi-temporal queries through
/// separate event time and ingestion time ranges.
///
/// # Examples
///
/// ```
/// # use llmspell_core::types::storage::vector::VectorQuery;
/// # use llmspell_core::state::StateScope;
/// let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 10)
///     .with_threshold(0.7)
///     .with_scope(StateScope::User("alice".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuery {
    /// Query vector
    pub vector: Vec<f32>,

    /// Number of results to return
    pub k: usize,

    /// Optional metadata filters
    pub filter: Option<HashMap<String, Value>>,

    /// Optional scope restriction
    pub scope: Option<StateScope>,

    /// Similarity threshold (0.0 to 1.0)
    pub threshold: Option<f32>,

    /// Include metadata in results
    pub include_metadata: bool,

    /// Filter by event time range (bi-temporal query)
    pub event_time_range: Option<(SystemTime, SystemTime)>,

    /// Filter by ingestion time range (bi-temporal query)
    pub ingestion_time_range: Option<(SystemTime, SystemTime)>,

    /// Exclude expired entries
    pub exclude_expired: bool,
}

impl VectorQuery {
    /// Create a new vector query
    #[must_use]
    pub const fn new(vector: Vec<f32>, k: usize) -> Self {
        Self {
            vector,
            k,
            filter: None,
            scope: None,
            threshold: None,
            include_metadata: true,
            event_time_range: None,
            ingestion_time_range: None,
            exclude_expired: true,
        }
    }

    /// Set scope filter
    #[must_use]
    pub fn with_scope(mut self, scope: StateScope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Filter by event time range
    #[must_use]
    pub const fn with_event_time_range(mut self, start: SystemTime, end: SystemTime) -> Self {
        self.event_time_range = Some((start, end));
        self
    }

    /// Filter by ingestion time range
    #[must_use]
    pub const fn with_ingestion_time_range(mut self, start: SystemTime, end: SystemTime) -> Self {
        self.ingestion_time_range = Some((start, end));
        self
    }

    /// Set whether to exclude expired entries
    #[must_use]
    pub const fn exclude_expired(mut self, exclude: bool) -> Self {
        self.exclude_expired = exclude;
        self
    }

    /// Add metadata filters
    #[must_use]
    pub fn with_filter(mut self, filter: HashMap<String, Value>) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set similarity threshold
    #[must_use]
    pub const fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

/// Result from vector search
///
/// Contains a matching vector with its similarity score and optional metadata.
/// Results are typically ordered by score (descending - higher is more similar).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorResult {
    /// Vector ID
    pub id: String,

    /// Similarity score (higher is better)
    pub score: f32,

    /// Optional vector data
    pub vector: Option<Vec<f32>>,

    /// Metadata if requested
    pub metadata: Option<HashMap<String, Value>>,

    /// Distance metric value
    pub distance: f32,
}

/// Storage statistics
///
/// Overall metrics for the vector storage system including counts,
/// sizes, and performance data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    /// Total number of vectors
    pub total_vectors: usize,

    /// Total storage size in bytes
    pub storage_bytes: usize,

    /// Number of namespaces/tenants
    pub namespace_count: usize,

    /// Index build time in milliseconds
    pub index_build_time_ms: Option<u64>,

    /// Average query time in milliseconds
    pub avg_query_time_ms: Option<f32>,

    /// Number of dimensions
    pub dimensions: Option<usize>,
}

/// Scoped statistics for a specific tenant/scope
///
/// Detailed metrics for vectors within a particular scope, useful for
/// per-tenant billing, usage tracking, and capacity planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedStats {
    /// Scope identifier
    pub scope: StateScope,

    /// Number of vectors in this scope
    pub vector_count: usize,

    /// Storage used in bytes
    pub storage_bytes: usize,

    /// Number of queries executed
    pub query_count: usize,

    /// Total tokens/embeddings processed
    pub tokens_processed: usize,

    /// Estimated cost in USD
    pub estimated_cost: f64,
}

// ============================================================================
// HNSW Configuration Types
// ============================================================================

/// Distance metrics supported by HNSW
///
/// Different metrics optimize for different similarity concepts:
/// - **Cosine**: Angle-based similarity (most common for embeddings)
/// - **Euclidean**: Geometric distance in vector space
/// - **InnerProduct**: Raw dot product (useful for normalized vectors)
/// - **Manhattan**: L1 distance (sum of absolute differences)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DistanceMetric {
    /// Cosine similarity (most common for embeddings)
    #[default]
    Cosine,

    /// Euclidean (L2) distance
    Euclidean,

    /// Inner product (dot product)
    InnerProduct,

    /// Manhattan (L1) distance
    Manhattan,
}

/// HNSW algorithm configuration
///
/// Configures the Hierarchical Navigable Small World index parameters.
/// Different configurations trade off between speed, accuracy, and memory usage.
///
/// # Presets
///
/// Use the preset methods for common configurations:
/// - `HNSWConfig::fast()`: Optimized for speed (m=12, ef_construction=100)
/// - `HNSWConfig::balanced()`: Good balance (m=16, ef_construction=200)
/// - `HNSWConfig::accurate()`: Optimized for recall (m=32, ef_construction=400)
///
/// # Examples
///
/// ```
/// # use llmspell_core::types::storage::vector::HNSWConfig;
/// // Use a preset
/// let config = HNSWConfig::accurate();
///
/// // Or customize
/// let config = HNSWConfig {
///     m: 24,
///     ef_construction: 300,
///     ef_search: 150,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNSWConfig {
    /// Number of bi-directional links created for each node (16-64 typical)
    /// Higher values give better recall but use more memory
    pub m: usize,

    /// Size of the dynamic candidate list during construction (200 typical)
    /// Higher values give better recall but slower construction
    pub ef_construction: usize,

    /// Size of the dynamic candidate list during search (50-200 typical)
    /// Higher values give better recall but slower search
    pub ef_search: usize,

    /// Maximum number of elements that can be stored
    pub max_elements: usize,

    /// Random seed for reproducible index construction
    pub seed: Option<u64>,

    /// Distance metric to use
    pub metric: DistanceMetric,

    /// Whether to allow replacing deleted elements
    pub allow_replace_deleted: bool,

    /// Number of threads to use for index operations
    pub num_threads: Option<usize>,

    /// Number of hierarchical layers in the graph (auto-calculated if None)
    pub nb_layers: Option<usize>,

    /// Batch size for parallel insertion operations
    pub parallel_batch_size: Option<usize>,

    /// Enable memory-mapped storage for large datasets (future feature)
    pub enable_mmap: bool,

    /// Memory map sync interval in seconds (if mmap enabled)
    pub mmap_sync_interval: Option<u64>,
}

impl Default for HNSWConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            max_elements: 1_000_000,
            seed: None,
            metric: DistanceMetric::Cosine,
            allow_replace_deleted: true,
            num_threads: None,
            nb_layers: None,
            parallel_batch_size: Some(128),
            enable_mmap: false,
            mmap_sync_interval: Some(60),
        }
    }
}

impl HNSWConfig {
    /// Create a configuration optimized for speed
    ///
    /// Trades some accuracy for faster construction and search.
    /// Suitable for development, testing, or applications where
    /// speed is more important than perfect recall.
    #[must_use]
    pub fn fast() -> Self {
        Self {
            m: 12,
            ef_construction: 100,
            ef_search: 50,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for accuracy
    ///
    /// Maximizes recall at the cost of slower construction and search.
    /// Suitable for production applications where accuracy is critical.
    #[must_use]
    pub fn accurate() -> Self {
        Self {
            m: 32,
            ef_construction: 400,
            ef_search: 200,
            ..Default::default()
        }
    }

    /// Create a configuration balanced between speed and accuracy
    ///
    /// Good general-purpose configuration for most applications.
    /// This is the default configuration.
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            ..Default::default()
        }
    }
}

/// Statistics for a specific namespace
///
/// Provides detailed metrics for a namespace including vector counts,
/// memory usage, and index health indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceStats {
    /// Namespace identifier
    pub namespace: String,

    /// Number of vectors in this namespace
    pub vector_count: usize,

    /// Total memory used in bytes
    pub memory_bytes: usize,

    /// Average number of connections per node
    pub avg_connections: f32,

    /// Index build time in milliseconds
    pub build_time_ms: Option<u64>,

    /// Last optimization timestamp
    pub last_optimized: Option<SystemTime>,
}
