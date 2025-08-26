//! Tenant usage tracking and cost calculation

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::debug;

/// Usage metrics for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Total number of vectors stored
    pub vector_count: usize,

    /// Total storage used in bytes
    pub storage_bytes: usize,

    /// Total number of queries executed
    pub query_count: u64,

    /// Total number of insertions
    pub insert_count: u64,

    /// Total number of deletions
    pub delete_count: u64,

    /// Total compute time in milliseconds
    pub compute_time_ms: u64,

    /// Last reset timestamp
    pub last_reset: SystemTime,

    /// Usage period start
    pub period_start: SystemTime,
}

impl Default for UsageMetrics {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            vector_count: 0,
            storage_bytes: 0,
            query_count: 0,
            insert_count: 0,
            delete_count: 0,
            compute_time_ms: 0,
            last_reset: now,
            period_start: now,
        }
    }
}

/// Real-time usage tracking
struct RealtimeMetrics {
    /// Current vector count
    vector_count: AtomicUsize,

    /// Current storage bytes
    storage_bytes: AtomicUsize,

    /// Queries in current second
    queries_current_second: AtomicU64,

    /// Timestamp of current second
    current_second: AtomicU64,
}

impl RealtimeMetrics {
    fn new() -> Self {
        Self {
            vector_count: AtomicUsize::new(0),
            storage_bytes: AtomicUsize::new(0),
            queries_current_second: AtomicU64::new(0),
            current_second: AtomicU64::new(0),
        }
    }
}

/// Usage report with cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    /// Tenant ID
    pub tenant_id: String,

    /// Usage metrics
    pub metrics: UsageMetrics,

    /// Estimated cost
    pub estimated_cost: CostEstimate,

    /// Report timestamp
    pub timestamp: SystemTime,
}

/// Cost estimate for usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Storage cost in USD
    pub storage_cost: f64,

    /// Query cost in USD
    pub query_cost: f64,

    /// Compute cost in USD
    pub compute_cost: f64,

    /// Total cost in USD
    pub total_cost: f64,
}

/// Cost rates per unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRates {
    /// Cost per GB per month for storage
    pub storage_per_gb_month: f64,

    /// Cost per 1000 queries
    pub per_1000_queries: f64,

    /// Cost per compute hour
    pub per_compute_hour: f64,
}

impl Default for CostRates {
    fn default() -> Self {
        Self {
            storage_per_gb_month: 0.10, // $0.10 per GB per month
            per_1000_queries: 0.01,     // $0.01 per 1000 queries
            per_compute_hour: 0.05,     // $0.05 per compute hour
        }
    }
}

/// Tenant usage tracker
pub struct TenantUsageTracker {
    /// Usage metrics per tenant
    metrics: DashMap<String, Arc<RwLock<UsageMetrics>>>,

    /// Real-time metrics per tenant
    realtime: DashMap<String, Arc<RealtimeMetrics>>,

    /// Cost rates
    cost_rates: CostRates,
}

impl TenantUsageTracker {
    /// Create a new usage tracker
    pub fn new() -> Self {
        Self {
            metrics: DashMap::new(),
            realtime: DashMap::new(),
            cost_rates: CostRates::default(),
        }
    }

    /// Set custom cost rates
    pub fn with_cost_rates(mut self, rates: CostRates) -> Self {
        self.cost_rates = rates;
        self
    }

    /// Initialize tracking for a new tenant
    pub async fn initialize_tenant(&self, tenant_id: &str) -> Result<UsageMetrics> {
        let metrics = UsageMetrics::default();
        self.metrics.insert(
            tenant_id.to_string(),
            Arc::new(RwLock::new(metrics.clone())),
        );
        self.realtime
            .insert(tenant_id.to_string(), Arc::new(RealtimeMetrics::new()));

        debug!("Initialized usage tracking for tenant {}", tenant_id);
        Ok(metrics)
    }

