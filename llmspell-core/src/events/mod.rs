//! ABOUTME: Event types and structures for llmspell-core
//! ABOUTME: Provides artifact lifecycle events and event metadata

pub mod artifact_events;

pub use artifact_events::{
    AccessType, ArtifactAccessedEvent, ArtifactCreatedEvent, ArtifactDeletedEvent,
    ArtifactDerivedEvent, ArtifactEvent, ArtifactEventBuilder, ArtifactEventType,
    ArtifactMetadataUpdatedEvent, ArtifactModifiedEvent, ArtifactValidatedEvent,
    ArtifactValidationFailedEvent, ArtifactVersionedEvent, DerivationType, Modification,
    StorageLocation, ValidationFailure, ValidationResults, ValidationType,
};
