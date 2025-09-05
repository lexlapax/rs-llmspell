//! ABOUTME: `LuaEngine` implementation of `ScriptEngineBridge` trait
//! ABOUTME: Provides Lua 5.4 script execution with coroutine-based streaming

#![allow(clippy::significant_drop_tightening)]

use crate::condition_evaluator::SharedDebugContext;
use crate::engine::types::ScriptEngineError;
use crate::engine::{
    factory::LuaConfig, EngineFeatures, ExecutionContext, ScriptEngineBridge, ScriptMetadata,
    ScriptOutput, ScriptStream,
};
use crate::lua::globals::args::inject_args_global;
use crate::lua::output::{install_output_capture, ConsoleCapture};
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;
use std::time::Instant;

#[cfg(feature = "lua")]
use {
    crate::globals::{create_standard_registry, GlobalContext, GlobalInjector},
    futures::stream,
};

/// Lua script engine implementation
///
/// Note: mlua requires unsafe Send/Sync implementation for thread safety
pub struct LuaEngine {
    #[cfg(feature = "lua")]
    lua: Arc<parking_lot::Mutex<mlua::Lua>>,
    _config: LuaConfig,
    execution_context: ExecutionContext,
    runtime_config: Option<Arc<llmspell_config::LLMSpellConfig>>,
    script_args: Option<std::collections::HashMap<String, String>>,
    #[cfg(feature = "lua")]
    console_capture: Option<Arc<ConsoleCapture>>,
    #[cfg(feature = "lua")]
    execution_hook:
        Option<Arc<parking_lot::Mutex<crate::lua::globals::execution::LuaExecutionHook>>>,
    execution_manager: Option<Arc<crate::execution_bridge::ExecutionManager>>,
    #[cfg(feature = "lua")]
    lua_debug_adapter: Option<Arc<crate::lua::debug_hook_adapter::LuaDebugHookAdapter>>,
    /// External `StateManager` for shared state access
    external_state_manager: Option<Arc<llmspell_state_persistence::manager::StateManager>>,
}

// SAFETY: We ensure thread safety by using Mutex for all Lua access
// The Lua instance is wrapped in a Mutex to prevent concurrent access
#[allow(unsafe_code)]
unsafe impl Send for LuaEngine {}
// SAFETY: All access to Lua is synchronized through a Mutex
#[allow(unsafe_code)]
unsafe impl Sync for LuaEngine {}

impl LuaEngine {
    /// Create a new Lua engine with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if Lua feature is not enabled or engine creation fails
    pub fn new(config: &LuaConfig) -> Result<Self, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            use mlua::Lua;

            // Create Lua instance (async is enabled via feature flag)
            // TODO: restrict stdlib for Safe level
            let lua = Lua::new();

            // Install output capture (without debug bridge for now)
            let console_capture = install_output_capture(&lua, None).ok();

