//! # Conditional Hook Pattern
//!
//! Advanced hook pattern that executes hooks based on dynamic conditions,
//! enabling sophisticated execution flow control and debug flow modification.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::{Hook, HookContext, HookMetadata, HookResult, Language, Priority};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Dynamic condition evaluation for conditional hooks
#[async_trait]
pub trait Condition: Send + Sync {
    /// Evaluate the condition against the current context
    async fn evaluate(&self, context: &HookContext) -> Result<bool>;

    /// Get a description of this condition
    fn description(&self) -> String;

    /// Get condition metadata for debugging
    fn metadata(&self) -> ConditionMetadata {
        ConditionMetadata {
            name: self.description(),
            condition_type: "custom".to_string(),
            parameters: Value::Null,
        }
    }
}

/// Metadata for condition debugging and introspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionMetadata {
    /// Human-readable name of the condition
    pub name: String,
    /// Type category of the condition
    pub condition_type: String,
    /// Configuration parameters for the condition
    pub parameters: Value,
}

/// Pre-built condition types for common use cases
#[derive(Debug, Clone)]
pub enum BuiltinCondition {
    /// Always evaluates to true
    Always,
    /// Always evaluates to false
    Never,
    /// Check if context contains specific metadata key
    HasMetadata(String),
    /// Check if metadata key equals specific value
    MetadataEquals(String, String),
    /// Check if hook point matches the specified point
    HookPointMatches(llmspell_hooks::HookPoint),
    /// Check if component type matches the specified type
    ComponentTypeMatches(llmspell_hooks::ComponentType),
    /// Check if debug mode is enabled
    DebugMode,
    /// Check if execution is in specific state
    ExecutionState(String),
    /// Check if an error occurred
    HasError,
    /// Custom JSON path condition with expected value
    JsonPath {
        /// JSON path to evaluate
        path: String,
        /// Expected value at the path
        expected: Value,
    },
}

