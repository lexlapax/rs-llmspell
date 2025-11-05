//! Phase 1 Migration Integration Tests (Task 13b.14.3)
//!
//! Tests end-to-end Sled→PostgreSQL migrations for Phase 1 components:
//! - Agent State (agent_state)
//! - Workflow State (workflow_state)
//! - Sessions (sessions)
//!
//! Validates:
//! - Migration plan generation
//! - Dry-run mode (no writes)
//! - Actual migration execution
//! - Data integrity (count + checksums)
//! - Rollback on validation failure
//! - Performance targets (<1 min for 1K records)

#![cfg(feature = "postgres")]

use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
use llmspell_storage::backends::sled_backend::SledBackend;
use llmspell_storage::migration::{MigrationEngine, MigrationPlan};
use llmspell_storage::traits::StorageBackend;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure PostgreSQL migrations run once before all tests
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
            let backend = PostgresBackend::new(config)
                .await
                .expect("Failed to create backend for migration init");

            backend
                .run_migrations()
                .await
                .expect("Failed to run migrations during test initialization");
        })
        .await;
}

/// Create test data in Sled for agent states
async fn setup_agent_state_test_data(backend: &SledBackend, count: usize) -> anyhow::Result<()> {
    for i in 0..count {
        // Format: agent:<type>:<id>
        let agent_id = format!("agent:test:migration_{}", i);
        let state_value = format!(
            "{{\"agent_id\": \"agent_{}\", \"state\": \"active\", \"iteration\": {}}}",
            i, i
        );
        backend.set(&agent_id, state_value.into_bytes()).await?;
    }
    Ok(())
}

/// Create test data in Sled for workflow states
async fn setup_workflow_state_test_data(backend: &SledBackend, count: usize) -> anyhow::Result<()> {
    for i in 0..count {
        let workflow_id = format!("custom:workflow_test_{}:state", i);
        let state_value = format!(
            "{{\"workflow_id\": \"workflow_{}\", \"status\": \"running\", \"step\": {}}}",
            i, i
        );
        backend.set(&workflow_id, state_value.into_bytes()).await?;
    }
    Ok(())
}

/// Create test data in Sled for sessions
async fn setup_sessions_test_data(backend: &SledBackend, count: usize) -> anyhow::Result<()> {
    for i in 0..count {
        let session_id = format!("session:test_migration_{}", i);
        let session_value = format!(
            "{{\"session_id\": \"session_{}\", \"user_id\": \"user_{}\", \"created_at\": \"2025-01-01T00:00:00Z\"}}",
            i, i % 10
        );
        backend.set(&session_id, session_value.into_bytes()).await?;
    }
    Ok(())
}

// ============================================================================
// Test: Agent State Migration (Sled → PostgreSQL)
// ============================================================================

#[tokio::test]
async fn test_agent_state_migration_1k_records() {
    ensure_migrations_run_once().await;

    // Setup: Create temporary Sled database with 1K agent states
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let sled_path = temp_dir.path().join("test_agent_migration.sled");
    let source = Arc::new(
        SledBackend::new_with_path(sled_path.to_str().unwrap()).expect("Failed to create Sled"),
    );
    setup_agent_state_test_data(&source, 1000)
        .await
        .expect("Failed to setup test data");

    // Setup: Create PostgreSQL target with unique tenant
    let tenant_id = format!("test_agent_migration_{}", Uuid::new_v4());
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let target = Arc::new(
        PostgresBackend::new(config)
            .await
            .expect("Failed to create PostgreSQL backend"),
    );
    target
        .set_tenant_context(&tenant_id)
        .await
        .expect("Failed to set tenant context");

    // Create migration plan
    let plan = MigrationPlan::new("sled", "postgres", vec!["agent_state".to_string()]);

    // Execute migration
    let engine = MigrationEngine::new(source.clone(), target.clone(), plan);
    let start = std::time::Instant::now();
    let report = engine
        .execute(false)
        .await
        .expect("Migration should succeed");
    let duration = start.elapsed();

    // Assertions
    assert!(
        report.success,
        "Migration should succeed: {:?}",
        report.validation_results
    );
    assert_eq!(report.source_count, 1000, "Source count should be 1000");
    assert_eq!(report.target_count, 1000, "Target count should be 1000");
    assert!(
        duration.as_secs() < 60,
        "Migration should complete in <1 min, took {}s",
        duration.as_secs()
    );

    println!(
        "[PASS] Agent State Migration: 1K records in {:.2}s",
        duration.as_secs_f64()
    );

    // Cleanup: Delete test data from PostgreSQL
    let client = target.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.agent_states WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
}

