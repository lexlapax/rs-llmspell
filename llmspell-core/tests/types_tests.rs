//! Unit tests for core types
//!
//! These tests verify behavior of ComponentId, Version, and ComponentMetadata

use llmspell_core::{ComponentId, ComponentMetadata, Version};
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

#[test]
fn test_component_id_deterministic() {
    // Same name should always produce same ID
    let name = "test-component";
    let id1 = ComponentId::from_name(name);
    let id2 = ComponentId::from_name(name);
    assert_eq!(id1, id2);
}

#[test]
fn test_component_id_uniqueness() {
    // Different names should produce different IDs
    let id1 = ComponentId::from_name("component-1");
    let id2 = ComponentId::from_name("component-2");
    assert_ne!(id1, id2);
}

#[test]
fn test_component_id_thread_safety() {
    // ComponentId generation should be thread-safe
    let handles: Vec<_> = (0..10)
        .map(|i| thread::spawn(move || ComponentId::from_name(&format!("thread-{}", i))))
        .collect();

    let ids: HashSet<ComponentId> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(ids.len(), 10);
}

#[test]
fn test_version_comparison() {
    let v1 = Version::new(1, 0, 0);
    let v2 = Version::new(1, 0, 1);
    let v3 = Version::new(1, 1, 0);
    let v4 = Version::new(2, 0, 0);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 < v4);
    assert!(v1 < v4);
}

#[test]
fn test_version_equality() {
    let v1 = Version::new(1, 2, 3);
    let v2 = Version::new(1, 2, 3);
    let v3 = Version::new(1, 2, 4);

    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

#[test]
fn test_version_compatibility() {
    let v1 = Version::new(1, 0, 0);
    let v2 = Version::new(1, 0, 1);
    let v3 = Version::new(1, 1, 0);
    let v4 = Version::new(2, 0, 0);

    // Same major version is compatible
    assert!(v1.is_compatible_with(&v2));
    assert!(v1.is_compatible_with(&v3));
    assert!(v2.is_compatible_with(&v3));

    // Different major version is not compatible
    assert!(!v1.is_compatible_with(&v4));
    assert!(!v2.is_compatible_with(&v4));
    assert!(!v3.is_compatible_with(&v4));
}

#[test]
fn test_version_display() {
    let v = Version::new(1, 2, 3);
    assert_eq!(v.to_string(), "1.2.3");

    let v0 = Version::new(0, 0, 0);
    assert_eq!(v0.to_string(), "0.0.0");

    let v_max = Version::new(u32::MAX, u32::MAX, u32::MAX);
    assert_eq!(
        v_max.to_string(),
        format!("{}.{}.{}", u32::MAX, u32::MAX, u32::MAX)
    );
}

#[test]
fn test_component_metadata_creation() {
    let metadata =
        ComponentMetadata::new("test-component".to_string(), "A test component".to_string());

    assert_eq!(metadata.name, "test-component");
    assert_eq!(metadata.description, "A test component");
    assert_eq!(metadata.version, Version::new(0, 1, 0));
}

#[test]
fn test_component_metadata_update_version() {
    let mut metadata = ComponentMetadata::new("test".to_string(), "Test component".to_string());

    let original_updated_at = metadata.updated_at;

    // Sleep briefly to ensure timestamp changes
    thread::sleep(Duration::from_millis(10));

    metadata.update_version(Version::new(1, 0, 0));

    assert_eq!(metadata.version, Version::new(1, 0, 0));
    assert!(metadata.updated_at > original_updated_at);
}

#[test]
fn test_component_metadata_serialization() {
    let metadata = ComponentMetadata::new(
        "serialization-test".to_string(),
        "Test serialization".to_string(),
    );

    // Serialize
    let json = serde_json::to_string(&metadata).unwrap();

    // Deserialize
    let deserialized: ComponentMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.id, deserialized.id);
    assert_eq!(metadata.name, deserialized.name);
    assert_eq!(metadata.description, deserialized.description);
    assert_eq!(metadata.version, deserialized.version);
}

#[test]
fn test_component_metadata_timestamps() {
    let metadata = ComponentMetadata::new(
        "timestamp-test".to_string(),
        "Component to test timestamps".to_string(),
    );

    // Check that timestamps are set
    assert!(metadata.created_at <= metadata.updated_at);

    // Both should be recent (within last second)
    let now = chrono::Utc::now();
    let diff = now - metadata.created_at;
    assert!(diff.num_seconds() < 1);
}

#[test]
fn test_component_id_serialization() {
    let id = ComponentId::from_name("serialization-test");

    // Serialize
    let json = serde_json::to_string(&id).unwrap();

    // Deserialize
    let deserialized: ComponentId = serde_json::from_str(&json).unwrap();

    assert_eq!(id, deserialized);
}

#[test]
fn test_version_serialization() {
    let version = Version::new(1, 2, 3);

    // Serialize
    let json = serde_json::to_string(&version).unwrap();

    // Deserialize
    let deserialized: Version = serde_json::from_str(&json).unwrap();

    assert_eq!(version, deserialized);
}
