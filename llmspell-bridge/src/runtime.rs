//! ABOUTME: Language-agnostic script runtime using `ScriptEngineBridge` abstraction
//! ABOUTME: Central execution orchestrator supporting multiple script engines

use crate::{
    engine::{ScriptEngineBridge, ScriptOutput, ScriptStream},
    providers::ProviderManager,
    registry::ComponentRegistry,
};

#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::engine::EngineFactory;
#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::tools::register_all_tools;

#[cfg(feature = "lua")]
use crate::engine::LuaConfig;

#[cfg(feature = "javascript")]
use crate::engine::JSConfig;
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use llmspell_core::traits::component_lookup::ComponentLookup;
use llmspell_core::traits::debug_context::DebugContext;
use llmspell_core::traits::script_executor::{
    ScriptExecutionMetadata, ScriptExecutionOutput, ScriptExecutor,
};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::{debug, info, instrument};

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
/// # }
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
    /// Debug context for debugging support (uses interior mutability)
    debug_context: Arc<RwLock<Option<Arc<dyn DebugContext>>>>,
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
    #[cfg(feature = "lua")]
    #[instrument(level = "info", skip(config), fields(
        engine_type = "lua",
        default_engine = %config.default_engine,
        events_enabled = config.events.enabled
    ))]
    pub async fn new_with_lua(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
        info!("Creating Lua script runtime");
        // Convert llmspell-config LuaConfig to bridge LuaConfig
        let lua_config = LuaConfig::default(); // For now, use defaults - TODO: proper conversion
        let engine = EngineFactory::create_lua_engine_with_runtime(
            &lua_config,
            Some(Arc::new(config.clone())),
        )?;
        Self::new_with_engine(engine, config).await
    }

    /// Create a new runtime with JavaScript engine
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    #[cfg(feature = "javascript")]
    #[instrument(level = "info", skip(config), fields(
        engine_type = "javascript",
        default_engine = %config.default_engine,
        events_enabled = config.events.enabled
    ))]
    pub async fn new_with_javascript(config: LLMSpellConfig) -> Result<Self, LLMSpellError> {
        info!("Creating JavaScript script runtime");
        // Convert llmspell-config JSConfig to bridge JSConfig
        let js_config = JSConfig::default(); // For now, use defaults - TODO: proper conversion
        let engine = EngineFactory::create_javascript_engine(&js_config)?;
        Self::new_with_engine(engine, config).await
    }

    /// Create Lua runtime with existing provider manager (Phase 11.FIX.1)
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    #[cfg(feature = "lua")]
    pub fn new_with_lua_and_provider(
        config: LLMSpellConfig,
        provider_manager: Arc<ProviderManager>,
    ) -> Result<Self, LLMSpellError> {
        info!("Creating Lua script runtime with existing provider manager");
        let lua_config = LuaConfig::default();
        let engine = EngineFactory::create_lua_engine_with_runtime(
            &lua_config,
            Some(Arc::new(config.clone())),
        )?;
        Self::new_with_engine_and_provider(engine, config, provider_manager)
    }

    /// Create JavaScript runtime with existing provider manager (Phase 11.FIX.1)
    ///
    /// # Errors
    ///
    /// Returns an error if runtime initialization fails
    #[cfg(feature = "javascript")]
    pub fn new_with_javascript_and_provider(
        config: LLMSpellConfig,
        provider_manager: Arc<ProviderManager>,
    ) -> Result<Self, LLMSpellError> {
        info!("Creating JavaScript script runtime with existing provider manager");
        let js_config = JSConfig::default();
        let engine = EngineFactory::create_javascript_engine(&js_config)?;
        Self::new_with_engine_and_provider(engine, config, provider_manager)
    }

    /// Create a new runtime with a specific engine by name
    ///
    /// # Errors
    ///
    /// Returns an error if the engine is not found or runtime initialization fails
    #[instrument(level = "info", skip(config), fields(
        engine_name = %engine_name,
        default_engine = %config.default_engine,
        events_enabled = config.events.enabled
    ))]
    pub async fn new_with_engine_name(
        engine_name: &str,
        config: LLMSpellConfig,
    ) -> Result<Self, LLMSpellError> {
        info!("Creating script runtime with engine: {}", engine_name);
        match engine_name {
            #[cfg(feature = "lua")]
            "lua" => Self::new_with_lua(config).await,
            #[cfg(feature = "javascript")]
            "javascript" | "js" => Self::new_with_javascript(config).await,
            _ => Err(LLMSpellError::Validation {
                field: Some("engine".to_string()),
                message: format!(
                    "Unsupported or disabled engine: '{}'. Available: {}",
                    engine_name,
                    Self::available_engines().join(", ")
                ),
            }),
        }
    }

    /// Get list of compiled script engines
    ///
    /// Returns a list of engine names that were compiled into this binary
    /// based on enabled features. Useful for error messages and diagnostics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_bridge::ScriptRuntime;
    ///
    /// let engines = ScriptRuntime::available_engines();
    /// println!("Available engines: {}", engines.join(", "));
    /// ```
    #[must_use]
    #[allow(clippy::vec_init_then_push)] // Cannot use vec![] with #[cfg] attributes
    #[allow(clippy::missing_const_for_fn)] // Vec::push is not const
    pub fn available_engines() -> Vec<&'static str> {
        #[allow(unused_mut)] // mut needed when at least one feature enabled
        let mut engines = Vec::new();
        #[cfg(feature = "lua")]
        engines.push("lua");
        #[cfg(feature = "javascript")]
        engines.push("javascript");
        engines
    }

    /// Core initialization with any engine
    #[cfg(any(feature = "lua", feature = "javascript"))]
    #[instrument(level = "debug", skip(engine, config), fields(
        engine_name = engine.get_engine_name(),
        events_enabled = config.events.enabled,
        tools_enabled = config.tools.enabled,
        providers_count = config.providers.providers.len()
    ))]
    async fn new_with_engine(
        mut engine: Box<dyn ScriptEngineBridge>,
        config: LLMSpellConfig,
    ) -> Result<Self, LLMSpellError> {
        debug!("Initializing script runtime with engine");
        // Create component registry with event support and templates based on config
        let registry = if config.events.enabled {
            // Create EventBus with default configuration
            // Note: Buffer size is hardcoded to 10000 in EventBus implementation
            let event_bus = Arc::new(llmspell_events::EventBus::new());

            // Convert config to EventConfig for llmspell-core
            let event_config = llmspell_core::traits::event::EventConfig {
                enabled: config.events.enabled,
                include_types: config.events.filtering.include_types.clone(),
                exclude_types: config.events.filtering.exclude_types.clone(),
                emit_timing_events: config.events.emit_timing_events,
                emit_state_events: config.events.emit_state_events,
                emit_debug_events: config.events.emit_debug_events,
                max_events_per_second: config.events.max_events_per_second,
            };

            Arc::new(
                ComponentRegistry::with_event_bus_and_templates(event_bus, event_config).map_err(
                    |e| LLMSpellError::Component {
                        message: format!("Failed to initialize component registry: {e}"),
                        source: None,
                    },
                )?,
            )
        } else {
            // Events disabled, create registry with built-in templates
            Arc::new(
                ComponentRegistry::with_templates().map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to initialize component registry: {e}"),
                    source: None,
                })?,
            )
        };

        // Register all Phase 2 tools with the registry using configuration
        register_all_tools(&registry, &config.tools).map_err(|e| LLMSpellError::Component {
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
            debug_context: Arc::new(RwLock::new(None)),
            _config: config,
        })
    }

    /// Create runtime with existing provider manager (Phase 11.FIX.1)
    /// This ensures a single `ProviderManager` instance is shared between kernel and script runtime
    #[cfg(any(feature = "lua", feature = "javascript"))]
    fn new_with_engine_and_provider(
        mut engine: Box<dyn ScriptEngineBridge>,
        config: LLMSpellConfig,
        provider_manager: Arc<ProviderManager>,
    ) -> Result<Self, LLMSpellError> {
        debug!("Initializing script runtime with engine and existing provider manager");
        // Create component registry with event support and templates based on config
        let registry = if config.events.enabled {
            let event_bus = Arc::new(llmspell_events::EventBus::new());
            let event_config = llmspell_core::traits::event::EventConfig {
                enabled: config.events.enabled,
                include_types: config.events.filtering.include_types.clone(),
                exclude_types: config.events.filtering.exclude_types.clone(),
                emit_timing_events: config.events.emit_timing_events,
                emit_state_events: config.events.emit_state_events,
                emit_debug_events: config.events.emit_debug_events,
                max_events_per_second: config.events.max_events_per_second,
            };
            Arc::new(
                ComponentRegistry::with_event_bus_and_templates(event_bus, event_config).map_err(
                    |e| LLMSpellError::Component {
                        message: format!("Failed to initialize component registry: {e}"),
                        source: None,
                    },
                )?,
            )
        } else {
            Arc::new(
                ComponentRegistry::with_templates().map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to initialize component registry: {e}"),
                    source: None,
                })?,
            )
        };

        register_all_tools(&registry, &config.tools).map_err(|e| LLMSpellError::Component {
            message: format!("Failed to register tools: {e}"),
            source: None,
        })?;

        // Use provided provider manager instead of creating new one
        engine.inject_apis(&registry, &provider_manager)?;

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
            debug_context: Arc::new(RwLock::new(None)),
            _config: config,
        })
    }

    /// Execute a script and return the output
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails
    #[instrument(level = "info", skip(self, script), fields(
        engine_name = self.engine.get_engine_name(),
        script_size = script.len(),
        execution_id = %uuid::Uuid::new_v4()
    ))]
    pub async fn execute_script(&self, script: &str) -> Result<ScriptOutput, LLMSpellError> {
        info!("Executing script with {} bytes", script.len());
        self.engine.execute_script(script).await
    }

    /// Get completion candidates for the given context
    ///
    /// This method is used for REPL tab completion to suggest available
    /// variables, functions, and other completable elements.
    ///
    /// # Arguments
    ///
    /// * `context` - The completion context containing the line and cursor position
    ///
    /// # Returns
    ///
    /// A vector of completion candidates suitable for the current context
    #[must_use]
    pub fn get_completion_candidates(
        &self,
        context: &crate::engine::bridge::CompletionContext,
    ) -> Vec<crate::engine::bridge::CompletionCandidate> {
        self.engine.get_completion_candidates(context)
    }

    /// Execute a script with streaming output
    ///
    /// # Errors
    ///
    /// Returns an error if the engine doesn't support streaming or script execution fails
    #[instrument(level = "debug", skip(self, script), fields(
        engine_name = self.engine.get_engine_name(),
        script_size = script.len(),
        execution_id = %uuid::Uuid::new_v4(),
        streaming_supported = self.engine.supports_streaming()
    ))]
    pub async fn execute_script_streaming(
        &self,
        script: &str,
    ) -> Result<ScriptStream, LLMSpellError> {
        debug!("Executing script with streaming output");
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
    #[instrument(level = "debug", skip(self, args), fields(
        engine_name = self.engine.get_engine_name(),
        argument_count = args.len()
    ))]
    pub async fn set_script_args(
        &mut self,
        args: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        debug!("Setting {} script arguments", args.len());
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

/// Implementation of `ScriptExecutor` trait for `ScriptRuntime`
///
/// This allows the kernel to execute scripts without directly depending on
/// the bridge crate, avoiding cyclic dependencies.
#[async_trait]
impl ScriptExecutor for ScriptRuntime {
    #[instrument(skip(self, script))]
    async fn execute_script(&self, script: &str) -> Result<ScriptExecutionOutput, LLMSpellError> {
        let start = Instant::now();

        // Execute using the underlying engine
        let engine_output = self.engine.execute_script(script).await?;

        // Convert ScriptOutput to ScriptExecutionOutput
        let output = ScriptExecutionOutput {
            output: engine_output.output,
            console_output: engine_output.console_output,
            metadata: ScriptExecutionMetadata {
                duration: start.elapsed(),
                language: engine_output.metadata.engine.clone(),
                exit_code: None, // ScriptMetadata doesn't have exit_code
                warnings: engine_output.metadata.warnings,
            },
        };

        Ok(output)
    }

    async fn execute_script_with_args(
        &self,
        script: &str,
        args: std::collections::HashMap<String, String>,
    ) -> Result<ScriptExecutionOutput, LLMSpellError> {
        let start = Instant::now();

        debug!("Executing script with {} arguments", args.len());

        // We need to temporarily set the args and then execute
        // Since we can't mutate self, we need to use a different approach
        // Create a new script with args injected as a preamble
        let script_with_args = if args.is_empty() {
            script.to_string()
        } else {
            let mut preamble = String::from("-- Injected script arguments\nARGS = {}\n");
            for (key, value) in &args {
                // Escape the value for Lua string
                let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
                writeln!(preamble, "ARGS[\"{key}\"] = \"{escaped_value}\"")
                    .expect("String write should never fail");
            }
            preamble.push_str("\n-- Original script\n");
            preamble.push_str(script);
            preamble
        };

        // Execute using the underlying engine
        let engine_output = self.engine.execute_script(&script_with_args).await?;

        // Convert ScriptOutput to ScriptExecutionOutput
        let output = ScriptExecutionOutput {
            output: engine_output.output,
            console_output: engine_output.console_output,
            metadata: ScriptExecutionMetadata {
                duration: start.elapsed(),
                language: engine_output.metadata.engine.clone(),
                exit_code: None,
                warnings: engine_output.metadata.warnings,
            },
        };

        Ok(output)
    }

    fn supports_streaming(&self) -> bool {
        self.engine.supports_streaming()
    }

    fn language(&self) -> &'static str {
        // Return the configured engine type
        // TODO: Add a method to get current engine language
        "lua" // Default for now since we use Lua primarily
    }

    async fn is_ready(&self) -> bool {
        // Engine is ready if it's been initialized
        // TODO: Add proper readiness check to ScriptEngineBridge trait
        true
    }

    fn set_debug_context(&self, context: Option<Arc<dyn DebugContext>>) {
        // Use interior mutability to set debug context
        self.debug_context.write().unwrap().clone_from(&context);

        // Also set it on the underlying engine if it supports debugging
        self.engine.set_debug_context(context);
    }

    fn supports_debugging(&self) -> bool {
        // Check if the underlying engine supports debugging
        self.engine.supports_debugging()
    }

    fn get_debug_context(&self) -> Option<Arc<dyn DebugContext>> {
        // Return the stored debug context
        let debug_context = self.debug_context.read().unwrap();
        debug_context.clone()
    }

    fn component_registry(&self) -> Option<Arc<dyn ComponentLookup>> {
        // Return the component registry as ComponentLookup trait
        Some(Arc::clone(&self.registry) as Arc<dyn ComponentLookup>)
    }

    fn template_registry_any(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        // Return the template registry as type-erased Any to avoid circular dependencies
        self.registry
            .template_registry()
            .map(|reg| reg as Arc<dyn std::any::Any + Send + Sync>)
    }

    fn get_completion_candidates(&self, line: &str, cursor_pos: usize) -> Vec<(String, String)> {
        // Create a CompletionContext from the provided line and cursor position
        let context = crate::engine::bridge::CompletionContext::new(line, cursor_pos);

        // Get completions from the underlying engine
        let candidates = self.engine.get_completion_candidates(&context);

        // Convert CompletionCandidate to tuple format expected by ScriptExecutor trait
        candidates
            .into_iter()
            .map(|candidate| {
                let display = format!("{:?}", candidate.kind).to_lowercase();
                (candidate.text, display)
            })
            .collect()
    }

    // === Template Operations (JSON-based API to avoid circular dependencies) ===

    fn handle_template_list(
        &self,
        category: Option<&str>,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Get templates by category or all
        let templates_metadata = if let Some(cat_str) = category {
            // Parse category string
            use llmspell_templates::core::TemplateCategory;
            let category = match cat_str.to_lowercase().as_str() {
                "research" => TemplateCategory::Research,
                "chat" => TemplateCategory::Chat,
                "analysis" => TemplateCategory::Analysis,
                "codegen" => TemplateCategory::CodeGen,
                "document" => TemplateCategory::Document,
                "workflow" => TemplateCategory::Workflow,
                _ => {
                    return Err(LLMSpellError::Validation {
                        field: Some("category".to_string()),
                        message: format!("Invalid category: {cat_str}"),
                    });
                }
            };
            template_registry.discover_by_category(&category)
        } else {
            template_registry.list_metadata()
        };

        // Convert to JSON
        let templates_json: Vec<serde_json::Value> = templates_metadata
            .iter()
            .map(|meta| {
                json!({
                    "id": meta.id,
                    "name": meta.name,
                    "description": meta.description,
                    "category": format!("{:?}", meta.category),
                    "version": meta.version,
                    "author": meta.author,
                    "tags": meta.tags,
                })
            })
            .collect();

        Ok(json!(templates_json))
    }

    fn handle_template_info(
        &self,
        template_id: &str,
        with_schema: bool,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Get template
        let template =
            template_registry
                .get(template_id)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Template not found: {e}"),
                    source: None,
                })?;

        let metadata = template.metadata();
        let mut info_json = json!({
            "id": metadata.id,
            "name": metadata.name,
            "description": metadata.description,
            "category": format!("{:?}", metadata.category),
            "version": metadata.version,
            "author": metadata.author,
            "requires": metadata.requires,
            "tags": metadata.tags,
        });

        // Add schema if requested
        if with_schema {
            let schema = template.config_schema();
            if let Ok(schema_json) = serde_json::to_value(schema) {
                info_json["schema"] = schema_json;
            }
        }

        Ok(info_json)
    }

    async fn handle_template_exec(
        &self,
        template_id: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get template from registry
        let template = self.get_template_from_registry(template_id)?;

        // Convert and validate params
        let template_params = Self::convert_and_validate_params(&template, &params)?;

        // Build execution context
        let context = llmspell_templates::context::ExecutionContext::builder()
            .build()
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to build execution context: {e}"),
                source: None,
            })?;

        // Execute template
        let output = template
            .execute(template_params, context)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template execution failed: {e}"),
                source: None,
            })?;

        // Convert output to JSON response
        Ok(json!({
            "result": Self::convert_template_result_to_json(&output.result),
            "artifacts": Self::convert_artifacts_to_json(&output.artifacts),
            "metrics": {
                "duration_ms": output.metrics.duration_ms,
                "tokens_used": output.metrics.tokens_used,
                "cost_usd": output.metrics.cost_usd,
                "agents_invoked": output.metrics.agents_invoked,
                "tools_invoked": output.metrics.tools_invoked,
                "rag_queries": output.metrics.rag_queries,
            }
        }))
    }

    fn handle_template_search(
        &self,
        query: &str,
        category: Option<&str>,
    ) -> Result<serde_json::Value, LLMSpellError> {
        use serde_json::json;

        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Search templates
        let mut results = template_registry.search(query);

        // Filter by category if specified
        if let Some(cat_str) = category {
            use llmspell_templates::core::TemplateCategory;
            let category = match cat_str.to_lowercase().as_str() {
                "research" => TemplateCategory::Research,
                "chat" => TemplateCategory::Chat,
                "analysis" => TemplateCategory::Analysis,
                "codegen" => TemplateCategory::CodeGen,
                "document" => TemplateCategory::Document,
                "workflow" => TemplateCategory::Workflow,
                _ => {
                    return Err(LLMSpellError::Validation {
                        field: Some("category".to_string()),
                        message: format!("Invalid category: {cat_str}"),
                    });
                }
            };
            results.retain(|m| m.category == category);
        }

        // Convert to JSON
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|meta| {
                json!({
                    "id": meta.id,
                    "name": meta.name,
                    "description": meta.description,
                    "category": format!("{:?}", meta.category),
                    "tags": meta.tags,
                })
            })
            .collect();

        Ok(json!(results_json))
    }

    fn handle_template_schema(
        &self,
        template_id: &str,
    ) -> Result<serde_json::Value, LLMSpellError> {
        // Get TemplateRegistry via type erasure
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        // Downcast to concrete TemplateRegistry
        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        // Get template
        let template =
            template_registry
                .get(template_id)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Template not found: {e}"),
                    source: None,
                })?;

        let schema = template.config_schema();
        serde_json::to_value(schema).map_err(|e| LLMSpellError::Component {
            message: format!("Failed to serialize schema: {e}"),
            source: None,
        })
    }
}

