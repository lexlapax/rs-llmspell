// ABOUTME: Simple test to verify retention policies work
// ABOUTME: Isolated test for debugging retention issues

#[cfg(test)]
mod test {
    use crate::backup::retention::*;
    use crate::backup::*;
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_retention_basic() {
        // Create test metadata
        let mut backups = HashMap::new();

        // Add 5 backups with different ages
        for i in 0..5 {
            let id = format!("backup_{}", i);
            let metadata = BackupMetadata {
                id: id.clone(),
                created_at: SystemTime::now() - Duration::from_secs(i * 3600), // Each backup 1 hour older
                backup_type: BackupType::Full,
                parent_id: None,
                schema_version: "1.0.0".to_string(),
                checksums: HashMap::new(),
                compression: None,
                encryption: None,
                stats: BackupStats {
                    total_entries: 10,
                    total_size: 1024,
                    duration_ms: 100,
                    scopes_backed_up: vec!["global".to_string()],
                },
            };
            backups.insert(id, metadata);
        }

        // Create retention policy (keep only 3 backups)
        let policy = CountBasedPolicy::new(3);

        // Create context
        let context = RetentionContext {
            all_backups: backups.values().cloned().collect(),
            total_size: 5 * 1024,
            storage_limit: None,
            current_time: SystemTime::now(),
        };

        // Evaluate each backup
        let mut should_delete = 0;
        let mut should_retain = 0;

        for backup in backups.values() {
            let decision = policy.evaluate(backup, &context);
            if decision.should_retain {
                should_retain += 1;
                println!("RETAIN: {} - {}", backup.id, decision.reason);
            } else {
                should_delete += 1;
                println!("DELETE: {} - {}", backup.id, decision.reason);
            }
        }

        assert_eq!(should_retain, 3);
        assert_eq!(should_delete, 2);
    }
}
