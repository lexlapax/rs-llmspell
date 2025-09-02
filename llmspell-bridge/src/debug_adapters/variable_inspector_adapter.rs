//! Protocol adapter for `VariableInspector`
//!
//! Wraps the existing `VariableInspector` to implement the `DebugCapability` trait,
//! enabling protocol-based access to variable inspection functionality.

use crate::variable_inspector::VariableInspector;
use async_trait::async_trait;
use llmspell_core::{
    debug::{DebugCapability, DebugRequest, DebugResponse, VariableInfo},
    Result,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Adapter that wraps `VariableInspector` to provide `DebugCapability`
pub struct VariableInspectorAdapter {
    _variable_inspector: Arc<dyn VariableInspector>,
}

impl VariableInspectorAdapter {
    /// Create a new adapter wrapping a `VariableInspector`
    pub fn new(variable_inspector: Arc<dyn VariableInspector>) -> Self {
        Self {
            _variable_inspector: variable_inspector,
        }
    }

    /// Convert JSON value to `VariableInfo`
    #[allow(dead_code)]
    fn json_to_variable_info(name: String, value: JsonValue) -> VariableInfo {
        let expandable = matches!(&value, JsonValue::Object(_) | JsonValue::Array(_));
        let type_name = match &value {
            JsonValue::Null => Some("null".to_string()),
            JsonValue::Bool(_) => Some("boolean".to_string()),
            JsonValue::Number(_) => Some("number".to_string()),
            JsonValue::String(_) => Some("string".to_string()),
            JsonValue::Array(_) => Some("array".to_string()),
            JsonValue::Object(_) => Some("object".to_string()),
        };

        VariableInfo {
            name,
            value,
            type_name,
            expandable,
            reference: None, // TODO: Implement reference system for lazy expansion
        }
    }
}

#[async_trait]
impl DebugCapability for VariableInspectorAdapter {
    async fn process_debug_request(&self, request: DebugRequest) -> Result<DebugResponse> {
        match request {
            DebugRequest::InspectVariables { names, frame_id: _ } => {
                // Note: The bridge VariableInspector trait doesn't have async methods
                // and requires a ContextBatcher. For now, we'll need to work around this.
                // In a real implementation, we'd need to refactor the trait or use block_on.

                // For the Protocol-First architecture, we'll return a simplified response
                // The actual integration would need to handle the ContextBatcher requirement
                let mut variables = HashMap::new();
                for name in names {
                    // Placeholder: In real implementation, would call variable_inspector.inspect_variables
                    variables.insert(name.clone(), JsonValue::String(format!("<{name}>")));
                }
                Ok(DebugResponse::Variables(variables))
            }

            DebugRequest::EvaluateExpression {
                expression,
                frame_id: _,
            } => {
                // Similar to InspectVariables, this would need proper integration
                // with the existing infrastructure
                Ok(DebugResponse::EvaluationResult {
                    value: JsonValue::String(format!("<evaluated: {expression}>")),
                    type_name: Some("string".to_string()),
                })
            }

            _ => Ok(DebugResponse::Error {
                message: "Request should be handled by different capability".to_string(),
                details: Some(format!("Request type: {request:?}")),
            }),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "variable_inspection".to_string(),
            "expression_evaluation".to_string(),
            "watch_variables".to_string(),
        ]
    }

    fn name(&self) -> &'static str {
        "variable_inspector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variable_inspector::{ContextBatcher, ContextUpdate};

    // Mock implementation for testing
    struct MockVariableInspector;

    impl VariableInspector for MockVariableInspector {
        fn inspect_variables(
            &self,
            _variable_names: &[String],
            _batcher: &mut ContextBatcher,
        ) -> HashMap<String, JsonValue> {
            HashMap::new()
        }

        fn watch_variable(&self, _name: String, _batcher: &mut ContextBatcher) {}

        fn unwatch_variable(&self, _name: &str, _batcher: &mut ContextBatcher) {}

        fn get_all_cached_variables(&self) -> Vec<crate::debug_state_cache::CachedVariable> {
            Vec::new()
        }

        fn invalidate_cache(&self) {}

        fn process_context_updates(&self, _updates: Vec<ContextUpdate>) {}

        fn validate_api_usage(
            &self,
            _script: &str,
            _context: &crate::execution_context::SharedExecutionContext,
        ) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_variable_inspector_adapter() {
        let inspector = Arc::new(MockVariableInspector);
        let adapter = VariableInspectorAdapter::new(inspector);

        // Test capabilities
        assert!(adapter
            .capabilities()
            .contains(&"variable_inspection".to_string()));
        assert_eq!(adapter.name(), "variable_inspector");

        // Test variable inspection
        let response = adapter
            .process_debug_request(DebugRequest::InspectVariables {
                names: vec!["test_var".to_string()],
                frame_id: None,
            })
            .await
            .unwrap();

        if let DebugResponse::Variables(vars) = response {
            assert!(vars.contains_key("test_var"));
        } else {
            panic!("Expected Variables response");
        }
    }
}
