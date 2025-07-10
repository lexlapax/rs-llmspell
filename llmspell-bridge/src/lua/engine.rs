//! ABOUTME: LuaEngine implementation of ScriptEngineBridge trait
//! ABOUTME: Provides Lua 5.4 script execution with coroutine-based streaming

use crate::engine::types::{ApiSurface, ScriptEngineError};
use crate::engine::{
    factory::{LuaConfig, StdlibLevel},
    EngineFeatures, ExecutionContext, ScriptEngineBridge, ScriptMetadata, ScriptOutput,
    ScriptStream,
};
use crate::{ComponentRegistry, ProviderManager};
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

/// Lua script engine implementation
///
/// Note: mlua requires unsafe Send/Sync implementation for thread safety
pub struct LuaEngine {
    #[cfg(feature = "lua")]
    lua: Arc<parking_lot::Mutex<mlua::Lua>>,
    _config: LuaConfig,
    #[cfg(feature = "lua")]
    api_injected: bool,
    execution_context: ExecutionContext,
}

// SAFETY: We ensure thread safety by using Mutex for all Lua access
unsafe impl Send for LuaEngine {}
unsafe impl Sync for LuaEngine {}

impl LuaEngine {
    /// Create a new Lua engine with the given configuration
    pub fn new(config: &LuaConfig) -> Result<Self, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            use mlua::Lua;

            // Create Lua instance (async is enabled via feature flag)
            let lua = match config.stdlib {
                StdlibLevel::None => Lua::new(),
                StdlibLevel::Safe => Lua::new(), // TODO: restrict stdlib
                StdlibLevel::Full => Lua::new(),
            };

            Ok(Self {
                lua: Arc::new(parking_lot::Mutex::new(lua)),
                _config: config.clone(),
                api_injected: false,
                execution_context: ExecutionContext::default(),
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
    pub fn engine_features() -> EngineFeatures {
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
}

#[async_trait]
impl ScriptEngineBridge for LuaEngine {
    async fn execute_script(&self, script: &str) -> Result<ScriptOutput, LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            if !self.api_injected {
                return Err(LLMSpellError::Component {
                    message: "APIs not injected. Call inject_apis first".to_string(),
                    source: None,
                });
            }

            let start_time = Instant::now();

            // For now, keep synchronous execution but prepare for async tool calls
            // The async execution will happen within tool calls, not at the script level
            let lua = self.lua.lock();

            // Execute the script
            let result: mlua::Result<mlua::Value> = lua.load(script).eval();

            match result {
                Ok(value) => {
                    // Convert Lua value to JSON
                    let output = lua_value_to_json(&lua, value)?;

                    Ok(ScriptOutput {
                        output,
                        console_output: vec![], // TODO: Capture console output
                        metadata: ScriptMetadata {
                            engine: "lua".to_string(),
                            execution_time_ms: start_time.elapsed().as_millis() as u64,
                            memory_usage_bytes: None, // TODO: Track memory usage
                            warnings: vec![],
                        },
                    })
                }
                Err(e) => Err(ScriptEngineError::ExecutionError {
                    engine: "lua".to_string(),
                    details: e.to_string(),
                }
                .into()),
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
            if !self.api_injected {
                return Err(LLMSpellError::Component {
                    message: "APIs not injected. Call inject_apis first".to_string(),
                    source: None,
                });
            }

            // For now, implement a simple non-streaming execution that returns a single chunk
            // Full streaming with coroutines requires more complex handling due to Send constraints
            let start_time = Instant::now();
            let lua = self.lua.lock();

            // Execute the script
            let result: mlua::Result<mlua::Value> = lua.load(script).eval();

            // Create a single chunk with the result
            let chunk = match result {
                Ok(value) => {
                    // Convert Lua value to JSON
                    let output = lua_value_to_json(&lua, value)?;
                    llmspell_core::types::AgentChunk {
                        stream_id: "lua-stream".to_string(),
                        chunk_index: 0,
                        content: llmspell_core::types::ChunkContent::Text(
                            serde_json::to_string(&output).unwrap_or_else(|_| "null".to_string()),
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
                            reason: format!("Script execution failed: {}", e),
                        },
                    ),
                    metadata: Default::default(),
                    timestamp: chrono::Utc::now(),
                },
            };

            // Create a stream from a single chunk
            use futures::stream;
            let chunk_stream = stream::once(async move { Ok(chunk) });
            let boxed_stream: llmspell_core::types::AgentStream = Box::pin(chunk_stream);

            Ok(ScriptStream {
                stream: boxed_stream,
                metadata: ScriptMetadata {
                    engine: "lua".to_string(),
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

    fn inject_apis(
        &mut self,
        registry: &Arc<ComponentRegistry>,
        providers: &Arc<ProviderManager>,
    ) -> Result<(), LLMSpellError> {
        #[cfg(feature = "lua")]
        {
            let lua = self.lua.lock();

            // Get the API surface definition
            let api_surface = ApiSurface::standard();

            // Inject Agent API
            super::api::inject_agent_api(
                &lua,
                &api_surface.agent_api,
                registry.clone(),
                providers.clone(),
            )?;

            // Inject Tool API
            super::api::inject_tool_api(&lua, &api_surface.tool_api, registry.clone())?;

            // Inject Workflow API
            super::api::inject_workflow_api(&lua, &api_surface.workflow_api, registry.clone())?;

            // Inject Streaming API
            super::api::inject_streaming_api(&lua, &api_surface.streaming_api)?;

            // Inject JSON API
            super::api::inject_json_api(&lua, &api_surface.json_api)?;

            self.api_injected = true;
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
}

#[cfg(feature = "lua")]
/// Convert a Lua value to JSON
fn lua_value_to_json(_lua: &mlua::Lua, value: mlua::Value) -> Result<Value, LLMSpellError> {
    use mlua::Value as LuaValue;

    match value {
        LuaValue::Nil => Ok(Value::Null),
        LuaValue::Boolean(b) => Ok(Value::Bool(b)),
        LuaValue::Integer(i) => Ok(Value::Number(i.into())),
        LuaValue::Number(n) => Ok(Value::from(n)),
        LuaValue::String(s) => {
            let str = s.to_str().map_err(|e| LLMSpellError::Component {
                message: format!("Failed to convert Lua string: {}", e),
                source: None,
            })?;
            Ok(Value::String(str.to_string()))
        }
        LuaValue::Table(table) => {
            // Check if it's an array
            if is_lua_array(&table) {
                let mut array = Vec::new();
                for i in 1..=table.len().map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to get table length: {}", e),
                    source: None,
                })? {
                    let value: LuaValue = table.get(i).map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to get table value: {}", e),
                        source: None,
                    })?;
                    array.push(lua_value_to_json(_lua, value)?);
                }
                Ok(Value::Array(array))
            } else {
                // It's an object
                let mut map = serde_json::Map::new();
                for pair in table.pairs::<LuaValue, LuaValue>() {
                    let (k, v) = pair.map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to iterate table: {}", e),
                        source: None,
                    })?;

                    if let LuaValue::String(key_str) = k {
                        let key = key_str.to_str().map_err(|e| LLMSpellError::Component {
                            message: format!("Failed to convert table key: {}", e),
                            source: None,
                        })?;
                        map.insert(key.to_string(), lua_value_to_json(_lua, v)?);
                    }
                }
                Ok(Value::Object(map))
            }
        }
        _ => Ok(Value::String(format!("<{:?}>", value))),
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
                mlua::Value::Integer(i) if i >= 1 && i <= len => continue,
                _ => return false,
            }
        }
        true
    } else {
        false
    }
}
