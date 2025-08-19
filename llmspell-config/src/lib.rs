//! ABOUTME: Central configuration management system for llmspell
//! ABOUTME: Handles TOML parsing, validation, and environment variable overrides

use anyhow::{Context, Result};
use llmspell_core::error::LLMSpellError;
use serde::{Deserialize, Serialize};
use std::env as std_env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::debug;

// Re-export engine configurations from bridge
pub use crate::engines::{EngineConfigs, JSConfig, LuaConfig};
pub use crate::env::{EnvCategory, EnvRegistry, EnvVarDef, EnvVarDefBuilder, IsolationMode};
pub use crate::providers::{ProviderConfig, ProviderManagerConfig, ProviderManagerConfigBuilder};
pub use crate::tools::{FileOperationsConfig, ToolsConfig};

pub mod engines;
pub mod env;
pub mod env_registry;
pub mod providers;
pub mod tools;
pub mod validation;

use crate::env_registry::register_standard_vars;

/// Configuration file discovery order
const CONFIG_SEARCH_PATHS: &[&str] = &[
    "llmspell.toml",
    ".llmspell.toml",
    "config/llmspell.toml",
    ".config/llmspell.toml",
];

/// Environment variable prefix (kept for documentation)
#[allow(dead_code)]
const ENV_PREFIX: &str = "LLMSPELL_";

/// Central LLMSpell configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LLMSpellConfig {
    /// Default script engine to use
    pub default_engine: String,
    /// Engine-specific configurations  
    pub engines: EngineConfigs,
    /// Provider configurations
    pub providers: ProviderManagerConfig,
    /// Global runtime settings
    pub runtime: GlobalRuntimeConfig,
    /// Tool-specific configurations
    pub tools: ToolsConfig,
    /// Hook system configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HookConfig>,
    /// Event system configuration
    pub events: EventsConfig,
}

impl Default for LLMSpellConfig {
    fn default() -> Self {
        Self {
            default_engine: "lua".to_string(),
            engines: EngineConfigs::default(),
            providers: ProviderManagerConfig::default(),
            runtime: GlobalRuntimeConfig::default(),
            tools: ToolsConfig::default(),
            hooks: None,
            events: EventsConfig::default(),
        }
    }
}

impl LLMSpellConfig {
    /// Create a new builder for `LLMSpellConfig`
    #[must_use]
    pub fn builder() -> LLMSpellConfigBuilder {
        LLMSpellConfigBuilder::new()
    }

