//! ABOUTME: Engine configuration definitions for llmspell
//! ABOUTME: Includes Lua, JavaScript, and custom engine configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Engine configurations
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct EngineConfigs {
    #[serde(default)]
    pub lua: LuaConfig,
    #[serde(default)]
    pub javascript: JSConfig,
    #[serde(flatten, default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Security/access level for script execution
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SecurityLevel {
    Safe,
    Privileged,
    Unrestricted,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Safe
    }
}

/// Standard library access level
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StdlibLevel {
    None,
    Basic,
    Standard,
    All,
}

impl Default for StdlibLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Lua engine configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LuaConfig {
    /// Standard library access level
    pub stdlib: StdlibLevel,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    /// Enable debug features
    pub enable_debug: bool,
    /// Execution timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Security level
    pub security_level: SecurityLevel,
}

impl Default for LuaConfig {
    fn default() -> Self {
        Self {
            stdlib: StdlibLevel::Standard,
            max_memory_bytes: Some(50_000_000), // 50MB
            enable_debug: false,
            timeout_ms: Some(30_000), // 30 seconds
            security_level: SecurityLevel::Safe,
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

    /// Set the standard library level
    #[must_use]
    pub const fn stdlib(mut self, level: StdlibLevel) -> Self {
        self.config.stdlib = level;
        self
    }

    /// Set the maximum memory limit
    #[must_use]
    pub const fn max_memory_bytes(mut self, bytes: Option<usize>) -> Self {
        self.config.max_memory_bytes = bytes;
        self
    }

    /// Enable or disable debug features
    #[must_use]
    pub const fn enable_debug(mut self, enable: bool) -> Self {
        self.config.enable_debug = enable;
        self
    }

    /// Set execution timeout
    #[must_use]
    pub const fn timeout_ms(mut self, timeout: Option<u64>) -> Self {
        self.config.timeout_ms = timeout;
        self
    }

    /// Set security level
    #[must_use]
    pub const fn security_level(mut self, level: SecurityLevel) -> Self {
        self.config.security_level = level;
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

/// JavaScript engine configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JSConfig {
    /// Enable strict mode
    pub strict_mode: bool,
    /// Maximum heap size in bytes
    pub max_heap_size_bytes: Option<usize>,
    /// Enable console API
    pub enable_console: bool,
    /// Execution timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Security level
    pub security_level: SecurityLevel,
}

impl Default for JSConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            max_heap_size_bytes: Some(50_000_000), // 50MB
            enable_console: true,
            timeout_ms: Some(30_000), // 30 seconds
            security_level: SecurityLevel::Safe,
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

    /// Set the maximum heap size
    #[must_use]
    pub const fn max_heap_size_bytes(mut self, bytes: Option<usize>) -> Self {
        self.config.max_heap_size_bytes = bytes;
        self
    }

    /// Enable or disable console API
    #[must_use]
    pub const fn enable_console(mut self, enable: bool) -> Self {
        self.config.enable_console = enable;
        self
    }

    /// Set execution timeout
    #[must_use]
    pub const fn timeout_ms(mut self, timeout: Option<u64>) -> Self {
        self.config.timeout_ms = timeout;
        self
    }

    /// Set security level
    #[must_use]
    pub const fn security_level(mut self, level: SecurityLevel) -> Self {
        self.config.security_level = level;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> JSConfig {
        self.config
    }
}

impl Default for JSConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_config_default() {
        let config = LuaConfig::default();
        assert_eq!(config.max_memory_bytes, Some(50_000_000));
        assert!(!config.enable_debug);
        assert_eq!(config.timeout_ms, Some(30_000));
    }

    #[test]
    fn test_lua_config_builder() {
        let config = LuaConfig::builder()
            .max_memory_bytes(Some(100_000_000))
            .enable_debug(true)
            .security_level(SecurityLevel::Privileged)
            .build();

        assert_eq!(config.max_memory_bytes, Some(100_000_000));
        assert!(config.enable_debug);
        assert!(matches!(config.security_level, SecurityLevel::Privileged));
    }

    #[test]
    fn test_js_config_default() {
        let config = JSConfig::default();
        assert!(config.strict_mode);
        assert_eq!(config.max_heap_size_bytes, Some(50_000_000));
        assert!(config.enable_console);
    }

    #[test]
    fn test_js_config_builder() {
        let config = JSConfig::builder()
            .strict_mode(false)
            .max_heap_size_bytes(Some(25_000_000))
            .security_level(SecurityLevel::Unrestricted)
            .build();

        assert!(!config.strict_mode);
        assert_eq!(config.max_heap_size_bytes, Some(25_000_000));
        assert!(matches!(config.security_level, SecurityLevel::Unrestricted));
    }

    #[test]
    fn test_engine_configs_serialization() {
        let configs = EngineConfigs {
            lua: LuaConfig::builder().enable_debug(true).build(),
            javascript: JSConfig::builder().strict_mode(false).build(),
            custom: HashMap::new(),
        };

        let serialized = serde_json::to_string(&configs).expect("Serialization should work");
        let deserialized: EngineConfigs =
            serde_json::from_str(&serialized).expect("Deserialization should work");

        assert!(deserialized.lua.enable_debug);
        assert!(!deserialized.javascript.strict_mode);
    }
}
