//! ABOUTME: Core types and foundational data structures
//! ABOUTME: Provides `ComponentId`, `Version`, `ComponentMetadata` and streaming types

pub mod agent_io;
mod media;
mod streaming;

pub use agent_io::{
    AgentInput, AgentInputBuilder, AgentOutput, AgentOutputBuilder, OutputMetadata, ToolCall,
    ToolOutput,
};
pub use media::{
    AudioFormat, AudioMetadata, ColorSpace, ImageFormat, ImageMetadata, MediaContent, MediaType,
    VideoFormat, VideoMetadata, MAX_AUDIO_SIZE, MAX_BINARY_SIZE, MAX_IMAGE_SIZE, MAX_VIDEO_SIZE,
};
pub use streaming::{
    AgentChunk, AgentStream, ChunkContent, ChunkMetadata, ControlMessage, ReasoningStep,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Unique identifier for components in the `LLMSpell` system.
///
/// `ComponentId` uses UUID v4 for random generation and UUID v5 for deterministic
/// generation from names. This allows both unique random IDs and reproducible IDs
/// for named components.
///
/// # Examples
///
/// ```
/// use llmspell_core::ComponentId;
///
/// // Create a random ID
/// let id1 = ComponentId::new();
/// let id2 = ComponentId::new();
/// assert_ne!(id1, id2);
///
/// // Create deterministic ID from name
/// let id3 = ComponentId::from_name("my-agent");
/// let id4 = ComponentId::from_name("my-agent");
/// assert_eq!(id3, id4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(Uuid);

impl ComponentId {
    /// Generate a new random `ComponentId`
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create `ComponentId` from name (deterministic)
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        let namespace = Uuid::NAMESPACE_DNS;
        Self(Uuid::new_v5(&namespace, name.as_bytes()))
    }

    /// Get inner UUID
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic version information for components.
///
/// Follows semantic versioning specification (major.minor.patch).
/// Used to track component versions and check compatibility.
///
/// # Examples
///
/// ```
/// use llmspell_core::Version;
///
/// let v1 = Version::new(1, 0, 0);
/// let v2 = Version::new(1, 1, 0);
///
/// // Check compatibility (same major version)
/// assert!(v1.is_compatible_with(&v2));
///
/// // Check if newer
/// assert!(v2.is_newer_than(&v1));
///
/// // Display version
/// assert_eq!(v1.to_string(), "1.0.0");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    #[must_use]
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this version is compatible with another (same major version)
    #[must_use]
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.major == other.major
    }

    /// Check if this version is newer than another
    #[must_use]
    pub fn is_newer_than(&self, other: &Version) -> bool {
        self > other
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Metadata for components in the `LLMSpell` system.
///
/// Contains essential information about a component including its ID, name,
/// version, description, and timestamps. This metadata is used throughout
/// the system for component identification and management.
///
/// # Examples
///
/// ```
/// use llmspell_core::{ComponentMetadata, Version};
///
/// let mut metadata = ComponentMetadata::new(
///     "research-agent".to_string(),
///     "An agent for conducting research".to_string(),
/// );
///
/// // Update version
/// metadata.update_version(Version::new(1, 1, 0));
///
/// assert_eq!(metadata.name, "research-agent");
/// assert_eq!(metadata.version, Version::new(1, 1, 0));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub id: ComponentId,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ComponentMetadata {
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: ComponentId::from_name(&name),
            name,
            version: Version::new(0, 1, 0),
            description,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the version and updated_at timestamp
    pub fn update_version(&mut self, version: Version) {
        self.version = version;
        self.updated_at = chrono::Utc::now();
    }

    /// Get the component type as a string for event emission
    ///
    /// Infers the component type from the name or ID pattern.
    /// Returns "agent", "tool", "workflow", or "component" as appropriate.
    pub fn component_type(&self) -> &str {
        // Try to infer from name patterns
        let name_lower = self.name.to_lowercase();

        if name_lower.contains("agent") {
            "agent"
        } else if name_lower.contains("tool") {
            "tool"
        } else if name_lower.contains("workflow") {
            "workflow"
        } else if name_lower.ends_with("_agent") {
            "agent"
        } else if name_lower.ends_with("_tool") {
            "tool"
        } else if name_lower.ends_with("_workflow") {
            "workflow"
        } else {
            // Default to generic component
            "component"
        }
    }
}

