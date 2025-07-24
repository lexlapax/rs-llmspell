//! ABOUTME: Event correlation system for tracking hook execution across component boundaries
//! ABOUTME: Provides distributed tracing and event correlation for complex cross-component scenarios

//! # Event Correlation for Cross-Component Hook Execution
//!
//! This module provides event correlation and distributed tracing capabilities for
//! hook execution across different components. It allows tracking the flow of execution
//! through agent -> tool -> workflow chains, providing observability and debugging support.
//!
//! ## Features
//!
//! - **Distributed Tracing**: Track execution across component boundaries
//! - **Event Correlation**: Correlate related events using unique IDs
//! - **Causality Tracking**: Understand cause-and-effect relationships
//! - **Performance Analysis**: Analyze execution timing across components
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_hooks::coordination::{EventCorrelator, CorrelationId};
//! use llmspell_hooks::{ComponentId, ComponentType, HookPoint};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let correlator = EventCorrelator::new();
//!
//! // Create a correlation for tracking a cross-component execution
//! let correlation_id = correlator.create_correlation().await;
//!
//! // Track events as they happen
//! let agent_id = ComponentId::new(ComponentType::Agent, "gpt-4".to_string());
//! correlator.record_event(
//!     &correlation_id,
//!     &agent_id,
//!     HookPoint::BeforeAgentExecution,
//!     "Starting agent execution"
//! ).await?;
//!
//! // Get the full trace
//! let trace = correlator.get_trace(&correlation_id).await?;
//! println!("Execution trace: {:?}", trace);
//! # Ok(())
//! # }
//! ```

use crate::{ComponentId, HookPoint};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Unique identifier for correlating events across components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    /// Creates a new correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a correlation ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Gets the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Creates a child correlation ID for nested operations
    pub fn create_child(&self) -> Self {
        // Simply create a new UUID for child - they should be unique
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Event correlator for tracking cross-component hook execution
#[derive(Debug)]
pub struct EventCorrelator {
    /// Active traces by correlation ID
    traces: RwLock<HashMap<CorrelationId, EventTrace>>,
    /// Configuration for the correlator
    config: CorrelatorConfig,
}

/// Configuration for event correlation
#[derive(Debug, Clone)]
pub struct CorrelatorConfig {
    /// Maximum number of events per trace
    pub max_events_per_trace: usize,
    /// Maximum number of active traces
    pub max_active_traces: usize,
    /// Trace retention duration
    pub trace_retention: Duration,
    /// Enable detailed timing information
    pub enable_detailed_timing: bool,
    /// Enable causality tracking
    pub enable_causality_tracking: bool,
}

impl Default for CorrelatorConfig {
    fn default() -> Self {
        Self {
            max_events_per_trace: 1000,
            max_active_traces: 10000,
            trace_retention: Duration::from_secs(24 * 60 * 60), // 24 hours
            enable_detailed_timing: true,
            enable_causality_tracking: true,
        }
    }
}

/// A complete trace of events for a correlation ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrace {
    /// The correlation ID for this trace
    pub correlation_id: CorrelationId,
    /// All events in chronological order
    pub events: Vec<TraceEvent>,
    /// Components involved in this trace
    pub components: Vec<ComponentId>,
    /// Metadata for the trace
    pub metadata: HashMap<String, String>,
    /// Trace creation time
    pub created_at: SystemTime,
    /// Last activity time
    pub last_activity: SystemTime,
    /// Trace status
    pub status: TraceStatus,
}

/// Individual event within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Event ID for uniqueness
    pub event_id: Uuid,
    /// Component that generated this event
    pub component_id: ComponentId,
    /// Hook point where this event occurred
    pub hook_point: HookPoint,
    /// Event type/category
    pub event_type: EventType,
    /// Human-readable event message
    pub message: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Duration from trace start
    pub elapsed: Duration,
    /// Optional parent event (for causality)
    pub parent_event_id: Option<Uuid>,
    /// Event-specific data
    pub data: HashMap<String, serde_json::Value>,
    /// Performance metrics at time of event
    pub metrics: Option<EventMetrics>,
}

