//! Tests for PostgreSQL Procedural Pattern Storage (Phase 13b.6.2)
//!
//! Verifies:
//! - Pattern recording and frequency tracking
//! - Learned pattern retrieval with filtering
//! - Tenant isolation
//! - Timestamp accuracy
//! - Performance (<10ms target)
//! - Concurrent updates

#![cfg(feature = "postgres")]

use llmspell_storage::backends::postgres::{
    PostgresBackend, PostgresConfig, PostgresProceduralStorage,
};
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

const SUPERUSER_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_dev_pass@localhost:5432/llmspell_dev";

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
async fn test_record_transition_creates_new_pattern() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("record-new");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record first transition
    let freq = storage
        .record_transition("global", "theme", "dark")
        .await
        .unwrap();

    assert_eq!(freq, 1, "First transition should have frequency 1");
}

#[tokio::test]
async fn test_record_transition_increments_frequency() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("record-increment");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record same transition multiple times
    let freq1 = storage
        .record_transition("global", "theme", "dark")
        .await
        .unwrap();
    let freq2 = storage
        .record_transition("global", "theme", "dark")
        .await
        .unwrap();
    let freq3 = storage
        .record_transition("global", "theme", "dark")
        .await
        .unwrap();

    assert_eq!(freq1, 1);
    assert_eq!(freq2, 2);
    assert_eq!(freq3, 3);
}

#[tokio::test]
async fn test_get_pattern_frequency_returns_correct_value() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("get-freq");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record pattern 5 times
    for _ in 0..5 {
        storage
            .record_transition("session:test", "config.lang", "en")
            .await
            .unwrap();
    }

    let freq = storage
        .get_pattern_frequency("session:test", "config.lang", "en")
        .await
        .unwrap();

    assert_eq!(freq, 5);
}

#[tokio::test]
async fn test_get_pattern_frequency_returns_zero_for_nonexistent() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("get-freq-zero");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    let freq = storage
        .get_pattern_frequency("global", "nonexistent", "value")
        .await
        .unwrap();

    assert_eq!(freq, 0);
}

#[tokio::test]
async fn test_get_learned_patterns_with_threshold() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("learned-threshold");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Create patterns with different frequencies
    for _ in 0..2 {
        storage
            .record_transition("global", "pattern1", "value1")
            .await
            .unwrap();
    }
    for _ in 0..3 {
        storage
            .record_transition("global", "pattern2", "value2")
            .await
            .unwrap();
    }
    for _ in 0..5 {
        storage
            .record_transition("global", "pattern3", "value3")
            .await
            .unwrap();
    }

    // Get patterns with frequency >= 3
    let patterns = storage.get_learned_patterns(3).await.unwrap();

    assert_eq!(
        patterns.len(),
        2,
        "Should have 2 patterns with frequency >= 3"
    );

    // Verify they're the right patterns (sorted by frequency descending)
    assert_eq!(patterns[0].key, "pattern3");
    assert_eq!(patterns[0].frequency, 5);
    assert_eq!(patterns[1].key, "pattern2");
    assert_eq!(patterns[1].frequency, 3);
}

#[tokio::test]
async fn test_get_learned_patterns_empty_when_none_match() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("learned-empty");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Create pattern with frequency 2
    for _ in 0..2 {
        storage
            .record_transition("global", "pattern", "value")
            .await
            .unwrap();
    }

    // Query for frequency >= 3
    let patterns = storage.get_learned_patterns(3).await.unwrap();

    assert_eq!(patterns.len(), 0, "Should have no patterns with freq >= 3");
}

#[tokio::test]
async fn test_get_learned_patterns_ordered_by_frequency() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("learned-order");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Create patterns with different frequencies (out of order)
    for _ in 0..3 {
        storage
            .record_transition("global", "pattern-mid", "mid")
            .await
            .unwrap();
    }
    for _ in 0..10 {
        storage
            .record_transition("global", "pattern-high", "high")
            .await
            .unwrap();
    }
    for _ in 0..5 {
        storage
            .record_transition("global", "pattern-med", "med")
            .await
            .unwrap();
    }

    let patterns = storage.get_learned_patterns(1).await.unwrap();

    assert_eq!(patterns.len(), 3);
    assert_eq!(patterns[0].frequency, 10, "Highest frequency first");
    assert_eq!(patterns[1].frequency, 5, "Middle frequency second");
    assert_eq!(patterns[2].frequency, 3, "Lowest frequency third");
}

