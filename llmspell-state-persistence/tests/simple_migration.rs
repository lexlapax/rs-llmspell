// ABOUTME: Simple migration tests that test actual implemented functionality
// ABOUTME: Tests basic field operations without complex nested transformations

use llmspell_state_persistence::{
    manager::SerializableState,
    migration::{DataTransformer, FieldTransform, StateTransformation},
};
use serde_json::json;
use std::time::SystemTime;

#[test]
fn test_field_copy() {
    let transformer = DataTransformer::new();

    let mut state = SerializableState {
        key: "test".to_string(),
        value: json!({
            "old_name": "John Doe",
            "age": 30
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    let mut transformation = StateTransformation::new(
        "copy_field".to_string(),
        "Copy old_name to new_name".to_string(),
        1,
        2,
    );

    transformation.add_transform(FieldTransform::Copy {
        from_field: "old_name".to_string(),
        to_field: "new_name".to_string(),
    });

    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();
    assert!(result.success);

    // The Copy transform should copy and remove the old field
    assert_eq!(state.value.get("new_name"), Some(&json!("John Doe")));
    assert!(state.value.get("old_name").is_none());
}

#[test]
fn test_field_default() {
    let transformer = DataTransformer::new();

    let mut state = SerializableState {
        key: "test".to_string(),
        value: json!({
            "name": "John Doe"
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    let mut transformation = StateTransformation::new(
        "add_defaults".to_string(),
        "Add default fields".to_string(),
        1,
        2,
    );

    transformation.add_transform(FieldTransform::Default {
        field: "status".to_string(),
        value: json!("active"),
    });

    transformation.add_transform(FieldTransform::Default {
        field: "verified".to_string(),
        value: json!(false),
    });

    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();
    assert!(result.success);

    assert_eq!(state.value.get("status"), Some(&json!("active")));
    assert_eq!(state.value.get("verified"), Some(&json!(false)));
    assert_eq!(state.value.get("name"), Some(&json!("John Doe")));
}

#[test]
fn test_field_remove() {
    let transformer = DataTransformer::new();

    let mut state = SerializableState {
        key: "test".to_string(),
        value: json!({
            "name": "John Doe",
            "deprecated_field": "old_value",
            "another_deprecated": 123
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    let mut transformation = StateTransformation::new(
        "remove_fields".to_string(),
        "Remove deprecated fields".to_string(),
        1,
        2,
    );

    transformation.add_transform(FieldTransform::Remove {
        field: "deprecated_field".to_string(),
    });

    transformation.add_transform(FieldTransform::Remove {
        field: "another_deprecated".to_string(),
    });

    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();
    assert!(result.success);

    assert!(state.value.get("deprecated_field").is_none());
    assert!(state.value.get("another_deprecated").is_none());
    assert_eq!(state.value.get("name"), Some(&json!("John Doe")));
}

#[test]
fn test_multiple_transforms() {
    let transformer = DataTransformer::new();

    let mut state = SerializableState {
        key: "test".to_string(),
        value: json!({
            "first_name": "John",
            "last_name": "Doe",
            "temp_field": "temporary"
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    };

    let mut transformation = StateTransformation::new(
        "multi_transform".to_string(),
        "Multiple field transformations".to_string(),
        1,
        2,
    );

    // Copy first_name to given_name
    transformation.add_transform(FieldTransform::Copy {
        from_field: "first_name".to_string(),
        to_field: "given_name".to_string(),
    });

    // Copy last_name to family_name
    transformation.add_transform(FieldTransform::Copy {
        from_field: "last_name".to_string(),
        to_field: "family_name".to_string(),
    });

    // Remove temp_field
    transformation.add_transform(FieldTransform::Remove {
        field: "temp_field".to_string(),
    });

    // Add default middle_name
    transformation.add_transform(FieldTransform::Default {
        field: "middle_name".to_string(),
        value: json!(""),
    });

    let result = transformer
        .transform_state(&mut state, &transformation)
        .unwrap();
    assert!(result.success);

    // Check all transformations were applied
    assert_eq!(state.value.get("given_name"), Some(&json!("John")));
    assert_eq!(state.value.get("family_name"), Some(&json!("Doe")));
    assert_eq!(state.value.get("middle_name"), Some(&json!("")));
    assert!(state.value.get("first_name").is_none());
    assert!(state.value.get("last_name").is_none());
    assert!(state.value.get("temp_field").is_none());
}