    /// Load configuration from TOML file with validation
    pub async fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        Self::from_toml(&content)
    }

    /// Parse TOML content with environment variable overrides and validation
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        let mut config: LLMSpellConfig = toml::from_str(content)?;

        // Use registry for environment overrides
        config.apply_env_registry()?;
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides using the centralized registry
    pub fn apply_env_registry(&mut self) -> Result<(), ConfigError> {
        // Create registry and load standard variables
        let registry = EnvRegistry::new();
        register_standard_vars(&registry).map_err(|e| ConfigError::Environment { message: e })?;

        // Load environment variables
        registry
            .load_from_env()
            .map_err(|e| ConfigError::Environment { message: e })?;

        // Build config from registry
        let env_config = registry
            .build_config()
            .map_err(|e| ConfigError::Environment { message: e })?;

        // Merge environment config into self
        self.merge_from_json(&env_config)?;

        Ok(())
    }

    /// Merge values from JSON config (from registry)
    fn merge_from_json(&mut self, json: &serde_json::Value) -> Result<(), ConfigError> {
        // Merge top-level values
        if let Some(engine) = json.get("default_engine").and_then(|v| v.as_str()) {
            debug!("Overriding default engine from env: {}", engine);
            self.default_engine = engine.to_string();
        }

        // Merge runtime values
        if let Some(runtime) = json.get("runtime").and_then(|v| v.as_object()) {
            if let Some(max_scripts) = runtime
                .get("max_concurrent_scripts")
                .and_then(|v| v.as_u64())
            {
                debug!(
                    "Overriding max_concurrent_scripts from env: {}",
                    max_scripts
                );
                self.runtime.max_concurrent_scripts = max_scripts as usize;
            }

            if let Some(timeout) = runtime
                .get("script_timeout_seconds")
                .and_then(|v| v.as_u64())
            {
                debug!("Overriding script_timeout_seconds from env: {}", timeout);
                self.runtime.script_timeout_seconds = timeout;
            }

            // Merge security settings
            if let Some(security) = runtime.get("security").and_then(|v| v.as_object()) {
                if let Some(allow_file) =
                    security.get("allow_file_access").and_then(|v| v.as_bool())
                {
                    debug!("Overriding allow_file_access from env: {}", allow_file);
                    self.runtime.security.allow_file_access = allow_file;
                }

                if let Some(allow_network) = security
                    .get("allow_network_access")
                    .and_then(|v| v.as_bool())
                {
                    debug!(
                        "Overriding allow_network_access from env: {}",
                        allow_network
                    );
                    self.runtime.security.allow_network_access = allow_network;
                }

                if let Some(allow_spawn) = security
                    .get("allow_process_spawn")
                    .and_then(|v| v.as_bool())
                {
                    self.runtime.security.allow_process_spawn = allow_spawn;
                }

                if let Some(max_memory) = security.get("max_memory_bytes").and_then(|v| v.as_u64())
                {
                    self.runtime.security.max_memory_bytes = Some(max_memory as usize);
                }

                if let Some(max_time) = security
                    .get("max_execution_time_ms")
                    .and_then(|v| v.as_u64())
                {
                    self.runtime.security.max_execution_time_ms = Some(max_time);
                }
            }

            // Merge state persistence settings
            if let Some(state) = runtime.get("state_persistence").and_then(|v| v.as_object()) {
                if let Some(backend) = state.get("backend_type").and_then(|v| v.as_str()) {
                    self.runtime.state_persistence.backend_type = backend.to_string();
                }

                // Flattened structure - direct access
                if let Some(enabled) = state.get("enabled").and_then(|v| v.as_bool()) {
                    self.runtime.state_persistence.enabled = enabled;
                }
                if let Some(migration) = state.get("migration_enabled").and_then(|v| v.as_bool()) {
                    self.runtime.state_persistence.migration_enabled = migration;
                }
                if let Some(backup_on_migration) =
                    state.get("backup_on_migration").and_then(|v| v.as_bool())
                {
                    self.runtime.state_persistence.backup_on_migration = backup_on_migration;
                }
                if let Some(backup_enabled) = state.get("backup_enabled").and_then(|v| v.as_bool())
                {
                    self.runtime.state_persistence.backup_enabled = backup_enabled;
                }
            }

            // Merge session settings
            if let Some(sessions) = runtime.get("sessions").and_then(|v| v.as_object()) {
                if let Some(enabled) = sessions.get("enabled").and_then(|v| v.as_bool()) {
                    self.runtime.sessions.enabled = enabled;
                }
                if let Some(backend) = sessions.get("storage_backend").and_then(|v| v.as_str()) {
                    self.runtime.sessions.storage_backend = backend.to_string();
                }
            }
        }

        // Merge provider configurations
        if let Some(providers) = json.get("providers").and_then(|v| v.as_object()) {
            // Handle flattened structure - provider configs are direct children
            for (name, config) in providers {
                // Skip default_provider field, only process provider configs
                if name == "default_provider" {
                    continue;
                }

                if let Some(provider_obj) = config.as_object() {
                    // Check if provider already exists in config
                    if let Some(existing_provider) = self.providers.providers.get_mut(name) {
                        // Provider exists - only update fields that are present in env config
                        if let Some(api_key) = provider_obj.get("api_key").and_then(|v| v.as_str())
                        {
                            existing_provider.api_key = Some(api_key.to_string());
                        }
                        if let Some(base_url) =
                            provider_obj.get("base_url").and_then(|v| v.as_str())
                        {
                            existing_provider.base_url = Some(base_url.to_string());
                        }
                        if let Some(model) =
                            provider_obj.get("default_model").and_then(|v| v.as_str())
                        {
                            existing_provider.default_model = Some(model.to_string());
                        }
                        if let Some(timeout) =
                            provider_obj.get("timeout_seconds").and_then(|v| v.as_u64())
                        {
                            existing_provider.timeout_seconds = Some(timeout);
                        }
                        if let Some(max_retries) =
                            provider_obj.get("max_retries").and_then(|v| v.as_u64())
                        {
                            existing_provider.max_retries = Some(max_retries as u32);
                        }
                        // Only update other fields if they're present
                        if let Some(provider_type) =
                            provider_obj.get("provider_type").and_then(|v| v.as_str())
                        {
                            existing_provider.provider_type = provider_type.to_string();
                        }
                        if let Some(enabled) = provider_obj.get("enabled").and_then(|v| v.as_bool())
                        {
                            existing_provider.enabled = enabled;
                        }
                        // Do NOT insert/replace - we already modified in place
                    } else {
                        // Provider doesn't exist in config - only create if it has minimum required fields
                        // Don't create incomplete providers from just API keys
                        if provider_obj.contains_key("default_model")
                            || provider_obj.contains_key("provider_type")
                        {
                            let mut provider_config = ProviderConfig {
                                name: name.clone(),
                                provider_type: provider_obj
                                    .get("provider_type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(name)
                                    .to_string(),
                                enabled: true,
                                ..Default::default()
                            };

                            if let Some(api_key) =
                                provider_obj.get("api_key").and_then(|v| v.as_str())
                            {
                                provider_config.api_key = Some(api_key.to_string());
                            }
                            if let Some(base_url) =
                                provider_obj.get("base_url").and_then(|v| v.as_str())
                            {
                                provider_config.base_url = Some(base_url.to_string());
                            }
                            if let Some(model) =
                                provider_obj.get("default_model").and_then(|v| v.as_str())
                            {
                                provider_config.default_model = Some(model.to_string());
                            }

                            self.providers
                                .providers
                                .insert(name.clone(), provider_config);
                        }
                        // Otherwise skip - don't create incomplete provider from just api_key
                    }
                }
            }
        }

        // Merge tool configurations
        if let Some(tools) = json.get("tools").and_then(|v| v.as_object()) {
            if let Some(file_ops) = tools.get("file_operations").and_then(|v| v.as_object()) {
                if let Some(enabled) = file_ops.get("enabled").and_then(|v| v.as_bool()) {
                    self.tools.file_operations.enabled = enabled;
                }
                if let Some(max_size) = file_ops.get("max_file_size").and_then(|v| v.as_u64()) {
                    self.tools.file_operations.max_file_size = max_size as usize;
                }
                // Handle allowed_paths - can be either string (from env) or array (from JSON)
                if let Some(paths_value) = file_ops.get("allowed_paths") {
                    if let Some(paths_str) = paths_value.as_str() {
                        // From environment variable - comma-separated string
                        self.tools.file_operations.allowed_paths =
                            paths_str.split(',').map(|s| s.trim().to_string()).collect();
                    } else if let Some(paths_array) = paths_value.as_array() {
                        // From JSON - array of strings
                        self.tools.file_operations.allowed_paths = paths_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
            }
            if let Some(network) = tools.get("network").and_then(|v| v.as_object()) {
                let net_config = self.tools.network.get_or_insert(Default::default());
                if let Some(timeout) = network.get("timeout_seconds").and_then(|v| v.as_u64()) {
                    net_config.timeout_seconds = timeout;
                }
            }
            if let Some(rate_limit) = tools.get("rate_limit_per_minute").and_then(|v| v.as_u64()) {
                self.tools.rate_limit_per_minute = Some(rate_limit as u32);
            }
        }

        // Merge hook configuration
        if let Some(hooks) = json.get("hooks").and_then(|v| v.as_object()) {
            let hook_config = self.hooks.get_or_insert(Default::default());
            if let Some(enabled) = hooks.get("enabled").and_then(|v| v.as_bool()) {
                hook_config.enabled = enabled;
            }
            if let Some(rate_limit) = hooks.get("rate_limit_per_minute").and_then(|v| v.as_u64()) {
                hook_config.rate_limit_per_minute = Some(rate_limit as u32);
            }
        }

        // Merge events configuration
        if let Some(events) = json.get("events").and_then(|v| v.as_object()) {
            if let Some(enabled) = events.get("enabled").and_then(|v| v.as_bool()) {
                debug!("Overriding events.enabled from env: {}", enabled);
                self.events.enabled = enabled;
            }

            if let Some(buffer_size) = events.get("buffer_size").and_then(|v| v.as_u64()) {
                debug!("Overriding events.buffer_size from env: {}", buffer_size);
                self.events.buffer_size = buffer_size as usize;
            }

            if let Some(emit_timing) = events.get("emit_timing_events").and_then(|v| v.as_bool()) {
                debug!(
                    "Overriding events.emit_timing_events from env: {}",
                    emit_timing
                );
                self.events.emit_timing_events = emit_timing;
            }

            if let Some(emit_state) = events.get("emit_state_events").and_then(|v| v.as_bool()) {
                debug!(
                    "Overriding events.emit_state_events from env: {}",
                    emit_state
                );
                self.events.emit_state_events = emit_state;
            }

            if let Some(emit_debug) = events.get("emit_debug_events").and_then(|v| v.as_bool()) {
                debug!(
                    "Overriding events.emit_debug_events from env: {}",
                    emit_debug
                );
                self.events.emit_debug_events = emit_debug;
            }

            if let Some(max_events) = events.get("max_events_per_second").and_then(|v| v.as_u64()) {
                debug!(
                    "Overriding events.max_events_per_second from env: {}",
                    max_events
                );
                self.events.max_events_per_second = Some(max_events as u32);
            }

            // Merge filtering configuration
            if let Some(filtering) = events.get("filtering").and_then(|v| v.as_object()) {
                if let Some(include_types_value) = filtering.get("include_types") {
                    if let Some(include_types_str) = include_types_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.include_types = include_types_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(include_types_array) = include_types_value.as_array() {
                        // From JSON - array of strings
                        self.events.filtering.include_types = include_types_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }

                if let Some(exclude_types_value) = filtering.get("exclude_types") {
                    if let Some(exclude_types_str) = exclude_types_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.exclude_types = exclude_types_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(exclude_types_array) = exclude_types_value.as_array() {
                        // From JSON - array of strings
                        self.events.filtering.exclude_types = exclude_types_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }

                if let Some(include_components_value) = filtering.get("include_components") {
                    if let Some(include_components_str) = include_components_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.include_components = include_components_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(include_components_array) =
                        include_components_value.as_array()
                    {
                        // From JSON - array of strings
                        self.events.filtering.include_components = include_components_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }

                if let Some(exclude_components_value) = filtering.get("exclude_components") {
                    if let Some(exclude_components_str) = exclude_components_value.as_str() {
                        // From environment variable - comma-separated string
                        self.events.filtering.exclude_components = exclude_components_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    } else if let Some(exclude_components_array) =
                        exclude_components_value.as_array()
                    {
                        // From JSON - array of strings
                        self.events.filtering.exclude_components = exclude_components_array
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
            }

            // Merge export configuration
            if let Some(export) = events.get("export").and_then(|v| v.as_object()) {
                if let Some(stdout) = export.get("stdout").and_then(|v| v.as_bool()) {
                    debug!("Overriding events.export.stdout from env: {}", stdout);
                    self.events.export.stdout = stdout;
                }

                if let Some(file) = export.get("file").and_then(|v| v.as_str()) {
                    debug!("Overriding events.export.file from env: {}", file);
                    self.events.export.file = Some(file.to_string());
                }

                if let Some(webhook) = export.get("webhook").and_then(|v| v.as_str()) {
                    debug!("Overriding events.export.webhook from env: {}", webhook);
                    self.events.export.webhook = Some(webhook.to_string());
                }

                if let Some(pretty_json) = export.get("pretty_json").and_then(|v| v.as_bool()) {
                    debug!(
                        "Overriding events.export.pretty_json from env: {}",
                        pretty_json
                    );
                    self.events.export.pretty_json = pretty_json;
                }
            }
        }

        Ok(())
    }

    /// Apply environment variable overrides (DEPRECATED - use apply_env_registry)
    #[deprecated(note = "Use apply_env_registry() for centralized environment handling")]
    pub fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        self.apply_env_registry()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        validation::validate_config(self)
    }

    /// Get engine-specific configuration
    pub fn get_engine_config(&self, engine_name: &str) -> Result<serde_json::Value, ConfigError> {
        match engine_name {
            "lua" => Ok(serde_json::to_value(&self.engines.lua)?),
            "javascript" | "js" => Ok(serde_json::to_value(&self.engines.javascript)?),
            custom => {
                self.engines
                    .custom
                    .get(custom)
                    .cloned()
                    .ok_or_else(|| ConfigError::Validation {
                        field: Some("engine".to_string()),
                        message: format!("Engine configuration not found for '{custom}'"),
                    })
            }
        }
    }

    /// Check if an engine is configured
    #[must_use]
    pub fn supports_engine(&self, engine_name: &str) -> bool {
        match engine_name {
            "lua" | "javascript" | "js" => true,
            custom => self.engines.custom.contains_key(custom),
        }
    }

    /// Discover configuration file in standard locations
    pub async fn discover_config_file() -> Result<Option<PathBuf>, ConfigError> {
        // Check current directory first
        for path in CONFIG_SEARCH_PATHS {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(Some(path));
            }
        }

        // Check home directory
        if let Ok(home_dir) = std_env::var("HOME").or_else(|_| std_env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home_dir);

            for filename in &[".llmspell.toml", ".config/llmspell.toml"] {
                let path = home_path.join(filename);
                if path.exists() {
                    return Ok(Some(path));
                }
            }
        }

        // Check system config directories (Linux/macOS style)
        if let Ok(xdg_config) = std_env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config)
                .join("llmspell")
                .join("config.toml");
            if path.exists() {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }

    /// Load configuration with automatic discovery
    pub async fn load_with_discovery(explicit_path: Option<&Path>) -> Result<Self, ConfigError> {
        // If explicit path provided, use it
        if let Some(path) = explicit_path {
            if path.exists() {
                return Self::load_from_file(path).await;
            } else {
                return Err(ConfigError::NotFound {
                    path: path.to_string_lossy().to_string(),
                    message: "Explicitly specified config file not found".to_string(),
                });
            }
        }

        // Discover config file
        if let Some(discovered_path) = Self::discover_config_file().await? {
            return Self::load_from_file(&discovered_path).await;
        }

        // No config file found, use defaults with environment overrides
        let mut config = Self::default();
        config.apply_env_registry()?;
        config.validate()?;

        Ok(config)
    }
}

/// Builder for `LLMSpellConfig`
#[derive(Debug, Clone)]
pub struct LLMSpellConfigBuilder {
    config: LLMSpellConfig,
}

impl LLMSpellConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: LLMSpellConfig::default(),
        }
    }

    /// Set the default script engine
    #[must_use]
    pub fn default_engine(mut self, engine: impl Into<String>) -> Self {
        self.config.default_engine = engine.into();
        self
    }

    /// Set the engine configurations
    #[must_use]
    pub fn engines(mut self, engines: EngineConfigs) -> Self {
        self.config.engines = engines;
        self
    }

    /// Set the provider configuration
    #[must_use]
    pub fn providers(mut self, providers: ProviderManagerConfig) -> Self {
        self.config.providers = providers;
        self
    }

    /// Set the global runtime configuration
    #[must_use]
    pub fn runtime(mut self, runtime: GlobalRuntimeConfig) -> Self {
        self.config.runtime = runtime;
        self
    }

    /// Set the tools configuration
    #[must_use]
    pub fn tools(mut self, tools: ToolsConfig) -> Self {
        self.config.tools = tools;
        self
    }

    /// Set the events configuration
    #[must_use]
    pub fn events(mut self, events: EventsConfig) -> Self {
        self.config.events = events;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> LLMSpellConfig {
        self.config
    }
}

impl Default for LLMSpellConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Global runtime configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GlobalRuntimeConfig {
    /// Maximum concurrent scripts
    pub max_concurrent_scripts: usize,
    /// Script execution timeout in seconds
    pub script_timeout_seconds: u64,
    /// Enable streaming by default
    pub enable_streaming: bool,
    /// Security settings
    pub security: SecurityConfig,
    /// State persistence settings
    pub state_persistence: StatePersistenceConfig,
    /// Session management settings
    pub sessions: SessionConfig,
}

impl Default for GlobalRuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_scripts: 10,
            script_timeout_seconds: 300,
            enable_streaming: true,
            security: SecurityConfig::default(),
            state_persistence: StatePersistenceConfig::default(),
            sessions: SessionConfig::default(),
        }
    }
}

