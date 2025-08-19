//! Integration test to verify event bus wiring through `ComponentRegistry`

use llmspell_bridge::ComponentRegistry;
use llmspell_core::traits::event::EventConfig;
use llmspell_core::types::{AgentInput, AgentOutput};
use llmspell_core::{BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Result};
use llmspell_events::{EventBus, Language};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Mock component for testing
struct TestComponent {
    metadata: ComponentMetadata,
}

impl TestComponent {
    fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "test-agent".to_string(),
                "Test agent for event integration".to_string(),
            ),
        }
    }
}

#[async_trait::async_trait]
impl BaseAgent for TestComponent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, _input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        Ok(AgentOutput::text("Test output"))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {error}")))
    }
}

#[tokio::test]
async fn test_event_bus_wiring_through_registry() {
    // Create EventBus and subscribe to events
    let event_bus = Arc::new(EventBus::new());
    let mut receiver = event_bus.subscribe("agent.*").await.unwrap();

    // Create ComponentRegistry with EventBus
    let event_config = EventConfig::default();
    let registry = ComponentRegistry::with_event_bus(event_bus.clone(), event_config);

    // Create a base execution context
    let base_context = ExecutionContext::new();

    // Create context with event emitter through registry
    let context = registry.create_execution_context(base_context);

    // Verify events field is populated
    assert!(
        context.events.is_some(),
        "Events should be injected by registry"
    );

    // Create test component and execute with events
    let component = TestComponent::new();
    let input = AgentInput::text("test input");

    // Execute with events (this should emit lifecycle events)
    let _output = component.execute_with_events(input, context).await.unwrap();

    // Check if we received the started event
    let started_event = timeout(Duration::from_millis(100), receiver.recv())
        .await
        .expect("Timeout waiting for started event")
        .expect("Should receive started event");

    assert_eq!(started_event.event_type, "agent.started");
    assert_eq!(started_event.language, Language::Rust);

    // Check if we received the completed event
    let completed_event = timeout(Duration::from_millis(100), receiver.recv())
        .await
        .expect("Timeout waiting for completed event")
        .expect("Should receive completed event");

    assert_eq!(completed_event.event_type, "agent.completed");
}

#[tokio::test]
async fn test_event_emission_can_be_disabled() {
    // Create EventBus
    let event_bus = Arc::new(EventBus::new());
    let mut receiver = event_bus.subscribe("*").await.unwrap();

    // Create ComponentRegistry with events DISABLED
    let event_config = EventConfig {
        enabled: false,
        ..EventConfig::default()
    };
    let registry = ComponentRegistry::with_event_bus(event_bus.clone(), event_config);

    // Create context through registry
    let context = registry.create_execution_context(ExecutionContext::new());

    // Events should NOT be injected when disabled
    assert!(
        context.events.is_none(),
        "Events should not be injected when disabled"
    );

    // Even if we manually add events and try to emit, nothing should happen
    // (This tests a different path, but good to verify)

    // Try to receive an event (should timeout)
    let result = timeout(Duration::from_millis(50), receiver.recv()).await;
    assert!(
        result.is_err(),
        "Should not receive any events when disabled"
    );
}

#[tokio::test]
async fn test_event_filtering_through_config() {
    // Create EventBus
    let event_bus = Arc::new(EventBus::new());
    let mut receiver = event_bus.subscribe("*").await.unwrap();

    // Create ComponentRegistry with event filtering
    let event_config = EventConfig {
        enabled: true,
        exclude_types: vec!["*.started".to_string()], // Exclude started events
        ..EventConfig::default()
    };

    let registry = ComponentRegistry::with_event_bus(event_bus.clone(), event_config);

    // Create context and component
    let context = registry.create_execution_context(ExecutionContext::new());
    let component = TestComponent::new();
    let input = AgentInput::text("test input");

    // Execute with events
    let _output = component.execute_with_events(input, context).await.unwrap();

    // Should only receive completed event (started is filtered out)
    let event = timeout(Duration::from_millis(100), receiver.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Should receive event");

    assert_eq!(event.event_type, "agent.completed");

    // Should not receive another event
    let result = timeout(Duration::from_millis(50), receiver.recv()).await;
    assert!(
        result.is_err(),
        "Should not receive started event (filtered)"
    );
}

#[tokio::test]
async fn test_registry_without_event_bus() {
    // Create ComponentRegistry without EventBus (using new())
    let registry = ComponentRegistry::new();

    // Create context through registry
    let context = registry.create_execution_context(ExecutionContext::new());

    // Events should be None when registry has no EventBus
    assert!(
        context.events.is_none(),
        "Events should be None when registry has no EventBus"
    );

    // Component execution should still work (just without events)
    let component = TestComponent::new();
    let input = AgentInput::text("test input");

    // This should work fine, just no events emitted
    let output = component.execute_with_events(input, context).await.unwrap();
    assert_eq!(output.text, "Test output");
}