/// Metadata for events in the `LLMSpell` system.
///
/// Contains correlation information for tracking events across components,
/// including trace IDs, span IDs, and custom attributes. Used for event
/// correlation, debugging, and observability.
///
/// # Examples
///
/// ```
/// use llmspell_core::EventMetadata;
///
/// let mut metadata = EventMetadata::new();
/// metadata.set_trace_id("trace-123".to_string());
/// metadata.set_span_id("span-456".to_string());
/// metadata.add_attribute("user_id", "user-789");
///
/// assert_eq!(metadata.trace_id(), Some("trace-123"));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Trace ID for distributed tracing
    trace_id: Option<String>,

    /// Span ID for the current operation
    span_id: Option<String>,

    /// Parent span ID for hierarchical tracing
    parent_span_id: Option<String>,

    /// Event correlation ID
    correlation_id: Option<String>,

    /// Event timestamp
    timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Source component that generated the event
    source: Option<ComponentId>,

    /// Custom attributes for additional metadata
    attributes: HashMap<String, String>,
}

impl EventMetadata {
    /// Create new empty event metadata
    #[must_use]
    pub fn new() -> Self {
        Self {
            timestamp: Some(chrono::Utc::now()),
            ..Default::default()
        }
    }

    /// Create metadata with trace and span IDs
    #[must_use]
    pub fn with_trace(trace_id: String, span_id: String) -> Self {
        Self {
            trace_id: Some(trace_id),
            span_id: Some(span_id),
            timestamp: Some(chrono::Utc::now()),
            ..Default::default()
        }
    }

    /// Get trace ID
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// Set trace ID
    pub fn set_trace_id(&mut self, trace_id: String) {
        self.trace_id = Some(trace_id);
    }

