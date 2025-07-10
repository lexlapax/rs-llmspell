//! ABOUTME: Common types for script engine abstraction layer
//! ABOUTME: Shared types for language-agnostic API definitions and conversions

use serde::{Deserialize, Serialize};

/// Language-agnostic API surface definition
#[derive(Debug, Clone)]
pub struct ApiSurface {
    pub agent_api: AgentApiDefinition,
    pub tool_api: ToolApiDefinition,
    pub workflow_api: WorkflowApiDefinition,
    pub streaming_api: StreamingApiDefinition,
    pub json_api: JsonApiDefinition,
}

impl ApiSurface {
    /// Create the standard API surface that all engines should implement
    pub fn standard() -> Self {
        Self {
            agent_api: AgentApiDefinition::standard(),
            tool_api: ToolApiDefinition::standard(),
            workflow_api: WorkflowApiDefinition::standard(),
            streaming_api: StreamingApiDefinition::standard(),
            json_api: JsonApiDefinition::standard(),
        }
    }
}

/// Agent API definition for script engines
#[derive(Debug, Clone)]
pub struct AgentApiDefinition {
    /// Global object name (e.g., "Agent" in Lua/JS)
    pub global_name: String,
    /// Constructor function name
    pub constructor: String,
    /// Method names
    pub methods: AgentMethods,
}

impl AgentApiDefinition {
    pub fn standard() -> Self {
        Self {
            global_name: "Agent".to_string(),
            constructor: "create".to_string(),
            methods: AgentMethods {
                execute: "execute".to_string(),
                stream_execute: "streamExecute".to_string(),
                get_config: "getConfig".to_string(),
                get_state: "getState".to_string(),
                set_state: "setState".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentMethods {
    pub execute: String,
    pub stream_execute: String,
    pub get_config: String,
    pub get_state: String,
    pub set_state: String,
}

/// Tool API definition for script engines
#[derive(Debug, Clone)]
pub struct ToolApiDefinition {
    /// Global object name (e.g., "Tool" in Lua/JS)
    pub global_name: String,
    /// Function to get tools
    pub get_function: String,
    /// Function to list available tools
    pub list_function: String,
    /// Method names
    pub methods: ToolMethods,
}

impl ToolApiDefinition {
    pub fn standard() -> Self {
        Self {
            global_name: "Tool".to_string(),
            get_function: "get".to_string(),
            list_function: "list".to_string(),
            methods: ToolMethods {
                execute: "execute".to_string(),
                get_schema: "getSchema".to_string(),
                validate_input: "validateInput".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolMethods {
    pub execute: String,
    pub get_schema: String,
    pub validate_input: String,
}

/// Workflow API definition for script engines
#[derive(Debug, Clone)]
pub struct WorkflowApiDefinition {
    /// Global object name
    pub global_name: String,
    /// Workflow type constructors
    pub constructors: WorkflowConstructors,
}

impl WorkflowApiDefinition {
    pub fn standard() -> Self {
        Self {
            global_name: "Workflow".to_string(),
            constructors: WorkflowConstructors {
                sequential: "sequential".to_string(),
                parallel: "parallel".to_string(),
                conditional: "conditional".to_string(),
                loop_workflow: "loop".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowConstructors {
    pub sequential: String,
    pub parallel: String,
    pub conditional: String,
    pub loop_workflow: String,
}

/// Streaming API definition for script engines
#[derive(Debug, Clone)]
pub struct StreamingApiDefinition {
    /// How to create streams (coroutines in Lua, async generators in JS)
    pub stream_type: StreamType,
    /// Chunk handling
    pub chunk_methods: ChunkMethods,
}

impl StreamingApiDefinition {
    pub fn standard() -> Self {
        Self {
            stream_type: StreamType::Coroutine, // Default, engines override
            chunk_methods: ChunkMethods {
                yield_chunk: "yield".to_string(),
                next_chunk: "next".to_string(),
                is_done: "isDone".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum StreamType {
    /// Lua-style coroutines
    Coroutine,
    /// JavaScript async generators
    AsyncGenerator,
    /// Python async iterators
    AsyncIterator,
    /// Custom streaming mechanism
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ChunkMethods {
    pub yield_chunk: String,
    pub next_chunk: String,
    pub is_done: String,
}

/// JSON API definition for script engines
#[derive(Debug, Clone)]
pub struct JsonApiDefinition {
    /// Global object name (e.g., "JSON" in Lua/JS)
    pub global_name: String,
    /// Function to parse JSON string to native value
    pub parse_function: String,
    /// Function to stringify native value to JSON
    pub stringify_function: String,
}

impl JsonApiDefinition {
    pub fn standard() -> Self {
        Self {
            global_name: "JSON".to_string(),
            parse_function: "parse".to_string(),
            stringify_function: "stringify".to_string(),
        }
    }
}

/// Common error types for script engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptEngineError {
    /// Script execution failed
    ExecutionError { engine: String, details: String },

    /// Script syntax error
    SyntaxError {
        engine: String,
        message: String,
        line: Option<u32>,
        column: Option<u32>,
    },

    /// API injection failed
    ApiInjectionError { engine: String, api_name: String },

    /// Feature not supported by engine
    UnsupportedFeature { engine: String, feature: String },

    /// Type conversion failed
    TypeConversionError { engine: String, details: String },

    /// Engine not found
    EngineNotFound { engine_name: String },

    /// Engine configuration invalid
    ConfigurationError { engine: String, details: String },

    /// Resource limit exceeded
    ResourceLimitExceeded {
        engine: String,
        resource: String,
        limit: String,
    },
}

impl From<ScriptEngineError> for llmspell_core::error::LLMSpellError {
    fn from(err: ScriptEngineError) -> Self {
        match err {
            ScriptEngineError::ExecutionError { engine, details } => {
                llmspell_core::error::LLMSpellError::Component {
                    message: format!("{} engine execution error: {}", engine, details),
                    source: None,
                }
            }
            ScriptEngineError::SyntaxError {
                engine,
                message,
                line,
                ..
            } => {
                let detail = if let Some(l) = line {
                    format!("{} at line {}", message, l)
                } else {
                    message
                };
                llmspell_core::error::LLMSpellError::Validation {
                    field: Some("script".to_string()),
                    message: format!("{} syntax error: {}", engine, detail),
                }
            }
            ScriptEngineError::UnsupportedFeature { engine, feature } => {
                llmspell_core::error::LLMSpellError::Component {
                    message: format!("Feature '{}' not supported by {} engine", feature, engine),
                    source: None,
                }
            }
            _ => llmspell_core::error::LLMSpellError::Component {
                message: format!("Script engine error: {:?}", err),
                source: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_api_surface() {
        let api = ApiSurface::standard();
        assert_eq!(api.agent_api.global_name, "Agent");
        assert_eq!(api.tool_api.global_name, "Tool");
        assert_eq!(api.workflow_api.global_name, "Workflow");
    }

    #[test]
    fn test_script_engine_error_conversion() {
        let err = ScriptEngineError::ExecutionError {
            engine: "lua".to_string(),
            details: "test error".to_string(),
        };

        let llm_err: llmspell_core::error::LLMSpellError = err.into();
        match llm_err {
            llmspell_core::error::LLMSpellError::Component { message, .. } => {
                assert!(message.contains("lua engine"));
            }
            _ => panic!("Expected component error"),
        }
    }
}
