//! ABOUTME: Pre-defined test scenarios for common agent testing patterns
//! ABOUTME: Provides reusable scenarios for testing agent behaviors and interactions

use llmspell_core::{
    types::{AgentInput, AgentOutput},
    ExecutionContext,
};
use std::time::Duration;

/// Common test scenarios
pub struct TestScenarios;

impl TestScenarios {
    /// Simple echo scenario
    #[must_use]
    pub fn echo_scenario() -> ScenarioConfig {
        ScenarioConfig {
            name: "Echo Scenario".to_string(),
            description: "Agent echoes input back".to_string(),
            inputs: vec![
                AgentInput::text("Hello, world!"),
                AgentInput::text("Test message"),
            ],
            expected_outputs: vec![
                ExpectedOutput::Contains("Hello, world!"),
                ExpectedOutput::Contains("Test message"),
            ],
            timeout: Duration::from_secs(5),
        }
    }

    /// Tool invocation scenario
    #[must_use]
    pub fn tool_scenario() -> ScenarioConfig {
        ScenarioConfig {
            name: "Tool Scenario".to_string(),
            description: "Agent invokes tools".to_string(),
            inputs: vec![
                AgentInput::text("Calculate 2 + 2"),
                AgentInput::text("Search for information"),
            ],
            expected_outputs: vec![
                ExpectedOutput::ToolCalled("calculator"),
                ExpectedOutput::ToolCalled("search"),
            ],
            timeout: Duration::from_secs(10),
        }
    }

    /// Error handling scenario
    #[must_use]
    pub fn error_scenario() -> ScenarioConfig {
        ScenarioConfig {
            name: "Error Scenario".to_string(),
            description: "Agent handles errors gracefully".to_string(),
            inputs: vec![
                AgentInput::text("Trigger error"),
                AgentInput::text("Invalid request"),
            ],
            expected_outputs: vec![ExpectedOutput::Error, ExpectedOutput::Error],
            timeout: Duration::from_secs(5),
        }
    }

    /// Performance scenario
    #[must_use]
    pub fn performance_scenario() -> ScenarioConfig {
        ScenarioConfig {
            name: "Performance Scenario".to_string(),
            description: "Tests agent performance under load".to_string(),
            inputs: (0..100)
                .map(|i| AgentInput::text(format!("Request {i}")))
                .collect(),
            expected_outputs: (0..100).map(|_| ExpectedOutput::Success).collect(),
            timeout: Duration::from_secs(30),
        }
    }

    /// State transition scenario
    #[must_use]
    pub fn state_transition_scenario() -> ScenarioConfig {
        ScenarioConfig {
            name: "State Transition Scenario".to_string(),
            description: "Tests agent state transitions".to_string(),
            inputs: vec![
                AgentInput::text("Initialize"),
                AgentInput::text("Start"),
                AgentInput::text("Pause"),
                AgentInput::text("Resume"),
                AgentInput::text("Stop"),
            ],
            expected_outputs: vec![
                ExpectedOutput::StateChange("Ready"),
                ExpectedOutput::StateChange("Running"),
                ExpectedOutput::StateChange("Paused"),
                ExpectedOutput::StateChange("Running"),
                ExpectedOutput::StateChange("Ready"),
            ],
            timeout: Duration::from_secs(10),
        }
    }
}

/// Configuration for a test scenario
#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    pub name: String,
    pub description: String,
    pub inputs: Vec<AgentInput>,
    pub expected_outputs: Vec<ExpectedOutput>,
    pub timeout: Duration,
}

/// Expected output patterns
#[derive(Debug, Clone)]
pub enum ExpectedOutput {
    /// Output should contain text
    Contains(&'static str),
    /// Output should exactly match
    Exact(&'static str),
    /// Tool should be called
    ToolCalled(&'static str),
    /// Should result in error
    Error,
    /// Should succeed
    Success,
    /// State should change to
    StateChange(&'static str),
}

impl ExpectedOutput {
    /// Validate output against expectation
    #[must_use]
    pub fn validate(&self, output: &Result<AgentOutput, llmspell_core::LLMSpellError>) -> bool {
        match (self, output) {
            (Self::Error, Err(_)) | (Self::Success, Ok(_)) => true,
            (Self::Contains(text), Ok(output)) => output.text.contains(text),
            (Self::Exact(text), Ok(output)) => output.text == *text,
            (Self::ToolCalled(tool), Ok(output)) => {
                output.tool_calls.iter().any(|call| call.tool_name == *tool)
            }
            _ => false,
        }
    }
}

/// Scenario runner
pub struct ScenarioRunner;

impl ScenarioRunner {
    /// Run a scenario against an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent execution fails during scenario testing.
    pub async fn run_scenario(
        agent: &dyn llmspell_core::BaseAgent,
        scenario: &ScenarioConfig,
    ) -> Result<ScenarioResult, anyhow::Error> {
        let start = std::time::Instant::now();
        let mut results = Vec::new();
        let mut all_passed = true;

        for (input, expected) in scenario.inputs.iter().zip(&scenario.expected_outputs) {
            let output = agent
                .execute(input.clone(), ExecutionContext::default())
                .await;
            let passed = expected.validate(&output);

            if !passed {
                all_passed = false;
            }

            results.push(TestResult {
                input: input.clone(),
                output,
                expected: expected.clone(),
                passed,
            });
        }

        Ok(ScenarioResult {
            scenario_name: scenario.name.clone(),
            passed: all_passed,
            duration: start.elapsed(),
            results,
        })
    }
}

/// Result of running a scenario
#[derive(Debug)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub results: Vec<TestResult>,
}

/// Individual test result
#[derive(Debug)]
pub struct TestResult {
    pub input: AgentInput,
    pub output: Result<AgentOutput, llmspell_core::LLMSpellError>,
    pub expected: ExpectedOutput,
    pub passed: bool,
}
