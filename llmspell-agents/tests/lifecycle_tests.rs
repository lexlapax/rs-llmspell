//! ABOUTME: Comprehensive integration tests for agent lifecycle management components
//! ABOUTME: Tests state machine, events, resources, shutdown, health monitoring, and middleware integration

use anyhow::Result;
use async_trait::async_trait;
use llmspell_agents::{
    health::{
        AgentHealthMonitor, HealthMonitorConfig, HealthStatus, ResourceHealthCheck,
        ResponsivenessHealthCheck, StateMachineHealthCheck,
    },
    lifecycle::{
        events::{
            EventSubscription, EventSystemConfig, LifecycleEventSystem, LoggingEventListener,
            MetricsEventListener,
        },
        middleware::{
            LifecycleMiddleware, LifecycleMiddlewareChain, LifecyclePhase, LoggingMiddleware,
            MetricsMiddleware, MiddlewareConfig, MiddlewareContext,
        },
        resources::{
            LoggingResourceHook, ResourceLimits, ResourceManager, ResourceRequest, ResourceType,
        },
        shutdown::{
            LoggingShutdownHook, ResourceCleanupHook, ShutdownConfig, ShutdownCoordinator,
            ShutdownPriority, ShutdownRequest,
        },
        state_machine::{AgentState, AgentStateMachine, StateMachineConfig},
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Integration test for complete lifecycle workflow
#[tokio::test]
async fn test_complete_lifecycle_workflow() {
    // Setup event system
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Setup resource manager
    let resource_manager = Arc::new(ResourceManager::new(
        ResourceLimits::default(),
        event_system.clone(),
    ));

    // Setup state machine
    let state_machine = Arc::new(AgentStateMachine::new(
        "test-agent".to_string(),
        StateMachineConfig::default(),
    ));

    // Setup health monitor
    let health_monitor = AgentHealthMonitor::new(
        "test-agent".to_string(),
        state_machine.clone(),
        resource_manager.clone(),
        event_system.clone(),
        HealthMonitorConfig::default(),
    );

    // Add health checks
    health_monitor
        .add_health_check(Arc::new(StateMachineHealthCheck::new(
            state_machine.clone(),
        )))
        .await;
    health_monitor
        .add_health_check(Arc::new(ResourceHealthCheck::new(resource_manager.clone())))
        .await;
    health_monitor
        .add_health_check(Arc::new(ResponsivenessHealthCheck))
        .await;

    // Setup shutdown coordinator
    let shutdown_coordinator = ShutdownCoordinator::new(
        event_system.clone(),
        resource_manager.clone(),
        ShutdownConfig::default(),
    );

    shutdown_coordinator
        .add_hook(Arc::new(LoggingShutdownHook))
        .await;
    shutdown_coordinator
        .add_hook(Arc::new(ResourceCleanupHook::new(resource_manager.clone())))
        .await;

    // Test complete workflow

    // 1. Initialize agent
    assert_eq!(
        state_machine.current_state().await,
        AgentState::Uninitialized
    );
    state_machine.initialize().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Ready);

    // 2. Allocate some resources
    let memory_request = ResourceRequest::new(
        "test-agent".to_string(),
        ResourceType::Memory,
        1024 * 1024, // 1MB
    );
    let _memory_allocation = resource_manager.allocate(memory_request).await.unwrap();

    let cpu_request = ResourceRequest::new(
        "test-agent".to_string(),
        ResourceType::Cpu,
        25, // 25%
    );
    let _cpu_allocation = resource_manager.allocate(cpu_request).await.unwrap();

    // 3. Start agent execution
    state_machine.start().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Running);

    // 4. Perform health check
    let health_result = health_monitor.check_health().await.unwrap();
    assert_eq!(health_result.status, HealthStatus::Healthy);
    assert!(health_monitor.is_healthy().await);

    // 5. Pause and resume
    state_machine.pause().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Paused);

    state_machine.resume().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Running);

    // 6. Test error and recovery
    state_machine.error("Test error".to_string()).await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Error);

    let health_result = health_monitor.check_health().await.unwrap();
    assert_eq!(health_result.status, HealthStatus::Critical);

    state_machine.recover().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Ready);

    // 7. Graceful shutdown
    let shutdown_request = ShutdownRequest::new("test-agent".to_string())
        .with_priority(ShutdownPriority::Normal)
        .with_reason("Test completed".to_string());

    let shutdown_result = shutdown_coordinator
        .shutdown_agent(shutdown_request, state_machine.clone())
        .await
        .unwrap();

    assert!(shutdown_result.success);
    assert_eq!(state_machine.current_state().await, AgentState::Terminated);

    // 8. Verify resources were cleaned up
    let allocations = resource_manager.get_agent_allocations("test-agent").await;
    assert_eq!(allocations.len(), 0);

    // 9. Final health check should show unhealthy (terminated)
    let health_result = health_monitor.check_health().await.unwrap();
    assert_eq!(health_result.status, HealthStatus::Unhealthy);
}
#[tokio::test]
async fn test_event_system_integration() {
    use llmspell_agents::lifecycle::events::{
        LifecycleEvent, LifecycleEventData, LifecycleEventType,
    };
    use llmspell_agents::lifecycle::state_machine::AgentState;

    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Setup event listeners
    let logging_listener = Arc::new(LoggingEventListener::new());
    let metrics_listener = Arc::new(MetricsEventListener::new());

    let logging_subscription = EventSubscription::new(logging_listener.clone());
    let metrics_subscription = EventSubscription::new(metrics_listener.clone());

    event_system.subscribe(logging_subscription).await;
    event_system.subscribe(metrics_subscription).await;

    // Setup state machine with event integration
    let state_machine = Arc::new(AgentStateMachine::default("event-test-agent".to_string()));

    // Perform state transitions
    state_machine.initialize().await.unwrap();
    state_machine.start().await.unwrap();
    state_machine.pause().await.unwrap();
    state_machine.error("Test error".to_string()).await.unwrap();
    state_machine.recover().await.unwrap();
    state_machine.terminate().await.unwrap();

    // Manually emit events to test the event system
    // (State machine doesn't currently integrate with event system)

    // Emit state change event
    let event = LifecycleEvent::new(
        LifecycleEventType::StateChanged,
        "event-test-agent".to_string(),
        LifecycleEventData::StateTransition {
            from: AgentState::Ready,
            to: AgentState::Running,
            duration: Some(Duration::from_millis(10)),
            reason: Some("Test transition".to_string()),
        },
        "test".to_string(),
    );
    event_system.emit(event).await.unwrap();

    // Emit error event
    let error_event = LifecycleEvent::new(
        LifecycleEventType::ErrorOccurred,
        "event-test-agent".to_string(),
        LifecycleEventData::Error {
            message: "Test error".to_string(),
            error_type: "test_error".to_string(),
            recovery_possible: true,
        },
        "test".to_string(),
    );
    event_system.emit(error_event).await.unwrap();

    // Give events time to process
    sleep(Duration::from_millis(100)).await;

    // Check that events were recorded
    let event_history = event_system.get_event_history().await;
    assert!(!event_history.is_empty());

    let agent_events = event_system.get_agent_events("event-test-agent").await;
    assert!(!agent_events.is_empty());

    // Check metrics were collected
    let metrics = metrics_listener.get_metrics().await;
    assert!(!metrics.is_empty());

    // Should have metrics for various event types
    assert!(metrics.keys().any(|k| k.contains("StateChanged")));
}
#[tokio::test]
async fn test_resource_management_integration() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let mut resource_manager =
        ResourceManager::new(ResourceLimits::default(), event_system.clone());

    resource_manager.add_hook(Arc::new(LoggingResourceHook));

    // Test resource allocation and limits
    let agent_id = "resource-test-agent";

    // Allocate various types of resources
    let resources = vec![
        (ResourceType::Memory, 512 * 1024 * 1024), // 512MB
        (ResourceType::Cpu, 30),                   // 30%
        (ResourceType::ToolAccess, 10),            // 10 tools
        (ResourceType::FileHandles, 100),          // 100 handles
    ];

    let mut allocations = Vec::new();
    for (resource_type, amount) in resources {
        let request = ResourceRequest::new(agent_id.to_string(), resource_type, amount);
        let allocation = resource_manager.allocate(request).await.unwrap();
        allocations.push(allocation);
    }

    // Check allocations
    let agent_allocations = resource_manager.get_agent_allocations(agent_id).await;
    assert_eq!(agent_allocations.len(), 4);

    // Test resource limits - should fail with excessive allocation
    let excessive_request = ResourceRequest::new(
        agent_id.to_string(),
        ResourceType::Memory,
        2 * 1024 * 1024 * 1024, // 2GB (exceeds default limit)
    );
    let result = resource_manager.allocate(excessive_request).await;
    assert!(result.is_err());

    // Test resource deallocation
    for allocation in &allocations {
        resource_manager.deallocate(&allocation.id).await.unwrap();
    }

    let agent_allocations = resource_manager.get_agent_allocations(agent_id).await;
    assert_eq!(agent_allocations.len(), 0);

    // Test bulk deallocation
    let request = ResourceRequest::new(agent_id.to_string(), ResourceType::Memory, 1024 * 1024);
    resource_manager.allocate(request).await.unwrap();

    resource_manager.deallocate_all(agent_id).await.unwrap();
    let agent_allocations = resource_manager.get_agent_allocations(agent_id).await;
    assert_eq!(agent_allocations.len(), 0);
}
#[tokio::test]
async fn test_middleware_integration() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let middleware_chain = LifecycleMiddlewareChain::new(event_system, MiddlewareConfig::default());

    // Add built-in middleware
    middleware_chain
        .add_middleware(Arc::new(LoggingMiddleware::new()))
        .await;
    middleware_chain
        .add_middleware(Arc::new(MetricsMiddleware::new()))
        .await;

    // Test middleware execution for various phases
    let phases = vec![
        LifecyclePhase::Initialization,
        LifecyclePhase::StateTransition,
        LifecyclePhase::TaskExecution,
        LifecyclePhase::ResourceAllocation,
        LifecyclePhase::HealthCheck,
        LifecyclePhase::Shutdown,
    ];

    for phase in phases {
        let context = MiddlewareContext::new("middleware-test-agent".to_string(), phase.clone());
        let result = middleware_chain.execute(context).await.unwrap();
        assert_eq!(result.agent_id, "middleware-test-agent");
        assert_eq!(result.phase, phase);
    }

    // Check that middleware was executed
    assert_eq!(middleware_chain.get_middleware_count().await, 2);

    let execution_history = middleware_chain.get_execution_history().await;
    assert!(!execution_history.is_empty());
}
#[tokio::test]
async fn test_shutdown_coordinator_integration() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let resource_manager = Arc::new(ResourceManager::new(
        ResourceLimits::default(),
        event_system.clone(),
    ));

    let shutdown_coordinator = ShutdownCoordinator::new(
        event_system.clone(),
        resource_manager.clone(),
        ShutdownConfig::default(),
    );

    shutdown_coordinator
        .add_hook(Arc::new(LoggingShutdownHook))
        .await;
    shutdown_coordinator
        .add_hook(Arc::new(ResourceCleanupHook::new(resource_manager.clone())))
        .await;

    // Create multiple agents for shutdown testing
    let mut agents = HashMap::new();
    let priorities = vec![
        ("critical-agent", ShutdownPriority::Critical),
        ("normal-agent", ShutdownPriority::Normal),
        ("background-agent", ShutdownPriority::Background),
    ];

    for (agent_id, _priority) in &priorities {
        let state_machine = Arc::new(AgentStateMachine::default((*agent_id).to_string()));
        state_machine.initialize().await.unwrap();
        state_machine.start().await.unwrap();
        agents.insert((*agent_id).to_string(), state_machine);
    }

    // Create shutdown requests
    let mut requests = Vec::new();
    for (agent_id, priority) in priorities {
        let request = ShutdownRequest::new(agent_id.to_string())
            .with_priority(priority)
            .with_reason("Integration test shutdown".to_string());
        requests.push(request);
    }

    // Test priority-based shutdown
    let results = shutdown_coordinator
        .shutdown_agents_by_priority(requests, agents.clone())
        .await
        .unwrap();

    assert_eq!(results.len(), 3);

    // Check that all shutdowns were successful
    for result in &results {
        assert!(result.success);
    }

    // Critical agent should be shut down first
    assert_eq!(results[0].agent_id, "critical-agent");

    // Background agent should be shut down last
    assert_eq!(results[2].agent_id, "background-agent");

    // Verify all agents are terminated
    for (_agent_id, state_machine) in agents {
        assert_eq!(state_machine.current_state().await, AgentState::Terminated);
    }
}
#[tokio::test]
async fn test_health_monitoring_integration() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let resource_manager = Arc::new(ResourceManager::new(
        ResourceLimits::default(),
        event_system.clone(),
    ));
    let state_machine = Arc::new(AgentStateMachine::default("health-test-agent".to_string()));

    state_machine.initialize().await.unwrap();

    let health_monitor = AgentHealthMonitor::new(
        "health-test-agent".to_string(),
        state_machine.clone(),
        resource_manager.clone(),
        event_system.clone(),
        HealthMonitorConfig::default(),
    );

    // Add all built-in health checks
    health_monitor
        .add_health_check(Arc::new(StateMachineHealthCheck::new(
            state_machine.clone(),
        )))
        .await;
    health_monitor
        .add_health_check(Arc::new(ResourceHealthCheck::new(resource_manager.clone())))
        .await;
    health_monitor
        .add_health_check(Arc::new(ResponsivenessHealthCheck))
        .await;

    // Test healthy state
    let result = health_monitor.check_health().await.unwrap();
    assert_eq!(result.status, HealthStatus::Healthy);
    assert!(health_monitor.is_healthy().await);

    // Test with some resource allocation
    let request = ResourceRequest::new(
        "health-test-agent".to_string(),
        ResourceType::Memory,
        10 * 1024 * 1024, // 10MB
    );
    resource_manager.allocate(request).await.unwrap();

    let result = health_monitor.check_health().await.unwrap();
    assert_eq!(result.status, HealthStatus::Healthy);
    assert!(result.metrics.contains_key("allocation_count"));

    // Test error state
    state_machine
        .error("Health test error".to_string())
        .await
        .unwrap();
    let result = health_monitor.check_health().await.unwrap();
    assert_eq!(result.status, HealthStatus::Critical);
    assert!(!health_monitor.is_healthy().await);

    // Test recovery
    state_machine.recover().await.unwrap();
    let result = health_monitor.check_health().await.unwrap();
    assert_eq!(result.status, HealthStatus::Healthy);

    // Test health history
    let history = health_monitor.get_health_history().await;
    assert!(!history.is_empty());

    let latest = health_monitor.get_latest_result().await;
    assert!(latest.is_some());
}
#[tokio::test]
async fn test_error_scenarios_and_recovery() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let resource_manager = Arc::new(ResourceManager::new(
        ResourceLimits::default(),
        event_system.clone(),
    ));
    let state_machine = Arc::new(AgentStateMachine::default("error-test-agent".to_string()));

    // Test invalid state transitions
    let result = state_machine.start().await;
    assert!(result.is_err()); // Can't start from Uninitialized

    state_machine.initialize().await.unwrap();

    let result = state_machine.pause().await;
    assert!(result.is_err()); // Can't pause from Ready

    // Test error state and recovery
    state_machine.start().await.unwrap();
    state_machine
        .error("Test error condition".to_string())
        .await
        .unwrap();

    assert_eq!(state_machine.current_state().await, AgentState::Error);
    assert_eq!(
        state_machine.get_last_error().await,
        Some("Test error condition".to_string())
    );

    // Test successful recovery
    state_machine.recover().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Ready);
    assert_eq!(state_machine.get_last_error().await, None);

    // Test resource allocation failures
    let excessive_request = ResourceRequest::new(
        "error-test-agent".to_string(),
        ResourceType::Memory,
        10 * 1024 * 1024 * 1024, // 10GB (exceeds default limit)
    );
    let result = resource_manager.allocate(excessive_request).await;
    assert!(result.is_err());

    // Test shutdown from error state
    state_machine
        .error("Another error".to_string())
        .await
        .unwrap();

    let shutdown_coordinator = ShutdownCoordinator::new(
        event_system,
        resource_manager.clone(),
        ShutdownConfig::default(),
    );

    let shutdown_request = ShutdownRequest::new("error-test-agent".to_string())
        .with_reason("Shutdown from error state".to_string());

    let result = shutdown_coordinator
        .shutdown_agent(shutdown_request, state_machine.clone())
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(state_machine.current_state().await, AgentState::Terminated);
}
#[tokio::test]
async fn test_concurrent_operations() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let resource_manager = Arc::new(ResourceManager::new(
        ResourceLimits::default(),
        event_system.clone(),
    ));

    // Test concurrent resource allocations
    let mut handles = Vec::new();
    for i in 0..10 {
        let agent_id = format!("concurrent-agent-{i}");
        let resource_manager = resource_manager.clone();

        let handle = tokio::spawn(async move {
            let request = ResourceRequest::new(
                agent_id,
                ResourceType::Memory,
                1024 * 1024, // 1MB
            );
            resource_manager.allocate(request).await
        });
        handles.push(handle);
    }

    // Wait for all allocations
    let mut successful_allocations = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            successful_allocations += 1;
        }
    }

    assert_eq!(successful_allocations, 10);

    // Test concurrent state machine operations
    let mut state_handles = Vec::new();
    for i in 0..5 {
        let agent_id = format!("state-agent-{i}");
        let state_machine = Arc::new(AgentStateMachine::default(agent_id));

        let handle = tokio::spawn(async move {
            state_machine.initialize().await?;
            state_machine.start().await?;
            state_machine.pause().await?;
            state_machine.resume().await?;
            state_machine.terminate().await?;
            Ok::<(), anyhow::Error>(())
        });
        state_handles.push(handle);
    }

    // Wait for all state transitions
    for handle in state_handles {
        handle.await.unwrap().unwrap();
    }
}
#[tokio::test]
async fn test_performance_requirements() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let resource_manager = Arc::new(ResourceManager::new(
        ResourceLimits::default(),
        event_system.clone(),
    ));
    let state_machine = Arc::new(AgentStateMachine::default("perf-test-agent".to_string()));

    // Test state transition performance (should be < 10ms as per requirements)
    let start = std::time::Instant::now();
    state_machine.initialize().await.unwrap();
    let initialization_time = start.elapsed();
    assert!(
        initialization_time < Duration::from_millis(10),
        "Initialization took {initialization_time:?}, expected < 10ms"
    );

    // Test resource allocation performance
    let start = std::time::Instant::now();
    let request = ResourceRequest::new(
        "perf-test-agent".to_string(),
        ResourceType::Memory,
        1024 * 1024,
    );
    let _allocation = resource_manager.allocate(request).await.unwrap();
    let allocation_time = start.elapsed();
    assert!(
        allocation_time < Duration::from_millis(50),
        "Resource allocation took {allocation_time:?}, expected < 50ms"
    );

    // Test health check performance
    let health_monitor = AgentHealthMonitor::new(
        "perf-test-agent".to_string(),
        state_machine.clone(),
        resource_manager.clone(),
        event_system,
        HealthMonitorConfig::default(),
    );

    health_monitor
        .add_health_check(Arc::new(StateMachineHealthCheck::new(
            state_machine.clone(),
        )))
        .await;

    let start = std::time::Instant::now();
    let _result = health_monitor.check_health().await.unwrap();
    let health_check_time = start.elapsed();
    assert!(
        health_check_time < Duration::from_millis(100),
        "Health check took {health_check_time:?}, expected < 100ms"
    );
}

