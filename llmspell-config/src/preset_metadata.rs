//! Preset metadata composition from layers
//!
//! This module reads preset TOML files and composes profile metadata
//! by combining metadata from the referenced layers.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::layer_metadata::load_layer_metadata;
use crate::ProfileMetadata;

/// Wrapper struct for parsing preset TOML files
#[derive(Debug, Deserialize)]
struct PresetFile {
    profile: PresetProfile,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PresetProfile {
    name: String,
    description: String,
    extends: Vec<String>,
}

/// Read the layer composition from a preset file
///
/// # Arguments
/// * `preset_name` - Name of the preset (e.g., "rag-dev", "minimal")
///
/// # Returns
/// * `Result<Vec<String>>` - List of layer paths (e.g., ["bases/cli", "features/rag"])
pub fn read_preset_composition(preset_name: &str) -> Result<Vec<String>> {
    let presets_dir = get_presets_directory()?;
    let toml_path = presets_dir.join(format!("{}.toml", preset_name));

    let content = fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read preset file: {}", toml_path.display()))?;

    let preset_file: PresetFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse preset TOML: {}", toml_path.display()))?;

    Ok(preset_file.profile.extends)
}

/// Compose profile metadata from preset layers
///
/// # Arguments
/// * `preset_name` - Name of the preset (e.g., "rag-dev")
///
/// # Returns
/// * `Result<ProfileMetadata>` - Composed metadata from all layers
pub fn compose_preset_metadata(preset_name: &str) -> Result<ProfileMetadata> {
    // Read layer composition
    let layers = read_preset_composition(preset_name)?;

    // Load metadata for each layer
    let mut all_use_cases = Vec::new();
    let mut all_features = Vec::new();
    let mut layer_descriptions = Vec::new();

    for layer_path in &layers {
        let layer_meta = load_layer_metadata(layer_path)
            .with_context(|| format!("Failed to load layer: {}", layer_path))?;

        layer_descriptions.push(layer_meta.description.clone());
        all_use_cases.extend(layer_meta.use_cases);
        all_features.extend(layer_meta.features);
    }

    // Deduplicate use cases and features
    let use_cases: Vec<String> = deduplicate_preserving_order(all_use_cases);
    let features: Vec<String> = deduplicate_preserving_order(all_features);

    // Derive category from layers
    let category = derive_category(&layers);

    // Compose description from layer descriptions
    let description = compose_description(preset_name, &layer_descriptions, &layers);

    Ok(ProfileMetadata {
        name: preset_name.to_string(),
        category: category.to_string(),
        description,
        use_cases,
        features,
        layers,
    })
}

/// Derive profile category from layer composition
///
/// Priority order:
/// 1. Feature layer determines primary category
/// 2. Backend/env modifiers for Production
/// 3. Default to Development
fn derive_category(layers: &[String]) -> &'static str {
    // Check for RAG features
    if layers
        .iter()
        .any(|l| l.contains("features/rag") || l.contains("features/memory"))
    {
        return "RAG";
    }

    // Check for local LLM
    if layers.iter().any(|l| l.contains("features/llm-local")) {
        return "Local LLM";
    }

    // Check for production indicators
    if layers
        .iter()
        .any(|l| l.contains("backends/postgres") || l.contains("envs/prod"))
    {
        return "Production";
    }

    // Check for minimal/core
    if layers.iter().any(|l| l.contains("features/minimal")) {
        return "Core";
    }

    // Check for full features
    if layers.iter().any(|l| l.contains("features/full")) {
        // If also has prod env or postgres, it's Production
        if layers
            .iter()
            .any(|l| l.contains("envs/prod") || l.contains("backends/postgres"))
        {
            return "Production";
        }
    }

    // Default to Development
    "Development"
}