impl GlobalRuntimeConfig {
    /// Create a new builder for `GlobalRuntimeConfig`
    #[must_use]
    pub fn builder() -> GlobalRuntimeConfigBuilder {
        GlobalRuntimeConfigBuilder::new()
    }
}

/// Builder for `GlobalRuntimeConfig`
#[derive(Debug, Clone)]
pub struct GlobalRuntimeConfigBuilder {
    config: GlobalRuntimeConfig,
}

impl GlobalRuntimeConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GlobalRuntimeConfig::default(),
        }
    }

    /// Set the maximum concurrent scripts
    #[must_use]
    pub const fn max_concurrent_scripts(mut self, max: usize) -> Self {
        self.config.max_concurrent_scripts = max;
        self
    }

    /// Set the script execution timeout in seconds
    #[must_use]
    pub const fn script_timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.script_timeout_seconds = timeout;
        self
    }

    /// Enable or disable streaming
    #[must_use]
    pub const fn enable_streaming(mut self, enable: bool) -> Self {
        self.config.enable_streaming = enable;
        self
    }

    /// Set the security configuration
    #[must_use]
    pub const fn security(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }

    /// Set the state persistence configuration
    #[must_use]
    pub fn state_persistence(mut self, persistence: StatePersistenceConfig) -> Self {
        self.config.state_persistence = persistence;
        self
    }

    /// Set the session configuration
    #[must_use]
    pub fn sessions(mut self, sessions: SessionConfig) -> Self {
        self.config.sessions = sessions;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> GlobalRuntimeConfig {
        self.config
    }
}

