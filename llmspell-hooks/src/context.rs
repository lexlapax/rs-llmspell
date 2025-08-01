// ABOUTME: HookContext implementation providing comprehensive data for hook execution
// ABOUTME: Includes component info, metadata, language support, and correlation tracking

use crate::types::{ComponentId, HookPoint, Language};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

/// Comprehensive hook context with all necessary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// The hook point being executed
    pub point: HookPoint,

    /// Component that triggered the hook
    pub component_id: ComponentId,

    /// Dynamic data associated with the hook
    pub data: HashMap<String, JsonValue>,

    /// String metadata for lightweight info
    pub metadata: HashMap<String, String>,

    /// Language context for cross-language support
    pub language: Language,

    /// Correlation ID for tracking related operations
    pub correlation_id: Uuid,

    /// Timestamp when the hook was triggered
    pub timestamp: DateTime<Utc>,

    /// Operation context (if applicable)
    pub operation: Option<OperationContext>,

    /// Parent context for nested hooks
    pub parent_context: Option<Box<HookContext>>,
}

/// Operation-specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationContext {
    pub operation_type: String,
    pub operation_id: Uuid,
    pub parameters: JsonValue,
    pub result: Option<JsonValue>,
    pub error: Option<String>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(point: HookPoint, component_id: ComponentId) -> Self {
        Self {
            point,
            component_id,
            data: HashMap::new(),
            metadata: HashMap::new(),
            language: Language::Native,
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            operation: None,
            parent_context: None,
        }
    }

    /// Create with a specific language
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Create with a correlation ID
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = id;
        self
    }

    /// Add operation context
    pub fn with_operation(mut self, operation: OperationContext) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Create a child context
    pub fn child(&self, point: HookPoint) -> Self {
        let mut child = Self::new(point, self.component_id.clone());
        child.correlation_id = self.correlation_id;
        child.language = self.language;
        child.parent_context = Some(Box::new(self.clone()));
        child
    }

    /// Insert data into the context
    pub fn insert_data(&mut self, key: String, value: JsonValue) {
        self.data.insert(key, value);
    }

    /// Get data from the context
    pub fn get_data(&self, key: &str) -> Option<&JsonValue> {
        self.data.get(key)
    }

    /// Insert metadata
    pub fn insert_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Check if this is an error context
    pub fn is_error(&self) -> bool {
        matches!(
            self.point,
            HookPoint::AgentError
                | HookPoint::ToolError
                | HookPoint::WorkflowError
                | HookPoint::EventHandlerError
        )
    }

    /// Get the error message if this is an error context
    pub fn error_message(&self) -> Option<&str> {
        if self.is_error() {
            self.operation.as_ref()?.error.as_deref()
        } else {
            None
        }
    }

    /// Get operation parameters if available
    pub fn operation_parameters(&self) -> Option<&JsonValue> {
        self.operation.as_ref().map(|op| &op.parameters)
    }

    /// Set operation result
    pub fn set_operation_result(&mut self, result: JsonValue) {
        if let Some(ref mut op) = self.operation {
            op.result = Some(result);
        }
    }

    /// Get operation result if available
    pub fn operation_result(&self) -> Option<&JsonValue> {
        self.operation.as_ref()?.result.as_ref()
    }

    /// Create a minimal context for testing
    #[cfg(test)]
#[cfg_attr(test_category = "hook")]
    pub fn test_context() -> Self {
        use crate::types::ComponentType;

        Self::new(
            HookPoint::BeforeAgentExecution,
            ComponentId::new(ComponentType::Agent, "test-agent".to_string()),
        )
    }
}

/// Builder for creating HookContext with fluent API
pub struct HookContextBuilder {
    context: HookContext,
}

impl HookContextBuilder {
    pub fn new(point: HookPoint, component_id: ComponentId) -> Self {
        Self {
            context: HookContext::new(point, component_id),
        }
    }

    pub fn language(mut self, language: Language) -> Self {
        self.context.language = language;
        self
    }

    pub fn correlation_id(mut self, id: Uuid) -> Self {
        self.context.correlation_id = id;
        self
    }

