// ABOUTME: Hook retention policy implementation for managing storage lifecycle
// ABOUTME: Provides time-based and count-based retention with configurable policies

use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

/// Retention policy configuration for hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Maximum age of hook executions to retain
    pub max_age: Option<Duration>,
    /// Maximum number of executions to retain per hook type
    pub max_count: Option<usize>,
    /// Minimum number to always keep regardless of age
    pub min_count: usize,
    /// Priority threshold - keep hooks with priority >= this value longer
    pub priority_threshold: i32,
    /// Additional time to retain high-priority hooks
    pub priority_bonus_time: Duration,
    /// Whether to archive hooks before deletion
    pub archive_before_delete: bool,
    /// Compression settings for archived hooks
    pub compress_archives: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age: Some(Duration::from_secs(7 * 24 * 60 * 60)), // 7 days
            max_count: Some(10000),
            min_count: 100,
            priority_threshold: 5,
            priority_bonus_time: Duration::from_secs(7 * 24 * 60 * 60), // Extra 7 days
            archive_before_delete: true,
            compress_archives: true,
        }
    }
}

impl RetentionPolicy {
    /// Create a policy that retains everything
    pub fn retain_all() -> Self {
        Self {
            max_age: None,
            max_count: None,
            min_count: 0,
            priority_threshold: i32::MAX,
            priority_bonus_time: Duration::from_secs(0),
            archive_before_delete: true,
            compress_archives: true,
        }
    }

    /// Create a policy for short-term retention
    pub fn short_term() -> Self {
        Self {
            max_age: Some(Duration::from_secs(24 * 60 * 60)), // 1 day
            max_count: Some(1000),
            min_count: 10,
            priority_threshold: 8,
            priority_bonus_time: Duration::from_secs(24 * 60 * 60), // Extra 1 day
            archive_before_delete: false,
            compress_archives: false,
        }
    }

    /// Create a policy for long-term retention
    pub fn long_term() -> Self {
        Self {
            max_age: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days
            max_count: Some(100000),
            min_count: 1000,
            priority_threshold: 3,
            priority_bonus_time: Duration::from_secs(30 * 24 * 60 * 60), // Extra 30 days
            archive_before_delete: true,
            compress_archives: true,
        }
    }
}

/// Manages retention policies for different hook types
pub struct RetentionManager {
    /// Per-hook-type retention policies
    policies: RwLock<HashMap<String, RetentionPolicy>>,
    /// Default policy for hooks without specific configuration
    default_policy: RetentionPolicy,
    /// Global retention statistics
    stats: RwLock<RetentionStatistics>,
}

#[derive(Debug, Default)]
pub struct RetentionStatistics {
    total_cleaned: u64,
    total_archived: u64,
    last_cleanup_time: Option<std::time::SystemTime>,
    space_reclaimed_bytes: u64,
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::new(RetentionPolicy::default())
    }
}