impl Default for GlobalRuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Allow file system access
    pub allow_file_access: bool,
    /// Allow network access
    pub allow_network_access: bool,
    /// Allow process spawning
    pub allow_process_spawn: bool,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_file_access: false,
            allow_network_access: true,
            allow_process_spawn: false,
            max_memory_bytes: Some(50_000_000),   // 50MB
            max_execution_time_ms: Some(300_000), // 5 minutes
        }
    }
}

/// State persistence configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StatePersistenceConfig {
    /// Enable state persistence (flattened from flags.core.enabled)
    pub enabled: bool,
    /// Enable migration functionality (flattened from flags.core.migration_enabled)
    pub migration_enabled: bool,
    /// Automatic backup on migration (flattened from flags.backup.backup_on_migration)
    pub backup_on_migration: bool,
    /// Enable backup functionality (flattened from flags.backup.backup_enabled)
    pub backup_enabled: bool,
    /// Backend type for storage (memory, file, redis, etc.)
    pub backend_type: String,
    /// Directory for schema definitions
    pub schema_directory: Option<String>,
    /// Maximum state size per key in bytes
    pub max_state_size_bytes: Option<usize>,
    /// Backup configuration
    pub backup: Option<BackupConfig>,
}

/// Backup configuration for state persistence
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupConfig {
    /// Directory for backup storage
    pub backup_dir: Option<String>,
    /// Enable compression for backups
    pub compression_enabled: bool,
    /// Compression type to use
    pub compression_type: String,
    /// Compression level (1-9)
    pub compression_level: u8,
    /// Enable incremental backups
    pub incremental_enabled: bool,
    /// Maximum number of backups to keep
    pub max_backups: Option<usize>,
    /// Maximum age of backups in seconds
    pub max_backup_age: Option<u64>,
}