    pub fn data(mut self, key: String, value: JsonValue) -> Self {
        self.context.data.insert(key, value);
        self
    }

    pub fn metadata(mut self, key: String, value: String) -> Self {
        self.context.metadata.insert(key, value);
        self
    }

    pub fn operation(mut self, operation: OperationContext) -> Self {
        self.context.operation = Some(operation);
        self
    }

    pub fn parent(mut self, parent: HookContext) -> Self {
        self.context.parent_context = Some(Box::new(parent));
        self
    }

    pub fn build(self) -> HookContext {
        self.context
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::types::ComponentType;
    use serde_json::json;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_context_creation() {
        let component_id = ComponentId::new(ComponentType::Agent, "test".to_string());
        let context = HookContext::new(HookPoint::BeforeAgentExecution, component_id.clone());

        assert_eq!(context.point, HookPoint::BeforeAgentExecution);
        assert_eq!(context.component_id, component_id);
        assert_eq!(context.language, Language::Native);
        assert!(context.data.is_empty());
        assert!(context.metadata.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_context_builder() {
        let component_id = ComponentId::new(ComponentType::Tool, "calculator".to_string());
        let correlation_id = Uuid::new_v4();

        let context = HookContextBuilder::new(HookPoint::BeforeToolExecution, component_id)
            .language(Language::Lua)
            .correlation_id(correlation_id)
            .data("input".to_string(), json!({"expression": "2+2"}))
            .metadata("tool_version".to_string(), "1.0.0".to_string())
            .build();

        assert_eq!(context.language, Language::Lua);
        assert_eq!(context.correlation_id, correlation_id);
        assert_eq!(
            context.get_data("input"),
            Some(&json!({"expression": "2+2"}))
        );
        assert_eq!(context.get_metadata("tool_version"), Some("1.0.0"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_child_context() {
        let component_id = ComponentId::new(ComponentType::Workflow, "pipeline".to_string());
        let parent = HookContext::new(HookPoint::BeforeWorkflowStart, component_id);
        let parent_correlation_id = parent.correlation_id;

        let child = parent.child(HookPoint::BeforeWorkflowStage);

        assert_eq!(child.correlation_id, parent_correlation_id);
        assert_eq!(child.language, parent.language);
        assert!(child.parent_context.is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_operation_context() {
        let component_id = ComponentId::new(ComponentType::Tool, "test".to_string());
        let mut context = HookContext::new(HookPoint::AfterToolExecution, component_id);

        let operation = OperationContext {
            operation_type: "calculate".to_string(),
            operation_id: Uuid::new_v4(),
            parameters: json!({"expression": "2+2"}),
            result: Some(json!({"value": 4})),
            error: None,
        };

        context.operation = Some(operation);

        assert_eq!(
            context.operation_parameters(),
            Some(&json!({"expression": "2+2"}))
        );
        assert_eq!(context.operation_result(), Some(&json!({"value": 4})));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_context() {
        let component_id = ComponentId::new(ComponentType::Agent, "test".to_string());
        let mut context = HookContext::new(HookPoint::AgentError, component_id);

        assert!(context.is_error());

        context.operation = Some(OperationContext {
            operation_type: "execute".to_string(),
            operation_id: Uuid::new_v4(),
            parameters: json!({}),
            result: None,
            error: Some("Test error".to_string()),
        });

        assert_eq!(context.error_message(), Some("Test error"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_context_serialization() {
        let component_id = ComponentId::new(ComponentType::System, "main".to_string());
        let context = HookContextBuilder::new(HookPoint::SystemStartup, component_id)
            .language(Language::JavaScript)
            .data("config".to_string(), json!({"debug": true}))
            .metadata("version".to_string(), "1.0.0".to_string())
            .build();

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: HookContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.point, HookPoint::SystemStartup);
        assert_eq!(deserialized.language, Language::JavaScript);
        assert_eq!(
            deserialized.get_data("config"),
            Some(&json!({"debug": true}))
        );
        assert_eq!(deserialized.get_metadata("version"), Some("1.0.0"));
    }
}
