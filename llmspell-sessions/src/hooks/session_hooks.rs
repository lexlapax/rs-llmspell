//! ABOUTME: Session-specific hooks implementing `ReplayableHook` trait for session lifecycle events
//! ABOUTME: Provides replayable hooks for session start, end, checkpoint, restore, and save operations

use anyhow::Result;
use async_trait::async_trait;
use llmspell_hooks::{
    context::HookContext,
    replay::ReplayManager,
    result::HookResult,
    traits::{Hook, ReplayableHook},
    types::{HookMetadata, Language, Priority},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::any::Any;
use std::sync::Arc;
use tracing::{debug, info};

/// Hook for session start events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartHook {
    /// Hook metadata
    metadata: HookMetadata,
    /// Whether to validate session configuration
    validate_config: bool,
    /// Whether to log session start
    log_start: bool,
}

impl Default for SessionStartHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStartHook {
    /// Create a new session start hook
    pub fn new() -> Self {
        Self {
            metadata: HookMetadata {
                name: "session_start".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Hook for session start events".to_string()),
                tags: vec!["session".to_string(), "lifecycle".to_string()],
                priority: Priority(100),
                language: Language::Native,
            },
            validate_config: true,
            log_start: true,
        }
    }

    /// Set whether to validate configuration
    #[must_use]
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_config = validate;
        self
    }

    /// Set whether to log session start
    #[must_use]
    pub fn with_logging(mut self, log: bool) -> Self {
        self.log_start = log;
        self
    }
}

#[async_trait]
impl Hook for SessionStartHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if self.log_start {
            if let Some(session_id) = context.get_data("session_id") {
                info!("Session starting: {}", session_id);
            }
        }

        // Validate session configuration if enabled
        if self.validate_config {
            if let Some(config) = context.get_data("session_config") {
                debug!("Validating session configuration: {:?}", config);
                // Add validation logic here
            }
        }

        // Add session start timestamp
        context.insert_data(
            "start_timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ReplayableHook for SessionStartHook {
    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }

    fn is_replayable(&self) -> bool {
        true
    }
}

/// Hook for session end events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndHook {
    /// Hook metadata
    metadata: HookMetadata,
    /// Whether to calculate session duration
    calculate_duration: bool,
    /// Whether to cleanup temporary resources
    cleanup_resources: bool,
}

impl Default for SessionEndHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionEndHook {
    /// Create a new session end hook
    pub fn new() -> Self {
        Self {
            metadata: HookMetadata {
                name: "session_end".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Hook for session end events".to_string()),
                tags: vec!["session".to_string(), "lifecycle".to_string()],
                priority: Priority(100),
                language: Language::Native,
            },
            calculate_duration: true,
            cleanup_resources: true,
        }
    }

    /// Set whether to calculate session duration
    #[must_use]
    pub fn with_duration_calculation(mut self, calculate: bool) -> Self {
        self.calculate_duration = calculate;
        self
    }

    /// Set whether to cleanup resources
    #[must_use]
    pub fn with_cleanup(mut self, cleanup: bool) -> Self {
        self.cleanup_resources = cleanup;
        self
    }
}

#[async_trait]
impl Hook for SessionEndHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if let Some(session_id) = context.get_data("session_id") {
            info!("Session ending: {}", session_id);
        }

        // Calculate session duration if enabled
        if self.calculate_duration {
            if let Some(created_at) = context.get_data("created_at") {
                if let Ok(created) =
                    chrono::DateTime::parse_from_rfc3339(created_at.as_str().unwrap_or_default())
                {
                    let duration = chrono::Utc::now() - created.with_timezone(&chrono::Utc);
                    context.insert_data(
                        "session_duration_ms".to_string(),
                        json!(duration.num_milliseconds()),
                    );
                }
            }
        }

        // Add session end timestamp
        context.insert_data(
            "end_timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ReplayableHook for SessionEndHook {
    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }

    fn is_replayable(&self) -> bool {
        true
    }
}

/// Hook for session checkpoint events (suspend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpointHook {
    /// Hook metadata
    metadata: HookMetadata,
    /// Whether to capture state snapshot
    capture_state: bool,
    /// Whether to validate checkpoint
    validate_checkpoint: bool,
}

impl Default for SessionCheckpointHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionCheckpointHook {
    /// Create a new session checkpoint hook
    pub fn new() -> Self {
        Self {
            metadata: HookMetadata {
                name: "session_checkpoint".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Hook for session checkpoint events".to_string()),
                tags: vec![
                    "session".to_string(),
                    "lifecycle".to_string(),
                    "checkpoint".to_string(),
                ],
                priority: Priority(100),
                language: Language::Native,
            },
            capture_state: true,
            validate_checkpoint: true,
        }
    }

