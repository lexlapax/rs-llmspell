//! ABOUTME: Event types and structures for artifact lifecycle and operations
//! ABOUTME: Provides events for artifact creation, modification, deletion, and correlation

use crate::state::artifact_correlation::{ArtifactId, ArtifactMetadata, ArtifactRelationship};
use crate::types::{ComponentId, EventMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Base event type for all artifact-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEvent {
    /// Event metadata (ID, timestamp, etc.)
    pub metadata: EventMetadata,

    /// Type of artifact event
    pub event_type: ArtifactEventType,

    /// Component that triggered the event
    pub source: ComponentId,

    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

/// Types of artifact events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactEventType {
    /// Artifact was created
    Created(ArtifactCreatedEvent),

    /// Artifact was modified
    Modified(ArtifactModifiedEvent),

    /// Artifact was deleted
    Deleted(ArtifactDeletedEvent),

    /// Artifact was accessed/read
    Accessed(ArtifactAccessedEvent),

    /// Artifact was derived from another
    Derived(ArtifactDerivedEvent),

    /// Artifact was versioned
    Versioned(ArtifactVersionedEvent),

    /// Artifact metadata was updated
    MetadataUpdated(ArtifactMetadataUpdatedEvent),

    /// Artifact was validated
    Validated(ArtifactValidatedEvent),

    /// Artifact failed validation
    ValidationFailed(ArtifactValidationFailedEvent),
}

/// Event emitted when an artifact is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCreatedEvent {
    /// The artifact that was created
    pub artifact: ArtifactMetadata,

    /// Location where artifact is stored (if applicable)
    pub storage_location: Option<StorageLocation>,

    /// Initial content hash (for integrity)
    pub content_hash: Option<String>,
}

/// Storage location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageLocation {
    /// File system path
    FileSystem(PathBuf),

    /// Memory storage ID
    Memory(String),

    /// Database reference
    Database { table: String, id: String },

    /// External storage (S3, etc.)
    External { provider: String, reference: String },
}

/// Event emitted when an artifact is modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactModifiedEvent {
    /// ID of the modified artifact
    pub artifact_id: ArtifactId,

    /// What was modified
    pub modifications: Vec<Modification>,

    /// Previous content hash
    pub previous_hash: Option<String>,

    /// New content hash
    pub new_hash: Option<String>,

    /// Size change in bytes
    pub size_delta: i64,
}

/// Types of modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modification {
    /// Content was changed
    Content,

    /// Metadata was updated
    Metadata { fields: Vec<String> },

    /// Tags were modified
    Tags {
        added: Vec<String>,
        removed: Vec<String>,
    },

    /// Permissions changed
    Permissions,

    /// Location changed
    Location {
        from: StorageLocation,
        to: StorageLocation,
    },
}

/// Event emitted when an artifact is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDeletedEvent {
    /// ID of the deleted artifact
    pub artifact_id: ArtifactId,

    /// Metadata of the deleted artifact (for recovery)
    pub artifact_metadata: ArtifactMetadata,

    /// Whether deletion is permanent
    pub permanent: bool,

    /// Backup location if soft-deleted
    pub backup_location: Option<StorageLocation>,
}

/// Event emitted when an artifact is accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactAccessedEvent {
    /// ID of the accessed artifact
    pub artifact_id: ArtifactId,

    /// Type of access
    pub access_type: AccessType,

    /// Purpose of access
    pub purpose: Option<String>,
}

/// Types of artifact access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessType {
    /// Artifact was read
    Read,

    /// Artifact was streamed
    Stream,

    /// Artifact metadata was queried
    MetadataQuery,

    /// Artifact was exported
    Export { format: String },
}

/// Event emitted when an artifact is derived from another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDerivedEvent {
    /// The new derived artifact
    pub derived_artifact: ArtifactMetadata,

    /// Parent artifact ID
    pub parent_id: ArtifactId,

    /// Type of derivation
    pub derivation_type: DerivationType,

    /// Relationship to parent
    pub relationship: ArtifactRelationship,
}

/// Types of artifact derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DerivationType {
    /// Transformed from parent
    Transformation { operation: String },

    /// Extracted from parent
    Extraction { part: String },

    /// Combined with other artifacts
    Combination { other_artifacts: Vec<ArtifactId> },

    /// Generated based on parent
    Generation { method: String },
}

/// Event emitted when an artifact is versioned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactVersionedEvent {
    /// Original artifact ID
    pub artifact_id: ArtifactId,

    /// New version artifact
    pub new_version: ArtifactMetadata,

    /// Version number or tag
    pub version: String,

    /// Changes in this version
    pub changes: Vec<String>,
}

/// Event emitted when artifact metadata is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadataUpdatedEvent {
    /// Artifact ID
    pub artifact_id: ArtifactId,

    /// Updated fields and their new values
    pub updates: HashMap<String, serde_json::Value>,

    /// Previous values (for rollback)
    pub previous_values: HashMap<String, serde_json::Value>,
}

/// Event emitted when an artifact is validated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactValidatedEvent {
    /// Artifact ID
    pub artifact_id: ArtifactId,

    /// Validation type
    pub validation_type: ValidationType,

    /// Validation results
    pub results: ValidationResults,
}

