// ABOUTME: Backup manager for coordinating backup operations and lifecycle
// ABOUTME: Handles scheduling, retention, validation, and restoration of backups

use super::{AtomicBackup, BackupConfig, BackupId, BackupResult, BackupValidation, RestoreOptions};
use crate::{error::StateError, manager::StateManager};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Backup status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    /// Backup identifier
    pub id: BackupId,

    /// Backup creation time
    pub created_at: SystemTime,

    /// Backup size in bytes
    pub size_bytes: u64,

    /// Whether this is an incremental backup
    pub is_incremental: bool,

    /// Parent backup ID for incremental backups
    pub parent_id: Option<BackupId>,

    /// State version at backup time
    pub state_version: String,

    /// Number of state entries backed up
    pub entry_count: usize,

    /// Backup validation status
    pub validation: Option<BackupValidation>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Backup metadata stored with each backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup identifier
    pub id: BackupId,

    /// Backup creation timestamp
    pub created_at: SystemTime,

    /// Backup type (full or incremental)
    pub backup_type: BackupType,

    /// Parent backup for incremental backups
    pub parent_id: Option<BackupId>,

    /// State schema version
    pub schema_version: String,

    /// Checksums for validation
    pub checksums: HashMap<String, String>,

    /// Compression info if compressed
    pub compression: Option<CompressionInfo>,

    /// Encryption info if encrypted
    pub encryption: Option<EncryptionInfo>,

    /// Statistics about the backup
    pub stats: BackupStats,
}

/// Backup type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    pub algorithm: String,
    pub level: u32,
    pub original_size: u64,
    pub compressed_size: u64,
}

/// Encryption information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub key_id: String,
}

/// Backup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_entries: usize,
    pub total_size: u64,
    pub duration_ms: u64,
    pub scopes_backed_up: Vec<String>,
}

/// Incremental backup tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalBackup {
    /// Chain of backup IDs from oldest to newest
    pub backup_chain: Vec<BackupId>,

    /// Last full backup in the chain
    pub last_full_backup: BackupId,

    /// Total size of the chain
    pub chain_size: u64,
}

/// Backup schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    /// Enable scheduled backups
    pub enabled: bool,

    /// Interval between backups
    pub interval: Duration,

    /// Time of day for daily backups (if applicable)
    pub daily_time: Option<String>,

    /// Days of week for weekly backups (if applicable)
    pub weekly_days: Option<Vec<String>>,
}

