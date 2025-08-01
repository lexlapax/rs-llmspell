//! ABOUTME: Integration tests for agent communication patterns and message passing
//! ABOUTME: Tests agent-to-agent communication, tool invocation, and coordination protocols

use llmspell_agents::testing::mocks;

use llmspell_agents::AgentState;
use llmspell_core::{
    traits::tool_capable::ToolCapable,
    types::{AgentInput, ToolCall},
    BaseAgent, ExecutionContext,
};
use mocks::{MockAgentBuilder, TestDoubles};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc, RwLock};

/// Test basic agent-to-agent communication
#[tokio::test]
async fn test_agent_to_agent_communication() {
    // Create two mock agents
    let agent1 = MockAgentBuilder::new("agent1")
        .agent_type("coordinator")
        .with_response(Some("coordinate".to_string()), "Coordinating with agent2")
        .build();

    let agent2 = MockAgentBuilder::new("agent2")
        .agent_type("worker")
        .with_response(Some("task".to_string()), "Task completed")
        .build();

    // Test communication flow
    let input1 = AgentInput::text("coordinate task");
    let output1 = agent1
        .execute(input1, ExecutionContext::default())
        .await
        .unwrap();
    assert!(output1.text.contains("Coordinating"));

    let input2 = AgentInput::text("execute task");
    let output2 = agent2
        .execute(input2, ExecutionContext::default())
        .await
        .unwrap();
    assert!(output2.text.contains("Task completed"));

    // Verify execution counts
    assert_eq!(agent1.execution_count(), 1);
    assert_eq!(agent2.execution_count(), 1);
}

/// Test agent tool invocation patterns
#[tokio::test]
async fn test_agent_tool_invocation() {
    // Create agent with tool capability
    let agent = MockAgentBuilder::new("tool_agent")
        .agent_type("tool_capable")
        .with_tool("calculator")
        .with_tool_response(
            Some("calculate".to_string()),
            "Calculating...",
            vec![ToolCall {
                tool_id: "calc1".to_string(),
                tool_name: "calculator".to_string(),
                parameters: HashMap::from([
                    ("operation".to_string(), serde_json::json!("add")),
                    ("a".to_string(), serde_json::json!(20)),
                    ("b".to_string(), serde_json::json!(22)),
                ]),
                result: None,
            }],
        )
        .build();

    // Test tool invocation through agent
    let input = AgentInput::text("calculate 20 + 22");
    let output = agent
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(output.text.contains("Calculating"));
    assert_eq!(output.tool_calls.len(), 1);
    assert_eq!(output.tool_calls[0].tool_name, "calculator");

    // Test tool availability
    assert!(agent.tool_available("calculator").await);
}

/// Test broadcast communication pattern
#[tokio::test]
async fn test_broadcast_communication() {
    let (tx, _) = broadcast::channel(100);
    let mut receivers = Vec::new();

    // Create multiple receiver agents
    for i in 0..3 {
        let mut rx = tx.subscribe();
        let agent_name = format!("receiver_{}", i);

        receivers.push(tokio::spawn(async move {
            let mut messages = Vec::new();
            while let Ok(msg) = rx.recv().await {
                messages.push(msg);
                if messages.len() >= 3 {
                    break;
                }
            }
            (agent_name, messages)
        }));
    }

    // Send messages
    tx.send("message1".to_string()).unwrap();
    tx.send("message2".to_string()).unwrap();
    tx.send("message3".to_string()).unwrap();

    // Collect results
    for receiver in receivers {
        let (_agent_name, messages) = receiver.await.unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0], "message1");
        assert_eq!(messages[1], "message2");
        assert_eq!(messages[2], "message3");
    }
}

/// Test request-response communication pattern
#[tokio::test]
async fn test_request_response_pattern() {
    let (tx, mut rx) = mpsc::channel::<(String, mpsc::Sender<String>)>(10);

    // Spawn responder
    let responder = tokio::spawn(async move {
        while let Some((request, response_tx)) = rx.recv().await {
            let response = format!("Response to: {}", request);
            response_tx.send(response).await.unwrap();
        }
    });

    // Send requests and collect responses
    let mut responses = Vec::new();
    for i in 0..5 {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        let request = format!("Request {}", i);

        tx.send((request.clone(), response_tx)).await.unwrap();

        if let Some(response) = response_rx.recv().await {
            responses.push(response);
        }
    }

    assert_eq!(responses.len(), 5);
    for (i, response) in responses.iter().enumerate() {
        assert!(response.contains(&format!("Request {}", i)));
    }

    drop(tx);
    responder.await.unwrap();
}

/// Test pipeline communication pattern
#[tokio::test]
async fn test_pipeline_communication() {
    // Create pipeline stages
    let stage1 = MockAgentBuilder::new("preprocessor")
        .with_response(None, "Preprocessed: {input}")
        .build();

    let stage2 = MockAgentBuilder::new("analyzer")
        .with_response(Some("Preprocessed".to_string()), "Analyzed data")
        .build();

    let stage3 = MockAgentBuilder::new("formatter")
        .with_response(Some("Analyzed".to_string()), "Formatted output")
        .build();

    // Execute pipeline
    let input = AgentInput::text("raw data");
    let output1 = stage1
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let input2 = AgentInput::text(&output1.text);
    let output2 = stage2
        .execute(input2, ExecutionContext::default())
        .await
        .unwrap();

    let input3 = AgentInput::text(&output2.text);
    let output3 = stage3
        .execute(input3, ExecutionContext::default())
        .await
        .unwrap();

    assert!(output3.text.contains("Formatted output"));
}

