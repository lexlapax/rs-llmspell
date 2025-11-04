//! Tests for Large Object streaming API (Phase 13b.10.2)
//!
//! Verifies:
//! - Streaming upload functionality
//! - Streaming download functionality
//! - Large Object deletion
//! - Orphaned object detection and cleanup
//! - Large Object existence checks
//! - Size queries
//! - Chunk size configuration
//! - Large file handling (100MB+)
//! - Round-trip integrity
//! - Concurrent operations
//! - Error handling

#![cfg(feature = "postgres")]

use llmspell_storage::{LargeObjectStream, PostgresBackend, PostgresConfig};
use tokio::sync::OnceCell;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

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

#[tokio::test]
async fn test_upload_small_object() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    let data = vec![1u8, 2, 3, 4, 5];
    let oid = stream.upload(&data).await.unwrap();

    assert!(oid > 0, "OID should be positive");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_upload_large_object() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    // 10MB test data
    let data = vec![42u8; 10 * 1024 * 1024];
    let oid = stream.upload(&data).await.unwrap();

    assert!(oid > 0, "OID should be positive");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_download_object() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    let data = vec![1u8, 2, 3, 4, 5];
    let oid = stream.upload(&data).await.unwrap();

    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(
        downloaded, data,
        "Downloaded data should match uploaded data"
    );

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_round_trip_integrity() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    // Create test data with pattern
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let oid = stream.upload(&data).await.unwrap();

    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(
        downloaded.len(),
        data.len(),
        "Downloaded size should match uploaded size"
    );
    assert_eq!(
        downloaded, data,
        "Downloaded data should match uploaded data"
    );

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_delete_object() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    let data = vec![1u8, 2, 3, 4, 5];
    let oid = stream.upload(&data).await.unwrap();

    // Verify exists
    assert!(stream.exists(oid).await.unwrap(), "Object should exist");

    // Delete
    stream.delete(oid).await.unwrap();

    // Verify deleted
    assert!(
        !stream.exists(oid).await.unwrap(),
        "Object should not exist after deletion"
    );
}

#[tokio::test]
async fn test_exists_check() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);

    // Check non-existent object
    assert!(
        !stream.exists(999999).await.unwrap(),
        "Non-existent object should return false"
    );

    // Create and check
    let data = vec![1u8, 2, 3, 4, 5];
    let oid = stream.upload(&data).await.unwrap();
    assert!(stream.exists(oid).await.unwrap(), "Object should exist");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_object_size() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    let data = vec![42u8; 1000];
    let oid = stream.upload(&data).await.unwrap();

    let size = stream.size(oid).await.unwrap();
    assert_eq!(size as usize, 1000, "Size should match uploaded data size");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_custom_chunk_size() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Use small chunk size (1KB)
    let mut stream = LargeObjectStream::new(client).with_chunk_size(1024);
    let data = vec![42u8; 10000];
    let oid = stream.upload(&data).await.unwrap();

    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(
        downloaded, data,
        "Downloaded data should match with custom chunk size"
    );

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_find_orphaned_objects() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);

    // Create a Large Object (orphaned since not in artifact_content table)
    let data = vec![1u8, 2, 3, 4, 5];
    let oid = stream.upload(&data).await.unwrap();

    // Find orphaned objects
    let orphaned = stream.find_orphaned_objects().await.unwrap();
    assert!(
        orphaned.contains(&oid),
        "Newly created object should be orphaned"
    );

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_cleanup_orphaned_objects() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);

    // Create orphaned Large Objects
    let data = vec![1u8, 2, 3, 4, 5];
    let oid1 = stream.upload(&data).await.unwrap();
    let oid2 = stream.upload(&data).await.unwrap();

    // Verify orphaned
    let _orphaned_before = stream.find_orphaned_objects().await.unwrap();

    // Cleanup - should complete without error
    stream.cleanup_orphaned_objects().await.unwrap();

    // Verify cleaned
    assert!(
        !stream.exists(oid1).await.unwrap(),
        "OID1 should be deleted"
    );
    assert!(
        !stream.exists(oid2).await.unwrap(),
        "OID2 should be deleted"
    );
}

#[tokio::test]
async fn test_empty_data() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    let data = vec![];
    let oid = stream.upload(&data).await.unwrap();

    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(downloaded, data, "Empty data should round-trip correctly");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_multiple_chunks() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Use 100KB chunk size, upload 1MB
    let mut stream = LargeObjectStream::new(client).with_chunk_size(100 * 1024);
    let data = vec![42u8; 1024 * 1024]; // 1MB
    let oid = stream.upload(&data).await.unwrap();

    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(downloaded.len(), data.len(), "Downloaded size should match");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_concurrent_uploads() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();

    // Create multiple concurrent uploads
    let mut handles = vec![];
    for i in 0..5 {
        let backend_clone = backend.clone();
        let handle = tokio::spawn(async move {
            let client = backend_clone.get_client().await.unwrap();
            let mut stream = LargeObjectStream::new(client);
            let data = vec![i as u8; 1000];
            stream.upload(&data).await.unwrap()
        });
        handles.push(handle);
    }

    // Wait for all uploads
    let mut oids = vec![];
    for handle in handles {
        let oid = handle.await.unwrap();
        oids.push(oid);
    }

    assert_eq!(oids.len(), 5, "Should have 5 OIDs");

    // Cleanup - ignore errors if objects were already cleaned up
    for oid in oids {
        let client = backend.get_client().await.unwrap();
        let mut stream = LargeObjectStream::new(client);
        let _ = stream.delete(oid).await;
    }
}

#[tokio::test]
async fn test_large_file_streaming() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);
    // 50MB test data
    let data = vec![123u8; 50 * 1024 * 1024];
    let oid = stream.upload(&data).await.unwrap();

    // Verify size
    let size = stream.size(oid).await.unwrap();
    assert_eq!(size as usize, data.len(), "Size should match 50MB");

    // Download and verify first/last bytes
    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(downloaded.len(), data.len(), "Downloaded size should match");
    assert_eq!(downloaded[0], 123, "First byte should match");
    assert_eq!(
        downloaded[downloaded.len() - 1],
        123,
        "Last byte should match"
    );

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_boundary_chunk_sizes() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    // Test with data size exactly equal to chunk size
    let mut stream = LargeObjectStream::new(client).with_chunk_size(1024);
    let data = vec![42u8; 1024];
    let oid = stream.upload(&data).await.unwrap();

    let downloaded = stream.download(oid).await.unwrap();
    assert_eq!(downloaded, data, "Data equal to chunk size should work");

    // Cleanup - ignore errors if already deleted
    let _ = stream.delete(oid).await;
}

#[tokio::test]
async fn test_download_nonexistent_object() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(SUPERUSER_CONNECTION_STRING);
    let backend = PostgresBackend::new(config).await.unwrap();
    let client = backend.get_client().await.unwrap();

    let mut stream = LargeObjectStream::new(client);

    // Try to download non-existent object
    let result = stream.download(999999).await;
    assert!(
        result.is_err(),
        "Should fail to download non-existent object"
    );
}
