// ABOUTME: Backup retention policy system for intelligent storage management
// ABOUTME: Implements configurable policies to preserve important backups while managing disk usage

use super::{BackupId, BackupMetadata, BackupType};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tracing::info;

/// Priority level for backup retention
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RetentionPriority {
    /// Never delete (e.g., most recent full backup)
    Critical = 4,
    /// Keep if possible (e.g., checkpoint backups)
    Important = 3,
    /// Normal retention rules apply
    Standard = 2,
    /// Delete first when space needed
    Low = 1,
}

/// Decision made by retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionDecision {
    pub backup_id: BackupId,
    pub should_retain: bool,
    pub priority: RetentionPriority,
    pub reason: String,
}

/// Context for retention policy evaluation
#[derive(Debug, Clone)]
pub struct RetentionContext {
    pub all_backups: Vec<BackupMetadata>,
    pub total_size: u64,
    pub storage_limit: Option<u64>,
    pub current_time: SystemTime,
}

/// Trait for backup retention policies
pub trait RetentionPolicy: Send + Sync {
    /// Evaluate whether a backup should be retained
    fn evaluate(&self, backup: &BackupMetadata, context: &RetentionContext) -> RetentionDecision;

    /// Get policy name for logging
    fn name(&self) -> &str;
}

/// Time-based retention policy
pub struct TimeBasedPolicy {
    max_age: Duration,
}

impl TimeBasedPolicy {
    pub fn new(max_age: Duration) -> Self {
        Self { max_age }
    }
}

impl RetentionPolicy for TimeBasedPolicy {
    fn evaluate(&self, backup: &BackupMetadata, context: &RetentionContext) -> RetentionDecision {
        let age = context
            .current_time
            .duration_since(backup.created_at)
            .unwrap_or(Duration::ZERO);

        let should_retain = age <= self.max_age;
        let priority = if age < self.max_age / 2 {
            RetentionPriority::Important
        } else {
            RetentionPriority::Standard
        };

        RetentionDecision {
            backup_id: backup.id.clone(),
            should_retain,
            priority,
            reason: format!("Age: {:?} (max: {:?})", age, self.max_age),
        }
    }

    fn name(&self) -> &str {
        "TimeBasedPolicy"
    }
}

/// Count-based retention policy
pub struct CountBasedPolicy {
    max_count: usize,
}

impl CountBasedPolicy {
    pub fn new(max_count: usize) -> Self {
        Self { max_count }
    }
}

impl RetentionPolicy for CountBasedPolicy {
    fn evaluate(&self, backup: &BackupMetadata, context: &RetentionContext) -> RetentionDecision {
        // Sort backups by creation time (newest first)
        let mut sorted_backups = context.all_backups.clone();
        sorted_backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let position = sorted_backups
            .iter()
            .position(|b| b.id == backup.id)
            .unwrap_or(usize::MAX);

        let should_retain = position < self.max_count;
        let priority = if position < self.max_count / 2 {
            RetentionPriority::Important
        } else {
            RetentionPriority::Standard
        };

        RetentionDecision {
            backup_id: backup.id.clone(),
            should_retain,
            priority,
            reason: format!("Position: {} (max count: {})", position + 1, self.max_count),
        }
    }

    fn name(&self) -> &str {
        "CountBasedPolicy"
    }
}

/// Importance-based retention policy
pub struct ImportanceBasedPolicy;

impl ImportanceBasedPolicy {
    pub fn new() -> Self {
        Self
    }

    /// Calculate importance score for a backup
    fn calculate_importance(
        &self,
        backup: &BackupMetadata,
        context: &RetentionContext,
    ) -> RetentionPriority {
        // Most recent full backup is critical
        if backup.backup_type == BackupType::Full {
            let is_newest_full = !context
                .all_backups
                .iter()
                .any(|b| b.backup_type == BackupType::Full && b.created_at > backup.created_at);

            if is_newest_full {
                return RetentionPriority::Critical;
            }
        }

        // First backup of the day/week/month is important
        if self.is_checkpoint_backup(backup, &context.all_backups) {
            return RetentionPriority::Important;
        }

        // Full backups are more important than incremental
        if backup.backup_type == BackupType::Full {
            RetentionPriority::Important
        } else {
            RetentionPriority::Standard
        }
    }

