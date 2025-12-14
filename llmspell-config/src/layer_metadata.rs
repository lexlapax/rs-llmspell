//! Layer metadata loading and caching
//!
//! This module provides functionality to load metadata from layer TOML files.
//! Layer metadata includes descriptive information about each layer (bases, features, envs, backends)
//! that is used to generate profile metadata dynamically.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Layer category types
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayerCategory {
    Base,
    Feature,
    Env,
    Backend,
}

/// Metadata for a single layer
#[derive(Debug, Clone, Deserialize)]
pub struct LayerMetadata {
    pub name: String,
    pub category: LayerCategory,
    pub description: String,
    pub use_cases: Vec<String>,
    pub features: Vec<String>,
}

/// Wrapper struct for parsing TOML files with metadata section
#[derive(Debug, Deserialize)]
struct LayerFile {
    metadata: LayerMetadata,
}

/// Global cache of loaded layer metadata
static LAYER_CACHE: LazyLock<HashMap<String, LayerMetadata>> = LazyLock::new(HashMap::new);

/// Load layer metadata from a layer path (e.g., "bases/cli", "features/rag")
///
/// # Arguments
/// * `layer_path` - Path to the layer relative to layers directory (e.g., "bases/cli")
///
/// # Returns
/// * `Result<LayerMetadata>` - The loaded metadata or an error
///
/// # Example
/// ```ignore
/// let metadata = load_layer_metadata("bases/cli")?;
/// assert_eq!(metadata.name, "cli");
/// assert_eq!(metadata.category, LayerCategory::Base);
/// ```
pub fn load_layer_metadata(layer_path: &str) -> Result<LayerMetadata> {
    // Check cache first
    if let Some(cached) = LAYER_CACHE.get(layer_path) {
        return Ok(cached.clone());
    }

    // Build path to layer file
    let layers_dir = get_layers_directory()?;
    let toml_path = layers_dir.join(format!("{}.toml", layer_path));

    // Read and parse TOML file
    let content = fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read layer file: {}", toml_path.display()))?;

    let layer_file: LayerFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse layer TOML: {}", toml_path.display()))?;

    Ok(layer_file.metadata)
}

/// Get the layers directory path
///
/// This looks for the layers directory relative to the crate root.
/// In development, this is `llmspell-config/layers/`.
fn get_layers_directory() -> Result<PathBuf> {
    // Try to find layers directory relative to CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let layers_path = PathBuf::from(manifest_dir).join("layers");
        if layers_path.exists() {
            return Ok(layers_path);
        }
    }

    // Fallback: try relative to current directory
    let layers_path = PathBuf::from("llmspell-config/layers");
    if layers_path.exists() {
        return Ok(layers_path);
    }

    anyhow::bail!("Could not find layers directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cli_layer() {
        let result = load_layer_metadata("bases/cli");
        assert!(
            result.is_ok(),
            "Failed to load bases/cli: {:?}",
            result.err()
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "cli");
        assert_eq!(metadata.category, LayerCategory::Base);
        assert!(!metadata.description.is_empty());
        assert!(!metadata.use_cases.is_empty());
        assert!(!metadata.features.is_empty());
    }

    #[test]
    fn test_load_rag_layer() {
        let result = load_layer_metadata("features/rag");
        assert!(
            result.is_ok(),
            "Failed to load features/rag: {:?}",
            result.err()
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "rag");
        assert_eq!(metadata.category, LayerCategory::Feature);
    }

    #[test]
    fn test_layer_categories() {
        // Test that each category can be loaded
        let base = load_layer_metadata("bases/cli").unwrap();
        assert_eq!(base.category, LayerCategory::Base);

        let feature = load_layer_metadata("features/rag").unwrap();
        assert_eq!(feature.category, LayerCategory::Feature);
    }
}