/// Type of trace event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Hook execution started
    HookStart,
    /// Hook execution completed successfully
    HookComplete,
    /// Hook execution failed
    HookError,
    /// Component initialization
    ComponentInit,
    /// Component cleanup
    ComponentCleanup,
    /// Context propagation event
    ContextPropagation,
    /// Performance warning
    PerformanceWarning,
    /// Custom user-defined event
    Custom(String),
}

/// Status of a trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceStatus {
    /// Trace is actively collecting events
    Active,
    /// Trace completed successfully
    Completed,
    /// Trace ended due to error
    Failed,
    /// Trace was abandoned/cancelled
    Abandoned,
    /// Trace exceeded retention period
    Expired,
}

/// Performance metrics captured at event time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    /// Memory usage at event time (bytes)
    pub memory_usage: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage: Option<f64>,
    /// Number of active threads
    pub thread_count: Option<u32>,
    /// Component-specific metrics
    pub component_metrics: HashMap<String, f64>,
}

impl EventCorrelator {
    /// Creates a new event correlator
    pub fn new() -> Self {
        Self::with_config(CorrelatorConfig::default())
    }

    /// Creates a new correlator with custom configuration
    pub fn with_config(config: CorrelatorConfig) -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Creates a new correlation ID for tracking events
    pub async fn create_correlation(&self) -> CorrelationId {
        let correlation_id = CorrelationId::new();

        let trace = EventTrace {
            correlation_id: correlation_id.clone(),
            events: Vec::new(),
            components: Vec::new(),
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            status: TraceStatus::Active,
        };

        let mut traces = self.traces.write().await;

        // Clean up expired traces if we're at capacity
        if traces.len() >= self.config.max_active_traces {
            self.cleanup_expired_traces(&mut traces).await;
        }

        traces.insert(correlation_id.clone(), trace);

        info!(
            correlation_id = %correlation_id,
            "Created new event correlation"
        );

        correlation_id
    }

    /// Starts tracking a chain of components
    pub async fn start_chain_trace(
        &self,
        correlation_id: CorrelationId,
        components: Vec<ComponentId>,
    ) -> Result<()> {
        let mut traces = self.traces.write().await;

        let trace = traces
            .get_mut(&correlation_id)
            .ok_or_else(|| anyhow::anyhow!("Correlation ID not found: {}", correlation_id))?;

        trace.components = components.clone();
        trace
            .metadata
            .insert("chain_type".to_string(), "cross_component".to_string());
        trace
            .metadata
            .insert("component_count".to_string(), components.len().to_string());

        debug!(
            correlation_id = %correlation_id,
            components = ?components,
            "Started chain trace"
        );

        Ok(())
    }

    /// Records an event in the trace
    pub async fn record_event(
        &self,
        correlation_id: &CorrelationId,
        component_id: &ComponentId,
        hook_point: HookPoint,
        message: impl Into<String>,
    ) -> Result<Uuid> {
        self.record_event_with_details(
            correlation_id,
            component_id,
            hook_point,
            EventType::Custom("hook_execution".to_string()),
            message,
            None,
            HashMap::new(),
        )
        .await
    }

