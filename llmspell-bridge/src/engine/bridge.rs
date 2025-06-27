//! ABOUTME: ScriptEngineBridge trait defining language-agnostic script engine interface
//! ABOUTME: Foundation for multi-language script execution (Lua, JavaScript, Python, etc.)

use async_trait::async_trait;
use llmspell_core::{error::LLMSpellError, types::AgentStream};
use serde_json::Value;
use std::sync::Arc;

/// Core abstraction for script execution engines
///
/// This trait enables language-agnostic script execution by providing
/// a common interface that all script engines must implement.
#[async_trait]
pub trait ScriptEngineBridge: Send + Sync {
    /// Execute a script and return the output
    async fn execute_script(&self, script: &str) -> Result<ScriptOutput, LLMSpellError>;

    /// Execute a script with streaming output support
    async fn execute_script_streaming(&self, script: &str) -> Result<ScriptStream, LLMSpellError>;

    /// Inject language-agnostic APIs into the engine
    ///
    /// This method is called during initialization to inject:
    /// - Agent creation and execution APIs
    /// - Tool discovery and execution APIs
    /// - Workflow orchestration APIs
    /// - Provider access APIs
    fn inject_apis(
        &mut self,
        registry: &Arc<crate::ComponentRegistry>,
        providers: &Arc<crate::ProviderManager>,
    ) -> Result<(), LLMSpellError>;

    /// Get the name of this script engine
    fn get_engine_name(&self) -> &'static str;

    /// Check if this engine supports streaming execution
    fn supports_streaming(&self) -> bool;

    /// Check if this engine supports multimodal content
    fn supports_multimodal(&self) -> bool;

    /// Get the features supported by this engine
    fn supported_features(&self) -> EngineFeatures;

    /// Get the current execution context
    fn get_execution_context(&self) -> Result<ExecutionContext, LLMSpellError>;

    /// Set the execution context
    fn set_execution_context(&mut self, context: ExecutionContext) -> Result<(), LLMSpellError>;
}

/// Output from script execution
#[derive(Debug, Clone)]
pub struct ScriptOutput {
    /// The main output value
    pub output: Value,
    /// Any console/print output captured
    pub console_output: Vec<String>,
    /// Execution metadata
    pub metadata: ScriptMetadata,
}

/// Streaming output from script execution
pub struct ScriptStream {
    /// The underlying stream of outputs
    pub stream: AgentStream,
    /// Execution metadata
    pub metadata: ScriptMetadata,
}

/// Metadata about script execution
#[derive(Debug, Clone)]
pub struct ScriptMetadata {
    /// Engine that executed the script
    pub engine: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<usize>,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Features supported by a script engine
#[derive(Debug, Clone, Default)]
pub struct EngineFeatures {
    /// Supports async/await or coroutines
    pub async_execution: bool,
    /// Supports streaming output
    pub streaming: bool,
    /// Supports multimodal content
    pub multimodal: bool,
    /// Supports debugging/breakpoints
    pub debugging: bool,
    /// Supports module imports
    pub modules: bool,
    /// Maximum script size in bytes
    pub max_script_size: Option<usize>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

/// Execution context for scripts
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Current working directory
    pub working_directory: String,
    /// Environment variables
    pub environment: std::collections::HashMap<String, String>,
    /// Script-specific state
    pub state: Value,
    /// Security restrictions
    pub security: SecurityContext,
}

/// Security context for script execution
#[derive(Debug, Clone, Default)]
pub struct SecurityContext {
    /// Allow file system access
    pub allow_file_access: bool,
    /// Allow network access
    pub allow_network_access: bool,
    /// Allow process spawning
    pub allow_process_spawn: bool,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_features_default() {
        let features = EngineFeatures::default();
        assert!(!features.async_execution);
        assert!(!features.streaming);
        assert!(!features.multimodal);
        assert!(features.max_script_size.is_none());
    }

    #[test]
    fn test_security_context_default() {
        let security = SecurityContext::default();
        assert!(!security.allow_file_access);
        assert!(!security.allow_network_access);
        assert!(!security.allow_process_spawn);
    }
}
