//! ABOUTME: Performance optimizations for workflow bridge operations
//! ABOUTME: Ensures workflow bridge overhead stays under 10ms requirement

use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

lazy_static! {
    /// Cache for workflow type information to avoid repeated discovery
    static ref WORKFLOW_TYPE_CACHE: Arc<RwLock<HashMap<String, WorkflowTypeInfo>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Cached workflow type information
#[derive(Clone, Debug)]
pub struct WorkflowTypeInfo {
    pub workflow_type: String,
    pub description: String,
    pub features: Vec<String>,
    pub required_params: Vec<String>,
    pub optional_params: Vec<String>,
}

/// Validator function type alias
type ValidatorFn = Box<dyn Fn(&Value) -> bool + Send + Sync>;

/// Performance-optimized parameter conversion
pub struct OptimizedConverter {
    /// Pre-compiled parameter validators
    validators: HashMap<String, ValidatorFn>,
}

impl Default for OptimizedConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizedConverter {
    pub fn new() -> Self {
        let mut validators = HashMap::new();

        // Pre-compile common validators
        validators.insert(
            "sequential".to_string(),
            Box::new(|v: &Value| {
                v.get("name").is_some() && v.get("steps").and_then(|s| s.as_array()).is_some()
            }) as ValidatorFn,
        );

        validators.insert(
            "parallel".to_string(),
            Box::new(|v: &Value| {
                v.get("name").is_some() && v.get("branches").and_then(|b| b.as_array()).is_some()
            }) as ValidatorFn,
        );

        validators.insert(
            "conditional".to_string(),
            Box::new(|v: &Value| v.get("name").is_some() && v.get("condition").is_some())
                as ValidatorFn,
        );

        validators.insert(
            "loop".to_string(),
            Box::new(|v: &Value| v.get("name").is_some() && v.get("iterator").is_some())
                as ValidatorFn,
        );

        Self { validators }
    }

    /// Fast parameter validation without full parsing
    pub fn validate_params(&self, workflow_type: &str, params: &Value) -> bool {
        self.validators
            .get(workflow_type)
            .map(|validator| validator(params))
            .unwrap_or(true)
    }
}

/// Workflow execution cache for recently executed workflows
pub struct ExecutionCache {
    /// LRU cache of workflow execution results
    cache: Arc<RwLock<lru::LruCache<String, CachedExecution>>>,
}

#[derive(Clone)]
struct CachedExecution {
    _workflow_id: String,
    result: Value,
    timestamp: std::time::Instant,
}

impl ExecutionCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap(),
            ))),
        }
    }

    /// Get cached execution if available and fresh
    pub fn get(&self, workflow_id: &str) -> Option<Value> {
        let mut cache = self.cache.write();
        if let Some(cached) = cache.get(workflow_id) {
            // Cache entries expire after 60 seconds
            if cached.timestamp.elapsed().as_secs() < 60 {
                return Some(cached.result.clone());
            }
        }
        None
    }

    /// Store execution result in cache
    pub fn put(&self, workflow_id: String, result: Value) {
        let mut cache = self.cache.write();
        cache.put(
            workflow_id.clone(),
            CachedExecution {
                _workflow_id: workflow_id,
                result,
                timestamp: std::time::Instant::now(),
            },
        );
    }
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    metrics: Arc<RwLock<Metrics>>,
}

