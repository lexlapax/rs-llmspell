//! ABOUTME: Migration engine - orchestrates migration workflow
//! ABOUTME: Handles pre-flight, backup, batch copy, validation, and rollback

use super::plan::MigrationPlan;
use super::progress::{MigrationProgress, MigrationReport};
use super::traits::{MigrationSource, MigrationTarget};
use super::validator::MigrationValidator;
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use std::sync::Arc;

/// Migration engine - coordinates entire migration workflow
pub struct MigrationEngine {
    source: Arc<dyn MigrationSource>,
    target: Arc<dyn MigrationTarget>,
    plan: MigrationPlan,
    validator: MigrationValidator,
}

impl MigrationEngine {
    /// Create new migration engine
    pub fn new(
        source: Arc<dyn MigrationSource>,
        target: Arc<dyn MigrationTarget>,
        plan: MigrationPlan,
    ) -> Self {
        let validator = MigrationValidator::new(source.clone(), target.clone());

        Self {
            source,
            target,
            plan,
            validator,
        }
    }

    /// Execute migration workflow
    ///
    /// # Arguments
    /// * `dry_run` - If true, performs validation only without data modification
    ///
    /// # Returns
    /// * `Result<MigrationReport>` - Migration report with results
    pub async fn execute(&self, dry_run: bool) -> Result<MigrationReport> {
        let start_time = Utc::now();

        // 1. Pre-flight validation
        let pre_flight = self.validator.pre_flight(&self.plan).await?;
        if !pre_flight.success {
            return Err(anyhow!("Pre-flight failed: {}", pre_flight.summary()));
        }

        if dry_run {
            println!("[DRY-RUN] Pre-flight validation passed");
            println!(
                "[DRY-RUN] Would migrate {} components",
                self.plan.components.len()
            );
            for component in &self.plan.components {
                let count = self.source.count(&component.name).await?;
                println!("[DRY-RUN]   - {}: {} records", component.name, count);
            }

            return Ok(MigrationReport::new(
                true,
                self.plan
                    .components
                    .iter()
                    .map(|c| c.name.clone())
                    .collect(),
                0,
                0,
                Duration::seconds(0),
            ));
        }

        // 2. Backup via BackupManager (Phase 13b.14.2 - will be implemented in next task)
        // TODO: Integrate BackupManager for rollback support

        // 3. Batch copy with progress reporting
        let mut total_source_count = 0;
        let mut total_target_count = 0;

        for component in &self.plan.components {
            println!("Migrating component: {}", component.name);

            // Count source records
            let source_count = self.source.count(&component.name).await?;
            total_source_count += source_count;

            // Create progress tracker
            let mut progress = MigrationProgress::new(component.name.clone(), source_count);

            // List all keys for component
            let keys = self.source.list_keys(&component.name).await?;

            // Batch copy
            for (i, key) in keys.iter().enumerate() {
                // Get source value
                if let Some(value) = self.source.get_value(&component.name, key).await? {
                    // Store in target
                    self.target.store(&component.name, key, &value).await?;
                }

                // Update progress every 100 records
                if (i + 1) % 100 == 0 || (i + 1) == keys.len() {
                    progress.update(i + 1);
                    println!("  {}", progress.format());
                }
            }

            // Final progress update
            progress.update(keys.len());
            println!("  {}", progress.format());

            // Count target records
            let target_count = self.target.count(&component.name).await?;
            total_target_count += target_count;
        }

        // 4. Post-migration validation
        let mut validation_results = Vec::new();
        let all_valid = true;

        for component in &self.plan.components {
            let report = self.validator.validate(&component.name).await?;
            if !report.success {
                // TODO: Rollback via BackupManager (Phase 13b.14.2)
                return Err(anyhow!("Validation failed: {}", report.summary()));
            }
            validation_results.push(report.summary());
        }

        // 5. Generate final report
        let duration = Utc::now().signed_duration_since(start_time);
        let mut report = MigrationReport::new(
            all_valid,
            self.plan
                .components
                .iter()
                .map(|c| c.name.clone())
                .collect(),
            total_source_count,
            total_target_count,
            duration,
        );

        for result in validation_results {
            report.add_validation(result);
        }

        Ok(report)
    }

    /// Rollback migration (restore from backup)
    ///
    /// # Arguments
    /// * `backup_id` - Backup ID to restore from
    /// * `component` - Component to rollback
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Phase 13b.14.2
    /// This will be implemented with BackupManager integration
    pub async fn _rollback(&self, _backup_id: String, _component: &str) -> Result<()> {
        // TODO: Implement BackupManager rollback in Phase 13b.14.2
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock source for testing
    struct MockSource {
        data: std::collections::HashMap<String, Vec<u8>>,
    }

    #[async_trait]
    impl MigrationSource for MockSource {
        async fn list_keys(&self, _component: &str) -> Result<Vec<String>> {
            Ok(self.data.keys().cloned().collect())
        }

        async fn get_value(&self, _component: &str, key: &str) -> Result<Option<Vec<u8>>> {
            Ok(self.data.get(key).cloned())
        }

        async fn count(&self, _component: &str) -> Result<usize> {
            Ok(self.data.len())
        }
    }

    // Mock target for testing
    struct MockTarget {
        data: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    }

    #[async_trait]
    impl MigrationTarget for MockTarget {
        async fn store(&self, _component: &str, key: &str, value: &[u8]) -> Result<()> {
            let mut data = self.data.lock().await;
            data.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn get_value(&self, _component: &str, key: &str) -> Result<Option<Vec<u8>>> {
            let data = self.data.lock().await;
            Ok(data.get(key).cloned())
        }

        async fn count(&self, _component: &str) -> Result<usize> {
            let data = self.data.lock().await;
            Ok(data.len())
        }

        async fn delete(&self, _component: &str, key: &str) -> Result<()> {
            let mut data = self.data.lock().await;
            data.remove(key);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_migration_engine_dry_run() {
        let mut source_data = std::collections::HashMap::new();
        source_data.insert("key1".to_string(), b"value1".to_vec());
        source_data.insert("key2".to_string(), b"value2".to_vec());

        let source = Arc::new(MockSource { data: source_data });
        let target = Arc::new(MockTarget {
            data: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        });

        let plan = MigrationPlan::new("mock", "mock", vec!["test".to_string()]);
        let engine = MigrationEngine::new(source, target.clone(), plan);

        let report = engine.execute(true).await.unwrap();
        assert!(report.success);

        // Verify no data was migrated in dry-run mode
        let target_count = target.count("test").await.unwrap();
        assert_eq!(target_count, 0);
    }

    #[tokio::test]
    async fn test_migration_engine_execute() {
        let mut source_data = std::collections::HashMap::new();
        source_data.insert("key1".to_string(), b"value1".to_vec());
        source_data.insert("key2".to_string(), b"value2".to_vec());

        let source = Arc::new(MockSource { data: source_data });
        let target = Arc::new(MockTarget {
            data: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        });

        let plan = MigrationPlan::new("mock", "mock", vec!["test".to_string()]);
        let engine = MigrationEngine::new(source.clone(), target.clone(), plan);

        let report = engine.execute(false).await.unwrap();
        assert!(report.success);
        assert_eq!(report.source_count, 2);
        assert_eq!(report.target_count, 2);

        // Verify data was migrated
        let target_count = target.count("test").await.unwrap();
        assert_eq!(target_count, 2);
    }
}
