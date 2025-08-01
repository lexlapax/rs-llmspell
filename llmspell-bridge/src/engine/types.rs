//! ABOUTME: Common types for script engine abstraction layer
//! ABOUTME: Shared error types and engine abstractions (API definitions moved to globals)

use serde::{Deserialize, Serialize};

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

    // API surface tests removed - functionality moved to globals

    #[cfg_attr(test_category = "unit")]
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
