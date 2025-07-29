// ABOUTME: Advanced schema evolution example with complex transformations and breaking changes
// ABOUTME: Demonstrates field renaming, splitting, merging, and type conversions

use llmspell_state_persistence::{
    config::FieldSchema,
    migration::{planner::MigrationPlanner, transforms::*, validator::*},
    schema::*,
    StateManager, StateScope,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧬 rs-llmspell Schema Evolution Example");
    println!("======================================");

    // Initialize StateManager
    let state_manager = StateManager::with_backend(
        llmspell_state_persistence::config::StorageBackendType::Memory,
        llmspell_state_persistence::config::PersistenceConfig::default(),
    )
    .await?;

    println!("✅ StateManager initialized");

    // Define evolutionary schema versions for a user management system

    // ===== SCHEMA v1.0.0: Basic User Profile =====
    let mut schema_v1_0 = EnhancedStateSchema::new(SemanticVersion::new(1, 0, 0));

    schema_v1_0.add_field(
        "user_id".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["uuid".to_string()],
        },
    );

    schema_v1_0.add_field(
        "username".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["min_length:3".to_string()],
        },
    );

    schema_v1_0.add_field(
        "full_name".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec![],
        },
    );

    schema_v1_0.add_field(
        "email".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["email".to_string()],
        },
    );

    schema_v1_0.add_field(
        "created_timestamp".to_string(),
        FieldSchema {
            field_type: "number".to_string(),
            required: true,
            default_value: None,
            validators: vec!["timestamp".to_string()],
        },
    );

    println!("📋 Schema v1.0.0 - Basic User Profile:");
    println!("   • user_id (string, required)");
    println!("   • username (string, required)");
    println!("   • full_name (string, required)");
    println!("   • email (string, required)");
    println!("   • created_timestamp (number, required)");

    // ===== SCHEMA v1.5.0: Extended Profile =====
    let mut schema_v1_5 = EnhancedStateSchema::new(SemanticVersion::new(1, 5, 0));

    // Keep existing fields
    for (field_name, field_schema) in &schema_v1_0.fields {
        schema_v1_5.add_field(field_name.clone(), field_schema.clone());
    }

    // Add new optional fields
    schema_v1_5.add_field(
        "phone".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: false,
            default_value: None,
            validators: vec!["phone".to_string()],
        },
    );

    schema_v1_5.add_field(
        "bio".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: false,
            default_value: Some(serde_json::json!("")),
            validators: vec!["max_length:500".to_string()],
        },
    );

    schema_v1_5.add_field(
        "preferences".to_string(),
        FieldSchema {
            field_type: "object".to_string(),
            required: false,
            default_value: Some(serde_json::json!({})),
            validators: vec![],
        },
    );

    println!("\n📋 Schema v1.5.0 - Extended Profile (backward compatible):");
    println!("   • All v1.0.0 fields +");
    println!("   • phone (string, optional)");
    println!("   • bio (string, optional)");
    println!("   • preferences (object, optional)");

    // ===== SCHEMA v2.0.0: Restructured Profile (Breaking Changes) =====
    let mut schema_v2_0 = EnhancedStateSchema::new(SemanticVersion::new(2, 0, 0));

    schema_v2_0.add_field(
        "user_id".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["uuid".to_string()],
        },
    );

    // BREAKING: username -> handle
    schema_v2_0.add_field(
        "handle".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["min_length:3".to_string(), "max_length:20".to_string()],
        },
    );

    // BREAKING: full_name -> first_name + last_name
    schema_v2_0.add_field(
        "first_name".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["min_length:1".to_string()],
        },
    );

    schema_v2_0.add_field(
        "last_name".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["min_length:1".to_string()],
        },
    );

    // Keep email
    schema_v2_0.add_field(
        "email".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["email".to_string()],
        },
    );

    // BREAKING: created_timestamp -> created_at (string ISO format)
    schema_v2_0.add_field(
        "created_at".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec!["iso_datetime".to_string()],
        },
    );

    // Merge phone into communication object
    schema_v2_0.add_field(
        "communication".to_string(),
        FieldSchema {
            field_type: "object".to_string(),
            required: false,
            default_value: Some(serde_json::json!({
                "phone": null,
                "preferred_method": "email"
            })),
            validators: vec![],
        },
    );

    // Enhanced profile object
    schema_v2_0.add_field(
        "profile".to_string(),
        FieldSchema {
            field_type: "object".to_string(),
            required: false,
            default_value: Some(serde_json::json!({
                "bio": "",
                "avatar_url": null,
                "location": null,
                "website": null
            })),
            validators: vec![],
        },
    );

    // Keep preferences but make it part of user settings
    schema_v2_0.add_field(
        "settings".to_string(),
        FieldSchema {
            field_type: "object".to_string(),
            required: false,
            default_value: Some(serde_json::json!({
                "preferences": {},
                "privacy": {"public_profile": true},
                "notifications": {"email": true, "push": false}
            })),
            validators: vec![],
        },
    );

    println!("\n📋 Schema v2.0.0 - Restructured Profile (breaking changes):");
    println!("   • user_id (unchanged)");
    println!("   • handle (renamed from username)");
    println!("   • first_name + last_name (split from full_name)");
    println!("   • email (unchanged)");
    println!("   • created_at (converted from created_timestamp)");
    println!("   • communication (phone merged into object)");
    println!("   • profile (enhanced bio + new fields)");
    println!("   • settings (preferences + privacy + notifications)");

    // Store sample data in v1.0.0 format
    let sample_users_v1 = [
        serde_json::json!({
            "user_id": "550e8400-e29b-41d4-a716-446655440001",
            "username": "alice_wonder",
            "full_name": "Alice Wonderland",
            "email": "alice@example.com",
            "created_timestamp": 1640995200 // 2022-01-01 00:00:00 UTC
        }),
        serde_json::json!({
            "user_id": "550e8400-e29b-41d4-a716-446655440002",
            "username": "bob_builder",
            "full_name": "Bob Builder",
            "email": "bob@example.com",
            "created_timestamp": 1641081600 // 2022-01-02 00:00:00 UTC
        }),
    ];

    println!("\n📝 Storing sample data in v1.0.0 format:");
    for (i, user) in sample_users_v1.iter().enumerate() {
        state_manager
            .set(StateScope::Global, &format!("user:v1:{}", i), user.clone())
            .await?;
        println!("   Stored: {}", user["username"]);
    }

    // Compatibility Analysis
    println!("\n🔍 Compatibility Analysis:");

    let compat_v1_0_to_v1_5 = CompatibilityChecker::check_compatibility(&schema_v1_0, &schema_v1_5);
    println!("v1.0.0 -> v1.5.0:");
    println!(
        "   • Compatible: {} (additive changes)",
        compat_v1_0_to_v1_5.compatible
    );
    println!(
        "   • Field changes: {}",
        compat_v1_0_to_v1_5.field_changes.len()
    );
    println!(
        "   • Breaking changes: {}",
        compat_v1_0_to_v1_5.breaking_changes.len()
    );
    println!("   • Risk level: {:?}", compat_v1_0_to_v1_5.risk_level);

    let compat_v1_5_to_v2_0 = CompatibilityChecker::check_compatibility(&schema_v1_5, &schema_v2_0);
    println!("v1.5.0 -> v2.0.0:");
    println!(
        "   • Compatible: {} (breaking changes)",
        compat_v1_5_to_v2_0.compatible
    );
    println!(
        "   • Field changes: {}",
        compat_v1_5_to_v2_0.field_changes.len()
    );
    println!(
        "   • Breaking changes: {}",
        compat_v1_5_to_v2_0.breaking_changes.len()
    );
    println!("   • Risk level: {:?}", compat_v1_5_to_v2_0.risk_level);

    // Advanced Transformation Examples
    println!("\n🔄 Advanced Transformation Examples:");

    let transformer = DataTransformer::new();

    // === Transformation 1: v1.0.0 -> v1.5.0 (Simple Addition) ===
    println!("\n1️⃣  v1.0.0 -> v1.5.0 Transformation:");

    let mut transform_v1_to_v1_5 = StateTransformation::new(
        "v1_0_to_v1_5".to_string(),
        "Add optional fields with defaults".to_string(),
        1,
        2,
    );

    // Add default values for new optional fields
    transform_v1_to_v1_5.add_transform(FieldTransform::Default {
        field: "bio".to_string(),
        value: serde_json::json!(""),
    });

    transform_v1_to_v1_5.add_transform(FieldTransform::Default {
        field: "preferences".to_string(),
        value: serde_json::json!({}),
    });

    let mut state_v1 = llmspell_state_persistence::manager::SerializableState {
        key: "transform_test_1".to_string(),
        value: sample_users_v1[0].clone(),
        timestamp: std::time::SystemTime::now(),
        schema_version: 1,
    };

    println!(
        "   Before: {}",
        serde_json::to_string_pretty(&state_v1.value)?
    );

    let result1 = transformer.transform_state(&mut state_v1, &transform_v1_to_v1_5)?;

    println!(
        "   After: {}",
        serde_json::to_string_pretty(&state_v1.value)?
    );
    println!(
        "   ✅ Success: {}, Fields transformed: {}",
        result1.success, result1.fields_transformed
    );

    // === Transformation 2: v1.5.0 -> v2.0.0 (Complex Restructuring) ===
    println!("\n2️⃣  v1.5.0 -> v2.0.0 Transformation (Complex):");

    let mut transform_v1_5_to_v2_0 = StateTransformation::new(
        "v1_5_to_v2_0".to_string(),
        "Restructure profile with breaking changes".to_string(),
        2,
        3,
    );

    // Rename username -> handle
    transform_v1_5_to_v2_0.add_transform(FieldTransform::Copy {
        from_field: "username".to_string(),
        to_field: "handle".to_string(),
    });

    // Split full_name -> first_name + last_name (simplified split on space)
    transform_v1_5_to_v2_0.add_transform(FieldTransform::Split {
        from_field: "full_name".to_string(),
        to_fields: vec!["first_name".to_string(), "last_name".to_string()],
        splitter: "comma_split".to_string(), // Note: simplified for demo
    });

    // Convert timestamp to ISO datetime (simplified)
    transform_v1_5_to_v2_0.add_transform(FieldTransform::Convert {
        from_field: "created_timestamp".to_string(),
        to_field: "created_at".to_string(),
        from_type: "number".to_string(),
        to_type: "string".to_string(),
        converter: "to_string".to_string(), // Simplified conversion
    });

    // Create communication object with phone
    transform_v1_5_to_v2_0.add_transform(FieldTransform::Default {
        field: "communication".to_string(),
        value: serde_json::json!({
            "phone": null,
            "preferred_method": "email"
        }),
    });

    // Create profile object from bio
    transform_v1_5_to_v2_0.add_transform(FieldTransform::Default {
        field: "profile".to_string(),
        value: serde_json::json!({
            "bio": "",
            "avatar_url": null,
            "location": null,
            "website": null
        }),
    });

    // Create settings object from preferences
    transform_v1_5_to_v2_0.add_transform(FieldTransform::Default {
        field: "settings".to_string(),
        value: serde_json::json!({
            "preferences": {},
            "privacy": {"public_profile": true},
            "notifications": {"email": true, "push": false}
        }),
    });

    // Apply complex transformation
    let mut state_v1_5 = state_v1.clone();
    state_v1_5.schema_version = 2;

    println!("   Before complex transformation:");
    println!("   {}", serde_json::to_string_pretty(&state_v1_5.value)?);

    let result2 = transformer.transform_state(&mut state_v1_5, &transform_v1_5_to_v2_0)?;

    println!("   After complex transformation:");
    println!("   {}", serde_json::to_string_pretty(&state_v1_5.value)?);
    println!(
        "   ✅ Success: {}, Fields transformed: {}",
        result2.success, result2.fields_transformed
    );

    // Migration Planning and Risk Assessment
    println!("\n📊 Migration Planning and Risk Assessment:");

    let mut planner = MigrationPlanner::new();
    planner.register_schema(schema_v1_0.clone())?;
    planner.register_schema(schema_v1_5.clone())?;
    planner.register_schema(schema_v2_0.clone())?;

    let v1_0_0 = SemanticVersion::new(1, 0, 0);
    let v1_5_0 = SemanticVersion::new(1, 5, 0);
    let v2_0_0 = SemanticVersion::new(2, 0, 0);

    // Analyze different migration paths
    let migration_paths = vec![
        (v1_0_0.clone(), v1_5_0.clone(), "Direct minor upgrade"),
        (
            v1_5_0.clone(),
            v2_0_0.clone(),
            "Major version with breaking changes",
        ),
        (
            v1_0_0.clone(),
            v2_0_0.clone(),
            "Direct major upgrade (skip v1.5)",
        ),
    ];

    for (from, to, description) in migration_paths {
        if planner.is_migration_possible(&from, &to) {
            let complexity = planner.estimate_complexity(&from, &to)?;

            println!("\n🛤️  Migration Path: {} -> {} ({})", from, to, description);
            println!("   • Field changes: {}", complexity.field_changes);
            println!("   • Breaking changes: {}", complexity.breaking_changes);
            println!(
                "   • Estimated duration: {:?}",
                complexity.estimated_duration
            );
            println!("   • Requires backup: {}", complexity.requires_backup);
            println!("   • Complexity score: {}", complexity.complexity_score);

            if complexity.is_simple() {
                println!("   • Risk assessment: ✅ LOW (Simple migration)");
            } else if complexity.is_complex() {
                println!("   • Risk assessment: ⚠️  HIGH (Complex migration - requires careful planning)");
            } else {
                println!("   • Risk assessment: ⚡ MEDIUM (Moderate complexity)");
            }
        } else {
            println!("\n🛤️  Migration Path: {} -> {} - ❌ NOT POSSIBLE", from, to);
        }
    }

    // Validation Examples
    println!("\n🔍 Advanced Validation Examples:");

    // Strict validation for production
    let strict_rules = ValidationRules::strict();
    let strict_validator = MigrationValidator::new(strict_rules);

    // Permissive validation for development
    let permissive_rules = ValidationRules::permissive();
    let permissive_validator = MigrationValidator::new(permissive_rules);

    // Test data with various validation scenarios
    let test_states = vec![
        // Valid complete state
        llmspell_state_persistence::manager::SerializableState {
            key: "valid_complete".to_string(),
            value: serde_json::json!({
                "user_id": "550e8400-e29b-41d4-a716-446655440003",
                "handle": "charlie_test",
                "first_name": "Charlie",
                "last_name": "Test",
                "email": "charlie@example.com",
                "created_at": "2022-01-01T00:00:00Z",
                "communication": {"phone": "+1234567890", "preferred_method": "email"},
                "profile": {"bio": "Test user", "avatar_url": null, "location": "Test City", "website": null},
                "settings": {"preferences": {"theme": "dark"}, "privacy": {"public_profile": true}, "notifications": {"email": true, "push": false}}
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 3,
        },
        // Valid minimal state (optional fields missing)
        llmspell_state_persistence::manager::SerializableState {
            key: "valid_minimal".to_string(),
            value: serde_json::json!({
                "user_id": "550e8400-e29b-41d4-a716-446655440004",
                "handle": "minimal",
                "first_name": "Min",
                "last_name": "User",
                "email": "min@example.com",
                "created_at": "2022-01-01T00:00:00Z"
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 3,
        },
        // Invalid state (missing required field)
        llmspell_state_persistence::manager::SerializableState {
            key: "invalid_missing_required".to_string(),
            value: serde_json::json!({
                "user_id": "550e8400-e29b-41d4-a716-446655440005",
                "handle": "incomplete",
                "first_name": "Incomplete",
                // Missing required last_name
                "email": "incomplete@example.com",
                "created_at": "2022-01-01T00:00:00Z"
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 3,
        },
    ];

    println!("\n📋 Strict Validation Results:");
    let strict_result = strict_validator.validate_post_migration(&test_states, &schema_v2_0)?;
    println!("   • Overall passed: {}", strict_result.passed);
    println!("   • Items validated: {}", strict_result.validated_items);
    println!("   • Warnings: {}", strict_result.warnings_count);
    println!("   • Errors: {}", strict_result.errors_count);
    println!("   • Critical issues: {}", strict_result.critical_count);

    if !strict_result.issues.is_empty() {
        println!("   📋 Issues found:");
        for issue in &strict_result.issues {
            println!("      [{:?}] {}", issue.severity, issue.message);
            if let Some(suggestion) = &issue.suggestion {
                println!("         💡 Suggestion: {}", suggestion);
            }
        }
    }

    println!("\n📋 Permissive Validation Results:");
    let permissive_result =
        permissive_validator.validate_post_migration(&test_states, &schema_v2_0)?;
    println!("   • Overall passed: {}", permissive_result.passed);
    println!(
        "   • Items validated: {}",
        permissive_result.validated_items
    );
    println!("   • Warnings: {}", permissive_result.warnings_count);
    println!("   • Errors: {}", permissive_result.errors_count);

    // Performance Simulation
    println!("\n⚡ Performance Simulation:");

    let start_time = std::time::Instant::now();
    let mut batch_states = Vec::new();

    // Generate batch of test states
    for i in 0..1000 {
        batch_states.push(llmspell_state_persistence::manager::SerializableState {
            key: format!("batch_user:{}", i),
            value: serde_json::json!({
                "user_id": format!("550e8400-e29b-41d4-a716-44665544{:04}", i),
                "username": format!("user_{}", i),
                "full_name": format!("User {}", i),
                "email": format!("user{}@example.com", i),
                "created_timestamp": 1640995200 + i as i64
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 1,
        });
    }

    let batch_creation_time = start_time.elapsed();
    println!(
        "   • Created 1000 test states in: {:?}",
        batch_creation_time
    );

    // Simulate batch transformation
    let transform_start = std::time::Instant::now();
    let mut transformed_count = 0;

    for state in &mut batch_states {
        let result = transformer.transform_state(state, &transform_v1_to_v1_5)?;
        if result.success {
            transformed_count += 1;
        }
    }

    let transform_duration = transform_start.elapsed();
    let avg_transform_time = transform_duration / 1000;

    println!(
        "   • Transformed {} states in: {:?}",
        transformed_count, transform_duration
    );
    println!(
        "   • Average time per transformation: {:?}",
        avg_transform_time
    );
    println!(
        "   • Throughput: {:.0} transformations/second",
        1000.0 / transform_duration.as_secs_f64()
    );

    println!("\n🎯 Schema Evolution Example Summary:");
    println!("=====================================");
    println!("This example demonstrates:");
    println!("  ✅ Multi-version schema definition and evolution");
    println!("  ✅ Backward compatible changes (v1.0 -> v1.5)");
    println!("  ✅ Breaking changes with restructuring (v1.5 -> v2.0)");
    println!("  ✅ Complex field transformations (rename, split, merge)");
    println!("  ✅ Type conversions and data restructuring");
    println!("  ✅ Migration risk assessment and complexity analysis");
    println!("  ✅ Comprehensive validation with different rule sets");
    println!("  ✅ Performance characteristics and batch processing");
    println!("  ✅ Real-world migration scenarios and best practices");

    println!("\n💡 Key Takeaways:");
    println!("  • Plan schema changes carefully - breaking changes are complex");
    println!("  • Use semantic versioning to communicate change impact");
    println!("  • Always validate migrations thoroughly before production");
    println!("  • Consider migration performance for large datasets");
    println!("  • Provide rollback capabilities for complex migrations");
    println!("  • Test with both strict and permissive validation");

    Ok(())
}
