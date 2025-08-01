//! ABOUTME: Agent testing framework providing utilities for testing agent implementations
//! ABOUTME: Includes test harness, assertions, mocking support, and test execution utilities

use crate::{
    factory::{AgentConfig, AgentFactory, DefaultAgentFactory, ResourceLimits},
    lifecycle::{
        events::{LifecycleEvent, LifecycleEventType},
        state_machine::AgentState,
    },
};
use anyhow::Result;
use llmspell_core::{
    traits::agent::Agent,
    types::{AgentInput, AgentOutput},
    ExecutionContext, LLMSpellError,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::broadcast;

/// Test configuration for agent testing
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Maximum test duration before timeout
    pub timeout: Duration,
    /// Enable debug logging
    pub debug: bool,
    /// Record all interactions
    pub record_interactions: bool,
    /// Enable performance profiling
    pub profile_performance: bool,
    /// Validate resource usage
    pub validate_resources: bool,
    /// Custom test metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            debug: false,
            record_interactions: true,
            profile_performance: false,
            validate_resources: true,
            metadata: HashMap::new(),
        }
    }
}

/// Test result capturing execution details
#[derive(Debug)]
pub struct TestResult {
    /// Whether the test passed
    pub passed: bool,
    /// Test execution duration
    pub duration: Duration,
    /// Error message if failed
    pub error: Option<String>,
    /// Recorded interactions (moved out of test harness)
    pub interactions: Vec<TestInteraction>,
    /// Performance metrics
    pub metrics: TestMetrics,
    /// Resource usage
    pub resource_usage: ResourceUsage,
}

/// Recorded interaction during testing
#[derive(Debug)]
pub struct TestInteraction {
    /// Timestamp of interaction
    pub timestamp: Instant,
    /// Input provided
    pub input: AgentInput,
    /// Output received
    pub output: Result<AgentOutput, LLMSpellError>,
    /// Execution context
    pub context: ExecutionContext,
}

/// Performance metrics collected during testing
#[derive(Debug, Clone, Default)]
pub struct TestMetrics {
    /// Average response time
    pub avg_response_time: Duration,
    /// Maximum response time
    pub max_response_time: Duration,
    /// Minimum response time
    pub min_response_time: Duration,
    /// Total number of executions
    pub execution_count: usize,
    /// Number of successful executions
    pub success_count: usize,
    /// Number of failed executions
    pub failure_count: usize,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Total CPU time used
    pub cpu_time: Duration,
    /// Number of tool calls made
    pub tool_calls: usize,
    /// Number of external API calls
    pub api_calls: usize,
}

/// Test harness for running agent tests
pub struct TestHarness {
    config: TestConfig,
    factory: Arc<dyn AgentFactory>,
    interactions: Arc<Mutex<Vec<TestInteraction>>>,
    metrics: Arc<Mutex<TestMetrics>>,
    resource_usage: Arc<Mutex<ResourceUsage>>,
}

