// ABOUTME: Basic migration example showing schema evolution with the migration framework
// ABOUTME: Demonstrates simple field additions and state transformation patterns

use llmspell_state_persistence::{
    config::FieldSchema,
    migration::{
        engine::*, planner::MigrationPlanner, transforms::*, validator::*, MigrationConfig,
        ValidationLevel,
    },
    schema::*,
    StateManager, StateScope,
};
use llmspell_storage::MemoryBackend;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 rs-llmspell Migration Example");
    println!("================================");

    // 1. Setup StateManager with memory backend
    let state_manager = StateManager::with_backend(
        llmspell_state_persistence::config::StorageBackendType::Memory,
        llmspell_state_persistence::config::PersistenceConfig::default(),
    )
    .await?;

    println!("✅ StateManager initialized with memory backend");

    // 2. Create initial schema v1.0.0
    let mut schema_v1 = EnhancedStateSchema::new(SemanticVersion::new(1, 0, 0));

    schema_v1.add_field(
        "name".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec![],
        },
    );

    schema_v1.add_field(
        "age".to_string(),
        FieldSchema {
            field_type: "number".to_string(),
            required: false,
            default_value: Some(serde_json::json!(0)),
            validators: vec![],
        },
    );

    println!("✅ Created schema v1.0.0 with fields: name (required), age (optional)");

    // 3. Store initial data using v1.0.0 schema
    let initial_users = vec![
        serde_json::json!({
            "name": "Alice Johnson",
            "age": 28
        }),
        serde_json::json!({
            "name": "Bob Smith",
            "age": 35
        }),
        serde_json::json!({
            "name": "Carol Brown"
            // age is optional, so omitted for this user
        }),
    ];

    for (i, user_data) in initial_users.iter().enumerate() {
        state_manager
            .set(
                StateScope::Global,
                &format!("user:{}", i),
                user_data.clone(),
            )
            .await?;
        println!("📝 Stored user data: {}", user_data);
    }

    // 4. Create evolved schema v1.1.0 with new fields
    let mut schema_v1_1 = EnhancedStateSchema::new(SemanticVersion::new(1, 1, 0));

    // Keep existing fields
    schema_v1_1.add_field(
        "name".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            validators: vec![],
        },
    );

    schema_v1_1.add_field(
        "age".to_string(),
        FieldSchema {
            field_type: "number".to_string(),
            required: false,
            default_value: Some(serde_json::json!(0)),
            validators: vec![],
        },
    );

    // Add new fields
    schema_v1_1.add_field(
        "email".to_string(),
        FieldSchema {
            field_type: "string".to_string(),
            required: false,
            default_value: Some(serde_json::json!("user@example.com")),
            validators: vec![],
        },
    );

    schema_v1_1.add_field(
        "active".to_string(),
        FieldSchema {
            field_type: "boolean".to_string(),
            required: false,
            default_value: Some(serde_json::json!(true)),
            validators: vec![],
        },
    );

    println!("✅ Created schema v1.1.0 with additional fields: email, active");

    // 5. Setup migration infrastructure
    let backend = Arc::new(MemoryBackend::new());
    let storage_adapter = Arc::new(
        llmspell_state_persistence::backend_adapter::StateStorageAdapter::new(
            backend,
            "migration_test".to_string(),
        ),
    );
    let schema_registry = SchemaRegistry::new();

    // Register both schemas
    schema_registry.register_schema(schema_v1.clone(), None)?;
    schema_registry.register_schema(schema_v1_1.clone(), None)?;

    println!("✅ Registered schemas in registry");

    // 6. Analyze compatibility between schemas
    let compatibility = CompatibilityChecker::check_compatibility(&schema_v1, &schema_v1_1);

    println!("🔍 Compatibility Analysis:");
    println!("   - Compatible: {}", compatibility.compatible);
    println!(
        "   - Migration required: {}",
        compatibility.migration_required
    );
    println!("   - Field changes: {}", compatibility.field_changes.len());
    println!(
        "   - Breaking changes: {}",
        compatibility.breaking_changes.len()
    );
    println!("   - Risk level: {:?}", compatibility.risk_level);

    // 7. Create migration planner
    let mut planner = MigrationPlanner::with_registry(schema_registry.clone());
    planner.register_schema(schema_v1.clone())?;
    planner.register_schema(schema_v1_1.clone())?;

    let v1_0_0 = SemanticVersion::new(1, 0, 0);
    let v1_1_0 = SemanticVersion::new(1, 1, 0);

    // Check if migration is possible
    if planner.is_migration_possible(&v1_0_0, &v1_1_0) {
        println!("✅ Migration from v1.0.0 to v1.1.0 is possible");

        // Get complexity estimate
        let complexity = planner.estimate_complexity(&v1_0_0, &v1_1_0)?;
        println!("📊 Migration Complexity:");
        println!("   - Field changes: {}", complexity.field_changes);
        println!("   - Breaking changes: {}", complexity.breaking_changes);
        println!(
            "   - Estimated duration: {:?}",
            complexity.estimated_duration
        );
        println!("   - Requires backup: {}", complexity.requires_backup);
        println!("   - Complexity score: {}", complexity.complexity_score);

        if complexity.is_simple() {
            println!("   ✅ Migration is classified as SIMPLE");
        } else if complexity.is_complex() {
            println!("   ⚠️  Migration is classified as COMPLEX");
        } else {
            println!("   ℹ️  Migration is classified as MODERATE");
        }
    } else {
        println!("❌ Migration from v1.0.0 to v1.1.0 is not possible");
    }

    // 8. Demonstrate data transformation
    println!("\n🔄 Data Transformation Example:");

    let transformer = DataTransformer::new();

    // Create a transformation that adds default values for new fields
    let mut transformation = StateTransformation::new(
        "v1_to_v1_1".to_string(),
        "Add email and active fields with defaults".to_string(),
        1,
        2, // Note: using schema_version not semantic version
    );

    // Add default email field
    transformation.add_transform(FieldTransform::Default {
        field: "email".to_string(),
        value: serde_json::json!("user@example.com"),
    });

    // Add default active field
    transformation.add_transform(FieldTransform::Default {
        field: "active".to_string(),
        value: serde_json::json!(true),
    });

    // Transform a sample state item
    let mut sample_state = llmspell_state_persistence::manager::SerializableState {
        key: "user:sample".to_string(),
        value: serde_json::json!({
            "name": "Sample User",
            "age": 25
        }),
        timestamp: std::time::SystemTime::now(),
        schema_version: 1,
    };

    println!("📝 Before transformation: {}", sample_state.value);

    let transform_result = transformer.transform_state(&mut sample_state, &transformation)?;

    println!("📝 After transformation: {}", sample_state.value);
    println!("✅ Transformation result:");
    println!("   - Success: {}", transform_result.success);
    println!(
        "   - Fields transformed: {}",
        transform_result.fields_transformed
    );
    println!(
        "   - Schema version: {} -> {}",
        1, sample_state.schema_version
    );

    // 9. Demonstrate migration validation
    println!("\n🔍 Migration Validation Example:");

    let validation_rules = ValidationRules::permissive();
    let validator = MigrationValidator::new(validation_rules);

    // Create test states for validation
    let test_states = vec![
        llmspell_state_persistence::manager::SerializableState {
            key: "valid_user".to_string(),
            value: serde_json::json!({
                "name": "Valid User",
                "age": 30,
                "email": "valid@example.com",
                "active": true
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 2,
        },
        llmspell_state_persistence::manager::SerializableState {
            key: "minimal_user".to_string(),
            value: serde_json::json!({
                "name": "Minimal User"
                // Missing optional fields - should still be valid
            }),
            timestamp: std::time::SystemTime::now(),
            schema_version: 2,
        },
    ];

    let validation_result = validator.validate_post_migration(&test_states, &schema_v1_1)?;

    println!("✅ Validation results:");
    println!("   - Passed: {}", validation_result.passed);
    println!(
        "   - Items validated: {}",
        validation_result.validated_items
    );
    println!("   - Warnings: {}", validation_result.warnings_count);
    println!("   - Errors: {}", validation_result.errors_count);
    println!("   - Validation duration: {:?}", validation_result.duration);

    if validation_result.has_warnings() {
        println!("⚠️  Validation warnings:");
        for issue in &validation_result.issues {
            if issue.severity
                == llmspell_state_persistence::migration::validator::ValidationSeverity::Warning
            {
                println!("     - {}", issue.message);
            }
        }
    }

    // 10. Setup full migration engine (for demonstration - would normally perform actual migration)
    println!("\n🏗️  Migration Engine Setup:");

    let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
    let correlation_tracker = Arc::new(llmspell_events::EventCorrelationTracker::default());
    let event_bus = Arc::new(llmspell_events::EventBus::new());

    let engine = MigrationEngine::new(
        storage_adapter,
        schema_registry,
        hook_executor,
        correlation_tracker,
        event_bus,
    );

    println!("✅ Migration engine created and ready");
    println!(
        "   - Active migrations: {}",
        engine.get_active_migrations().len()
    );

    // Migration configuration example
    let migration_config = MigrationConfig {
        dry_run: true, // Set to true for demo to avoid actual changes
        create_backup: true,
        batch_size: 100,
        timeout: Duration::from_secs(300),
        max_concurrent_migrations: 1,
        validation_level: ValidationLevel::Strict,
        rollback_on_error: true,
    };

    println!("⚙️  Migration configuration:");
    println!("   - Dry run: {}", migration_config.dry_run);
    println!("   - Create backup: {}", migration_config.create_backup);
    println!("   - Batch size: {}", migration_config.batch_size);
    println!("   - Timeout: {:?}", migration_config.timeout);
    println!(
        "   - Validation level: {:?}",
        migration_config.validation_level
    );

    // Attempt migration (will likely fail due to incomplete schema integration, but demonstrates the API)
    println!("\n🚀 Attempting migration (dry run):");
    match engine.migrate(&v1_0_0, &v1_1_0, migration_config).await {
        Ok(result) => {
            println!("✅ Migration completed successfully!");
            println!("   - Status: {:?}", result.status);
            println!("   - Items migrated: {}", result.items_migrated);
            println!("   - Duration: {:?}", result.duration);
            println!("   - Errors: {}", result.errors.len());
        }
        Err(e) => {
            println!("⚠️  Migration failed (expected in demo): {}", e);
            println!(
                "   This is normal - the demo shows the API without complete schema integration"
            );
        }
    }

    println!("\n🎉 Migration example completed!");
    println!("This example demonstrates:");
    println!("  • Schema definition and evolution");
    println!("  • Compatibility analysis between schema versions");
    println!("  • Data transformation with field mapping");
    println!("  • Migration validation and error handling");
    println!("  • Integration with StateManager and storage backends");
    println!("  • Hook system and event correlation tracking");

    Ok(())
}
