//! ABOUTME: Script execution trait for language-agnostic script runtime
//! ABOUTME: Defines the interface for executing scripts without cyclic dependencies

use crate::error::LLMSpellError;
use crate::traits::component_lookup::ComponentLookup;
use crate::traits::debug_context::DebugContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
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

    /// Execute a script with arguments and return the output
    ///
    /// # Arguments
    /// * `script` - The script source code to execute
    /// * `args` - HashMap of argument name to value that will be available as ARGS global
    ///
    /// # Returns
    /// The execution output including return value and console output
    async fn execute_script_with_args(
        &self,
        script: &str,
        _args: std::collections::HashMap<String, String>,
    ) -> Result<ScriptExecutionOutput, LLMSpellError> {
        // Default implementation just calls execute_script
        // Implementations should override this to properly inject args
        self.execute_script(script).await
    }

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

    /// Set debug context for debugging support
    ///
    /// Default implementation does nothing for backward compatibility.
    /// Executors that support debugging should override this method.
    /// Uses &self instead of &mut self to allow use with Arc\<dyn ScriptExecutor\>
    fn set_debug_context(&self, _context: Option<Arc<dyn DebugContext>>) {
        // Default: ignore (for backward compatibility)
    }

    /// Check if this executor supports debugging
    ///
    /// Default returns false. Executors with debug support should override.
    fn supports_debugging(&self) -> bool {
        false
    }

    /// Get the current debug context if set
    ///
    /// Default returns None. Executors with debug support should override.
    fn get_debug_context(&self) -> Option<Arc<dyn DebugContext>> {
        None
    }

    /// Access to component registry for tool discovery and invocation
    ///
    /// Returns the ComponentLookup implementation that provides access to
    /// tools, agents, and workflows. This allows kernels to query and
    /// execute actual components instead of using placeholders.
    ///
    /// Default returns None for backward compatibility.
    /// Executors with component registry should override this method.
    fn component_registry(&self) -> Option<Arc<dyn ComponentLookup>> {
        None
    }

    /// Access to template registry for template discovery and execution
    ///
    /// Returns the TemplateRegistry as a type-erased Arc<dyn Any> to avoid
    /// circular dependencies (llmspell-core can't depend on llmspell-templates).
    /// Callers should downcast to `Arc<TemplateRegistry>` using `Arc::downcast`.
    ///
    /// Default returns None for backward compatibility.
    /// Executors with template registry should override this method.
    ///
    /// # Type Erasure Pattern
    ///
    /// ```rust,ignore
    /// use std::any::Any;
    /// use llmspell_templates::registry::TemplateRegistry;
    ///
    /// if let Some(reg_any) = executor.template_registry_any() {
    ///     if let Ok(template_registry) = Arc::downcast::<TemplateRegistry>(reg_any.clone()) {
    ///         // Use template_registry
    ///     }
    /// }
    /// ```
    fn template_registry_any(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        None
    }

    /// Get completion candidates for the given line and cursor position
    ///
    /// This is used for REPL tab completion to suggest available variables,
    /// functions, and other completable elements in the script context.
    ///
    /// Default implementation returns empty vector for backward compatibility.
    /// Executors with completion support should override this method.
    ///
    /// # Arguments
    ///
    /// * `line` - The current line being edited
    /// * `cursor_pos` - The cursor position within the line
    ///
    /// # Returns
    ///
    /// A vector of (replacement_text, display_text) pairs for completion candidates
    fn get_completion_candidates(&self, _line: &str, _cursor_pos: usize) -> Vec<(String, String)> {
        Vec::new()
    }

    // === Template Operations (JSON-based API to avoid circular dependencies) ===

    /// List templates, optionally filtered by category
    ///
    /// Returns JSON array of template metadata. Category filter is string-based
    /// to avoid importing template types (e.g., "research", "chat", "analysis").
    ///
    /// Default returns empty array for backward compatibility.
    fn handle_template_list(&self, _category: Option<&str>) -> Result<Value, LLMSpellError> {
        Ok(serde_json::json!([]))
    }

    /// Get template information by ID, optionally including schema
    ///
    /// Returns JSON object with template metadata and optionally config schema.
    ///
    /// Default returns error for backward compatibility.
    fn handle_template_info(&self, _template_id: &str, _with_schema: bool) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Template operations not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Execute a template with given parameters
    ///
    /// Returns JSON object with execution result including output, metrics, and metadata.
    ///
    /// Default returns error for backward compatibility.
    async fn handle_template_exec(&self, _template_id: &str, _params: Value) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Template execution not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Search templates by query string, optionally filtered by category
    ///
    /// Returns JSON array of matching template metadata.
    ///
    /// Default returns empty array for backward compatibility.
    fn handle_template_search(&self, _query: &str, _category: Option<&str>) -> Result<Value, LLMSpellError> {
        Ok(serde_json::json!([]))
    }

    /// Get template configuration schema by ID
    ///
    /// Returns JSON object describing the template's parameter schema.
    ///
    /// Default returns error for backward compatibility.
    fn handle_template_schema(&self, _template_id: &str) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Template schema not supported by this executor".to_string(),
            source: None,
        })
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
