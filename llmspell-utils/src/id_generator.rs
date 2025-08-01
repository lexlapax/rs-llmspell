// ABOUTME: Component ID generation using UUIDs with namespace support
// ABOUTME: Provides consistent ID generation for all LLMSpell components

//! Component ID generation utilities
//!
//! This module provides UUID-based ID generation for components,
//! with support for namespaced IDs and deterministic generation.

use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Type alias for component IDs
pub type ComponentId = String;

/// Counter for sequential IDs (used in tests)
static SEQUENTIAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique component ID with prefix
///
/// Creates a UUID v4 with the given prefix. The format is: `{prefix}_{uuid}`
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::generate_component_id;
///
/// let agent_id = generate_component_id("agent");
/// assert!(agent_id.starts_with("agent_"));
/// assert_eq!(agent_id.len(), 42); // "agent_" (6) + UUID (36)
/// ```
#[must_use]
pub fn generate_component_id(prefix: &str) -> ComponentId {
    let uuid = Uuid::new_v4();
    format!("{prefix}_{uuid}")
}

/// Generate a short component ID with prefix
///
/// Creates a shorter ID using only the first 8 characters of the UUID.
/// Less collision-resistant but more readable.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::generate_short_id;
///
/// let tool_id = generate_short_id("tool");
/// assert!(tool_id.starts_with("tool_"));
/// assert!(tool_id.len() <= 14); // "tool_" (5) + 8 UUID chars
/// ```
#[must_use]
pub fn generate_short_id(prefix: &str) -> ComponentId {
    let uuid = Uuid::new_v4();
    let short_uuid = &uuid.to_string()[..8];
    format!("{prefix}_{short_uuid}")
}

/// Generate a deterministic component ID
///
/// Creates a UUID v5 (namespace + name based) for deterministic generation.
/// The same namespace and name will always produce the same ID.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::{generate_deterministic_id, NAMESPACE_AGENT};
///
/// let id1 = generate_deterministic_id(NAMESPACE_AGENT, "my-agent");
/// let id2 = generate_deterministic_id(NAMESPACE_AGENT, "my-agent");
/// assert_eq!(id1, id2);
/// ```
#[must_use]
pub fn generate_deterministic_id(namespace: &Uuid, name: &str) -> ComponentId {
    let uuid = Uuid::new_v5(namespace, name.as_bytes());
    uuid.to_string()
}

/// Generate a sequential ID for testing
///
/// Creates IDs with incrementing numbers. Useful for tests where
/// you need predictable, ordered IDs.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::generate_sequential_id;
///
/// let id1 = generate_sequential_id("test");
/// let id2 = generate_sequential_id("test");
/// assert!(id1 < id2); // Lexicographically ordered
/// ```
#[must_use]
pub fn generate_sequential_id(prefix: &str) -> ComponentId {
    let count = SEQUENTIAL_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}_{count:08}")
}

/// Validate a component ID format
///
/// Checks if the ID follows the expected format: `{prefix}_{uuid}`
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::{generate_component_id, validate_component_id};
///
/// let id = generate_component_id("agent");
/// assert!(validate_component_id(&id, Some("agent")));
/// assert!(!validate_component_id(&id, Some("tool")));
/// assert!(!validate_component_id("invalid-id", None));
/// ```
#[must_use]
pub fn validate_component_id(id: &str, expected_prefix: Option<&str>) -> bool {
    let parts: Vec<&str> = id.splitn(2, '_').collect();

    if parts.len() != 2 {
        return false;
    }

    // Check prefix if specified
    if let Some(prefix) = expected_prefix {
        if parts[0] != prefix {
            return false;
        }
    }

    // Validate UUID part
    Uuid::parse_str(parts[1]).is_ok()
}

/// Extract the prefix from a component ID
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::{generate_component_id, extract_prefix};
///
/// let id = generate_component_id("workflow");
/// assert_eq!(extract_prefix(&id), Some("workflow"));
/// assert_eq!(extract_prefix("invalid"), None);
/// ```
#[must_use]
pub fn extract_prefix(id: &str) -> Option<&str> {
    id.find('_').map(|pos| &id[..pos])
}

/// Extract the UUID part from a component ID
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::{generate_component_id, extract_uuid};
///
/// let id = generate_component_id("agent");
/// let uuid = extract_uuid(&id).unwrap();
/// assert_eq!(uuid.len(), 36); // Standard UUID length
/// ```
#[must_use]
pub fn extract_uuid(id: &str) -> Option<&str> {
    id.find('_').map(|pos| &id[pos + 1..])
}

