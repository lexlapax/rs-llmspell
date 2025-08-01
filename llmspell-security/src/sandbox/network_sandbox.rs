//! ABOUTME: Network access control sandbox
//! ABOUTME: Controls HTTP requests and network access to allowed domains with rate limiting

use super::{SandboxContext, SandboxViolation};
use llmspell_core::{error::LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Network request information
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub timestamp: Instant,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window duration in seconds
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

/// Rate limiter for network requests
#[derive(Debug)]
struct RateLimiter {
    config: RateLimitConfig,
    requests: Vec<Instant>,
}

impl RateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Vec::new(),
        }
    }

    fn check_and_record(&mut self) -> bool {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(self.config.window_seconds);

        // Remove old requests outside the window
        self.requests.retain(|&timestamp| timestamp > window_start);

        // Check if we're under the limit
        if self.requests.len() < self.config.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Network sandbox for controlling network access
pub struct NetworkSandbox {
    context: SandboxContext,
    violations: Vec<SandboxViolation>,
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    default_rate_limit: RateLimitConfig,
}

impl NetworkSandbox {
    /// Create a new network sandbox
    pub fn new(context: SandboxContext) -> Result<Self> {
        Ok(Self {
            context,
            violations: Vec::new(),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            default_rate_limit: RateLimitConfig::default(),
        })
    }

    /// Set default rate limiting configuration
    pub fn with_rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.default_rate_limit = config;
        self
    }

    /// Validate a network request
    pub async fn validate_request(&mut self, url: &str, method: &str) -> Result<()> {
        // Parse URL to extract domain
        let domain = self.extract_domain(url)?;

        // Check if domain is allowed
        if !self.context.is_domain_allowed(&domain) {
            let violation = SandboxViolation::NetworkAccess {
                domain: domain.clone(),
                operation: method.to_string(),
                reason: "Domain not in allowed list".to_string(),
            };
            self.violations.push(violation.clone());
            warn!("Network access violation: {}", violation);
            return Err(LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("network_access".to_string()),
            });
        }

        // Check rate limits
        self.check_rate_limit(&domain).await?;

        debug!("Network request validated: {} {}", method, url);
        Ok(())
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &str) -> Result<String> {
        // Simple URL parsing - in production, you might want to use the `url` crate
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                Ok(after_protocol[..end].to_string())
            } else if let Some(end) = after_protocol.find('?') {
                Ok(after_protocol[..end].to_string())
            } else {
                Ok(after_protocol.to_string())
            }
        } else {
            Err(LLMSpellError::Validation {
                message: format!("Invalid URL format: {}", url),
                field: Some("url".to_string()),
            })
        }
    }

    /// Check rate limits for a domain
    async fn check_rate_limit(&mut self, domain: &str) -> Result<()> {
        let mut limiters = self.rate_limiters.write().await;

        let limiter = limiters
            .entry(domain.to_string())
            .or_insert_with(|| RateLimiter::new(self.default_rate_limit.clone()));

        if !limiter.check_and_record() {
            let violation = SandboxViolation::ResourceLimit {
                resource: "network_requests".to_string(),
                limit: self.default_rate_limit.max_requests as u64,
                actual: limiter.requests.len() as u64,
                reason: format!("Rate limit exceeded for domain: {}", domain),
            };
            self.violations.push(violation.clone());
            warn!("Rate limit violation: {}", violation);
            return Err(LLMSpellError::Security {
                message: violation.to_string(),
                violation_type: Some("rate_limit".to_string()),
            });
        }

        Ok(())
    }

    /// Make a safe HTTP GET request
    pub async fn get(&mut self, url: &str) -> Result<String> {
        self.validate_request(url, "GET").await?;

        debug!("Making GET request to: {}", url);

        // In a real implementation, you would use a proper HTTP client
        // For now, we'll simulate the request
        self.simulate_http_request(url, "GET").await
    }

    /// Make a safe HTTP POST request
    pub async fn post(&mut self, url: &str, body: &str) -> Result<String> {
        self.validate_request(url, "POST").await?;

        debug!(
            "Making POST request to: {} with body length: {}",
            url,
            body.len()
        );

        // In a real implementation, you would use a proper HTTP client
        self.simulate_http_request(url, "POST").await
    }

    /// Make a safe HTTP PUT request
    pub async fn put(&mut self, url: &str, body: &str) -> Result<String> {
        self.validate_request(url, "PUT").await?;

        debug!(
            "Making PUT request to: {} with body length: {}",
            url,
            body.len()
        );

        self.simulate_http_request(url, "PUT").await
    }

    /// Make a safe HTTP DELETE request
    pub async fn delete(&mut self, url: &str) -> Result<String> {
        self.validate_request(url, "DELETE").await?;

        debug!("Making DELETE request to: {}", url);

        self.simulate_http_request(url, "DELETE").await
    }

    /// Simulate HTTP request (placeholder for real implementation)
    async fn simulate_http_request(&self, url: &str, method: &str) -> Result<String> {
        // In a real implementation, this would use reqwest or similar
        // For testing purposes, we'll return a mock response
        tokio::time::sleep(Duration::from_millis(100)).await; // Simulate network delay

        Ok(format!(
            "{{\"mock_response\": true, \"url\": \"{}\", \"method\": \"{}\", \"status\": 200}}",
            url, method
        ))
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let limiters = self.rate_limiters.read().await;
        let mut domain_stats = HashMap::new();

        for (domain, limiter) in limiters.iter() {
            domain_stats.insert(
                domain.clone(),
                DomainStats {
                    recent_requests: limiter.requests.len() as u32,
                    window_seconds: limiter.config.window_seconds,
                    max_requests: limiter.config.max_requests,
                },
            );
        }

        NetworkStats {
            total_violations: self.violations.len(),
            domain_stats,
        }
    }

    /// Get all violations that occurred
    pub fn get_violations(&self) -> &[SandboxViolation] {
        &self.violations
    }

    /// Clear violations history
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
}