impl Default for StatePersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true, // In-memory state by default
            migration_enabled: false,
            backup_on_migration: true,
            backup_enabled: false,
            backend_type: "memory".to_string(),
            schema_directory: None,
            max_state_size_bytes: Some(10_000_000), // 10MB per key
            backup: None,
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: Some("./backups".to_string()),
            compression_enabled: true,
            compression_type: "zstd".to_string(),
            compression_level: 3,
            incremental_enabled: true,
            max_backups: Some(10),
            max_backup_age: Some(2_592_000), // 30 days
        }
    }
}

/// Hook system configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HookConfig {
    /// Enable hook system
    pub enabled: bool,
    /// Rate limiting for hooks (executions per minute)
    pub rate_limit_per_minute: Option<u32>,
    /// Hook timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: Option<f64>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate_limit_per_minute: Some(100),
            timeout_ms: Some(5000),
            circuit_breaker_threshold: Some(0.01), // 1% overhead threshold
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Enable session management
    pub enabled: bool,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Maximum artifacts per session
    pub max_artifacts_per_session: usize,
    /// Artifact compression threshold in bytes
    pub artifact_compression_threshold: usize,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Storage backend type (memory, sled)
    pub storage_backend: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_sessions: 100,
            max_artifacts_per_session: 1000,
            artifact_compression_threshold: 10240, // 10KB
            session_timeout_seconds: 3600,         // 1 hour
            storage_backend: "memory".to_string(),
        }
    }
}

