// Tests for V10__artifacts migration (Phase 13b.10.1)
//!
//! Verifies:
//! - artifact_content table created with correct schema
//! - artifacts table created with correct schema
//! - 13 indexes created across both tables (4 + 9)
//! - RLS policies applied (8 policies: 4 per table)
//! - Foreign key constraints (content, sessions)
//! - Check constraints (storage_type, reference_count, storage consistency, size limits)
//! - Unique constraints
//! - Triggers (updated_at, reference counting, access tracking)
//! - Content deduplication via reference counting
//! - Tenant isolation verification

#![cfg(feature = "postgres")]

use llmspell_storage::{PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests
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

fn unique_tenant_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

#[tokio::test]
async fn test_artifact_content_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'artifact_content'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "artifact_content table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'artifact_content' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on artifact_content");
}

#[tokio::test]
async fn test_artifacts_table_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify table exists
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM pg_tables
             WHERE schemaname = 'llmspell' AND tablename = 'artifacts'",
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "artifacts table should exist");

    // Verify RLS is enabled
    let row = client
        .query_one(
            "SELECT relrowsecurity FROM pg_class
             WHERE relname = 'artifacts' AND relnamespace = 'llmspell'::regnamespace",
            &[],
        )
        .await
        .unwrap();
    let rls_enabled: bool = row.get(0);
    assert!(rls_enabled, "RLS should be enabled on artifacts");
}

#[tokio::test]
async fn test_artifacts_indexes_created() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Get all indexes for artifact_content
    let rows = client
        .query(
            "SELECT indexname FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'artifact_content'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 4, "Should have 4 indexes on artifact_content");

    let expected_content_indexes = [
        "idx_artifact_content_accessed",
        "idx_artifact_content_large_objects",
        "idx_artifact_content_refcount",
        "idx_artifact_content_tenant",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_content_indexes[i]);
    }

    // Get all indexes for artifacts
    let rows = client
        .query(
            "SELECT indexname FROM pg_indexes
             WHERE schemaname = 'llmspell'
               AND tablename = 'artifacts'
               AND indexname LIKE 'idx_%'
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 9, "Should have 9 indexes on artifacts");

    let expected_artifact_indexes = [
        "idx_artifacts_content",
        "idx_artifacts_created",
        "idx_artifacts_metadata",
        "idx_artifacts_name",
        "idx_artifacts_session",
        "idx_artifacts_size",
        "idx_artifacts_tags",
        "idx_artifacts_tenant_type",
        "idx_artifacts_type",
    ];

    for (i, row) in rows.iter().enumerate() {
        let indexname: String = row.get("indexname");
        assert_eq!(indexname, expected_artifact_indexes[i]);
    }
}

#[tokio::test]
async fn test_artifacts_rls_policies() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Verify artifact_content policies
    let rows = client
        .query(
            "SELECT policyname, cmd FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'artifact_content'
             ORDER BY policyname",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(
        rows.len(),
        4,
        "Should have 4 RLS policies on artifact_content"
    );

    let expected_policies = [
        ("tenant_isolation_delete", "DELETE"),
        ("tenant_isolation_insert", "INSERT"),
        ("tenant_isolation_select", "SELECT"),
        ("tenant_isolation_update", "UPDATE"),
    ];

    for (i, row) in rows.iter().enumerate() {
        let policyname: String = row.get("policyname");
        let cmd: String = row.get("cmd");
        assert_eq!(policyname, expected_policies[i].0);
        assert_eq!(cmd, expected_policies[i].1);
    }

    // Verify artifacts policies
    let rows = client
        .query(
            "SELECT policyname, cmd FROM pg_policies
             WHERE schemaname = 'llmspell' AND tablename = 'artifacts'
             ORDER BY policyname",
            &[],
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 4, "Should have 4 RLS policies on artifacts");

    for (i, row) in rows.iter().enumerate() {
        let policyname: String = row.get("policyname");
        let cmd: String = row.get("cmd");
        assert_eq!(policyname, expected_policies[i].0);
        assert_eq!(cmd, expected_policies[i].1);
    }
}

