//! Session Policies
//!
//! Rate limiting, timeouts, resource management, and other policy enforcement.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Session policy trait
pub trait SessionPolicy: Send + Sync + std::fmt::Debug {
    /// Check if a message/operation is allowed
    ///
    /// # Errors
    ///
    /// Returns an error if policy evaluation fails
    fn check(&self, msg: &serde_json::Value) -> Result<bool>;

    /// Get policy name
    fn name(&self) -> &str;

    /// Reset policy state
    fn reset(&self);
}

/// Rate limiting policy
#[derive(Debug)]
pub struct RateLimitPolicy {
    /// Maximum requests per window
    max_requests: u32,
    /// Time window
    window: Duration,
    /// Request tracking
    requests: Arc<parking_lot::RwLock<Vec<Instant>>>,
}

impl RateLimitPolicy {
    /// Create new rate limit policy
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    /// Clean old requests outside the window
    fn clean_old_requests(&self, now: Instant) {
        let mut requests = self.requests.write();
        requests.retain(|&req_time| now.duration_since(req_time) < self.window);
    }
}

impl SessionPolicy for RateLimitPolicy {
    fn check(&self, _msg: &serde_json::Value) -> Result<bool> {
        let now = Instant::now();

        // Clean old requests
        self.clean_old_requests(now);

        // Check rate limit
        let mut requests = self.requests.write();
        if requests.len() >= self.max_requests as usize {
            warn!(
                "Rate limit exceeded: {} requests in {:?}",
                requests.len(),
                self.window
            );
            return Ok(false);
        }

        // Add current request
        requests.push(now);
        debug!("Rate limit: {}/{}", requests.len(), self.max_requests);

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "RateLimit"
    }

    fn reset(&self) {
        self.requests.write().clear();
    }
}

/// Timeout policy
#[derive(Debug)]
pub struct TimeoutPolicy {
    /// Maximum execution time
    max_duration: Duration,
    /// Execution start times
    executions: Arc<parking_lot::RwLock<HashMap<String, Instant>>>,
}

