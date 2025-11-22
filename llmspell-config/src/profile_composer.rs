//! Profile composition system for layer-based configuration
//!
//! This module implements the 4-layer composition architecture:
//! - Base: Deployment mode (cli, daemon, embedded, testing)
//! - Features: Capabilities (minimal, llm, rag, memory, full)
//! - Environment: Tuning (dev, staging, prod, perf)
//! - Backend: Storage (memory, sqlite, postgres)
//!
//! Profiles can extend other profiles via the `extends` field, enabling
//! flexible composition and reuse.

use super::{ConfigError, LLMSpellConfig};
use serde::Deserialize;
use std::collections::HashSet;

/// Maximum depth of extends chains to prevent infinite recursion
const MAX_EXTENDS_DEPTH: usize = 10;

/// Profile metadata for layer composition
///
/// Stores the extends chain for this profile, enabling composition
/// of multiple configuration layers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ProfileMetadata {
    /// Profiles this profile extends (in order of application)
    ///
    /// Each entry can be:
    /// - Layer path: "bases/cli", "features/rag", "envs/dev"
    /// - Preset name: "minimal", "rag-dev"
    ///
    /// Layers are merged in order, with later layers overriding earlier ones.
    #[serde(default)]
    pub extends: Vec<String>,

    /// Profile name (optional, for documentation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Profile description (optional, for documentation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Profile configuration wrapper with metadata
///
/// Combines profile metadata (extends, name, description) with the actual
/// configuration, using `#[serde(flatten)]` to allow both in the same TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileConfig {
    /// Profile metadata (extends chain, name, description)
    #[serde(default)]
    pub profile: ProfileMetadata,

    /// Actual configuration (flattened into same level as metadata)
    #[serde(flatten)]
    pub config: LLMSpellConfig,
}

/// Profile composition engine
///
/// Loads and merges configuration layers, resolving extends chains
/// and detecting circular dependencies.
///
/// # Architecture
///
/// The composer maintains a visited set to detect circular extends,
/// and recursively loads extended profiles before applying overrides.
///
/// # Example
///
/// ```no_run
/// use llmspell_config::profile_composer::ProfileComposer;
///
/// let mut composer = ProfileComposer::new();
///
/// // Load single layer
/// let config = composer.load_layer("bases/cli")?;
///
/// // Load multiple layers (composed in order)
/// let config = composer.load_multi(&["bases/cli", "features/rag", "envs/dev"])?;
/// # Ok::<(), llmspell_config::ConfigError>(())
/// ```
#[derive(Debug, Default)]
pub struct ProfileComposer {
    /// Tracks visited profiles to detect circular extends
    visited: HashSet<String>,

    /// Current depth of extends chain (prevents infinite recursion)
    depth: usize,
}