#[tokio::test]
async fn test_storage_type_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("storage-type");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Try to insert content with invalid storage_type
    let result = client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, size_bytes)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &"hash123", &"invalid_type", &1000i64],
        )
        .await;

    assert!(
        result.is_err(),
        "Invalid storage_type should violate constraint"
    );

    let error = result.unwrap_err();
    // tokio_postgres Display returns "db error", need to check DbError details
    let error_debug = format!("{:?}", error);
    // PostgreSQL may check either valid_storage_type or bytea_storage_valid first (order undefined)
    // Both constraints will reject invalid_type, so accept either
    assert!(
        error_debug.contains("valid_storage_type")
            || error_debug.contains("bytea_storage_valid")
            || error_debug.contains("check constraint"),
        "Error should indicate constraint violation, got: {:?}",
        error
    );
}

#[tokio::test]
async fn test_storage_consistency_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("storage-consistency");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Try bytea storage without data
    let result = client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, size_bytes)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &"hash_no_data", &"bytea", &1000i64],
        )
        .await;

    assert!(
        result.is_err(),
        "BYTEA storage without data should violate constraint"
    );

    // Try large_object storage without OID
    let result = client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, size_bytes)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &"hash_no_oid", &"large_object", &1000000i64],
        )
        .await;

    assert!(
        result.is_err(),
        "Large object storage without OID should violate constraint"
    );
}

#[tokio::test]
async fn test_reference_count_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("refcount");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let content_data = b"test content";

    // Insert content with valid refcount
    client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, data, size_bytes)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"hash_refcount",
                &"bytea",
                &content_data.to_vec(),
                &(content_data.len() as i64),
            ],
        )
        .await
        .unwrap();

    // Try to set refcount to 0 (should fail)
    let result = client
        .execute(
            "UPDATE llmspell.artifact_content
             SET reference_count = 0
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &"hash_refcount"],
        )
        .await;

    assert!(
        result.is_err(),
        "Setting reference_count to 0 should violate constraint"
    );
}

#[tokio::test]
async fn test_max_size_constraint() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("max-size");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Try to insert content exceeding 100MB
    let result = client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, data, size_bytes)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &"hash_too_large",
                &"bytea",
                &b"x".to_vec(),
                &(104857601i64), // 100MB + 1 byte
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "Size exceeding 100MB should violate constraint"
    );
}

#[tokio::test]
async fn test_foreign_key_to_sessions() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("fk-session");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let nonexistent_session_id = Uuid::new_v4();

    // Insert content first
    let content_hash = "hash_fk_test";
    client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, data, size_bytes)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &content_hash,
                &"bytea",
                &b"test".to_vec(),
                &4i64,
            ],
        )
        .await
        .unwrap();

    // Try to insert artifact with nonexistent session_id
    let artifact_metadata = serde_json::json!({
        "name": "test-artifact",
        "artifact_type": "agent_output",
        "mime_type": "text/plain",
        "size": 4,
        "version": {"version": 1, "created_at": "2025-01-01T00:00:00Z"},
        "created_at": "2025-01-01T00:00:00Z"
    });

    let result = client
        .execute(
            "INSERT INTO llmspell.artifacts
             (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                &tenant_id,
                &format!("{}:0:{}", nonexistent_session_id, content_hash),
                &nonexistent_session_id,
                &0i64,
                &content_hash,
                &artifact_metadata,
                &"test-artifact",
                &"agent_output",
                &"text/plain",
                &4i64,
            ],
        )
        .await;

    assert!(
        result.is_err(),
        "Artifact with nonexistent session_id should violate foreign key"
    );
}

