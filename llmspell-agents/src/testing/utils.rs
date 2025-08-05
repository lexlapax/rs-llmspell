//! ABOUTME: Utility functions and helpers for agent testing
//! ABOUTME: Provides common utilities for test setup, assertions, and data generation

use crate::factory::{AgentConfig, ResourceLimits};
use llmspell_core::{
    types::{AgentInput, ColorSpace, ImageFormat, ImageMetadata, MediaContent, MediaType},
    ExecutionContext,
};
use std::time::Duration;

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate random agent input
    #[must_use]
    pub fn random_input() -> AgentInput {
        let texts = [
            "Hello, how are you?",
            "What's the weather like?",
            "Can you help me with a task?",
            "Tell me a story",
            "Explain quantum physics",
        ];

        let idx = rand::random::<usize>() % texts.len();
        AgentInput::text(texts[idx])
    }

    /// Generate agent input with media
    #[must_use]
    pub fn input_with_media() -> AgentInput {
        AgentInput::builder()
            .text("Analyze this image")
            .add_media(MediaContent::Image {
                data: vec![0, 1, 2, 3], // Mock image data
                format: ImageFormat::Png,
                metadata: ImageMetadata {
                    width: 100,
                    height: 100,
                    color_space: ColorSpace::RGB,
                    has_transparency: false,
                    dpi: Some(72),
                },
            })
            .build()
    }

    /// Generate complex input
    #[must_use]
    pub fn complex_input() -> AgentInput {
        AgentInput::builder()
            .text("Multi-modal request")
            .parameter("temperature", 0.7)
            .parameter("max_tokens", 1000)
            .output_modalities(vec![MediaType::Text, MediaType::Image])
            .build()
    }

    /// Generate execution context with metadata
    #[must_use]
    pub fn context_with_metadata() -> ExecutionContext {
        let mut context = ExecutionContext::new();
        context.conversation_id = Some(uuid::Uuid::new_v4().to_string());
        context.user_id = Some("test_user".to_string());
        context.session_id = Some(uuid::Uuid::new_v4().to_string());
        context
    }
}

/// Test agent configurations
pub struct TestConfigs;

impl TestConfigs {
    /// Basic agent configuration
    #[must_use]
    pub fn basic_agent() -> AgentConfig {
        AgentConfig {
            name: "test_agent".to_string(),
            description: "Test agent for unit tests".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Tool-enabled agent configuration
    #[must_use]
    pub fn tool_agent() -> AgentConfig {
        AgentConfig {
            name: "tool_agent".to_string(),
            description: "Agent with tool capabilities".to_string(),
            agent_type: "tool_capable".to_string(),
            model: None,
            allowed_tools: vec![
                "calculator".to_string(),
                "search".to_string(),
                "file_reader".to_string(),
            ],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Resource-limited agent configuration
    #[must_use]
    pub fn limited_agent() -> AgentConfig {
        AgentConfig {
            name: "limited_agent".to_string(),
            description: "Agent with resource limits".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 5,
                max_memory_mb: 256,
                max_tool_calls: 10,
                max_recursion_depth: 3,
            },
        }
    }
}

/// Test assertions
pub struct TestAssertions;

impl TestAssertions {
    /// Assert duration is within range
    ///
    /// # Errors
    ///
    /// Returns an error if the duration is outside the specified range
    pub fn assert_duration_range(
        actual: Duration,
        min: Duration,
        max: Duration,
    ) -> Result<(), String> {
        if actual < min {
            return Err(format!("Duration {actual:?} is less than minimum {min:?}"));
        }
        if actual > max {
            return Err(format!("Duration {actual:?} exceeds maximum {max:?}"));
        }
        Ok(())
    }

    /// Assert approximately equal with tolerance
    ///
    /// # Errors
    ///
    /// Returns an error if the difference exceeds the tolerance
    pub fn assert_approx_eq(actual: f64, expected: f64, tolerance: f64) -> Result<(), String> {
        let diff = (actual - expected).abs();
        if diff > tolerance {
            return Err(format!(
                "Value {actual} differs from expected {expected} by {diff} (tolerance: {tolerance})"
            ));
        }
        Ok(())
    }
}

/// Test environment setup
pub struct TestEnvironment;

impl TestEnvironment {
    /// Set up test environment
    pub const fn setup() {
        // Initialize logging for tests
        // Note: logging initialization handled by test framework
    }

    /// Clean up test environment
    pub const fn cleanup() {
        // Clean up any test artifacts
    }

    /// Run with timeout
    ///
    /// # Errors
    ///
    /// Returns an error if the future times out
    pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, String>
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::time::timeout(duration, future).await {
            Ok(result) => Ok(result),
            Err(_) => Err(format!("Operation timed out after {duration:?}")),
        }
    }
}

/// Performance measurement utilities
pub struct PerformanceMeasure {
    start: std::time::Instant,
    name: String,
}

impl PerformanceMeasure {
    /// Start measuring
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            start: std::time::Instant::now(),
            name: name.into(),
        }
    }

    /// End measurement and log
    pub fn end(self) -> Duration {
        let duration = self.start.elapsed();
        tracing::debug!("{} took {:?}", self.name, duration);
        duration
    }
}

/// Test report generator
#[derive(Default)]
pub struct TestReport {
    results: Vec<TestReportEntry>,
}

#[derive(Debug)]
struct TestReportEntry {
    #[allow(dead_code)]
    test_name: String,
    passed: bool,
    duration: Duration,
    #[allow(dead_code)]
    error: Option<String>,
}

impl TestReport {
    /// Create new report
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add test result
    pub fn add_result(
        &mut self,
        test_name: impl Into<String>,
        passed: bool,
        duration: Duration,
        error: Option<String>,
    ) {
        self.results.push(TestReportEntry {
            test_name: test_name.into(),
            passed,
            duration,
            error,
        });
    }

    /// Generate summary
    #[must_use]
    pub fn summary(&self) -> String {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();

        format!(
            "Test Summary: {total} total, {passed} passed, {failed} failed, {total_duration:?} total time"
        )
    }

    /// Get pass rate
    #[must_use]
    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let passed = self.results.iter().filter(|r| r.passed).count() as f64;
        let total = self.results.len() as f64;
        passed / total * 100.0
    }
}
