//! Application discovery and metadata system
//! Configurable filesystem-based app discovery with caching

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{debug, warn};

/// Application metadata parsed from config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub complexity: Option<String>,
    pub agents: Option<u32>,
    pub tags: Option<Vec<String>>,
    pub path: PathBuf,
    pub config_path: Option<PathBuf>,
    pub main_script: PathBuf,
}

/// Application discovery configuration
#[derive(Debug, Clone)]
pub struct AppDiscoveryConfig {
    pub search_paths: Vec<PathBuf>,
    pub cache_duration: Duration,
    pub require_main_lua: bool,
    pub require_config_toml: bool,
}

impl Default for AppDiscoveryConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![
                PathBuf::from("examples/script-users/applications"),
                PathBuf::from("~/.llmspell/apps"),
                PathBuf::from("/usr/local/share/llmspell/apps"),
            ],
            cache_duration: Duration::from_secs(300), // 5 minutes
            require_main_lua: true,
            require_config_toml: false,
        }
    }
}

/// Cached application discovery results
#[derive(Debug)]
struct CachedResults {
    apps: HashMap<String, AppMetadata>,
    last_scan: SystemTime,
}

/// Application discovery system
pub struct AppDiscovery {
    config: AppDiscoveryConfig,
    cache: Option<CachedResults>,
}

impl AppDiscovery {
    /// Create new app discovery system with default config
    pub fn new() -> Self {
        Self {
            config: AppDiscoveryConfig::default(),
            cache: None,
        }
    }

    /// Create new app discovery system with custom config
    pub fn with_config(config: AppDiscoveryConfig) -> Self {
        Self {
            config,
            cache: None,
        }
    }

    /// Add search path
    pub fn add_search_path<P: Into<PathBuf>>(&mut self, path: P) {
        self.config.search_paths.push(path.into());
        self.invalidate_cache();
    }

    /// Set search paths (replaces existing)
    pub fn set_search_paths(&mut self, paths: Vec<PathBuf>) {
        self.config.search_paths = paths;
        self.invalidate_cache();
    }

    /// Invalidate cache to force rescan
    pub fn invalidate_cache(&mut self) {
        self.cache = None;
    }

    /// Discover all applications
    pub fn discover_apps(&mut self) -> Result<HashMap<String, AppMetadata>> {
        // Check if cache is valid
        if let Some(ref cached) = self.cache {
            let cache_age = SystemTime::now()
                .duration_since(cached.last_scan)
                .unwrap_or(Duration::from_secs(u64::MAX));

            if cache_age < self.config.cache_duration {
                debug!("Using cached app discovery results (age: {:?})", cache_age);
                return Ok(cached.apps.clone());
            }
        }

        debug!(
            "Scanning for applications in {:?}",
            self.config.search_paths
        );

        let mut apps = HashMap::new();

        for search_path in &self.config.search_paths {
            if let Err(e) = self.scan_directory(search_path, &mut apps) {
                warn!("Failed to scan directory {:?}: {}", search_path, e);
                continue;
            }
        }

        debug!("Discovered {} applications", apps.len());

        // Update cache
        self.cache = Some(CachedResults {
            apps: apps.clone(),
            last_scan: SystemTime::now(),
        });

        Ok(apps)
    }

    /// Get application by name
    pub fn get_app(&mut self, name: &str) -> Result<Option<AppMetadata>> {
        let apps = self.discover_apps()?;
        Ok(apps.get(name).cloned())
    }

