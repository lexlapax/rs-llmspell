//! ABOUTME: PostgreSQL API key storage with encryption (Phase 13b.13.2)
//! ABOUTME: Encrypted API key storage with rotation and expiration support

use super::backend::PostgresBackend;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Row;

/// API key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyMetadata {
    pub key_id: String,
    pub service: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub usage_count: u64,
}

/// API key storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeyStats {
    pub total_keys: u64,
    pub active_keys: u64,
    pub expired_keys: u64,
    pub keys_by_service: HashMap<String, u64>,
    pub total_usage: u64,
    pub avg_usage_per_key: f64,
}

/// PostgreSQL API key storage with pgcrypto encryption
#[derive(Debug, Clone)]
pub struct PostgresApiKeyStorage {
    backend: Arc<PostgresBackend>,
    encryption_passphrase: String,
}

impl PostgresApiKeyStorage {
    /// Create new API key storage with encryption passphrase
    pub fn new(backend: Arc<PostgresBackend>, encryption_passphrase: String) -> Self {
        Self {
            backend,
            encryption_passphrase,
        }
    }

    async fn get_tenant_context(&self) -> Result<String> {
        self.backend
            .get_tenant_context()
            .await
            .ok_or_else(|| anyhow!("Tenant context not set"))
    }

    fn metadata_from_row(row: &Row) -> Result<ApiKeyMetadata> {
        Ok(ApiKeyMetadata {
            key_id: row.get("key_id"),
            service: row.get("service"),
            created_at: row.get("created_at"),
            last_used: row.get("last_used_at"),
            expires_at: row.get("expires_at"),
            is_active: row.get("is_active"),
            usage_count: row.get::<_, i64>("usage_count") as u64,
        })
    }

    /// Store encrypted API key
    pub async fn store(&self, key_id: &str, key: &str, metadata: &ApiKeyMetadata) -> Result<()> {
        let tenant_id = self.get_tenant_context().await?;
        let client = self.backend.get_client().await?;

        let key_metadata: Value = serde_json::to_value(&metadata.service)?;

        client
            .execute(
                "INSERT INTO llmspell.api_keys
                 (key_id, tenant_id, service, encrypted_key, key_metadata, created_at, last_used_at,
                  expires_at, is_active, usage_count)
                 VALUES ($1, $2, $3, pgp_sym_encrypt($4::TEXT, $5::TEXT), $6::JSONB,
                         $7, $8, $9, $10, $11)",
                &[
                    &key_id,
                    &tenant_id,
                    &metadata.service,
                    &key,
                    &self.encryption_passphrase,
                    &key_metadata,
                    &metadata.created_at,
                    &metadata.last_used,
                    &metadata.expires_at,
                    &metadata.is_active,
                    &(metadata.usage_count as i64),
                ],
            )
            .await
            .map_err(|e| anyhow!("Failed to store API key: {}", e))?;

        Ok(())
    }

    /// Retrieve and decrypt API key
    pub async fn get(&self, key_id: &str) -> Result<Option<String>> {
        let tenant_id = self.get_tenant_context().await?;
        let client = self.backend.get_client().await?;

        let row = client
            .query_opt(
                "SELECT pgp_sym_decrypt(encrypted_key, $2::TEXT) as decrypted_key
                 FROM llmspell.api_keys
                 WHERE key_id = $1 AND tenant_id = $3",
                &[&key_id, &self.encryption_passphrase, &tenant_id],
            )
            .await?;

        Ok(row.map(|r| r.get::<_, String>("decrypted_key")))
    }

    /// Get API key metadata
    pub async fn get_metadata(&self, key_id: &str) -> Result<Option<ApiKeyMetadata>> {
        let tenant_id = self.get_tenant_context().await?;
        let client = self.backend.get_client().await?;

        let row = client
            .query_opt(
                "SELECT key_id, service, created_at, last_used_at, expires_at, is_active, usage_count
                 FROM llmspell.api_keys
                 WHERE key_id = $1 AND tenant_id = $2",
                &[&key_id, &tenant_id],
            )
            .await?;

        row.as_ref().map(Self::metadata_from_row).transpose()
    }

