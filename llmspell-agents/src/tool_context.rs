//! ABOUTME: Tool execution context integration for enhanced context propagation
//! ABOUTME: Provides context enrichment, inheritance, and tool-specific context handling

#![allow(clippy::significant_drop_tightening)]

use llmspell_core::{ExecutionContext, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

/// Enhanced execution context for tool operations that provides
/// additional tool-specific context, inheritance, and state management.
///
/// This wraps the base `ExecutionContext` to provide tool-aware
/// context propagation and management.
///
/// # Examples
///
/// ```
/// use llmspell_agents::tool_context::{ToolExecutionContext, ContextInheritanceRule};
/// use llmspell_core::ExecutionContext;
/// use serde_json::json;
///
/// # async fn example() -> llmspell_core::Result<()> {
/// let base_context = ExecutionContext::new();
/// let mut tool_context = ToolExecutionContext::new(base_context);
///
/// // Add tool-specific data
/// tool_context.set_tool_data("current_tool", json!({"name": "file_processor"})).await;
///
/// // Set up inheritance
/// tool_context.set_inheritance_rule("user_preferences", ContextInheritanceRule::Inherit);
///
/// // Create child context for nested tool execution
/// let child_context = tool_context.create_child_context("child_tool").await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolExecutionContext {
    /// Base execution context
    base_context: ExecutionContext,
    /// Tool-specific data storage
    tool_data: Arc<RwLock<HashMap<String, JsonValue>>>,
    /// Shared data that can be accessed by all tools in the execution chain
    shared_data: Arc<RwLock<HashMap<String, JsonValue>>>,
    /// Context inheritance rules
    inheritance_rules: Arc<RwLock<HashMap<String, ContextInheritanceRule>>>,
    /// Parent context reference (for hierarchical contexts)
    parent_context: Option<Arc<ToolExecutionContext>>,
    /// Context identifier
    context_id: String,
    /// Tool execution history
    execution_history: Arc<RwLock<Vec<ToolExecutionRecord>>>,
}

/// Rules for how context data should be inherited in nested tool executions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextInheritanceRule {
    /// Inherit value from parent context
    Inherit,
    /// Do not inherit, start fresh
    Isolate,
    /// Inherit but create a copy (modifications don't affect parent)
    Copy,
    /// Inherit and share (modifications affect parent)
    Share,
    /// Custom inheritance logic
    Custom(String),
}

/// Record of tool execution within a context
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionRecord {
    /// Name of the tool that was executed
    pub tool_name: String,
    /// Parameters passed to the tool
    pub parameters: JsonValue,
    /// Execution start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Execution duration
    pub duration: Option<std::time::Duration>,
    /// Whether execution was successful
    pub success: bool,
    /// Output or error message
    pub result: String,
    /// Tool-specific metadata
    pub metadata: HashMap<String, JsonValue>,
}

/// Context enhancement options
#[derive(Debug, Clone)]
pub struct ContextEnhancementOptions {
    /// Whether to track tool execution history
    pub track_execution_history: bool,
    /// Whether to enable context inheritance
    pub enable_inheritance: bool,
    /// Whether to enable shared data across tools
    pub enable_shared_data: bool,
    /// Maximum number of execution records to keep
    pub max_execution_history: usize,
    /// Default inheritance rule for new data
    pub default_inheritance_rule: ContextInheritanceRule,
}

impl Default for ContextEnhancementOptions {
    fn default() -> Self {
        Self {
            track_execution_history: true,
            enable_inheritance: true,
            enable_shared_data: true,
            max_execution_history: 100,
            default_inheritance_rule: ContextInheritanceRule::Copy,
        }
    }
}