    /// List all application names
    pub fn list_apps(&mut self) -> Result<Vec<String>> {
        let apps = self.discover_apps()?;
        let mut names: Vec<String> = apps.keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    /// Search applications by tag
    pub fn search_by_tag(&mut self, tag: &str) -> Result<Vec<AppMetadata>> {
        let apps = self.discover_apps()?;
        let mut results = Vec::new();

        for app in apps.values() {
            if let Some(ref tags) = app.tags {
                if tags.iter().any(|t| t.contains(tag)) {
                    results.push(app.clone());
                }
            }
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(results)
    }

    /// Search applications by complexity
    pub fn search_by_complexity(&mut self, complexity: &str) -> Result<Vec<AppMetadata>> {
        let apps = self.discover_apps()?;
        let mut results = Vec::new();

        for app in apps.values() {
            if let Some(ref app_complexity) = app.complexity {
                if app_complexity
                    .to_lowercase()
                    .contains(&complexity.to_lowercase())
                {
                    results.push(app.clone());
                }
            }
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(results)
    }

    /// Scan a directory for applications
    fn scan_directory(&self, path: &Path, apps: &mut HashMap<String, AppMetadata>) -> Result<()> {
        // Expand tilde in path
        let expanded_path = if path.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                home.join(path.strip_prefix("~").unwrap())
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        if !expanded_path.exists() {
            debug!("Search path does not exist: {:?}", expanded_path);
            return Ok(());
        }

        let entries = std::fs::read_dir(&expanded_path)
            .with_context(|| format!("Failed to read directory: {:?}", expanded_path))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let app_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Check for required files
            let main_lua = path.join("main.lua");
            let config_toml = path.join("config.toml");

            if self.config.require_main_lua && !main_lua.exists() {
                debug!("Skipping {} - no main.lua", app_name);
                continue;
            }

            if self.config.require_config_toml && !config_toml.exists() {
                debug!("Skipping {} - no config.toml", app_name);
                continue;
            }

            // Parse metadata
            match self.parse_app_metadata(&app_name, &path) {
                Ok(metadata) => {
                    debug!("Found app: {} at {:?}", app_name, path);
                    apps.insert(app_name, metadata);
                }
                Err(e) => {
                    warn!("Failed to parse metadata for {}: {}", app_name, e);
                }
            }
        }

        Ok(())
    }

    /// Parse application metadata from directory
    fn parse_app_metadata(&self, name: &str, path: &Path) -> Result<AppMetadata> {
        let config_path = path.join("config.toml");
        let main_script = path.join("main.lua");

        let mut metadata = AppMetadata {
            name: name.to_string(),
            description: None,
            version: None,
            complexity: None,
            agents: None,
            tags: None,
            path: path.to_path_buf(),
            config_path: if config_path.exists() {
                Some(config_path.clone())
            } else {
                None
            },
            main_script,
        };

        // Parse config.toml if it exists
        if config_path.exists() {
            if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                if let Ok(config_data) = toml::from_str::<toml::Value>(&config_content) {
                    // Extract metadata from config
                    if let Some(app_section) = config_data.get("app") {
                        if let Some(desc) = app_section.get("description").and_then(|v| v.as_str())
                        {
                            metadata.description = Some(desc.to_string());
                        }
                        if let Some(version) = app_section.get("version").and_then(|v| v.as_str()) {
                            metadata.version = Some(version.to_string());
                        }
                        if let Some(complexity) =
                            app_section.get("complexity").and_then(|v| v.as_str())
                        {
                            metadata.complexity = Some(complexity.to_string());
                        }
                        if let Some(agents) = app_section.get("agents").and_then(|v| v.as_integer())
                        {
                            metadata.agents = Some(agents as u32);
                        }
                        if let Some(tags) = app_section.get("tags").and_then(|v| v.as_array()) {
                            let tag_strings: Vec<String> = tags
                                .iter()
                                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                                .collect();
                            if !tag_strings.is_empty() {
                                metadata.tags = Some(tag_strings);
                            }
                        }
                    }
                }
            }
        }

        // Fallback: parse from script comments if no config
        if metadata.description.is_none() && metadata.main_script.exists() {
            if let Ok(script_content) = std::fs::read_to_string(&metadata.main_script) {
                metadata.description = self.extract_description_from_script(&script_content);
            }
        }

        Ok(metadata)
    }

    /// Extract description from script comments
    fn extract_description_from_script(&self, content: &str) -> Option<String> {
        for line in content.lines().take(20) {
            let line = line.trim();
            if line.starts_with("-- Purpose:") {
                return Some(line.trim_start_matches("-- Purpose:").trim().to_string());
            }
            if line.starts_with("-- Description:") {
                return Some(
                    line.trim_start_matches("-- Description:")
                        .trim()
                        .to_string(),
                );
            }
            if line.starts_with("-- Application:") && line.contains(" - ") {
                if let Some(desc_part) = line.split(" - ").nth(1) {
                    return Some(desc_part.trim().to_string());
                }
            }
        }
        None
    }
}

impl Default for AppDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_discovery_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut discovery = AppDiscovery::with_config(AppDiscoveryConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            cache_duration: Duration::from_secs(1),
            require_main_lua: true,
            require_config_toml: false,
        });

        let apps = discovery.discover_apps().unwrap();
        assert!(apps.is_empty());
    }

    #[test]
    fn test_app_discovery_with_valid_app() {
        let temp_dir = TempDir::new().unwrap();
        let app_dir = temp_dir.path().join("test-app");
        std::fs::create_dir_all(&app_dir).unwrap();

        // Create main.lua
        std::fs::write(
            app_dir.join("main.lua"),
            "-- Purpose: Test application\nprint('test')",
        )
        .unwrap();

        // Create config.toml
        let config_content = r#"
[app]
description = "Test application for discovery"
version = "1.0.0"
complexity = "Simple"
agents = 2
tags = ["test", "example"]
"#;
        std::fs::write(app_dir.join("config.toml"), config_content).unwrap();

        let mut discovery = AppDiscovery::with_config(AppDiscoveryConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            cache_duration: Duration::from_secs(1),
            require_main_lua: true,
            require_config_toml: false,
        });

        let apps = discovery.discover_apps().unwrap();
        assert_eq!(apps.len(), 1);

        let app = apps.get("test-app").unwrap();
        assert_eq!(app.name, "test-app");
        assert_eq!(
            app.description.as_ref().unwrap(),
            "Test application for discovery"
        );
        assert_eq!(app.version.as_ref().unwrap(), "1.0.0");
        assert_eq!(app.complexity.as_ref().unwrap(), "Simple");
        assert_eq!(app.agents.unwrap(), 2);
        assert!(app.tags.as_ref().unwrap().contains(&"test".to_string()));
    }

    #[test]
    fn test_app_discovery_cache() {
        let temp_dir = TempDir::new().unwrap();
        let mut discovery = AppDiscovery::with_config(AppDiscoveryConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            cache_duration: Duration::from_secs(60),
            require_main_lua: true,
            require_config_toml: false,
        });

        // First scan
        let _apps1 = discovery.discover_apps().unwrap();

        // Second scan should use cache
        let _apps2 = discovery.discover_apps().unwrap();

        // Cache should be used (we can't easily test timing in unit tests,
        // but this verifies the cache mechanism doesn't crash)
        assert!(discovery.cache.is_some());
    }

    #[test]
    fn test_search_by_tag() {
        let temp_dir = TempDir::new().unwrap();
        let app_dir = temp_dir.path().join("tagged-app");
        std::fs::create_dir_all(&app_dir).unwrap();

        std::fs::write(app_dir.join("main.lua"), "print('test')").unwrap();

        let config_content = r#"
[app]
tags = ["productivity", "file-management"]
"#;
        std::fs::write(app_dir.join("config.toml"), config_content).unwrap();

        let mut discovery = AppDiscovery::with_config(AppDiscoveryConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            cache_duration: Duration::from_secs(1),
            require_main_lua: true,
            require_config_toml: false,
        });

        let results = discovery.search_by_tag("productivity").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "tagged-app");

        let results = discovery.search_by_tag("nonexistent").unwrap();
        assert!(results.is_empty());
    }
}