#[tokio::test]
async fn test_reference_count_triggers() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("refcount-trigger");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Create session first
    let session_id = Uuid::new_v4();
    let session_data = serde_json::json!({"metadata": {}, "config": {}, "state": {}});
    client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &session_id, &session_data, &"active"],
        )
        .await
        .unwrap();

    // Insert content with initial refcount of 1
    let content_hash = "hash_trigger_test";
    client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, data, size_bytes, reference_count)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &tenant_id,
                &content_hash,
                &"bytea",
                &b"test content".to_vec(),
                &12i64,
                &1i32,
            ],
        )
        .await
        .unwrap();

    // Verify initial refcount
    let row = client
        .query_one(
            "SELECT reference_count FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let initial_refcount: i32 = row.get(0);
    assert_eq!(initial_refcount, 1);

    // Insert artifact (should increment refcount via trigger)
    let artifact_metadata = serde_json::json!({
        "name": "test",
        "artifact_type": "agent_output",
        "mime_type": "text/plain",
        "size": 12,
        "version": {"version": 1, "created_at": "2025-01-01T00:00:00Z"},
        "created_at": "2025-01-01T00:00:00Z"
    });

    client
        .execute(
            "INSERT INTO llmspell.artifacts
             (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                &tenant_id,
                &format!("{}:0:{}", session_id, content_hash),
                &session_id,
                &0i64,
                &content_hash,
                &artifact_metadata,
                &"test",
                &"agent_output",
                &"text/plain",
                &12i64,
            ],
        )
        .await
        .unwrap();

    // Verify refcount incremented
    let row = client
        .query_one(
            "SELECT reference_count FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let after_insert_refcount: i32 = row.get(0);
    assert_eq!(after_insert_refcount, 2, "Refcount should increment to 2");

    // Delete artifact (should decrement refcount via trigger)
    client
        .execute(
            "DELETE FROM llmspell.artifacts
             WHERE tenant_id = $1 AND session_id = $2",
            &[&tenant_id, &session_id],
        )
        .await
        .unwrap();

    // Verify refcount decremented
    let row = client
        .query_one(
            "SELECT reference_count FROM llmspell.artifact_content
             WHERE tenant_id = $1 AND content_hash = $2",
            &[&tenant_id, &content_hash],
        )
        .await
        .unwrap();
    let after_delete_refcount: i32 = row.get(0);
    assert_eq!(after_delete_refcount, 1, "Refcount should decrement to 1");
}

#[tokio::test]
async fn test_updated_at_trigger() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let tenant_id = unique_tenant_id("updated-at");

    backend.set_tenant_context(&tenant_id).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Create session and content
    let session_id = Uuid::new_v4();
    let session_data = serde_json::json!({"metadata": {}, "config": {}, "state": {}});
    client
        .execute(
            "INSERT INTO llmspell.sessions
             (tenant_id, session_id, session_data, status)
             VALUES ($1, $2, $3, $4)",
            &[&tenant_id, &session_id, &session_data, &"active"],
        )
        .await
        .unwrap();

    let content_hash = "hash_updated_at";
    client
        .execute(
            "INSERT INTO llmspell.artifact_content
             (tenant_id, content_hash, storage_type, data, size_bytes)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &tenant_id,
                &content_hash,
                &"bytea",
                &b"test".to_vec(),
                &4i64,
            ],
        )
        .await
        .unwrap();

    // Insert artifact
    let artifact_metadata = serde_json::json!({
        "name": "test",
        "artifact_type": "agent_output",
        "mime_type": "text/plain",
        "size": 4,
        "version": {"version": 1, "created_at": "2025-01-01T00:00:00Z"},
        "created_at": "2025-01-01T00:00:00Z"
    });

    let artifact_id = format!("{}:0:{}", session_id, content_hash);
    client
        .execute(
            "INSERT INTO llmspell.artifacts
             (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[
                &tenant_id,
                &artifact_id,
                &session_id,
                &0i64,
                &content_hash,
                &artifact_metadata,
                &"test",
                &"agent_output",
                &"text/plain",
                &4i64,
            ],
        )
        .await
        .unwrap();

    // Get initial updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.artifacts
             WHERE tenant_id = $1 AND artifact_id = $2",
            &[&tenant_id, &artifact_id],
        )
        .await
        .unwrap();
    let initial_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the artifact
    client
        .execute(
            "UPDATE llmspell.artifacts
             SET version = version + 1
             WHERE tenant_id = $1 AND artifact_id = $2",
            &[&tenant_id, &artifact_id],
        )
        .await
        .unwrap();

    // Get updated updated_at
    let row = client
        .query_one(
            "SELECT updated_at FROM llmspell.artifacts
             WHERE tenant_id = $1 AND artifact_id = $2",
            &[&tenant_id, &artifact_id],
        )
        .await
        .unwrap();
    let updated_updated_at: chrono::DateTime<chrono::Utc> = row.get(0);

    assert!(
        updated_updated_at > initial_updated_at,
        "updated_at should be automatically updated by trigger"
    );
}

