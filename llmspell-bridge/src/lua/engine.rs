//! ABOUTME: `LuaEngine` implementation of `ScriptEngineBridge` trait
//! ABOUTME: Provides Lua 5.4 script execution with coroutine-based streaming

#![allow(clippy::significant_drop_tightening)]

use crate::engine::types::ScriptEngineError;
use crate::engine::{
    factory::LuaConfig, EngineFeatures, ExecutionContext, ScriptEngineBridge, ScriptMetadata,
    ScriptOutput, ScriptStream,
};
use crate::lua::globals::args::inject_args_global;
use crate::lua::output_capture::{install_output_capture, ConsoleCapture};
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn};

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
            debugging: false, // Not implemented yet
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

            // Check if state persistence is enabled and create state access
            if let Some(runtime_config) = &self.runtime_config {
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
                            warn!("Failed to create state adapter: {}, state will not be available in context", e);
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

    async fn set_script_args(
        &mut self,
        args: std::collections::HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        self.script_args = Some(args);
        Ok(())
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