    /// Set whether to capture state snapshot
    #[must_use]
    pub fn with_state_capture(mut self, capture: bool) -> Self {
        self.capture_state = capture;
        self
    }

    /// Set whether to validate checkpoint
    #[must_use]
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_checkpoint = validate;
        self
    }
}

#[async_trait]
impl Hook for SessionCheckpointHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if let Some(session_id) = context.get_data("session_id") {
            info!("Creating checkpoint for session: {}", session_id);
        }

        // Capture state snapshot if enabled
        if self.capture_state {
            if let Some(state) = context.get_data("session_state") {
                context.insert_data("checkpoint_state".to_string(), state.clone());
            }
        }

        // Add checkpoint metadata
        context.insert_data(
            "checkpoint_timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        if let Some(action) = context.get_data("action") {
            context.insert_metadata(
                "checkpoint_action".to_string(),
                action.as_str().unwrap_or("unknown").to_string(),
            );
        }

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ReplayableHook for SessionCheckpointHook {
    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }

    fn is_replayable(&self) -> bool {
        true
    }
}

/// Hook for session restore events (resume)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRestoreHook {
    /// Hook metadata
    metadata: HookMetadata,
    /// Whether to validate restored state
    validate_state: bool,
    /// Whether to restore metadata
    restore_metadata: bool,
}

impl Default for SessionRestoreHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRestoreHook {
    /// Create a new session restore hook
    pub fn new() -> Self {
        Self {
            metadata: HookMetadata {
                name: "session_restore".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Hook for session restore events".to_string()),
                tags: vec![
                    "session".to_string(),
                    "lifecycle".to_string(),
                    "restore".to_string(),
                ],
                priority: Priority(100),
                language: Language::Native,
            },
            validate_state: true,
            restore_metadata: true,
        }
    }

    /// Set whether to validate restored state
    #[must_use]
    pub fn with_state_validation(mut self, validate: bool) -> Self {
        self.validate_state = validate;
        self
    }

    /// Set whether to restore metadata
    #[must_use]
    pub fn with_metadata_restore(mut self, restore: bool) -> Self {
        self.restore_metadata = restore;
        self
    }
}

#[async_trait]
impl Hook for SessionRestoreHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if let Some(session_id) = context.get_data("session_id") {
            info!("Restoring session: {}", session_id);
        }

        // Validate restored state if enabled
        if self.validate_state {
            if let Some(state) = context.get_data("session_state") {
                debug!("Validating restored state: {:?}", state);
                // Add validation logic here
            }
        }

        // Add restore metadata
        context.insert_data(
            "restore_timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        if let Some(action) = context.get_data("action") {
            context.insert_metadata(
                "restore_action".to_string(),
                action.as_str().unwrap_or("unknown").to_string(),
            );
        }

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ReplayableHook for SessionRestoreHook {
    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }

    fn is_replayable(&self) -> bool {
        true
    }
}

/// Hook for session save events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSaveHook {
    /// Hook metadata
    metadata: HookMetadata,
    /// Whether to compress session data
    compress_data: bool,
    /// Whether to calculate checksums
    calculate_checksums: bool,
}

impl Default for SessionSaveHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionSaveHook {
    /// Create a new session save hook
    pub fn new() -> Self {
        Self {
            metadata: HookMetadata {
                name: "session_save".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Hook for session save events".to_string()),
                tags: vec![
                    "session".to_string(),
                    "lifecycle".to_string(),
                    "persistence".to_string(),
                ],
                priority: Priority(100),
                language: Language::Native,
            },
            compress_data: true,
            calculate_checksums: true,
        }
    }

    /// Set whether to compress data
    #[must_use]
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress_data = compress;
        self
    }

    /// Set whether to calculate checksums
    #[must_use]
    pub fn with_checksums(mut self, checksums: bool) -> Self {
        self.calculate_checksums = checksums;
        self
    }
}

#[async_trait]
impl Hook for SessionSaveHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if let Some(session_id) = context.get_data("session_id") {
            info!("Saving session: {}", session_id);
        }

        // Add save metadata
        context.insert_data(
            "save_timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        // Add compression flag
        if self.compress_data {
            context.insert_metadata("compression_enabled".to_string(), "true".to_string());
        }

        // Add checksum flag
        if self.calculate_checksums {
            context.insert_metadata("checksums_enabled".to_string(), "true".to_string());
        }

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ReplayableHook for SessionSaveHook {
    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }

    fn is_replayable(&self) -> bool {
        true
    }
}