    /// Remove tenant tracking
    pub async fn remove_tenant(&self, tenant_id: &str) -> Result<()> {
        self.metrics.remove(tenant_id);
        self.realtime.remove(tenant_id);

        debug!("Removed usage tracking for tenant {}", tenant_id);
        Ok(())
    }

    /// Record vector insertion
    pub async fn record_insert(
        &self,
        tenant_id: &str,
        vector_count: usize,
        bytes: usize,
    ) -> Result<()> {
        if let Some(metrics) = self.metrics.get(tenant_id) {
            let mut metrics = metrics.write().await;
            metrics.vector_count += vector_count;
            metrics.storage_bytes += bytes;
            metrics.insert_count += 1;
        }

        if let Some(realtime) = self.realtime.get(tenant_id) {
            realtime
                .vector_count
                .fetch_add(vector_count, Ordering::Relaxed);
            realtime.storage_bytes.fetch_add(bytes, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Record vector deletion
    pub async fn record_delete(&self, tenant_id: &str, vector_count: usize) -> Result<()> {
        if let Some(metrics) = self.metrics.get(tenant_id) {
            let mut metrics = metrics.write().await;
            metrics.vector_count = metrics.vector_count.saturating_sub(vector_count);
            metrics.delete_count += 1;
        }

        if let Some(realtime) = self.realtime.get(tenant_id) {
            realtime.vector_count.fetch_sub(
                vector_count.min(realtime.vector_count.load(Ordering::Relaxed)),
                Ordering::Relaxed,
            );
        }

        Ok(())
    }

    /// Record query execution
    pub async fn record_search(&self, tenant_id: &str) -> Result<()> {
        if let Some(metrics) = self.metrics.get(tenant_id) {
            let mut metrics = metrics.write().await;
            metrics.query_count += 1;
        }

        if let Some(realtime) = self.realtime.get(tenant_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let current = realtime.current_second.load(Ordering::Relaxed);

            if current != now {
                // Reset counter for new second
                realtime.current_second.store(now, Ordering::Relaxed);
                realtime.queries_current_second.store(1, Ordering::Relaxed);
            } else {
                realtime
                    .queries_current_second
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    /// Record compute time
    pub async fn record_compute_time(&self, tenant_id: &str, duration: Duration) -> Result<()> {
        if let Some(metrics) = self.metrics.get(tenant_id) {
            let mut metrics = metrics.write().await;
            metrics.compute_time_ms += duration.as_millis() as u64;
        }

        Ok(())
    }

    /// Get current queries per second
    pub fn get_queries_per_second(&self, tenant_id: &str) -> Result<u32> {
        if let Some(realtime) = self.realtime.get(tenant_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let current = realtime.current_second.load(Ordering::Relaxed);

            if current == now {
                Ok(realtime.queries_current_second.load(Ordering::Relaxed) as u32)
            } else {
                // Different second, no queries yet
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// Get usage metrics for a tenant
    pub async fn get_metrics(&self, tenant_id: &str) -> Result<UsageMetrics> {
        if let Some(metrics) = self.metrics.get(tenant_id) {
            Ok(metrics.read().await.clone())
        } else {
            Ok(UsageMetrics::default())
        }
    }

    /// Generate usage report with cost estimate
    pub async fn generate_report(&self, tenant_id: &str) -> Result<UsageReport> {
        let metrics = self.get_metrics(tenant_id).await?;
        let estimated_cost = self.calculate_cost(&metrics);

        Ok(UsageReport {
            tenant_id: tenant_id.to_string(),
            metrics,
            estimated_cost,
            timestamp: SystemTime::now(),
        })
    }

    /// Calculate cost estimate
    fn calculate_cost(&self, metrics: &UsageMetrics) -> CostEstimate {
        // Calculate time period in hours
        let period_duration = SystemTime::now()
            .duration_since(metrics.period_start)
            .unwrap_or(Duration::from_secs(0));
        let period_hours = period_duration.as_secs_f64() / 3600.0;

        // Storage cost (prorated for the period)
        let storage_gb = metrics.storage_bytes as f64 / 1_073_741_824.0; // Convert to GB
        let storage_cost =
            storage_gb * self.cost_rates.storage_per_gb_month * (period_hours / 720.0); // 720 hours per month

        // Query cost
        let query_cost = (metrics.query_count as f64 / 1000.0) * self.cost_rates.per_1000_queries;

        // Compute cost
        let compute_hours = metrics.compute_time_ms as f64 / 3_600_000.0; // Convert ms to hours
        let compute_cost = compute_hours * self.cost_rates.per_compute_hour;

        CostEstimate {
            storage_cost,
            query_cost,
            compute_cost,
            total_cost: storage_cost + query_cost + compute_cost,
        }
    }

    /// Reset usage metrics for a tenant (e.g., for billing period)
    pub async fn reset_metrics(&self, tenant_id: &str) -> Result<UsageMetrics> {
        if let Some(metrics) = self.metrics.get(tenant_id) {
            let mut metrics = metrics.write().await;
            let old_metrics = metrics.clone();

            // Reset counters but keep current counts
            metrics.query_count = 0;
            metrics.insert_count = 0;
            metrics.delete_count = 0;
            metrics.compute_time_ms = 0;
            metrics.last_reset = SystemTime::now();
            metrics.period_start = SystemTime::now();

            Ok(old_metrics)
        } else {
            Ok(UsageMetrics::default())
        }
    }

    /// Get all tenant reports
    pub async fn get_all_reports(&self) -> Vec<UsageReport> {
        let mut reports = Vec::new();

        for entry in self.metrics.iter() {
            let tenant_id = entry.key().clone();
            if let Ok(report) = self.generate_report(&tenant_id).await {
                reports.push(report);
            }
        }

        reports
    }
}

impl Default for TenantUsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_usage_tracking() {
        let tracker = TenantUsageTracker::new();

        // Initialize tenant
        tracker.initialize_tenant("test-tenant").await.unwrap();

        // Record some operations
        tracker
            .record_insert("test-tenant", 10, 1024)
            .await
            .unwrap();
        tracker.record_search("test-tenant").await.unwrap();
        tracker.record_search("test-tenant").await.unwrap();
        tracker
            .record_compute_time("test-tenant", Duration::from_millis(100))
            .await
            .unwrap();

        // Get metrics
        let metrics = tracker.get_metrics("test-tenant").await.unwrap();
        assert_eq!(metrics.vector_count, 10);
        assert_eq!(metrics.storage_bytes, 1024);
        assert_eq!(metrics.query_count, 2);
        assert_eq!(metrics.insert_count, 1);
        assert_eq!(metrics.compute_time_ms, 100);
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let tracker = TenantUsageTracker::new().with_cost_rates(CostRates {
            storage_per_gb_month: 0.10,
            per_1000_queries: 0.01,
            per_compute_hour: 0.05,
        });

        // Initialize tenant
        tracker.initialize_tenant("test-tenant").await.unwrap();

        // Record usage
        tracker
            .record_insert("test-tenant", 1000, 1_073_741_824)
            .await
            .unwrap(); // 1GB
        for _ in 0..1000 {
            tracker.record_search("test-tenant").await.unwrap();
        }
        tracker
            .record_compute_time("test-tenant", Duration::from_secs(3600))
            .await
            .unwrap(); // 1 hour

        // Generate report
        let report = tracker.generate_report("test-tenant").await.unwrap();

        // Check costs (approximate due to time-based proration)
        assert!(report.estimated_cost.storage_cost > 0.0);
        assert_eq!(report.estimated_cost.query_cost, 0.01); // 1000 queries = $0.01
        assert_eq!(report.estimated_cost.compute_cost, 0.05); // 1 hour = $0.05
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let tracker = TenantUsageTracker::new();

        // Initialize tenant
        tracker.initialize_tenant("test-tenant").await.unwrap();

        // Record multiple queries
        for _ in 0..5 {
            tracker.record_search("test-tenant").await.unwrap();
        }

        // Check QPS
        let qps = tracker.get_queries_per_second("test-tenant").unwrap();
        assert_eq!(qps, 5);
    }
}