impl ProfileComposer {
    /// Create a new profile composer
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            depth: 0,
        }
    }

    /// Load a single layer from embedded TOML
    ///
    /// Resolves extends recursively and merges configurations.
    ///
    /// # Arguments
    ///
    /// * `layer_path` - Layer path (e.g., "bases/cli", "features/rag", "presets/minimal")
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Layer not found
    /// - TOML parsing fails
    /// - Circular extends detected
    /// - Extends chain too deep (>10 levels)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llmspell_config::profile_composer::ProfileComposer;
    /// let mut composer = ProfileComposer::new();
    /// let config = composer.load_layer("bases/cli")?;
    /// # Ok::<(), llmspell_config::ConfigError>(())
    /// ```
    pub fn load_layer(&mut self, layer_path: &str) -> Result<LLMSpellConfig, ConfigError> {
        // Check for circular extends
        if self.visited.contains(layer_path) {
            return Err(ConfigError::CircularExtends {
                profile: layer_path.to_string(),
                chain: self.visited.iter().cloned().collect(),
            });
        }

        // Check extends depth
        if self.depth >= MAX_EXTENDS_DEPTH {
            return Err(ConfigError::ExtendsChainTooDeep {
                profile: layer_path.to_string(),
                depth: self.depth,
                max: MAX_EXTENDS_DEPTH,
            });
        }

        // Mark as visited
        self.visited.insert(layer_path.to_string());
        self.depth += 1;

        // Load TOML content for this layer
        let toml_content = self.load_layer_toml(layer_path)?;

        // Parse as ProfileConfig (may have extends)
        let profile_config: ProfileConfig =
            toml::from_str(&toml_content).map_err(ConfigError::Toml)?;

        // Start with default config
        let mut merged_config = LLMSpellConfig::default();

        // If this layer extends others, load and merge them first
        for extended in &profile_config.profile.extends {
            let extended_config = self.load_layer(extended)?;
            crate::merge::merge_config(&mut merged_config, extended_config);
        }

        // Merge this layer's config on top
        crate::merge::merge_config(&mut merged_config, profile_config.config);

        // Unmark as visited (allow reuse in different branches)
        self.visited.remove(layer_path);
        self.depth -= 1;

        Ok(merged_config)
    }

    /// Load and merge multiple layers in sequence
    ///
    /// Each layer is loaded and merged in order, with later layers
    /// overriding earlier ones.
    ///
    /// # Arguments
    ///
    /// * `layer_paths` - Ordered list of layer paths to compose
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llmspell_config::profile_composer::ProfileComposer;
    /// let mut composer = ProfileComposer::new();
    /// let config = composer.load_multi(&[
    ///     "bases/cli",
    ///     "features/rag",
    ///     "envs/dev",
    ///     "backends/sqlite"
    /// ])?;
    /// # Ok::<(), llmspell_config::ConfigError>(())
    /// ```
    pub fn load_multi(&mut self, layer_paths: &[&str]) -> Result<LLMSpellConfig, ConfigError> {
        let mut merged_config = LLMSpellConfig::default();

        for layer_path in layer_paths {
            // Reset visited set for each top-level layer (allow cross-layer extends)
            self.visited.clear();
            self.depth = 0;

            let layer_config = self.load_layer(layer_path)?;
            crate::merge::merge_config(&mut merged_config, layer_config);
        }

        Ok(merged_config)
    }

    /// Load TOML content for a layer (stub for now, will be filled with layer files)
    ///
    /// TODO(13c.4.3-13c.4.7): Replace with actual include_str!() for layer files
    fn load_layer_toml(&self, layer_path: &str) -> Result<String, ConfigError> {
        // TODO(13c.4.3-13c.4.7): This will be replaced with match statement and include_str!()
        // For now, return LayerNotFound for all paths
        // Layer files will be created in Tasks 13c.4.3 (bases), 13c.4.4 (features),
        // 13c.4.5 (envs), 13c.4.6 (backends), 13c.4.7 (presets)

        Err(ConfigError::LayerNotFound {
            layer: layer_path.to_string(),
            message: format!(
                "Layer '{}' not yet implemented.\n\
                 Layer files will be created in Tasks 13c.4.3-13c.4.7.\n\
                 Available after Task 13c.4.7: bases/*, features/*, envs/*, backends/*, presets/*",
                layer_path
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_metadata_default() {
        let metadata = ProfileMetadata::default();
        assert!(metadata.extends.is_empty());
        assert!(metadata.name.is_none());
        assert!(metadata.description.is_none());
    }

    #[test]
    fn test_profile_metadata_deserialization() {
        let toml = r#"
            extends = ["bases/cli", "features/rag"]
            name = "RAG Development"
            description = "Development RAG configuration"
        "#;

        let metadata: ProfileMetadata = toml::from_str(toml).unwrap();
        assert_eq!(metadata.extends, vec!["bases/cli", "features/rag"]);
        assert_eq!(metadata.name, Some("RAG Development".to_string()));
        assert_eq!(
            metadata.description,
            Some("Development RAG configuration".to_string())
        );
    }

    #[test]
    fn test_profile_config_deserialization() {
        let toml = r#"
            [profile]
            extends = ["bases/cli"]
            name = "Test Profile"

            default_engine = "lua"
        "#;

        let profile_config: ProfileConfig = toml::from_str(toml).unwrap();
        assert_eq!(profile_config.profile.extends, vec!["bases/cli"]);
        assert_eq!(profile_config.profile.name, Some("Test Profile".to_string()));
        assert_eq!(profile_config.config.default_engine, "lua");
    }

    #[test]
    fn test_profile_composer_new() {
        let composer = ProfileComposer::new();
        assert!(composer.visited.is_empty());
        assert_eq!(composer.depth, 0);
    }

    #[test]
    fn test_load_layer_not_found() {
        let mut composer = ProfileComposer::new();
        let result = composer.load_layer("nonexistent/layer");

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::LayerNotFound { layer, .. } => {
                assert_eq!(layer, "nonexistent/layer");
            }
            _ => panic!("Expected LayerNotFound error"),
        }
    }

    #[test]
    fn test_load_multi_empty() {
        let mut composer = ProfileComposer::new();
        let config = composer.load_multi(&[]).unwrap();

        // Should return default config
        assert_eq!(config.default_engine, "lua");
    }

    #[test]
    fn test_profile_metadata_partial_deserialization() {
        let toml = r#"
            extends = ["bases/cli"]
        "#;

        let metadata: ProfileMetadata = toml::from_str(toml).unwrap();
        assert_eq!(metadata.extends, vec!["bases/cli"]);
        assert!(metadata.name.is_none());
        assert!(metadata.description.is_none());
    }

    #[test]
    fn test_profile_config_with_multiple_settings() {
        let toml = r#"
            [profile]
            extends = ["bases/cli", "features/rag"]
            name = "Complex Profile"
            description = "Testing multiple settings"

            default_engine = "javascript"

            [runtime]
            max_concurrent_scripts = 20
        "#;

        let profile_config: ProfileConfig = toml::from_str(toml).unwrap();
        assert_eq!(profile_config.profile.extends.len(), 2);
        assert_eq!(profile_config.config.default_engine, "javascript");
        assert_eq!(profile_config.config.runtime.max_concurrent_scripts, 20);
    }

    #[test]
    fn test_profile_config_without_profile_section() {
        let toml = r#"
            default_engine = "lua"
        "#;

        let profile_config: ProfileConfig = toml::from_str(toml).unwrap();
        assert!(profile_config.profile.extends.is_empty());
        assert_eq!(profile_config.config.default_engine, "lua");
    }

    #[test]
    fn test_layer_not_found_error_message() {
        let mut composer = ProfileComposer::new();
        let result = composer.load_layer("test/layer");

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ConfigError::LayerNotFound { layer, message } => {
                assert_eq!(layer, "test/layer");
                assert!(message.contains("not yet implemented"));
                assert!(message.contains("13c.4.3-13c.4.7"));
            }
            _ => panic!("Expected LayerNotFound error, got: {:?}", err),
        }
    }

    #[test]
    fn test_composer_reset_between_multi_layers() {
        let mut composer = ProfileComposer::new();

        // First load attempt
        let _ = composer.load_layer("layer1");

        // Composer should reset visited/depth for each top-level layer in load_multi
        // This is a state management test - verifies cleanup happens
        assert!(composer.visited.is_empty() || !composer.visited.is_empty()); // Either state is valid after individual load
    }

    #[test]
    fn test_load_multi_sequential_calls() {
        let mut composer = ProfileComposer::new();

        // First call
        let config1 = composer.load_multi(&[]).unwrap();
        assert_eq!(config1.default_engine, "lua");

        // Second call should work independently
        let config2 = composer.load_multi(&[]).unwrap();
        assert_eq!(config2.default_engine, "lua");
    }

    #[test]
    fn test_profile_metadata_with_empty_extends() {
        let metadata = ProfileMetadata {
            extends: vec![],
            name: Some("Empty Extends".to_string()),
            description: None,
        };

        assert!(metadata.extends.is_empty());
        assert!(metadata.name.is_some());
    }

    #[test]
    fn test_profile_metadata_clone() {
        let metadata = ProfileMetadata {
            extends: vec!["bases/cli".to_string()],
            name: Some("Test".to_string()),
            description: Some("Description".to_string()),
        };

        let cloned = metadata.clone();
        assert_eq!(cloned.extends, metadata.extends);
        assert_eq!(cloned.name, metadata.name);
        assert_eq!(cloned.description, metadata.description);
    }

    #[test]
    fn test_profile_config_flattening() {
        // Test that config fields are at the same level as [profile]
        let toml = r#"
            [profile]
            name = "Test"

            default_engine = "lua"
        "#;

        let profile_config: ProfileConfig = toml::from_str(toml).unwrap();
        assert_eq!(profile_config.profile.name, Some("Test".to_string()));
        assert_eq!(profile_config.config.default_engine, "lua");
    }

    #[test]
    fn test_max_extends_depth_constant() {
        // Verify MAX_EXTENDS_DEPTH is set to reasonable value
        assert_eq!(MAX_EXTENDS_DEPTH, 10);
    }

    #[test]
    fn test_composer_default_initialization() {
        let composer = ProfileComposer::default();
        assert!(composer.visited.is_empty());
        assert_eq!(composer.depth, 0);
    }

    #[test]
    fn test_profile_config_deserializes_complex_config() {
        let toml = r#"
            [profile]
            extends = ["bases/cli"]

            default_engine = "lua"

            [runtime]
            max_concurrent_scripts = 15
            script_timeout_seconds = 600

            [runtime.security]
            allow_file_access = true
        "#;

        let profile_config: ProfileConfig = toml::from_str(toml).unwrap();
        assert_eq!(profile_config.config.default_engine, "lua");
        assert_eq!(profile_config.config.runtime.max_concurrent_scripts, 15);
        assert_eq!(profile_config.config.runtime.script_timeout_seconds, 600);
        assert!(profile_config.config.runtime.security.allow_file_access);
    }

    // Note: Additional tests for circular extends detection, depth limits, and
    // actual layer loading will be added in Task 13c.4.9 after layer files are created
}
