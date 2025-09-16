//! ABOUTME: Session-specific hook context extensions and helper methods
//! ABOUTME: Provides utilities for enriching `HookContext` with comprehensive session metadata

use crate::sessions::{Session, SessionId};
use llmspell_hooks::{
    context::{HookContext, HookContextBuilder, OperationContext},
    types::{ComponentId, ComponentType, HookPoint},
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Extension trait for `HookContextBuilder` with session-specific methods
pub trait SessionHookContextExt {
    /// Add comprehensive session metadata to the context
    fn with_session_metadata(
        self,
        session: &Session,
    ) -> impl std::future::Future<Output = Self> + Send;

    /// Add session performance metrics to the context
    fn with_session_metrics(
        self,
        session: &Session,
    ) -> impl std::future::Future<Output = Self> + Send;

    /// Add artifact operation context
    #[must_use]
    fn with_artifact_operation(self, operation_type: &str, artifact_id: Option<&str>) -> Self;

    /// Add session timing information
    fn with_session_timing(
        self,
        session: &Session,
    ) -> impl std::future::Future<Output = Self> + Send;

    /// Add session state snapshot
    fn with_session_state(
        self,
        session: &Session,
    ) -> impl std::future::Future<Output = Self> + Send;
}

impl SessionHookContextExt for HookContextBuilder {
    async fn with_session_metadata(self, session: &Session) -> Self {
        let metadata = session.metadata.read().await;
        let session_id = metadata.id;

        let mut builder = self
            .data("session_id".to_string(), json!(session_id.to_string()))
            .data(
                "session_status".to_string(),
                json!(metadata.status.to_string()),
            )
            .data(
                "created_at".to_string(),
                json!(metadata.created_at.to_rfc3339()),
            )
            .data(
                "updated_at".to_string(),
                json!(metadata.updated_at.to_rfc3339()),
            )
            .metadata("session_status".to_string(), metadata.status.to_string())
            .metadata(
                "operation_count".to_string(),
                metadata.operation_count.to_string(),
            )
            .metadata(
                "artifact_count".to_string(),
                metadata.artifact_count.to_string(),
            );

        if let Some(ref name) = metadata.name {
            builder = builder.data("session_name".to_string(), json!(name));
        }

        if let Some(ref description) = metadata.description {
            builder = builder.data("session_description".to_string(), json!(description));
        }

        if let Some(ref created_by) = metadata.created_by {
            builder = builder.data("created_by".to_string(), json!(created_by));
        }

        if let Some(ref parent_id) = metadata.parent_session_id {
            builder = builder.data(
                "parent_session_id".to_string(),
                json!(parent_id.to_string()),
            );
        }

        if !metadata.tags.is_empty() {
            builder = builder.data("tags".to_string(), json!(metadata.tags));
        }

        if !metadata.custom_metadata.is_empty() {
            builder = builder.data(
                "custom_metadata".to_string(),
                json!(metadata.custom_metadata),
            );
        }

        builder
    }

    async fn with_session_metrics(self, session: &Session) -> Self {
        let metadata = session.metadata.read().await;

        let mut metrics = HashMap::new();
        metrics.insert("operation_count", json!(metadata.operation_count));
        metrics.insert("artifact_count", json!(metadata.artifact_count));
        metrics.insert("total_artifact_size", json!(metadata.total_artifact_size));

        if let Some(duration) = metadata.duration() {
            metrics.insert("session_duration_seconds", json!(duration.num_seconds()));
            metrics.insert("session_duration_ms", json!(duration.num_milliseconds()));
        }

        self.data("performance_metrics".to_string(), json!(metrics))
            .metadata(
                "total_artifact_size".to_string(),
                metadata.total_artifact_size.to_string(),
            )
    }

    fn with_artifact_operation(self, operation_type: &str, artifact_id: Option<&str>) -> Self {
        let operation = OperationContext {
            operation_type: operation_type.to_string(),
            operation_id: Uuid::new_v4(),
            parameters: json!({
                "artifact_id": artifact_id
            }),
            result: None,
            error: None,
        };

        self.operation(operation)
    }

    async fn with_session_timing(self, session: &Session) -> Self {
        let metadata = session.metadata.read().await;

        let mut timing_data = HashMap::new();
        timing_data.insert("created_at", json!(metadata.created_at.to_rfc3339()));
        timing_data.insert("updated_at", json!(metadata.updated_at.to_rfc3339()));

        if let Some(started_at) = metadata.started_at {
            timing_data.insert("started_at", json!(started_at.to_rfc3339()));
        }

        if let Some(ended_at) = metadata.ended_at {
            timing_data.insert("ended_at", json!(ended_at.to_rfc3339()));
        }

        self.data("timing".to_string(), json!(timing_data))
    }

    async fn with_session_state(self, session: &Session) -> Self {
        let state = session.get_all_state().await;

        if state.is_empty() {
            self
        } else {
            self.data("session_state".to_string(), json!(state))
        }
    }
}

/// Helper functions for creating session hook contexts
pub struct SessionHookContextHelper;

impl SessionHookContextHelper {
    /// Create a comprehensive session lifecycle context
    pub async fn create_lifecycle_context(
        hook_point: HookPoint,
        session: &Session,
        component_name: &str,
    ) -> HookContext {
        let _session_id = session.id().await;
        let component_id = ComponentId::new(
            ComponentType::Custom("SessionManager".to_string()),
            component_name.to_string(),
        );

        HookContextBuilder::new(hook_point, component_id)
            .with_session_metadata(session)
            .await
            .with_session_metrics(session)
            .await
            .with_session_timing(session)
            .await
            .build()
    }

    /// Create an artifact operation context
    pub async fn create_artifact_context(
        session: &Session,
        operation_type: &str,
        artifact_id: &str,
        component_name: &str,
    ) -> HookContext {
        let _session_id = session.id().await;
        let component_id = ComponentId::new(
            ComponentType::Custom("SessionManager".to_string()),
            component_name.to_string(),
        );

        HookContextBuilder::new(HookPoint::AfterToolExecution, component_id)
            .with_session_metadata(session)
            .await
            .with_artifact_operation(operation_type, Some(artifact_id))
            .data("artifact_id".to_string(), json!(artifact_id))
            .metadata("operation_type".to_string(), operation_type.to_string())
            .build()
    }

    /// Create a minimal session context for performance-critical paths
    pub fn create_minimal_context(
        hook_point: HookPoint,
        session_id: &SessionId,
        component_name: &str,
    ) -> HookContext {
        let component_id = ComponentId::new(
            ComponentType::Custom("SessionManager".to_string()),
            component_name.to_string(),
        );

        HookContextBuilder::new(hook_point, component_id)
            .data("session_id".to_string(), json!(session_id.to_string()))
            .metadata("context_type".to_string(), "minimal".to_string())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::types::CreateSessionOptions;
    use llmspell_hooks::types::HookPoint;
    #[tokio::test]
    async fn test_session_context_metadata_enrichment() {
        let options = CreateSessionOptions {
            name: Some("test-session".to_string()),
            description: Some("Test session for context extensions".to_string()),
            created_by: Some("test-user".to_string()),
            tags: vec!["test".to_string(), "context".to_string()],
            ..Default::default()
        };

        let session = Session::new(options);

        // Add some test data
        session
            .set_state("test_key".to_string(), json!("test_value"))
            .await
            .unwrap();
        session.increment_operation_count().await.unwrap();
        session.increment_artifact_count().await.unwrap();

        let component_id = ComponentId::new(
            ComponentType::Custom("Test".to_string()),
            "test-component".to_string(),
        );

        let context = HookContextBuilder::new(HookPoint::SessionStart, component_id)
            .with_session_metadata(&session)
            .await
            .with_session_metrics(&session)
            .await
            .with_session_timing(&session)
            .await
            .with_session_state(&session)
            .await
            .build();

        // Verify session data is present
        assert!(context.data.contains_key("session_id"));
        assert!(context.data.contains_key("session_name"));
        assert!(context.data.contains_key("session_description"));
        assert!(context.data.contains_key("created_by"));
        assert!(context.data.contains_key("tags"));
        assert!(context.data.contains_key("performance_metrics"));
        assert!(context.data.contains_key("timing"));
        assert!(context.data.contains_key("session_state"));

        // Verify metadata
        assert!(context.metadata.contains_key("session_status"));
        assert!(context.metadata.contains_key("operation_count"));
        assert!(context.metadata.contains_key("artifact_count"));

        // Verify specific values
        assert_eq!(
            context.data.get("session_name").unwrap(),
            &json!("test-session")
        );
        assert_eq!(context.data.get("created_by").unwrap(), &json!("test-user"));
    }
    #[tokio::test]
    async fn test_lifecycle_context_helper() {
        let session = Session::new(CreateSessionOptions::default());

        let context = SessionHookContextHelper::create_lifecycle_context(
            HookPoint::SessionStart,
            &session,
            "test-manager",
        )
        .await;

        assert_eq!(context.point, HookPoint::SessionStart);
        assert!(context.data.contains_key("session_id"));
        assert!(context.data.contains_key("performance_metrics"));
        assert!(context.data.contains_key("timing"));
    }
    #[tokio::test]
    async fn test_artifact_context_helper() {
        let session = Session::new(CreateSessionOptions::default());
        let artifact_id = "test-artifact-123";

        let context = SessionHookContextHelper::create_artifact_context(
            &session,
            "store_artifact",
            artifact_id,
            "test-manager",
        )
        .await;

        assert_eq!(context.point, HookPoint::AfterToolExecution);
        assert!(context.data.contains_key("artifact_id"));
        assert_eq!(
            context.data.get("artifact_id").unwrap(),
            &json!(artifact_id)
        );
        assert!(context.operation.is_some());

        let operation = context.operation.unwrap();
        assert_eq!(operation.operation_type, "store_artifact");
    }
    #[tokio::test]
    async fn test_minimal_context_helper() {
        let session = Session::new(CreateSessionOptions::default());
        let session_id = session.id().await;

        let context = SessionHookContextHelper::create_minimal_context(
            HookPoint::SessionSave,
            &session_id,
            "test-manager",
        );

        assert_eq!(context.point, HookPoint::SessionSave);
        assert!(context.data.contains_key("session_id"));
        assert_eq!(context.metadata.get("context_type").unwrap(), "minimal");

        // Should not have heavy metadata
        assert!(!context.data.contains_key("performance_metrics"));
        assert!(!context.data.contains_key("session_state"));
    }
}
