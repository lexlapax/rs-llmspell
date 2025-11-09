//! ABOUTME: Migration adapters for existing storage backends
//! ABOUTME: Implements MigrationSource and MigrationTarget for Sled and PostgreSQL

use super::traits::MigrationSource;
use crate::backends::sled_backend::SledBackend;
use crate::traits::StorageBackend;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

#[cfg(feature = "postgres")]
use super::traits::MigrationTarget;

#[cfg(feature = "postgres")]
use crate::backends::postgres::PostgresBackend;

// =============================================================================
// SledBackend → MigrationSource (Phase 1: Source for agent_state, workflow_state, sessions)
// =============================================================================

#[async_trait]
impl MigrationSource for SledBackend {
    async fn list_keys(&self, component: &str) -> Result<Vec<String>> {
        let prefix = component_to_prefix(component);
        // Disambiguate: call StorageBackend::list_keys explicitly
        StorageBackend::list_keys(self, &prefix).await
    }

    async fn get_value(&self, _component: &str, key: &str) -> Result<Option<Vec<u8>>> {
        // Key is already fully qualified (e.g., "agent:123" or "custom:workflow_456")
        // Just get it directly from Sled
        StorageBackend::get(self, key).await
    }

    async fn count(&self, component: &str) -> Result<usize> {
        let prefix = component_to_prefix(component);
        // Disambiguate: call StorageBackend::list_keys explicitly
        let keys = StorageBackend::list_keys(self, &prefix).await?;
        Ok(keys.len())
    }
}

// =============================================================================
// PostgresBackend → MigrationTarget (Phase 1: Target for agent_state, workflow_state, sessions)
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

// =============================================================================
// Component Name → Key Prefix Mapping
// =============================================================================

/// Convert component name to storage key prefix
///
/// Phase 1 Components:
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

#[async_trait]
impl MigrationSource for Arc<SledBackend> {
    async fn list_keys(&self, component: &str) -> Result<Vec<String>> {
        // Call the MigrationSource trait method on the inner SledBackend
        MigrationSource::list_keys(&**self, component).await
    }

    async fn get_value(&self, component: &str, key: &str) -> Result<Option<Vec<u8>>> {
        MigrationSource::get_value(&**self, component, key).await
    }

    async fn count(&self, component: &str) -> Result<usize> {
        MigrationSource::count(&**self, component).await
    }
}

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

    #[tokio::test]
    async fn test_sled_migration_source() {
        let sled = SledBackend::new_temporary().unwrap();

        // Store test data with proper prefixes
        StorageBackend::set(&sled, "agent:123", b"agent data".to_vec())
            .await
            .unwrap();
        StorageBackend::set(&sled, "agent:456", b"agent data 2".to_vec())
            .await
            .unwrap();
        StorageBackend::set(&sled, "custom:workflow_abc", b"workflow data".to_vec())
            .await
            .unwrap();

        // Test list_keys for agent_state (using MigrationSource trait)
        let keys = MigrationSource::list_keys(&sled, "agent_state")
            .await
            .unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"agent:123".to_string()));

        // Test count
        let count = MigrationSource::count(&sled, "agent_state").await.unwrap();
        assert_eq!(count, 2);

        // Test get_value
        let value = MigrationSource::get_value(&sled, "agent_state", "agent:123")
            .await
            .unwrap();
        assert_eq!(value, Some(b"agent data".to_vec()));
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