#[derive(Default)]
struct Metrics {
    total_operations: u64,
    total_duration_ms: u64,
    operation_durations: Vec<u64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
        }
    }

    /// Record operation duration
    pub fn record_operation(&self, duration_ms: u64) {
        let mut metrics = self.metrics.write();
        metrics.total_operations += 1;
        metrics.total_duration_ms += duration_ms;
        metrics.operation_durations.push(duration_ms);

        // Keep only last 1000 operations
        if metrics.operation_durations.len() > 1000 {
            metrics.operation_durations.remove(0);
        }
    }

    /// Get average operation duration
    pub fn average_duration_ms(&self) -> f64 {
        let metrics = self.metrics.read();
        if metrics.total_operations == 0 {
            0.0
        } else {
            metrics.total_duration_ms as f64 / metrics.total_operations as f64
        }
    }

    /// Get p99 operation duration
    pub fn p99_duration_ms(&self) -> u64 {
        let metrics = self.metrics.read();
        if metrics.operation_durations.is_empty() {
            return 0;
        }

        let mut durations = metrics.operation_durations.clone();
        durations.sort_unstable();
        let idx = (durations.len() as f64 * 0.99) as usize;
        durations.get(idx).copied().unwrap_or(0)
    }

    /// Check if performance is within acceptable bounds
    pub fn is_within_bounds(&self) -> bool {
        self.p99_duration_ms() < 10
    }
}

/// Optimized workflow discovery with caching
pub async fn get_workflow_info_cached(workflow_type: &str) -> Option<WorkflowTypeInfo> {
    // Check cache first
    {
        let cache = WORKFLOW_TYPE_CACHE.read();
        if let Some(info) = cache.get(workflow_type) {
            return Some(info.clone());
        }
    }

    // Cache miss - would normally fetch from registry
    // For now, return static info
    let info = match workflow_type {
        "sequential" => WorkflowTypeInfo {
            workflow_type: "sequential".to_string(),
            description: "Execute steps in sequence".to_string(),
            features: vec!["ordered_execution".to_string()],
            required_params: vec!["name".to_string(), "steps".to_string()],
            optional_params: vec!["error_strategy".to_string()],
        },
        "parallel" => WorkflowTypeInfo {
            workflow_type: "parallel".to_string(),
            description: "Execute branches in parallel".to_string(),
            features: vec!["concurrent_execution".to_string()],
            required_params: vec!["name".to_string(), "branches".to_string()],
            optional_params: vec!["max_concurrency".to_string()],
        },
        "conditional" => WorkflowTypeInfo {
            workflow_type: "conditional".to_string(),
            description: "Execute based on conditions".to_string(),
            features: vec!["branching_logic".to_string()],
            required_params: vec!["name".to_string(), "condition".to_string()],
            optional_params: vec!["default_branch".to_string()],
        },
        "loop" => WorkflowTypeInfo {
            workflow_type: "loop".to_string(),
            description: "Execute in a loop".to_string(),
            features: vec!["iteration".to_string()],
            required_params: vec!["name".to_string(), "iterator".to_string()],
            optional_params: vec!["max_iterations".to_string()],
        },
        _ => return None,
    };

    // Store in cache
    {
        let mut cache = WORKFLOW_TYPE_CACHE.write();
        cache.insert(workflow_type.to_string(), info.clone());
    }

    Some(info)
}

#[cfg(test)]
#[cfg_attr(test_category = "bridge")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_optimized_converter() {
        let converter = OptimizedConverter::new();

        // Test valid sequential params
        let params = serde_json::json!({
            "name": "test",
            "steps": []
        });
        assert!(converter.validate_params("sequential", &params));

        // Test invalid sequential params
        let params = serde_json::json!({
            "name": "test"
        });
        assert!(!converter.validate_params("sequential", &params));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_execution_cache() {
        let cache = ExecutionCache::new(10);

        // Store result
        cache.put(
            "workflow1".to_string(),
            serde_json::json!({"result": "success"}),
        );

        // Retrieve result
        let cached = cache.get("workflow1");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()["result"], "success");

        // Non-existent entry
        assert!(cache.get("workflow2").is_none());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();

        // Record some operations
        metrics.record_operation(5);
        metrics.record_operation(8);
        metrics.record_operation(3);

        // Check average
        assert!((metrics.average_duration_ms() - 5.333).abs() < 0.001);

        // Check p99
        assert_eq!(metrics.p99_duration_ms(), 8);

        // Check bounds
        assert!(metrics.is_within_bounds());

        // Record slow operation
        metrics.record_operation(15);
        assert!(!metrics.is_within_bounds());
    }
}