#[tokio::test]
async fn test_tenant_isolation_patterns_dont_cross() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());

    let tenant_a = unique_tenant_id("isolation-a");
    let tenant_b = unique_tenant_id("isolation-b");

    // Record pattern for tenant A
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let storage = PostgresProceduralStorage::new(Arc::clone(&backend));
    for _ in 0..5 {
        storage
            .record_transition("global", "theme", "dark")
            .await
            .unwrap();
    }

    // Record different pattern for tenant B
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let memory_b = PostgresProceduralStorage::new(Arc::clone(&backend));
    for _ in 0..3 {
        memory_b
            .record_transition("global", "theme", "light")
            .await
            .unwrap();
    }

    // Query as tenant A - should only see dark theme
    backend.set_tenant_context(&tenant_a).await.unwrap();
    let freq_a = storage
        .get_pattern_frequency("global", "theme", "dark")
        .await
        .unwrap();
    let freq_a_light = storage
        .get_pattern_frequency("global", "theme", "light")
        .await
        .unwrap();

    assert_eq!(freq_a, 5, "Tenant A should see dark theme with freq 5");
    assert_eq!(
        freq_a_light, 0,
        "Tenant A should not see tenant B's light theme"
    );

    // Query as tenant B - should only see light theme
    backend.set_tenant_context(&tenant_b).await.unwrap();
    let freq_b = memory_b
        .get_pattern_frequency("global", "theme", "light")
        .await
        .unwrap();
    let freq_b_dark = memory_b
        .get_pattern_frequency("global", "theme", "dark")
        .await
        .unwrap();

    assert_eq!(freq_b, 3, "Tenant B should see light theme with freq 3");
    assert_eq!(
        freq_b_dark, 0,
        "Tenant B should not see tenant A's dark theme"
    );
}

#[tokio::test]
async fn test_multiple_values_for_same_scope_key() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("multi-value");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record different values for same scope:key
    for _ in 0..3 {
        storage
            .record_transition("global", "theme", "dark")
            .await
            .unwrap();
    }
    for _ in 0..5 {
        storage
            .record_transition("global", "theme", "light")
            .await
            .unwrap();
    }
    for _ in 0..2 {
        storage
            .record_transition("global", "theme", "auto")
            .await
            .unwrap();
    }

    let freq_dark = storage
        .get_pattern_frequency("global", "theme", "dark")
        .await
        .unwrap();
    let freq_light = storage
        .get_pattern_frequency("global", "theme", "light")
        .await
        .unwrap();
    let freq_auto = storage
        .get_pattern_frequency("global", "theme", "auto")
        .await
        .unwrap();

    assert_eq!(freq_dark, 3);
    assert_eq!(freq_light, 5);
    assert_eq!(freq_auto, 2);

    let patterns = storage.get_learned_patterns(3).await.unwrap();
    assert_eq!(
        patterns.len(),
        2,
        "Should have 2 patterns with freq >= 3 (dark and light)"
    );
}

#[tokio::test]
async fn test_pattern_struct_fields() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("pattern-fields");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record pattern
    for _ in 0..3 {
        storage
            .record_transition("session:abc", "config.lang", "en")
            .await
            .unwrap();
    }

    let patterns = storage.get_learned_patterns(3).await.unwrap();
    assert_eq!(patterns.len(), 1);

    let pattern = &patterns[0];
    assert_eq!(pattern.scope, "session:abc");
    assert_eq!(pattern.key, "config.lang");
    assert_eq!(pattern.value, "en");
    assert_eq!(pattern.frequency, 3);
    assert!(pattern.first_seen > 0, "first_seen should be set");
    assert!(pattern.last_seen > 0, "last_seen should be set");
    assert!(
        pattern.last_seen >= pattern.first_seen,
        "last_seen should be >= first_seen"
    );
}

#[tokio::test]
async fn test_first_seen_stays_constant_last_seen_updates() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("timestamp-update");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record first transition
    storage
        .record_transition("global", "test", "value")
        .await
        .unwrap();

    let patterns1 = storage.get_learned_patterns(1).await.unwrap();
    let first_seen_1 = patterns1[0].first_seen;
    let last_seen_1 = patterns1[0].last_seen;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Record again
    storage
        .record_transition("global", "test", "value")
        .await
        .unwrap();

    let patterns2 = storage.get_learned_patterns(1).await.unwrap();
    let first_seen_2 = patterns2[0].first_seen;
    let last_seen_2 = patterns2[0].last_seen;

    assert_eq!(
        first_seen_1, first_seen_2,
        "first_seen should remain constant"
    );
    assert!(
        last_seen_2 > last_seen_1,
        "last_seen should update to later timestamp"
    );
}

#[tokio::test]
async fn test_empty_scope_and_key() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("empty-strings");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Record pattern with empty scope
    let freq = storage.record_transition("", "key", "value").await.unwrap();
    assert_eq!(freq, 1);

    // Record pattern with empty key
    let freq = storage
        .record_transition("scope", "", "value")
        .await
        .unwrap();
    assert_eq!(freq, 1);

    // Verify retrieval
    let freq = storage
        .get_pattern_frequency("", "key", "value")
        .await
        .unwrap();
    assert_eq!(freq, 1);

    let freq = storage
        .get_pattern_frequency("scope", "", "value")
        .await
        .unwrap();
    assert_eq!(freq, 1);
}