/// Helper methods for template execution
impl ScriptRuntime {
    /// Helper to get template from registry with type erasure
    fn get_template_from_registry(
        &self,
        template_id: &str,
    ) -> Result<std::sync::Arc<dyn llmspell_templates::core::Template>, LLMSpellError> {
        let registry_any =
            self.template_registry_any()
                .ok_or_else(|| LLMSpellError::Component {
                    message: "Template registry not available".to_string(),
                    source: None,
                })?;

        let template_registry = std::sync::Arc::downcast::<
            llmspell_templates::registry::TemplateRegistry,
        >(registry_any)
        .map_err(|_| LLMSpellError::Component {
            message: "Failed to access template registry".to_string(),
            source: None,
        })?;

        template_registry
            .get(template_id)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Template not found: {e}"),
                source: None,
            })
    }

    /// Helper to convert and validate JSON params to `TemplateParams`
    fn convert_and_validate_params(
        template: &std::sync::Arc<dyn llmspell_templates::core::Template>,
        params: &serde_json::Value,
    ) -> Result<llmspell_templates::core::TemplateParams, LLMSpellError> {
        let params_obj = params
            .as_object()
            .ok_or_else(|| LLMSpellError::Validation {
                field: Some("params".to_string()),
                message: "Parameters must be a JSON object".to_string(),
            })?;

        let mut template_params = llmspell_templates::core::TemplateParams::new();
        for (key, value) in params_obj {
            template_params.insert(key.clone(), value.clone());
        }

        template
            .validate(&template_params)
            .map_err(|e| LLMSpellError::Validation {
                field: Some("params".to_string()),
                message: format!("Parameter validation failed: {e}"),
            })?;

        Ok(template_params)
    }

    /// Helper to convert `TemplateResult` to JSON
    fn convert_template_result_to_json(
        result: &llmspell_templates::core::TemplateResult,
    ) -> serde_json::Value {
        use serde_json::json;

        match result {
            llmspell_templates::core::TemplateResult::Text(text) => {
                json!({"type": "text", "value": text})
            }
            llmspell_templates::core::TemplateResult::Structured(value) => {
                json!({"type": "structured", "value": value})
            }
            llmspell_templates::core::TemplateResult::File(path) => {
                json!({"type": "file", "path": path.display().to_string()})
            }
            llmspell_templates::core::TemplateResult::Multiple(results) => {
                let results_json: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| match r {
                        llmspell_templates::core::TemplateResult::Text(t) => {
                            json!({"type": "text", "value": t})
                        }
                        llmspell_templates::core::TemplateResult::File(p) => {
                            json!({"type": "file", "path": p.display().to_string()})
                        }
                        llmspell_templates::core::TemplateResult::Structured(v) => {
                            json!({"type": "structured", "value": v})
                        }
                        llmspell_templates::core::TemplateResult::Multiple(_) => {
                            json!({"type": "nested_multiple"})
                        }
                    })
                    .collect();
                json!({"type": "multiple", "results": results_json})
            }
        }
    }

    /// Helper to convert artifacts to JSON
    fn convert_artifacts_to_json(
        artifacts: &[llmspell_templates::artifacts::Artifact],
    ) -> Vec<serde_json::Value> {
        use serde_json::json;

        artifacts
            .iter()
            .map(|a| {
                json!({
                    "filename": a.filename,
                    "mime_type": a.mime_type,
                    "size": a.size()
                })
            })
            .collect()
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
