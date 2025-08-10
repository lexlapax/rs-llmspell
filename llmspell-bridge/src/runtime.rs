//! ABOUTME: Language-agnostic script runtime using `ScriptEngineBridge` abstraction
//! ABOUTME: Central execution orchestrator supporting multiple script engines

use crate::{
    engine::{EngineFactory, JSConfig, LuaConfig, ScriptEngineBridge, ScriptOutput, ScriptStream},
    providers::{ProviderManager, ProviderManagerConfig},
    registry::ComponentRegistry,
    tools::register_all_tools,
};
use llmspell_core::error::LLMSpellError;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Central script runtime that uses `ScriptEngineBridge` abstraction
///
/// The `ScriptRuntime` is the main entry point for executing scripts in `LLMSpell`.
/// It provides a language-agnostic interface that can work with multiple script
/// engines (Lua, JavaScript, Python, etc.) through the `ScriptEngineBridge` trait.
///
/// # Examples
///
/// ## Basic Script Execution
///
/// ```rust,no_run
/// use llmspell_bridge::{ScriptRuntime, RuntimeConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a runtime with default configuration
/// let config = RuntimeConfig::default();
/// let runtime = ScriptRuntime::new_with_lua(config).await?;
///
/// // Execute a simple Lua script
/// let output = runtime.execute_script("return 42").await?;
/// println!("Result: {:?}", output.output);
/// # Ok(())
/// # }
/// ```
///
/// ## Working with Agents (Placeholder - Phase 2)
///
/// ```rust,no_run
/// use llmspell_bridge::{ScriptRuntime, RuntimeConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runtime = ScriptRuntime::new_with_lua(RuntimeConfig::default()).await?;
///
/// let script = r#"
///     -- Create an agent (placeholder functionality)
///     local agent = Agent.create({
///         name = "assistant",
///         system_prompt = "You are a helpful assistant"
///     })
///     
///     -- Execute the agent (returns placeholder response)
///     local response = agent:execute("Hello!")
///     return response.text
/// "#;
///
/// let output = runtime.execute_script(script).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Streaming Execution
///
/// ```rust,no_run
/// use llmspell_bridge::{ScriptRuntime, RuntimeConfig};
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runtime = ScriptRuntime::new_with_lua(RuntimeConfig::default()).await?;
///
/// // Check if streaming is supported
/// if runtime.supports_streaming() {
///     let mut stream = runtime.execute_script_streaming("return 'streaming output'").await?;
///     
///     // Process chunks as they arrive
///     while let Some(chunk) = stream.stream.next().await {
///         let chunk = chunk?;
///         println!("Received chunk: {:?}", chunk);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct ScriptRuntime {
    /// Language-agnostic script engine
    engine: Box<dyn ScriptEngineBridge>,
    /// Component registry for agents, tools, workflows
    registry: Arc<ComponentRegistry>,
    /// Provider manager for LLM access
    provider_manager: Arc<ProviderManager>,
    /// Execution context
    execution_context: Arc<RwLock<crate::engine::ExecutionContext>>,
    /// Runtime configuration
    _config: RuntimeConfig,
}

impl ScriptRuntime {
    /// Create a new runtime with Lua engine
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_bridge::{ScriptRuntime, RuntimeConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // With default configuration
    /// let runtime = ScriptRuntime::new_with_lua(RuntimeConfig::default()).await?;
    ///
    /// // With custom configuration
    /// let mut config = RuntimeConfig::default();
    /// config.runtime.security.allow_file_access = true;
    /// let runtime = ScriptRuntime::new_with_lua(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    pub async fn new_with_lua(config: RuntimeConfig) -> Result<Self, LLMSpellError> {
        let lua_config = config.engines.lua.clone();
        let config_arc = Arc::new(config.clone());
        let engine =
            EngineFactory::create_lua_engine_with_runtime(&lua_config, Some(config_arc.clone()))?;
        Self::new_with_engine(engine, config).await
    }

