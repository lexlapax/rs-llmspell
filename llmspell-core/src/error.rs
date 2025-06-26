//! ABOUTME: Error types and handling for rs-llmspell
//! ABOUTME: Provides LLMSpellError enum and Result type alias

use thiserror::Error;

/// Comprehensive error enum for all LLMSpell operations
#[derive(Debug, Error)]
pub enum LLMSpellError {
    #[error("Component error: {message}")]
    Component { message: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("LLM provider error: {message}")]
    Provider { message: String },
    
    #[error("Script execution error: {message}")]
    Script { message: String },
    
    #[error("Tool execution error: {message}")]
    Tool { message: String },
    
    #[error("Workflow execution error: {message}")]
    Workflow { message: String },
    
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    #[error("Security violation: {message}")]
    Security { message: String },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Convenience Result type alias
pub type Result<T> = std::result::Result<T, LLMSpellError>;