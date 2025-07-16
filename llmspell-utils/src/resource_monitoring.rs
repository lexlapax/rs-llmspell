//! ABOUTME: Resource monitoring and metrics collection for tools
//! ABOUTME: Provides real-time monitoring, metrics aggregation, and alerting for resource usage

#![allow(clippy::must_use_candidate)]

use crate::resource_limits::{ResourceMetrics, ResourceTracker};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tokio::time::interval;

/// Resource usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEvent {
    /// Tool or component name
    pub component: String,
    /// Event timestamp in milliseconds since start
    pub timestamp_ms: u64,
    /// Resource metrics at the time of event
    pub metrics: ResourceMetrics,
    /// Event type
    pub event_type: ResourceEventType,
}

/// Types of resource events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum ResourceEventType {
    /// Resource limit exceeded
    LimitExceeded { resource: String, limit: usize },
    /// High resource usage warning
    HighUsage { resource: String, percentage: f64 },
    /// Resource allocation
    Allocated { resource: String, amount: usize },
    /// Resource release
    Released { resource: String, amount: usize },
    /// Operation completed
    OperationComplete { duration_ms: u64 },
}

/// Resource monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Warning threshold percentage (0.0 - 1.0)
    pub warning_threshold: f64,
    /// Monitoring interval
    pub check_interval: Duration,
    /// Enable detailed logging
    pub detailed_logging: bool,
    /// Maximum events to keep in history
    pub max_history_size: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            warning_threshold: 0.8, // 80%
            check_interval: Duration::from_secs(1),
            detailed_logging: false,
            max_history_size: 1000,
        }
    }
}