    /// Create a new runtime with JavaScript engine
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    pub async fn new_with_javascript(config: RuntimeConfig) -> Result<Self, LLMSpellError> {
        let js_config = config.engines.javascript.clone();
        let engine = EngineFactory::create_javascript_engine(&js_config)?;
        Self::new_with_engine(engine, config).await
    }

    /// Create a new runtime with a specific engine by name
    ///
    /// # Errors
    ///
    /// Returns an error if the engine is not found or runtime initialization fails
    pub async fn new_with_engine_name(
        engine_name: &str,
        config: RuntimeConfig,
    ) -> Result<Self, LLMSpellError> {
        let engine_config = config.get_engine_config(engine_name)?;
        let engine = EngineFactory::create_from_name(engine_name, &engine_config)?;
        Self::new_with_engine(engine, config).await
    }

    /// Core initialization with any engine
    async fn new_with_engine(
        mut engine: Box<dyn ScriptEngineBridge>,
        config: RuntimeConfig,
    ) -> Result<Self, LLMSpellError> {
        // Create component registry
        let registry = Arc::new(ComponentRegistry::new());

        // Register all Phase 2 tools with the registry
        register_all_tools(&registry).map_err(|e| LLMSpellError::Component {
            message: format!("Failed to register tools: {e}"),
            source: None,
        })?;

        // Create provider manager
        let provider_config = config.providers.clone();
        let provider_manager = Arc::new(ProviderManager::new(provider_config).await?);

        // Inject APIs into the engine
        engine.inject_apis(&registry, &provider_manager)?;

        // Create execution context
        let execution_context = Arc::new(RwLock::new(crate::engine::ExecutionContext {
            working_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            environment: std::env::vars().collect(),
            state: serde_json::Value::Object(serde_json::Map::new()),
            security: config.runtime.security.clone().into(),
        }));

        Ok(Self {
            engine,
            registry,
            provider_manager,
            execution_context,
            _config: config,
        })
    }

    /// Execute a script and return the output
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails
    pub async fn execute_script(&self, script: &str) -> Result<ScriptOutput, LLMSpellError> {
        self.engine.execute_script(script).await
    }

    /// Execute a script with streaming output
    ///
    /// # Errors
    ///
    /// Returns an error if the engine doesn't support streaming or script execution fails
    pub async fn execute_script_streaming(
        &self,
        script: &str,
    ) -> Result<ScriptStream, LLMSpellError> {
        if !self.engine.supports_streaming() {
            return Err(LLMSpellError::Component {
                message: format!(
                    "{} engine does not support streaming execution",
                    self.engine.get_engine_name()
                ),
                source: None,
            });
        }
        self.engine.execute_script_streaming(script).await
    }

    /// Get the name of the current engine
    #[must_use]
    pub fn get_engine_name(&self) -> &'static str {
        self.engine.get_engine_name()
    }

    /// Check if the engine supports streaming
    #[must_use]
    pub fn supports_streaming(&self) -> bool {
        self.engine.supports_streaming()
    }

    /// Check if the engine supports multimodal content
    #[must_use]
    pub fn supports_multimodal(&self) -> bool {
        self.engine.supports_multimodal()
    }

    /// Get the engine's supported features
    #[must_use]
    pub fn get_engine_features(&self) -> crate::engine::EngineFeatures {
        self.engine.supported_features()
    }

    /// Get the component registry
    #[must_use]
    pub const fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }

    /// Get the provider manager
    #[must_use]
    pub const fn provider_manager(&self) -> &Arc<ProviderManager> {
        &self.provider_manager
    }

    /// Get the current execution context
    ///
    /// # Panics
    ///
    /// Panics if the execution context lock is poisoned
    #[must_use]
    pub fn get_execution_context(&self) -> crate::engine::ExecutionContext {
        self.execution_context.read().unwrap().clone()
    }

    /// Update the execution context
    ///
    /// # Errors
    ///
    /// Returns an error if the context lock is poisoned
    ///
    /// # Panics
    ///
    /// Panics if the write lock cannot be acquired
    pub fn set_execution_context(
        &self,
        context: crate::engine::ExecutionContext,
    ) -> Result<(), LLMSpellError> {
        {
            let mut ctx = self.execution_context.write().unwrap();
            *ctx = context;
        } // Explicitly drop the lock here
        Ok(())
    }
}

