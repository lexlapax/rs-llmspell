//! ABOUTME: Integration of artifact collectors with session management
//! ABOUTME: Processes collected artifacts from hook system and stores them in sessions

use crate::{
    artifact::{
        ArtifactMetadata, ArtifactStorage, ArtifactStorageOps, ArtifactType, SessionArtifact,
    },
    error::{Result, SessionError},
    SessionId,
};
use llmspell_hooks::{
    AgentOutputCollector, ArtifactData, CollectionConfig, HookContext, HookPoint, HookRegistry,
    ToolResultCollector,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Register artifact collectors with the hook registry
pub fn register_artifact_collectors(
    registry: &Arc<HookRegistry>,
    config: &CollectorConfig,
) -> Result<()> {
    // Register tool result collector
    if config.collect_tool_results {
        let tool_collector = ToolResultCollector::with_config(config.tool_config.clone());
        registry
            .register(HookPoint::AfterToolExecution, tool_collector)
            .map_err(|e| {
                SessionError::Configuration(format!("Failed to register tool collector: {e}"))
            })?;
        info!("Registered tool result collector");
    }

    // Register agent output collector
    if config.collect_agent_outputs {
        let agent_collector = AgentOutputCollector::with_config(config.agent_config.clone());
        registry
            .register(HookPoint::AfterAgentExecution, agent_collector)
            .map_err(|e| {
                SessionError::Configuration(format!("Failed to register agent collector: {e}"))
            })?;
        info!("Registered agent output collector");
    }

    Ok(())
}

/// Configuration for artifact collectors
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Whether to collect tool results
    pub collect_tool_results: bool,
    /// Configuration for tool result collection
    pub tool_config: CollectionConfig,
    /// Whether to collect agent outputs
    pub collect_agent_outputs: bool,
    /// Configuration for agent output collection
    pub agent_config: CollectionConfig,
    /// Whether to store collected artifacts automatically
    pub auto_store_artifacts: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            collect_tool_results: true,
            tool_config: CollectionConfig::default(),
            collect_agent_outputs: true,
            agent_config: CollectionConfig::default(),
            auto_store_artifacts: true,
        }
    }
}

/// Process collected artifacts from hook context
pub async fn process_collected_artifact(
    context: &HookContext,
    session_id: &SessionId,
    artifact_storage: &Arc<ArtifactStorage>,
) -> Result<()> {
    // Check if artifact was collected
    if let Some(collected_data) = context.data.get("collected_artifact") {
        let artifact_type_str = collected_data
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SessionError::Storage("Missing artifact type".to_string()))?;

        let artifact_data: ArtifactData = serde_json::from_value(
            collected_data
                .get("data")
                .ok_or_else(|| SessionError::Storage("Missing artifact data".to_string()))?
                .clone(),
        )?;

        // Map string type to ArtifactType enum
        let artifact_type = match artifact_type_str {
            "tool_result" => ArtifactType::ToolResult,
            "agent_output" => ArtifactType::AgentOutput,
            _ => ArtifactType::Custom(artifact_type_str.to_string()),
        };

        // Create artifact metadata
        let mut metadata = ArtifactMetadata::new(
            artifact_data.name.clone(),
            artifact_type,
            artifact_data.content.len(),
        );

        metadata.mime_type.clone_from(&artifact_data.mime_type);
        metadata.description = Some(format!("Auto-collected {artifact_type_str}"));

        // Add artifact-specific metadata
        if let Some(obj) = artifact_data.metadata.as_object() {
            for (key, value) in obj {
                metadata.custom.insert(key.clone(), value.clone());
            }
        }

        // Add tags
        for tag in &artifact_data.tags {
            let _ = metadata.add_tag(tag.clone());
        }

        // Get next sequence number for this session
        // For now, use a simple timestamp-based sequence
        let sequence = chrono::Utc::now().timestamp_micros().unsigned_abs();

        // Create the session artifact
        let session_artifact = SessionArtifact::create_with_metadata(
            *session_id,
            sequence,
            metadata.artifact_type.clone(),
            artifact_data.name,
            artifact_data.content,
            metadata.created_by.clone(),
        )?;

        // Store the artifact
        let artifact_id = artifact_storage
            .store_artifact(&session_artifact)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to store artifact: {e}")))?;

        debug!(
            "Stored collected artifact {} for session {}",
            artifact_id, session_id
        );
    }

    Ok(())
}