impl ToolExecutionContext {
    /// Create a new tool execution context
    #[must_use]
    pub fn new(base_context: ExecutionContext) -> Self {
        Self {
            base_context,
            tool_data: Arc::new(RwLock::new(HashMap::new())),
            shared_data: Arc::new(RwLock::new(HashMap::new())),
            inheritance_rules: Arc::new(RwLock::new(HashMap::new())),
            parent_context: None,
            context_id: uuid::Uuid::new_v4().to_string(),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new tool execution context with options
    #[must_use]
    pub fn with_options(
        base_context: ExecutionContext,
        _options: ContextEnhancementOptions,
    ) -> Self {
        // For now, options are ignored, but can be used to configure behavior
        Self::new(base_context)
    }

    /// Get the base execution context
    #[must_use]
    pub const fn base_context(&self) -> &ExecutionContext {
        &self.base_context
    }

    /// Get the context ID
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to &self reference
    pub fn context_id(&self) -> &str {
        &self.context_id
    }

    /// Set tool-specific data
    #[instrument(skip(self))]
    pub async fn set_tool_data(&self, key: impl Into<String> + std::fmt::Debug, value: JsonValue) {
        let mut data = self.tool_data.write().await;
        data.insert(key.into(), value);
    }

    /// Get tool-specific data
    #[instrument(skip(self))]
    pub async fn get_tool_data(&self, key: &str) -> Option<JsonValue> {
        let data = self.tool_data.read().await;
        data.get(key).cloned()
    }

    /// Set shared data that can be accessed by all tools
    #[instrument(skip(self))]
    pub async fn set_shared_data(
        &self,
        key: impl Into<String> + std::fmt::Debug,
        value: JsonValue,
    ) {
        let mut data = self.shared_data.write().await;
        data.insert(key.into(), value);
    }

    /// Get shared data
    #[instrument(skip(self))]
    pub async fn get_shared_data(&self, key: &str) -> Option<JsonValue> {
        let data = self.shared_data.read().await;
        data.get(key).cloned()
    }

    /// Set inheritance rule for a data key
    #[instrument(skip(self))]
    pub async fn set_inheritance_rule(
        &self,
        key: impl Into<String> + std::fmt::Debug,
        rule: ContextInheritanceRule,
    ) {
        let mut rules = self.inheritance_rules.write().await;
        rules.insert(key.into(), rule);
    }

    /// Get inheritance rule for a data key
    #[instrument(skip(self))]
    pub async fn get_inheritance_rule(&self, key: &str) -> ContextInheritanceRule {
        let rules = self.inheritance_rules.read().await;
        rules
            .get(key)
            .cloned()
            .unwrap_or(ContextInheritanceRule::Copy)
    }

    /// Create a child context for nested tool execution
    ///
    /// # Errors
    ///
    /// Returns an error if context creation fails
    #[instrument(skip(self))]
    pub async fn create_child_context(
        &self,
        child_id: impl Into<String> + std::fmt::Debug,
    ) -> Result<Self> {
        // Collect inheritance data first
        let parent_data = self.tool_data.read().await;
        let parent_rules = self.inheritance_rules.read().await;

        let mut inherited_data = HashMap::new();
        for (key, value) in parent_data.iter() {
            let rule = parent_rules
                .get(key)
                .unwrap_or(&ContextInheritanceRule::Copy);
            match rule {
                ContextInheritanceRule::Inherit | ContextInheritanceRule::Copy => {
                    inherited_data.insert(key.clone(), value.clone());
                }
                ContextInheritanceRule::Share => {
                    // For shared inheritance, we would need more complex logic
                    // For now, treat as copy
                    inherited_data.insert(key.clone(), value.clone());
                }
                ContextInheritanceRule::Isolate => {
                    // Don't inherit this value
                }
                ContextInheritanceRule::Custom(_) => {
                    // Custom logic would be implemented here
                    inherited_data.insert(key.clone(), value.clone());
                }
            }
        }

        // Drop the locks before creating child
        drop(parent_data);
        drop(parent_rules);

        let child = Self {
            base_context: self.base_context.clone(),
            tool_data: Arc::new(RwLock::new(inherited_data)),
            shared_data: self.shared_data.clone(), // Share with parent
            inheritance_rules: Arc::new(RwLock::new(HashMap::new())),
            parent_context: Some(Arc::new(self.clone())),
            context_id: format!("{}::{}", self.context_id, child_id.into()),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        };

        Ok(child)
    }

    /// Record tool execution
    #[instrument(skip(self))]
    pub async fn record_execution(
        &self,
        tool_name: impl Into<String> + std::fmt::Debug,
        parameters: JsonValue,
        success: bool,
        result: impl Into<String> + std::fmt::Debug,
        duration: Option<std::time::Duration>,
    ) {
        let record = ToolExecutionRecord {
            tool_name: tool_name.into(),
            parameters,
            start_time: chrono::Utc::now(),
            duration,
            success,
            result: result.into(),
            metadata: HashMap::new(),
        };

        let mut history = self.execution_history.write().await;
        history.push(record);

        // Keep only recent executions (prevent memory leaks)
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// Get execution history
    #[instrument(skip(self))]
    pub async fn get_execution_history(&self) -> Vec<ToolExecutionRecord> {
        let history = self.execution_history.read().await;
        history.clone()
    }

    /// Get the last executed tool
    #[instrument(skip(self))]
    pub async fn get_last_execution(&self) -> Option<ToolExecutionRecord> {
        let history = self.execution_history.read().await;
        history.last().cloned()
    }

    /// Check if a tool has been executed in this context
    #[instrument(skip(self))]
    pub async fn has_executed_tool(&self, tool_name: &str) -> bool {
        let history = self.execution_history.read().await;
        history.iter().any(|record| record.tool_name == tool_name)
    }

    /// Get all data as a combined view
    #[instrument(skip(self))]
    pub async fn get_all_data(&self) -> HashMap<String, JsonValue> {
        let mut all_data = HashMap::new();

        // Start with tool data
        let tool_data = self.tool_data.read().await;
        all_data.extend(tool_data.clone());

        // Add shared data
        let shared_data = self.shared_data.read().await;
        for (key, value) in shared_data.iter() {
            all_data.insert(format!("shared::{key}"), value.clone());
        }

        all_data
    }

    /// Export context state for serialization
    #[instrument(skip(self))]
    pub async fn export_state(&self) -> ContextState {
        ContextState {
            context_id: self.context_id.clone(),
            tool_data: self.tool_data.read().await.clone(),
            shared_data: self.shared_data.read().await.clone(),
            inheritance_rules: self.inheritance_rules.read().await.clone(),
            execution_history: self.execution_history.read().await.clone(),
        }
    }

    /// Import context state from serialization
    ///
    /// # Errors
    ///
    /// Returns an error if state import fails
    #[instrument(skip(self))]
    pub async fn import_state(&self, state: ContextState) -> Result<()> {
        {
            let mut tool_data = self.tool_data.write().await;
            *tool_data = state.tool_data;
        }

        {
            let mut shared_data = self.shared_data.write().await;
            *shared_data = state.shared_data;
        }

        {
            let mut inheritance_rules = self.inheritance_rules.write().await;
            *inheritance_rules = state.inheritance_rules;
        }

        {
            let mut execution_history = self.execution_history.write().await;
            *execution_history = state.execution_history;
        }

        Ok(())
    }

    /// Convert to base `ExecutionContext` for tool invocation
    #[must_use]
    pub fn to_execution_context(&self) -> ExecutionContext {
        self.base_context.clone()
    }
}

impl Clone for ToolExecutionContext {
    fn clone(&self) -> Self {
        Self {
            base_context: self.base_context.clone(),
            tool_data: self.tool_data.clone(),
            shared_data: self.shared_data.clone(),
            inheritance_rules: self.inheritance_rules.clone(),
            parent_context: self.parent_context.clone(),
            context_id: self.context_id.clone(),
            execution_history: self.execution_history.clone(),
        }
    }
}

/// Serializable context state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextState {
    pub context_id: String,
    pub tool_data: HashMap<String, JsonValue>,
    pub shared_data: HashMap<String, JsonValue>,
    pub inheritance_rules: HashMap<String, ContextInheritanceRule>,
    pub execution_history: Vec<ToolExecutionRecord>,
}

/// Context manager for handling multiple tool execution contexts
pub struct ToolContextManager {
    contexts: Arc<RwLock<HashMap<String, Arc<ToolExecutionContext>>>>,
    default_options: ContextEnhancementOptions,
}

impl ToolContextManager {
    /// Create a new context manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            default_options: ContextEnhancementOptions::default(),
        }
    }

