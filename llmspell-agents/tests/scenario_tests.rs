//! ABOUTME: Tests using pre-defined test scenarios
//! ABOUTME: Validates agent behavior against common usage patterns

use llmspell_agents::testing::{framework, mocks, scenarios};

use framework::{TestConfig, TestHarness};
use mocks::MockAgentBuilder;
use scenarios::{ScenarioRunner, TestScenarios};

/// Test echo scenario
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_echo_scenario() {
    let scenario = TestScenarios::echo_scenario();

    // Create echo agent
    let agent = MockAgentBuilder::new("echo_agent")
        .with_response(Some("Hello".to_string()), "Hello, world!")
        .with_response(Some("Test".to_string()), "Test message")
        .build();

    // Run scenario
    let result = ScenarioRunner::run_scenario(&agent, &scenario)
        .await
        .unwrap();

    assert!(result.passed);
    assert_eq!(result.scenario_name, "Echo Scenario");
    assert_eq!(result.results.len(), 2);

    // All individual tests should pass
    for test_result in &result.results {
        assert!(test_result.passed);
    }
}

/// Test tool invocation scenario
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_tool_scenario() {
    use llmspell_core::types::ToolCall;

    let scenario = TestScenarios::tool_scenario();

    // Create agent with tool capabilities
    let agent = MockAgentBuilder::new("tool_agent")
        .with_tool("calculator")
        .with_tool("search")
        .with_tool_response(
            Some("Calculate".to_string()),
            "Calculating result...",
            vec![ToolCall {
                tool_id: "calc1".to_string(),
                tool_name: "calculator".to_string(),
                parameters: std::collections::HashMap::new(),
                result: None,
            }],
        )
        .with_tool_response(
            Some("Search".to_string()),
            "Searching for information...",
            vec![ToolCall {
                tool_id: "search1".to_string(),
                tool_name: "search".to_string(),
                parameters: std::collections::HashMap::new(),
                result: None,
            }],
        )
        .build();

    // Run scenario
    let result = ScenarioRunner::run_scenario(&agent, &scenario)
        .await
        .unwrap();

    assert!(result.passed);
    assert_eq!(result.results.len(), 2);
}

/// Test error handling scenario
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_error_scenario() {
    let scenario = TestScenarios::error_scenario();

    // Create agent that fails on error-related inputs
    let agent = MockAgentBuilder::new("error_agent")
        .will_fail("Error triggered")
        .build();

    // Run scenario - expecting errors
    let result = ScenarioRunner::run_scenario(&agent, &scenario)
        .await
        .unwrap();

    // Scenario should still complete even with errors
    assert_eq!(result.scenario_name, "Error Scenario");
    // The scenario passes if errors are handled as expected
    assert!(result.passed); // Errors were expected, so scenario passes
                            // Verify that outputs were indeed errors
    for test_result in &result.results {
        assert!(test_result.output.is_err(), "Expected error output");
    }
}

/// Test performance scenario
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_performance_scenario() {
    let scenario = TestScenarios::performance_scenario();

    // Create fast-responding agent
    let agent = MockAgentBuilder::new("perf_agent")
        .with_response(None, "Quick response")
        .build();

    // Run scenario
    let start = std::time::Instant::now();
    let result = ScenarioRunner::run_scenario(&agent, &scenario)
        .await
        .unwrap();
    let total_duration = start.elapsed();

    assert!(result.passed);
    assert_eq!(result.results.len(), 100);

    // Should handle 100 requests quickly
    assert!(total_duration < std::time::Duration::from_secs(5));

    // Average response time should be low
    let avg_time = total_duration / 100;
    assert!(avg_time < std::time::Duration::from_millis(50));
}

/// Test state transition scenario
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_state_transition_scenario() {
    use llmspell_agents::AgentState;

    let scenario = TestScenarios::state_transition_scenario();

    // Create stateful agent
    let agent = MockAgentBuilder::new("state_agent")
        .with_state_transition(AgentState::Ready)
        .with_response(Some("Initialize".to_string()), "Initialized")
        .with_state_transition(AgentState::Running)
        .with_response(Some("Start".to_string()), "Started")
        .with_state_transition(AgentState::Paused)
        .with_response(Some("Pause".to_string()), "Paused")
        .with_state_transition(AgentState::Running)
        .with_response(Some("Resume".to_string()), "Resumed")
        .with_state_transition(AgentState::Ready)
        .with_response(Some("Stop".to_string()), "Stopped")
        .build();

    // Run scenario
    let result = ScenarioRunner::run_scenario(&agent, &scenario)
        .await
        .unwrap();

    // Note: Our mock doesn't actually validate state transitions
    // This test is more about the scenario framework working
    assert_eq!(result.scenario_name, "State Transition Scenario");
    assert_eq!(result.results.len(), 5);
}

/// Test scenario with test harness integration
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_scenario_with_harness() {
    let config = TestConfig {
        timeout: std::time::Duration::from_secs(10),
        debug: true,
        record_interactions: true,
        profile_performance: true,
        validate_resources: true,
        metadata: Default::default(),
    };

    let harness = TestHarness::new(config);

    // Create test scenario
    let scenario = scenarios::TestScenarios::echo_scenario();

    // Convert scenario to harness test
    let agent_config = llmspell_agents::AgentConfig {
        name: "harness_test_agent".to_string(),
        description: "Agent for harness testing".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec![],
        custom_config: Default::default(),
        resource_limits: Default::default(),
    };

    let test_result = harness
        .await
        .run_test(agent_config, |agent| async move {
            // Note: Agent starts in Uninitialized state but should handle this gracefully
            // The execute method will initialize automatically if needed

            // Run scenario against agent
            let scenario_result = ScenarioRunner::run_scenario(agent.as_ref(), &scenario).await?;

            if scenario_result.passed {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Scenario failed: {:?}", scenario_result))
            }
        })
        .await
        .unwrap();

    if !test_result.passed {
        if let Some(error) = &test_result.error {
            println!("Test failed with error: {}", error);
        }
        println!("Test result: {:?}", test_result);
    }
    assert!(test_result.passed);
    // Note: Interactions might be empty depending on harness implementation
    // Just check that the test ran successfully
}

/// Test custom scenario creation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_custom_scenario() {
    use llmspell_core::types::AgentInput;
    use scenarios::{ExpectedOutput, ScenarioConfig};

    // Create custom scenario
    let custom_scenario = ScenarioConfig {
        name: "Custom Test".to_string(),
        description: "A custom test scenario".to_string(),
        inputs: vec![
            AgentInput::text("ping"),
            AgentInput::text("status"),
            AgentInput::text("help"),
        ],
        expected_outputs: vec![
            ExpectedOutput::Contains("pong"),
            ExpectedOutput::Contains("ready"),
            ExpectedOutput::Contains("Available commands"),
        ],
        timeout: std::time::Duration::from_secs(5),
    };

    // Create agent with matching responses
    let agent = MockAgentBuilder::new("custom_agent")
        .with_response(Some("ping".to_string()), "pong")
        .with_response(Some("status".to_string()), "System ready")
        .with_response(
            Some("help".to_string()),
            "Available commands: ping, status, help",
        )
        .build();

    // Run custom scenario
    let result = ScenarioRunner::run_scenario(&agent, &custom_scenario)
        .await
        .unwrap();

    assert!(result.passed);
    assert_eq!(result.scenario_name, "Custom Test");
    assert_eq!(result.results.len(), 3);

    // All tests should pass
    for (i, test_result) in result.results.iter().enumerate() {
        assert!(test_result.passed, "Test {} failed", i);
    }
}