/// Artifact collection hook processor
///
/// This integrates with `SessionManager` to process hooks that collect artifacts
pub struct ArtifactCollectionProcessor {
    artifact_storage: Arc<ArtifactStorage>,
    config: CollectorConfig,
}

impl ArtifactCollectionProcessor {
    /// Create a new artifact collection processor
    pub fn new(artifact_storage: Arc<ArtifactStorage>, config: CollectorConfig) -> Self {
        Self {
            artifact_storage,
            config,
        }
    }

    /// Process hook context for artifact collection
    ///
    /// This is called after hooks execute to check if any artifacts were collected
    pub async fn process_hook_context(
        &self,
        context: &HookContext,
        session_id: &SessionId,
    ) -> Result<()> {
        if !self.config.auto_store_artifacts {
            return Ok(());
        }

        // Check if this hook collected any artifacts
        if context
            .metadata
            .get("artifact_collected")
            .map(std::string::String::as_str)
            == Some("true")
        {
            if let Err(e) =
                process_collected_artifact(context, session_id, &self.artifact_storage).await
            {
                warn!("Failed to process collected artifact: {}", e);
            }
        }

        Ok(())
    }

    /// Check if a hook point should trigger artifact collection
    pub fn should_process_hook_point(point: &HookPoint) -> bool {
        matches!(
            point,
            HookPoint::AfterToolExecution | HookPoint::AfterAgentExecution
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{ComponentId, ComponentType};
    use llmspell_storage::MemoryBackend;
    use serde_json::json;
    #[tokio::test]
    async fn test_process_collected_artifact() {
        let storage = Arc::new(MemoryBackend::new());
        let artifact_storage = Arc::new(ArtifactStorage::with_backend(storage));
        let session_id = SessionId::new();

        let component_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());
        let mut context = HookContext::new(HookPoint::AfterToolExecution, component_id);

        // Add collected artifact data
        context.insert_data(
            "collected_artifact".to_string(),
            json!({
                "type": "tool_result",
                "data": {
                    "name": "test_result.json",
                    "content": b"test content",
                    "mime_type": "application/json",
                    "metadata": {
                        "tool_name": "test-tool",
                        "operation_type": "test"
                    },
                    "tags": ["test", "tool_result"]
                }
            }),
        );

        // Process the collected artifact
        let result = process_collected_artifact(&context, &session_id, &artifact_storage).await;
        assert!(result.is_ok());

        // Verify artifact was stored
        let artifacts = artifact_storage
            .list_session_artifacts(&session_id)
            .await
            .unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].name, "test_result.json");
    }
    #[tokio::test]
    async fn test_artifact_collection_processor() {
        let storage = Arc::new(MemoryBackend::new());
        let artifact_storage = Arc::new(ArtifactStorage::with_backend(storage));
        let config = CollectorConfig::default();

        let processor = ArtifactCollectionProcessor::new(artifact_storage, config);
        let session_id = SessionId::new();

        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::AfterAgentExecution, component_id);

        // Mark as artifact collected
        context.insert_metadata("artifact_collected".to_string(), "true".to_string());
        context.insert_data(
            "collected_artifact".to_string(),
            json!({
                "type": "agent_output",
                "data": {
                    "name": "agent_response.txt",
                    "content": b"Agent response text",
                    "mime_type": "text/plain",
                    "metadata": {
                        "agent_name": "test-agent"
                    },
                    "tags": ["agent_output"]
                }
            }),
        );

        // Process the context
        let result = processor.process_hook_context(&context, &session_id).await;
        assert!(result.is_ok());
    }
    #[test]
    fn test_should_process_hook_point() {
        assert!(ArtifactCollectionProcessor::should_process_hook_point(
            &HookPoint::AfterToolExecution
        ));
        assert!(ArtifactCollectionProcessor::should_process_hook_point(
            &HookPoint::AfterAgentExecution
        ));
        assert!(!ArtifactCollectionProcessor::should_process_hook_point(
            &HookPoint::BeforeToolExecution
        ));
    }
}