/// Custom middleware for testing middleware chain
struct TestMiddleware {
    name: String,
    should_fail: bool,
}

impl TestMiddleware {
    fn new(name: &str, should_fail: bool) -> Self {
        Self {
            name: name.to_string(),
            should_fail,
        }
    }
}

#[async_trait]
impl LifecycleMiddleware for TestMiddleware {
    async fn before(&self, context: &mut MiddlewareContext) -> Result<()> {
        context.set_data(&format!("{}_before", self.name), "executed");
        if self.should_fail {
            return Err(anyhow::anyhow!("Test middleware {} failed", self.name));
        }
        Ok(())
    }

    async fn after(&self, context: &mut MiddlewareContext) -> Result<()> {
        context.set_data(&format!("{}_after", self.name), "executed");
        Ok(())
    }

    async fn on_error(
        &self,
        context: &mut MiddlewareContext,
        _error: &anyhow::Error,
    ) -> Result<()> {
        context.set_data(&format!("{}_error", self.name), "handled");
        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to(&self, _phase: LifecyclePhase) -> bool {
        true
    }
}
#[tokio::test]
async fn test_custom_middleware() {
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
    let middleware_chain = LifecycleMiddlewareChain::new(event_system, MiddlewareConfig::default());

    // Add custom test middleware
    middleware_chain
        .add_middleware(Arc::new(TestMiddleware::new("test1", false)))
        .await;
    middleware_chain
        .add_middleware(Arc::new(TestMiddleware::new("test2", false)))
        .await;

    let context = MiddlewareContext::new(
        "custom-test-agent".to_string(),
        LifecyclePhase::Initialization,
    );

    let result = middleware_chain.execute(context).await.unwrap();

    // Check that both middleware executed
    assert_eq!(
        result.get_data("test1_before"),
        Some(&"executed".to_string())
    );
    assert_eq!(
        result.get_data("test1_after"),
        Some(&"executed".to_string())
    );
    assert_eq!(
        result.get_data("test2_before"),
        Some(&"executed".to_string())
    );
    assert_eq!(
        result.get_data("test2_after"),
        Some(&"executed".to_string())
    );
}