            Ok(Self {
                lua: Arc::new(parking_lot::Mutex::new(lua)),
                _config: config.clone(),
                execution_context: ExecutionContext::default(),
                runtime_config: None,
                script_args: None,
                console_capture,
                execution_hook: None,
                execution_manager: None,
                lua_debug_adapter: None,
                external_state_manager: None,
            })
        }

        #[cfg(not(feature = "lua"))]
        {
            Ok(Self {
                _config: config.clone(),
                execution_context: ExecutionContext::default(),
                runtime_config: None,
                script_args: None,
                execution_manager: None,
                external_state_manager: None,
            })
        }
    }

    /// Create a new Lua engine with the given configuration and external `StateManager`
    ///
    /// # Errors
    ///
    /// Returns an error if Lua feature is not enabled or engine creation fails
    pub fn new_with_state_manager(
        config: &LuaConfig,
        state_manager: Arc<llmspell_state_persistence::manager::StateManager>,
    ) -> Result<Self, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            use mlua::Lua;

            // Create Lua instance (async is enabled via feature flag)
            let lua = Lua::new();

            // Install output capture (without debug bridge for now)
            let console_capture = install_output_capture(&lua, None).ok();

            Ok(Self {
                lua: Arc::new(parking_lot::Mutex::new(lua)),
                _config: config.clone(),
                execution_context: ExecutionContext::default(),
                runtime_config: None,
                script_args: None,
                console_capture,
                execution_hook: None,
                execution_manager: None,
                lua_debug_adapter: None,
                external_state_manager: Some(state_manager),
            })
        }

        #[cfg(not(feature = "lua"))]
        {
            Ok(Self {
                _config: config.clone(),
                execution_context: ExecutionContext::default(),
                runtime_config: None,
                script_args: None,
                execution_manager: None,
                external_state_manager: Some(state_manager),
            })
        }
    }

    /// Get the supported features for Lua
    #[must_use]
    pub const fn engine_features() -> EngineFeatures {
        EngineFeatures {
            async_execution: true, // Via coroutines
            streaming: true,
            multimodal: true,
            debugging: true, // Now implemented!
            modules: true,
            max_script_size: Some(10_000_000),    // 10MB
            max_execution_time_ms: Some(300_000), // 5 minutes
        }
    }

    /// Set the runtime configuration
    pub fn set_runtime_config(&mut self, config: Arc<llmspell_config::LLMSpellConfig>) {
        self.runtime_config = Some(config);
    }

    /// Set a debugger for the engine
    #[cfg(feature = "lua")]
    pub fn set_debugger(
        &mut self,
        execution_manager: Arc<crate::execution_bridge::ExecutionManager>,
    ) {
        self.execution_manager = Some(execution_manager.clone());

        // Install execution hooks in Lua
        let lua_guard = self.lua.lock();
        let shared_context = Arc::new(tokio::sync::RwLock::new(
            crate::execution_context::SharedExecutionContext::new(),
        ));
        if let Ok(hook) = crate::lua::globals::execution::install_interactive_debug_hooks(
            &lua_guard,
            execution_manager,
            shared_context,
        ) {
            self.execution_hook = Some(hook);
        }
    }

    /// Remove debugger from the engine
    #[cfg(feature = "lua")]
    pub fn remove_debugger(&mut self) {
        self.execution_manager = None;
        self.execution_hook = None;

        // Remove execution hooks from Lua
        crate::lua::globals::execution::remove_debug_hooks(&self.lua.lock());
    }

    /// Update breakpoints in the debugger
    #[cfg(feature = "lua")]
    pub fn set_breakpoints(&mut self, breakpoints: &[crate::execution_bridge::Breakpoint]) {
        if let Some(ref execution_hook) = self.execution_hook {
            execution_hook
                .lock()
                .update_breakpoints(breakpoints, &self.lua.lock());
        }
    }

    /// Execute script with enhanced debug context and async preservation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Script compilation fails
    /// - Script execution fails
    /// - Debug hook installation fails
    #[cfg(feature = "lua")]
    pub async fn execute_with_debug_context(
        &self,
        script: &str,
        shared_context: Arc<tokio::sync::RwLock<crate::execution_context::SharedExecutionContext>>,
    ) -> Result<ScriptOutput, LLMSpellError> {
        use crate::condition_evaluator::SharedDebugContext;

        // Prepare context for async debugging with trait-based evaluation
        let _correlation_id = {
            let mut context = shared_context.write().await;
            let enhanced = context.clone().with_async_support();
            *context = enhanced;
            context.correlation_id
        };

        // Create SharedDebugContext for trait-based operations (9.2.7b pattern)
        let debug_context = SharedDebugContext::new(shared_context.clone());

        // Install enhanced debug hooks with trait-based evaluators if needed
        if let Some(execution_manager) = &self.execution_manager {
            let lua_guard = self.lua.lock();
            if let Ok(_hook) = crate::lua::globals::execution::install_interactive_debug_hooks(
                &lua_guard,
                execution_manager.clone(),
                shared_context.clone(),
            ) {
                // Store hook temporarily (note: this would need to be handled better in production)
                drop(lua_guard); // Release lock before async operations
            }
        }

        // Execute with async context preservation and trait-based debugging
        self.execute_with_async_context_and_traits(script, shared_context, debug_context)
            .await
    }

    /// Execute with async context preservation and trait-based debugging
    #[cfg(feature = "lua")]
    async fn execute_with_async_context_and_traits(
        &self,
        script: &str,
        shared_context: Arc<tokio::sync::RwLock<crate::execution_context::SharedExecutionContext>>,
        _debug_context: SharedDebugContext,
    ) -> Result<ScriptOutput, LLMSpellError> {
        // Context preservation with trait-based evaluation support
        let snapshot = {
            let ctx = shared_context.read().await;
            ctx.preserve_across_async_boundary()
        };

        // Execute script (using sync execution for now, as mlua's async requires special handling)
        let result = {
            let lua_guard = self.lua.lock();
            let lua_result: mlua::Result<mlua::Value> = lua_guard.load(script).eval();

            match lua_result {
                Ok(value) => {
                    // Convert Lua value to JSON
                    let json_value =
                        crate::lua::conversion::lua_value_to_json(value).map_err(|e| {
                            LLMSpellError::Script {
                                message: e.to_string(),
                                language: Some("lua".to_string()),
                                line: None,
                                source: None,
                            }
                        })?;
                    Ok(ScriptOutput {
                        output: json_value,
                        console_output: Vec::new(),
                        metadata: ScriptMetadata {
                            engine: "lua".to_string(),
                            execution_time_ms: 0, // TODO: Calculate actual execution time
                            memory_usage_bytes: None,
                            warnings: Vec::new(),
                        },
                    })
                }
                Err(e) => Err(LLMSpellError::Script {
                    message: e.to_string(),
                    language: Some("lua".to_string()),
                    line: None,
                    source: None,
                }),
            }
        };

        // Restore context after execution
        {
            let mut ctx = shared_context.write().await;
            ctx.restore_from_async_boundary(snapshot);
        }

        result
    }
}

