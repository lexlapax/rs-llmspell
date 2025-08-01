//! ABOUTME: Artifact collector hooks for automatically capturing outputs
//! ABOUTME: Provides collectors for tool results, agent outputs, and custom artifacts

use crate::{Hook, HookContext};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub mod agent_output;
pub mod tool_result;

pub use agent_output::AgentOutputCollector;
pub use tool_result::ToolResultCollector;

/// Configuration for artifact collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Minimum size in bytes to collect (avoid tiny artifacts)
    pub min_size: usize,
    /// Maximum size in bytes to collect (prevent huge artifacts)
    pub max_size: usize,
    /// Whether to collect error outputs
    pub collect_errors: bool,
    /// Sampling rate (0.0 to 1.0, where 1.0 collects everything)
    pub sampling_rate: f64,
    /// Tags to automatically add to collected artifacts
    pub auto_tags: Vec<String>,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            min_size: 100,              // Don't collect tiny outputs
            max_size: 10 * 1024 * 1024, // 10MB limit
            collect_errors: true,
            sampling_rate: 1.0,
            auto_tags: vec!["auto-collected".to_string()],
        }
    }
}

/// Trait for hooks that collect artifacts
#[async_trait]
pub trait ArtifactCollector: Hook {
    /// Check if this context should result in artifact collection
    async fn should_collect(&self, context: &HookContext) -> bool;

    /// Extract artifact data from the context
    async fn extract_artifact_data(&self, context: &HookContext) -> Result<ArtifactData>;

    /// Get the artifact type for this collector
    fn artifact_type(&self) -> &str;

    /// Get collection configuration
    fn config(&self) -> &CollectionConfig;
}

/// Data extracted for artifact creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactData {
    /// Name for the artifact
    pub name: String,
    /// Content to store
    pub content: Vec<u8>,
    /// MIME type
    pub mime_type: String,
    /// Additional metadata
    pub metadata: JsonValue,
    /// Tags for the artifact
    pub tags: Vec<String>,
}

/// Check if content size is within configured limits
pub fn is_size_acceptable(size: usize, config: &CollectionConfig) -> bool {
    size >= config.min_size && size <= config.max_size
}

/// Check if we should sample this collection (for rate limiting)
pub fn should_sample(config: &CollectionConfig) -> bool {
    if config.sampling_rate >= 1.0 {
        return true;
    }
    if config.sampling_rate <= 0.0 {
        return false;
    }

    // Simple random sampling
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < config.sampling_rate
}