    /// Records an event with full details
    #[allow(clippy::too_many_arguments)]
    pub async fn record_event_with_details(
        &self,
        correlation_id: &CorrelationId,
        component_id: &ComponentId,
        hook_point: HookPoint,
        event_type: EventType,
        message: impl Into<String>,
        parent_event_id: Option<Uuid>,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<Uuid> {
        let event_id = Uuid::new_v4();
        let now = SystemTime::now();
        let message = message.into();

        let mut traces = self.traces.write().await;
        let trace = traces
            .get_mut(correlation_id)
            .ok_or_else(|| anyhow::anyhow!("Correlation ID not found: {}", correlation_id))?;

        if trace.status != TraceStatus::Active {
            warn!(
                correlation_id = %correlation_id,
                status = ?trace.status,
                "Attempted to record event on inactive trace"
            );
            return Err(anyhow::anyhow!("Trace is not active"));
        }

        // Check event limit
        if trace.events.len() >= self.config.max_events_per_trace {
            warn!(
                correlation_id = %correlation_id,
                event_count = trace.events.len(),
                "Trace exceeded maximum events limit"
            );
            trace.status = TraceStatus::Failed;
            return Err(anyhow::anyhow!("Trace exceeded maximum events"));
        }

        let elapsed = now
            .duration_since(trace.created_at)
            .unwrap_or(Duration::ZERO);

        let metrics = if self.config.enable_detailed_timing {
            Some(self.collect_event_metrics(component_id).await)
        } else {
            None
        };

        let event = TraceEvent {
            event_id,
            component_id: component_id.clone(),
            hook_point: hook_point.clone(),
            event_type: event_type.clone(),
            message: message.clone(),
            timestamp: now,
            elapsed,
            parent_event_id,
            data,
            metrics,
        };

        trace.events.push(event);
        trace.last_activity = now;

        // Add component if not already tracked
        if !trace.components.contains(component_id) {
            trace.components.push(component_id.clone());
        }

        debug!(
            correlation_id = %correlation_id,
            event_id = %event_id,
            component_id = ?component_id,
            hook_point = ?hook_point,
            event_type = ?event_type,
            message = %message,
            "Recorded trace event"
        );

        Ok(event_id)
    }

    /// Marks a trace as completed
    pub async fn complete_trace(&self, correlation_id: &CorrelationId) -> Result<()> {
        let mut traces = self.traces.write().await;
        let trace = traces
            .get_mut(correlation_id)
            .ok_or_else(|| anyhow::anyhow!("Correlation ID not found: {}", correlation_id))?;

        trace.status = TraceStatus::Completed;
        trace.last_activity = SystemTime::now();

        info!(
            correlation_id = %correlation_id,
            event_count = trace.events.len(),
            component_count = trace.components.len(),
            "Completed event trace"
        );

        Ok(())
    }

    /// Marks a trace as failed
    pub async fn fail_trace(
        &self,
        correlation_id: &CorrelationId,
        error: impl Into<String>,
    ) -> Result<()> {
        let mut traces = self.traces.write().await;
        let trace = traces
            .get_mut(correlation_id)
            .ok_or_else(|| anyhow::anyhow!("Correlation ID not found: {}", correlation_id))?;

        trace.status = TraceStatus::Failed;
        trace.last_activity = SystemTime::now();
        trace
            .metadata
            .insert("failure_reason".to_string(), error.into());

        warn!(
            correlation_id = %correlation_id,
            event_count = trace.events.len(),
            "Failed event trace"
        );

        Ok(())
    }

    /// Gets a complete trace
    pub async fn get_trace(&self, correlation_id: &CorrelationId) -> Result<EventTrace> {
        let traces = self.traces.read().await;
        traces
            .get(correlation_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Correlation ID not found: {}", correlation_id))
    }

    /// Gets all active traces
    pub async fn get_active_traces(&self) -> Vec<EventTrace> {
        let traces = self.traces.read().await;
        traces
            .values()
            .filter(|trace| trace.status == TraceStatus::Active)
            .cloned()
            .collect()
    }

    /// Gets traces for a specific component
    pub async fn get_traces_for_component(&self, component_id: &ComponentId) -> Vec<EventTrace> {
        let traces = self.traces.read().await;
        traces
            .values()
            .filter(|trace| trace.components.contains(component_id))
            .cloned()
            .collect()
    }

    /// Analyzes trace performance
    pub async fn analyze_trace_performance(
        &self,
        correlation_id: &CorrelationId,
    ) -> Result<TraceAnalysis> {
        let trace = self.get_trace(correlation_id).await?;

        let mut analysis = TraceAnalysis {
            correlation_id: correlation_id.clone(),
            total_duration: Duration::ZERO,
            component_durations: HashMap::new(),
            hook_point_durations: HashMap::new(),
            event_count: trace.events.len(),
            component_count: trace.components.len(),
            bottlenecks: Vec::new(),
            warnings: Vec::new(),
        };

        if let (Some(_first), Some(last)) = (trace.events.first(), trace.events.last()) {
            analysis.total_duration = last.elapsed;
        }

        // Analyze per-component timing
        for component in &trace.components {
            let component_events: Vec<_> = trace
                .events
                .iter()
                .filter(|e| &e.component_id == component)
                .collect();

            if let (Some(first), Some(last)) = (component_events.first(), component_events.last()) {
                let duration = last.elapsed - first.elapsed;
                analysis
                    .component_durations
                    .insert(component.clone(), duration);

                // Check for bottlenecks (components taking >50% of total time)
                if duration > analysis.total_duration / 2 {
                    analysis.bottlenecks.push(format!(
                        "Component {:?} took {:.2}% of total execution time",
                        component,
                        (duration.as_secs_f64() / analysis.total_duration.as_secs_f64()) * 100.0
                    ));
                }
            }
        }

        // Analyze per-hook-point timing
        let mut hook_point_times: HashMap<HookPoint, Vec<Duration>> = HashMap::new();
        for event in &trace.events {
            hook_point_times
                .entry(event.hook_point.clone())
                .or_default()
                .push(event.elapsed);
        }

        for (hook_point, times) in hook_point_times {
            if let (Some(min_time), Some(max_time)) = (times.iter().min(), times.iter().max()) {
                let duration = *max_time - *min_time;
                analysis.hook_point_durations.insert(hook_point, duration);
            }
        }

        Ok(analysis)
    }

    /// Cleans up expired traces
    async fn cleanup_expired_traces(&self, traces: &mut HashMap<CorrelationId, EventTrace>) {
        let retention_cutoff = SystemTime::now() - self.config.trace_retention;
        let initial_count = traces.len();

        traces.retain(|_, trace| {
            trace.last_activity > retention_cutoff && trace.status == TraceStatus::Active
        });

        let cleaned_count = initial_count - traces.len();
        if cleaned_count > 0 {
            info!(
                cleaned_count = cleaned_count,
                remaining_count = traces.len(),
                "Cleaned up expired traces"
            );
        }
    }

    /// Collects event metrics at the current moment
    async fn collect_event_metrics(&self, _component_id: &ComponentId) -> EventMetrics {
        // This would collect actual system metrics in a real implementation
        EventMetrics {
            memory_usage: None, // Would use a memory profiler
            cpu_usage: None,    // Would use system metrics
            thread_count: None, // Would count active threads
            component_metrics: HashMap::new(),
        }
    }
}

/// Analysis results for a trace
#[derive(Debug, Clone)]
pub struct TraceAnalysis {
    /// The correlation ID analyzed
    pub correlation_id: CorrelationId,
    /// Total execution duration
    pub total_duration: Duration,
    /// Duration per component
    pub component_durations: HashMap<ComponentId, Duration>,
    /// Duration per hook point
    pub hook_point_durations: HashMap<HookPoint, Duration>,
    /// Total number of events
    pub event_count: usize,
    /// Number of components involved
    pub component_count: usize,
    /// Identified bottlenecks
    pub bottlenecks: Vec<String>,
    /// Performance warnings
    pub warnings: Vec<String>,
}

impl Default for EventCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentType;

    #[test]
    fn test_correlation_id_creation() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();

        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_correlation_id_child() {
        let parent = CorrelationId::new();
        let child = parent.create_child();

        assert_ne!(parent, child);
        // Each child should be unique
        let child2 = parent.create_child();
        assert_ne!(child, child2);
    }

    #[tokio::test]
    async fn test_correlator_creation() {
        let correlator = EventCorrelator::new();

        let correlation_id = correlator.create_correlation().await;
        assert!(!correlation_id.to_string().is_empty());

        let traces = correlator.get_active_traces().await;
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_event_recording() {
        let correlator = EventCorrelator::new();
        let correlation_id = correlator.create_correlation().await;

        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());

        let event_id = correlator
            .record_event(
                &correlation_id,
                &component_id,
                HookPoint::BeforeAgentExecution,
                "Test event",
            )
            .await
            .expect("Should record event");

        assert!(!event_id.is_nil());

        let trace = correlator
            .get_trace(&correlation_id)
            .await
            .expect("Should get trace");

        assert_eq!(trace.events.len(), 1);
        assert_eq!(trace.events[0].component_id, component_id);
        assert_eq!(trace.events[0].message, "Test event");
    }

    #[tokio::test]
    async fn test_chain_trace() {
        let correlator = EventCorrelator::new();
        let correlation_id = correlator.create_correlation().await;

        let agent_id = ComponentId::new(ComponentType::Agent, "agent".to_string());
        let tool_id = ComponentId::new(ComponentType::Tool, "tool".to_string());
        let workflow_id = ComponentId::new(ComponentType::Workflow, "workflow".to_string());

        let components = vec![agent_id.clone(), tool_id.clone(), workflow_id.clone()];

        correlator
            .start_chain_trace(correlation_id.clone(), components.clone())
            .await
            .expect("Should start chain trace");

        // Record events for each component
        correlator
            .record_event(
                &correlation_id,
                &agent_id,
                HookPoint::BeforeAgentExecution,
                "Agent start",
            )
            .await
            .expect("Should record agent event");

        correlator
            .record_event(
                &correlation_id,
                &tool_id,
                HookPoint::BeforeToolExecution,
                "Tool start",
            )
            .await
            .expect("Should record tool event");

        correlator
            .record_event(
                &correlation_id,
                &workflow_id,
                HookPoint::BeforeWorkflowStart,
                "Workflow start",
            )
            .await
            .expect("Should record workflow event");

        let trace = correlator
            .get_trace(&correlation_id)
            .await
            .expect("Should get trace");

        assert_eq!(trace.events.len(), 3);
        assert_eq!(trace.components, components);
        assert_eq!(
            trace.metadata.get("chain_type"),
            Some(&"cross_component".to_string())
        );
    }

    #[tokio::test]
    async fn test_trace_completion() {
        let correlator = EventCorrelator::new();
        let correlation_id = correlator.create_correlation().await;

        correlator
            .complete_trace(&correlation_id)
            .await
            .expect("Should complete trace");

        let trace = correlator
            .get_trace(&correlation_id)
            .await
            .expect("Should get trace");

        assert_eq!(trace.status, TraceStatus::Completed);
    }

    #[tokio::test]
    async fn test_trace_failure() {
        let correlator = EventCorrelator::new();
        let correlation_id = correlator.create_correlation().await;

        correlator
            .fail_trace(&correlation_id, "Test failure")
            .await
            .expect("Should fail trace");

        let trace = correlator
            .get_trace(&correlation_id)
            .await
            .expect("Should get trace");

        assert_eq!(trace.status, TraceStatus::Failed);
        assert_eq!(
            trace.metadata.get("failure_reason"),
            Some(&"Test failure".to_string())
        );
    }

    #[tokio::test]
    async fn test_trace_analysis() {
        let correlator = EventCorrelator::new();
        let correlation_id = correlator.create_correlation().await;

        let agent_id = ComponentId::new(ComponentType::Agent, "agent".to_string());
        let tool_id = ComponentId::new(ComponentType::Tool, "tool".to_string());

        // Record some events with delays to create measurable durations
        correlator
            .record_event(
                &correlation_id,
                &agent_id,
                HookPoint::BeforeAgentExecution,
                "Agent start",
            )
            .await
            .expect("Should record agent event");

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        correlator
            .record_event(
                &correlation_id,
                &tool_id,
                HookPoint::BeforeToolExecution,
                "Tool start",
            )
            .await
            .expect("Should record tool event");

        let analysis = correlator
            .analyze_trace_performance(&correlation_id)
            .await
            .expect("Should analyze trace");

        assert_eq!(analysis.correlation_id, correlation_id);
        assert_eq!(analysis.event_count, 2);
        assert!(analysis.total_duration > Duration::ZERO);
    }
}