/// Runtime configuration supporting multiple engines
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeConfig {
    /// Default script engine to use
    pub default_engine: String,
    /// Engine-specific configurations
    pub engines: EngineConfigs,
    /// Provider configurations
    pub providers: ProviderManagerConfig,
    /// Global runtime settings
    pub runtime: GlobalRuntimeConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_engine: "lua".to_string(),
            engines: EngineConfigs::default(),
            providers: ProviderManagerConfig::default(),
            runtime: GlobalRuntimeConfig::default(),
        }
    }
}

impl RuntimeConfig {
    /// Create a new builder for `RuntimeConfig`
    #[must_use]
    pub fn builder() -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::new()
    }

    /// Get engine-specific configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the engine is not found or configuration serialization fails
    pub fn get_engine_config(&self, engine_name: &str) -> Result<serde_json::Value, LLMSpellError> {
        match engine_name {
            "lua" => Ok(serde_json::to_value(&self.engines.lua)?),
            "javascript" | "js" => Ok(serde_json::to_value(&self.engines.javascript)?),
            custom => {
                self.engines
                    .custom
                    .get(custom)
                    .cloned()
                    .ok_or_else(|| LLMSpellError::Validation {
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
}

/// Builder for `RuntimeConfig`
#[derive(Debug, Clone)]
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl RuntimeConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default(),
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

    /// Set the Lua configuration
    #[must_use]
    pub fn lua_config(mut self, config: LuaConfig) -> Self {
        self.config.engines.lua = config;
        self
    }

    /// Set the JavaScript configuration
    #[must_use]
    pub fn javascript_config(mut self, config: JSConfig) -> Self {
        self.config.engines.javascript = config;
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

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> RuntimeConfig {
        self.config
    }
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine configurations
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EngineConfigs {
    pub lua: LuaConfig,
    pub javascript: JSConfig,
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
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
    pub fn max_concurrent_scripts(mut self, max: usize) -> Self {
        self.config.max_concurrent_scripts = max;
        self
    }

    /// Set the script execution timeout in seconds
    #[must_use]
    pub fn script_timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.script_timeout_seconds = timeout;
        self
    }

    /// Enable or disable streaming
    #[must_use]
    pub fn enable_streaming(mut self, enable: bool) -> Self {
        self.config.enable_streaming = enable;
        self
    }

    /// Set the security configuration
    #[must_use]
    pub fn security(mut self, security: SecurityConfig) -> Self {
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
    /// Enable state persistence
    pub enabled: bool,
    /// Backend type for storage (memory, file, redis, etc.)
    pub backend_type: String,
    /// Enable migration functionality
    pub migration_enabled: bool,
    /// Directory for schema definitions
    pub schema_directory: Option<String>,
    /// Automatic backup on migration
    pub backup_on_migration: bool,
    /// Maximum state size per key in bytes
    pub max_state_size_bytes: Option<usize>,
    /// Enable backup functionality
    pub backup_enabled: bool,
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
            enabled: false,
            backend_type: "memory".to_string(),
            migration_enabled: false,
            schema_directory: None,
            backup_on_migration: true,
            max_state_size_bytes: Some(10_000_000), // 10MB per key
            backup_enabled: false,
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

impl From<SecurityConfig> for crate::engine::SecurityContext {
    fn from(config: SecurityConfig) -> Self {
        Self {
            allow_file_access: config.allow_file_access,
            allow_network_access: config.allow_network_access,
            allow_process_spawn: config.allow_process_spawn,
            max_memory_bytes: config.max_memory_bytes,
            max_execution_time_ms: config.max_execution_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.default_engine, "lua");
        assert!(config.supports_engine("lua"));
        assert!(config.supports_engine("javascript"));
        assert!(!config.supports_engine("python"));
    }
    #[test]
    fn test_security_config_conversion() {
        let config = SecurityConfig::default();
        let context: crate::engine::SecurityContext = config.into();
        assert!(!context.allow_file_access);
        assert!(context.allow_network_access);
        assert!(!context.allow_process_spawn);
    }
}
