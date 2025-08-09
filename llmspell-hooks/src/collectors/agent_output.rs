//! ABOUTME: Agent output collector for capturing agent execution results  
//! ABOUTME: Automatically collects agent outputs as artifacts in sessions

use super::{is_size_acceptable, should_sample, ArtifactCollector, ArtifactData, CollectionConfig};
use crate::{Hook, HookContext, HookMetadata, HookPoint, HookResult};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;

/// Collector for agent execution outputs
pub struct AgentOutputCollector {
    config: CollectionConfig,
}

impl AgentOutputCollector {
    pub fn new() -> Self {
        Self::with_config(CollectionConfig::default())
    }

    pub fn with_config(config: CollectionConfig) -> Self {
        Self { config }
    }
}

impl Default for AgentOutputCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for AgentOutputCollector {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Only process AfterAgentExecution hooks
        if context.point != HookPoint::AfterAgentExecution {
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
                        "type": "agent_output",
                        "data": artifact_data,
                    }),
                );

                // Add metadata
                context.insert_metadata("artifact_collected".to_string(), "true".to_string());
                context.insert_metadata("artifact_type".to_string(), "agent_output".to_string());

                Ok(HookResult::Continue)
            }
            Err(e) => {
                // Log error but don't fail the hook chain
                tracing::warn!("Failed to collect agent output artifact: {}", e);
                Ok(HookResult::Continue)
            }
        }
    }

    fn metadata(&self) -> HookMetadata {
        HookMetadata {
            name: "AgentOutputCollector".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Collects agent execution outputs as artifacts".to_string()),
            priority: crate::Priority::NORMAL,
            language: crate::Language::Native,
            tags: vec!["collector".to_string(), "agent".to_string()],
        }
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        context.point == HookPoint::AfterAgentExecution
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl ArtifactCollector for AgentOutputCollector {
    async fn should_collect(&self, context: &HookContext) -> bool {
        // Check for response data (common pattern in agent outputs)
        let has_response = context.data.contains_key("response")
            || context.data.contains_key("output")
            || context.data.contains_key("result");

        // Check if it's an error
        let has_error = context.data.contains_key("error");

        if has_error && !self.config.collect_errors {
            return false;
        }

        // Must have some output
        if !has_response && !has_error {
            return false;
        }

        // Check sampling
        if !should_sample(&self.config) {
            return false;
        }

        true
    }

    async fn extract_artifact_data(&self, context: &HookContext) -> Result<ArtifactData> {
        let agent_name = &context.component_id.name;

        // Try to extract the main content from various possible fields
        let (content_value, is_error) = if let Some(response) = context.data.get("response") {
            (response.clone(), false)
        } else if let Some(output) = context.data.get("output") {
            (output.clone(), false)
        } else if let Some(result) = context.data.get("result") {
            (result.clone(), false)
        } else if let Some(error) = context.data.get("error") {
            (error.clone(), true)
        } else {
            return Err(anyhow!("No output data found in context"));
        };

        // Convert to bytes - handle both string and complex JSON
        let content = if let Some(text) = content_value.as_str() {
            text.as_bytes().to_vec()
        } else {
            serde_json::to_vec_pretty(&content_value)?
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

        // Determine MIME type based on content
        let mime_type = if content_value.is_string() {
            "text/plain".to_string()
        } else {
            "application/json".to_string()
        };

        // Generate artifact name
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let extension = if mime_type == "text/plain" {
            "txt"
        } else {
            "json"
        };
        let name = if is_error {
            format!("{}_error_{}.{}", agent_name, timestamp, extension)
        } else {
            format!("{}_output_{}.{}", agent_name, timestamp, extension)
        };

        // Build metadata
        let mut metadata = json!({
            "agent_name": agent_name,
            "is_error": is_error,
            "collected_at": Utc::now().to_rfc3339(),
            "correlation_id": context.correlation_id.to_string(),
        });

        // Add token usage if available
        if let Some(token_usage) = context.data.get("token_usage") {
            metadata["token_usage"] = token_usage.clone();
        }

        // Add model info if available
        if let Some(model) = context.metadata.get("model") {
            metadata["model"] = json!(model);
        }

        // Add provider info if available
        if let Some(provider) = context.metadata.get("provider") {
            metadata["provider"] = json!(provider);
        }

        // Build tags
        let mut tags = self.config.auto_tags.clone();
        tags.push("agent_output".to_string());
        tags.push(agent_name.clone());
        if is_error {
            tags.push("error".to_string());
        }

        // Add model as tag if available
        if let Some(model) = context.metadata.get("model") {
            tags.push(format!("model:{}", model));
        }

        Ok(ArtifactData {
            name,
            content,
            mime_type,
            metadata,
            tags,
        })
    }

    fn artifact_type(&self) -> &str {
        "agent_output"
    }

    fn config(&self) -> &CollectionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType};
    #[tokio::test]
    async fn test_agent_output_collection() {
        let config = CollectionConfig {
            min_size: 10, // Lower minimum for tests
            ..Default::default()
        };
        let collector = AgentOutputCollector::with_config(config);
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::AfterAgentExecution, component_id);

        // Add response data
        context.insert_data(
            "response".to_string(),
            json!({
                "message": "Hello, I'm an AI assistant. How can I help you?",
                "confidence": 0.95
            }),
        );

        // Add metadata
        context.insert_metadata("model".to_string(), "gpt-4".to_string());
        context.insert_metadata("provider".to_string(), "openai".to_string());

        // Should collect
        assert!(collector.should_collect(&context).await);

        // Extract artifact data
        let artifact_data = collector.extract_artifact_data(&context).await.unwrap();
        assert!(artifact_data.name.contains("test-agent_output_"));
        assert_eq!(artifact_data.mime_type, "application/json");
        assert!(artifact_data.tags.contains(&"agent_output".to_string()));
        assert!(artifact_data.tags.contains(&"model:gpt-4".to_string()));

        // Execute hook
        let result = collector.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that artifact was added to context
        assert!(context.data.contains_key("collected_artifact"));
    }
    #[tokio::test]
    async fn test_text_output_collection() {
        let config = CollectionConfig {
            min_size: 10, // Lower minimum for tests
            ..Default::default()
        };
        let collector = AgentOutputCollector::with_config(config);
        let component_id = ComponentId::new(ComponentType::Agent, "writer-agent".to_string());
        let mut context = HookContext::new(HookPoint::AfterAgentExecution, component_id);

        // Add plain text response
        context.insert_data(
            "output".to_string(),
            json!("This is a plain text response from the agent."),
        );

        // Should collect
        assert!(collector.should_collect(&context).await);

        // Extract artifact data
        let artifact_data = collector.extract_artifact_data(&context).await.unwrap();
        assert!(artifact_data.name.ends_with(".txt"));
        assert_eq!(artifact_data.mime_type, "text/plain");
    }
    #[tokio::test]
    async fn test_size_limits() {
        let config = CollectionConfig {
            min_size: 10,
            max_size: 100,
            ..Default::default()
        };

        let collector = AgentOutputCollector::with_config(config);
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::AfterAgentExecution, component_id);

        // Add response that's too small
        context.insert_data("response".to_string(), json!("tiny"));
        assert!(collector.should_collect(&context).await);
        let result = collector.extract_artifact_data(&context).await;
        assert!(result.is_err());

        // Add response that's within limits
        context.insert_data(
            "response".to_string(),
            json!("This is a reasonable size response"),
        );
        let result = collector.extract_artifact_data(&context).await;
        assert!(result.is_ok());
    }
}
