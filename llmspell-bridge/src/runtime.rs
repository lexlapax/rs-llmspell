//! ABOUTME: Language-agnostic script runtime using `ScriptEngineBridge` abstraction
//! ABOUTME: Central execution orchestrator supporting multiple script engines

use crate::{
    debug_coordinator::DebugCoordinator,
    diagnostics_bridge::DiagnosticsBridge,
    engine::{EngineFactory, JSConfig, LuaConfig, ScriptEngineBridge, ScriptOutput, ScriptStream},
    execution_bridge::{
        Breakpoint, DebugCommand, DebugState, ExecutionManager, StackFrame, Variable,
    },
    execution_context::SharedExecutionContext,
    lua::debug_state_cache_impl::LuaDebugStateCache,
    providers::ProviderManager,
    registry::ComponentRegistry,
    tools::register_all_tools,
};
use llmspell_config::LLMSpellConfig;
use llmspell_core::error::LLMSpellError;
use llmspell_state_persistence::manager::StateManager;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;

/// Output event from script execution
#[derive(Debug, Clone)]
pub enum OutputEvent {
    /// Standard output text
    Stdout(String),
    /// Standard error text
    Stderr(String),
    /// Execution result value
    Result(Value),
    /// Execution error
    Error { name: String, message: String },
}

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
    /// Runtime configuration
    _config: LLMSpellConfig,
    /// Execution manager for debugging support
    execution_manager: Option<Arc<ExecutionManager>>,
    /// Debug coordinator for language-agnostic debug operations
    debug_coordinator: Option<Arc<DebugCoordinator>>,
    /// Diagnostics bridge for debug output
    _diagnostics_bridge: Option<Arc<DiagnosticsBridge>>,
    /// Shared execution context for debugging
    _shared_execution_context: Option<Arc<TokioRwLock<SharedExecutionContext>>>,
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
            _ => Err(LLMSpellError::Validation {
                field: Some("engine".to_string()),
                message: format!("Unsupported engine: {engine_name}. Available: lua, javascript"),
            }),
        }
    }

    /// Create a new runtime with a specific engine and external `StateManager`
    ///
    /// This constructor allows sharing a `StateManager` instance between the kernel
    /// and the runtime, ensuring they use the same state backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the engine is not found or runtime initialization fails
    pub async fn new_with_engine_and_state_manager(
        engine_name: &str,
        config: LLMSpellConfig,
        state_manager: Arc<StateManager>,
    ) -> Result<Self, LLMSpellError> {
        match engine_name {
            "lua" => {
                let lua_config = LuaConfig::default();
                let engine = EngineFactory::create_lua_engine_with_state_manager(
                    &lua_config,
                    Some(Arc::new(config.clone())),
                    state_manager,
                )?;
                Self::new_with_engine(engine, config).await
            }
            "javascript" | "js" => {
                // JavaScript engine doesn't support external StateManager yet
                // Fall back to creating its own
                Self::new_with_javascript(config).await
            }
            _ => Err(LLMSpellError::Validation {
                field: Some("engine".to_string()),
                message: format!("Unsupported engine: {engine_name}. Available: lua, javascript"),
            }),
        }
    }

    /// Initialize debug infrastructure based on configuration
    async fn init_debug_infrastructure(
        config: &LLMSpellConfig,
        engine: &mut Box<dyn ScriptEngineBridge>,
    ) -> (
        Option<Arc<ExecutionManager>>,
        Option<Arc<DebugCoordinator>>,
        Option<Arc<DiagnosticsBridge>>,
        Option<Arc<TokioRwLock<SharedExecutionContext>>>,
    ) {
        Self::log_debug_init_start(config);

        if !config.debug.enabled {
            tracing::info!("Debug not enabled, skipping debug infrastructure initialization");
            return (None, None, None, None);
        }

        let components = Self::create_and_setup_debug_components(config, engine).await;
        Self::log_debug_init_complete(&config.debug.mode);

        components
    }

    fn log_debug_init_start(config: &LLMSpellConfig) {
        tracing::info!(
            "init_debug_infrastructure called with debug.enabled = {}, debug.mode = {}",
            config.debug.enabled,
            config.debug.mode
        );
    }

    fn log_debug_init_complete(mode: &str) {
        tracing::info!(
            "Debug mode enabled ({}) - initialized DiagnosticsBridge, ExecutionManager, and DebugCoordinator",
            mode
        );
    }

    async fn create_and_setup_debug_components(
        config: &LLMSpellConfig,
        engine: &mut Box<dyn ScriptEngineBridge>,
    ) -> (
        Option<Arc<ExecutionManager>>,
        Option<Arc<DebugCoordinator>>,
        Option<Arc<DiagnosticsBridge>>,
        Option<Arc<TokioRwLock<SharedExecutionContext>>>,
    ) {
        tracing::info!("Debug enabled, initializing debug infrastructure");

        // Create core debug components
        let (diagnostics, shared_context, exec_manager) = Self::create_debug_components();

        // Create and configure debug coordinator
        let coordinator = Self::create_debug_coordinator(&exec_manager, &shared_context).await;

        // Install debug hooks based on mode
        Self::setup_and_install_debug_hooks(
            config,
            engine,
            &exec_manager,
            &coordinator,
            &shared_context,
            &diagnostics,
        );

        (
            Some(exec_manager),
            Some(coordinator),
            Some(diagnostics),
            Some(shared_context),
        )
    }

    fn create_debug_components() -> (
        Arc<DiagnosticsBridge>,
        Arc<TokioRwLock<SharedExecutionContext>>,
        Arc<ExecutionManager>,
    ) {
        let diagnostics = Arc::new(DiagnosticsBridge::builder().build());
        let shared_context = Arc::new(TokioRwLock::new(SharedExecutionContext::new()));
        let debug_cache = Arc::new(LuaDebugStateCache::new());
        let exec_manager = Arc::new(ExecutionManager::new(debug_cache));

        (diagnostics, shared_context, exec_manager)
    }

    async fn create_debug_coordinator(
        exec_manager: &Arc<ExecutionManager>,
        shared_context: &Arc<TokioRwLock<SharedExecutionContext>>,
    ) -> Arc<DebugCoordinator> {
        // Create capabilities for DebugCoordinator
        let capabilities: Arc<
            TokioRwLock<HashMap<String, Arc<dyn llmspell_core::debug::DebugCapability>>>,
        > = Arc::new(TokioRwLock::new(HashMap::new()));

        // Add ExecutionManagerAdapter as a capability
        let adapter: Arc<dyn llmspell_core::debug::DebugCapability> =
            Arc::new(crate::debug_adapters::ExecutionManagerAdapter::new(
                exec_manager.clone(),
                "debug_session".to_string(),
            ));
        capabilities
            .write()
            .await
            .insert("execution_manager".to_string(), adapter);

        // Create DebugCoordinator with ExecutionManager
        Arc::new(DebugCoordinator::new(
            shared_context.clone(),
            capabilities,
            exec_manager.clone(),
        ))
    }

    fn setup_and_install_debug_hooks(
        config: &LLMSpellConfig,
        engine: &mut Box<dyn ScriptEngineBridge>,
        exec_manager: &Arc<ExecutionManager>,
        coordinator: &Arc<DebugCoordinator>,
        shared_context: &Arc<TokioRwLock<SharedExecutionContext>>,
        diagnostics: &Arc<DiagnosticsBridge>,
    ) {
        // Select debug hook based on mode
        let debug_hook = Self::create_debug_hook(
            &config.debug.mode,
            exec_manager,
            coordinator,
            shared_context,
            diagnostics,
        );

        tracing::info!(
            "Runtime: Installing debug hooks for mode: {}",
            config.debug.mode
        );
        if let Err(e) = engine.install_debug_hooks(debug_hook) {
            tracing::error!("Runtime: Failed to install debug hooks: {}", e);
        } else {
            tracing::info!("Runtime: Successfully installed debug hooks");
        }
    }

    fn create_debug_hook(
        mode: &str,
        exec_manager: &Arc<ExecutionManager>,
        coordinator: &Arc<DebugCoordinator>,
        shared_context: &Arc<TokioRwLock<SharedExecutionContext>>,
        diagnostics: &Arc<DiagnosticsBridge>,
    ) -> Arc<dyn crate::debug_runtime::DebugHook> {
        match mode {
            "interactive" => {
                // Use LuaDebugHookAdapter for interactive debugging (breakpoints, stepping)
                // This bridges Layer 1 (DebugHook) to Layer 3 (HookHandler)
                Arc::new(crate::lua::debug_hook_adapter::LuaDebugHookAdapter::new(
                    exec_manager.clone(),
                    coordinator.clone(),
                    shared_context.clone(),
                ))
            }
            _ => {
                // Default to simple tracing for "tracing" mode or any other value
                Arc::new(SimpleTracingHook::new(
                    true, // Always trace when debug is enabled
                    diagnostics.clone(),
                ))
            }
        }
    }

    /// Core initialization with any engine
    async fn new_with_engine(
        mut engine: Box<dyn ScriptEngineBridge>,
        config: LLMSpellConfig,
    ) -> Result<Self, LLMSpellError> {
        // Create component registry with event support based on config
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

            Arc::new(ComponentRegistry::with_event_bus(event_bus, event_config))
        } else {
            // Events disabled, create registry without event bus
            Arc::new(ComponentRegistry::new())
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

        // Initialize debug infrastructure if enabled
        let (execution_manager, debug_coordinator, diagnostics_bridge, shared_execution_context) =
            Self::init_debug_infrastructure(&config, &mut engine).await;

        Ok(Self {
            engine,
            registry,
            provider_manager,
            execution_context,
            _config: config,
            execution_manager,
            debug_coordinator,
            _diagnostics_bridge: diagnostics_bridge,
            _shared_execution_context: shared_execution_context,
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

    /// Execute a script with output capture callback
    ///
    /// This method executes a script and sends output through the provided callback.
    /// This is primarily used by the kernel to route output through the protocol layer.
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails
    pub async fn execute_script_with_callback<F>(
        &self,
        script: &str,
        mut output_callback: F,
    ) -> Result<ScriptOutput, LLMSpellError>
    where
        F: FnMut(OutputEvent),
    {
        // Execute the script normally
        let result = self.engine.execute_script(script).await?;

        // Send console output as stdout events
        for line in &result.console_output {
            output_callback(OutputEvent::Stdout(line.clone()));
        }

        // Send the result value
        output_callback(OutputEvent::Result(result.output.clone()));

        Ok(result)
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

    /// Install debug hooks for execution control
    ///
    /// This method allows installing debug hooks that can control script execution
    /// for debugging purposes (breakpoints, stepping, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if the engine doesn't support debugging or hook installation fails
    pub fn install_debug_hooks(
        &mut self,
        hook: Arc<dyn crate::debug_runtime::DebugHook>,
    ) -> Result<(), LLMSpellError> {
        self.engine.install_debug_hooks(hook)
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

    // Debug interface methods

    /// Set a debugger instance for the runtime
    pub fn set_debugger(&mut self, debugger: &Arc<ExecutionManager>) {
        self.execution_manager = Some(debugger.clone());
        // TODO: Propagate to engine when engine supports it
    }

    /// Get the execution manager for debug operations
    /// Returns None if debug mode is disabled
    #[must_use]
    pub fn get_execution_manager(&self) -> Option<Arc<ExecutionManager>> {
        self.execution_manager.clone()
    }

    /// Get the debug coordinator instance if available
    #[must_use]
    pub fn get_debug_coordinator(&self) -> Option<Arc<DebugCoordinator>> {
        self.debug_coordinator.clone()
    }

    /// Set breakpoints for debugging
    pub async fn set_breakpoints(&mut self, breakpoints: Vec<Breakpoint>) -> Vec<Breakpoint> {
        // Prefer DebugCoordinator if available (it delegates to ExecutionManager)
        if let Some(ref coordinator) = self.debug_coordinator {
            // Clear existing breakpoints first
            let existing = coordinator.get_breakpoints().await;
            for bp in existing {
                let _ = coordinator.remove_breakpoint(&bp.id).await;
            }

            // Add new breakpoints through coordinator
            let mut verified_breakpoints = Vec::new();
            for bp in breakpoints {
                match coordinator.add_breakpoint(bp.clone()).await {
                    Ok(id) => {
                        let mut verified = bp;
                        verified.id = id;
                        verified_breakpoints.push(verified);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to add breakpoint: {}", e);
                    }
                }
            }
            verified_breakpoints
        } else if let Some(ref execution_manager) = self.execution_manager {
            // Fallback to ExecutionManager directly if no coordinator
            // Clear existing breakpoints
            execution_manager.clear().await;

            // Add new breakpoints
            let mut verified_breakpoints = Vec::new();
            for bp in breakpoints {
                let id = execution_manager.add_breakpoint(bp.clone()).await;
                let mut verified = bp;
                verified.id = id;
                verified_breakpoints.push(verified);
            }

            // TODO: Propagate to engine when engine supports it
            verified_breakpoints
        } else {
            Vec::new()
        }
    }

    /// Get current debug state
    pub async fn get_debug_state(&self) -> DebugState {
        // Prefer DebugCoordinator if available
        if let Some(ref coordinator) = self.debug_coordinator {
            coordinator.get_debug_state().await
        } else if let Some(ref execution_manager) = self.execution_manager {
            execution_manager.get_state().await
        } else {
            DebugState::Terminated
        }
    }

    /// Get stack trace during debugging
    pub async fn get_stack_trace(&self) -> Vec<StackFrame> {
        // Prefer DebugCoordinator if available
        if let Some(ref coordinator) = self.debug_coordinator {
            coordinator.get_call_stack().await
        } else if let Some(ref execution_manager) = self.execution_manager {
            execution_manager.get_stack_trace().await
        } else {
            Vec::new()
        }
    }

    /// Get variables in current or specified frame
    pub async fn get_variables(&self, frame_id: Option<&str>) -> Vec<Variable> {
        // Prefer DebugCoordinator if available for current frame
        if frame_id.is_none() {
            if let Some(ref coordinator) = self.debug_coordinator {
                // Get locals from coordinator and convert to Vec<Variable>
                let locals = coordinator.inspect_locals().await;
                return locals
                    .into_iter()
                    .map(|(name, value)| Variable {
                        name,
                        value: value.to_string(),
                        var_type: "unknown".to_string(),
                        has_children: false,
                        reference: None,
                    })
                    .collect();
            }
        }

        // Otherwise use ExecutionManager for specific frame or fallback
        if let Some(ref execution_manager) = self.execution_manager {
            if let Some(frame_id) = frame_id {
                execution_manager
                    .get_cached_variables(frame_id)
                    .await
                    .unwrap_or_default()
            } else {
                // Get variables from top frame
                let frames = execution_manager.get_stack_trace().await;
                if let Some(top_frame) = frames.first() {
                    execution_manager
                        .get_cached_variables(&top_frame.id)
                        .await
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    /// Send a debug command (continue, step, pause, etc.)
    pub async fn send_debug_command(&self, command: DebugCommand) {
        // Prefer DebugCoordinator if available (it delegates to ExecutionManager)
        if let Some(ref coordinator) = self.debug_coordinator {
            match command {
                DebugCommand::Continue => {
                    coordinator.resume().await;
                }
                DebugCommand::StepInto => {
                    coordinator.step_into().await;
                }
                DebugCommand::StepOver => {
                    coordinator.step_over().await;
                }
                DebugCommand::StepOut => {
                    coordinator.step_out().await;
                }
                DebugCommand::Terminate => {
                    // Coordinator doesn't have terminate, use ExecutionManager directly
                    if let Some(ref execution_manager) = self.execution_manager {
                        execution_manager.set_state(DebugState::Terminated).await;
                    }
                }
                DebugCommand::Pause => {
                    // Pause will be handled by the engine
                }
            }
        } else if let Some(ref execution_manager) = self.execution_manager {
            // Fallback to ExecutionManager directly if no coordinator
            match command {
                DebugCommand::Continue => {
                    execution_manager.set_state(DebugState::Running).await;
                }
                DebugCommand::Terminate => {
                    execution_manager.set_state(DebugState::Terminated).await;
                }
                _ => {
                    // Step commands and pause will be handled by the engine
                }
            }
        }
    }

    /// Check if debugging is supported and enabled
    #[must_use]
    pub fn supports_debugging(&self) -> bool {
        self.engine.supported_features().debugging && self.execution_manager.is_some()
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

/// Simple debug hook that outputs trace information to stdout
struct SimpleTracingHook {
    trace_enabled: bool,
    _diagnostics: Arc<DiagnosticsBridge>,
}

impl SimpleTracingHook {
    #[allow(clippy::missing_const_for_fn)] // Arc cannot be used in const context
    fn new(trace_enabled: bool, diagnostics: Arc<DiagnosticsBridge>) -> Self {
        Self {
            trace_enabled,
            _diagnostics: diagnostics,
        }
    }
}

#[async_trait::async_trait]
impl crate::debug_runtime::DebugHook for SimpleTracingHook {
    async fn on_line(&self, line: u32, source: &str) -> crate::debug_runtime::DebugControl {
        if self.trace_enabled {
            // Output trace information to stdout
            tracing::debug!("Line {}: {}", line, source);
        }
        crate::debug_runtime::DebugControl::Continue
    }

    async fn on_function_enter(
        &self,
        name: &str,
        args: Vec<String>,
    ) -> crate::debug_runtime::DebugControl {
        if self.trace_enabled {
            if args.is_empty() {
                tracing::debug!("Entering function: {}", name);
            } else {
                tracing::debug!("Entering function: {}({})", name, args.join(", "));
            }
        }
        crate::debug_runtime::DebugControl::Continue
    }

    async fn on_function_exit(
        &self,
        name: &str,
        result: Option<String>,
    ) -> crate::debug_runtime::DebugControl {
        if self.trace_enabled {
            if let Some(res) = result {
                tracing::debug!("Exiting function: {} -> {}", name, res);
            } else {
                tracing::debug!("Exiting function: {}", name);
            }
        }
        crate::debug_runtime::DebugControl::Continue
    }

    async fn on_exception(&self, error: &str, line: u32) -> crate::debug_runtime::DebugControl {
        tracing::error!("Exception at line {}: {}", line, error);
        crate::debug_runtime::DebugControl::Continue
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
}