#[async_trait]
impl Condition for BuiltinCondition {
    async fn evaluate(&self, context: &HookContext) -> Result<bool> {
        match self {
            BuiltinCondition::Always => Ok(true),
            BuiltinCondition::Never => Ok(false),
            BuiltinCondition::HasMetadata(key) => Ok(context.get_metadata(key).is_some()),
            BuiltinCondition::MetadataEquals(key, expected) => Ok(context
                .get_metadata(key)
                .is_some_and(|value| value == expected)),
            BuiltinCondition::HookPointMatches(expected_point) => {
                Ok(context.point == *expected_point)
            }
            BuiltinCondition::ComponentTypeMatches(expected_type) => {
                Ok(context.component_id.component_type == *expected_type)
            }
            BuiltinCondition::DebugMode => Ok(context
                .get_metadata("debug_mode")
                .is_some_and(|v| v == "true")),
            BuiltinCondition::ExecutionState(expected_state) => Ok(context
                .get_metadata("execution_state")
                .is_some_and(|state| state == expected_state)),
            BuiltinCondition::HasError => {
                Ok(context.get_metadata("error").is_some() || context.data.contains_key("error"))
            }
            BuiltinCondition::JsonPath { path, expected } => {
                // Simple JSON path evaluation
                if let Some(value) = Self::evaluate_json_path(context, path) {
                    Ok(value == *expected)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn description(&self) -> String {
        match self {
            BuiltinCondition::Always => "Always true".to_string(),
            BuiltinCondition::Never => "Always false".to_string(),
            BuiltinCondition::HasMetadata(key) => format!("Has metadata key '{key}'"),
            BuiltinCondition::MetadataEquals(key, value) => {
                format!("Metadata '{key}' equals '{value}'")
            }
            BuiltinCondition::HookPointMatches(point) => {
                format!("Hook point matches {point:?}")
            }
            BuiltinCondition::ComponentTypeMatches(comp_type) => {
                format!("Component type matches {comp_type:?}")
            }
            BuiltinCondition::DebugMode => "Debug mode enabled".to_string(),
            BuiltinCondition::ExecutionState(state) => {
                format!("Execution state is '{state}'")
            }
            BuiltinCondition::HasError => "Error is present".to_string(),
            BuiltinCondition::JsonPath { path, expected } => {
                format!("JSON path '{path}' equals {expected:?}")
            }
        }
    }
}

impl BuiltinCondition {
    /// Simple JSON path evaluation (supports basic dot notation)
    fn evaluate_json_path(context: &HookContext, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();

        // Start with the entire data as JSON value
        let mut current_value = serde_json::to_value(&context.data).ok()?;

        for part in parts {
            if let Some(value) = current_value.get(part) {
                current_value = value.clone();
            } else {
                return None;
            }
        }

        Some(current_value)
    }
}

/// Logical operators for combining conditions
pub enum LogicalCondition {
    /// Logical AND - both conditions must be true
    And(Box<dyn Condition>, Box<dyn Condition>),
    /// Logical OR - either condition must be true
    Or(Box<dyn Condition>, Box<dyn Condition>),
    /// Logical NOT - inverts the condition result
    Not(Box<dyn Condition>),
}

#[async_trait]
impl Condition for LogicalCondition {
    async fn evaluate(&self, context: &HookContext) -> Result<bool> {
        match self {
            LogicalCondition::And(left, right) => {
                Ok(left.evaluate(context).await? && right.evaluate(context).await?)
            }
            LogicalCondition::Or(left, right) => {
                Ok(left.evaluate(context).await? || right.evaluate(context).await?)
            }
            LogicalCondition::Not(condition) => Ok(!condition.evaluate(context).await?),
        }
    }

    fn description(&self) -> String {
        match self {
            LogicalCondition::And(left, right) => {
                format!("({}) AND ({})", left.description(), right.description())
            }
            LogicalCondition::Or(left, right) => {
                format!("({}) OR ({})", left.description(), right.description())
            }
            LogicalCondition::Not(condition) => {
                format!("NOT ({})", condition.description())
            }
        }
    }
}

/// Builder for creating complex conditions
pub struct ConditionBuilder {
    condition: Option<Box<dyn Condition>>,
}

impl ConditionBuilder {
    /// Create a new condition builder
    pub fn new() -> Self {
        Self { condition: None }
    }

    /// Set condition to always return true
    #[must_use]
    pub fn always(mut self) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::Always));
        self
    }

    /// Set condition to always return false
    #[must_use]
    pub fn never(mut self) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::Never));
        self
    }

    /// Check if context has specific metadata key
    #[must_use]
    pub fn has_metadata(mut self, key: String) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::HasMetadata(key)));
        self
    }

    /// Check if metadata key equals specific value
    #[must_use]
    pub fn metadata_equals(mut self, key: String, value: String) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::MetadataEquals(key, value)));
        self
    }

    /// Check if hook point matches specific point
    #[must_use]
    pub fn hook_point_matches(mut self, point: llmspell_hooks::HookPoint) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::HookPointMatches(point)));
        self
    }

    /// Check if debug mode is enabled
    #[must_use]
    pub fn debug_mode(mut self) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::DebugMode));
        self
    }

    /// Check if an error is present
    #[must_use]
    pub fn has_error(mut self) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::HasError));
        self
    }

    /// Check JSON path against expected value
    #[must_use]
    pub fn json_path(mut self, path: String, expected: Value) -> Self {
        self.condition = Some(Box::new(BuiltinCondition::JsonPath { path, expected }));
        self
    }

    /// Combine with another condition using logical AND
    #[must_use]
    pub fn and(mut self, other: Box<dyn Condition>) -> Self {
        if let Some(current) = self.condition.take() {
            self.condition = Some(Box::new(LogicalCondition::And(current, other)));
        } else {
            self.condition = Some(other);
        }
        self
    }

    /// Combine with another condition using logical OR
    #[must_use]
    pub fn or(mut self, other: Box<dyn Condition>) -> Self {
        if let Some(current) = self.condition.take() {
            self.condition = Some(Box::new(LogicalCondition::Or(current, other)));
        } else {
            self.condition = Some(other);
        }
        self
    }

    /// Negate the current condition
    #[must_use]
    pub fn negate(mut self) -> Self {
        if let Some(current) = self.condition.take() {
            self.condition = Some(Box::new(LogicalCondition::Not(current)));
        }
        self
    }

    /// Build the final condition
    ///
    /// # Errors
    ///
    /// Returns an error if no condition was specified during building.
    pub fn build(self) -> Result<Box<dyn Condition>> {
        self.condition
            .ok_or_else(|| anyhow::anyhow!("No condition specified"))
    }
}

impl Default for ConditionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Conditional hook that executes based on dynamic conditions
pub struct ConditionalHook {
    condition: Box<dyn Condition>,
    hook: Arc<dyn Hook>,
    metadata: HookMetadata,
    else_hook: Option<Arc<dyn Hook>>,
}