/// Registry of session hooks for easy registration
pub struct SessionHookRegistry;

impl SessionHookRegistry {
    /// Register all session hooks with a `ReplayManager`
    pub fn register_with_replay_manager(replay_manager: &ReplayManager) -> Result<()> {
        // Register session start hook
        let start_hook = SessionStartHook::new();
        replay_manager.register_hook(start_hook.replay_id(), Arc::new(start_hook));

        // Register session end hook
        let end_hook = SessionEndHook::new();
        replay_manager.register_hook(end_hook.replay_id(), Arc::new(end_hook));

        // Register checkpoint hook
        let checkpoint_hook = SessionCheckpointHook::new();
        replay_manager.register_hook(checkpoint_hook.replay_id(), Arc::new(checkpoint_hook));

        // Register restore hook
        let restore_hook = SessionRestoreHook::new();
        replay_manager.register_hook(restore_hook.replay_id(), Arc::new(restore_hook));

        // Register save hook
        let save_hook = SessionSaveHook::new();
        replay_manager.register_hook(save_hook.replay_id(), Arc::new(save_hook));

        info!("Registered all session hooks with replay manager");
        Ok(())
    }

    /// Create a default set of session hooks
    pub fn default_hooks() -> Vec<Box<dyn ReplayableHook>> {
        vec![
            Box::new(SessionStartHook::new()),
            Box::new(SessionEndHook::new()),
            Box::new(SessionCheckpointHook::new()),
            Box::new(SessionRestoreHook::new()),
            Box::new(SessionSaveHook::new()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::{
        context::HookContextBuilder,
        types::{ComponentId, ComponentType, HookPoint},
    };

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_start_hook() {
        let hook = SessionStartHook::new();
        let component_id = ComponentId::new(
            ComponentType::Custom("SessionManager".to_string()),
            "test-manager".to_string(),
        );

        let mut context = HookContextBuilder::new(HookPoint::SessionStart, component_id)
            .data("session_id".to_string(), json!("test-session-123"))
            .data("session_config".to_string(), json!({"auto_save": true}))
            .build();

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert!(context.data.contains_key("start_timestamp"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_end_hook() {
        let hook = SessionEndHook::new();
        let component_id = ComponentId::new(
            ComponentType::Custom("SessionManager".to_string()),
            "test-manager".to_string(),
        );

        let created_at = chrono::Utc::now() - chrono::Duration::hours(1);
        let mut context = HookContextBuilder::new(HookPoint::SessionEnd, component_id)
            .data("session_id".to_string(), json!("test-session-123"))
            .data("created_at".to_string(), json!(created_at.to_rfc3339()))
            .build();

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert!(context.data.contains_key("end_timestamp"));
        assert!(context.data.contains_key("session_duration_ms"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_replay_id() {
        let start_hook = SessionStartHook::new();
        assert_eq!(start_hook.replay_id(), "session_start:1.0.0");

        let end_hook = SessionEndHook::new();
        assert_eq!(end_hook.replay_id(), "session_end:1.0.0");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_hook_serialization() {
        let hook = SessionCheckpointHook::new();
        let component_id = ComponentId::new(
            ComponentType::Custom("SessionManager".to_string()),
            "test-manager".to_string(),
        );

        let context = HookContextBuilder::new(HookPoint::SessionCheckpoint, component_id)
            .data("session_id".to_string(), json!("test-session-123"))
            .data("session_state".to_string(), json!({"key": "value"}))
            .build();

        // Test context serialization
        let serialized = hook.serialize_context(&context).unwrap();
        let deserialized = hook.deserialize_context(&serialized).unwrap();

        assert_eq!(deserialized.point, context.point);
        assert_eq!(
            deserialized.data.get("session_id"),
            context.data.get("session_id")
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_default_hooks_creation() {
        let hooks = SessionHookRegistry::default_hooks();
        assert_eq!(hooks.len(), 5);

        // Verify hook types
        let hook_names: Vec<String> = hooks.iter().map(|h| h.metadata().name).collect();

        assert!(hook_names.contains(&"session_start".to_string()));
        assert!(hook_names.contains(&"session_end".to_string()));
        assert!(hook_names.contains(&"session_checkpoint".to_string()));
        assert!(hook_names.contains(&"session_restore".to_string()));
        assert!(hook_names.contains(&"session_save".to_string()));
    }
}