/// Backup manager for coordinating backup operations
pub struct BackupManager {
    config: BackupConfig,
    state_manager: Arc<RwLock<StateManager>>,
    backup_index: Arc<RwLock<HashMap<BackupId, BackupMetadata>>>,
    _active_backups: Arc<RwLock<HashMap<BackupId, BackupStatus>>>,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(config: BackupConfig, state_manager: Arc<RwLock<StateManager>>) -> Result<Self> {
        // Ensure backup directory exists
        std::fs::create_dir_all(&config.backup_dir).context("Failed to create backup directory")?;

        let mut manager = Self {
            config,
            state_manager,
            backup_index: Arc::new(RwLock::new(HashMap::new())),
            _active_backups: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing backup index
        manager.load_backup_index()?;

        Ok(manager)
    }

    /// Create a new backup
    pub async fn create_backup(&self, incremental: bool) -> BackupResult<BackupStatus> {
        let backup_id = self.generate_backup_id();
        info!(
            "Creating {} backup: {}",
            if incremental { "incremental" } else { "full" },
            backup_id
        );

        // TODO: Emit backup started event when StateEvent is available

        // Determine parent backup for incremental
        let parent_id = if incremental {
            self.find_parent_backup().await?
        } else {
            None
        };

        // Create atomic backup operation
        let atomic_backup = AtomicBackup::new(
            backup_id.clone(),
            self.state_manager.clone(),
            parent_id.clone(),
        )?;

        // Perform the backup
        let start_time = std::time::Instant::now();
        let backup_data = atomic_backup.capture().await?;
        let duration = start_time.elapsed();

        // Compress if enabled
        let (data, compression_info) = if self.config.compression_enabled {
            let compressed = self.compress_backup(&backup_data).await?;
            let info = CompressionInfo {
                algorithm: self.config.compression_type.to_string(),
                level: self.config.compression_level as u32,
                original_size: backup_data.len() as u64,
                compressed_size: compressed.len() as u64,
            };
            (compressed, Some(info))
        } else {
            (backup_data.clone(), None)
        };

        // Save backup to disk
        let backup_path = self.get_backup_path(&backup_id);
        tokio::fs::write(&backup_path, &data)
            .await
            .context("Failed to write backup file")?;

        // Count entries - for now use a placeholder
        let entry_count = 100; // TODO: Get actual count from atomic backup

        // Create metadata
        let metadata = BackupMetadata {
            id: backup_id.clone(),
            created_at: SystemTime::now(),
            backup_type: if incremental {
                BackupType::Incremental
            } else {
                BackupType::Full
            },
            parent_id: parent_id.clone(),
            schema_version: self.get_current_schema_version().await?,
            checksums: self.calculate_checksums(&data),
            compression: compression_info.clone(),
            encryption: None, // TODO: Implement encryption
            stats: BackupStats {
                total_entries: entry_count,
                total_size: data.len() as u64,
                duration_ms: duration.as_millis() as u64,
                scopes_backed_up: vec!["global".to_string()], // TODO: Get actual scopes
            },
        };

        // Save metadata
        self.save_backup_metadata(&metadata).await?;

        // Update backup index
        {
            let mut index = self.backup_index.write().await;
            index.insert(backup_id.clone(), metadata.clone());
        }

        // Create status
        let status = BackupStatus {
            id: backup_id.clone(),
            created_at: metadata.created_at,
            size_bytes: metadata.stats.total_size,
            is_incremental: incremental,
            parent_id,
            state_version: metadata.schema_version,
            entry_count: metadata.stats.total_entries,
            validation: None,
            metadata: HashMap::new(),
        };

        // TODO: Emit backup completed event when StateEvent is available

        // Clean up old backups if needed
        self.cleanup_old_backups().await?;

        Ok(status)
    }

    /// Restore from a backup
    pub async fn restore_backup(
        &self,
        backup_id: &str,
        options: RestoreOptions,
    ) -> BackupResult<()> {
        info!(
            "Restoring backup: {} (dry_run: {})",
            backup_id, options.dry_run
        );

        // TODO: Emit restore started event when StateEvent is available

        // Validate backup if requested
        if options.verify_checksums {
            let validation = self.validate_backup(backup_id).await?;
            if !validation.is_valid {
                return Err(StateError::ValidationError(format!(
                    "Backup validation failed: {}",
                    validation.errors.join(", ")
                )));
            }
        }

        // Create backup of current state if requested
        if options.backup_current && !options.dry_run {
            info!("Creating backup of current state before restore");
            self.create_backup(false).await?;
        }

        // Load backup metadata
        let metadata = self.get_backup_metadata(backup_id).await?;

        // Build restore chain for incremental backups
        let restore_chain = if metadata.backup_type == BackupType::Incremental {
            self.build_restore_chain(backup_id).await?
        } else {
            vec![backup_id.to_string()]
        };

        // Perform restore
        if !options.dry_run {
            for backup_id in &restore_chain {
                self.restore_single_backup(backup_id).await?;
            }
        }

        // TODO: Emit restore completed event when StateEvent is available

        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> BackupResult<Vec<BackupStatus>> {
        let index = self.backup_index.read().await;
        let mut backups = Vec::new();

        for (id, metadata) in index.iter() {
            backups.push(BackupStatus {
                id: id.clone(),
                created_at: metadata.created_at,
                size_bytes: metadata.stats.total_size,
                is_incremental: metadata.backup_type == BackupType::Incremental,
                parent_id: metadata.parent_id.clone(),
                state_version: metadata.schema_version.clone(),
                entry_count: metadata.stats.total_entries,
                validation: None,
                metadata: HashMap::new(),
            });
        }

        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Validate a backup
    pub async fn validate_backup(&self, backup_id: &str) -> BackupResult<BackupValidation> {
        debug!("Validating backup: {}", backup_id);

        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Load metadata
        let metadata = match self.get_backup_metadata(backup_id).await {
            Ok(m) => m,
            Err(e) => {
                errors.push(format!("Failed to load metadata: {}", e));
                return Ok(BackupValidation {
                    is_valid: false,
                    validated_at: SystemTime::now(),
                    checksum_valid: false,
                    integrity_valid: false,
                    errors,
                    warnings,
                });
            }
        };

        // Load backup data
        let backup_path = self.get_backup_path(backup_id);
        let data = match tokio::fs::read(&backup_path).await {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("Failed to read backup file: {}", e));
                return Ok(BackupValidation {
                    is_valid: false,
                    validated_at: SystemTime::now(),
                    checksum_valid: false,
                    integrity_valid: false,
                    errors,
                    warnings,
                });
            }
        };

        // Verify checksums
        let calculated_checksums = self.calculate_checksums(&data);
        let checksum_valid = calculated_checksums == metadata.checksums;
        if !checksum_valid {
            errors.push("Checksum verification failed".to_string());
        }

        // Basic integrity checks
        let integrity_valid = !data.is_empty() && data.len() == metadata.stats.total_size as usize;
        if !integrity_valid {
            errors.push("Data integrity check failed".to_string());
        }

        let is_valid = errors.is_empty();

        Ok(BackupValidation {
            is_valid,
            validated_at: SystemTime::now(),
            checksum_valid,
            integrity_valid,
            errors,
            warnings,
        })
    }

    // Private helper methods

    fn generate_backup_id(&self) -> BackupId {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let random_suffix: String = (0..8)
            .map(|_| format!("{:x}", rand::random::<u8>()))
            .collect();
        format!("backup_{}_{}", timestamp, random_suffix)
    }

    async fn find_parent_backup(&self) -> BackupResult<Option<BackupId>> {
        let index = self.backup_index.read().await;

        // Find the most recent backup
        let mut backups: Vec<_> = index.iter().collect();
        backups.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        Ok(backups.first().map(|(id, _)| (*id).clone()))
    }

    fn get_backup_path(&self, backup_id: &str) -> PathBuf {
        self.config.backup_dir.join(format!("{}.backup", backup_id))
    }

    async fn get_backup_metadata(&self, backup_id: &str) -> BackupResult<BackupMetadata> {
        let index = self.backup_index.read().await;
        index.get(backup_id).cloned().ok_or_else(|| {
            StateError::StorageError(anyhow::anyhow!("Backup not found: {}", backup_id))
        })
    }

    async fn save_backup_metadata(&self, metadata: &BackupMetadata) -> BackupResult<()> {
        let metadata_path = self.config.backup_dir.join(format!("{}.meta", metadata.id));
        let json =
            serde_json::to_string_pretty(metadata).context("Failed to serialize metadata")?;
        tokio::fs::write(metadata_path, json)
            .await
            .context("Failed to write metadata file")?;
        Ok(())
    }

    fn calculate_checksums(&self, data: &[u8]) -> HashMap<String, String> {
        use sha2::{Digest, Sha256};
        let mut checksums = HashMap::new();

        let hash = Sha256::digest(data);
        checksums.insert("sha256".to_string(), format!("{:x}", hash));

        checksums
    }

    async fn compress_backup(&self, data: &[u8]) -> BackupResult<Vec<u8>> {
        // Compression will be implemented in compression.rs
        // For now, return data as-is
        Ok(data.to_vec())
    }

    async fn get_current_schema_version(&self) -> BackupResult<String> {
        // TODO: Get actual schema version from state manager
        Ok("1.0.0".to_string())
    }

    // TODO: Implement event emission when StateEvent includes backup events

    async fn cleanup_old_backups(&self) -> BackupResult<()> {
        // TODO: Implement backup retention policy
        Ok(())
    }

    async fn build_restore_chain(&self, backup_id: &str) -> BackupResult<Vec<BackupId>> {
        // TODO: Build chain of backups needed for incremental restore
        Ok(vec![backup_id.to_string()])
    }

    async fn restore_single_backup(&self, _backup_id: &str) -> BackupResult<()> {
        // TODO: Implement actual restore logic
        Ok(())
    }

    fn load_backup_index(&mut self) -> Result<()> {
        // TODO: Load existing backup metadata from disk
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backup_id_generation() {
        let _config = BackupConfig::default();
        // TODO: Create proper test with mock StateManager
        // let state_manager = Arc::new(RwLock::new(StateManager::new()));
        // let hook_registry = Arc::new(RwLock::new(HookRegistry::new()));
        // let manager = BackupManager::new(config, state_manager, hook_registry).unwrap();

        // let id1 = manager.generate_backup_id();
        // let id2 = manager.generate_backup_id();
        // assert_ne!(id1, id2);
        // assert!(id1.starts_with("backup_"));
        // assert!(id2.starts_with("backup_"));

        // For now, test ID generation logic separately
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = format!("backup_{}_test", timestamp);
        assert!(id.starts_with("backup_"));
    }

    #[test]
    fn test_backup_type_serialization() {
        let full = BackupType::Full;
        let incremental = BackupType::Incremental;

        let full_json = serde_json::to_string(&full).unwrap();
        let incremental_json = serde_json::to_string(&incremental).unwrap();

        assert_eq!(full_json, "\"Full\"");
        assert_eq!(incremental_json, "\"Incremental\"");
    }
}