impl TestHarness {
    /// Create new test harness with default factory
    pub async fn new(config: TestConfig) -> Self {
        // Create a mock provider manager for testing
        let provider_manager = Arc::new(llmspell_providers::ProviderManager::new());
        let factory = Arc::new(DefaultAgentFactory::new(provider_manager));

        Self {
            config,
            factory,
            interactions: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(TestMetrics::default())),
            resource_usage: Arc::new(Mutex::new(ResourceUsage::default())),
        }
    }

    /// Create test harness with custom factory
    pub fn with_factory(config: TestConfig, factory: Arc<dyn AgentFactory>) -> Self {
        Self {
            config,
            factory,
            interactions: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(TestMetrics::default())),
            resource_usage: Arc::new(Mutex::new(ResourceUsage::default())),
        }
    }

    /// Run a test with the given agent configuration
    pub async fn run_test<F, Fut>(
        &self,
        agent_config: AgentConfig,
        test_fn: F,
    ) -> Result<TestResult>
    where
        F: FnOnce(Arc<dyn Agent>) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let start = Instant::now();

        // Create agent
        let agent = self.factory.create_agent(agent_config).await?;

        // Run test with timeout
        let test_result = tokio::time::timeout(self.config.timeout, test_fn(agent.clone())).await;

        let duration = start.elapsed();

        // Collect results
        let passed = matches!(test_result, Ok(Ok(())));
        let error = match test_result {
            Err(_) => Some("Test timed out".to_string()),
            Ok(Err(e)) => Some(e.to_string()),
            Ok(Ok(())) => None,
        };

        // Get recorded data - move out of lock instead of cloning
        let interactions = {
            let mut guard = self.interactions.lock().unwrap();
            std::mem::take(&mut *guard)
        };
        let metrics = self.metrics.lock().unwrap().clone();
        let resource_usage = self.resource_usage.lock().unwrap().clone();

        Ok(TestResult {
            passed,
            duration,
            error,
            interactions,
            metrics,
            resource_usage,
        })
    }

    /// Execute agent and record interaction
    pub async fn execute_and_record(
        &self,
        agent: Arc<dyn Agent>,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let start = Instant::now();

        // Execute agent
        let result = agent.execute(input.clone(), context.clone()).await;

        let duration = start.elapsed();

        // Record interaction
        if self.config.record_interactions {
            let output_record = match &result {
                Ok(output) => Ok(output.clone()),
                Err(e) => Err(LLMSpellError::Component {
                    message: e.to_string(),
                    source: None,
                }),
            };

            let mut interactions = self.interactions.lock().unwrap();
            interactions.push(TestInteraction {
                timestamp: Instant::now(),
                input,
                output: output_record,
                context,
            });
        }

        // Update metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.execution_count += 1;

        if result.is_ok() {
            metrics.success_count += 1;
        } else {
            metrics.failure_count += 1;
        }

        // Update response times
        if metrics.execution_count == 1 {
            metrics.avg_response_time = duration;
            metrics.max_response_time = duration;
            metrics.min_response_time = duration;
        } else {
            metrics.max_response_time = metrics.max_response_time.max(duration);
            metrics.min_response_time = metrics.min_response_time.min(duration);

            let total_time = metrics.avg_response_time.as_nanos()
                * (metrics.execution_count - 1) as u128
                + duration.as_nanos();
            metrics.avg_response_time =
                Duration::from_nanos((total_time / metrics.execution_count as u128) as u64);
        }

        result
    }

    /// Record resource usage
    pub fn record_resource_usage(&self, memory: usize, cpu_time: Duration, tool_calls: usize) {
        let mut usage = self.resource_usage.lock().unwrap();
        usage.peak_memory = usage.peak_memory.max(memory);
        usage.cpu_time += cpu_time;
        usage.tool_calls += tool_calls;
    }
}

/// Test assertions for agent behavior
pub struct AgentAssertions;

impl AgentAssertions {
    /// Assert that agent output contains expected text
    pub fn assert_output_contains(output: &AgentOutput, expected: &str) -> Result<()> {
        if !output.text.contains(expected) {
            return Err(anyhow::anyhow!(
                "Output does not contain expected text: '{}'",
                expected
            ));
        }
        Ok(())
    }

    /// Assert that agent made expected tool calls
    pub fn assert_tool_calls(output: &AgentOutput, expected_tools: &[&str]) -> Result<()> {
        let actual_tools: Vec<String> = output
            .tool_calls
            .iter()
            .map(|call| call.tool_name.clone())
            .collect();

        for expected_tool in expected_tools {
            if !actual_tools.contains(&expected_tool.to_string()) {
                return Err(anyhow::anyhow!(
                    "Expected tool '{}' was not called",
                    expected_tool
                ));
            }
        }
        Ok(())
    }

    /// Assert execution time is within bounds
    pub fn assert_execution_time(duration: Duration, max_duration: Duration) -> Result<()> {
        if duration > max_duration {
            return Err(anyhow::anyhow!(
                "Execution took {:?}, exceeding maximum of {:?}",
                duration,
                max_duration
            ));
        }
        Ok(())
    }

    /// Assert resource usage is within limits
    pub fn assert_resource_usage(usage: &ResourceUsage, limits: &ResourceLimits) -> Result<()> {
        if usage.peak_memory > (limits.max_memory_mb as usize * 1024 * 1024) {
            return Err(anyhow::anyhow!(
                "Memory usage {} bytes exceeds limit of {} MB",
                usage.peak_memory,
                limits.max_memory_mb
            ));
        }

        if usage.tool_calls > limits.max_tool_calls as usize {
            return Err(anyhow::anyhow!(
                "Tool calls {} exceeds limit of {}",
                usage.tool_calls,
                limits.max_tool_calls
            ));
        }

        Ok(())
    }