/// Network statistics
#[derive(Debug)]
pub struct NetworkStats {
    pub total_violations: usize,
    pub domain_stats: HashMap<String, DomainStats>,
}

/// Domain-specific statistics
#[derive(Debug)]
pub struct DomainStats {
    pub recent_requests: u32,
    pub window_seconds: u64,
    pub max_requests: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};

    fn create_test_sandbox() -> NetworkSandbox {
        let security_reqs = SecurityRequirements::safe()
            .with_network_access("api.example.com")
            .with_network_access("github.com");

        let context = SandboxContext::new(
            "test-sandbox".to_string(),
            security_reqs,
            ResourceLimits::strict(),
        );

        NetworkSandbox::new(context).unwrap()
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_domain_validation() {
        let mut sandbox = create_test_sandbox();

        // Valid domain
        assert!(sandbox
            .validate_request("https://api.example.com/data", "GET")
            .await
            .is_ok());

        // Invalid domain
        assert!(sandbox
            .validate_request("https://malicious.com/data", "GET")
            .await
            .is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_domain_extraction() {
        let sandbox = create_test_sandbox();

        assert_eq!(
            sandbox
                .extract_domain("https://api.example.com/v1/data")
                .unwrap(),
            "api.example.com"
        );
        assert_eq!(
            sandbox.extract_domain("http://github.com").unwrap(),
            "github.com"
        );
        assert_eq!(
            sandbox
                .extract_domain("https://sub.domain.com/path?query=1")
                .unwrap(),
            "sub.domain.com"
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_http_methods() {
        let mut sandbox = create_test_sandbox();

        // Test all HTTP methods
        assert!(sandbox.get("https://api.example.com/data").await.is_ok());
        assert!(sandbox
            .post("https://api.example.com/data", "{}")
            .await
            .is_ok());
        assert!(sandbox
            .put("https://api.example.com/data", "{}")
            .await
            .is_ok());
        assert!(sandbox.delete("https://api.example.com/data").await.is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_rate_limiting() {
        let rate_config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 1,
        };

        let mut sandbox = create_test_sandbox().with_rate_limit(rate_config);

        // First two requests should succeed
        assert!(sandbox.get("https://api.example.com/1").await.is_ok());
        assert!(sandbox.get("https://api.example.com/2").await.is_ok());

        // Third request should fail due to rate limit
        assert!(sandbox.get("https://api.example.com/3").await.is_err());

        // Check that violation was recorded
        assert!(!sandbox.get_violations().is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_network_stats() {
        let mut sandbox = create_test_sandbox();

        // Make some requests
        let _ = sandbox.get("https://api.example.com/data").await;
        let _ = sandbox.get("https://github.com/repo").await;

        let stats = sandbox.get_network_stats().await;
        assert_eq!(stats.domain_stats.len(), 2);
        assert!(stats.domain_stats.contains_key("api.example.com"));
        assert!(stats.domain_stats.contains_key("github.com"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_violation_tracking() {
        let mut sandbox = create_test_sandbox();

        // Make an invalid request
        let _ = sandbox.get("https://blocked.com/data").await;

        // Check violations
        let violations = sandbox.get_violations();
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            SandboxViolation::NetworkAccess { domain, .. } => {
                assert_eq!(domain, "blocked.com");
            }
            _ => panic!("Expected NetworkAccess violation"),
        }

        // Clear violations
        sandbox.clear_violations();
        assert!(sandbox.get_violations().is_empty());
    }
}