impl ConditionalHook {
    /// Create a new conditional hook
    pub fn new(name: &str, condition: Box<dyn Condition>, hook: Arc<dyn Hook>) -> Self {
        Self {
            condition,
            hook,
            else_hook: None,
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("Conditional hook execution".to_string()),
                priority: Priority::NORMAL,
                language: Language::Native,
                tags: vec!["conditional".to_string(), "dynamic".to_string()],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Add an else hook to execute when condition is false
    #[must_use]
    pub fn with_else_hook(mut self, else_hook: Arc<dyn Hook>) -> Self {
        self.else_hook = Some(else_hook);
        self
    }

    /// Set custom metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get the condition description
    pub fn condition_description(&self) -> String {
        self.condition.description()
    }
}

#[async_trait]
impl Hook for ConditionalHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        tracing::debug!(
            "ConditionalHook '{}': Evaluating condition: {}",
            self.metadata.name,
            self.condition.description()
        );

        let condition_result = self.condition.evaluate(context).await?;

        tracing::debug!(
            "ConditionalHook '{}': Condition result: {}",
            self.metadata.name,
            condition_result
        );

        if condition_result {
            // Condition is true, execute main hook
            tracing::debug!(
                "ConditionalHook '{}': Executing main hook",
                self.metadata.name
            );
            self.hook.execute(context).await
        } else if let Some(else_hook) = &self.else_hook {
            // Condition is false, execute else hook if present
            tracing::debug!(
                "ConditionalHook '{}': Executing else hook",
                self.metadata.name
            );
            else_hook.execute(context).await
        } else {
            // No else hook, continue
            tracing::debug!(
                "ConditionalHook '{}': Condition false, no else hook - continuing",
                self.metadata.name
            );
            Ok(HookResult::Continue)
        }
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        // Always should execute - the condition evaluation happens in execute()
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{ComponentId, ComponentType, FnHook, HookPoint};

    fn create_test_context() -> HookContext {
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        HookContext::new(HookPoint::SystemStartup, component_id)
    }

    #[tokio::test]
    async fn test_builtin_conditions() {
        let mut context = create_test_context();
        context.insert_metadata("test_key".to_string(), "test_value".to_string());

        // Test HasMetadata
        let condition = BuiltinCondition::HasMetadata("test_key".to_string());
        assert!(condition.evaluate(&context).await.unwrap());

        let condition = BuiltinCondition::HasMetadata("nonexistent".to_string());
        assert!(!condition.evaluate(&context).await.unwrap());

        // Test MetadataEquals
        let condition =
            BuiltinCondition::MetadataEquals("test_key".to_string(), "test_value".to_string());
        assert!(condition.evaluate(&context).await.unwrap());

        let condition =
            BuiltinCondition::MetadataEquals("test_key".to_string(), "wrong_value".to_string());
        assert!(!condition.evaluate(&context).await.unwrap());

        // Test Always/Never
        assert!(BuiltinCondition::Always.evaluate(&context).await.unwrap());
        assert!(!BuiltinCondition::Never.evaluate(&context).await.unwrap());
    }

    #[tokio::test]
    async fn test_logical_conditions() {
        let context = create_test_context();

        let true_condition = Box::new(BuiltinCondition::Always);
        let false_condition = Box::new(BuiltinCondition::Never);

        // Test AND
        let and_true = LogicalCondition::And(
            Box::new(BuiltinCondition::Always),
            Box::new(BuiltinCondition::Always),
        );
        assert!(and_true.evaluate(&context).await.unwrap());

        let and_false = LogicalCondition::And(true_condition, false_condition);
        assert!(!and_false.evaluate(&context).await.unwrap());

        // Test OR
        let or_true = LogicalCondition::Or(
            Box::new(BuiltinCondition::Always),
            Box::new(BuiltinCondition::Never),
        );
        assert!(or_true.evaluate(&context).await.unwrap());

        // Test NOT
        let not_false = LogicalCondition::Not(Box::new(BuiltinCondition::Always));
        assert!(!not_false.evaluate(&context).await.unwrap());
    }

    #[tokio::test]
    async fn test_condition_builder() {
        let context = create_test_context();

        let condition = ConditionBuilder::new()
            .always()
            .and(Box::new(BuiltinCondition::HasMetadata(
                "nonexistent".to_string(),
            )))
            .build()
            .unwrap();

        assert!(!condition.evaluate(&context).await.unwrap());

        let condition = ConditionBuilder::new()
            .never()
            .or(Box::new(BuiltinCondition::Always))
            .build()
            .unwrap();

        assert!(condition.evaluate(&context).await.unwrap());
    }

    #[tokio::test]
    async fn test_conditional_hook() {
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let main_hook = Arc::new(FnHook::new("main", |_ctx| {
            Ok(HookResult::Modified(
                serde_json::json!({"executed": "main"}),
            ))
        }));

        let else_hook = Arc::new(FnHook::new("else", |_ctx| {
            Ok(HookResult::Modified(
                serde_json::json!({"executed": "else"}),
            ))
        }));

        // Test with true condition
        let conditional = ConditionalHook::new(
            "test_conditional",
            Box::new(BuiltinCondition::Always),
            main_hook.clone(),
        );

        let result = conditional.execute(&mut context).await.unwrap();
        if let HookResult::Modified(data) = result {
            assert_eq!(data["executed"], "main");
        } else {
            panic!("Expected Modified result");
        }

        // Test with false condition and else hook
        let conditional = ConditionalHook::new(
            "test_conditional_else",
            Box::new(BuiltinCondition::Never),
            main_hook,
        )
        .with_else_hook(else_hook);

        let result = conditional.execute(&mut context).await.unwrap();
        if let HookResult::Modified(data) = result {
            assert_eq!(data["executed"], "else");
        } else {
            panic!("Expected Modified result");
        }

        // Test with false condition and no else hook
        let conditional = ConditionalHook::new(
            "test_conditional_no_else",
            Box::new(BuiltinCondition::Never),
            Arc::new(FnHook::new("never_executed", |_ctx| {
                Ok(HookResult::Cancel("Should not execute".to_string()))
            })),
        );

        let result = conditional.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }
}
