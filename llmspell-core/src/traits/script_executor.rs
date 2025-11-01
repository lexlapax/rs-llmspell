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
pub trait ScriptExecutor: Send + Sync + 'static {
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

    /// Set session manager for template infrastructure (Phase 12.8.2.5)
    ///
    /// Uses type erasure to avoid circular dependency between llmspell-core and llmspell-kernel.
    /// The session manager is passed as `Arc<dyn Any>` and implementations should downcast it
    /// to the concrete `SessionManager` type.
    ///
    /// Default implementation does nothing for backward compatibility.
    /// Executors with session support should override this method.
    ///
    /// # Type Erasure Pattern
    ///
    /// ```rust,ignore
    /// use std::any::Any;
    /// use std::sync::Arc;
    /// use llmspell_kernel::sessions::SessionManager;
    ///
    /// // In kernel initialization:
    /// let session_manager = Arc::new(SessionManager::new(...)?);
    /// script_executor.set_session_manager_any(
    ///     session_manager as Arc<dyn Any + Send + Sync>
    /// );
    /// ```
    fn set_session_manager_any(&self, _manager: Arc<dyn std::any::Any + Send + Sync>) {
        // Default: ignore (for backward compatibility)
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
    /// Returns the TemplateRegistry as a type-erased `Arc<dyn Any>` to avoid
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
    fn handle_template_info(
        &self,
        _template_id: &str,
        _with_schema: bool,
    ) -> Result<Value, LLMSpellError> {
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
    async fn handle_template_exec(
        &self,
        _template_id: &str,
        _params: Value,
    ) -> Result<Value, LLMSpellError> {
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
    fn handle_template_search(
        &self,
        _query: &str,
        _category: Option<&str>,
    ) -> Result<Value, LLMSpellError> {
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

    // === Memory Operations (Phase 13.12.1 - follows template pattern) ===

    /// Add episodic memory entry
    ///
    /// Returns JSON object with status. Follows template operation pattern.
    ///
    /// Default returns error for backward compatibility.
    fn handle_memory_add(
        &self,
        _session_id: &str,
        _role: &str,
        _content: &str,
        _metadata: Value,
    ) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Memory add not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Search episodic memory
    ///
    /// Returns JSON array of matching memory entries.
    ///
    /// Default returns error for backward compatibility.
    fn handle_memory_search(
        &self,
        _session_id: Option<&str>,
        _query: &str,
        _limit: usize,
    ) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Memory search not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Query semantic knowledge graph
    ///
    /// Returns JSON array of matching entities.
    ///
    /// Default returns error for backward compatibility.
    fn handle_memory_query(&self, _query: &str, _limit: usize) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Memory query not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Get memory statistics
    ///
    /// Returns JSON object with memory statistics.
    ///
    /// Default returns error for backward compatibility.
    fn handle_memory_stats(&self) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Memory stats not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Consolidate episodic to semantic memory
    ///
    /// Returns JSON object with consolidation results.
    ///
    /// Default returns error for backward compatibility.
    fn handle_memory_consolidate(
        &self,
        _session_id: Option<&str>,
        _force: bool,
    ) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Memory consolidate not supported by this executor".to_string(),
            source: None,
        })
    }

    // === Context Operations (Phase 13.12.3 - follows template pattern) ===

    /// Assemble context with specified strategy
    ///
    /// Returns JSON object with assembled context chunks.
    ///
    /// Default returns error for backward compatibility.
    fn handle_context_assemble(
        &self,
        _query: &str,
        _strategy: &str,
        _budget: usize,
        _session_id: Option<&str>,
    ) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Context assemble not supported by this executor".to_string(),
            source: None,
        })
    }

    /// List available context strategies
    ///
    /// Returns JSON array of strategy metadata.
    ///
    /// Default returns hardcoded list.
    fn handle_context_strategies(&self) -> Result<Value, LLMSpellError> {
        use serde_json::json;
        Ok(json!([
            {
                "name": "hybrid",
                "description": "Combines RAG, episodic, and semantic memory (recommended)"
            },
            {
                "name": "episodic",
                "description": "Conversation history only"
            },
            {
                "name": "semantic",
                "description": "Knowledge graph entities only"
            },
            {
                "name": "rag",
                "description": "Document retrieval only"
            }
        ]))
    }

    /// Analyze token usage across strategies
    ///
    /// Returns JSON array with per-strategy analysis.
    ///
    /// Default returns error for backward compatibility.
    fn handle_context_analyze(&self, _query: &str, _budget: usize) -> Result<Value, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Context analyze not supported by this executor".to_string(),
            source: None,
        })
    }

    /// Downcast support for accessing concrete executor implementations (Phase 12.8.fix)
    ///
    /// Enables downcasting from `Arc<dyn ScriptExecutor>` to concrete types like `ScriptRuntime`.
    /// This is needed for wiring infrastructure components that aren't part of the trait
    /// (like RAG, state manager) without creating circular dependencies.
    ///
    /// Implementers must return `self` to enable downcasting.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///     self
    /// }
    /// ```
    fn as_any(&self) -> &dyn std::any::Any;
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
