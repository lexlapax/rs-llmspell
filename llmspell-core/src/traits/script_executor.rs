//! ABOUTME: Script execution trait for language-agnostic script runtime
//! ABOUTME: Defines the interface for executing scripts without cyclic dependencies

use crate::error::LLMSpellError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Output from script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionOutput {
    /// The main output value from the script
    pub output: Value,
    /// Console/print output captured during execution
    pub console_output: Vec<String>,
    /// Execution metadata
    pub metadata: ScriptExecutionMetadata,
}

/// Metadata about script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionMetadata {
    /// Time taken to execute the script
    pub duration: Duration,
    /// Language the script was executed in
    pub language: String,
    /// Exit code if applicable
    pub exit_code: Option<i32>,
    /// Any errors that occurred but were handled
    pub warnings: Vec<String>,
}

/// Trait for executing scripts in various languages
///
/// This trait provides a language-agnostic interface for script execution,
/// allowing the kernel to execute scripts without depending on specific
/// script runtime implementations.
#[async_trait]
pub trait ScriptExecutor: Send + Sync {
    /// Execute a script and return the output
    ///
    /// # Arguments
    /// * `script` - The script source code to execute
    ///
    /// # Returns
    /// The execution output including return value and console output
    async fn execute_script(&self, script: &str) -> Result<ScriptExecutionOutput, LLMSpellError>;

    /// Check if streaming execution is supported
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Get the supported language for this executor
    fn language(&self) -> &'static str;

    /// Check if the executor is ready
    async fn is_ready(&self) -> bool {
        true
    }
}

/// Factory trait for creating script executors
#[async_trait]
pub trait ScriptExecutorFactory: Send + Sync {
    /// Create a new script executor for the given language
    async fn create_executor(
        &self,
        language: &str,
    ) -> Result<Box<dyn ScriptExecutor>, LLMSpellError>;

    /// List supported languages
    fn supported_languages(&self) -> Vec<String>;
}