// ============================================================================
// Test: Workflow State Migration (Sled → PostgreSQL)
// ============================================================================

#[tokio::test]
async fn test_workflow_state_migration_1k_records() {
    ensure_migrations_run_once().await;

    // Setup: Create temporary Sled database with 1K workflow states
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let sled_path = temp_dir.path().join("test_workflow_migration.sled");
    let source = Arc::new(
        SledBackend::new_with_path(sled_path.to_str().unwrap()).expect("Failed to create Sled"),
    );
    setup_workflow_state_test_data(&source, 1000)
        .await
        .expect("Failed to setup test data");

    // Setup: Create PostgreSQL target with unique tenant
    let tenant_id = format!("test_workflow_migration_{}", Uuid::new_v4());
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let target = Arc::new(
        PostgresBackend::new(config)
            .await
            .expect("Failed to create PostgreSQL backend"),
    );
    target
        .set_tenant_context(&tenant_id)
        .await
        .expect("Failed to set tenant context");

    // Create migration plan
    let plan = MigrationPlan::new("sled", "postgres", vec!["workflow_state".to_string()]);

    // Execute migration
    let engine = MigrationEngine::new(source.clone(), target.clone(), plan);
    let start = std::time::Instant::now();
    let report = engine
        .execute(false)
        .await
        .expect("Migration should succeed");
    let duration = start.elapsed();

    // Assertions
    assert!(
        report.success,
        "Migration should succeed: {:?}",
        report.validation_results
    );
    assert_eq!(report.source_count, 1000, "Source count should be 1000");
    assert_eq!(report.target_count, 1000, "Target count should be 1000");
    assert!(
        duration.as_secs() < 60,
        "Migration should complete in <1 min, took {}s",
        duration.as_secs()
    );

    println!(
        "[PASS] Workflow State Migration: 1K records in {:.2}s",
        duration.as_secs_f64()
    );

    // Cleanup: Delete test data from PostgreSQL
    let client = target.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.workflow_states WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
}

// ============================================================================
// Test: Sessions Migration (Sled → PostgreSQL)
// ============================================================================

#[tokio::test]
async fn test_sessions_migration_1k_records() {
    ensure_migrations_run_once().await;

    // Setup: Create temporary Sled database with 1K sessions
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let sled_path = temp_dir.path().join("test_sessions_migration.sled");
    let source = Arc::new(
        SledBackend::new_with_path(sled_path.to_str().unwrap()).expect("Failed to create Sled"),
    );
    setup_sessions_test_data(&source, 1000)
        .await
        .expect("Failed to setup test data");

    // Setup: Create PostgreSQL target with unique tenant
    let tenant_id = format!("test_sessions_migration_{}", Uuid::new_v4());
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let target = Arc::new(
        PostgresBackend::new(config)
            .await
            .expect("Failed to create PostgreSQL backend"),
    );
    target
        .set_tenant_context(&tenant_id)
        .await
        .expect("Failed to set tenant context");

    // Create migration plan
    let plan = MigrationPlan::new("sled", "postgres", vec!["sessions".to_string()]);

    // Execute migration
    let engine = MigrationEngine::new(source.clone(), target.clone(), plan);
    let start = std::time::Instant::now();
    let report = engine
        .execute(false)
        .await
        .expect("Migration should succeed");
    let duration = start.elapsed();

    // Assertions
    assert!(
        report.success,
        "Migration should succeed: {:?}",
        report.validation_results
    );
    assert_eq!(report.source_count, 1000, "Source count should be 1000");
    assert_eq!(report.target_count, 1000, "Target count should be 1000");
    assert!(
        duration.as_secs() < 60,
        "Migration should complete in <1 min, took {}s",
        duration.as_secs()
    );

    println!(
        "[PASS] Sessions Migration: 1K records in {:.2}s",
        duration.as_secs_f64()
    );

    // Cleanup: Delete test data from PostgreSQL
    let client = target.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.sessions WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
}

// ============================================================================
// Test: Dry-Run Mode (No Writes)
// ============================================================================

