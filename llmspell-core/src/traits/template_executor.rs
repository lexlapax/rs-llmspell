//! ABOUTME: Template execution trait for workflow-template delegation
//! ABOUTME: Enables workflows to execute templates without circular dependencies

use crate::Result;
use async_trait::async_trait;

/// Template executor trait for workflow step execution
///
/// This trait abstracts template execution to avoid circular dependencies
/// between `llmspell-workflows` and `llmspell-bridge`.
///
/// # Architecture
///
/// - `llmspell-core`: Defines `TemplateExecutor` trait (this file)
/// - `llmspell-bridge`: Implements trait for `TemplateBridge`
/// - `llmspell-workflows`: Uses `Arc<dyn TemplateExecutor>` in `StepExecutionContext`
///
/// This design follows existing patterns (`StateAccess`, `EventEmitter`) and maintains
/// dependency hygiene: workflows remains low-level, bridge remains high-level.
///
/// # Phase
///
/// Introduced in Phase 13.13 (Workflow-Template Delegation) to enable templates
/// as composable workflow steps via `StepType::Template`.
///
/// # Example
///
/// ```
/// use llmspell_core::traits::template_executor::TemplateExecutor;
/// use std::sync::Arc;
/// use serde_json::json;
///
/// async fn execute_template_step(
///     executor: &Arc<dyn TemplateExecutor>,
///     template_id: &str,
/// ) -> Result<serde_json::Value, llmspell_core::LLMSpellError> {
///     let params = json!({
///         "topic": "Rust async programming",
///         "max_sources": 10,
///     });
///
///     executor.execute_template(template_id, params).await
/// }
/// ```
#[async_trait]
pub trait TemplateExecutor: Send + Sync {
    /// Execute a template with given parameters
    ///
    /// # Arguments
    ///
    /// * `template_id` - Template identifier (e.g., "research-assistant")
    /// * `params` - Template parameters as JSON value
    ///
    /// # Returns
    ///
    /// Template execution result as JSON value containing:
    /// - `result`: Template output (structure depends on template)
    /// - `metrics`: Execution metrics (duration, tokens, etc.)
    ///
    /// # Errors
    ///
    /// Returns `LLMSpellError` if:
    /// - Template not found in registry
    /// - Parameter validation fails against template schema
    /// - Template execution fails
    /// - Result serialization fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = json!({
    ///     "topic": "Rust ownership",
    ///     "max_sources": 5,
    /// });
    ///
    /// let result = executor.execute_template("research-assistant", params).await?;
    /// ```
    async fn execute_template(
        &self,
        template_id: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value>;
}