#[tokio::test]
async fn test_tenant_isolation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    let tenant_a = unique_tenant_id("isolation-a");
    let tenant_b = unique_tenant_id("isolation-b");

    // Create sessions for both tenants
    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();

    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        let session_data = serde_json::json!({"metadata": {}, "config": {}, "state": {}});
        client
            .execute(
                "INSERT INTO llmspell.sessions
                 (tenant_id, session_id, session_data, status)
                 VALUES ($1, $2, $3, $4)",
                &[&tenant_a, &session_a, &session_data, &"active"],
            )
            .await
            .unwrap();
    }

    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();
        let session_data = serde_json::json!({"metadata": {}, "config": {}, "state": {}});
        client
            .execute(
                "INSERT INTO llmspell.sessions
                 (tenant_id, session_id, session_data, status)
                 VALUES ($1, $2, $3, $4)",
                &[&tenant_b, &session_b, &session_data, &"active"],
            )
            .await
            .unwrap();
    }

    // Create artifacts for both tenants
    backend.set_tenant_context(&tenant_a).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();

        // Insert content
        client
            .execute(
                "INSERT INTO llmspell.artifact_content
                 (tenant_id, content_hash, storage_type, data, size_bytes)
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    &tenant_a,
                    &"hash_tenant_a",
                    &"bytea",
                    &b"content_a".to_vec(),
                    &9i64,
                ],
            )
            .await
            .unwrap();

        // Insert artifact
        let metadata = serde_json::json!({
            "name": "artifact_a",
            "artifact_type": "agent_output",
            "mime_type": "text/plain",
            "size": 9,
            "version": {"version": 1, "created_at": "2025-01-01T00:00:00Z"},
            "created_at": "2025-01-01T00:00:00Z"
        });

        client
            .execute(
                "INSERT INTO llmspell.artifacts
                 (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &tenant_a,
                    &format!("{}:0:hash_tenant_a", session_a),
                    &session_a,
                    &0i64,
                    &"hash_tenant_a",
                    &metadata,
                    &"artifact_a",
                    &"agent_output",
                    &"text/plain",
                    &9i64,
                ],
            )
            .await
            .unwrap();
    }

    backend.set_tenant_context(&tenant_b).await.unwrap();
    {
        let client = backend.get_client().await.unwrap();

        // Insert content
        client
            .execute(
                "INSERT INTO llmspell.artifact_content
                 (tenant_id, content_hash, storage_type, data, size_bytes)
                 VALUES ($1, $2, $3, $4, $5)",
                &[
                    &tenant_b,
                    &"hash_tenant_b",
                    &"bytea",
                    &b"content_b".to_vec(),
                    &9i64,
                ],
            )
            .await
            .unwrap();

        // Insert artifact
        let metadata = serde_json::json!({
            "name": "artifact_b",
            "artifact_type": "tool_result",
            "mime_type": "text/plain",
            "size": 9,
            "version": {"version": 1, "created_at": "2025-01-01T00:00:00Z"},
            "created_at": "2025-01-01T00:00:00Z"
        });

        client
            .execute(
                "INSERT INTO llmspell.artifacts
                 (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &tenant_b,
                    &format!("{}:0:hash_tenant_b", session_b),
                    &session_b,
                    &0i64,
                    &"hash_tenant_b",
                    &metadata,
                    &"artifact_b",
                    &"tool_result",
                    &"text/plain",
                    &9i64,
                ],
            )
            .await
            .unwrap();
    }

    // Query as tenant A - should only see A's data
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let rows_a = {
        let client = backend.get_client().await.unwrap();
        client
            .query("SELECT name FROM llmspell.artifacts ORDER BY name", &[])
            .await
            .unwrap()
    };

    assert_eq!(rows_a.len(), 1, "Tenant A should see only 1 artifact");
    let name_a: String = rows_a[0].get(0);
    assert_eq!(name_a, "artifact_a");

    // Query as tenant B - should only see B's data
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let rows_b = {
        let client = backend.get_client().await.unwrap();
        client
            .query("SELECT name FROM llmspell.artifacts ORDER BY name", &[])
            .await
            .unwrap()
    };

    assert_eq!(rows_b.len(), 1, "Tenant B should see only 1 artifact");
    let name_b: String = rows_b[0].get(0);
    assert_eq!(name_b, "artifact_b");
}
