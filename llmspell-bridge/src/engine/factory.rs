//! ABOUTME: Engine factory for creating script engines by name or configuration
//! ABOUTME: Supports built-in engines (Lua, JavaScript) and third-party plugins

use crate::engine::bridge::{EngineFeatures, ScriptEngineBridge};
use llmspell_core::error::LLMSpellError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Factory for creating script engines
pub struct EngineFactory;

impl EngineFactory {
    /// Create a Lua engine with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if Lua feature is not enabled or engine creation fails
    pub fn create_lua_engine(
        config: &LuaConfig,
    ) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError> {
        Self::create_lua_engine_with_runtime(config, None)
    }

    /// Create a Lua engine with the given configuration and runtime config
    ///
    /// # Errors
    ///
    /// Returns an error if Lua feature is not enabled or engine creation fails
    pub fn create_lua_engine_with_runtime(
        config: &LuaConfig,
        runtime_config: Option<Arc<crate::runtime::RuntimeConfig>>,
    ) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            use crate::lua::LuaEngine;
            let mut engine = LuaEngine::new(config)?;
            if let Some(rc) = runtime_config {
                engine.set_runtime_config(rc);
            }
            Ok(Box::new(engine))
        }
        #[cfg(not(feature = "lua"))]
        {
            Err(LLMSpellError::Component {
                message: "Lua engine not enabled. Enable the 'lua' feature.".to_string(),
                source: None,
            })
        }
    }

    /// Create a JavaScript engine with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if JavaScript feature is not enabled or engine creation fails
    #[allow(unused_variables)]
    pub fn create_javascript_engine(
        config: &JSConfig,
    ) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError> {
        #[cfg(feature = "javascript")]
        {
            use crate::javascript::JSEngine;
            Ok(Box::new(JSEngine::new(config)?))
        }
        #[cfg(not(feature = "javascript"))]
        {
            Err(LLMSpellError::Component {
                message: "JavaScript engine not enabled. Enable the 'javascript' feature."
                    .to_string(),
                source: None,
            })
        }
    }

    /// Create an engine by name with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The engine name is unknown
    /// - The configuration is invalid for the engine type
    /// - The engine creation fails
    ///
    /// # Panics
    ///
    /// Panics if the plugin registry lock is poisoned
    pub fn create_from_name(
        name: &str,
        config: &Value,
    ) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError> {
        match name {
            "lua" => {
                let lua_config =
                    serde_json::from_value::<LuaConfig>(config.clone()).map_err(|e| {
                        LLMSpellError::Validation {
                            field: Some("config".to_string()),
                            message: format!("Invalid Lua configuration: {e}"),
                        }
                    })?;
                Self::create_lua_engine(&lua_config)
            }
            "javascript" | "js" => {
                let js_config =
                    serde_json::from_value::<JSConfig>(config.clone()).map_err(|e| {
                        LLMSpellError::Validation {
                            field: Some("config".to_string()),
                            message: format!("Invalid JavaScript configuration: {e}"),
                        }
                    })?;
                Self::create_javascript_engine(&js_config)
            }
            _ => {
                // Check if it's a registered plugin
                let registry = PLUGIN_REGISTRY
                    .read()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                registry.get(name).map_or_else(
                    || {
                        Err(LLMSpellError::Validation {
                            field: Some("engine".to_string()),
                            message: format!("Unknown engine: {name}. Available: lua, javascript"),
                        })
                    },
                    |plugin| plugin.create_engine(config.clone()),
                )
            }
        }
    }

    /// List all available engines (built-in and plugins)
    ///
    /// # Panics
    ///
    /// Panics if the plugin registry lock is poisoned
    #[must_use]
    pub fn list_available_engines() -> Vec<EngineInfo> {
        let mut engines = vec![];

        #[cfg(feature = "lua")]
        engines.push(EngineInfo {
            name: "lua".to_string(),
            description: "Lua 5.4 scripting engine".to_string(),
            version: "5.4".to_string(),
            features: crate::lua::LuaEngine::engine_features(),
        });

        #[cfg(feature = "javascript")]
        engines.push(EngineInfo {
            name: "javascript".to_string(),
            description: "JavaScript (ES2020) engine".to_string(),
            version: "ES2020".to_string(),
            features: crate::javascript::JSEngine::engine_features(),
        });

        // Add registered plugins
        {
            let registry = PLUGIN_REGISTRY
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for (name, plugin) in registry.iter() {
                engines.push(EngineInfo {
                    name: name.clone(),
                    description: plugin.description(),
                    version: plugin.version(),
                    features: plugin.supported_features(),
                });
            }
        } // Explicitly drop the lock here

        engines
    }
}

/// Information about an available engine
#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub features: EngineFeatures,
}

/// Configuration for the Lua engine
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LuaConfig {
    /// Standard library access level
    pub stdlib: StdlibLevel,
    /// Maximum memory usage in bytes
    pub max_memory: Option<usize>,
    /// Enable debug features
    pub debug: bool,
    /// Custom package paths
    pub package_paths: Vec<String>,
}

impl Default for LuaConfig {
    fn default() -> Self {
        Self {
            stdlib: StdlibLevel::Safe,
            max_memory: Some(50_000_000), // 50MB default
            debug: false,
            package_paths: vec![],
        }
    }
}

impl LuaConfig {
    /// Create a new builder for `LuaConfig`
    #[must_use]
    pub fn builder() -> LuaConfigBuilder {
        LuaConfigBuilder::new()
    }
}

