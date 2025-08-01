//! ABOUTME: Tool result collector for capturing tool execution outputs
//! ABOUTME: Automatically collects tool results as artifacts in sessions

use super::{is_size_acceptable, should_sample, ArtifactCollector, ArtifactData, CollectionConfig};
use crate::{Hook, HookContext, HookMetadata, HookPoint, HookResult};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;

/// Collector for tool execution results
pub struct ToolResultCollector {
    config: CollectionConfig,
}

impl ToolResultCollector {
    pub fn new() -> Self {
        Self::with_config(CollectionConfig::default())
    }

    pub fn with_config(config: CollectionConfig) -> Self {
        Self { config }
    }
}

impl Default for ToolResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for ToolResultCollector {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Only process AfterToolExecution hooks
        if context.point != HookPoint::AfterToolExecution {
            return Ok(HookResult::Continue);
        }

        // Check if we should collect
        if !self.should_collect(context).await {
            return Ok(HookResult::Continue);
        }

        // Extract artifact data
        match self.extract_artifact_data(context).await {
            Ok(artifact_data) => {
                // Add artifact data to context for session to process
                context.insert_data(
                    "collected_artifact".to_string(),
                    json!({
                        "type": "tool_result",
                        "data": artifact_data,
                    }),
                );

                // Add metadata
                context.insert_metadata("artifact_collected".to_string(), "true".to_string());
                context.insert_metadata("artifact_type".to_string(), "tool_result".to_string());

                Ok(HookResult::Continue)
            }
            Err(e) => {
                // Log error but don't fail the hook chain
                tracing::warn!("Failed to collect tool result artifact: {}", e);
                Ok(HookResult::Continue)
            }
        }
    }

    fn metadata(&self) -> HookMetadata {
        HookMetadata {
            name: "ToolResultCollector".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Collects tool execution results as artifacts".to_string()),
            priority: crate::Priority::NORMAL,
            language: crate::Language::Native,
            tags: vec!["collector".to_string(), "tool".to_string()],
        }
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        context.point == HookPoint::AfterToolExecution
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl ArtifactCollector for ToolResultCollector {
    async fn should_collect(&self, context: &HookContext) -> bool {
        // Check if there's an operation result
        let has_result = context
            .operation
            .as_ref()
            .and_then(|op| op.result.as_ref())
            .is_some();

        // Check if it's an error and we should collect errors
        let is_error = context
            .operation
            .as_ref()
            .and_then(|op| op.error.as_ref())
            .is_some();

        if is_error && !self.config.collect_errors {
            return false;
        }

        // Must have either result or error
        if !has_result && !is_error {
            return false;
        }

        // Check sampling
        if !should_sample(&self.config) {
            return false;
        }

        true
    }

    async fn extract_artifact_data(&self, context: &HookContext) -> Result<ArtifactData> {
        let operation = context
            .operation
            .as_ref()
            .ok_or_else(|| anyhow!("No operation context available"))?;

        // Get tool name from component
        let tool_name = &context.component_id.name;

        // Determine if this is an error result
        let is_error = operation.error.is_some();

        // Extract content
        let content = if let Some(result) = &operation.result {
            serde_json::to_vec_pretty(result)?
        } else if let Some(error) = &operation.error {
            json!({
                "error": error,
                "tool": tool_name,
                "timestamp": Utc::now().to_rfc3339(),
            })
            .to_string()
            .into_bytes()
        } else {
            return Err(anyhow!("No result or error to collect"));
        };

        // Check size limits
        if !is_size_acceptable(content.len(), &self.config) {
            return Err(anyhow!(
                "Content size {} is outside configured limits [{}, {}]",
                content.len(),
                self.config.min_size,
                self.config.max_size
            ));
        }

        // Generate artifact name
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let name = if is_error {
            format!("{}_error_{}.json", tool_name, timestamp)
        } else {
            format!("{}_result_{}.json", tool_name, timestamp)
        };

        // Build metadata
        let metadata = json!({
            "tool_name": tool_name,
            "operation_type": operation.operation_type,
            "operation_id": operation.operation_id.to_string(),
            "is_error": is_error,
            "collected_at": Utc::now().to_rfc3339(),
            "parameters": operation.parameters,
        });

        // Build tags
        let mut tags = self.config.auto_tags.clone();
        tags.push("tool_result".to_string());
        tags.push(tool_name.clone());
        if is_error {
            tags.push("error".to_string());
        }

        Ok(ArtifactData {
            name,
            content,
            mime_type: "application/json".to_string(),
            metadata,
            tags,
        })
    }

    fn artifact_type(&self) -> &str {
        "tool_result"
    }

    fn config(&self) -> &CollectionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::OperationContext;
    use crate::types::{ComponentId, ComponentType};
    use uuid::Uuid;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_tool_result_collection() {
        let mut config = CollectionConfig::default();
        config.min_size = 10; // Lower minimum for tests
        let collector = ToolResultCollector::with_config(config);
        let component_id = ComponentId::new(ComponentType::Tool, "calculator".to_string());
        let mut context = HookContext::new(HookPoint::AfterToolExecution, component_id);

        // Add operation with result
        context.operation = Some(OperationContext {
            operation_type: "calculate".to_string(),
            operation_id: Uuid::new_v4(),
            parameters: json!({"expression": "2+2"}),
            result: Some(json!({"value": 4})),
            error: None,
        });

        // Should collect
        assert!(collector.should_collect(&context).await);

        // Extract artifact data
        let artifact_data = collector.extract_artifact_data(&context).await.unwrap();
        assert!(artifact_data.name.contains("calculator_result_"));
        assert_eq!(artifact_data.mime_type, "application/json");
        assert!(artifact_data.tags.contains(&"tool_result".to_string()));

        // Execute hook
        let result = collector.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that artifact was added to context
        assert!(context.data.contains_key("collected_artifact"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_error_collection() {
        let collector = ToolResultCollector::new();
        let component_id = ComponentId::new(ComponentType::Tool, "failing_tool".to_string());
        let mut context = HookContext::new(HookPoint::AfterToolExecution, component_id);

        // Add operation with error
        context.operation = Some(OperationContext {
            operation_type: "process".to_string(),
            operation_id: Uuid::new_v4(),
            parameters: json!({"input": "bad data"}),
            result: None,
            error: Some("Invalid input format".to_string()),
        });

        // Should collect errors by default
        assert!(collector.should_collect(&context).await);

        // Extract artifact data
        let artifact_data = collector.extract_artifact_data(&context).await.unwrap();
        assert!(artifact_data.name.contains("failing_tool_error_"));
        assert!(artifact_data.tags.contains(&"error".to_string()));
    }
}