    /// Check if backup is a checkpoint (first of day/week/month)
    fn is_checkpoint_backup(
        &self,
        backup: &BackupMetadata,
        all_backups: &[BackupMetadata],
    ) -> bool {
        // This is a simplified check - in production, you'd want proper date parsing
        let backup_time = backup.created_at;

        // Check if it's the first backup of its day
        !all_backups.iter().any(|b| {
            b.id != backup.id
                && b.created_at < backup_time
                && backup_time
                    .duration_since(b.created_at)
                    .map(|d| d < Duration::from_secs(24 * 3600))
                    .unwrap_or(false)
        })
    }
}

impl Default for ImportanceBasedPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl RetentionPolicy for ImportanceBasedPolicy {
    fn evaluate(&self, backup: &BackupMetadata, context: &RetentionContext) -> RetentionDecision {
        let priority = self.calculate_importance(backup, context);
        let should_retain = priority >= RetentionPriority::Important;

        RetentionDecision {
            backup_id: backup.id.clone(),
            should_retain,
            priority,
            reason: format!("Importance: {:?}", priority),
        }
    }

    fn name(&self) -> &str {
        "ImportanceBasedPolicy"
    }
}

/// Composite policy that combines multiple policies
pub struct CompositePolicy {
    policies: Vec<Box<dyn RetentionPolicy>>,
}

impl CompositePolicy {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn add_policy(mut self, policy: Box<dyn RetentionPolicy>) -> Self {
        self.policies.push(policy);
        self
    }
}

impl Default for CompositePolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl RetentionPolicy for CompositePolicy {
    fn evaluate(&self, backup: &BackupMetadata, context: &RetentionContext) -> RetentionDecision {
        let mut should_retain = false;
        let mut highest_priority = RetentionPriority::Low;
        let mut reasons = Vec::new();

        for policy in &self.policies {
            let decision = policy.evaluate(backup, context);

            // If any policy says to retain, we retain
            if decision.should_retain {
                should_retain = true;
            }

            // Use the highest priority from all policies
            if decision.priority > highest_priority {
                highest_priority = decision.priority;
            }

            reasons.push(format!("{}: {}", policy.name(), decision.reason));
        }

        RetentionDecision {
            backup_id: backup.id.clone(),
            should_retain,
            priority: highest_priority,
            reason: reasons.join(", "),
        }
    }

    fn name(&self) -> &str {
        "CompositePolicy"
    }
}

/// Report of retention policy execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionReport {
    pub evaluated_count: usize,
    pub retained_count: usize,
    pub deleted_count: usize,
    pub space_freed: u64,
    pub decisions: Vec<RetentionDecision>,
    pub execution_time: Duration,
}

impl RetentionReport {
    pub fn log_summary(&self) {
        info!(
            "Retention policy executed: evaluated {}, retained {}, deleted {} backups, freed {} bytes in {:?}",
            self.evaluated_count,
            self.retained_count,
            self.deleted_count,
            self.space_freed,
            self.execution_time
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::BackupStats;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_retention_priority_ordering() {
        assert!(RetentionPriority::Critical > RetentionPriority::Important);
        assert!(RetentionPriority::Important > RetentionPriority::Standard);
        assert!(RetentionPriority::Standard > RetentionPriority::Low);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_time_based_policy() {
        let policy = TimeBasedPolicy::new(Duration::from_secs(7 * 24 * 3600)); // 7 days

        let mut backup = BackupMetadata {
            id: "test_backup".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(3 * 24 * 3600), // 3 days old
            backup_type: BackupType::Full,
            parent_id: None,
            schema_version: "1.0.0".to_string(),
            checksums: std::collections::HashMap::new(),
            compression: None,
            encryption: None,
            stats: BackupStats {
                total_entries: 100,
                total_size: 1024,
                duration_ms: 500,
                scopes_backed_up: vec!["global".to_string()],
            },
        };

        let context = RetentionContext {
            all_backups: vec![backup.clone()],
            total_size: 1024,
            storage_limit: None,
            current_time: SystemTime::now(),
        };

        let decision = policy.evaluate(&backup, &context);
        assert!(decision.should_retain);
        assert_eq!(decision.priority, RetentionPriority::Important);

        // Test with old backup
        backup.created_at = SystemTime::now() - Duration::from_secs(10 * 24 * 3600); // 10 days old
        let decision = policy.evaluate(&backup, &context);
        assert!(!decision.should_retain);
    }
}