/// Event system configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EventsConfig {
    /// Enable event system globally
    pub enabled: bool,
    /// EventBus buffer size for queuing events
    pub buffer_size: usize,
    /// Enable timing/performance events
    pub emit_timing_events: bool,
    /// Enable state change events
    pub emit_state_events: bool,
    /// Enable debug-level events
    pub emit_debug_events: bool,
    /// Maximum events per second (rate limiting)
    pub max_events_per_second: Option<u32>,
    /// Event filtering configuration
    pub filtering: EventFilterConfig,
    /// Event export configuration
    pub export: EventExportConfig,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            emit_timing_events: true,
            emit_state_events: false,
            emit_debug_events: false,
            max_events_per_second: None,
            filtering: EventFilterConfig::default(),
            export: EventExportConfig::default(),
        }
    }
}

/// Event filtering configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EventFilterConfig {
    /// Event types to include (glob patterns)
    pub include_types: Vec<String>,
    /// Event types to exclude (glob patterns)
    pub exclude_types: Vec<String>,
    /// Component IDs to include (glob patterns)
    pub include_components: Vec<String>,
    /// Component IDs to exclude (glob patterns)
    pub exclude_components: Vec<String>,
}

impl Default for EventFilterConfig {
    fn default() -> Self {
        Self {
            include_types: vec!["*".to_string()], // Include all by default
            exclude_types: Vec::new(),
            include_components: vec!["*".to_string()],
            exclude_components: Vec::new(),
        }
    }
}