    /// Create a new context manager with options
    #[must_use]
    pub fn with_options(options: ContextEnhancementOptions) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            default_options: options,
        }
    }

    /// Create and register a new context
    #[instrument(skip(self))]
    pub async fn create_context(
        &self,
        context_id: impl Into<String> + std::fmt::Debug,
        base_context: ExecutionContext,
    ) -> Arc<ToolExecutionContext> {
        let context_id = context_id.into();
        let tool_context = Arc::new(ToolExecutionContext::with_options(
            base_context,
            self.default_options.clone(),
        ));

        let mut contexts = self.contexts.write().await;
        contexts.insert(context_id.clone(), tool_context.clone());

        tool_context
    }

    /// Get an existing context
    #[instrument(skip(self))]
    pub async fn get_context(&self, context_id: &str) -> Option<Arc<ToolExecutionContext>> {
        let contexts = self.contexts.read().await;
        contexts.get(context_id).cloned()
    }

    /// Remove a context
    #[instrument(skip(self))]
    pub async fn remove_context(&self, context_id: &str) -> bool {
        let mut contexts = self.contexts.write().await;
        contexts.remove(context_id).is_some()
    }

    /// List all context IDs
    #[instrument(skip(self))]
    pub async fn list_contexts(&self) -> Vec<String> {
        let contexts = self.contexts.read().await;
        contexts.keys().cloned().collect()
    }

    /// Get context count
    #[instrument(skip(self))]
    pub async fn context_count(&self) -> usize {
        let contexts = self.contexts.read().await;
        contexts.len()
    }
}

impl Default for ToolContextManager {
    fn default() -> Self {
        Self::new()
    }
}

