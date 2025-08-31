//! Event throughput performance tests using the unified test framework

use async_trait::async_trait;
use llmspell_events::{EventBus, UniversalEvent, Language};
use llmspell_testing::test_framework::{
    TestExecutor, TestResult, ExecutionContext, ExecutionMode,
    WorkloadClass, TelemetryCollector
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Configuration for event throughput tests
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventTestConfig {
    /// Event pattern for publishing
    pub event_pattern: String,
    /// Number of subscribers
    pub num_subscribers: usize,
    /// Subscriber pattern
    pub subscriber_pattern: String,
    /// Enable rate limiting
    pub rate_limiting: bool,
}

impl Default for EventTestConfig {
    fn default() -> Self {
        Self {
            event_pattern: "test.event".to_string(),
            num_subscribers: 3,
            subscriber_pattern: "test.*".to_string(),
            rate_limiting: false,
        }
    }
}

/// Result from event throughput test
#[derive(Debug, Clone)]
pub struct ThroughputResult {
    pub events_per_second: f64,
    pub total_events: usize,
    pub duration: Duration,
    pub events_received: usize,
    pub success: bool,
}

impl TestResult for ThroughputResult {
    fn is_success(&self) -> bool {
        self.success
    }
    
    fn summary(&self) -> String {
        format!(
            "Throughput: {:.0} events/sec, Total: {}, Duration: {:?}, Received: {}",
            self.events_per_second, self.total_events, self.duration, self.events_received
        )
    }
    
    fn metrics(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "events_per_second": self.events_per_second,
            "total_events": self.total_events,
            "duration_ms": self.duration.as_millis(),
            "events_received": self.events_received,
        }))
    }
}

/// Event throughput test executor
pub struct EventThroughputExecutor {
    event_bus: Arc<EventBus>,
}

impl EventThroughputExecutor {
    /// Create a new event throughput executor
    pub fn new() -> Self {
        Self {
            event_bus: Arc::new(EventBus::new()),
        }
    }
    
    /// Create a test event
    fn create_event(pattern: &str, index: usize) -> UniversalEvent {
        UniversalEvent::new(
            format!("{}.{}", pattern, index % 100),
            serde_json::json!({
                "index": index,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
                "data": format!("event-{}", index)
            }),
            Language::Rust,
        )
    }
}

#[async_trait]
impl TestExecutor for EventThroughputExecutor {
    type Config = EventTestConfig;
    type Result = ThroughputResult;
    
