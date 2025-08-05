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
            ScriptEngineError::ExecutionError { engine, details } => Self::Component {
                message: format!("{engine} engine execution error: {details}"),
                source: None,
            },
            ScriptEngineError::SyntaxError {
                engine,
                message,
                line,
                ..
            } => Self::Validation {
                field: Some("script".to_string()),
                message: format!(
                    "{engine} syntax error: {}",
                    line.map_or_else(|| message.clone(), |l| format!("{message} at line {l}"))
                ),
            },
            ScriptEngineError::UnsupportedFeature { engine, feature } => Self::Component {
                message: format!("Feature '{feature}' not supported by {engine} engine"),
                source: None,
            },
            _ => Self::Component {
                message: format!("Script engine error: {err:?}"),
                source: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // API surface tests removed - functionality moved to globals
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
