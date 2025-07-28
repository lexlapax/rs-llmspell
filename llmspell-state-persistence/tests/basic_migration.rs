// ABOUTME: Basic migration tests that test actual implemented functionality
// ABOUTME: Focuses on what's working rather than unimplemented features

use llmspell_state_persistence::{
    config::{PersistenceConfig, StorageBackendType},
    manager::{SerializableState, StateManager},
    migration::{DataTransformer, FieldTransform, StateTransformation},
    StateScope,
};
use serde_json::json;
use std::sync::Arc;
use std::time::SystemTime;

#[tokio::test]
async fn test_basic_migration_with_defaults() {
    // Create test data
    let mut state = SerializableState {
        key: "user_1".to_string(),
        value: json!({
            "username": "john_doe",
            "email": "john@example.com",
            "age": 30
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    // Create transformation that uses only implemented transforms
    let mut transformation = StateTransformation::new(
        "add_defaults".to_string(),
        "Add default fields".to_string(),
        1,
        2,
    );

    // Add some default fields
    transformation.add_transform(FieldTransform::Default {
        field: "verified".to_string(),
        value: json!(false),
    });

    transformation.add_transform(FieldTransform::Default {
        field: "created_at".to_string(),
        value: json!("2024-01-01T00:00:00Z"),
    });

    // Remove age field
    transformation.add_transform(FieldTransform::Remove {
        field: "age".to_string(),
    });

    // Execute transformation
    let transformer = DataTransformer::new();
    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();

    assert!(result.success);
    assert_eq!(state.value.get("username"), Some(&json!("john_doe")));
    assert_eq!(state.value.get("email"), Some(&json!("john@example.com")));
    assert_eq!(state.value.get("verified"), Some(&json!(false)));
    assert_eq!(
        state.value.get("created_at"),
        Some(&json!("2024-01-01T00:00:00Z"))
    );
    assert!(state.value.get("age").is_none());
}

#[tokio::test]
async fn test_copy_and_remove_migration() {
    let mut state = SerializableState {
        key: "product_1".to_string(),
        value: json!({
            "old_name": "Widget",
            "old_price": 99.99,
            "deprecated_field": "remove_me"
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    let mut transformation = StateTransformation::new(
        "modernize_schema".to_string(),
        "Update field names".to_string(),
        1,
        2,
    );

    // Copy fields to new names
    transformation.add_transform(FieldTransform::Copy {
        from_field: "old_name".to_string(),
        to_field: "name".to_string(),
    });

    transformation.add_transform(FieldTransform::Copy {
        from_field: "old_price".to_string(),
        to_field: "price".to_string(),
    });

    // Remove deprecated field
    transformation.add_transform(FieldTransform::Remove {
        field: "deprecated_field".to_string(),
    });

    let transformer = DataTransformer::new();
    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();

    assert!(result.success);
    assert_eq!(state.value.get("name"), Some(&json!("Widget")));
    assert_eq!(state.value.get("price"), Some(&json!(99.99)));
    assert!(state.value.get("old_name").is_none());
    assert!(state.value.get("old_price").is_none());
    assert!(state.value.get("deprecated_field").is_none());
}

#[tokio::test]
async fn test_state_persistence_with_migration() {
    // Create state manager
    let config = PersistenceConfig {
        enabled: true,
        ..Default::default()
    };
    let state_manager = Arc::new(
        StateManager::with_backend(StorageBackendType::Memory, config)
            .await
            .unwrap(),
    );

    // Save initial state
    state_manager
        .set(
            StateScope::Global,
            "config",
            json!({
                "version": "1.0",
                "debug": true,
                "old_setting": "deprecated"
            }),
        )
        .await
        .unwrap();

    // Load and migrate state
    let value = state_manager
        .get(StateScope::Global, "config")
        .await
        .unwrap()
        .expect("State should exist");

    let mut state = SerializableState {
        key: "config".to_string(),
        value,
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    // Apply migration
    let mut transformation = StateTransformation::new(
        "update_config".to_string(),
        "Modernize config".to_string(),
        1,
        2,
    );

    transformation.add_transform(FieldTransform::Copy {
        from_field: "version".to_string(),
        to_field: "app_version".to_string(),
    });

    transformation.add_transform(FieldTransform::Remove {
        field: "old_setting".to_string(),
    });

    transformation.add_transform(FieldTransform::Default {
        field: "environment".to_string(),
        value: json!("production"),
    });

    let transformer = DataTransformer::new();
    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();
    assert!(result.success);

    // Save migrated state back
    state_manager
        .set(StateScope::Global, "config", state.value.clone())
        .await
        .unwrap();

    // Verify migrated state
    let migrated = state_manager
        .get(StateScope::Global, "config")
        .await
        .unwrap()
        .expect("Migrated state should exist");

    assert_eq!(migrated.get("app_version"), Some(&json!("1.0")));
    assert_eq!(migrated.get("debug"), Some(&json!(true)));
    assert_eq!(migrated.get("environment"), Some(&json!("production")));
    assert!(migrated.get("version").is_none());
    assert!(migrated.get("old_setting").is_none());
}