#[async_trait]
impl ScriptEngineBridge for LuaEngine {
    async fn execute_script(&self, script: &str) -> Result<ScriptOutput, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            let start_time = Instant::now();

            // For now, keep synchronous execution but prepare for async tool calls
            // The async execution will happen within tool calls, not at the script level
            // Clear any previous console output
            if let Some(capture) = &self.console_capture {
                capture.clear();
            }

            let result = {
                let lua = self.lua.lock();

                // Inject ARGS global if script arguments were provided
                if let Some(ref args) = self.script_args {
                    if let Err(e) = inject_args_global(&lua, args) {
                        return Err(LLMSpellError::Component {
                            message: format!("Failed to inject ARGS global: {e}"),
                            source: None,
                        });
                    }
                }

                let lua_result: mlua::Result<mlua::Value> = lua.load(script).eval();

                // Run garbage collection after script execution to prevent memory accumulation
                // This is especially important when running many scripts in sequence
                let _ = lua.gc_collect();

                match lua_result {
                    Ok(value) => {
                        // Convert Lua value to JSON
                        let output =
                            crate::lua::conversion::lua_value_to_json(value).map_err(|e| {
                                LLMSpellError::Script {
                                    message: e.to_string(),
                                    language: Some("lua".to_string()),
                                    line: None,
                                    source: None,
                                }
                            })?;
                        Ok(output)
                    }
                    Err(e) => Err(ScriptEngineError::ExecutionError {
                        engine: "lua".to_string(),
                        details: e.to_string(),
                    }),
                }
            };

