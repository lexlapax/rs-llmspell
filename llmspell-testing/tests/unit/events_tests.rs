// ABOUTME: Integration tests for event system using TestEventCollector
// ABOUTME: Tests event emission across components, workflows, and tools

use llmspell_core::traits::event::{EventConfig, EventData, EventEmitter};
use llmspell_core::{ComponentId, Result};
use llmspell_testing::event_helpers::{
    assert_correlated_events, assert_event_count, assert_event_data_contains, assert_event_emitted,
    assert_event_sequence, create_correlated_event_data, create_test_event_data, event_data,
    TestEventCollector,
};
use serde_json::json;
use std::sync::Arc;

/// Mock component that emits events during execution
struct MockEventEmittingComponent {
    component_id: ComponentId,
    events: Arc<TestEventCollector>,
}

impl MockEventEmittingComponent {
    fn new(events: Arc<TestEventCollector>) -> Self {
        Self {
            component_id: ComponentId::new(),
            events,
        }
    }

    /// Simulate agent execution with lifecycle events
    async fn execute_agent(&self, input: &str) -> Result<String> {
        // Emit start event
        self.events
            .emit(
                "agent.started",
                event_data::agent_execution_data(&self.component_id.to_string(), input),
            )
            .await?;

        // Simulate work
        let result = format!("Processed: {}", input);

        // Emit completion event
        self.events
            .emit(
                "agent.completed",
                json!({
                    "agent_id": self.component_id.to_string(),
                    "input": input,
                    "output": result,
                    "duration_ms": 42
                }),
            )
            .await?;

        Ok(result)
    }

    /// Simulate tool execution with events
    async fn execute_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Emit tool start event
        self.events
            .emit(
                "tool.started",
                event_data::tool_execution_data(tool_name, params.clone()),
            )
            .await?;

        // Simulate tool work
        let result = json!({
            "tool": tool_name,
            "input": params,
            "result": "success"
        });

        // Emit tool completion event
        self.events
            .emit(
                "tool.completed",
                json!({
                    "tool": tool_name,
                    "parameters": params,
                    "result": result,
                    "duration_ms": 15
                }),
            )
            .await?;