/// Builder for `LuaConfig`
#[derive(Debug, Clone)]
pub struct LuaConfigBuilder {
    config: LuaConfig,
}

impl LuaConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: LuaConfig::default(),
        }
    }

    /// Set the standard library access level
    #[must_use]
    pub const fn stdlib(mut self, level: StdlibLevel) -> Self {
        self.config.stdlib = level;
        self
    }

    /// Set the maximum memory usage in bytes
    #[must_use]
    pub const fn max_memory(mut self, memory: Option<usize>) -> Self {
        self.config.max_memory = memory;
        self
    }

    /// Enable or disable debug features
    #[must_use]
    pub const fn debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    /// Add a package path
    #[must_use]
    pub fn add_package_path(mut self, path: impl Into<String>) -> Self {
        self.config.package_paths.push(path.into());
        self
    }

    /// Set all package paths at once
    #[must_use]
    pub fn package_paths(mut self, paths: Vec<String>) -> Self {
        self.config.package_paths = paths;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> LuaConfig {
        self.config
    }
}

impl Default for LuaConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Lua standard library access levels
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StdlibLevel {
    /// No standard library
    None,
    /// Safe subset only
    Safe,
    /// Full standard library
    Full,
}

/// Configuration for the JavaScript engine
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct JSConfig {
    /// Enable strict mode
    pub strict_mode: bool,
    /// Maximum heap size in bytes
    pub max_heap_size: Option<usize>,
    /// Enable console API
    pub enable_console: bool,
    /// Module resolution strategy
    pub module_resolution: ModuleResolution,
}

impl Default for JSConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            max_heap_size: Some(100_000_000), // 100MB default
            enable_console: true,
            module_resolution: ModuleResolution::Node,
        }
    }
}

impl JSConfig {
    /// Create a new builder for `JSConfig`
    #[must_use]
    pub fn builder() -> JSConfigBuilder {
        JSConfigBuilder::new()
    }
}

/// Builder for `JSConfig`
#[derive(Debug, Clone)]
pub struct JSConfigBuilder {
    config: JSConfig,
}

impl JSConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: JSConfig::default(),
        }
    }

    /// Enable or disable strict mode
    #[must_use]
    pub const fn strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }

    /// Set the maximum heap size in bytes
    #[must_use]
    pub const fn max_heap_size(mut self, size: Option<usize>) -> Self {
        self.config.max_heap_size = size;
        self
    }

    /// Enable or disable console API
    #[must_use]
    pub const fn enable_console(mut self, enable: bool) -> Self {
        self.config.enable_console = enable;
        self
    }

    /// Set the module resolution strategy
    #[must_use]
    pub const fn module_resolution(mut self, resolution: ModuleResolution) -> Self {
        self.config.module_resolution = resolution;
        self
    }

    /// Build the configuration
    #[must_use]
    pub const fn build(self) -> JSConfig {
        self.config
    }
}

impl Default for JSConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript module resolution strategies
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleResolution {
    /// Node.js-style resolution
    Node,
    /// Browser-style resolution
    Browser,
    /// Deno-style resolution
    Deno,
}

// Plugin system for third-party engines

type PluginRegistry = Arc<RwLock<HashMap<String, Box<dyn ScriptEnginePlugin>>>>;

static PLUGIN_REGISTRY: LazyLock<PluginRegistry> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Plugin interface for third-party script engines
pub trait ScriptEnginePlugin: Send + Sync {
    /// Get the name of this engine
    fn engine_name(&self) -> &str;

    /// Get a description of this engine
    fn description(&self) -> String;

    /// Get the version of this engine
    fn version(&self) -> String;

    /// Create an instance of this engine
    ///
    /// # Errors
    ///
    /// Returns an error if engine creation fails or configuration is invalid
    fn create_engine(&self, config: Value) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError>;

    /// Get the features supported by this engine
    fn supported_features(&self) -> EngineFeatures;
}

/// Register a third-party engine plugin
///
/// # Panics
///
/// Panics if the plugin registry lock is poisoned
pub fn register_engine_plugin<P: ScriptEnginePlugin + 'static>(plugin: P) {
    let mut registry = PLUGIN_REGISTRY
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    registry.insert(plugin.engine_name().to_string(), Box::new(plugin));
}

/// Unregister an engine plugin
///
/// # Panics
///
/// Panics if the plugin registry lock is poisoned
#[must_use]
pub fn unregister_engine_plugin(name: &str) -> bool {
    let mut registry = PLUGIN_REGISTRY
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    registry.remove(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lua_config_default() {
        let config = LuaConfig::default();
        assert_eq!(config.max_memory, Some(50_000_000));
        assert!(!config.debug);
        assert!(config.package_paths.is_empty());
    }
    #[test]
    fn test_js_config_default() {
        let config = JSConfig::default();
        assert!(config.strict_mode);
        assert_eq!(config.max_heap_size, Some(100_000_000));
        assert!(config.enable_console);
    }
    #[test]
    fn test_engine_factory_unknown_engine() {
        let result = EngineFactory::create_from_name("unknown", &Value::Null);
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                LLMSpellError::Validation { field, .. } => {
                    assert_eq!(field, Some("engine".to_string()));
                }
                _ => panic!("Expected validation error"),
            }
        }
    }
}
