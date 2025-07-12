//! ABOUTME: Rate limiting metrics collection and monitoring
//! ABOUTME: Tracks request counts, denials, and rate limit status per provider

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit metrics for a provider
#[derive(Debug, Clone)]
pub struct RateLimitMetrics {
    /// Provider name
    pub provider: String,
    /// Total requests allowed
    pub requests_allowed: u64,
    /// Total requests denied due to rate limits
    pub requests_denied: u64,
    /// Current requests remaining (if known)
    pub requests_remaining: Option<u32>,
    /// Total rate limit for current window (if known)
    pub rate_limit_total: Option<u32>,
    /// Time when metrics were last updated
    pub last_updated: Instant,
    /// Time when rate limit resets
    pub reset_time: Option<Instant>,
    /// Average response time for successful requests
    pub avg_response_time_ms: Option<f64>,
    /// Number of retries performed
    pub total_retries: u64,
    /// Number of requests that succeeded after retry
    pub successful_retries: u64,
}

impl RateLimitMetrics {
    /// Create new metrics for a provider
    fn new(provider: String) -> Self {
        Self {
            provider,
            requests_allowed: 0,
            requests_denied: 0,
            requests_remaining: None,
            rate_limit_total: None,
            last_updated: Instant::now(),
            reset_time: None,
            avg_response_time_ms: None,
            total_retries: 0,
            successful_retries: 0,
        }
    }

    /// Calculate rate limit usage percentage
    pub fn usage_percentage(&self) -> Option<f64> {
        match (self.requests_remaining, self.rate_limit_total) {
            (Some(remaining), Some(total)) if total > 0 => {
                let used = total.saturating_sub(remaining);
                Some((used as f64 / total as f64) * 100.0)
            }
            _ => None,
        }
    }

    /// Check if rate limit is critical (>90% used)
    pub fn is_critical(&self) -> bool {
        self.usage_percentage().map(|p| p > 90.0).unwrap_or(false)
    }

    /// Check if rate limit is warning level (>75% used)
    pub fn is_warning(&self) -> bool {
        self.usage_percentage().map(|p| p > 75.0).unwrap_or(false)
    }
}

/// Collects and manages rate limit metrics
pub struct MetricsCollector {
    /// Metrics per provider
    metrics: Arc<RwLock<HashMap<String, RateLimitMetrics>>>,
    /// Response time tracking
    response_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            response_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record an allowed request
    pub async fn record_allowed(&self, provider: &str) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider.to_string())
            .or_insert_with(|| RateLimitMetrics::new(provider.to_string()));

        entry.requests_allowed += 1;
        entry.last_updated = Instant::now();
    }

    /// Record a denied request
    pub async fn record_denied(&self, provider: &str) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider.to_string())
            .or_insert_with(|| RateLimitMetrics::new(provider.to_string()));

        entry.requests_denied += 1;
        entry.last_updated = Instant::now();
    }

    /// Update remaining requests count
    pub async fn update_remaining(&self, provider: &str, remaining: u32) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider.to_string())
            .or_insert_with(|| RateLimitMetrics::new(provider.to_string()));

        entry.requests_remaining = Some(remaining);
        entry.last_updated = Instant::now();
    }

    /// Update rate limit total
    pub async fn update_limit(&self, provider: &str, limit: u32) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider.to_string())
            .or_insert_with(|| RateLimitMetrics::new(provider.to_string()));

        entry.rate_limit_total = Some(limit);
        entry.last_updated = Instant::now();
    }

    /// Update reset time
    pub async fn update_reset_time(&self, provider: &str, reset_at: Instant) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider.to_string())
            .or_insert_with(|| RateLimitMetrics::new(provider.to_string()));

        entry.reset_time = Some(reset_at);
        entry.last_updated = Instant::now();
    }

    /// Record a response time
    pub async fn record_response_time(&self, provider: &str, duration: Duration) {
        let mut response_times = self.response_times.write().await;
        let times = response_times
            .entry(provider.to_string())
            .or_insert_with(Vec::new);

        times.push(duration);

        // Keep only last 100 response times
        if times.len() > 100 {
            times.drain(0..times.len() - 100);
        }

        // Update average in metrics
        let avg_ms =
            times.iter().map(|d| d.as_secs_f64() * 1000.0).sum::<f64>() / times.len() as f64;

        let mut metrics = self.metrics.write().await;
        if let Some(entry) = metrics.get_mut(provider) {
            entry.avg_response_time_ms = Some(avg_ms);
        }
    }

    /// Record a retry attempt
    pub async fn record_retry(&self, provider: &str, succeeded: bool) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(provider.to_string())
            .or_insert_with(|| RateLimitMetrics::new(provider.to_string()));

        entry.total_retries += 1;
        if succeeded {
            entry.successful_retries += 1;
        }
        entry.last_updated = Instant::now();
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, provider: &str) -> Option<RateLimitMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(provider).cloned()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, RateLimitMetrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get providers with critical rate limit usage
    pub async fn get_critical_providers(&self) -> Vec<String> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .filter(|(_, m)| m.is_critical())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Get providers with warning level usage
    pub async fn get_warning_providers(&self) -> Vec<String> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .filter(|(_, m)| m.is_warning() && !m.is_critical())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Reset metrics for a provider
    pub async fn reset_provider(&self, provider: &str) {
        let mut metrics = self.metrics.write().await;
        if let Some(entry) = metrics.get_mut(provider) {
            *entry = RateLimitMetrics::new(provider.to_string());
        }

        let mut response_times = self.response_times.write().await;
        response_times.remove(provider);
    }

    /// Reset all metrics
    pub async fn reset_all(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();

        let mut response_times = self.response_times.write().await;
        response_times.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // Record some metrics
        collector.record_allowed("test_provider").await;
        collector.record_allowed("test_provider").await;
        collector.record_denied("test_provider").await;
        collector.update_remaining("test_provider", 50).await;
        collector.update_limit("test_provider", 100).await;

        // Check metrics
        let metrics = collector
            .get_provider_metrics("test_provider")
            .await
            .unwrap();
        assert_eq!(metrics.requests_allowed, 2);
        assert_eq!(metrics.requests_denied, 1);
        assert_eq!(metrics.requests_remaining, Some(50));
        assert_eq!(metrics.rate_limit_total, Some(100));
        assert_eq!(metrics.usage_percentage(), Some(50.0));
        assert!(!metrics.is_critical());
        assert!(!metrics.is_warning());

        // Test critical threshold
        collector.update_remaining("test_provider", 5).await;
        let metrics = collector
            .get_provider_metrics("test_provider")
            .await
            .unwrap();
        assert_eq!(metrics.usage_percentage(), Some(95.0));
        assert!(metrics.is_critical());
    }

    #[tokio::test]
    async fn test_response_time_tracking() {
        let collector = MetricsCollector::new();

        // Record response times
        collector
            .record_response_time("api1", Duration::from_millis(100))
            .await;
        collector
            .record_response_time("api1", Duration::from_millis(200))
            .await;
        collector
            .record_response_time("api1", Duration::from_millis(150))
            .await;

        // Check average
        let metrics = collector.get_provider_metrics("api1").await.unwrap();
        assert_eq!(metrics.avg_response_time_ms, Some(150.0));
    }
}