#[tokio::test]
async fn test_long_strings() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("long-strings");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Create long strings (within schema limits: scope/key 500 chars)
    let long_scope = "a".repeat(400);
    let long_key = "b".repeat(400);
    let long_value = "c".repeat(1000);

    let freq = storage
        .record_transition(&long_scope, &long_key, &long_value)
        .await
        .unwrap();

    assert_eq!(freq, 1);

    let retrieved_freq = storage
        .get_pattern_frequency(&long_scope, &long_key, &long_value)
        .await
        .unwrap();

    assert_eq!(retrieved_freq, 1);
}

#[tokio::test]
async fn test_special_characters() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("special-chars");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Test special characters
    let special_scope = "scope:with|pipes|and:colons";
    let special_key = "key.with.dots-and-dashes_underscores";
    let special_value = r#"{"json": "value", "with": ["arrays", "and", "stuff"]}"#;

    let freq = storage
        .record_transition(special_scope, special_key, special_value)
        .await
        .unwrap();

    assert_eq!(freq, 1);

    let retrieved_freq = storage
        .get_pattern_frequency(special_scope, special_key, special_value)
        .await
        .unwrap();

    assert_eq!(retrieved_freq, 1);
}

#[tokio::test]
async fn test_concurrent_updates() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("concurrent");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = Arc::new(PostgresProceduralStorage::new(backend));

    // Spawn 10 concurrent tasks each recording the same pattern 10 times
    let mut handles = vec![];
    for _ in 0..10 {
        let mem = Arc::clone(&storage);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                mem.record_transition("global", "concurrent", "test")
                    .await
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final frequency is 100
    let freq = storage
        .get_pattern_frequency("global", "concurrent", "test")
        .await
        .unwrap();

    assert_eq!(freq, 100, "Concurrent updates should all be counted");
}

#[tokio::test]
async fn test_integration_realistic_workflow() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("integration");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Simulate realistic workflow: user changes theme multiple times
    storage
        .record_transition("user:session", "ui.theme", "light")
        .await
        .unwrap();
    storage
        .record_transition("user:session", "ui.theme", "dark")
        .await
        .unwrap();
    storage
        .record_transition("user:session", "ui.theme", "light")
        .await
        .unwrap();
    storage
        .record_transition("user:session", "ui.theme", "dark")
        .await
        .unwrap();
    storage
        .record_transition("user:session", "ui.theme", "dark")
        .await
        .unwrap();

    // User changes language
    for _ in 0..4 {
        storage
            .record_transition("user:session", "ui.lang", "en")
            .await
            .unwrap();
    }

    // User changes other settings
    storage
        .record_transition("user:session", "ui.font_size", "14")
        .await
        .unwrap();
    storage
        .record_transition("user:session", "ui.font_size", "16")
        .await
        .unwrap();

    // Get learned patterns (frequency >= 3)
    let patterns = storage.get_learned_patterns(3).await.unwrap();

    // Should have 2 learned patterns: dark theme (3x) and en language (4x)
    assert_eq!(patterns.len(), 2);

    // Verify patterns
    let lang_pattern = patterns.iter().find(|p| p.key == "ui.lang").unwrap();
    assert_eq!(lang_pattern.value, "en");
    assert_eq!(lang_pattern.frequency, 4);

    let theme_pattern = patterns.iter().find(|p| p.key == "ui.theme").unwrap();
    assert_eq!(theme_pattern.value, "dark");
    assert_eq!(theme_pattern.frequency, 3);
}

#[tokio::test]
async fn test_performance_pattern_queries() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let tenant_id = unique_tenant_id("perf");

    backend.set_tenant_context(&tenant_id).await.unwrap();

    let storage = PostgresProceduralStorage::new(backend);

    // Insert 100 patterns
    for i in 0..100 {
        for _ in 0..3 {
            storage
                .record_transition("global", &format!("pattern-{}", i), "value")
                .await
                .unwrap();
        }
    }

    // Time get_learned_patterns query
    let start = std::time::Instant::now();
    let patterns = storage.get_learned_patterns(3).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(patterns.len(), 100);
    assert!(
        duration.as_millis() < 50,
        "Query should be <50ms, was {}ms",
        duration.as_millis()
    );

    // Time get_pattern_frequency query
    let start = std::time::Instant::now();
    let _freq = storage
        .get_pattern_frequency("global", "pattern-50", "value")
        .await
        .unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 10,
        "Pattern frequency query should be <10ms, was {}ms",
        duration.as_millis()
    );
}
