//! ABOUTME: Utility functions and helpers for agent testing
//! ABOUTME: Provides common utilities for test setup, assertions, and data generation

use crate::{AgentConfig, ResourceLimits};
use llmspell_core::{
    types::{AgentInput, MediaContent, MediaType},
    ExecutionContext,
};
use std::{collections::HashMap, time::Duration};

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate random agent input
    pub fn random_input() -> AgentInput {
        let texts = vec![
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
    pub fn input_with_media() -> AgentInput {
        AgentInput::builder()
            .text("Analyze this image")
            .add_media(MediaContent {
                media_type: MediaType::Image,
                data: vec![0, 1, 2, 3], // Mock image data
                metadata: HashMap::new(),
            })
            .build()
    }

    /// Generate complex input
    pub fn complex_input() -> AgentInput {
        AgentInput::builder()
            .text("Multi-modal request")
            .parameter("temperature", 0.7)
            .parameter("max_tokens", 1000)
            .output_modalities(vec![MediaType::Text, MediaType::Image])
            .build()
    }

    /// Generate execution context with metadata
    pub fn context_with_metadata() -> ExecutionContext {
        let mut context = ExecutionContext::default();
        context.metadata.insert(
            "request_id".to_string(),
            serde_json::json!(uuid::Uuid::new_v4().to_string()),
        );
        context.metadata.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );
        context
            .metadata
            .insert("source".to_string(), serde_json::json!("test"));
        context
    }
}

/// Test agent configurations
pub struct TestConfigs;

impl TestConfigs {
    /// Basic agent configuration
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
    pub fn assert_duration_range(
        actual: Duration,
        min: Duration,
        max: Duration,
    ) -> Result<(), String> {
        if actual < min {
            return Err(format!(
                "Duration {:?} is less than minimum {:?}",
                actual, min
            ));
        }
        if actual > max {
            return Err(format!("Duration {:?} exceeds maximum {:?}", actual, max));
        }
        Ok(())
    }

    /// Assert approximately equal with tolerance
    pub fn assert_approx_eq(actual: f64, expected: f64, tolerance: f64) -> Result<(), String> {
        let diff = (actual - expected).abs();
        if diff > tolerance {
            return Err(format!(
                "Value {} differs from expected {} by {} (tolerance: {})",
                actual, expected, diff, tolerance
            ));
        }
        Ok(())
    }
}

/// Test environment setup
pub struct TestEnvironment;

impl TestEnvironment {
    /// Set up test environment
    pub fn setup() {
        // Initialize logging for tests
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Clean up test environment
    pub fn cleanup() {
        // Clean up any test artifacts
    }

    /// Run with timeout
    pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, String>
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::time::timeout(duration, future).await {
            Ok(result) => Ok(result),
            Err(_) => Err(format!("Operation timed out after {:?}", duration)),
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
        log::debug!("{} took {:?}", self.name, duration);
        duration
    }
}

/// Test report generator
pub struct TestReport {
    results: Vec<TestReportEntry>,
}

#[derive(Debug)]
struct TestReportEntry {
    test_name: String,
    passed: bool,
    duration: Duration,
    error: Option<String>,
}

impl TestReport {
    /// Create new report
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
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
    pub fn summary(&self) -> String {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();

        format!(
            "Test Summary: {} total, {} passed, {} failed, {:?} total time",
            total, passed, failed, total_duration
        )
    }

    /// Get pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let passed = self.results.iter().filter(|r| r.passed).count() as f64;
        let total = self.results.len() as f64;
        passed / total * 100.0
    }
}
