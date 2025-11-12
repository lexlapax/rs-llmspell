//! ABOUTME: Migration adapters for existing storage backends (deprecated - Phase 13c)
//! ABOUTME: Sled-to-PostgreSQL migration removed; use SQLite or PostgreSQL directly

#[cfg(feature = "postgres")]
use super::traits::MigrationTarget;

#[cfg(feature = "postgres")]
use crate::backends::postgres::PostgresBackend;

use crate::traits::StorageBackend;

#[cfg(feature = "postgres")]
use anyhow::Result;

#[cfg(feature = "postgres")]
use async_trait::async_trait;

#[cfg(feature = "postgres")]
use std::sync::Arc;

// =============================================================================
// PostgresBackend → MigrationTarget (retained for future SQLite→PostgreSQL migrations)
// =============================================================================

#[cfg(feature = "postgres")]
#[async_trait]
impl MigrationTarget for PostgresBackend {
    async fn store(&self, _component: &str, key: &str, value: &[u8]) -> Result<()> {
        // Key is already fully qualified (e.g., "agent:123" or "custom:workflow_456")
        // PostgresBackend's StorageBackend impl will route to correct table
        StorageBackend::set(self, key, value.to_vec()).await
    }

    async fn get_value(&self, _component: &str, key: &str) -> Result<Option<Vec<u8>>> {
        // Use StorageBackend::get to read back from PostgreSQL
        StorageBackend::get(self, key).await
    }

    async fn count(&self, component: &str) -> Result<usize> {
        let prefix = component_to_prefix(component);
        let keys = StorageBackend::list_keys(self, &prefix).await?;
        Ok(keys.len())
    }

    async fn delete(&self, _component: &str, key: &str) -> Result<()> {
        // Use StorageBackend::delete to remove from PostgreSQL
        StorageBackend::delete(self, key).await
    }
}

#[cfg(feature = "postgres")]
// =============================================================================
// Component Name → Key Prefix Mapping
// =============================================================================

/// Convert component name to storage key prefix
///
/// Supported Components:
/// - "agent_state" → "agent:"
/// - "workflow_state" → "custom:workflow_"
/// - "sessions" → "session:"
fn component_to_prefix(component: &str) -> String {
    match component {
        "agent_state" => "agent:".to_string(),
        "workflow_state" => "custom:workflow_".to_string(),
        "sessions" => "session:".to_string(),
        _ => format!("{}:", component), // Fallback: just add colon
    }
}

// =============================================================================
// Arc-wrapped implementations (for use in MigrationEngine)
// =============================================================================

#[cfg(feature = "postgres")]
#[async_trait]
impl MigrationTarget for Arc<PostgresBackend> {
    async fn store(&self, component: &str, key: &str, value: &[u8]) -> Result<()> {
        MigrationTarget::store(&**self, component, key, value).await
    }

    async fn get_value(&self, component: &str, key: &str) -> Result<Option<Vec<u8>>> {
        MigrationTarget::get_value(&**self, component, key).await
    }

    async fn count(&self, component: &str) -> Result<usize> {
        MigrationTarget::count(&**self, component).await
    }

    async fn delete(&self, component: &str, key: &str) -> Result<()> {
        MigrationTarget::delete(&**self, component, key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_prefix_mapping() {
        assert_eq!(component_to_prefix("agent_state"), "agent:");
        assert_eq!(component_to_prefix("workflow_state"), "custom:workflow_");
        assert_eq!(component_to_prefix("sessions"), "session:");
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn test_postgres_migration_target() {
        use crate::backends::postgres::PostgresConfig;

        const TEST_CONNECTION_STRING: &str =
            "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

        let config = PostgresConfig::new(TEST_CONNECTION_STRING);
        let backend = match PostgresBackend::new(config).await {
            Ok(b) => b,
            Err(_) => {
                eprintln!("Skipping postgres test - database not available");
                return;
            }
        };

        // Run migrations to ensure schema exists
        if let Err(e) = backend.run_migrations().await {
            eprintln!("Skipping postgres test - migrations failed: {}", e);
            return;
        }

        // Set tenant context
        let tenant_id = format!("test_migration_{}", uuid::Uuid::new_v4());
        backend.set_tenant_context(&tenant_id).await.unwrap();

        // Test store (using MigrationTarget trait with correct agent key format)
        // Format: agent:<agent_type>:<agent_id>
        let test_key = "agent:test:migration_123";
        let test_data = br#"{"agent_id": "migration_123", "state": "test", "iteration": 0}"#;
        MigrationTarget::store(&backend, "agent_state", test_key, test_data)
            .await
            .unwrap();

        // Test count
        let count = MigrationTarget::count(&backend, "agent_state")
            .await
            .unwrap();
        assert!(count >= 1);

        // Cleanup
        StorageBackend::delete(&backend, test_key).await.unwrap();
    }
}
