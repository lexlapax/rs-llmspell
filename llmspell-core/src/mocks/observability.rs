//! Mock implementation of observability framework traits

use crate::error::LLMSpellError;
#[cfg(test)]
use crate::traits::observability::HealthStatus;
use crate::traits::observability::{
    Alert, AlertSeverity, HealthCheck, Metric, MetricType, ObservabilityFramework, SpanHandle,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock observability framework for testing
#[derive(Debug, Default)]
pub struct MockObservabilityFramework {
    metrics: Arc<RwLock<Vec<Metric>>>,
    spans: Arc<RwLock<HashMap<String, SpanHandle>>>,
    health_checks: Arc<RwLock<Vec<HealthCheck>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
}

impl MockObservabilityFramework {
    /// Create a new mock observability framework
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the count of recorded metrics
    pub async fn metric_count(&self) -> usize {
        self.metrics.read().await.len()
    }

    /// Get the count of active alerts
    pub async fn alert_count(&self) -> usize {
        self.alerts.read().await.len()
    }
}

#[async_trait]
impl ObservabilityFramework for MockObservabilityFramework {
    async fn record_metric(&self, metric: Metric) -> Result<(), LLMSpellError> {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        Ok(())
    }

    async fn increment_counter(
        &self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        let metric = Metric {
            name: name.to_string(),
            value: MetricType::Counter(1),
            labels,
            help: None,
            timestamp: chrono::Utc::now(),
        };
        self.record_metric(metric).await
    }

    async fn set_gauge(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        let metric = Metric {
            name: name.to_string(),
            value: MetricType::Gauge(value),
            labels,
            help: None,
            timestamp: chrono::Utc::now(),
        };
        self.record_metric(metric).await
    }

    async fn observe_histogram(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        let metric = Metric {
            name: name.to_string(),
            value: MetricType::Histogram {
                values: vec![value],
                buckets: vec![0.1, 0.5, 1.0, 5.0, 10.0],
            },
            labels,
            help: None,
            timestamp: chrono::Utc::now(),
        };
        self.record_metric(metric).await
    }

    async fn get_metrics(&self, name_prefix: &str) -> Result<Vec<Metric>, LLMSpellError> {
        let metrics = self.metrics.read().await;
        Ok(metrics
            .iter()
            .filter(|m| m.name.starts_with(name_prefix))
            .cloned()
            .collect())
    }

    async fn start_span(
        &self,
        operation_name: &str,
        parent_id: Option<&str>,
    ) -> Result<SpanHandle, LLMSpellError> {
        let span = SpanHandle {
            id: uuid::Uuid::new_v4().to_string(),
            trace_id: parent_id
                .map(ToString::to_string)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            parent_id: parent_id.map(ToString::to_string),
            operation: operation_name.to_string(),
            start_time: chrono::Utc::now(),
            attributes: HashMap::new(),
        };

        let mut spans = self.spans.write().await;
        spans.insert(span.id.clone(), span.clone());

        Ok(span)
    }

    async fn add_span_attributes(
        &self,
        span_id: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        let mut spans = self.spans.write().await;
        let span = spans
            .get_mut(span_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Span not found".to_string(),
                resource_type: Some("span".to_string()),
                source: None,
            })?;
        span.attributes.extend(attributes);
        Ok(())
    }

    async fn add_span_event(
        &self,
        span_id: &str,
        _event_name: &str,
        _attributes: HashMap<String, String>,
    ) -> Result<(), LLMSpellError> {
        let spans = self.spans.read().await;
        if !spans.contains_key(span_id) {
            return Err(LLMSpellError::Resource {
                message: "Span not found".to_string(),
                resource_type: Some("span".to_string()),
                source: None,
            });
        }
        // Mock implementation - just verify span exists
        Ok(())
    }

    async fn register_health_check(&self, _component: &str) -> Result<(), LLMSpellError> {
        // Mock implementation - just return success
        Ok(())
    }

    async fn report_health(&self, check: HealthCheck) -> Result<(), LLMSpellError> {
        let mut health_checks = self.health_checks.write().await;
        health_checks.push(check);
        Ok(())
    }

    async fn get_system_health(&self) -> Result<Vec<HealthCheck>, LLMSpellError> {
        let health_checks = self.health_checks.read().await;
        Ok(health_checks.clone())
    }

    async fn trigger_alert(&self, alert: Alert) -> Result<(), LLMSpellError> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
        Ok(())
    }

    async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), LLMSpellError> {
        let alerts = self.alerts.read().await;
        if !alerts.iter().any(|a| a.id == alert_id) {
            return Err(LLMSpellError::Resource {
                message: "Alert not found".to_string(),
                resource_type: Some("alert".to_string()),
                source: None,
            });
        }
        // Mock implementation - just verify alert exists
        Ok(())
    }

    async fn get_active_alerts(
        &self,
        severity: Option<AlertSeverity>,
    ) -> Result<Vec<Alert>, LLMSpellError> {
        let alerts = self.alerts.read().await;
        Ok(alerts
            .iter()
            .filter(|a| severity.is_none_or(|s| a.severity == s))
            .cloned()
            .collect())
    }

    async fn export_prometheus(&self) -> Result<String, LLMSpellError> {
        let metrics = self.metrics.read().await;
        let mut output = String::new();

        for metric in metrics.iter() {
            if let Some(help) = &metric.help {
                output.push_str(&format!("# HELP {} {}\n", metric.name, help));
            }

            match &metric.value {
                MetricType::Counter(v) => {
                    output.push_str(&format!("# TYPE {} counter\n", metric.name));
                    output.push_str(&format!("{} {}\n", metric.name, v));
                }
                MetricType::Gauge(v) => {
                    output.push_str(&format!("# TYPE {} gauge\n", metric.name));
                    output.push_str(&format!("{} {}\n", metric.name, v));
                }
                _ => {}
            }
        }

        Ok(output)
    }

    async fn export_opentelemetry(&self) -> Result<Vec<u8>, LLMSpellError> {
        // Mock implementation - return empty bytes
        Ok(vec![])
    }

    async fn export_to_collector(&self, _endpoint: &str) -> Result<(), LLMSpellError> {
        // Mock implementation - just return success
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metric_recording() {
        let obs = MockObservabilityFramework::new();

        // Record different metric types
        obs.increment_counter("requests_total", HashMap::new())
            .await
            .unwrap();
        obs.set_gauge("temperature", 23.5, HashMap::new())
            .await
            .unwrap();
        obs.observe_histogram("response_time", 0.250, HashMap::new())
            .await
            .unwrap();

        assert_eq!(obs.metric_count().await, 3);

        // Query metrics
        let metrics = obs.get_metrics("requests").await.unwrap();
        assert_eq!(metrics.len(), 1);
    }

    #[tokio::test]
    async fn test_span_tracing() {
        let obs = MockObservabilityFramework::new();

        // Start root span
        let root = obs.start_span("http_request", None).await.unwrap();
        assert!(root.parent_id.is_none());

        // Start child span
        let child = obs
            .start_span("database_query", Some(&root.id))
            .await
            .unwrap();
        assert_eq!(child.parent_id, Some(root.id.clone()));

        // Add attributes
        let mut attrs = HashMap::new();
        attrs.insert("method".to_string(), "GET".to_string());
        attrs.insert("path".to_string(), "/api/users".to_string());
        obs.add_span_attributes(&root.id, attrs).await.unwrap();

        // Add event
        obs.add_span_event(&root.id, "request_complete", HashMap::new())
            .await
            .unwrap();

        // End span - just verify it returns a duration
        let _duration = child.end();
    }

    #[tokio::test]
    async fn test_health_checks() {
        let obs = MockObservabilityFramework::new();

        // Register component
        obs.register_health_check("database").await.unwrap();

        // Report health
        let check = HealthCheck {
            component: "database".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Connection pool healthy".to_string()),
            timestamp: chrono::Utc::now(),
            details: HashMap::new(),
        };
        obs.report_health(check).await.unwrap();

        // Get system health
        let health = obs.get_system_health().await.unwrap();
        assert_eq!(health.len(), 1);
        assert_eq!(health[0].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_alerting() {
        let obs = MockObservabilityFramework::new();

        // Trigger alerts
        let alert1 = Alert {
            id: "alert-1".to_string(),
            name: "High CPU".to_string(),
            severity: AlertSeverity::Warning,
            message: "CPU usage above 80%".to_string(),
            source: "system-monitor".to_string(),
            triggered_at: chrono::Utc::now(),
            context: HashMap::new(),
        };

        let alert2 = Alert {
            id: "alert-2".to_string(),
            name: "Service Down".to_string(),
            severity: AlertSeverity::Critical,
            message: "Database unreachable".to_string(),
            source: "health-checker".to_string(),
            triggered_at: chrono::Utc::now(),
            context: HashMap::new(),
        };

        obs.trigger_alert(alert1).await.unwrap();
        obs.trigger_alert(alert2).await.unwrap();

        assert_eq!(obs.alert_count().await, 2);

        // Get critical alerts
        let critical_alerts = obs
            .get_active_alerts(Some(AlertSeverity::Critical))
            .await
            .unwrap();
        assert_eq!(critical_alerts.len(), 1);

        // Acknowledge alert
        obs.acknowledge_alert("alert-1").await.unwrap();

        // Try to acknowledge non-existent alert
        assert!(obs.acknowledge_alert("alert-999").await.is_err());
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let obs = MockObservabilityFramework::new();

        obs.increment_counter("http_requests_total", HashMap::new())
            .await
            .unwrap();
        obs.set_gauge("memory_usage_bytes", 1048576.0, HashMap::new())
            .await
            .unwrap();

        let prometheus = obs.export_prometheus().await.unwrap();
        assert!(prometheus.contains("# TYPE http_requests_total counter"));
        assert!(prometheus.contains("# TYPE memory_usage_bytes gauge"));
    }
}