// Predefined namespaces for deterministic ID generation
/// Namespace for agent IDs
pub const NAMESPACE_AGENT: &Uuid = &Uuid::from_bytes([
    0x6b, 0xa1, 0x3d, 0x5a, 0x3f, 0x5d, 0x4a, 0x5b, 0x9f, 0x7e, 0x3d, 0x5a, 0x2f, 0x4d, 0x3a, 0x2b,
]);

/// Namespace for tool IDs
pub const NAMESPACE_TOOL: &Uuid = &Uuid::from_bytes([
    0x7c, 0xb2, 0x4e, 0x6b, 0x4f, 0x6e, 0x5b, 0x6c, 0xaf, 0x8f, 0x4e, 0x6b, 0x3f, 0x5e, 0x4b, 0x3c,
]);

/// Namespace for workflow IDs
pub const NAMESPACE_WORKFLOW: &Uuid = &Uuid::from_bytes([
    0x8d, 0xc3, 0x5f, 0x7c, 0x5f, 0x7f, 0x6c, 0x7d, 0xbf, 0x9f, 0x5f, 0x7c, 0x4f, 0x6f, 0x5c, 0x4d,
]);

/// Builder for creating customized component IDs
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::id_generator::ComponentIdBuilder;
///
/// let id = ComponentIdBuilder::new()
///     .with_prefix("custom")
///     .with_timestamp()
///     .build();
///
/// assert!(id.starts_with("custom_"));
/// assert!(id.contains("_")); // Contains timestamp separator
/// ```
pub struct ComponentIdBuilder {
    prefix: Option<String>,
    use_timestamp: bool,
    use_short: bool,
    custom_suffix: Option<String>,
}

impl ComponentIdBuilder {
    /// Create a new ID builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefix: None,
            use_timestamp: false,
            use_short: false,
            custom_suffix: None,
        }
    }

    /// Set the prefix for the ID
    #[must_use]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Include a timestamp in the ID
    #[must_use]
    pub fn with_timestamp(mut self) -> Self {
        self.use_timestamp = true;
        self
    }

    /// Use short UUID format
    #[must_use]
    pub fn short(mut self) -> Self {
        self.use_short = true;
        self
    }

    /// Add a custom suffix
    #[must_use]
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.custom_suffix = Some(suffix.into());
        self
    }

    /// Build the component ID
    #[must_use]
    pub fn build(self) -> ComponentId {
        let mut parts = Vec::new();

        // Add prefix
        if let Some(prefix) = self.prefix {
            parts.push(prefix);
        }

        // Add timestamp if requested
        if self.use_timestamp {
            use chrono::Utc;
            parts.push(Utc::now().timestamp().to_string());
        }

        // Add UUID
        let uuid = Uuid::new_v4();
        let uuid_str = if self.use_short {
            uuid.to_string()[..8].to_string()
        } else {
            uuid.to_string()
        };
        parts.push(uuid_str);

        // Add custom suffix
        if let Some(suffix) = self.custom_suffix {
            parts.push(suffix);
        }

        parts.join("_")
    }
}

impl Default for ComponentIdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_generate_component_id() {
        let id1 = generate_component_id("agent");
        let id2 = generate_component_id("agent");

