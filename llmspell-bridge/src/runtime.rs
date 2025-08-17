//! ABOUTME: Language-agnostic script runtime using `ScriptEngineBridge` abstraction
//! ABOUTME: Central execution orchestrator supporting multiple script engines

use crate::{
    engine::{EngineFactory, JSConfig, LuaConfig, ScriptEngineBridge, ScriptOutput, ScriptStream},
    providers::ProviderManager,
    registry::ComponentRegistry,
    tools::register_all_tools,
};
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use std::collections::HashMap;
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
/// use llmspell_bridge::ScriptRuntime;
/// use llmspell_config::LLMSpellConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a runtime with default configuration
/// let config = LLMSpellConfig::default();
/// let runtime = ScriptRuntime::new_with_lua(config).await?;
///
/// // Execute a simple Lua script
/// let output = runtime.execute_script("return 42").await?;
/// println!("Result: {:?}", output.output);
/// # Ok(())
/// # }\
/// ```
///
/// ## Working with Agents (Placeholder - Phase 2)
///
/// ```rust,no_run
/// use llmspell_bridge::ScriptRuntime;
/// use llmspell_config::LLMSpellConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runtime = ScriptRuntime::new_with_lua(LLMSpellConfig::default()).await?;
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
/// use llmspell_bridge::ScriptRuntime;
/// use llmspell_config::LLMSpellConfig;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let runtime = ScriptRuntime::new_with_lua(LLMSpellConfig::default()).await?;
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
    _config: LLMSpellConfig,
}

impl ScriptRuntime {
    /// Create a new runtime with Lua engine
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use llmspell_bridge::ScriptRuntime;
    /// use llmspell_config::LLMSpellConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // With default configuration
    /// let runtime = ScriptRuntime::new_with_lua(LLMSpellConfig::default()).await?;
    ///
    /// // With custom configuration
    /// let mut config = LLMSpellConfig::default();
    /// config.runtime.security.allow_file_access = true;
    /// let runtime = ScriptRuntime::new_with_lua(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    pub async fn new_with_lua(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
        // Convert llmspell-config LuaConfig to bridge LuaConfig
        let lua_config = LuaConfig::default(); // For now, use defaults - TODO: proper conversion
        let engine = EngineFactory::create_lua_engine_with_runtime(&lua_config, None)?;
        Self::new_with_engine(engine, config).await
    }

    /// Create a new runtime with JavaScript engine
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    pub async fn new_with_javascript(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
        // Convert llmspell-config JSConfig to bridge JSConfig
        let js_config = JSConfig::default(); // For now, use defaults - TODO: proper conversion
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
        config: LLMSpellConfig,
    ) -> Result<Self, LLMSpellError> {
        match engine_name {
            "lua" => Self::new_with_lua(config).await,
            "javascript" | "js" => Self::new_with_javascript(config).await,
            _ => Err(LLMSpellError::Configuration {
                message: format!("Unsupported engine: {engine_name}"),
                source: None,
            }),
        }
    }

    /// Core initialization with any engine
    async fn new_with_engine(
        mut engine: Box<dyn ScriptEngineBridge>,
        config: LLMSpellConfig,
    ) -> Result<Self, LLMSpellError> {
        // Create component registry
        let registry = Arc::new(ComponentRegistry::new());

        // Register all Phase 2 tools with the registry
        register_all_tools(&registry).map_err(|e| LLMSpellError::Component {
            message: format!("Failed to register tools: {e}"),
            source: None,
        })?;

        // Create provider manager using config from llmspell-config
        let provider_manager = Arc::new(ProviderManager::new(config.providers.clone()).await?);

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
            security: crate::engine::SecurityContext {
                allow_file_access: config.runtime.security.allow_file_access,
                allow_network_access: config.runtime.security.allow_network_access,
                allow_process_spawn: config.runtime.security.allow_process_spawn,
                max_memory_bytes: config.runtime.security.max_memory_bytes,
                max_execution_time_ms: config.runtime.security.max_execution_time_ms,
            },
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

    /// Set script arguments to be passed to the script
    ///
    /// These arguments will be made available to the script in a language-specific way:
    /// - Lua: Available as global `ARGS` table
    /// - JavaScript: Available as global `args` object
    /// - Python: Available as `sys.argv` equivalent
    ///
    /// # Errors
    ///
    /// Returns an error if the engine fails to set arguments
    pub async fn set_script_args(
        &mut self,
        args: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        self.engine.set_script_args(args).await
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

impl From<llmspell_config::SecurityConfig> for crate::engine::SecurityContext {
    fn from(config: llmspell_config::SecurityConfig) -> Self {
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
        let config = LLMSpellConfig::default();
        assert_eq!(config.default_engine, "lua");
        assert!(config.supports_engine("lua"));
        assert!(config.supports_engine("javascript"));
        assert!(!config.supports_engine("python"));
    }
    #[test]
    fn test_security_config_conversion() {
        let config = llmspell_config::SecurityConfig::default();
        let context: crate::engine::SecurityContext = config.into();
        assert!(!context.allow_file_access);
        assert!(context.allow_network_access);
        assert!(!context.allow_process_spawn);
    }
}
