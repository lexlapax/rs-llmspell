//! ABOUTME: Core types and foundational data structures
//! ABOUTME: Provides ComponentId, Version, and ComponentMetadata types

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for components in the LLMSpell system.
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
    /// Generate a new random ComponentId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create ComponentId from name (deterministic)
    pub fn from_name(name: &str) -> Self {
        let namespace = Uuid::NAMESPACE_DNS;
        Self(Uuid::new_v5(&namespace, name.as_bytes()))
    }
    
    /// Get inner UUID
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
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    /// Check if this version is compatible with another (same major version)
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.major == other.major
    }
    
    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &Version) -> bool {
        self > other
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Metadata for components in the LLMSpell system.
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
        let mut metadata = ComponentMetadata::new(
            "test".to_string(),
            "test component".to_string()
        );
        
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
        let metadata = ComponentMetadata::new(
            "test".to_string(),
            "test component".to_string()
        );
        
        // Test JSON serialization roundtrip
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ComponentMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.description, deserialized.description);
    }
}