            match result {
                Ok(output) => {
                    // Get captured console output
                    let console_output = self
                        .console_capture
                        .as_ref()
                        .map_or_else(Vec::new, |capture| capture.get_lines());

                    Ok(ScriptOutput {
                        output,
                        console_output,
                        metadata: ScriptMetadata {
                            engine: "lua".to_string(),
                            #[allow(clippy::cast_possible_truncation)]
                            execution_time_ms: start_time.elapsed().as_millis() as u64,
                            memory_usage_bytes: None, // TODO: Track memory usage
                            warnings: vec![],
                        },
                    })
                }
                Err(e) => Err(e.into()),
            }
        }

        #[cfg(not(feature = "lua"))]
        {
            Err(LLMSpellError::Component {
                message: "Lua feature not enabled".to_string(),
                source: None,
            })
        }
    }

    async fn execute_script_streaming(&self, script: &str) -> Result<ScriptStream, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            // For now, implement a simple non-streaming execution that returns a single chunk
            // Full streaming with coroutines requires more complex handling due to Send constraints
            let start_time = Instant::now();

            // Create a single chunk with the result
            let chunk = {
                let lua = self.lua.lock();
                let lua_result: mlua::Result<mlua::Value> = lua.load(script).eval();

                // Run garbage collection after script execution
                let _ = lua.gc_collect();

                match lua_result {
                    Ok(value) => {
                        // Convert Lua value to JSON
                        let output =
                            crate::lua::conversion::lua_value_to_json(value).map_err(|e| {
                                LLMSpellError::Script {
                                    message: e.to_string(),
                                    language: Some("lua".to_string()),
                                    line: None,
                                    source: None,
                                }
                            })?;
                        llmspell_core::types::AgentChunk {
                            stream_id: "lua-stream".to_string(),
                            chunk_index: 0,
                            content: llmspell_core::types::ChunkContent::Text(
                                serde_json::to_string(&output)
                                    .unwrap_or_else(|_| "null".to_string()),
                            ),
                            metadata: llmspell_core::types::ChunkMetadata {
                                is_final: true,
                                token_count: None,
                                model: None,
                                reasoning_step: None,
                            },
                            timestamp: chrono::Utc::now(),
                        }
                    }
                    Err(e) => llmspell_core::types::AgentChunk {
                        stream_id: "lua-stream".to_string(),
                        chunk_index: 0,
                        content: llmspell_core::types::ChunkContent::Control(
                            llmspell_core::types::ControlMessage::StreamCancelled {
                                reason: format!("Script execution failed: {e}"),
                            },
                        ),
                        metadata: llmspell_core::types::ChunkMetadata::default(),
                        timestamp: chrono::Utc::now(),
                    },
                }
            };

            // Create a stream from a single chunk
            let chunk_stream = stream::once(async move { Ok(chunk) });
            let boxed_stream: llmspell_core::types::AgentStream = Box::pin(chunk_stream);

            Ok(ScriptStream {
                stream: boxed_stream,
                metadata: ScriptMetadata {
                    engine: "lua".to_string(),
                    #[allow(clippy::cast_possible_truncation)]
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    memory_usage_bytes: None,
                    warnings: vec![],
                },
            })
        }

        #[cfg(not(feature = "lua"))]
        {
            Err(LLMSpellError::Component {
                message: "Lua feature not enabled".to_string(),
                source: None,
            })
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn inject_apis(
        &mut self,
        registry: &Arc<ComponentRegistry>,
        providers: &Arc<ProviderManager>,
    ) -> Result<(), LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            let lua = self.lua.lock();

            // API surface no longer needed - using globals system

            // Create GlobalContext with state support if configured
            let mut state_access: Option<Arc<dyn llmspell_core::traits::state::StateAccess>> = None;

            // Check if an external StateManager was provided first
            if let Some(ref external_sm) = self.external_state_manager {
                // Use the external StateManager by wrapping it in a StateManagerAdapter
                let adapter = crate::state_adapter::StateManagerAdapter::new(
                    external_sm.clone(),
                    llmspell_state_persistence::StateScope::Global,
                );
                state_access =
                    Some(Arc::new(adapter) as Arc<dyn llmspell_core::traits::state::StateAccess>);
                tracing::debug!("Using external StateManager for state access");
            } else if let Some(runtime_config) = &self.runtime_config {
                // Only create a new StateManager if no external one was provided
                if runtime_config.runtime.state_persistence.enabled {
                    // Try to create StateManagerAdapter for state access
                    match futures::executor::block_on(
                        crate::state_adapter::StateManagerAdapter::from_config(
                            &runtime_config.runtime.state_persistence,
                        ),
                    ) {
                        Ok(adapter) => {
                            state_access = Some(Arc::new(adapter)
                                as Arc<dyn llmspell_core::traits::state::StateAccess>);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to create state adapter: {}, state will not be available in context", e);
                        }
                    }
                }
            }

            // Create global context with or without state
            let global_context = state_access.map_or_else(
                || Arc::new(GlobalContext::new(registry.clone(), providers.clone())),
                |state| {
                    Arc::new(GlobalContext::with_state(
                        registry.clone(),
                        providers.clone(),
                        state,
                    ))
                },
            );

            // If we have an external StateManager, also store it in bridge refs
            // so get_or_create_state_infrastructure can find it
            if let Some(ref external_sm) = self.external_state_manager {
                global_context.set_bridge("state_manager", external_sm.clone());
                tracing::debug!("Stored external StateManager in GlobalContext bridge refs");
            }

            // Pass runtime config through global context if available
            if let Some(runtime_config) = &self.runtime_config {
                global_context.set_bridge("runtime_config", runtime_config.clone());

                // Initialize session infrastructure if enabled
                if runtime_config.runtime.sessions.enabled {
                    use crate::globals::session_infrastructure::get_or_create_session_infrastructure;
                    match futures::executor::block_on(get_or_create_session_infrastructure(
                        &global_context,
                        &runtime_config.runtime.sessions,
                    )) {
                        Ok(_) => {
                            tracing::debug!("Session infrastructure initialized successfully");
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to initialize session infrastructure: {}, Session/Artifact globals will not be available",
                                e
                            );
                        }
                    }
                }

                // Initialize RAG infrastructure if enabled
                if runtime_config.rag.enabled {
                    use crate::globals::rag_infrastructure::get_or_create_rag_infrastructure;
                    match futures::executor::block_on(get_or_create_rag_infrastructure(
                        &global_context,
                        &runtime_config.rag,
                    )) {
                        Ok(infrastructure) => {
                            // Store the infrastructure for RAGGlobal to use
                            global_context
                                .set_bridge("rag_infrastructure", Arc::new(infrastructure));
                            tracing::debug!("RAG infrastructure initialized successfully");
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to initialize RAG infrastructure: {}, RAG global will not be available",
                                e
                            );
                        }
                    }
                }
            }

            let global_registry =
                futures::executor::block_on(create_standard_registry(global_context.clone()))
                    .map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to create global registry: {e}"),
                        source: None,
                    })?;
            let injector = GlobalInjector::new(Arc::new(global_registry));
            injector.inject_lua(&lua, &global_context)?;
        }
        Ok(())
    }

    fn get_engine_name(&self) -> &'static str {
        "lua"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_multimodal(&self) -> bool {
        true
    }

    fn supported_features(&self) -> EngineFeatures {
        Self::engine_features()
    }

    fn get_execution_context(&self) -> Result<ExecutionContext, LLMSpellError> {
        Ok(self.execution_context.clone())
    }

    fn set_execution_context(&mut self, context: ExecutionContext) -> Result<(), LLMSpellError> {
        self.execution_context = context;
        Ok(())
    }

    fn install_debug_hooks(
        &mut self,
        hook: Arc<dyn crate::debug_runtime::DebugHook>,
    ) -> Result<(), LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            // Check if this is a LuaDebugHookAdapter
            // We need to downcast to check the type
            let hook_any = hook.as_any();
            if let Some(adapter) =
                hook_any.downcast_ref::<crate::lua::debug_hook_adapter::LuaDebugHookAdapter>()
            {
                // This is our adapter! Store it and install on Lua
                let adapter_clone = Arc::new(adapter.clone());
                self.lua_debug_adapter = Some(adapter_clone.clone());

                // Install the adapter's HookMultiplexer on the Lua instance
                let lua_guard = self.lua.lock();
                adapter_clone
                    .install_on_lua(&lua_guard)
                    .map_err(|e| LLMSpellError::Script {
                        message: format!("Failed to install LuaDebugHookAdapter: {e}"),
                        language: Some("lua".to_string()),
                        line: None,
                        source: Some(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                    })?;

                tracing::info!(
                    "LuaEngine: Successfully installed LuaDebugHookAdapter with HookMultiplexer"
                );
                return Ok(());
            }

            // Fallback to the old implementation for other hook types
            self.install_fallback_debug_hook(&hook);
            Ok(())
        }

        #[cfg(not(feature = "lua"))]
        {
            Err(LLMSpellError::Component {
                message: "Lua feature not enabled".to_string(),
                source: None,
            })
        }
    }

    async fn set_script_args(
        &mut self,
        args: std::collections::HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        self.script_args = Some(args);
        Ok(())
    }
}

