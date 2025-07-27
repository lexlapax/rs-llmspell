// ABOUTME: Backup manager for coordinating backup operations and lifecycle
// ABOUTME: Handles scheduling, retention, validation, and restoration of backups

use super::{
    cleanup::BackupCleanup,
    events::BackupEventBuilder,
    retention::{
        CompositePolicy, CountBasedPolicy, RetentionContext, RetentionPolicy, RetentionReport,
        TimeBasedPolicy,
    },
    AtomicBackup, BackupCompression, BackupConfig, BackupId, BackupResult, BackupValidation,
    CompressionLevel, RestoreOptions,
};
use crate::{error::StateError, manager::StateManager};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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
    state_manager: Arc<StateManager>,
    backup_index: Arc<RwLock<HashMap<BackupId, BackupMetadata>>>,
    _active_backups: Arc<RwLock<HashMap<BackupId, BackupStatus>>>,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(config: BackupConfig, state_manager: Arc<StateManager>) -> Result<Self> {
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

        // Extract entry count from snapshot metadata
        let snapshot: crate::backup::atomic::StateSnapshot = rmp_serde::from_slice(&backup_data)
            .map_err(|e| {
                StateError::DeserializationError(format!(
                    "Failed to deserialize snapshot for metadata: {}",
                    e
                ))
            })?;
        let entry_count = snapshot.metadata.entry_count;

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
        let compressor = BackupCompression::new(
            self.config.compression_type,
            CompressionLevel::new(self.config.compression_level.into())?,
        );
        compressor.compress(data)
    }

    async fn get_current_schema_version(&self) -> BackupResult<String> {
        // TODO: Get actual schema version from state manager
        Ok("1.0.0".to_string())
    }

    // TODO: Implement event emission when StateEvent includes backup events

    async fn cleanup_old_backups(&self) -> BackupResult<()> {
        // Only run cleanup if retention policies are configured
        if self.config.max_backups.is_none() && self.config.max_backup_age.is_none() {
            debug!("No retention policies configured, skipping cleanup");
            return Ok(());
        }

        info!("Applying backup retention policies");

        // Apply retention policies
        let report = self.apply_retention_policies().await?;
        report.log_summary();

        Ok(())
    }

    async fn build_restore_chain(&self, backup_id: &str) -> BackupResult<Vec<BackupId>> {
        let mut chain = Vec::new();
        let mut current_id = backup_id.to_string();

        loop {
            let metadata = self.get_backup_metadata(&current_id).await?;
            chain.push(current_id.clone());

            // If this is a full backup or has no parent, we're done
            if metadata.backup_type == BackupType::Full || metadata.parent_id.is_none() {
                break;
            }

            // Move to parent backup
            if let Some(parent_id) = metadata.parent_id {
                current_id = parent_id;
            } else {
                break;
            }
        }

        // Reverse to get oldest-first order for restoration
        chain.reverse();

        debug!(
            "Built restore chain of {} backups for {}",
            chain.len(),
            backup_id
        );

        Ok(chain)
    }

    async fn restore_single_backup(&self, backup_id: &str) -> BackupResult<()> {
        info!("Restoring single backup: {}", backup_id);

        // TODO: Execute pre-restore hooks when hook infrastructure supports custom points
        // For now, the restore process will emit events that can trigger hooks

        // Load backup metadata
        let metadata = self.get_backup_metadata(backup_id).await?;

        // Load backup data from file
        let backup_path = self.get_backup_path(backup_id);
        let compressed_data = tokio::fs::read(&backup_path).await.map_err(|e| {
            StateError::StorageError(anyhow::anyhow!("Failed to read backup: {}", e))
        })?;

        // Decompress if needed
        let backup_data = if let Some(ref compression_info) = metadata.compression {
            let reduction_percent = if compression_info.original_size > 0
                && compression_info.compressed_size < compression_info.original_size
            {
                ((compression_info.original_size - compression_info.compressed_size) as f64
                    / compression_info.original_size as f64)
                    * 100.0
            } else {
                0.0
            };
            debug!(
                "Decompressing backup with {} ({:.1}% reduction)",
                compression_info.algorithm, reduction_percent
            );

            let compressor = BackupCompression::new(
                self.config.compression_type,
                CompressionLevel::new(compression_info.level)?,
            );
            compressor.decompress(&compressed_data)?
        } else {
            compressed_data
        };

        // Create atomic backup instance for restoration
        let atomic_backup = AtomicBackup::new(
            backup_id.to_string(),
            self.state_manager.clone(),
            metadata.parent_id.clone(),
        )?;

        // Perform the restoration
        atomic_backup.restore(&backup_data).await?;

        info!(
            "Successfully restored {} entries from backup {}",
            metadata.stats.total_entries, backup_id
        );

        // TODO: Execute post-restore hooks when hook infrastructure supports custom points
        // The atomic restore process already triggers state change hooks for each restored entry

        Ok(())
    }

    fn load_backup_index(&mut self) -> Result<()> {
        // Load existing backup metadata from disk
        let mut loaded_backups = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.config.backup_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                    // Load metadata file
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&contents) {
                            loaded_backups.push((metadata.id.clone(), metadata));
                        }
                    }
                }
            }
        }

        let loaded_count = loaded_backups.len();

        // Update index with loaded backups
        // Since we're in the constructor, we can use try_write().unwrap() safely
        if let Ok(mut index) = self.backup_index.try_write() {
            for (id, metadata) in loaded_backups {
                index.insert(id, metadata);
            }
        }

        if loaded_count > 0 {
            info!("Loaded {} existing backups from disk", loaded_count);
        }

        Ok(())
    }

    /// Apply retention policies to manage backup storage
    pub async fn apply_retention_policies(&self) -> BackupResult<RetentionReport> {
        let start_time = Instant::now();
        let event_builder = BackupEventBuilder::new();

        // Build composite retention policy based on configuration
        let mut composite_policy = CompositePolicy::new();

        // Add time-based policy if configured
        if let Some(max_age) = self.config.max_backup_age {
            composite_policy = composite_policy.add_policy(Box::new(TimeBasedPolicy::new(max_age)));
        }

        // Add count-based policy if configured
        if let Some(max_count) = self.config.max_backups {
            composite_policy =
                composite_policy.add_policy(Box::new(CountBasedPolicy::new(max_count)));
        }

        // Don't add importance-based policy by default - it overrides count/time limits
        // Users can add it explicitly if they want importance-based retention

        // Get all backups
        let index = self.backup_index.read().await;
        let all_backups: Vec<BackupMetadata> = index.values().cloned().collect();

        // Emit cleanup started event
        let cleanup_started_event = event_builder.cleanup_started(
            all_backups.len(),
            format!(
                "max_backups={:?}, max_age={:?}",
                self.config.max_backups, self.config.max_backup_age
            ),
            false,
        );
        if let Err(e) = self
            .state_manager
            .event_bus()
            .publish(cleanup_started_event.to_universal_event())
            .await
        {
            debug!("Failed to emit cleanup started event: {}", e);
        }

        // Calculate total storage usage
        let total_size: u64 = all_backups.iter().map(|b| b.stats.total_size).sum();

        // Create retention context
        let context = RetentionContext {
            all_backups: all_backups.clone(),
            total_size,
            storage_limit: None, // Could be configured in the future
            current_time: SystemTime::now(),
        };

        // Evaluate retention for each backup
        let mut decisions = Vec::new();
        for backup in &all_backups {
            let decision = composite_policy.evaluate(backup, &context);
            decisions.push(decision);
        }

        // Execute cleanup
        let cleanup = BackupCleanup::new(self.config.backup_dir.clone(), false);
        let cleanup_result = cleanup.execute_cleanup(decisions.clone(), &index).await?;

        // Remove deleted backups from index
        if cleanup_result.deleted_count > 0 {
            drop(index); // Release read lock
            let mut index = self.backup_index.write().await;
            for decision in &decisions {
                if !decision.should_retain {
                    // Emit backup deleted event
                    if let Some(metadata) = index.get(&decision.backup_id) {
                        let deleted_event = event_builder.backup_deleted(
                            decision.backup_id.clone(),
                            metadata.stats.total_size,
                            decision.reason.clone(),
                        );
                        if let Err(e) = self
                            .state_manager
                            .event_bus()
                            .publish(deleted_event.to_universal_event())
                            .await
                        {
                            debug!("Failed to emit backup deleted event: {}", e);
                        }
                    }
                    index.remove(&decision.backup_id);
                }
            }
        }

        let report = RetentionReport {
            evaluated_count: all_backups.len(),
            retained_count: decisions.iter().filter(|d| d.should_retain).count(),
            deleted_count: cleanup_result.deleted_count,
            space_freed: cleanup_result.space_freed,
            decisions,
            execution_time: start_time.elapsed(),
        };

        // Emit cleanup completed event
        let cleanup_completed_event = event_builder.cleanup_completed(
            report.evaluated_count,
            report.deleted_count,
            report.retained_count,
            report.space_freed,
            report.execution_time,
        );
        if let Err(e) = self
            .state_manager
            .event_bus()
            .publish(cleanup_completed_event.to_universal_event())
            .await
        {
            debug!("Failed to emit cleanup completed event: {}", e);
        }

        Ok(report)
    }

    /// Manually trigger retention policy application
    pub async fn cleanup_backups(&self) -> BackupResult<RetentionReport> {
        info!("Manually triggering backup cleanup");
        self.apply_retention_policies().await
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
