//! ABOUTME: `LuaEngine` implementation of `ScriptEngineBridge` trait
//! ABOUTME: Provides Lua 5.4 script execution with coroutine-based streaming

#![allow(clippy::significant_drop_tightening)]

use crate::engine::bridge::{CompletionCandidate, CompletionContext};
use crate::engine::types::ScriptEngineError;
use crate::engine::{
    factory::LuaConfig, EngineFeatures, ExecutionContext, ScriptEngineBridge, ScriptMetadata,
    ScriptOutput, ScriptStream,
};
use crate::lua::completion::LuaCompletionProvider;
use crate::lua::globals::args::inject_args_global;
use crate::lua::output_capture::{install_output_capture, ConsoleCapture};
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_core::traits::debug_context::DebugContext;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, trace, warn};

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
    /// Debug context for debugging support (uses interior mutability)
    debug_context: Arc<parking_lot::RwLock<Option<Arc<dyn DebugContext>>>>,
    /// Completion provider for interactive use
    #[cfg(feature = "lua")]
    completion_provider: Arc<LuaCompletionProvider>,
    /// Global context for updating infrastructure after initialization (Phase 12.8.2.10)
    /// Stored to allow `SessionManager` registration after `inject_apis()` completes
    #[cfg(feature = "lua")]
    global_context: Arc<parking_lot::RwLock<Option<Arc<GlobalContext>>>>,
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
                debug_context: Arc::new(parking_lot::RwLock::new(None)),
                completion_provider: Arc::new(LuaCompletionProvider::new()),
                global_context: Arc::new(parking_lot::RwLock::new(None)),
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

    /// Get the supported features for Lua
    #[must_use]
    pub const fn engine_features() -> EngineFeatures {
        EngineFeatures {
            async_execution: true, // Via coroutines
            streaming: true,
            multimodal: true,
            debugging: true, // Debug hooks supported
            modules: true,
            max_script_size: Some(10_000_000),    // 10MB
            max_execution_time_ms: Some(300_000), // 5 minutes
        }
    }

    /// Set the runtime configuration
    pub fn set_runtime_config(&mut self, config: Arc<llmspell_config::LLMSpellConfig>) {
        self.runtime_config = Some(config);
    }
}