        assert!(id1.starts_with("agent_"));
        assert!(id2.starts_with("agent_"));
        assert_ne!(id1, id2); // Should be unique
        assert_eq!(id1.len(), 42); // "agent_" (6) + UUID (36)
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_generate_short_id() {
        let id = generate_short_id("tool");
        assert!(id.starts_with("tool_"));
        assert!(id.len() <= 14); // "tool_" (5) + 8 chars max
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_generate_deterministic_id() {
        let id1 = generate_deterministic_id(NAMESPACE_AGENT, "test-agent");
        let id2 = generate_deterministic_id(NAMESPACE_AGENT, "test-agent");
        let id3 = generate_deterministic_id(NAMESPACE_AGENT, "other-agent");

        assert_eq!(id1, id2); // Same input produces same output
        assert_ne!(id1, id3); // Different input produces different output
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_generate_sequential_id() {
        let id1 = generate_sequential_id("seq");
        let id2 = generate_sequential_id("seq");
        let id3 = generate_sequential_id("seq");

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1.starts_with("seq_"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_validate_component_id() {
        let valid_id = generate_component_id("workflow");
        assert!(validate_component_id(&valid_id, Some("workflow")));
        assert!(!validate_component_id(&valid_id, Some("agent")));
        assert!(validate_component_id(&valid_id, None));

        assert!(!validate_component_id("invalid", None));
        assert!(!validate_component_id("invalid_format", None));
        assert!(!validate_component_id("prefix_not-a-uuid", None));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_extract_prefix() {
        let id = generate_component_id("custom");
        assert_eq!(extract_prefix(&id), Some("custom"));
        assert_eq!(extract_prefix("no-underscore"), None);
        assert_eq!(extract_prefix("multiple_under_scores"), Some("multiple"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_extract_uuid() {
        let id = generate_component_id("prefix");
        let uuid_part = extract_uuid(&id).unwrap();
        assert_eq!(uuid_part.len(), 36);
        assert!(Uuid::parse_str(uuid_part).is_ok());

        assert_eq!(extract_uuid("no-underscore"), None);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_component_id_builder() {
        // Basic usage
        let id1 = ComponentIdBuilder::new().with_prefix("builder").build();
        assert!(id1.starts_with("builder_"));

        // Short ID
        let id2 = ComponentIdBuilder::new()
            .with_prefix("short")
            .short()
            .build();
        assert!(id2.starts_with("short_"));
        assert!(id2.len() < 20);

        // With suffix
        let id3 = ComponentIdBuilder::new()
            .with_prefix("suffixed")
            .with_suffix("v1")
            .build();
        assert!(id3.starts_with("suffixed_"));
        assert!(id3.ends_with("_v1"));

        // With timestamp
        let id4 = ComponentIdBuilder::new()
            .with_prefix("timed")
            .with_timestamp()
            .build();
        assert!(id4.starts_with("timed_"));
        let parts: Vec<&str> = id4.split('_').collect();
        assert!(parts.len() >= 3); // prefix, timestamp, uuid
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_namespace_uniqueness() {
        // Ensure namespaces are different
        assert_ne!(NAMESPACE_AGENT, NAMESPACE_TOOL);
        assert_ne!(NAMESPACE_AGENT, NAMESPACE_WORKFLOW);
        assert_ne!(NAMESPACE_TOOL, NAMESPACE_WORKFLOW);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_different_prefixes() {
        let agent_id = generate_component_id("agent");
        let tool_id = generate_component_id("tool");
        let workflow_id = generate_component_id("workflow");

        assert!(agent_id.starts_with("agent_"));
        assert!(tool_id.starts_with("tool_"));
        assert!(workflow_id.starts_with("workflow_"));

        // Validate each
        assert!(validate_component_id(&agent_id, Some("agent")));
        assert!(validate_component_id(&tool_id, Some("tool")));
        assert!(validate_component_id(&workflow_id, Some("workflow")));
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[cfg_attr(test_category = "unit")]
        #[test]
        fn test_id_generation_properties(prefix in "[a-z]+") {
            let id = generate_component_id(&prefix);

            // ID should start with prefix
            assert!(id.starts_with(&format!("{prefix}_")));

            // ID should be validatable
            assert!(validate_component_id(&id, Some(&prefix)));

            // Should be able to extract parts
            assert_eq!(extract_prefix(&id), Some(prefix.as_str()));
            assert!(extract_uuid(&id).is_some());
        }

        #[cfg_attr(test_category = "unit")]
        #[test]
        fn test_deterministic_properties(name in "[a-zA-Z0-9-]+") {
            let id1 = generate_deterministic_id(NAMESPACE_AGENT, &name);
            let id2 = generate_deterministic_id(NAMESPACE_AGENT, &name);

            // Same input always produces same output
            assert_eq!(id1, id2);

            // Valid UUID format
            assert!(Uuid::parse_str(&id1).is_ok());
        }

        #[cfg_attr(test_category = "unit")]
        #[test]
        fn test_builder_properties(
            prefix in "[a-z]+",
            suffix in "[a-z0-9]+",
        ) {
            let id = ComponentIdBuilder::new()
                .with_prefix(&prefix)
                .with_suffix(&suffix)
                .build();

            assert!(id.starts_with(&format!("{prefix}_")));
            assert!(id.ends_with(&format!("_{suffix}")));
        }
    }
}
