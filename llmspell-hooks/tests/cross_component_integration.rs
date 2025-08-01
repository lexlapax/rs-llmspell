//! ABOUTME: Integration tests for cross-component hook coordination
//! ABOUTME: Tests Agent → Tool → Workflow execution chains with hook coordination

use llmspell_hooks::coordination::{CrossComponentCoordinator, ExecutionChain};
use llmspell_hooks::{ComponentId, ComponentType, HookContext, HookPoint};
use std::time::Duration;

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[tokio::test]
async fn test_agent_tool_workflow_chain() {
    let coordinator = CrossComponentCoordinator::new();

    // Create components
    let agent_id = ComponentId::new(ComponentType::Agent, "gpt-4".to_string());
    let tool_id = ComponentId::new(ComponentType::Tool, "calculator".to_string());
    let workflow_id = ComponentId::new(ComponentType::Workflow, "analysis".to_string());

    // Create execution chain
    let chain = ExecutionChain::new()
        .with_name("agent-tool-workflow")
        .add_component(agent_id.clone())
        .add_component(tool_id.clone())
        .add_component(workflow_id.clone())
        .with_metadata("purpose", "integration-test");

    // Register the chain
    let chain_id = coordinator
        .register_chain("test-chain", chain)
        .await
        .expect("Should register chain successfully");

    assert!(!chain_id.is_nil());

    // Start chain execution
    let initial_context = HookContext::new(HookPoint::BeforeAgentInit, agent_id.clone());
    let correlation_id = coordinator
        .start_chain_execution("test-chain", initial_context)
        .await
        .expect("Should start chain execution");

    assert!(!correlation_id.to_string().is_empty());

    // Verify chain is in executing state
    let state = coordinator
        .get_chain_state("test-chain")
        .await
        .expect("Should get chain state");

    // The chain should be in executing state
    use llmspell_hooks::coordination::ChainState;
    match state {
        ChainState::Executing {
            current_component, ..
        } => {
            assert_eq!(current_component, 0); // Should be at first component
        }
        other => panic!("Expected Executing state, got {:?}", other),
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[tokio::test]
async fn test_chain_cleanup() {
    let coordinator = CrossComponentCoordinator::new();

    let agent_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
    let chain = ExecutionChain::new()
        .with_name("cleanup-test")
        .add_component(agent_id);

    coordinator
        .register_chain("cleanup-test", chain)
        .await
        .expect("Should register chain");

    // Verify chain exists
    assert!(coordinator.get_chain_state("cleanup-test").await.is_some());

    // Clean up the chain
    coordinator
        .cleanup_chain("cleanup-test")
        .await
        .expect("Should cleanup chain");

    // Chain should no longer exist
    assert!(coordinator.get_chain_state("cleanup-test").await.is_none());
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[tokio::test]
async fn test_coordinator_capacity_limits() {
    use llmspell_hooks::coordination::CoordinatorConfig;

    // Create a coordinator with very limited capacity
    let config = CoordinatorConfig {
        max_active_chains: 2,
        max_chain_execution_time: Duration::from_secs(1),
        ..Default::default()
    };

    let coordinator = CrossComponentCoordinator::with_config(config);

    let agent_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());

    // Register two chains (should succeed)
    for i in 0..2 {
        let chain = ExecutionChain::new()
            .with_name(format!("chain-{}", i))
            .add_component(agent_id.clone());

        coordinator
            .register_chain(&format!("chain-{}", i), chain)
            .await
            .expect("Should register chain within capacity");
    }

    // Try to register a third chain (should fail due to capacity)
    let chain = ExecutionChain::new()
        .with_name("chain-overflow")
        .add_component(agent_id);

    let result = coordinator.register_chain("chain-overflow", chain).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Maximum number of active chains"));
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[tokio::test]
async fn test_cross_component_context() {
    use llmspell_hooks::coordination::CrossComponentContext;
    use std::collections::HashMap;

    let coordinator = CrossComponentCoordinator::new();

    let agent_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
    let tool_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());

    let chain = ExecutionChain::new()
        .with_name("context-test")
        .add_component(agent_id.clone())
        .add_component(tool_id.clone());

    coordinator
        .register_chain("context-test", chain)
        .await
        .expect("Should register chain");

    // Create a cross-component context
    let base_context = HookContext::new(HookPoint::BeforeAgentInit, agent_id.clone());
    let correlation_id = coordinator
        .start_chain_execution("context-test", base_context.clone())
        .await
        .expect("Should start execution");

    let cross_context = CrossComponentContext {
        hook_context: base_context,
        chain_id: uuid::Uuid::new_v4(),
        current_component: agent_id,
        chain_position: 0,
        chain_length: 2,
        correlation_id,
        propagated_data: HashMap::new(),
        previous_metrics: Vec::new(),
    };

    // Verify context structure
    assert_eq!(cross_context.chain_position, 0);
    assert_eq!(cross_context.chain_length, 2);
    assert!(cross_context.propagated_data.is_empty());
    assert!(cross_context.previous_metrics.is_empty());
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "hook")]
#[test]
fn test_execution_chain_builder() {
    let agent_id = ComponentId::new(ComponentType::Agent, "agent".to_string());
    let tool_id = ComponentId::new(ComponentType::Tool, "tool".to_string());
    let workflow_id = ComponentId::new(ComponentType::Workflow, "workflow".to_string());

    let chain = ExecutionChain::new()
        .with_name("builder-test")
        .add_component(agent_id.clone())
        .add_component(tool_id.clone())
        .add_component(workflow_id.clone())
        .with_metadata("test", "value")
        .with_metadata("version", "1.0");

    assert_eq!(chain.name, "builder-test");
    assert_eq!(chain.components.len(), 3);
    assert_eq!(chain.components[0], agent_id);
    assert_eq!(chain.components[1], tool_id);
    assert_eq!(chain.components[2], workflow_id);
    assert_eq!(chain.metadata.get("test"), Some(&"value".to_string()));
    assert_eq!(chain.metadata.get("version"), Some(&"1.0".to_string()));

    // Test navigation methods
    assert_eq!(chain.get_next_component(0), Some(&tool_id));
    assert_eq!(chain.get_next_component(1), Some(&workflow_id));
    assert_eq!(chain.get_next_component(2), None);

    assert!(!chain.is_last_component(0));
    assert!(!chain.is_last_component(1));
    assert!(chain.is_last_component(2));
}