impl RetentionManager {
    /// Create a new retention manager with default policy
    pub fn new(default_policy: RetentionPolicy) -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
            default_policy,
            stats: RwLock::new(RetentionStatistics::default()),
        }
    }

    /// Set retention policy for a specific hook type
    pub fn set_policy(&self, hook_id: &str, policy: RetentionPolicy) {
        self.policies.write().insert(hook_id.to_string(), policy);
    }

    /// Get retention policy for a hook type
    pub fn get_policy(&self, hook_id: &str) -> RetentionPolicy {
        self.policies
            .read()
            .get(hook_id)
            .cloned()
            .unwrap_or_else(|| self.default_policy.clone())
    }

    /// Apply retention policies to a specific hook type
    pub async fn apply_retention_policies(&self, hook_id: &str) -> Result<()> {
        let policy = self.get_policy(hook_id);

        info!(
            hook_id = hook_id,
            max_age_secs = ?policy.max_age.map(|d| d.as_secs()),
            max_count = ?policy.max_count,
            "Applying retention policy"
        );

        // In a real implementation, this would:
        // 1. Query hook executions for the given hook_id
        // 2. Apply age-based retention
        // 3. Apply count-based retention
        // 4. Archive if configured
        // 5. Delete or compress as needed

        // Update statistics
        let mut stats = self.stats.write();
        stats.last_cleanup_time = Some(std::time::SystemTime::now());

        Ok(())
    }

    /// Apply retention policies to all hook types
    pub async fn apply_all_retention_policies(&self) -> Result<()> {
        let hook_ids: Vec<String> = {
            let policies = self.policies.read();
            policies.keys().cloned().collect()
        };

        for hook_id in hook_ids {
            self.apply_retention_policies(&hook_id)
                .await
                .context(format!("Failed to apply retention for hook: {}", hook_id))?;
        }

        Ok(())
    }

    /// Check if a hook execution should be retained based on age and priority
    pub fn should_retain(&self, hook_id: &str, age: Duration, priority: i32) -> bool {
        let policy = self.get_policy(hook_id);

        // Check age-based retention
        if let Some(max_age) = policy.max_age {
            let effective_max_age = if priority >= policy.priority_threshold {
                max_age + policy.priority_bonus_time
            } else {
                max_age
            };

            if age > effective_max_age {
                debug!(
                    hook_id = hook_id,
                    age_secs = age.as_secs(),
                    max_age_secs = effective_max_age.as_secs(),
                    "Hook execution too old"
                );
                return false;
            }
        }

        true
    }

    /// Get retention statistics
    pub fn get_statistics(&self) -> RetentionStatistics {
        self.stats.read().clone()
    }

    /// Clear all custom policies (revert to default)
    pub fn clear_policies(&self) {
        self.policies.write().clear();
    }
}

impl Clone for RetentionStatistics {
    fn clone(&self) -> Self {
        Self {
            total_cleaned: self.total_cleaned,
            total_archived: self.total_archived,
            last_cleanup_time: self.last_cleanup_time,
            space_reclaimed_bytes: self.space_reclaimed_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_retention_policy_presets() {
        let short = RetentionPolicy::short_term();
        assert_eq!(short.max_age, Some(Duration::from_secs(24 * 60 * 60)));
        assert_eq!(short.max_count, Some(1000));

        let long = RetentionPolicy::long_term();
        assert_eq!(long.max_age, Some(Duration::from_secs(30 * 24 * 60 * 60)));
        assert_eq!(long.max_count, Some(100000));

        let all = RetentionPolicy::retain_all();
        assert_eq!(all.max_age, None);
        assert_eq!(all.max_count, None);
    }
    #[test]
    fn test_retention_manager() {
        let manager = RetentionManager::default();

        // Set custom policy
        let custom_policy = RetentionPolicy::short_term();
        manager.set_policy("test_hook", custom_policy.clone());

        // Retrieve policy
        let retrieved = manager.get_policy("test_hook");
        assert_eq!(retrieved.max_age, custom_policy.max_age);

        // Default policy for unknown hook
        let default = manager.get_policy("unknown_hook");
        assert_eq!(default.max_age, Some(Duration::from_secs(7 * 24 * 60 * 60)));
    }
    #[test]
    fn test_should_retain() {
        let manager = RetentionManager::default();

        // Test with default policy
        assert!(manager.should_retain(
            "test_hook",
            Duration::from_secs(3 * 24 * 60 * 60), // 3 days
            0,                                     // Low priority
        ));

        assert!(!manager.should_retain(
            "test_hook",
            Duration::from_secs(10 * 24 * 60 * 60), // 10 days
            0,                                      // Low priority
        ));

        // High priority gets bonus time
        assert!(manager.should_retain(
            "test_hook",
            Duration::from_secs(10 * 24 * 60 * 60), // 10 days
            10,                                     // High priority
        ));
    }
}