    /// Get span ID
    #[must_use]
    pub fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }

    /// Set span ID
    pub fn set_span_id(&mut self, span_id: String) {
        self.span_id = Some(span_id);
    }

    /// Get parent span ID
    #[must_use]
    pub fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }

    /// Set parent span ID
    pub fn set_parent_span_id(&mut self, parent_span_id: String) {
        self.parent_span_id = Some(parent_span_id);
    }

    /// Get correlation ID
    #[must_use]
    pub fn correlation_id(&self) -> Option<&str> {
        self.correlation_id.as_deref()
    }

    /// Set correlation ID
    pub fn set_correlation_id(&mut self, correlation_id: String) {
        self.correlation_id = Some(correlation_id);
    }

    /// Get source component
    #[must_use]
    pub fn source(&self) -> Option<&ComponentId> {
        self.source.as_ref()
    }

    /// Set source component
    pub fn set_source(&mut self, source: ComponentId) {
        self.source = Some(source);
    }

    /// Add custom attribute
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Get custom attribute
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    /// Get all attributes
    #[must_use]
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// Create a child event metadata with new span ID
    #[must_use]
    pub fn create_child(&self, span_id: String) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Some(span_id),
            parent_span_id: self.span_id.clone(),
            correlation_id: self.correlation_id.clone(),
            timestamp: Some(chrono::Utc::now()),
            source: self.source,
            attributes: HashMap::new(), // Child starts with fresh attributes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_component_id_generation() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();

        // Each new ID should be unique
        assert_ne!(id1, id2);
        assert_ne!(id1.uuid(), id2.uuid());
    }
    #[test]
    fn test_component_id_from_name_deterministic() {
        let name = "test-component";
        let id1 = ComponentId::from_name(name);
        let id2 = ComponentId::from_name(name);

        // Same name should generate same ID
        assert_eq!(id1, id2);
        assert_eq!(id1.uuid(), id2.uuid());
    }
    #[test]
    fn test_component_id_from_different_names() {
        let id1 = ComponentId::from_name("component-a");
        let id2 = ComponentId::from_name("component-b");

        // Different names should generate different IDs
        assert_ne!(id1, id2);
    }
    #[test]
    fn test_component_id_display() {
        let id = ComponentId::from_name("test");
        let display_str = format!("{}", id);

        // Should display as UUID string
        assert!(display_str.len() == 36); // UUID string length
        assert!(display_str.contains('-')); // UUID format
    }
    #[test]
    fn test_component_id_serialization() {
        let id = ComponentId::from_name("test");

        // Test JSON serialization roundtrip
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ComponentId = serde_json::from_str(&json).unwrap();

        assert_eq!(id, deserialized);
    }
    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }
    #[test]
    fn test_version_comparison() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v1_1_1 = Version::new(1, 1, 1);
        let v2_0_0 = Version::new(2, 0, 0);

        // Test ordering
        assert!(v1_0_0 < v1_1_0);
        assert!(v1_1_0 < v1_1_1);
        assert!(v1_1_1 < v2_0_0);

        // Test newer_than
        assert!(v1_1_0.is_newer_than(&v1_0_0));
        assert!(v2_0_0.is_newer_than(&v1_1_1));
        assert!(!v1_0_0.is_newer_than(&v1_1_0));
    }
    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        // Same major version should be compatible
        assert!(v1_0_0.is_compatible_with(&v1_1_0));
        assert!(v1_1_0.is_compatible_with(&v1_0_0));

        // Different major version should not be compatible
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
    }
    #[test]
    fn test_version_display() {
        let version = Version::new(1, 2, 3);
        assert_eq!(format!("{}", version), "1.2.3");
    }
    #[test]
    fn test_version_serialization() {
        let version = Version::new(1, 2, 3);

        // Test JSON serialization roundtrip
        let json = serde_json::to_string(&version).unwrap();
        let deserialized: Version = serde_json::from_str(&json).unwrap();

        assert_eq!(version, deserialized);
    }
    #[test]
    fn test_component_metadata_creation() {
        let name = "test-component".to_string();
        let description = "A test component".to_string();

        let metadata = ComponentMetadata::new(name.clone(), description.clone());

        assert_eq!(metadata.name, name);
        assert_eq!(metadata.description, description);
        assert_eq!(metadata.version, Version::new(0, 1, 0));
        assert_eq!(metadata.id, ComponentId::from_name(&name));

        // Timestamps should be recent
        let now = chrono::Utc::now();
        let duration = now - metadata.created_at;
        assert!(duration.num_seconds() < 5); // Created within last 5 seconds
    }
    #[test]
    fn test_component_metadata_version_update() {
        let mut metadata = ComponentMetadata::new("test".to_string(), "test component".to_string());

        let original_updated_at = metadata.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        let new_version = Version::new(1, 0, 0);
        metadata.update_version(new_version.clone());

        assert_eq!(metadata.version, new_version);
        assert!(metadata.updated_at > original_updated_at);
    }
    #[test]
    fn test_component_metadata_serialization() {
        let metadata = ComponentMetadata::new("test".to_string(), "test component".to_string());

        // Test JSON serialization roundtrip
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ComponentMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.description, deserialized.description);
    }
    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata::new();

        assert!(metadata.timestamp.is_some());
        assert!(metadata.trace_id.is_none());
        assert!(metadata.span_id.is_none());
        assert!(metadata.attributes.is_empty());
    }
    #[test]
    fn test_event_metadata_with_trace() {
        let trace_id = "trace-123".to_string();
        let span_id = "span-456".to_string();

        let metadata = EventMetadata::with_trace(trace_id.clone(), span_id.clone());

        assert_eq!(metadata.trace_id(), Some("trace-123"));
        assert_eq!(metadata.span_id(), Some("span-456"));
        assert!(metadata.timestamp.is_some());
    }
    #[test]
    fn test_event_metadata_setters_getters() {
        let mut metadata = EventMetadata::new();

        metadata.set_trace_id("trace-789".to_string());
        metadata.set_span_id("span-012".to_string());
        metadata.set_parent_span_id("parent-345".to_string());
        metadata.set_correlation_id("corr-678".to_string());
        metadata.set_source(ComponentId::from_name("test-component"));

        assert_eq!(metadata.trace_id(), Some("trace-789"));
        assert_eq!(metadata.span_id(), Some("span-012"));
        assert_eq!(metadata.parent_span_id(), Some("parent-345"));
        assert_eq!(metadata.correlation_id(), Some("corr-678"));
        assert!(metadata.source().is_some());
    }
    #[test]
    fn test_event_metadata_attributes() {
        let mut metadata = EventMetadata::new();

        metadata.add_attribute("user_id", "user-123");
        metadata.add_attribute("session_id", "session-456");

        assert_eq!(metadata.get_attribute("user_id"), Some("user-123"));
        assert_eq!(metadata.get_attribute("session_id"), Some("session-456"));
        assert_eq!(metadata.get_attribute("non_existent"), None);
        assert_eq!(metadata.attributes().len(), 2);
    }
    #[test]
    fn test_event_metadata_create_child() {
        let mut parent =
            EventMetadata::with_trace("trace-parent".to_string(), "span-parent".to_string());
        parent.set_correlation_id("corr-parent".to_string());
        parent.add_attribute("parent_attr", "parent_value");

        let child = parent.create_child("span-child".to_string());

        assert_eq!(child.trace_id(), Some("trace-parent"));
        assert_eq!(child.span_id(), Some("span-child"));
        assert_eq!(child.parent_span_id(), Some("span-parent"));
        assert_eq!(child.correlation_id(), Some("corr-parent"));
        assert!(child.attributes().is_empty()); // Child starts with fresh attributes
    }
    #[test]
    fn test_event_metadata_serialization() {
        let mut metadata =
            EventMetadata::with_trace("trace-ser".to_string(), "span-ser".to_string());
        metadata.add_attribute("key", "value");

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EventMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.trace_id(), deserialized.trace_id());
        assert_eq!(metadata.span_id(), deserialized.span_id());
        assert_eq!(
            metadata.get_attribute("key"),
            deserialized.get_attribute("key")
        );
    }
}
