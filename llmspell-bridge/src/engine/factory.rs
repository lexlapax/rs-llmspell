//! ABOUTME: Engine factory for creating script engines by name or configuration
//! ABOUTME: Supports built-in engines (Lua, JavaScript) and third-party plugins

use crate::engine::bridge::{EngineFeatures, ScriptEngineBridge};
use llmspell_core::error::LLMSpellError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Factory for creating script engines
pub struct EngineFactory;

impl EngineFactory {
    /// Create a Lua engine with the given configuration
    pub fn create_lua_engine(
        config: &LuaConfig,
    ) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            use crate::lua::LuaEngine;
            Ok(Box::new(LuaEngine::new(config)?))
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
                            message: format!("Invalid Lua configuration: {}", e),
                        }
                    })?;
                Self::create_lua_engine(&lua_config)
            }
            "javascript" | "js" => {
                let js_config =
                    serde_json::from_value::<JSConfig>(config.clone()).map_err(|e| {
                        LLMSpellError::Validation {
                            field: Some("config".to_string()),
                            message: format!("Invalid JavaScript configuration: {}", e),
                        }
                    })?;
                Self::create_javascript_engine(&js_config)
            }
            _ => {
                // Check if it's a registered plugin
                let registry = PLUGIN_REGISTRY.read().unwrap();
                if let Some(plugin) = registry.get(name) {
                    plugin.create_engine(config.clone())
                } else {
                    Err(LLMSpellError::Validation {
                        field: Some("engine".to_string()),
                        message: format!("Unknown engine: {}. Available: lua, javascript", name),
                    })
                }
            }
        }
    }

    /// List all available engines (built-in and plugins)
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
        let registry = PLUGIN_REGISTRY.read().unwrap();
        for (name, plugin) in registry.iter() {
            engines.push(EngineInfo {
                name: name.clone(),
                description: plugin.description(),
                version: plugin.version(),
                features: plugin.supported_features(),
            });
        }

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

lazy_static::lazy_static! {
    static ref PLUGIN_REGISTRY: Arc<RwLock<HashMap<String, Box<dyn ScriptEnginePlugin>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Plugin interface for third-party script engines
pub trait ScriptEnginePlugin: Send + Sync {
    /// Get the name of this engine
    fn engine_name(&self) -> &str;

    /// Get a description of this engine
    fn description(&self) -> String;

    /// Get the version of this engine
    fn version(&self) -> String;

    /// Create an instance of this engine
    fn create_engine(&self, config: Value) -> Result<Box<dyn ScriptEngineBridge>, LLMSpellError>;

    /// Get the features supported by this engine
    fn supported_features(&self) -> EngineFeatures;
}

/// Register a third-party engine plugin
pub fn register_engine_plugin<P: ScriptEnginePlugin + 'static>(plugin: P) {
    let mut registry = PLUGIN_REGISTRY.write().unwrap();
    registry.insert(plugin.engine_name().to_string(), Box::new(plugin));
}

/// Unregister an engine plugin
pub fn unregister_engine_plugin(name: &str) -> bool {
    let mut registry = PLUGIN_REGISTRY.write().unwrap();
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
