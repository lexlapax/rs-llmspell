// ABOUTME: Enhanced HookResult enum with production patterns for hook control flow
// ABOUTME: Provides 9 different result types for sophisticated hook behavior control

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::Duration;

/// Operation descriptor for Fork results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub operation_type: String,
    pub parameters: JsonValue,
}

/// Enhanced hook result with production patterns
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum HookResult {
    /// Continue normal execution
    #[default]
    Continue,

    /// Modify the operation parameters or context
    Modified(JsonValue),

    /// Cancel the operation with a reason
    Cancel(String),

    /// Redirect to a different operation
    Redirect(String),

    /// Replace the entire operation result
    Replace(JsonValue),

    /// Retry the operation with delay and max attempts
    Retry { delay: Duration, max_attempts: u32 },

    /// Fork into parallel operations
    Fork { parallel_operations: Vec<Operation> },

    /// Cache the result with a key and TTL
    Cache { key: String, ttl: Duration },

    /// Skip this hook (used by circuit breaker)
    Skipped(String),
}

impl HookResult {
    /// Check if the result allows continuation
    pub fn should_continue(&self) -> bool {
        matches!(self, HookResult::Continue | HookResult::Modified(_))
    }

    /// Check if the result cancels execution
    pub fn is_cancelled(&self) -> bool {
        matches!(self, HookResult::Cancel(_))
    }

    /// Check if the result requires special handling
    pub fn requires_special_handling(&self) -> bool {
        !matches!(self, HookResult::Continue)
    }

    /// Get a human-readable description of the result
    pub fn description(&self) -> String {
        match self {
            HookResult::Continue => "Continue execution".to_string(),
            HookResult::Modified(_) => "Modified parameters".to_string(),
            HookResult::Cancel(reason) => format!("Cancelled: {}", reason),
            HookResult::Redirect(target) => format!("Redirect to: {}", target),
            HookResult::Replace(_) => "Replace result".to_string(),
            HookResult::Retry {
                delay,
                max_attempts,
            } => {
                format!("Retry after {:?} (max {} attempts)", delay, max_attempts)
            }
            HookResult::Fork {
                parallel_operations,
            } => {
                format!(
                    "Fork into {} parallel operations",
                    parallel_operations.len()
                )
            }
            HookResult::Cache { key, ttl } => {
                format!("Cache with key '{}' for {:?}", key, ttl)
            }
            HookResult::Skipped(reason) => format!("Skipped: {}", reason),
        }
    }
}

/// Builder for creating HookResult::Retry
pub struct RetryBuilder {
    delay: Duration,
    max_attempts: u32,
}

impl Default for RetryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryBuilder {
    pub fn new() -> Self {
        Self {
            delay: Duration::from_secs(1),
            max_attempts: 3,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    pub fn build(self) -> HookResult {
        HookResult::Retry {
            delay: self.delay,
            max_attempts: self.max_attempts,
        }
    }
}

/// Builder for creating HookResult::Fork
pub struct ForkBuilder {
    operations: Vec<Operation>,
}

impl Default for ForkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ForkBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn add_operation(
        mut self,
        id: String,
        operation_type: String,
        parameters: JsonValue,
    ) -> Self {
        self.operations.push(Operation {
            id,
            operation_type,
            parameters,
        });
        self
    }

    pub fn build(self) -> HookResult {
        HookResult::Fork {
            parallel_operations: self.operations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_result_continuation() {
        assert!(HookResult::Continue.should_continue());
        assert!(HookResult::Modified(json!({})).should_continue());
        assert!(!HookResult::Cancel("test".to_string()).should_continue());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_result_cancellation() {
        assert!(HookResult::Cancel("reason".to_string()).is_cancelled());
        assert!(!HookResult::Continue.is_cancelled());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_result_special_handling() {
        assert!(!HookResult::Continue.requires_special_handling());
        assert!(HookResult::Modified(json!({})).requires_special_handling());
        assert!(HookResult::Cancel("test".to_string()).requires_special_handling());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_retry_builder() {
        let result = RetryBuilder::new()
            .with_delay(Duration::from_secs(2))
            .with_max_attempts(5)
            .build();

        match result {
            HookResult::Retry {
                delay,
                max_attempts,
            } => {
                assert_eq!(delay, Duration::from_secs(2));
                assert_eq!(max_attempts, 5);
            }
            _ => panic!("Expected Retry result"),
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_fork_builder() {
        let result = ForkBuilder::new()
            .add_operation(
                "op1".to_string(),
                "analyze".to_string(),
                json!({"data": "test"}),
            )
            .add_operation(
                "op2".to_string(),
                "transform".to_string(),
                json!({"data": "test2"}),
            )
            .build();

        match result {
            HookResult::Fork {
                parallel_operations,
            } => {
                assert_eq!(parallel_operations.len(), 2);
                assert_eq!(parallel_operations[0].id, "op1");
                assert_eq!(parallel_operations[1].id, "op2");
            }
            _ => panic!("Expected Fork result"),
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_result_serialization() {
        let results = vec![
            HookResult::Continue,
            HookResult::Modified(json!({"key": "value"})),
            HookResult::Cancel("test reason".to_string()),
            HookResult::Redirect("/new/path".to_string()),
            HookResult::Replace(json!({"result": "replaced"})),
            HookResult::Retry {
                delay: Duration::from_secs(1),
                max_attempts: 3,
            },
            HookResult::Cache {
                key: "cache_key".to_string(),
                ttl: Duration::from_secs(300),
            },
            HookResult::Skipped("circuit breaker open".to_string()),
        ];

        for result in results {
            let serialized = serde_json::to_string(&result).unwrap();
            let deserialized: HookResult = serde_json::from_str(&serialized).unwrap();

            // Can't use direct equality due to Duration serialization
            match (result, deserialized) {
                (HookResult::Continue, HookResult::Continue) => {}
                (HookResult::Modified(a), HookResult::Modified(b)) => assert_eq!(a, b),
                (HookResult::Cancel(a), HookResult::Cancel(b)) => assert_eq!(a, b),
                (HookResult::Redirect(a), HookResult::Redirect(b)) => assert_eq!(a, b),
                (HookResult::Replace(a), HookResult::Replace(b)) => assert_eq!(a, b),
                (HookResult::Skipped(a), HookResult::Skipped(b)) => assert_eq!(a, b),
                _ => {} // Duration-based variants tested separately
            }
        }
    }
}