/// Event export configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct EventExportConfig {
    /// Export events to stdout (for debugging)
    pub stdout: bool,
    /// Export events to file
    pub file: Option<String>,
    /// Export events to webhook
    pub webhook: Option<String>,
    /// Pretty-print JSON output
    pub pretty_json: bool,
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path} - {message}")]
    NotFound { path: String, message: String },

    #[error("Configuration validation failed in field '{field:?}': {message}")]
    Validation {
        field: Option<String>,
        message: String,
    },

    #[error("Environment variable parsing error: {message}")]
    Environment { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("General error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<ConfigError> for LLMSpellError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::Validation { field, message } => {
                LLMSpellError::Validation { field, message }
            }
            ConfigError::NotFound { path, message } => LLMSpellError::Configuration {
                message: format!("Configuration file not found: {} - {}", path, message),
                source: None,
            },
            _ => LLMSpellError::Configuration {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llmspell_config_default() {
        let config = LLMSpellConfig::default();
        assert_eq!(config.default_engine, "lua");
        assert!(config.supports_engine("lua"));
        assert!(config.supports_engine("javascript"));
        assert!(!config.supports_engine("python"));
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(!config.allow_file_access);
        assert!(config.allow_network_access);
        assert!(!config.allow_process_spawn);
        assert_eq!(config.max_memory_bytes, Some(50_000_000));
        assert_eq!(config.max_execution_time_ms, Some(300_000));
    }

    #[test]
    fn test_config_builder() {
        let config = LLMSpellConfig::builder()
            .default_engine("javascript")
            .build();

        assert_eq!(config.default_engine, "javascript");
    }

    #[test]
    fn test_minimal_toml_config() {
        let toml_str = r#"default_engine = "lua""#;
        let result = LLMSpellConfig::from_toml(toml_str);

        // Should parse successfully
        assert!(
            result.is_ok(),
            "Failed to parse minimal config: {:?}",
            result
        );

        let config = result.unwrap();
        assert_eq!(config.default_engine, "lua");
    }

    #[test]
    fn test_empty_toml_config() {
        let toml_str = "";
        let result = LLMSpellConfig::from_toml(toml_str);

        // Should use defaults
        assert!(result.is_ok(), "Failed to parse empty config: {:?}", result);

        let config = result.unwrap();
        assert_eq!(config.default_engine, "lua");
    }
}