/// Test error propagation in communication
#[tokio::test]
async fn test_error_propagation() {
    // Create failing agent
    let failing_agent = TestDoubles::failing_agent("error_agent", "Communication error");

    let input = AgentInput::text("test");
    let result = failing_agent
        .execute(input, ExecutionContext::default())
        .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Communication error"));
}

/// Test timeout handling in communication
#[tokio::test]
async fn test_communication_timeout() {
    let slow_agent = TestDoubles::slow_agent("slow_agent", Duration::from_secs(2));

    let input = AgentInput::text("test");
    let execution = slow_agent.execute(input, ExecutionContext::default());

    let result = tokio::time::timeout(Duration::from_millis(500), execution).await;
    assert!(result.is_err()); // Should timeout
}

/// Test concurrent communication
#[tokio::test]
async fn test_concurrent_communication() {
    let agent = Arc::new(
        MockAgentBuilder::new("concurrent_agent")
            .with_response(None, "Processed")
            .build(),
    );

    let mut handles = Vec::new();

    // Spawn multiple concurrent executions
    for i in 0..10 {
        let agent_clone = agent.clone();
        let handle = tokio::spawn(async move {
            let input = AgentInput::text(format!("Task {}", i));
            agent_clone
                .execute(input, ExecutionContext::default())
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 10);
    assert_eq!(agent.execution_count(), 10);
}

/// Test hierarchical communication pattern
#[tokio::test]
async fn test_hierarchical_communication() {
    // Create supervisor agent
    let supervisor = MockAgentBuilder::new("supervisor")
        .agent_type("supervisor")
        .with_tool("worker1")
        .with_tool("worker2")
        .with_response(Some("delegate".to_string()), "Tasks delegated to workers")
        .build();

    // Create worker agents
    let worker1 = MockAgentBuilder::new("worker1")
        .agent_type("worker")
        .with_response(None, "Worker 1 completed task")
        .build();

    let worker2 = MockAgentBuilder::new("worker2")
        .agent_type("worker")
        .with_response(None, "Worker 2 completed task")
        .build();

    // Test delegation pattern
    let input = AgentInput::text("delegate tasks");
    let output = supervisor
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(output.text.contains("delegated"));
    assert!(supervisor.tool_available("worker1").await);
    assert!(supervisor.tool_available("worker2").await);

    // Workers execute independently
    let worker_input = AgentInput::text("execute");
    let w1_output = worker1
        .execute(worker_input.clone(), ExecutionContext::default())
        .await
        .unwrap();
    let w2_output = worker2
        .execute(worker_input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(w1_output.text.contains("Worker 1"));
    assert!(w2_output.text.contains("Worker 2"));
}

/// Test event-driven communication
#[tokio::test]
async fn test_event_driven_communication() {
    let (event_tx, mut event_rx) = broadcast::channel(100);

    // Create event producer agent
    let mut producer = MockAgentBuilder::new("event_producer")
        .with_response(None, "Event produced")
        .build();
    producer.set_event_sender(event_tx.clone());

    // Create event consumer
    let consumer = tokio::spawn(async move {
        let mut events = Vec::new();
        while let Ok(event) = event_rx.recv().await {
            events.push(event);
            if events.len() >= 2 {
                break;
            }
        }
        events
    });

    // Producer generates events
    producer.add_state_transition(AgentState::Ready);
    producer.add_state_transition(AgentState::Running);

    let input = AgentInput::text("produce events");
    producer
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Collect consumed events
    drop(event_tx);
    let events = consumer.await.unwrap();
    assert!(!events.is_empty());
}

/// Test shared state communication
#[tokio::test]
async fn test_shared_state_communication() {
    let shared_state = Arc::new(RwLock::new(HashMap::<String, String>::new()));

    // Writer agent
    let state_clone = shared_state.clone();
    let writer = tokio::spawn(async move {
        for i in 0..5 {
            let mut state = state_clone.write().await;
            state.insert(format!("key{}", i), format!("value{}", i));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    // Reader agent
    let state_clone = shared_state.clone();
    let reader = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let state = state_clone.read().await;
        state.len()
    });

    writer.await.unwrap();
    let count = reader.await.unwrap();
    assert_eq!(count, 5);
}

/// Test communication resilience
#[tokio::test]
async fn test_communication_resilience() {
    // Create agent that fails intermittently
    let agent = Arc::new(MockAgentBuilder::new("resilient_agent").build());

    // Test alternating success/failure pattern
    let mut success_count = 0;
    for i in 0..10 {
        // Set failure state before each attempt
        agent.set_failure(i % 2 == 0, "Intermittent failure");

        // Small delay to ensure state is set
        tokio::time::sleep(Duration::from_millis(10)).await;

        let input = AgentInput::text(format!("attempt {}", i));
        let result = agent.execute(input, ExecutionContext::default()).await;

        if result.is_ok() {
            success_count += 1;
        }
    }

    // Should have roughly 50% success rate
    assert!(
        success_count == 5,
        "Expected 50% success rate (5 successes), got {} successes out of 10",
        success_count
    );
}

/// Test communication with context propagation
#[tokio::test]
async fn test_context_propagation() {
    let agent = MockAgentBuilder::new("context_agent")
        .with_response(None, "Processed with context")
        .build();

    // Create context with IDs
    let mut context = ExecutionContext::new();
    context.conversation_id = Some("conv123".to_string());
    context.user_id = Some("user123".to_string());
    context.session_id = Some("session456".to_string());

    let input = AgentInput::text("process with context");
    let _output = agent.execute(input, context.clone()).await.unwrap();

    // Verify context was received
    let last_context = agent.last_context().unwrap();
    assert_eq!(last_context.conversation_id, Some("conv123".to_string()));
    assert_eq!(last_context.user_id, Some("user123".to_string()));
    assert_eq!(last_context.session_id, Some("session456".to_string()));
}