impl TimeoutPolicy {
    /// Create new timeout policy
    pub fn new(max_duration: Duration) -> Self {
        Self {
            max_duration,
            executions: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Start tracking an execution
    pub fn start_execution(&self, id: String) {
        self.executions.write().insert(id, Instant::now());
    }

    /// Check if execution has timed out
    pub fn check_timeout(&self, id: &str) -> bool {
        if let Some(start_time) = self.executions.read().get(id) {
            if start_time.elapsed() > self.max_duration {
                warn!("Execution {} timed out after {:?}", id, self.max_duration);
                return true;
            }
        }
        false
    }

    /// End tracking an execution
    pub fn end_execution(&self, id: &str) {
        self.executions.write().remove(id);
    }
}

impl SessionPolicy for TimeoutPolicy {
    fn check(&self, msg: &serde_json::Value) -> Result<bool> {
        // Check message type
        let msg_type = msg
            .get("header")
            .and_then(|h| h.get("msg_type"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let msg_id = msg
            .get("header")
            .and_then(|h| h.get("msg_id"))
            .and_then(|id| id.as_str())
            .unwrap_or("");

        match msg_type {
            "execute_request" => {
                // Start tracking execution
                self.start_execution(msg_id.to_string());
                Ok(true)
            }
            "execute_reply" | "execute_result" => {
                // Check timeout and end tracking
                let timed_out = self.check_timeout(msg_id);
                self.end_execution(msg_id);
                Ok(!timed_out)
            }
            _ => {
                // Check ongoing executions for timeout
                let executions = self.executions.read();
                for (id, start_time) in executions.iter() {
                    if start_time.elapsed() > self.max_duration {
                        warn!("Execution {} timed out", id);
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }

    fn name(&self) -> &'static str {
        "Timeout"
    }

    fn reset(&self) {
        self.executions.write().clear();
    }
}

/// Resource limit policy
#[derive(Debug)]
pub struct ResourceLimitPolicy {
    /// Maximum memory in bytes
    max_memory: u64,
    /// Maximum CPU percentage
    max_cpu: f32,
    /// Maximum open files
    max_files: usize,
    /// Current resource usage
    usage: Arc<parking_lot::RwLock<ResourceUsage>>,
}

#[derive(Debug, Clone, Default)]
struct ResourceUsage {
    memory_bytes: u64,
    cpu_percent: f32,
    open_files: usize,
}

impl ResourceLimitPolicy {
    /// Create new resource limit policy
    pub fn new(max_memory: u64, max_cpu: f32, max_files: usize) -> Self {
        Self {
            max_memory,
            max_cpu,
            max_files,
            usage: Arc::new(parking_lot::RwLock::new(ResourceUsage::default())),
        }
    }

    /// Update resource usage
    pub fn update_usage(&self, memory: u64, cpu: f32, files: usize) {
        let mut usage = self.usage.write();
        usage.memory_bytes = memory;
        usage.cpu_percent = cpu;
        usage.open_files = files;
    }

    /// Check if resources are within limits
    pub fn check_limits(&self) -> bool {
        let usage = self.usage.read();

        if usage.memory_bytes > self.max_memory {
            warn!(
                "Memory limit exceeded: {} > {}",
                usage.memory_bytes, self.max_memory
            );
            return false;
        }

        if usage.cpu_percent > self.max_cpu {
            warn!(
                "CPU limit exceeded: {}% > {}%",
                usage.cpu_percent, self.max_cpu
            );
            return false;
        }

        if usage.open_files > self.max_files {
            warn!(
                "Open files limit exceeded: {} > {}",
                usage.open_files, self.max_files
            );
            return false;
        }

        true
    }
}

impl SessionPolicy for ResourceLimitPolicy {
    fn check(&self, _msg: &serde_json::Value) -> Result<bool> {
        Ok(self.check_limits())
    }

    fn name(&self) -> &'static str {
        "ResourceLimit"
    }

    fn reset(&self) {
        *self.usage.write() = ResourceUsage::default();
    }
}

/// Content filter policy
#[derive(Debug)]
pub struct ContentFilterPolicy {
    /// Blocked patterns
    blocked_patterns: Vec<regex::Regex>,
    /// Allowed patterns (whitelist)
    allowed_patterns: Vec<regex::Regex>,
}

impl ContentFilterPolicy {
    /// Create new content filter
    ///
    /// # Errors
    ///
    /// Returns an error if regex patterns are invalid
    pub fn new(blocked: Vec<String>, allowed: Vec<String>) -> Result<Self> {
        let blocked_patterns = blocked
            .into_iter()
            .map(|p| regex::Regex::new(&p))
            .collect::<Result<Vec<_>, _>>()?;

        let allowed_patterns = allowed
            .into_iter()
            .map(|p| regex::Regex::new(&p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            blocked_patterns,
            allowed_patterns,
        })
    }

    /// Check content against filters
    fn check_content(&self, content: &str) -> bool {
        // Check whitelist first if present
        if !self.allowed_patterns.is_empty() {
            let allowed = self.allowed_patterns.iter().any(|p| p.is_match(content));
            if !allowed {
                debug!("Content not in whitelist");
                return false;
            }
        }

        // Check blacklist
        for pattern in &self.blocked_patterns {
            if pattern.is_match(content) {
                warn!("Content matched blocked pattern: {:?}", pattern);
                return false;
            }
        }

        true
    }
}

impl SessionPolicy for ContentFilterPolicy {
    fn check(&self, msg: &serde_json::Value) -> Result<bool> {
        // Extract code content from execute_request
        if let Some(content) = msg
            .get("content")
            .and_then(|c| c.get("code"))
            .and_then(|code| code.as_str())
        {
            return Ok(self.check_content(content));
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "ContentFilter"
    }

    fn reset(&self) {
        // No state to reset
    }
}

/// Composite policy that combines multiple policies
#[derive(Debug)]
pub struct CompositePolicy {
    /// List of policies
    policies: Vec<Arc<dyn SessionPolicy>>,
}

impl CompositePolicy {
    /// Create new composite policy
    pub fn new(policies: Vec<Arc<dyn SessionPolicy>>) -> Self {
        Self { policies }
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: Arc<dyn SessionPolicy>) {
        self.policies.push(policy);
    }
}

impl SessionPolicy for CompositePolicy {
    fn check(&self, msg: &serde_json::Value) -> Result<bool> {
        for policy in &self.policies {
            if !policy.check(msg)? {
                debug!("Policy {} rejected message", policy.name());
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "Composite"
    }

    fn reset(&self) {
        for policy in &self.policies {
            policy.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_policy() {
        let policy = RateLimitPolicy::new(3, Duration::from_secs(1));
        let msg = serde_json::json!({});

        // First 3 requests should pass
        assert!(policy.check(&msg).unwrap());
        assert!(policy.check(&msg).unwrap());
        assert!(policy.check(&msg).unwrap());

        // 4th request should fail
        assert!(!policy.check(&msg).unwrap());

        // Wait for window to pass
        std::thread::sleep(Duration::from_secs(1));

        // Should pass again
        assert!(policy.check(&msg).unwrap());
    }

    #[test]
    fn test_timeout_policy() {
        let policy = TimeoutPolicy::new(Duration::from_millis(100));

        let execute_request = serde_json::json!({
            "header": {
                "msg_type": "execute_request",
                "msg_id": "test-123"
            }
        });

        // Start execution
        assert!(policy.check(&execute_request).unwrap());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Check should fail due to timeout
        let execute_reply = serde_json::json!({
            "header": {
                "msg_type": "execute_reply",
                "msg_id": "test-123"
            }
        });
        assert!(!policy.check(&execute_reply).unwrap());
    }

    #[test]
    fn test_resource_limit_policy() {
        let policy = ResourceLimitPolicy::new(1024 * 1024, 50.0, 10);
        let msg = serde_json::json!({});

        // Within limits
        policy.update_usage(512 * 1024, 25.0, 5);
        assert!(policy.check(&msg).unwrap());

        // Exceed memory limit
        policy.update_usage(2 * 1024 * 1024, 25.0, 5);
        assert!(!policy.check(&msg).unwrap());

        // Reset and check again
        policy.reset();
        assert!(policy.check(&msg).unwrap());
    }

    #[test]
    fn test_content_filter_policy() {
        let policy =
            ContentFilterPolicy::new(vec!["unsafe_.*".to_string()], vec!["safe_.*".to_string()])
                .unwrap();

        // Blocked content
        let blocked_msg = serde_json::json!({
            "content": {
                "code": "unsafe_function()"
            }
        });
        assert!(!policy.check(&blocked_msg).unwrap());

        // Allowed content
        let allowed_msg = serde_json::json!({
            "content": {
                "code": "safe_function()"
            }
        });
        assert!(policy.check(&allowed_msg).unwrap());
    }
}