/// Compose a description from layer descriptions and preset name
fn compose_description(
    preset_name: &str,
    layer_descriptions: &[String],
    layers: &[String],
) -> String {
    // Special cases for well-known presets
    match preset_name {
        "rag-dev" => "RAG development with trace logging".to_string(),
        "rag-prod" => "RAG production with SQLite".to_string(),
        "rag-perf" => "RAG performance tuned".to_string(),
        "minimal" => "Tools only, no LLM features".to_string(),
        "development" => "Dev environment with cloud LLM providers".to_string(),
        "providers" => "All LLM providers (OpenAI, Anthropic, Gemini, Ollama, Candle)".to_string(),
        "ollama" => "Local Ollama models".to_string(),
        "candle" => "Local Candle ML models".to_string(),
        "memory" => "Adaptive memory system".to_string(),
        "state" => "State persistence + sessions".to_string(),
        "sessions" => "Session management with artifacts".to_string(),
        "default" => "Minimal CLI setup".to_string(),
        "postgres-prod" => "Production PostgreSQL backend".to_string(),
        "daemon-dev" => "Daemon mode development".to_string(),
        "daemon-prod" => "Daemon mode production".to_string(),
        "gemini-prod" => "Full Phase 13 stack + Gemini".to_string(),
        "openai-prod" => "Full Phase 13 stack + OpenAI".to_string(),
        "claude-prod" => "Full Phase 13 stack + Claude/Anthropic".to_string(),
        "full-local-ollama" => "Complete local stack (Ollama + SQLite)".to_string(),
        "research" => "Full features + trace logging".to_string(),
        _ => {
            // Generic composition from layers
            let feature_desc = layers
                .iter()
                .find(|l| l.starts_with("features/"))
                .and_then(|_l| layer_descriptions.iter().find(|d| !d.is_empty()))
                .cloned()
                .unwrap_or_else(|| "Custom configuration".to_string());

            let env = if layers.iter().any(|l| l.contains("envs/dev")) {
                " (development)"
            } else if layers.iter().any(|l| l.contains("envs/prod")) {
                " (production)"
            } else {
                ""
            };

            format!("{}{}", feature_desc, env)
        }
    }
}

/// Deduplicate a vector while preserving order
fn deduplicate_preserving_order(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

/// Get the presets directory path
fn get_presets_directory() -> Result<PathBuf> {
    // Try to find presets directory relative to CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let presets_path = PathBuf::from(manifest_dir).join("presets");
        if presets_path.exists() {
            return Ok(presets_path);
        }
    }

    // Fallback: try relative to current directory
    let presets_path = PathBuf::from("llmspell-config/presets");
    if presets_path.exists() {
        return Ok(presets_path);
    }

    anyhow::bail!("Could not find presets directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_preset_composition() {
        let result = read_preset_composition("rag-dev");
        assert!(
            result.is_ok(),
            "Failed to read rag-dev composition: {:?}",
            result.err()
        );

        let layers = result.unwrap();
        assert!(layers.contains(&"bases/cli".to_string()));
        assert!(layers.contains(&"features/rag".to_string()));
    }

    #[test]
    fn test_compose_preset_metadata() {
        let result = compose_preset_metadata("rag-dev");
        assert!(
            result.is_ok(),
            "Failed to compose rag-dev metadata: {:?}",
            result.err()
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "rag-dev");
        assert_eq!(metadata.category, "RAG");
        assert!(!metadata.description.is_empty());
        assert!(!metadata.use_cases.is_empty());
        assert!(!metadata.features.is_empty());
        assert!(!metadata.layers.is_empty());
    }

    #[test]
    fn test_derive_category() {
        assert_eq!(
            derive_category(&["bases/cli".to_string(), "features/rag".to_string()]),
            "RAG"
        );
        assert_eq!(
            derive_category(&["bases/cli".to_string(), "features/llm-local".to_string()]),
            "Local LLM"
        );
        assert_eq!(
            derive_category(&["bases/cli".to_string(), "features/minimal".to_string()]),
            "Core"
        );
        assert_eq!(
            derive_category(&["bases/daemon".to_string(), "backends/postgres".to_string()]),
            "Production"
        );
    }

    #[test]
    fn test_deduplicate() {
        let items = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item1".to_string(),
            "item3".to_string(),
        ];
        let result = deduplicate_preserving_order(items);
        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }
}
