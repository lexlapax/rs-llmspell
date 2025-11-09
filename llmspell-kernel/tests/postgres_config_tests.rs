//! Configuration tests for PostgreSQL backend selection (Phase 13b.2.7)
//!
//! Tests TOML parsing, validation, and defaults for PostgreSQL kernel state configuration

#![cfg(feature = "postgres")]

use llmspell_kernel::state::config::{PostgresConfig, StorageBackendType};

#[test]
fn test_postgres_config_default() {
    let config = PostgresConfig::default();

    assert_eq!(config.connection_string, "");
    assert_eq!(config.pool_size, 20);
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.enable_rls);
}

#[test]
fn test_postgres_config_serialization() {
    let config = PostgresConfig {
        connection_string: "postgresql://localhost/test".to_string(),
        pool_size: 10,
        timeout_ms: 3000,
        enable_rls: false,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&config).expect("Failed to serialize");

    // Deserialize back
    let deserialized: PostgresConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(
        deserialized.connection_string,
        "postgresql://localhost/test"
    );
    assert_eq!(deserialized.pool_size, 10);
    assert_eq!(deserialized.timeout_ms, 3000);
    assert!(!deserialized.enable_rls);
}

#[test]
fn test_postgres_config_serde_defaults() {
    // Test that serde uses defaults for missing fields
    let json = r#"{"connection_string": "postgresql://localhost/test"}"#;

    let config: PostgresConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.connection_string, "postgresql://localhost/test");
    assert_eq!(config.pool_size, 20, "Should use default pool_size");
    assert_eq!(config.timeout_ms, 5000, "Should use default timeout_ms");
    assert!(config.enable_rls, "Should use default enable_rls");
}

#[test]
fn test_storage_backend_type_postgres_variant() {
    let config = PostgresConfig {
        connection_string: "postgresql://localhost/llmspell".to_string(),
        pool_size: 15,
        timeout_ms: 10000,
        enable_rls: true,
    };

    let backend_type = StorageBackendType::Postgres(config.clone());

    // Test pattern matching
    match backend_type {
        StorageBackendType::Postgres(pg_config) => {
            assert_eq!(pg_config.connection_string, config.connection_string);
            assert_eq!(pg_config.pool_size, config.pool_size);
        }
        _ => panic!("Expected Postgres variant"),
    }
}

#[test]
fn test_postgres_backend_type_serialization() {
    let config = PostgresConfig {
        connection_string: "postgresql://user:pass@host:5432/db".to_string(),
        pool_size: 25,
        timeout_ms: 7500,
        enable_rls: false,
    };

    let backend_type = StorageBackendType::Postgres(config);

    // Serialize
    let json = serde_json::to_string(&backend_type).expect("Failed to serialize");

    // Deserialize
    let deserialized: StorageBackendType =
        serde_json::from_str(&json).expect("Failed to deserialize");

    match deserialized {
        StorageBackendType::Postgres(pg_config) => {
            assert_eq!(
                pg_config.connection_string,
                "postgresql://user:pass@host:5432/db"
            );
            assert_eq!(pg_config.pool_size, 25);
            assert_eq!(pg_config.timeout_ms, 7500);
            assert!(!pg_config.enable_rls);
        }
        _ => panic!("Expected Postgres variant after deserialization"),
    }
}

#[test]
fn test_toml_postgres_config_parsing() {
    let toml_str = r#"
connection_string = "postgresql://llmspell:password@localhost:5432/llmspell_dev"
pool_size = 30
timeout_ms = 15000
enable_rls = true
"#;

    let config: PostgresConfig = toml::from_str(toml_str).expect("Failed to parse TOML");

    assert_eq!(
        config.connection_string,
        "postgresql://llmspell:password@localhost:5432/llmspell_dev"
    );
    assert_eq!(config.pool_size, 30);
    assert_eq!(config.timeout_ms, 15000);
    assert!(config.enable_rls);
}

#[test]
fn test_toml_postgres_config_with_defaults() {
    let toml_str = r#"
connection_string = "postgresql://localhost/test"
"#;

    let config: PostgresConfig = toml::from_str(toml_str).expect("Failed to parse TOML");

    assert_eq!(config.connection_string, "postgresql://localhost/test");
    assert_eq!(config.pool_size, 20, "Should use default");
    assert_eq!(config.timeout_ms, 5000, "Should use default");
    assert!(config.enable_rls, "Should use default");
}

#[test]
fn test_toml_storage_backend_type_postgres() {
    let toml_str = r#"
[Postgres]
connection_string = "postgresql://localhost/llmspell"
pool_size = 20
timeout_ms = 5000
enable_rls = true
"#;

    let backend_type: StorageBackendType = toml::from_str(toml_str).expect("Failed to parse TOML");

    match backend_type {
        StorageBackendType::Postgres(config) => {
            assert_eq!(config.connection_string, "postgresql://localhost/llmspell");
            assert_eq!(config.pool_size, 20);
        }
        _ => panic!("Expected Postgres variant"),
    }
}