// Implement serialization for ContextInheritanceRule
impl serde::Serialize for ContextInheritanceRule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Inherit => serializer.serialize_str("inherit"),
            Self::Isolate => serializer.serialize_str("isolate"),
            Self::Copy => serializer.serialize_str("copy"),
            Self::Share => serializer.serialize_str("share"),
            Self::Custom(s) => serializer.serialize_str(&format!("custom:{s}")),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ContextInheritanceRule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "inherit" => Ok(Self::Inherit),
            "isolate" => Ok(Self::Isolate),
            "copy" => Ok(Self::Copy),
            "share" => Ok(Self::Share),
            s if s.starts_with("custom:") => Ok(Self::Custom(s[7..].to_string())),
            _ => Err(serde::de::Error::custom("Invalid inheritance rule")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ExecutionContext;
    use serde_json::json;
    #[tokio::test]
    async fn test_tool_execution_context_creation() {
        let base_context = ExecutionContext::new();
        let tool_context = ToolExecutionContext::new(base_context);

        assert!(!tool_context.context_id().is_empty());
        assert_eq!(tool_context.get_tool_data("nonexistent").await, None);
    }
    #[tokio::test]
    async fn test_tool_data_storage() {
        let base_context = ExecutionContext::new();
        let tool_context = ToolExecutionContext::new(base_context);

        tool_context
            .set_tool_data("test_key", json!({"value": 42}))
            .await;
        let retrieved = tool_context.get_tool_data("test_key").await;

        assert_eq!(retrieved, Some(json!({"value": 42})));
    }
    #[tokio::test]
    async fn test_shared_data_storage() {
        let base_context = ExecutionContext::new();
        let tool_context = ToolExecutionContext::new(base_context);

        tool_context
            .set_shared_data("shared_key", json!("shared_value"))
            .await;
        let retrieved = tool_context.get_shared_data("shared_key").await;

        assert_eq!(retrieved, Some(json!("shared_value")));
    }
    #[tokio::test]
    async fn test_inheritance_rules() {
        let base_context = ExecutionContext::new();
        let tool_context = ToolExecutionContext::new(base_context);

        tool_context
            .set_inheritance_rule("test_key", ContextInheritanceRule::Inherit)
            .await;
        let rule = tool_context.get_inheritance_rule("test_key").await;

        assert_eq!(rule, ContextInheritanceRule::Inherit);
    }
    #[tokio::test]
    async fn test_child_context_creation() {
        let base_context = ExecutionContext::new();
        let parent_context = ToolExecutionContext::new(base_context);

        parent_context
            .set_tool_data("parent_data", json!("parent_value"))
            .await;
        parent_context
            .set_inheritance_rule("parent_data", ContextInheritanceRule::Copy)
            .await;

        let child_context = parent_context.create_child_context("child").await.unwrap();

        assert!(child_context.context_id().contains("child"));
        assert_eq!(
            child_context.get_tool_data("parent_data").await,
            Some(json!("parent_value"))
        );
    }
    #[tokio::test]
    async fn test_execution_recording() {
        let base_context = ExecutionContext::new();
        let tool_context = ToolExecutionContext::new(base_context);

        tool_context
            .record_execution(
                "test_tool",
                json!({"param": "value"}),
                true,
                "success",
                Some(std::time::Duration::from_millis(100)),
            )
            .await;

        let history = tool_context.get_execution_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].tool_name, "test_tool");
        assert!(history[0].success);

        assert!(tool_context.has_executed_tool("test_tool").await);
        assert!(!tool_context.has_executed_tool("other_tool").await);
    }
    #[tokio::test]
    async fn test_context_manager() {
        let manager = ToolContextManager::new();
        let base_context = ExecutionContext::new();

        let _context = manager.create_context("test_context", base_context).await;
        assert_eq!(manager.context_count().await, 1);

        let retrieved = manager.get_context("test_context").await;
        assert!(retrieved.is_some());

        let contexts = manager.list_contexts().await;
        assert_eq!(contexts.len(), 1);
        assert!(contexts.contains(&"test_context".to_string()));

        assert!(manager.remove_context("test_context").await);
        assert_eq!(manager.context_count().await, 0);
    }
    #[tokio::test]
    async fn test_context_serialization() {
        let base_context = ExecutionContext::new();
        let tool_context = ToolExecutionContext::new(base_context);

        tool_context.set_tool_data("test", json!("value")).await;
        tool_context
            .set_inheritance_rule("test", ContextInheritanceRule::Inherit)
            .await;

        let state = tool_context.export_state().await;
        assert_eq!(state.tool_data.get("test"), Some(&json!("value")));
        assert_eq!(
            state.inheritance_rules.get("test"),
            Some(&ContextInheritanceRule::Inherit)
        );

        // Test serialization round-trip
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: ContextState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.tool_data.get("test"), Some(&json!("value")));
    }
}
