//! ABOUTME: Artifact-specific hook utilities and constants
//! ABOUTME: Provides artifact lifecycle hook points and helper functions

use crate::types::HookPoint;
use llmspell_core::events::{ArtifactEvent, ArtifactEventType};
use llmspell_core::state::ArtifactId;

/// Artifact-specific hook point constants
pub struct ArtifactHookPoints;

impl ArtifactHookPoints {
    /// Hook point triggered before an artifact is created
    pub const BEFORE_CREATE: &'static str = "artifact:before_create";

    /// Hook point triggered after an artifact is created
    pub const AFTER_CREATE: &'static str = "artifact:after_create";

    /// Hook point triggered before an artifact is modified
    pub const BEFORE_MODIFY: &'static str = "artifact:before_modify";

    /// Hook point triggered after an artifact is modified
    pub const AFTER_MODIFY: &'static str = "artifact:after_modify";

    /// Hook point triggered before an artifact is deleted
    pub const BEFORE_DELETE: &'static str = "artifact:before_delete";

    /// Hook point triggered after an artifact is deleted
    pub const AFTER_DELETE: &'static str = "artifact:after_delete";

    /// Hook point triggered before an artifact is validated
    pub const BEFORE_VALIDATE: &'static str = "artifact:before_validate";

    /// Hook point triggered after an artifact is validated
    pub const AFTER_VALIDATE: &'static str = "artifact:after_validate";

    /// Hook point triggered when validation fails
    pub const VALIDATION_FAILED: &'static str = "artifact:validation_failed";

    /// Hook point triggered when an artifact is derived from another
    pub const ARTIFACT_DERIVED: &'static str = "artifact:derived";

    /// Hook point triggered when an artifact is accessed
    pub const ARTIFACT_ACCESSED: &'static str = "artifact:accessed";

    /// Convert string to HookPoint
    pub fn to_hook_point(s: &str) -> HookPoint {
        HookPoint::Custom(s.to_string())
    }
}

/// Convert artifact event to hook point
pub fn event_to_hook_point(event: &ArtifactEvent) -> HookPoint {
    match &event.event_type {
        ArtifactEventType::Created(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::AFTER_CREATE)
        }
        ArtifactEventType::Modified(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::AFTER_MODIFY)
        }
        ArtifactEventType::Deleted(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::AFTER_DELETE)
        }
        ArtifactEventType::Validated(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::AFTER_VALIDATE)
        }
        ArtifactEventType::ValidationFailed(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::VALIDATION_FAILED)
        }
        ArtifactEventType::Derived(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::ARTIFACT_DERIVED)
        }
        ArtifactEventType::Accessed(_) => {
            ArtifactHookPoints::to_hook_point(ArtifactHookPoints::ARTIFACT_ACCESSED)
        }
        _ => HookPoint::Custom(format!("artifact:{}", event.event_name())),
    }
}

/// Add artifact information to hook context metadata
pub fn add_artifact_to_context(
    context: &mut crate::context::HookContext,
    artifact_id: &ArtifactId,
    operation: &str,
) {
    context.insert_metadata("artifact_id".to_string(), artifact_id.to_string());
    context.insert_metadata("artifact_operation".to_string(), operation.to_string());
}

/// Extract artifact information from hook context
pub fn get_artifact_from_context(context: &crate::context::HookContext) -> Option<ArtifactId> {
    context
        .metadata
        .get("artifact_id")
        .map(|id| ArtifactId::new(id.clone()))
}

/// Check if a hook point is artifact-related
pub fn is_artifact_hook_point(point: &HookPoint) -> bool {
    match point {
        HookPoint::Custom(s) => s.starts_with("artifact:"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::events::ArtifactEventBuilder;
    use llmspell_core::state::ArtifactMetadata;
    use llmspell_core::types::ComponentId;
    #[test]
    fn test_artifact_hook_points() {
        let point = ArtifactHookPoints::to_hook_point(ArtifactHookPoints::BEFORE_CREATE);
        assert!(is_artifact_hook_point(&point));

        let non_artifact_point = HookPoint::BeforeAgentInit;
        assert!(!is_artifact_hook_point(&non_artifact_point));
    }
    #[test]
    fn test_event_to_hook_point() {
        let component_id = ComponentId::new();
        let artifact_id = ArtifactId::new("test-artifact");
        let metadata = ArtifactMetadata::new(
            artifact_id.clone(),
            "test".to_string(),
            "test.txt".to_string(),
            component_id.clone(),
        );

        let event = ArtifactEventBuilder::new(component_id).created(metadata, None);

        let hook_point = event_to_hook_point(&event);
        assert!(is_artifact_hook_point(&hook_point));
    }
}