    /// Update metadata
    pub async fn update_metadata(&self, key_id: &str, metadata: &ApiKeyMetadata) -> Result<()> {
        let tenant_id = self.get_tenant_context().await?;
        let client = self.backend.get_client().await?;

        let result = client
            .execute(
                "UPDATE llmspell.api_keys
                 SET last_used_at = $1, expires_at = $2, is_active = $3, usage_count = $4
                 WHERE key_id = $5 AND tenant_id = $6",
                &[
                    &metadata.last_used,
                    &metadata.expires_at,
                    &metadata.is_active,
                    &(metadata.usage_count as i64),
                    &key_id,
                    &tenant_id,
                ],
            )
            .await?;

        if result == 0 {
            return Err(anyhow!("Key not found: {}", key_id));
        }

        Ok(())
    }

    /// Delete API key
    pub async fn delete(&self, key_id: &str) -> Result<()> {
        let tenant_id = self.get_tenant_context().await?;
        let client = self.backend.get_client().await?;

        let result = client
            .execute(
                "DELETE FROM llmspell.api_keys WHERE key_id = $1 AND tenant_id = $2",
                &[&key_id, &tenant_id],
            )
            .await?;

        if result == 0 {
            return Err(anyhow!("Key not found: {}", key_id));
        }

        Ok(())
    }

    /// List all key IDs for tenant
    pub async fn list_keys(&self) -> Result<Vec<String>> {
        let tenant_id = self.get_tenant_context().await?;
        let client = self.backend.get_client().await?;

        let rows = client
            .query(
                "SELECT key_id FROM llmspell.api_keys WHERE tenant_id = $1 ORDER BY created_at DESC",
                &[&tenant_id],
            )
            .await?;

        Ok(rows.iter().map(|r| r.get::<_, String>("key_id")).collect())
    }

    /// Rotate API key
    pub async fn rotate_key(&self, old_key_id: &str, new_key: &str) -> Result<String> {
        let client = self.backend.get_client().await?;

        // Call PostgreSQL function to deactivate old key and generate new key_id
        let row = client
            .query_one(
                "SELECT llmspell.rotate_api_key($1) as new_key_id",
                &[&old_key_id],
            )
            .await?;

        let new_key_id: String = row.get("new_key_id");

        // Get old metadata
        let old_metadata = self
            .get_metadata(old_key_id)
            .await?
            .ok_or_else(|| anyhow!("Old key not found"))?;

        // Create new metadata
        let new_metadata = ApiKeyMetadata {
            key_id: new_key_id.clone(),
            service: old_metadata.service.clone(),
            created_at: Utc::now(),
            last_used: None,
            expires_at: old_metadata.expires_at,
            is_active: true,
            usage_count: 0,
        };

        // Store new key
        self.store(&new_key_id, new_key, &new_metadata).await?;

        // Update rotated_from field
        client
            .execute(
                "UPDATE llmspell.api_keys SET rotated_from = $1 WHERE key_id = $2",
                &[&old_key_id, &new_key_id],
            )
            .await?;

        Ok(new_key_id)
    }

    /// Cleanup expired keys
    pub async fn cleanup_expired_keys(&self) -> Result<usize> {
        let client = self.backend.get_client().await?;

        let rows = client
            .query("SELECT * FROM llmspell.cleanup_expired_api_keys()", &[])
            .await?;

        Ok(rows.len())
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> Result<ApiKeyStats> {
        let client = self.backend.get_client().await?;

        let row = client
            .query_one(
                "SELECT
                    total_keys, active_keys, expired_keys, keys_by_service,
                    total_usage, avg_usage_per_key::DOUBLE PRECISION as avg_usage_per_key
                 FROM llmspell.get_api_key_stats()",
                &[],
            )
            .await?;

        let total_keys: i64 = row.get("total_keys");
        let active_keys: i64 = row.get("active_keys");
        let expired_keys: i64 = row.get("expired_keys");
        let keys_by_service: Value = row.get("keys_by_service");
        let total_usage: i64 = row.get("total_usage");
        let avg_usage_per_key: f64 = row.get("avg_usage_per_key");

        let keys_by_service_map: HashMap<String, u64> = keys_by_service
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0)))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ApiKeyStats {
            total_keys: total_keys as u64,
            active_keys: active_keys as u64,
            expired_keys: expired_keys as u64,
            keys_by_service: keys_by_service_map,
            total_usage: total_usage as u64,
            avg_usage_per_key,
        })
    }
}