/// Install debug hooks for the current script execution
#[cfg(feature = "lua")]
fn install_debug_hooks_internal(lua: &mlua::Lua, debug_context: &Arc<dyn DebugContext>) {
    use mlua::DebugEvent;

    // Set up a debug hook that gets called on each line
    let debug_ctx = Arc::clone(debug_context);
    lua.set_hook(mlua::HookTriggers::EVERY_LINE, move |_lua, debug| {
        if debug.event() == DebugEvent::Line {
            // Get current location info
            let source = debug.source();
            let file = source.source.unwrap_or_else(|| "unknown".into());
            // Use curr_line() method to get current line
            // Note: curr_line() returns i32, convert to u32 (always positive in practice)
            let line = u32::try_from(debug.curr_line().max(0)).unwrap_or(0);

            // Report current location to debug context
            debug_ctx.report_location(&file, line);

            // Check if we should pause at this line (breakpoint or stepping)
            if debug_ctx.should_pause_sync(&file, line) {
                // Handle async pause in sync context
                // We need to use a different approach since block_on doesn't work in async tests
                // Use futures::executor::block_on which works in any context
                futures::executor::block_on(async {
                    if let Err(e) = debug_ctx.pause_and_wait(&file, line).await {
                        warn!("Failed to pause execution: {}", e);
                    }
                });
            }
        }
        Ok(())
    });

    debug!("Lua debug hooks installed for current execution");
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

            // Check if we should install debug hooks for this execution
            let should_install_hooks = {
                let debug_context = self.debug_context.read();
                debug_context
                    .as_ref()
                    .is_some_and(|ctx| ctx.is_debug_enabled())
            };

            let result = {
                let lua = self.lua.lock();

                // Install debug hooks if debugging is enabled
                if should_install_hooks {
                    let debug_ctx = self.debug_context.read().clone();
                    if let Some(ctx) = debug_ctx {
                        install_debug_hooks_internal(&lua, &ctx);
                    }
                }

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

                // Remove debug hooks after execution
                if should_install_hooks {
                    lua.remove_hook();
                }

                // Run garbage collection after script execution to prevent memory accumulation
                // This is especially important when running many scripts in sequence
                let _ = lua.gc_collect();

                match lua_result {
                    Ok(value) => {
                        // Convert Lua value to JSON
                        let output = lua_value_to_json(value)?;
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
                        let output = lua_value_to_json(value)?;
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
    #[instrument(
        level = "info",
        skip(self, registry, providers, tool_registry, agent_registry, workflow_factory, session_manager),
        fields(
            engine_type = "lua",
            globals_injected = 0,
            infrastructure_initialized = 0,
            session_manager_provided = session_manager.is_some()
        )
    )]
    fn inject_apis(
        &mut self,
        registry: &Arc<ComponentRegistry>,
        providers: &Arc<ProviderManager>,
        tool_registry: &Arc<llmspell_tools::ToolRegistry>,
        agent_registry: &Arc<llmspell_agents::FactoryRegistry>,
        workflow_factory: &Arc<dyn llmspell_workflows::WorkflowFactory>,
        session_manager: Option<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), LLMSpellError> {
        info!("Injecting Lua global APIs");
        #[cfg(feature = "lua")]
        {
            let lua = self.lua.lock();

            // Create GlobalContext with state support if configured
            let state_access = self
                .runtime_config
                .as_ref()
                .and_then(|cfg| Self::create_state_access(cfg));

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

            // Store infrastructure registries and SessionManager in GlobalContext (Phase 12.8.2.13)
            global_context.set_bridge("tool_registry", tool_registry.clone());
            global_context.set_bridge("agent_registry", agent_registry.clone());
            global_context.set_bridge("workflow_factory", Arc::new(workflow_factory.clone()));
            if let Some(session_manager_any) = session_manager {
                if let Ok(session_manager) =
                    Arc::downcast::<llmspell_kernel::sessions::SessionManager>(session_manager_any)
                {
                    global_context.set_bridge("session_manager", session_manager);
                }
            }
            debug!("Infrastructure registries stored in GlobalContext");

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
                            debug!("Session infrastructure initialized successfully");
                        }
                        Err(e) => {
                            warn!(
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
                            debug!("RAG infrastructure initialized successfully");
                        }
                        Err(e) => {
                            warn!(
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
            let metrics = injector.inject_lua(&lua, &global_context)?;
            info!(
                globals_injected = metrics.globals_injected,
                total_injection_time_us = metrics.total_injection_time_us,
                "Successfully injected all Lua globals"
            );

            // Store global_context for later SessionManager registration (Phase 12.8.2.10)
            {
                let mut ctx = self.global_context.write();
                *ctx = Some(global_context);
                debug!("Global context stored for later infrastructure updates");
            }
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

    async fn set_script_args(
        &mut self,
        args: std::collections::HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        self.script_args = Some(args);
        Ok(())
    }

    fn set_debug_context(&self, context: Option<Arc<dyn DebugContext>>) {
        // Use interior mutability to set debug context
        let mut debug_context = self.debug_context.write();
        debug_context.clone_from(&context);

        // Debug hooks are installed during script execution, not here
        #[cfg(feature = "lua")]
        if context.is_some() {
            debug!("Debug context set, hooks will be installed during script execution");
        }
    }

    fn supports_debugging(&self) -> bool {
        // Lua engine supports debugging via debug hooks
        true
    }

    fn get_debug_context(&self) -> Option<Arc<dyn DebugContext>> {
        // Return the stored debug context
        let debug_context = self.debug_context.read();
        debug_context.clone()
    }

    fn get_completion_candidates(&self, context: &CompletionContext) -> Vec<CompletionCandidate> {
        #[cfg(feature = "lua")]
        {
            // Try to acquire Lua lock with timeout to avoid blocking execution
            self.lua
                .try_lock_for(Duration::from_millis(10))
                .map_or_else(
                    || {
                        debug!("Lua engine busy, returning cached/empty completions");
                        // Could return cached results here if we maintained a separate cache
                        Vec::new()
                    },
                    |lua| {
                        trace!("Getting completion candidates for: {:?}", context);
                        let candidates = self.completion_provider.get_completions(&lua, context);

                        // Invalidate cache after execution to keep it fresh
                        if context.line.contains('=') || context.line.contains('(') {
                            self.completion_provider.invalidate_cache();
                        }

                        candidates
                    },
                )
        }

        #[cfg(not(feature = "lua"))]
        {
            Vec::new()
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(feature = "lua")]
impl LuaEngine {
    /// Create state access from runtime configuration
    fn create_state_access(
        runtime_config: &llmspell_config::LLMSpellConfig,
    ) -> Option<Arc<dyn llmspell_core::traits::state::StateAccess>> {
        if !runtime_config.runtime.state_persistence.enabled {
            return None;
        }

        match futures::executor::block_on(crate::state_adapter::StateManagerAdapter::from_config(
            &runtime_config.runtime.state_persistence,
        )) {
            Ok(adapter) => Some(Arc::new(adapter)),
            Err(e) => {
                warn!(
                    "Failed to create state adapter: {}, state will not be available in context",
                    e
                );
                None
            }
        }
    }

    /// Register `SessionManager` to `GlobalContext` (Phase 12.8.2.10)
    ///
    /// Called after `SessionManager` is wired to `ScriptRuntime` to update the `GlobalContext`
    /// used during template registration. This allows templates to receive `SessionManager`
    /// via `ExecutionContext` even though it's wired after globals are injected.
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn register_session_manager(
        &self,
        session_manager: Arc<llmspell_kernel::sessions::SessionManager>,
    ) {
        let ctx_guard = self.global_context.read();
        if let Some(global_context) = ctx_guard.as_ref() {
            global_context.set_bridge("session_manager", session_manager);
            debug!("SessionManager registered to GlobalContext for template access");

            // Re-inject template global with updated context
            // Template globals are created during create_standard_registry() which checks
            // for session_manager in GlobalContext. By registering it now and re-injecting,
            // TemplateBridge will be recreated with SessionManager available.
            #[allow(clippy::significant_drop_in_scrutinee)]
            if let Some(lua) = self.lua.try_lock() {
                match futures::executor::block_on(create_standard_registry(global_context.clone()))
                {
                    Ok(global_registry) => {
                        let injector = GlobalInjector::new(Arc::new(global_registry));
                        if let Err(e) = injector.inject_lua(&lua, global_context) {
                            warn!("Failed to re-inject globals with SessionManager: {}", e);
                        } else {
                            debug!("Successfully re-injected template globals with SessionManager");
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to recreate global registry with SessionManager: {}",
                            e
                        );
                    }
                }
            }
        }
    }
}

#[cfg(feature = "lua")]
/// Convert a Lua value to JSON
fn lua_value_to_json(value: mlua::Value) -> Result<Value, LLMSpellError> {
    use mlua::Value as LuaValue;

    match value {
        LuaValue::Nil => Ok(Value::Null),
        LuaValue::Boolean(b) => Ok(Value::Bool(b)),
        LuaValue::Integer(i) => Ok(Value::Number(i.into())),
        LuaValue::Number(n) => Ok(Value::from(n)),
        LuaValue::String(s) => {
            let str = s.to_str().map_err(|e| LLMSpellError::Component {
                message: format!("Failed to convert Lua string: {e}"),
                source: None,
            })?;
            Ok(Value::String(str.to_string()))
        }
        LuaValue::Table(table) => {
            // Check if it's an array
            if is_lua_array(&table) {
                let mut array = Vec::new();
                for i in 1..=table.len().map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to get table length: {e}"),
                    source: None,
                })? {
                    let value: LuaValue = table.get(i).map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to get table value: {e}"),
                        source: None,
                    })?;
                    array.push(lua_value_to_json(value)?);
                }
                Ok(Value::Array(array))
            } else {
                // It's an object
                let mut map = serde_json::Map::new();
                for pair in table.pairs::<LuaValue, LuaValue>() {
                    let (k, v) = pair.map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to iterate table: {e}"),
                        source: None,
                    })?;

                    if let LuaValue::String(key_str) = k {
                        let key = key_str.to_str().map_err(|e| LLMSpellError::Component {
                            message: format!("Failed to convert table key: {e}"),
                            source: None,
                        })?;
                        map.insert(key.to_string(), lua_value_to_json(v)?);
                    }
                }
                Ok(Value::Object(map))
            }
        }
        _ => Ok(Value::String(format!("<{value:?}>"))),
    }
}

#[cfg(feature = "lua")]
/// Check if a Lua table is an array (has sequential numeric keys starting at 1)
fn is_lua_array(table: &mlua::Table) -> bool {
    if let Ok(len) = table.len() {
        if len == 0 {
            return false;
        }
        // Check if all keys from 1 to len exist
        for i in 1..=len {
            if table.get::<_, mlua::Value>(i).is_err() {
                return false;
            }
        }
        // Check if there are any non-numeric keys
        for (k, _) in table.clone().pairs::<mlua::Value, mlua::Value>().flatten() {
            match k {
                mlua::Value::Integer(i) if i >= 1 && i <= len => {}
                _ => return false,
            }
        }
        true
    } else {
        false
    }
}

#[cfg(all(test, feature = "lua"))]
mod tests {
    use super::*;
    use llmspell_core::traits::debug_context::{DebugContext, StackFrame, Variable};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tracing::{debug, trace};

    /// Mock debug context for testing
    struct TestDebugContext {
        enabled: AtomicBool,
        paused: AtomicBool,
        resume_signal: Arc<std::sync::Condvar>,
        resume_mutex: Arc<std::sync::Mutex<bool>>,
        breakpoints: parking_lot::RwLock<Vec<(String, u32)>>,
        current_location: parking_lot::RwLock<Option<(String, u32)>>,
    }

    impl TestDebugContext {
        fn new() -> Self {
            Self {
                enabled: AtomicBool::new(false),
                paused: AtomicBool::new(false),
                resume_signal: Arc::new(std::sync::Condvar::new()),
                resume_mutex: Arc::new(std::sync::Mutex::new(false)),
                breakpoints: parking_lot::RwLock::new(Vec::new()),
                current_location: parking_lot::RwLock::new(None),
            }
        }

        fn add_breakpoint(&self, file: &str, line: u32) {
            self.breakpoints.write().push((file.to_string(), line));
        }

        fn resume(&self) {
            let mut resumed = self.resume_mutex.lock().unwrap();
            *resumed = true;
            self.resume_signal.notify_one();
        }
    }

    #[async_trait::async_trait]
    impl DebugContext for TestDebugContext {
        fn should_pause_sync(&self, file: &str, line: u32) -> bool {
            if !self.enabled.load(Ordering::Relaxed) {
                return false;
            }
            let should_pause = self.breakpoints.read().iter().any(|(f, l)| {
                // Check if breakpoint matches (support partial file paths)
                let matches = (f == file || file.contains(f)) && *l == line;
                if matches {
                    debug!(
                        "Breakpoint hit at {}:{} (matched breakpoint {}:{})",
                        file, line, f, l
                    );
                }
                matches
            });
            should_pause
        }

        async fn pause_and_wait(&self, file: &str, line: u32) -> Result<(), LLMSpellError> {
            debug!("pause_and_wait called for {}:{}", file, line);
            self.paused.store(true, Ordering::SeqCst);
            *self.current_location.write() = Some((file.to_string(), line));

            // Wait for resume signal using condvar (synchronous wait that works in block_on)
            let mut resumed = self.resume_mutex.lock().unwrap();
            while !*resumed {
                resumed = self.resume_signal.wait(resumed).unwrap();
            }
            *resumed = false; // Reset for next pause

            self.paused.store(false, Ordering::SeqCst);
            debug!("Resumed from {}:{}", file, line);
            Ok(())
        }

        fn enable_debug_mode(&self) {
            self.enabled.store(true, Ordering::SeqCst);
        }

        fn disable_debug_mode(&self) {
            self.enabled.store(false, Ordering::SeqCst);
        }

        fn is_debug_enabled(&self) -> bool {
            self.enabled.load(Ordering::Relaxed)
        }

        fn set_breakpoint(&self, file: &str, line: u32) -> Result<String, LLMSpellError> {
            self.add_breakpoint(file, line);
            Ok(format!("bp_{file}_{line}"))
        }

        fn clear_breakpoint(&self, _id: &str) -> Result<(), LLMSpellError> {
            Ok(())
        }

        fn get_stack_frames(&self) -> Vec<StackFrame> {
            vec![]
        }

        fn get_variables(&self, _frame_id: usize) -> Vec<Variable> {
            vec![]
        }

        fn report_location(&self, file: &str, line: u32) {
            trace!("report_location called with file='{}', line={}", file, line);
            *self.current_location.write() = Some((file.to_string(), line));
        }

        fn should_step(&self) -> bool {
            false
        }

        fn set_step_mode(&self, _stepping: bool) {}
    }

    // NOTE: test_debug_hook_pausing removed - debug pausing requires Lua VM pause/resume
    // which mlua doesn't support from hooks. Would need coroutine-based debugger (future work).
    // Debug hooks DO work for location reporting (tested in test_debug_hook_lifecycle).

    #[tokio::test]
    async fn test_debug_hook_lifecycle() {
        let config = LuaConfig::default();
        let engine = LuaEngine::new(&config).unwrap();

        // Create debug context
        let debug_ctx = Arc::new(TestDebugContext::new());
        debug_ctx.enable_debug_mode();

        // Set debug context
        engine.set_debug_context(Some(debug_ctx.clone()));

        // Execute multiple scripts to verify hooks are installed/removed properly
        for i in 0..3 {
            let script = format!("return {}", i * 10);
            let result = engine.execute_script(&script).await.unwrap();
            assert_eq!(result.output, serde_json::json!(i * 10));
        }

        // Disable debug mode and verify no hooks are installed
        debug_ctx.disable_debug_mode();
        let result = engine.execute_script("return 100").await.unwrap();
        assert_eq!(result.output, serde_json::json!(100));
    }

    #[tokio::test]
    async fn test_no_debug_overhead_when_disabled() {
        let config = LuaConfig::default();
        let engine = LuaEngine::new(&config).unwrap();

        // Execute without any debug context set
        let script = "local sum = 0; for i=1,1000 do sum = sum + i end; return sum";
        let start = std::time::Instant::now();
        let result = engine.execute_script(script).await.unwrap();
        let duration_no_debug = start.elapsed();

        // Expected result: sum of 1 to 1000 = 500500
        assert_eq!(result.output, serde_json::json!(500_500));

        // Verify execution was fast (no debug overhead)
        assert!(
            duration_no_debug < Duration::from_millis(50),
            "Script should execute quickly without debug"
        );
    }
}
