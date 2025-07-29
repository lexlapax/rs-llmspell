// ABOUTME: Example demonstrating backup retention policies
// ABOUTME: Shows how automatic cleanup maintains backup limits

use llmspell_state_persistence::{
    backup::BackupManager,
    config::{BackupConfig, CompressionType, PersistenceConfig, StorageBackendType},
    StateManager, StateScope,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip logging initialization for this example

    // Create temporary directory for backups
    let temp_dir = TempDir::new()?;
    println!("Using backup directory: {:?}", temp_dir.path());

    // Create state manager
    let state_manager = Arc::new(
        StateManager::with_backend(StorageBackendType::Memory, PersistenceConfig::default())
            .await?,
    );

    // Create backup manager with retention policies
    let backup_config = BackupConfig {
        backup_dir: temp_dir.path().to_path_buf(),
        compression_enabled: false,
        compression_type: CompressionType::None,
        compression_level: 3,
        encryption_enabled: false,
        max_backups: Some(3), // Keep only 3 backups
        max_backup_age: None,
        incremental_enabled: false,
        full_backup_interval: Duration::from_secs(3600),
    };

    let backup_manager = Arc::new(BackupManager::new(backup_config, state_manager.clone())?);

    println!("\nCreating 5 backups with max_backups=3...");

    // Create 5 backups
    for i in 0..5 {
        // Add some state data
        state_manager
            .set(
                StateScope::Global,
                &format!("key_{}", i),
                json!({
                    "iteration": i,
                    "data": format!("Test data for backup {}", i)
                }),
            )
            .await?;

        // Create backup
        let status = backup_manager.create_backup(false).await?;
        println!("Created backup #{}: {}", i + 1, status.id);

        // Small delay to ensure different timestamps
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // List final backups
    println!("\nFinal backup list:");
    let backups = backup_manager.list_backups().await?;
    for (i, backup) in backups.iter().enumerate() {
        println!(
            "  {}. {} (created: {:?})",
            i + 1,
            backup.id,
            backup.created_at
        );
    }

    println!("\nExpected: 3 backups (automatic cleanup should have removed the 2 oldest)");
    println!("Actual: {} backups", backups.len());

    // Manually trigger cleanup to show it's a no-op
    println!("\nManually triggering cleanup...");
    let report = backup_manager.cleanup_backups().await?;
    println!("Cleanup report:");
    println!("  - Evaluated: {} backups", report.evaluated_count);
    println!("  - Deleted: {} backups", report.deleted_count);
    println!("  - Retained: {} backups", report.retained_count);

    // Show retention decisions
    println!("\nRetention decisions:");
    for decision in &report.decisions {
        println!(
            "  - {}: {} ({})",
            decision.backup_id,
            if decision.should_retain {
                "RETAIN"
            } else {
                "DELETE"
            },
            decision.reason
        );
    }

    Ok(())
}
