//! ABOUTME: `ScriptEngineBridge` trait defining language-agnostic script engine interface
//! ABOUTME: Foundation for multi-language script execution (Lua, JavaScript, Python, etc.)

use async_trait::async_trait;
use llmspell_core::{
    error::LLMSpellError, traits::debug_context::DebugContext, types::AgentStream,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Dependencies for API injection into script engines
///
/// This struct bundles all the dependencies needed to inject APIs into script engines,
/// replacing the previous 8-parameter approach for better maintainability and clarity.
#[derive(Clone)]
pub struct ApiDependencies {
    /// Component registry for tools/agents/workflows (script layer)
    pub registry: Arc<crate::ComponentRegistry>,
    /// Provider manager for LLM access
    pub providers: Arc<crate::ProviderManager>,
    /// Tool registry from `ScriptRuntime` (infrastructure layer)
    pub tool_registry: Arc<llmspell_tools::ToolRegistry>,
    /// Agent factory registry from `ScriptRuntime` (infrastructure layer)
    pub agent_registry: Arc<llmspell_agents::FactoryRegistry>,
    /// Workflow factory from `ScriptRuntime` (infrastructure layer)
    pub workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,
    /// Optional `SessionManager` for template infrastructure (Phase 12.8.2.11)
    pub session_manager: Option<Arc<dyn std::any::Any + Send + Sync>>,
    /// Optional `StateManager` for state operations (Phase 13c.2.8.15)
    pub state_manager: Option<Arc<dyn std::any::Any + Send + Sync>>,
    /// Optional RAG infrastructure (Phase 13c.2.8.15)
    pub rag: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl ApiDependencies {
    /// Create new API dependencies with required components
    pub fn new(
        registry: Arc<crate::ComponentRegistry>,
        providers: Arc<crate::ProviderManager>,
        tool_registry: Arc<llmspell_tools::ToolRegistry>,
        agent_registry: Arc<llmspell_agents::FactoryRegistry>,
        workflow_factory: Arc<dyn llmspell_workflows::WorkflowFactory>,
    ) -> Self {
        Self {
            registry,
            providers,
            tool_registry,
            agent_registry,
            workflow_factory,
            session_manager: None,
            state_manager: None,
            rag: None,
        }
    }

    /// Add session manager to dependencies (builder pattern)
    #[must_use]
    pub fn with_session_manager(
        mut self,
        session_manager: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        self.session_manager = Some(session_manager);
        self
    }

    /// Add state manager to dependencies (builder pattern)
    #[must_use]
    pub fn with_state_manager(
        mut self,
        state_manager: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        self.state_manager = Some(state_manager);
        self
    }

    /// Add RAG infrastructure to dependencies (builder pattern)
    #[must_use]
    pub fn with_rag(mut self, rag: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        self.rag = Some(rag);
        self
    }
}

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
    /// - Session management APIs (if `SessionManager` provided)
    ///
    /// # Arguments
    ///
    /// * `deps` - API dependencies bundled in a struct for cleaner API
    ///
    /// # Errors
    ///
    /// Returns an error if API injection fails
    fn inject_apis(&mut self, deps: &ApiDependencies) -> Result<(), LLMSpellError>;

    /// Set script arguments to be made available in the script environment
    ///
    /// Arguments are passed as a `HashMap` and made available in a language-specific way:
    /// - Lua: Global `ARGS` table
    /// - JavaScript: Global `args` object
    /// - Python: `sys.argv` equivalent
    ///
    /// # Errors
    ///
    /// Returns an error if arguments cannot be set in the engine
    async fn set_script_args(&mut self, args: HashMap<String, String>)
        -> Result<(), LLMSpellError>;

    /// Get the name of this script engine
    fn get_engine_name(&self) -> &'static str;

    /// Check if this engine supports streaming execution
    fn supports_streaming(&self) -> bool;

    /// Check if this engine supports multimodal content
    fn supports_multimodal(&self) -> bool;

    /// Get the features supported by this engine
    fn supported_features(&self) -> EngineFeatures;

    /// Get the current execution context
    ///
    /// # Errors
    ///
    /// Returns an error if the execution context is invalid
    fn get_execution_context(&self) -> Result<ExecutionContext, LLMSpellError>;

    /// Set the execution context
    ///
    /// # Errors
    ///
    /// Returns an error if the execution context cannot be set
    fn set_execution_context(&mut self, context: ExecutionContext) -> Result<(), LLMSpellError>;

    /// Set debug context for debugging support
    ///
    /// Default implementation does nothing for backward compatibility.
    /// Engines that support debugging should override this method.
    /// Uses &self instead of &mut self to allow use with Arc
    fn set_debug_context(&self, _context: Option<Arc<dyn DebugContext>>) {
        // Default: ignore (for backward compatibility)
    }

    /// Check if this engine supports debugging
    ///
    /// Returns true if the engine has debug support capabilities.
    /// Default returns false for backward compatibility.
    fn supports_debugging(&self) -> bool {
        false
    }

    /// Get the current debug context if set
    ///
    /// Default returns None. Engines with debug support should override.
    fn get_debug_context(&self) -> Option<Arc<dyn DebugContext>> {
        None
    }

    /// Get completion candidates for interactive use (REPL, IDE)
    ///
    /// This is used for tab completion and IntelliSense-like features.
    /// Default implementation returns empty vector for backward compatibility.
    ///
    /// # Arguments
    ///
    /// * `context` - The completion context containing the text and cursor position
    ///
    /// # Returns
    ///
    /// A vector of completion candidates appropriate for the context
    fn get_completion_candidates(&self, _context: &CompletionContext) -> Vec<CompletionCandidate> {
        Vec::new()
    }

    /// Downcast support for accessing concrete engine types (Phase 12.8.2.10)
    ///
    /// Enables downcasting `Box<dyn ScriptEngineBridge>` to concrete engine types
    /// for calling engine-specific methods not in the trait.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Context for completion requests
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// The full line of text
    pub line: String,
    /// Cursor position in the line
    pub cursor_pos: usize,
    /// The word being completed (extracted from line and cursor)
    pub word: String,
    /// Position where the word starts
    pub word_start: usize,
}

impl CompletionContext {
    /// Create a new completion context
    #[must_use]
    pub fn new(line: &str, cursor_pos: usize) -> Self {
        // Extract the word being completed
        let before_cursor = &line[..cursor_pos.min(line.len())];
        let word_start = before_cursor
            .rfind(|c: char| c.is_whitespace() || c == '.' || c == ':' || c == '(' || c == ',')
            .map_or(0, |i| i + 1);

        let word = line[word_start..cursor_pos.min(line.len())].to_string();

        Self {
            line: line.to_string(),
            cursor_pos,
            word,
            word_start,
        }
    }

    /// Check if we're completing after a dot (member access)
    #[must_use]
    pub fn is_member_access(&self) -> Option<String> {
        if self.word_start > 0 {
            let before = &self.line[..self.word_start];
            if let Some(stripped) = before.strip_suffix('.') {
                // Extract the object name before the dot
                let obj_start = stripped
                    .rfind(|c: char| c.is_whitespace() || c == '(' || c == ',')
                    .map_or(0, |i| i + 1);
                return Some(stripped[obj_start..].to_string());
            }
        }
        None
    }

    /// Check if we're completing after a colon (method call)
    #[must_use]
    pub fn is_method_call(&self) -> Option<String> {
        if self.word_start > 0 {
            let before = &self.line[..self.word_start];
            if let Some(stripped) = before.strip_suffix(':') {
                let obj_start = stripped
                    .rfind(|c: char| c.is_whitespace() || c == '(' || c == ',')
                    .map_or(0, |i| i + 1);
                return Some(stripped[obj_start..].to_string());
            }
        }
        None
    }

    /// Check if we're inside function arguments (between parentheses)
    #[must_use]
    pub fn is_inside_function_args(&self) -> bool {
        // Check if we're inside parentheses by counting open/close before cursor
        let before_cursor = &self.line[..self.cursor_pos.min(self.line.len())];
        let open_count = before_cursor.matches('(').count();
        let close_count = before_cursor.matches(')').count();

        // We're inside if there are more open parens than close parens
        open_count > close_count
    }

    /// Get the function name if we're inside its arguments
    #[must_use]
    pub fn get_function_context(&self) -> Option<String> {
        if !self.is_inside_function_args() {
            return None;
        }

        // Find the last unmatched open paren before cursor
        let before_cursor = &self.line[..self.cursor_pos.min(self.line.len())];
        let mut depth = 0;
        let mut last_open_pos = None;

        // Iterate in reverse using char_indices
        let chars: Vec<(usize, char)> = before_cursor.char_indices().collect();
        for (i, ch) in chars.iter().rev() {
            match ch {
                ')' => depth += 1,
                '(' => {
                    if depth == 0 {
                        last_open_pos = Some(*i);
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        if let Some(pos) = last_open_pos {
            // Extract the function name before the open paren
            let before_paren = &self.line[..pos];
            let func_start = before_paren
                .rfind(|c: char| c.is_whitespace() || c == '=' || c == '(' || c == ',')
                .map_or(0, |i| i + 1);

            let func_name = before_paren[func_start..].trim();
            if !func_name.is_empty() {
                return Some(func_name.to_string());
            }
        }

        None
    }
}

/// A completion candidate
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    /// The text to insert
    pub text: String,
    /// The kind of completion
    pub kind: CompletionKind,
    /// Optional signature (for functions)
    pub signature: Option<String>,
    /// Optional documentation
    pub documentation: Option<String>,
}

/// Kind of completion candidate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    /// A variable
    Variable,
    /// A function
    Function,
    /// A method
    Method,
    /// A property/field
    Property,
    /// A language keyword
    Keyword,
    /// A module/library
    Module,
    /// Unknown/other
    Other,
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
#[allow(clippy::struct_excessive_bools)]
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
