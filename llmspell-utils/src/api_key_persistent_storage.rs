//! ABOUTME: Persistent storage backend for API keys using sled database
//! ABOUTME: Provides encrypted key storage with metadata persistence

#![allow(clippy::missing_errors_doc)]

use crate::api_key_manager::{ApiKeyMetadata, ApiKeyStorage};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Persistent storage implementation using sled
pub struct PersistentApiKeyStorage {
    db: sled::Db,
    encryption_key: [u8; 32],
}

/// Key-value pair for storage
#[derive(Serialize, Deserialize)]
struct StoredKeyData {
    encrypted_key: Vec<u8>,
    metadata: ApiKeyMetadata,
}

impl PersistentApiKeyStorage {
    /// Create a new persistent storage backend
    pub fn new<P: AsRef<Path>>(path: P, encryption_key: [u8; 32]) -> Result<Self, String> {
        let db = sled::open(path).map_err(|e| format!("Failed to open database: {e}"))?;
        Ok(Self { db, encryption_key })
    }

    /// Encrypt a key using simple XOR (for demonstration)
    /// In production, use proper encryption like AES-256-GCM
    fn encrypt(&self, key: &str) -> Vec<u8> {
        key.bytes()
            .enumerate()
            .map(|(i, b)| b ^ self.encryption_key[i % 32])
            .collect()
    }

    /// Decrypt a key
    fn decrypt(&self, encrypted: &[u8]) -> String {
        encrypted
            .iter()
            .enumerate()
            .map(|(i, &b)| (b ^ self.encryption_key[i % 32]) as char)
            .collect()
    }
}

impl ApiKeyStorage for PersistentApiKeyStorage {
    fn store(&mut self, key_id: &str, key: &str, metadata: &ApiKeyMetadata) -> Result<(), String> {
        let encrypted_key = self.encrypt(key);
        let stored_data = StoredKeyData {
            encrypted_key,
            metadata: metadata.clone(),
        };

        let serialized = serde_json::to_vec(&stored_data)
            .map_err(|e| format!("Failed to serialize key data: {e}"))?;

        self.db
            .insert(key_id.as_bytes(), serialized)
            .map_err(|e| format!("Failed to store key: {e}"))?;

        self.db
            .flush()
            .map_err(|e| format!("Failed to flush database: {e}"))?;

        Ok(())
    }

    fn retrieve(&self, key_id: &str) -> Result<Option<String>, String> {
        match self.db.get(key_id.as_bytes()) {
            Ok(Some(data)) => {
                let stored_data: StoredKeyData = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize key data: {e}"))?;
                Ok(Some(self.decrypt(&stored_data.encrypted_key)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to retrieve key: {e}")),
        }
    }

    fn retrieve_metadata(&self, key_id: &str) -> Result<Option<ApiKeyMetadata>, String> {
        match self.db.get(key_id.as_bytes()) {
            Ok(Some(data)) => {
                let stored_data: StoredKeyData = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize key data: {e}"))?;
                Ok(Some(stored_data.metadata))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to retrieve metadata: {e}")),
        }
    }

    fn update_metadata(&mut self, key_id: &str, metadata: &ApiKeyMetadata) -> Result<(), String> {
        match self.db.get(key_id.as_bytes()) {
            Ok(Some(data)) => {
                let mut stored_data: StoredKeyData = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize key data: {e}"))?;

                stored_data.metadata = metadata.clone();

                let serialized = serde_json::to_vec(&stored_data)
                    .map_err(|e| format!("Failed to serialize key data: {e}"))?;

                self.db
                    .insert(key_id.as_bytes(), serialized)
                    .map_err(|e| format!("Failed to update metadata: {e}"))?;

                self.db
                    .flush()
                    .map_err(|e| format!("Failed to flush database: {e}"))?;

                Ok(())
            }
            Ok(None) => Err("Key not found".to_string()),
            Err(e) => Err(format!("Failed to retrieve key for update: {e}")),
        }
    }

    fn delete(&mut self, key_id: &str) -> Result<(), String> {
        self.db
            .remove(key_id.as_bytes())
            .map_err(|e| format!("Failed to delete key: {e}"))?;

        self.db
            .flush()
            .map_err(|e| format!("Failed to flush database: {e}"))?;

        Ok(())
    }

    /// List all key IDs
    fn list_keys(&self) -> Result<Vec<String>, String> {
        let mut keys = Vec::new();
        for item in self.db.iter() {
            match item {
                Ok((key, _)) => {
                    if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                        keys.push(key_str);
                    }
                }
                Err(e) => return Err(format!("Failed to iterate keys: {e}")),
            }
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_metadata(service: &str) -> ApiKeyMetadata {
        ApiKeyMetadata {
            key_id: format!("{service}_key"),
            service: service.to_string(),
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: true,
            usage_count: 0,
        }
    }

    #[test]
    fn test_persistent_storage() {
        let temp_dir = TempDir::new().unwrap();
        let encryption_key = [42u8; 32];

        let mut storage = PersistentApiKeyStorage::new(temp_dir.path(), encryption_key).unwrap();

        // Store a key
        let metadata = create_test_metadata("test_service");
        storage
            .store("test_key", "my_secret_key", &metadata)
            .unwrap();

        // Retrieve the key
        let retrieved = storage.retrieve("test_key").unwrap();
        assert_eq!(retrieved, Some("my_secret_key".to_string()));

        // Retrieve metadata
        let retrieved_metadata = storage.retrieve_metadata("test_key").unwrap().unwrap();
        assert_eq!(retrieved_metadata.service, "test_service");

        // Update metadata
        let mut updated_metadata = metadata.clone();
        updated_metadata.usage_count = 5;
        storage
            .update_metadata("test_key", &updated_metadata)
            .unwrap();

        let retrieved_metadata = storage.retrieve_metadata("test_key").unwrap().unwrap();
        assert_eq!(retrieved_metadata.usage_count, 5);

        // Delete the key
        storage.delete("test_key").unwrap();
        assert_eq!(storage.retrieve("test_key").unwrap(), None);
    }

    #[test]
    fn test_encryption_decryption() {
        let temp_dir = TempDir::new().unwrap();
        let encryption_key = [123u8; 32];

        let storage = PersistentApiKeyStorage::new(temp_dir.path(), encryption_key).unwrap();

        let original = "my_super_secret_api_key";
        let encrypted = storage.encrypt(original);
        let decrypted = storage.decrypt(&encrypted);

        assert_eq!(original, decrypted);
        assert_ne!(original.as_bytes(), encrypted.as_slice());
    }
}