    async fn execute(&self, context: ExecutionContext<Self::Config>) -> Self::Result {
        let workload = self.adapt_workload(context.mode);
        let event_count = workload.event_count();
        let batch_size = workload.batch_size();
        
        // Record start
        let start = Instant::now();
        context.telemetry.record_metric("event_count", event_count as f64);
        
        // Create subscribers if configured
        let mut receivers = Vec::new();
        for i in 0..context.config.num_subscribers {
            match self.event_bus.subscribe(&context.config.subscriber_pattern).await {
                Ok(receiver) => receivers.push(receiver),
                Err(e) => {
                    context.telemetry.record_metric("subscriber_error", 1.0);
                    eprintln!("Failed to create subscriber {}: {}", i, e);
                }
            }
        }
        
        // Spawn receiver tasks to count events
        let (tx, mut rx) = mpsc::channel(event_count);
        for mut receiver in receivers {
            let tx = tx.clone();
            tokio::spawn(async move {
                while let Some(_event) = receiver.recv().await {
                    let _ = tx.send(1).await;
                }
            });
        }
        drop(tx); // Drop original sender
        
        // Publish events in batches
        let publish_timer = context.telemetry.start_operation("publish_events");
        let mut published = 0;
        
        for batch_start in (0..event_count).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(event_count);
            
            for i in batch_start..batch_end {
                let event = Self::create_event(&context.config.event_pattern, i);
                
                // Apply timeout if configured
                let publish_result = if let Some(timeout) = context.timeout {
                    let remaining = timeout.saturating_sub(start.elapsed());
                    if remaining.is_zero() {
                        break; // Timeout reached
                    }
                    tokio::time::timeout(
                        remaining.min(Duration::from_millis(100)),
                        self.event_bus.publish(event)
                    ).await
                } else {
                    Ok(self.event_bus.publish(event).await)
                };
                
                match publish_result {
                    Ok(Ok(_)) => {
                        published += 1;
                        context.telemetry.increment("events_published");
                    }
                    Ok(Err(e)) => {
                        context.telemetry.increment("publish_errors");
                        if context.config.rate_limiting {
                            // Expected in rate limiting scenarios
                            continue;
                        } else {
                            eprintln!("Publish error: {}", e);
                        }
                    }
                    Err(_) => {
                        context.telemetry.increment("publish_timeouts");
                        break; // Timeout
                    }
                }
            }
            
            // Small delay between batches for heavyweight workloads
            if workload.is_heavyweight() && batch_start + batch_size < event_count {
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }
        
        publish_timer.complete();
        
        // Wait for receivers with timeout
        let receive_timer = context.telemetry.start_operation("receive_events");
        let mut received = 0;
        let receive_timeout = Duration::from_secs(5);
        let receive_deadline = Instant::now() + receive_timeout;
        
        while Instant::now() < receive_deadline {
            match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Some(_)) => received += 1,
                Ok(None) => break, // Channel closed
                Err(_) => {
                    // Check if we've received enough
                    if received >= published * context.config.num_subscribers * 80 / 100 {
                        break; // Got 80% of expected events
                    }
                }
            }
        }
        
        receive_timer.complete();
        
        // Calculate results
        let duration = start.elapsed();
        let events_per_second = published as f64 / duration.as_secs_f64();
        
        // Record final metrics
        context.telemetry.record_metric("events_per_second", events_per_second);
        context.telemetry.record_metric("total_published", published as f64);
        context.telemetry.record_metric("total_received", received as f64);
        context.telemetry.record_duration("total_duration", duration);
        
        // Determine success based on workload expectations
        let success = match workload {
            WorkloadClass::Micro => events_per_second > 100.0,
            WorkloadClass::Small => events_per_second > 1_000.0,
            WorkloadClass::Medium => events_per_second > 5_000.0,
            WorkloadClass::Large => events_per_second > 10_000.0,
            WorkloadClass::Stress => events_per_second > 50_000.0,
        };
        
        ThroughputResult {
            events_per_second,
            total_events: published,
            duration,
            events_received: received,
            success,
        }
    }
    
    fn default_config(&self) -> Self::Config {
        EventTestConfig::default()
    }
    
    fn adapt_workload(&self, mode: ExecutionMode) -> WorkloadClass {
        // Custom workload adaptation for event tests
        match mode {
            ExecutionMode::Test => WorkloadClass::Small,    // 1K events
            ExecutionMode::Bench => WorkloadClass::Large,    // 100K events
            ExecutionMode::Stress => WorkloadClass::Stress,  // 1M events
            ExecutionMode::CI => WorkloadClass::Medium,      // 10K events
        }
    }
}

// Tests using the executor
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_throughput_basic() {
        let executor = EventThroughputExecutor::new();
        let context = ExecutionContext::test_default(EventTestConfig::default());
        
        let result = executor.execute(context).await;
        
        assert!(result.is_success(), "Test failed: {}", result.summary());
        assert!(result.events_per_second > 1000.0, 
                "Throughput too low: {:.0} events/sec", result.events_per_second);
    }
    
    #[tokio::test]
    async fn test_high_frequency_events() {
        let executor = EventThroughputExecutor::new();
        
        let config = EventTestConfig {
            event_pattern: "high_freq.event".to_string(),
            num_subscribers: 3,
            subscriber_pattern: "high_freq.*".to_string(),
            rate_limiting: true,
        };
        
        let context = ExecutionContext::test_default(config)
            .with_timeout(Duration::from_secs(5)); // Hard timeout to prevent hanging
        
        let result = executor.execute(context).await;
        
        println!("High frequency test result: {}", result.summary());
        assert!(result.is_success(), "High frequency test failed");
        assert!(result.duration < Duration::from_secs(5), "Test took too long");
    }
    
    #[tokio::test]
    async fn test_with_multiple_subscribers() {
        let executor = EventThroughputExecutor::new();
        
        let config = EventTestConfig {
            event_pattern: "multi.event".to_string(),
            num_subscribers: 10,
            subscriber_pattern: "multi.*".to_string(),
            rate_limiting: false,
        };
        
        let context = ExecutionContext::test_default(config);
        let result = executor.execute(context).await;
        
        assert!(result.is_success(), "Multi-subscriber test failed: {}", result.summary());
    }
    
    #[tokio::test]
    async fn test_micro_workload() {
        let executor = EventThroughputExecutor::new();
        
        // Force micro workload
        let context = ExecutionContext::new(
            EventTestConfig::default(),
            ExecutionMode::Test
        ).with_timeout(Duration::from_millis(500));
        
        let result = executor.execute(context).await;
        
        assert!(result.total_events <= 1000, "Micro workload published too many events");
        assert!(result.duration < Duration::from_secs(1), "Micro workload took too long");
    }
}