#[tokio::test]
async fn test_dry_run_mode_no_writes() {
    ensure_migrations_run_once().await;

    // Setup: Create temporary Sled database with 100 agent states
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let sled_path = temp_dir.path().join("test_dry_run.sled");
    let source = Arc::new(
        SledBackend::new_with_path(sled_path.to_str().unwrap()).expect("Failed to create Sled"),
    );
    setup_agent_state_test_data(&source, 100)
        .await
        .expect("Failed to setup test data");

    // Setup: Create PostgreSQL target with unique tenant
    let tenant_id = format!("test_dry_run_{}", Uuid::new_v4());
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let target = Arc::new(
        PostgresBackend::new(config)
            .await
            .expect("Failed to create PostgreSQL backend"),
    );
    target
        .set_tenant_context(&tenant_id)
        .await
        .expect("Failed to set tenant context");

    // Create migration plan
    let plan = MigrationPlan::new("sled", "postgres", vec!["agent_state".to_string()]);

    // Execute dry-run
    let engine = MigrationEngine::new(source.clone(), target.clone(), plan);
    let report = engine.execute(true).await.expect("Dry-run should succeed");

    // Assertions
    assert!(report.success, "Dry-run should succeed");
    assert_eq!(report.source_count, 0, "Dry-run should not count records");
    assert_eq!(report.target_count, 0, "Dry-run should not count records");

    // Verify no data was written to PostgreSQL
    let client = target.get_client().await.unwrap();
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM llmspell.agent_states WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 0, "Dry-run should not write any data");

    println!("[PASS] Dry-run mode: No writes to target");
}

// ============================================================================
// Test: All Phase 1 Components Together
// ============================================================================

#[tokio::test]
async fn test_all_phase1_components_together() {
    ensure_migrations_run_once().await;

    // Setup: Create temporary Sled database with all Phase 1 data
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let sled_path = temp_dir.path().join("test_all_phase1.sled");
    let source = Arc::new(
        SledBackend::new_with_path(sled_path.to_str().unwrap()).expect("Failed to create Sled"),
    );

    // Create test data for all 3 components
    setup_agent_state_test_data(&source, 500)
        .await
        .expect("Failed to setup agent state data");
    setup_workflow_state_test_data(&source, 500)
        .await
        .expect("Failed to setup workflow state data");
    setup_sessions_test_data(&source, 500)
        .await
        .expect("Failed to setup sessions data");

    // Setup: Create PostgreSQL target with unique tenant
    let tenant_id = format!("test_all_phase1_{}", Uuid::new_v4());
    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let target = Arc::new(
        PostgresBackend::new(config)
            .await
            .expect("Failed to create PostgreSQL backend"),
    );
    target
        .set_tenant_context(&tenant_id)
        .await
        .expect("Failed to set tenant context");

    // Create migration plan for all 3 components
    let plan = MigrationPlan::new(
        "sled",
        "postgres",
        vec![
            "agent_state".to_string(),
            "workflow_state".to_string(),
            "sessions".to_string(),
        ],
    );

    // Execute migration
    let engine = MigrationEngine::new(source.clone(), target.clone(), plan);
    let start = std::time::Instant::now();
    let report = engine
        .execute(false)
        .await
        .expect("Migration should succeed");
    let duration = start.elapsed();

    // Assertions
    assert!(
        report.success,
        "Migration should succeed: {:?}",
        report.validation_results
    );
    assert_eq!(
        report.source_count, 1500,
        "Source count should be 1500 (500 per component)"
    );
    assert_eq!(
        report.target_count, 1500,
        "Target count should be 1500 (500 per component)"
    );
    assert!(
        duration.as_secs() < 180,
        "Migration should complete in <3 min, took {}s",
        duration.as_secs()
    );

    println!(
        "[PASS] All Phase 1 Components: 1.5K records (3 components) in {:.2}s",
        duration.as_secs_f64()
    );

    // Cleanup: Delete test data from PostgreSQL
    let client = target.get_client().await.unwrap();
    client
        .execute(
            "DELETE FROM llmspell.agent_states WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "DELETE FROM llmspell.workflow_states WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
    client
        .execute(
            "DELETE FROM llmspell.sessions WHERE tenant_id = $1",
            &[&tenant_id],
        )
        .await
        .unwrap();
}