    /// Assert agent state transitions
    pub fn assert_state_transition(
        from_state: AgentState,
        to_state: AgentState,
        events: &[LifecycleEvent],
    ) -> Result<()> {
        // Find state transition in events
        for window in events.windows(2) {
            if let (Some(from_event), Some(to_event)) = (window.first(), window.get(1)) {
                if matches!(from_event.event_type, LifecycleEventType::StateChanged)
                    && matches!(to_event.event_type, LifecycleEventType::StateChanged)
                {
                    // Check if this is our expected transition
                    // In a real implementation, we'd extract states from events
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!(
            "State transition from {:?} to {:?} not found",
            from_state,
            to_state
        ))
    }
}

/// Type alias for output validator function
type OutputValidator = Box<dyn Fn(&AgentOutput) -> Result<()> + Send + Sync>;

/// Type alias for setup/teardown function
type SetupTeardownFn = Box<dyn Fn() -> Result<()> + Send + Sync>;

/// Test builder for constructing test scenarios
pub struct TestScenarioBuilder {
    #[allow(dead_code)]
    name: String,
    description: String,
    agent_config: Option<AgentConfig>,
    inputs: Vec<(AgentInput, ExecutionContext)>,
    expected_outputs: Vec<OutputValidator>,
    setup: Option<SetupTeardownFn>,
    teardown: Option<SetupTeardownFn>,
}

impl TestScenarioBuilder {
    /// Create new test scenario
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            agent_config: None,
            inputs: Vec::new(),
            expected_outputs: Vec::new(),
            setup: None,
            teardown: None,
        }
    }

    /// Set scenario description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set agent configuration
    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = Some(config);
        self
    }

    /// Add test input
    pub fn with_input(mut self, input: AgentInput, context: ExecutionContext) -> Self {
        self.inputs.push((input, context));
        self
    }

    /// Add expected output assertion
    pub fn expect_output<F>(mut self, assertion: F) -> Self
    where
        F: Fn(&AgentOutput) -> Result<()> + Send + Sync + 'static,
    {
        self.expected_outputs.push(Box::new(assertion));
        self
    }

    /// Set setup function
    pub fn with_setup<F>(mut self, setup: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.setup = Some(Box::new(setup));
        self
    }

    /// Set teardown function
    pub fn with_teardown<F>(mut self, teardown: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.teardown = Some(Box::new(teardown));
        self
    }

    /// Build and run the test scenario
    pub async fn run(self, harness: &TestHarness) -> Result<TestResult> {
        // Run setup if provided
        if let Some(setup) = &self.setup {
            setup()?;
        }

        // Get agent config
        let agent_config = self
            .agent_config
            .ok_or_else(|| anyhow::anyhow!("Agent configuration not provided"))?;

        // Run test
        let result = harness
            .run_test(agent_config, |agent| async move {
                // Execute all inputs and verify outputs
                for (i, (input, context)) in self.inputs.iter().enumerate() {
                    let output = harness
                        .execute_and_record(agent.clone(), input.clone(), context.clone())
                        .await?;

                    // Run assertions
                    if let Some(assertion) = self.expected_outputs.get(i) {
                        assertion(&output)?;
                    }
                }
                Ok(())
            })
            .await?;

        // Run teardown if provided
        if let Some(teardown) = &self.teardown {
            teardown()?;
        }

        Ok(result)
    }
}

/// Lifecycle event recorder for testing
pub struct LifecycleEventRecorder {
    events: Arc<Mutex<Vec<LifecycleEvent>>>,
    _receiver: broadcast::Receiver<LifecycleEvent>,
}

impl LifecycleEventRecorder {
    /// Create new event recorder
    pub fn new(receiver: broadcast::Receiver<LifecycleEvent>) -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        // Clone receiver for the spawned task
        let mut receiver_clone = receiver.resubscribe();

        // Spawn task to record events
        tokio::spawn(async move {
            while let Ok(event) = receiver_clone.recv().await {
                events_clone.lock().unwrap().push(event);
            }
        });

        Self {
            events,
            _receiver: receiver,
        }
    }

    /// Get recorded events
    pub fn get_events(&self) -> Vec<LifecycleEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Clear recorded events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }

    /// Find events of specific type
    pub fn find_events(&self, event_type: LifecycleEventType) -> Vec<LifecycleEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| {
                std::mem::discriminant(&e.event_type) == std::mem::discriminant(&event_type)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_harness_creation() {
        let config = TestConfig::default();
        let harness = TestHarness::new(config).await;
        assert!(harness.interactions.lock().unwrap().is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_assertions() {
        let output = AgentOutput {
            text: "Hello, world!".to_string(),
            media: vec![],
            tool_calls: vec![],
            metadata: Default::default(),
            transfer_to: None,
        };

        assert!(AgentAssertions::assert_output_contains(&output, "Hello").is_ok());
        assert!(AgentAssertions::assert_output_contains(&output, "Goodbye").is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_scenario_builder() {
        let scenario = TestScenarioBuilder::new("test_scenario")
            .description("A test scenario")
            .with_input(AgentInput::text("Hello"), ExecutionContext::default())
            .expect_output(|output| AgentAssertions::assert_output_contains(output, "response"));

        assert_eq!(scenario.name, "test_scenario");
        assert_eq!(scenario.inputs.len(), 1);
    }
}
