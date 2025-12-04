//! Profile Resolution and Multi-Layer Syntax Parsing
//!
//! Handles parsing of profile specification strings into layer lists for ProfileComposer.
//! Supports three syntax forms:
//!
//! 1. **Single preset name** (backward compatible): `minimal`, `development`, `rag-prod`
//! 2. **Explicit preset path**: `presets/minimal`, `presets/rag-dev`
//! 3. **Multi-layer composition**: `bases/cli,features/rag,envs/dev,backends/sqlite`
//!
//! # Examples
//!
//! ```rust
//! use llmspell_config::profile_resolver::resolve_profile_spec;
//!
//! // Single preset name (backward compatible)
//! let layers = resolve_profile_spec("minimal");
//! assert_eq!(layers, vec!["minimal"]);
//!
//! // Explicit preset path
//! let layers = resolve_profile_spec("presets/rag-dev");
//! assert_eq!(layers, vec!["rag-dev"]);
//!
//! // Multi-layer composition
//! let layers = resolve_profile_spec("bases/cli,features/rag,envs/dev");
//! assert_eq!(layers, vec!["bases/cli", "features/rag", "envs/dev"]);
//! ```

use tracing::debug;

/// Resolve a profile specification string into a list of layer names
///
/// This function parses the profile specification and returns a vector of layer names
/// that can be passed to `ProfileComposer::load_multi()`.
///
/// # Supported Formats
///
/// ## Single Preset Name (Backward Compatible)
/// - Input: `"minimal"`, `"development"`, `"rag-prod"`
/// - Output: `vec!["minimal"]`, `vec!["development"]`, `vec!["rag-prod"]`
/// - These are simple names without slashes, treated as preset names
///
/// ## Explicit Preset Path
/// - Input: `"presets/minimal"`, `"presets/rag-dev"`
/// - Output: `vec!["minimal"]`, `vec!["rag-dev"]`
/// - The `presets/` prefix is stripped since ProfileComposer already handles preset resolution
///
/// ## Multi-Layer Composition
/// - Input: `"bases/cli,features/rag,envs/dev"`
/// - Output: `vec!["bases/cli", "features/rag", "envs/dev"]`
/// - Comma-separated layer paths, each passed through to ProfileComposer
///
/// # Examples
///
/// ```rust
/// use llmspell_config::profile_resolver::resolve_profile_spec;
///
/// // Backward compatible preset names
/// assert_eq!(resolve_profile_spec("minimal"), vec!["minimal"]);
/// assert_eq!(resolve_profile_spec("gemini-prod"), vec!["gemini-prod"]);
///
/// // Explicit preset paths
/// assert_eq!(resolve_profile_spec("presets/minimal"), vec!["minimal"]);
/// assert_eq!(resolve_profile_spec("presets/rag-dev"), vec!["rag-dev"]);
///
/// // Multi-layer composition
/// let layers = resolve_profile_spec("bases/cli,features/rag,envs/dev");
/// assert_eq!(layers, vec!["bases/cli", "features/rag", "envs/dev"]);
///
/// let layers = resolve_profile_spec("bases/daemon,features/full,envs/prod,backends/postgres");
/// assert_eq!(layers, vec!["bases/daemon", "features/full", "envs/prod", "backends/postgres"]);
/// ```
pub fn resolve_profile_spec(spec: &str) -> Vec<&str> {
    // Check if this is multi-layer composition (contains comma)
    if spec.contains(',') {
        debug!("Parsing multi-layer composition: {}", spec);
        // Split by comma and trim whitespace from each layer
        let layers: Vec<&str> = spec.split(',').map(str::trim).collect();
        debug!("Resolved to {} layers: {:?}", layers.len(), layers);
        return layers;
    }

    // Check if this is an explicit preset path (starts with "presets/")
    if let Some(preset_name) = spec.strip_prefix("presets/") {
        debug!("Stripping 'presets/' prefix, resolved to: {}", preset_name);
        return vec![preset_name];
    }

    // Single preset name (backward compatible)
    debug!("Single preset name: {}", spec);
    vec![spec]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_preset_name() {
        assert_eq!(resolve_profile_spec("minimal"), vec!["minimal"]);
        assert_eq!(resolve_profile_spec("development"), vec!["development"]);
        assert_eq!(resolve_profile_spec("rag-prod"), vec!["rag-prod"]);
        assert_eq!(resolve_profile_spec("gemini-prod"), vec!["gemini-prod"]);
        assert_eq!(resolve_profile_spec("openai-prod"), vec!["openai-prod"]);
    }

    #[test]
    fn test_explicit_preset_path() {
        assert_eq!(resolve_profile_spec("presets/minimal"), vec!["minimal"]);
        assert_eq!(resolve_profile_spec("presets/rag-dev"), vec!["rag-dev"]);
        assert_eq!(
            resolve_profile_spec("presets/gemini-prod"),
            vec!["gemini-prod"]
        );
    }

    #[test]
    fn test_multi_layer_composition() {
        let result = resolve_profile_spec("bases/cli,features/rag,envs/dev");
        assert_eq!(result, vec!["bases/cli", "features/rag", "envs/dev"]);

        let result = resolve_profile_spec("bases/daemon,features/full,envs/prod,backends/postgres");
        assert_eq!(
            result,
            vec![
                "bases/daemon",
                "features/full",
                "envs/prod",
                "backends/postgres"
            ]
        );
    }

    #[test]
    fn test_multi_layer_with_whitespace() {
        let result = resolve_profile_spec("bases/cli, features/rag, envs/dev");
        assert_eq!(result, vec!["bases/cli", "features/rag", "envs/dev"]);

        let result = resolve_profile_spec(" bases/cli , features/minimal ");
        assert_eq!(result, vec!["bases/cli", "features/minimal"]);
    }

    #[test]
    fn test_single_layer_path() {
        // Single layer paths (no comma, but with slash) should work
        assert_eq!(resolve_profile_spec("bases/cli"), vec!["bases/cli"]);
        assert_eq!(resolve_profile_spec("features/rag"), vec!["features/rag"]);
        assert_eq!(resolve_profile_spec("envs/prod"), vec!["envs/prod"]);
    }

    #[test]
    fn test_two_layer_composition() {
        let result = resolve_profile_spec("bases/cli,features/minimal");
        assert_eq!(result, vec!["bases/cli", "features/minimal"]);
    }

    #[test]
    fn test_full_stack_composition() {
        let result = resolve_profile_spec("bases/cli,features/full,envs/prod,backends/sqlite");
        assert_eq!(
            result,
            vec!["bases/cli", "features/full", "envs/prod", "backends/sqlite"]
        );
    }
}