        Ok(result)
    }

    /// Simulate workflow execution with step events
    async fn execute_workflow(&self, workflow_id: &str, steps: Vec<&str>) -> Result<()> {
        let correlation_id = format!("workflow-{}", workflow_id);

        // Emit workflow start
        let start_event = EventData::new("workflow.started")
            .with_data(event_data::workflow_step_data(workflow_id, "start"))
            .with_correlation(&correlation_id);
        self.events.emit_structured(start_event).await?;

        // Emit step events
        for (i, step) in steps.iter().enumerate() {
            let step_event = create_correlated_event_data(
                "workflow.step.started",
                event_data::workflow_step_data(workflow_id, step),
                &correlation_id,
            );
            self.events.emit_structured(step_event).await?;

            // Simulate step completion
            let complete_event = create_correlated_event_data(
                "workflow.step.completed",
                json!({
                    "workflow_id": workflow_id,
                    "step": step,
                    "step_index": i,
                    "duration_ms": 100 + i * 10
                }),
                &correlation_id,
            );
            self.events.emit_structured(complete_event).await?;
        }

        // Emit workflow completion
        let end_event = EventData::new("workflow.completed")
            .with_data(json!({
                "workflow_id": workflow_id,
                "total_steps": steps.len(),
                "result": "success"
            }))
            .with_correlation(&correlation_id);
        self.events.emit_structured(end_event).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_lifecycle_events() {
        let collector = Arc::new(TestEventCollector::new());
        let component = MockEventEmittingComponent::new(collector.clone());

        // Execute agent
        let result = component
            .execute_agent("test input")
            .await
            .expect("Agent execution should succeed");

        assert_eq!(result, "Processed: test input");

        // Verify events were emitted
        assert_event_count(&collector, 2);
        assert_event_emitted(&collector, "agent.started");
        assert_event_emitted(&collector, "agent.completed");

        // Verify event sequence
        assert_event_sequence(&collector, &["agent.started", "agent.completed"]);

        // Verify event data
        assert_event_data_contains(&collector, "agent.started", "input", &json!("test input"));
        assert_event_data_contains(
            &collector,
            "agent.completed",
            "output",
            &json!("Processed: test input"),
        );
        assert_event_data_contains(&collector, "agent.completed", "duration_ms", &json!(42));
    }

    #[tokio::test]
    async fn test_tool_execution_events() {
        let collector = Arc::new(TestEventCollector::new());
        let component = MockEventEmittingComponent::new(collector.clone());

        let params = json!({"operation": "add", "a": 5, "b": 3});

        // Execute tool
        let result = component
            .execute_tool("calculator", params.clone())
            .await
            .expect("Tool execution should succeed");

        // Verify events
        assert_event_count(&collector, 2);
        assert_event_emitted(&collector, "tool.started");
        assert_event_emitted(&collector, "tool.completed");

        // Verify tool-specific data
        assert_event_data_contains(&collector, "tool.started", "tool", &json!("calculator"));
        assert_event_data_contains(&collector, "tool.started", "parameters", &params);
        assert_event_data_contains(&collector, "tool.completed", "duration_ms", &json!(15));

        // Verify result structure
        assert!(result["tool"] == "calculator");
        assert!(result["result"] == "success");
    }

    #[tokio::test]
    async fn test_workflow_execution_events() {
        let collector = Arc::new(TestEventCollector::new());
        let component = MockEventEmittingComponent::new(collector.clone());

        let workflow_id = "test-workflow-123";
        let steps = vec!["validation", "processing", "finalization"];

        // Execute workflow
        component
            .execute_workflow(workflow_id, steps.clone())
            .await
            .expect("Workflow execution should succeed");

        // Should have start + (2 events per step * 3 steps) + end = 8 events
        assert_event_count(&collector, 8);

        // Verify lifecycle events
        assert_event_emitted(&collector, "workflow.started");
        assert_event_emitted(&collector, "workflow.completed");
        assert_event_emitted(&collector, "workflow.step.started");
        assert_event_emitted(&collector, "workflow.step.completed");

        // Verify correlated events
        let correlation_id = format!("workflow-{}", workflow_id);
        assert_correlated_events(&collector, &correlation_id, 8);

        // Verify step events for each step
        for step in &steps {
            let step_events = collector.get_events_of_type("workflow.step.started");
            let step_found = step_events
                .iter()
                .any(|e| e.data.get("step") == Some(&json!(step)));
            assert!(step_found, "Should find step event for {}", step);
        }
    }

    #[tokio::test]
    async fn test_event_collector_disabled_behavior() {
        let collector = Arc::new(TestEventCollector::disabled());
        let component = MockEventEmittingComponent::new(collector.clone());

        // Execute with disabled collector
        component
            .execute_agent("test input")
            .await
            .expect("Agent execution should succeed even with disabled events");

        // No events should be captured
        assert_event_count(&collector, 0);
        assert!(!collector.has_event_type("agent.started"));
        assert!(!collector.has_event_type("agent.completed"));
    }

    #[tokio::test]
    async fn test_event_collector_filtering() {
        let config = EventConfig {
            enabled: true,
            include_types: vec!["agent.*".to_string()],
            exclude_types: vec!["*.started".to_string()],
            ..EventConfig::default()
        };

        let collector = Arc::new(TestEventCollector::with_config(config));
        let component = MockEventEmittingComponent::new(collector.clone());

        // Execute agent
        component
            .execute_agent("test input")
            .await
            .expect("Agent execution should succeed");

        // Should capture completed but not started (due to filtering)
        // Note: TestEventCollector doesn't implement actual filtering logic,
        // this test is more about the configuration interface
        assert_event_count(&collector, 2); // Both events still captured in TestEventCollector

        // But we can verify the config is set correctly
        let event_config = collector.config();
        assert!(event_config.enabled);
        assert_eq!(event_config.include_types, vec!["agent.*"]);
        assert_eq!(event_config.exclude_types, vec!["*.started"]);
    }

    #[tokio::test]
    async fn test_complex_multi_component_workflow() {
        let collector = Arc::new(TestEventCollector::new());
        let component = MockEventEmittingComponent::new(collector.clone());

        // Simulate complex workflow with multiple components
        let workflow_correlation = "complex-workflow-456";

        // Start workflow
        let workflow_start = EventData::new("complex.workflow.started")
            .with_data(json!({"workflow_type": "multi_component"}))
            .with_correlation(workflow_correlation);
        collector
            .emit_structured(workflow_start)
            .await
            .expect("Failed to emit workflow start");

        // Execute agent
        component
            .execute_agent("process data")
            .await
            .expect("Agent execution should succeed");

        // Execute tool
        component
            .execute_tool("data_processor", json!({"data": "sample"}))
            .await
            .expect("Tool execution should succeed");

        // Execute sub-workflow
        component
            .execute_workflow("sub-workflow", vec!["analyze", "transform"])
            .await
            .expect("Sub-workflow execution should succeed");

        // End workflow
        let workflow_end = EventData::new("complex.workflow.completed")
            .with_data(json!({"workflow_type": "multi_component", "result": "success"}))
            .with_correlation(workflow_correlation);
        collector
            .emit_structured(workflow_end)
            .await
            .expect("Failed to emit workflow end");

        // Verify we have all expected events
        // 1 start + 2 agent + 2 tool + 6 sub-workflow + 1 end = 12 events
        assert_event_count(&collector, 12);

        // Verify each component type emitted events
        assert_event_emitted(&collector, "complex.workflow.started");
        assert_event_emitted(&collector, "complex.workflow.completed");
        assert_event_emitted(&collector, "agent.started");
        assert_event_emitted(&collector, "agent.completed");
        assert_event_emitted(&collector, "tool.started");
        assert_event_emitted(&collector, "tool.completed");
        assert_event_emitted(&collector, "workflow.started");
        assert_event_emitted(&collector, "workflow.step.started");

        // Verify correlation for main workflow events
        assert_correlated_events(&collector, workflow_correlation, 2);

        // Verify sub-workflow correlation (separate from main workflow)
        let sub_workflow_correlation = "workflow-sub-workflow";
        assert_correlated_events(&collector, sub_workflow_correlation, 6);
    }

    #[tokio::test]
    async fn test_event_data_helpers() {
        let _collector = TestEventCollector::new();

        // Test event data creation helpers
        let agent_data = event_data::agent_execution_data("test-agent", "hello world");
        assert_eq!(agent_data["agent_id"], "test-agent");
        assert_eq!(agent_data["input"], "hello world");

        let tool_data = event_data::tool_execution_data("calculator", json!({"op": "add"}));
        assert_eq!(tool_data["tool"], "calculator");
        assert_eq!(tool_data["parameters"]["op"], "add");

        let workflow_data = event_data::workflow_step_data("wf-123", "validation");
        assert_eq!(workflow_data["workflow_id"], "wf-123");
        assert_eq!(workflow_data["step"], "validation");

        let error_data = event_data::error_data("ValidationError", "Invalid input format");
        assert_eq!(error_data["error_type"], "ValidationError");
        assert_eq!(error_data["message"], "Invalid input format");

        // Test structured event creation
        let basic_event = create_test_event_data("test.basic", json!({"key": "value"}));
        assert_eq!(basic_event.event_type, "test.basic");
        assert_eq!(basic_event.data["key"], "value");

        let correlated_event = create_correlated_event_data(
            "test.correlated",
            json!({"key": "value"}),
            "correlation-123",
        );
        assert_eq!(correlated_event.event_type, "test.correlated");
        assert_eq!(
            correlated_event.correlation_id,
            Some("correlation-123".to_string())
        );
    }

    #[test]
    fn test_event_collector_utility_methods() {
        let collector = TestEventCollector::new();

        // Test initial state
        assert_eq!(collector.event_count(), 0);
        assert!(!collector.has_event_type("any.event"));
        assert!(collector.latest_event().is_none());

        // Manually add some events for testing utilities
        let events = vec![
            create_test_event_data("first.event", json!({"order": 1})),
            create_test_event_data("second.event", json!({"order": 2})),
            create_correlated_event_data("third.event", json!({"order": 3}), "corr-123"),
            create_correlated_event_data("fourth.event", json!({"order": 4}), "corr-123"),
        ];

        for event in events {
            collector.add_event(event);
        }

        // Test utility methods
        assert_eq!(collector.event_count(), 4);
        assert!(collector.has_event_type("first.event"));
        assert!(collector.has_event_type("second.event"));
        assert!(!collector.has_event_type("nonexistent.event"));

        // Test latest event
        let latest = collector.latest_event().unwrap();
        assert_eq!(latest.event_type, "fourth.event");
        assert_eq!(latest.data["order"], 4);

        // Test type filtering
        let first_events = collector.get_events_of_type("first.event");
        assert_eq!(first_events.len(), 1);
        assert_eq!(first_events[0].data["order"], 1);

        // Test correlation filtering
        let correlated = collector.get_correlated_events("corr-123");
        assert_eq!(correlated.len(), 2);
        assert_eq!(correlated[0].event_type, "third.event");
        assert_eq!(correlated[1].event_type, "fourth.event");

        // Test clear
        collector.clear();
        assert_eq!(collector.event_count(), 0);
        assert!(!collector.has_event_type("first.event"));
    }
}