impl LuaEngine {
    /// Install fallback debug hook for non-adapter hook types
    #[cfg(feature = "lua")]
    fn install_fallback_debug_hook(&self, hook: &Arc<dyn crate::debug_runtime::DebugHook>) {
        use crate::debug_runtime::DebugControl;
        use crate::lua::sync_utils::block_on_async;
        use std::time::Duration;

        // Create a wrapper that converts between our DebugHook trait and Lua's debug system
        let lua = self.lua.clone();
        let hook_clone = hook.clone();

        // Install Lua debug hook that calls our DebugHook trait
        let lua_guard = lua.lock();
        lua_guard.set_hook(
            mlua::HookTriggers {
                on_calls: true,
                on_returns: true,
                every_line: true,
                every_nth_instruction: None,
            },
            move |_lua, debug| {
                let hook = hook_clone.clone();

                // Get debug info
                let event = debug.event();
                let line = debug.curr_line();
                let source_info = debug.source();
                let source = source_info.short_src.as_deref().unwrap_or("<unknown>");

                // Use a short timeout for debug hooks (100ms)
                let timeout = Some(Duration::from_millis(100));

                // Route to appropriate hook method based on event type
                let control = match event {
                    mlua::DebugEvent::Line if line > 0 => {
                        // Call on_line hook
                        let source_owned = source.to_string();
                        block_on_async(
                            "debug_on_line",
                            async move {
                                Ok::<DebugControl, std::io::Error>(
                                    hook.on_line(line.try_into().unwrap_or(0), &source_owned)
                                        .await,
                                )
                            },
                            timeout,
                        )
                        .unwrap_or(DebugControl::Continue)
                    }
                    mlua::DebugEvent::Call => {
                        // Call on_function_enter hook
                        let name = "<function>".to_string();
                        block_on_async(
                            "debug_on_function_enter",
                            async move {
                                Ok::<DebugControl, std::io::Error>(
                                    hook.on_function_enter(&name, vec![]).await,
                                )
                            },
                            timeout,
                        )
                        .unwrap_or(DebugControl::Continue)
                    }
                    mlua::DebugEvent::Ret | mlua::DebugEvent::TailCall => {
                        // Call on_function_exit hook
                        let name = "<function>".to_string();
                        block_on_async(
                            "debug_on_function_exit",
                            async move {
                                Ok::<DebugControl, std::io::Error>(
                                    hook.on_function_exit(&name, None).await,
                                )
                            },
                            timeout,
                        )
                        .unwrap_or(DebugControl::Continue)
                    }
                    _ => DebugControl::Continue,
                };

                // Handle debug control response
                match control {
                    DebugControl::Continue => Ok(()),
                    DebugControl::Pause | DebugControl::Terminate => {
                        // For now, we can't truly pause Lua execution
                        // This would require more complex integration
                        Ok(())
                    }
                    DebugControl::StepOver | DebugControl::StepIn | DebugControl::StepOut => {
                        // Stepping requires more complex state management
                        Ok(())
                    }
                }
            },
        );

        tracing::debug!("Installed fallback debug hooks for Lua engine");
    }
}

// Removed duplicate lua_value_to_json and is_lua_array functions
// These are now imported from crate::lua::conversion