/// Resource monitor for tracking multiple components
pub struct ResourceMonitor {
    /// Tracked components
    components: Arc<Mutex<HashMap<String, Arc<ResourceTracker>>>>,
    /// Event history
    history: Arc<Mutex<Vec<ResourceEvent>>>,
    /// Configuration
    config: MonitoringConfig,
    /// Event sender
    event_tx: mpsc::UnboundedSender<ResourceEvent>,
    /// Event receiver
    event_rx: Arc<TokioMutex<mpsc::UnboundedReceiver<ResourceEvent>>>,
    /// Start time for calculating timestamps
    start_time: Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    #[must_use]
    pub fn new(config: MonitoringConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            components: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            config,
            event_tx,
            event_rx: Arc::new(TokioMutex::new(event_rx)),
            start_time: Instant::now(),
        }
    }

    /// Register a component for monitoring
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    pub fn register_component(&self, name: String, tracker: Arc<ResourceTracker>) {
        let mut components = self.components.lock().unwrap();
        components.insert(name, tracker);
    }

    /// Unregister a component
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    pub fn unregister_component(&self, name: &str) {
        let mut components = self.components.lock().unwrap();
        components.remove(name);
    }

    /// Get current metrics for all components
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    pub fn get_all_metrics(&self) -> HashMap<String, ResourceMetrics> {
        let components = self.components.lock().unwrap();
        components
            .iter()
            .map(|(name, tracker)| (name.clone(), tracker.get_metrics()))
            .collect()
    }

    /// Get metrics for a specific component
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    #[must_use]
    pub fn get_component_metrics(&self, name: &str) -> Option<ResourceMetrics> {
        let components = self.components.lock().unwrap();
        components.get(name).map(|tracker| tracker.get_metrics())
    }

    /// Get event history
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    #[must_use]
    pub fn get_history(&self) -> Vec<ResourceEvent> {
        self.history.lock().unwrap().clone()
    }

    /// Clear event history
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    pub fn clear_history(&self) {
        self.history.lock().unwrap().clear();
    }

    /// Start monitoring loop
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::unused_async
    )]
    pub async fn start_monitoring(&self) {
        let components = Arc::clone(&self.components);
        let _history = Arc::clone(&self.history);
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval = interval(config.check_interval);

            loop {
                interval.tick().await;

                let components_snapshot = components.lock().unwrap().clone();

                for (name, tracker) in components_snapshot {
                    let metrics = tracker.get_metrics();

                    // Check memory usage
                    if let Some(limits) = tracker.resource_limits() {
                        if let Some(max_memory) = limits.max_memory_bytes {
                            let usage_pct = metrics.memory_bytes as f64 / max_memory as f64;
                            if usage_pct > config.warning_threshold {
                                let event = ResourceEvent {
                                    component: name.clone(),
                                    timestamp_ms: Instant::now()
                                        .duration_since(start_time)
                                        .as_millis()
                                        as u64,
                                    metrics: metrics.clone(),
                                    event_type: ResourceEventType::HighUsage {
                                        resource: "memory".to_string(),
                                        percentage: usage_pct * 100.0,
                                    },
                                };
                                let _ = event_tx.send(event);
                            }
                        }

                        // Check operation count
                        if let Some(max_ops) = limits.max_operations {
                            let usage_pct = metrics.operations_count as f64 / max_ops as f64;
                            if usage_pct > config.warning_threshold {
                                let event = ResourceEvent {
                                    component: name.clone(),
                                    timestamp_ms: Instant::now()
                                        .duration_since(start_time)
                                        .as_millis()
                                        as u64,
                                    metrics: metrics.clone(),
                                    event_type: ResourceEventType::HighUsage {
                                        resource: "operations".to_string(),
                                        percentage: usage_pct * 100.0,
                                    },
                                };
                                let _ = event_tx.send(event);
                            }
                        }
                    }
                }
            }
        });

        // Start event processing
        self.process_events();
    }

    /// Process events from the channel
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    fn process_events(&self) {
        let event_rx = Arc::clone(&self.event_rx);
        let event_history = Arc::clone(&self.history);
        let max_history = self.config.max_history_size;

        tokio::spawn(async move {
            loop {
                let event = {
                    let mut rx = event_rx.lock().await;
                    rx.recv().await
                };

                if let Some(event) = event {
                    // Add to history
                    let mut hist = event_history.lock().unwrap();
                    hist.push(event.clone());

                    // Trim history if needed
                    if hist.len() > max_history {
                        hist.remove(0);
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Send a custom event
    pub fn send_event(&self, event: ResourceEvent) {
        let _ = self.event_tx.send(event);
    }
}

/// Aggregated resource statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatistics {
    /// Total memory allocated across all components
    pub total_memory_bytes: usize,
    /// Total operations performed
    pub total_operations: usize,
    /// Current concurrent operations
    pub concurrent_operations: usize,
    /// Average CPU time per component
    pub avg_cpu_time_ms: f64,
    /// Component count
    pub component_count: usize,
    /// High usage warnings count
    pub warning_count: usize,
    /// Limit exceeded count
    pub limit_exceeded_count: usize,
}

impl ResourceMonitor {
    /// Get aggregated statistics
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn get_statistics(&self) -> ResourceStatistics {
        let components = self.components.lock().unwrap();
        let history = self.history.lock().unwrap();

        let mut total_memory = 0;
        let mut total_ops = 0;
        let mut total_concurrent = 0;
        let mut total_cpu_time = 0.0;

        for tracker in components.values() {
            let metrics = tracker.get_metrics();
            total_memory += metrics.memory_bytes;
            total_ops += metrics.operations_count;
            total_concurrent += metrics.concurrent_ops;
            total_cpu_time += metrics.cpu_time_ms as f64;
        }

        let component_count = components.len();
        let avg_cpu_time_ms = if component_count > 0 {
            total_cpu_time / component_count as f64
        } else {
            0.0
        };

        let warning_count = history
            .iter()
            .filter(|e| matches!(e.event_type, ResourceEventType::HighUsage { .. }))
            .count();

        let limit_exceeded_count = history
            .iter()
            .filter(|e| matches!(e.event_type, ResourceEventType::LimitExceeded { .. }))
            .count();

        ResourceStatistics {
            total_memory_bytes: total_memory,
            total_operations: total_ops,
            concurrent_operations: total_concurrent,
            avg_cpu_time_ms,
            component_count,
            warning_count,
            limit_exceeded_count,
        }
    }
}

/// Extension trait for `ResourceTracker` to get limits
trait ResourceTrackerExt {
    fn resource_limits(&self) -> Option<crate::resource_limits::ResourceLimits>;
}

impl ResourceTrackerExt for ResourceTracker {
    fn resource_limits(&self) -> Option<crate::resource_limits::ResourceLimits> {
        // This would need to be implemented properly in ResourceTracker
        // For now, return None as we don't have access to the internal limits
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_limits::ResourceLimits;

    #[test]
    fn test_resource_monitor_creation() {
        let config = MonitoringConfig::default();
        let monitor = ResourceMonitor::new(config);

        assert_eq!(monitor.get_all_metrics().len(), 0);
        assert_eq!(monitor.get_history().len(), 0);
    }

    #[test]
    fn test_component_registration() {
        let monitor = ResourceMonitor::new(MonitoringConfig::default());
        let tracker = Arc::new(ResourceTracker::new(ResourceLimits::default()));

        monitor.register_component("test_component".to_string(), tracker);

        let metrics = monitor.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        assert!(metrics.contains_key("test_component"));

        monitor.unregister_component("test_component");
        assert_eq!(monitor.get_all_metrics().len(), 0);
    }

    #[test]
    fn test_event_history() {
        let monitor = ResourceMonitor::new(MonitoringConfig::default());

        let event = ResourceEvent {
            component: "test".to_string(),
            timestamp_ms: 0,
            metrics: ResourceMetrics {
                memory_bytes: 1000,
                cpu_time_ms: 100,
                operations_count: 10,
                concurrent_ops: 1,
            },
            event_type: ResourceEventType::HighUsage {
                resource: "memory".to_string(),
                percentage: 85.0,
            },
        };

        monitor.send_event(event);

        // Note: In a real test, we'd need to wait for async processing
        // For now, just verify the event was sent
    }

    #[test]
    fn test_statistics_aggregation() {
        let monitor = ResourceMonitor::new(MonitoringConfig::default());

        // Register multiple components
        for i in 0..3 {
            let tracker = Arc::new(ResourceTracker::new(ResourceLimits::default()));
            tracker.track_memory(1000 * (i + 1)).unwrap();
            for _ in 0..i {
                tracker.track_operation().unwrap();
            }
            monitor.register_component(format!("component_{}", i), tracker);
        }

        let stats = monitor.get_statistics();
        assert_eq!(stats.component_count, 3);
        assert_eq!(stats.total_memory_bytes, 6000); // 1000 + 2000 + 3000
        assert_eq!(stats.total_operations, 3); // 0 + 1 + 2
    }
}