/// Event emitted when artifact validation fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactValidationFailedEvent {
    /// Artifact ID
    pub artifact_id: ArtifactId,

    /// Validation type
    pub validation_type: ValidationType,

    /// Failure reasons
    pub failures: Vec<ValidationFailure>,
}

/// Types of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// Schema validation
    Schema { schema_name: String },

    /// Content validation
    Content,

    /// Security validation
    Security,

    /// Integrity check
    Integrity,

    /// Custom validation
    Custom { validator: String },
}

/// Validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Whether validation passed
    pub valid: bool,

    /// Validation score (0-100)
    pub score: Option<u8>,

    /// Warnings that don't block validity
    pub warnings: Vec<String>,

    /// Additional validation data
    pub data: HashMap<String, serde_json::Value>,
}

/// Validation failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailure {
    /// What failed
    pub field: Option<String>,

    /// Why it failed
    pub reason: String,

    /// Expected value/format
    pub expected: Option<String>,

    /// Actual value/format
    pub actual: Option<String>,
}

impl ArtifactEvent {
    /// Create a new artifact event
    pub fn new(event_type: ArtifactEventType, source: ComponentId) -> Self {
        Self {
            metadata: EventMetadata::new(),
            event_type,
            source,
            context: HashMap::new(),
        }
    }

    /// Add context to the event
    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Get event name for routing/filtering
    pub fn event_name(&self) -> &'static str {
        match &self.event_type {
            ArtifactEventType::Created(_) => "artifact.created",
            ArtifactEventType::Modified(_) => "artifact.modified",
            ArtifactEventType::Deleted(_) => "artifact.deleted",
            ArtifactEventType::Accessed(_) => "artifact.accessed",
            ArtifactEventType::Derived(_) => "artifact.derived",
            ArtifactEventType::Versioned(_) => "artifact.versioned",
            ArtifactEventType::MetadataUpdated(_) => "artifact.metadata_updated",
            ArtifactEventType::Validated(_) => "artifact.validated",
            ArtifactEventType::ValidationFailed(_) => "artifact.validation_failed",
        }
    }
}

/// Builder for artifact events
pub struct ArtifactEventBuilder {
    source: ComponentId,
    context: HashMap<String, serde_json::Value>,
}

impl ArtifactEventBuilder {
    /// Create a new event builder
    pub fn new(source: ComponentId) -> Self {
        Self {
            source,
            context: HashMap::new(),
        }
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Build a created event
    pub fn created(
        self,
        artifact: ArtifactMetadata,
        location: Option<StorageLocation>,
    ) -> ArtifactEvent {
        ArtifactEvent {
            metadata: EventMetadata::new(),
            event_type: ArtifactEventType::Created(ArtifactCreatedEvent {
                artifact,
                storage_location: location,
                content_hash: None,
            }),
            source: self.source,
            context: self.context,
        }
    }

    /// Build a modified event
    pub fn modified(
        self,
        artifact_id: ArtifactId,
        modifications: Vec<Modification>,
    ) -> ArtifactEvent {
        ArtifactEvent {
            metadata: EventMetadata::new(),
            event_type: ArtifactEventType::Modified(ArtifactModifiedEvent {
                artifact_id,
                modifications,
                previous_hash: None,
                new_hash: None,
                size_delta: 0,
            }),
            source: self.source,
            context: self.context,
        }
    }

    /// Build a deleted event
    pub fn deleted(
        self,
        artifact_id: ArtifactId,
        metadata: ArtifactMetadata,
        permanent: bool,
    ) -> ArtifactEvent {
        ArtifactEvent {
            metadata: EventMetadata::new(),
            event_type: ArtifactEventType::Deleted(ArtifactDeletedEvent {
                artifact_id,
                artifact_metadata: metadata,
                permanent,
                backup_location: None,
            }),
            source: self.source,
            context: self.context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_event_builder() {
        let component_id = ComponentId::new();
        let artifact_id = ArtifactId::new("test-artifact");
        let metadata = ArtifactMetadata::new(
            artifact_id.clone(),
            "test".to_string(),
            "test.txt".to_string(),
            component_id.clone(),
        );

        let event = ArtifactEventBuilder::new(component_id.clone())
            .with_context("user", serde_json::json!("test-user"))
            .created(
                metadata,
                Some(StorageLocation::Memory("test-storage".to_string())),
            );

        assert_eq!(event.event_name(), "artifact.created");
        assert_eq!(event.source, component_id);
        assert_eq!(event.context.get("user").unwrap(), "test-user");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_event_names() {
        let component_id = ComponentId::new();

        let created = ArtifactEvent::new(
            ArtifactEventType::Created(ArtifactCreatedEvent {
                artifact: ArtifactMetadata::new(
                    ArtifactId::new("test"),
                    "test".to_string(),
                    "test.txt".to_string(),
                    component_id.clone(),
                ),
                storage_location: None,
                content_hash: None,
            }),
            component_id.clone(),
        );
        assert_eq!(created.event_name(), "artifact.created");

        let modified = ArtifactEvent::new(
            ArtifactEventType::Modified(ArtifactModifiedEvent {
                artifact_id: ArtifactId::new("test"),
                modifications: vec![Modification::Content],
                previous_hash: None,
                new_hash: None,
                size_delta: 0,
            }),
            component_id,
        );
        assert_eq!(modified.event_name(), "artifact.modified");
    }
}
