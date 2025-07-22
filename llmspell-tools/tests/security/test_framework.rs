//! ABOUTME: Core security test framework with utilities and common patterns
//! ABOUTME: Provides reusable components for security testing across all tools

use llmspell_core::{
    traits::tool::Tool,
    types::{AgentInput, AgentOutput},
    BaseAgent, ExecutionContext, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Security test result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTestResult {
    /// Test name
    pub test_name: String,
    /// Tool being tested
    pub tool_name: String,
    /// Attack vector used
    pub attack_vector: String,
    /// Whether the attack was prevented
    pub prevented: bool,
    /// Response from the tool
    pub response: Option<String>,
    /// Error if any
    pub error: Option<String>,
    /// Execution time
    pub execution_time: Duration,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl SecurityTestResult {
    /// Create a new test result
    pub fn new(test_name: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            tool_name: tool_name.into(),
            attack_vector: String::new(),
            prevented: false,
            response: None,
            error: None,
            execution_time: Duration::default(),
            metadata: HashMap::new(),
        }
    }

    /// Set the attack vector
    pub fn with_attack_vector(mut self, vector: impl Into<String>) -> Self {
        self.attack_vector = vector.into();
        self
    }

    /// Mark as prevented
    pub fn mark_prevented(mut self) -> Self {
        self.prevented = true;
        self
    }

    /// Add response
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response = Some(response.into());
        self
    }

    /// Add error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set execution time
    pub fn with_execution_time(mut self, duration: Duration) -> Self {
        self.execution_time = duration;
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Security test case definition
#[derive(Debug, Clone)]
pub struct SecurityTestCase {
    /// Test name
    pub name: String,
    /// Description
    pub description: String,
    /// Attack payload
    pub payload: Value,
    /// Expected behavior
    pub expected_behavior: ExpectedBehavior,
    /// Severity if vulnerable
    pub severity: Severity,
    /// Categories
    pub categories: Vec<TestCategory>,
}

/// Expected behavior for security tests
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedBehavior {
    /// Should reject with error
    Reject,
    /// Should sanitize and continue
    Sanitize,
    /// Should timeout
    Timeout,
    /// Should rate limit
    RateLimit,
    /// Should contain specific text
    ContainsText(String),
    /// Should not contain specific text
    NotContainsText(String),
    /// Custom validation function
    Custom(String), // Function name for custom validation
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Test categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestCategory {
    Injection,
    PathTraversal,
    SSRF,
    XXE,
    ResourceExhaustion,
    Authentication,
    Authorization,
    InformationDisclosure,
    Cryptography,
    Validation,
    DoS,
}

/// Security test runner
pub struct SecurityTestRunner {
    /// Test results
    results: Arc<Mutex<Vec<SecurityTestResult>>>,
    /// Test configuration
    config: SecurityTestConfig,
    /// Statistics
    stats: Arc<Mutex<TestStatistics>>,
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct SecurityTestConfig {
    /// Timeout per test
    pub timeout: Duration,
    /// Enable verbose output
    pub verbose: bool,
    /// Categories to test
    pub categories: Vec<TestCategory>,
    /// Tools to test
    pub tools: Vec<String>,
    /// Skip slow tests
    pub skip_slow: bool,
    /// Maximum concurrent tests
    pub max_concurrent: usize,
}

impl Default for SecurityTestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            verbose: false,
            categories: vec![],
            tools: vec![],
            skip_slow: false,
            max_concurrent: 4,
        }
    }
}

/// Test statistics
#[derive(Debug, Default)]
pub struct TestStatistics {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub vulnerabilities_found: usize,
    pub by_severity: HashMap<Severity, usize>,
    pub by_category: HashMap<TestCategory, usize>,
    pub by_tool: HashMap<String, ToolStats>,
}

#[derive(Debug, Default)]
pub struct ToolStats {
    pub tests_run: usize,
    pub vulnerabilities: usize,
    pub average_response_time: Duration,
}

impl SecurityTestRunner {
    /// Create a new test runner
    pub fn new(config: SecurityTestConfig) -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            config,
            stats: Arc::new(Mutex::new(TestStatistics::default())),
        }
    }

    /// Run a single test case
    pub async fn run_test_case<T: BaseAgent + Tool + Send + Sync>(
        &self,
        tool: &T,
        test_case: &SecurityTestCase,
    ) -> SecurityTestResult {
        let start = Instant::now();
        let mut result = SecurityTestResult::new(&test_case.name, tool.metadata().name.clone())
            .with_attack_vector(format!("{:?}", test_case.payload));

        // Create input from payload
        let input = match self.create_input(&test_case.payload) {
            Ok(input) => input,
            Err(e) => {
                return result
                    .with_error(format!("Failed to create input: {}", e))
                    .with_execution_time(start.elapsed());
            }
        };

        // Execute with timeout
        let execution_result = tokio::time::timeout(
            self.config.timeout,
            tool.execute(input, ExecutionContext::new()),
        )
        .await;

        match execution_result {
            Ok(Ok(output)) => {
                result = result.with_response(output.text.clone());
                result.prevented = self.check_prevention(&output, &test_case.expected_behavior);
            }
            Ok(Err(e)) => {
                result = result.with_error(e.to_string());
                // Error might mean the attack was prevented
                result.prevented = matches!(
                    test_case.expected_behavior,
                    ExpectedBehavior::Reject | ExpectedBehavior::Sanitize
                );
            }
            Err(_) => {
                result = result.with_error("Timeout");
                result.prevented = matches!(test_case.expected_behavior, ExpectedBehavior::Timeout);
            }
        }

        result = result.with_execution_time(start.elapsed());

        // Update statistics
        self.update_stats(&result, test_case).await;

        result
    }

    /// Run multiple test cases
    pub async fn run_test_cases<T: BaseAgent + Tool + Send + Sync>(
        &self,
        tool: &T,
        test_cases: Vec<SecurityTestCase>,
    ) -> Vec<SecurityTestResult> {
        let mut results = Vec::new();

        for test_case in test_cases {
            if self.should_skip_test(&test_case) {
                continue;
            }

            let result = self.run_test_case(tool, &test_case).await;

            if self.config.verbose {
                self.print_result(&result);
            }

            results.push(result);
        }

        results
    }

    /// Create input from payload
    fn create_input(&self, payload: &Value) -> Result<AgentInput> {
        let mut input = AgentInput::text("");

        if let Value::Object(params) = payload {
            // Wrap parameters in a "parameters" object
            let wrapped = json!({ "parameters": params });
            if let Value::Object(map) = wrapped {
                input.parameters = map.into_iter().collect();
            }
        }

        Ok(input)
    }

    /// Check if attack was prevented
    fn check_prevention(&self, output: &AgentOutput, expected: &ExpectedBehavior) -> bool {
        match expected {
            ExpectedBehavior::Reject => false, // If we got output, it wasn't rejected
            ExpectedBehavior::Sanitize => {
                // Check if output appears sanitized
                !output.text.contains("root:")
                    && !output.text.contains("/etc/passwd")
                    && !output.text.contains("DROP TABLE")
            }
            ExpectedBehavior::ContainsText(text) => output.text.contains(text),
            ExpectedBehavior::NotContainsText(text) => !output.text.contains(text),
            _ => true,
        }
    }

    /// Update statistics
    async fn update_stats(&self, result: &SecurityTestResult, test_case: &SecurityTestCase) {
        let mut stats = self.stats.lock().await;
        stats.total_tests += 1;

        if result.prevented {
            stats.passed += 1;
        } else {
            stats.failed += 1;
            stats.vulnerabilities_found += 1;
            *stats.by_severity.entry(test_case.severity).or_insert(0) += 1;
        }

        for category in &test_case.categories {
            *stats.by_category.entry(*category).or_insert(0) += 1;
        }

        let tool_stats = stats.by_tool.entry(result.tool_name.clone()).or_default();
        tool_stats.tests_run += 1;
        if !result.prevented {
            tool_stats.vulnerabilities += 1;
        }
    }

    /// Should skip test
    fn should_skip_test(&self, test_case: &SecurityTestCase) -> bool {
        // Skip if categories don't match
        if !self.config.categories.is_empty() {
            let has_category = test_case
                .categories
                .iter()
                .any(|c| self.config.categories.contains(c));
            if !has_category {
                return true;
            }
        }

        false
    }

    /// Print test result
    fn print_result(&self, result: &SecurityTestResult) {
        let status = if result.prevented {
            "✓ PASS"
        } else {
            "✗ FAIL"
        };
        let color = if result.prevented {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        };
        let reset = "\x1b[0m";

        println!(
            "{}{}{} {} - {} ({}ms)",
            color,
            status,
            reset,
            result.test_name,
            result.tool_name,
            result.execution_time.as_millis()
        );

        if let Some(error) = &result.error {
            println!("  Error: {}", error);
        }
    }

    /// Generate report
    pub async fn generate_report(&self) -> SecurityTestReport {
        let stats = self.stats.lock().await.clone();
        let results = self.results.lock().await.clone();

        SecurityTestReport {
            timestamp: chrono::Utc::now(),
            config: self.config.clone(),
            statistics: stats,
            results: results.clone(),
            vulnerabilities: self.extract_vulnerabilities(&results),
        }
    }

    /// Extract vulnerabilities from results
    fn extract_vulnerabilities(&self, results: &[SecurityTestResult]) -> Vec<Vulnerability> {
        results
            .iter()
            .filter(|r| !r.prevented)
            .map(|r| Vulnerability {
                tool: r.tool_name.clone(),
                test: r.test_name.clone(),
                attack_vector: r.attack_vector.clone(),
                severity: Severity::High, // Would be determined from test case
                description: format!(
                    "Tool {} is vulnerable to attack vector: {}",
                    r.tool_name, r.attack_vector
                ),
                remediation: String::new(),
            })
            .collect()
    }
}

/// Security test report
#[derive(Debug, Clone, Serialize)]
pub struct SecurityTestReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub config: SecurityTestConfig,
    pub statistics: TestStatistics,
    pub results: Vec<SecurityTestResult>,
    pub vulnerabilities: Vec<Vulnerability>,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize)]
pub struct Vulnerability {
    pub tool: String,
    pub test: String,
    pub attack_vector: String,
    pub severity: Severity,
    pub description: String,
    pub remediation: String,
}

/// Helper to create test execution context
pub fn create_test_context() -> ExecutionContext {
    ExecutionContext::new()
}

/// Helper to create parameter-based input
pub fn create_params_input(params: Value) -> Result<AgentInput> {
    let mut input = AgentInput::text("");
    let wrapped = json!({ "parameters": params });
    if let Value::Object(map) = wrapped {
        input.parameters = map.into_iter().collect();
    }
    Ok(input)
}

// Implement Serialize for types that need it
impl Serialize for SecurityTestConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SecurityTestConfig", 6)?;
        state.serialize_field("timeout_secs", &self.timeout.as_secs())?;
        state.serialize_field("verbose", &self.verbose)?;
        state.serialize_field(
            "categories",
            &self
                .categories
                .iter()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>(),
        )?;
        state.serialize_field("tools", &self.tools)?;
        state.serialize_field("skip_slow", &self.skip_slow)?;
        state.serialize_field("max_concurrent", &self.max_concurrent)?;
        state.end()
    }
}

impl Serialize for TestStatistics {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TestStatistics", 8)?;
        state.serialize_field("total_tests", &self.total_tests)?;
        state.serialize_field("passed", &self.passed)?;
        state.serialize_field("failed", &self.failed)?;
        state.serialize_field("skipped", &self.skipped)?;
        state.serialize_field("vulnerabilities_found", &self.vulnerabilities_found)?;

        // Convert enums to strings for serialization
        let severity_map: HashMap<String, usize> = self
            .by_severity
            .iter()
            .map(|(k, v)| (format!("{:?}", k), *v))
            .collect();
        state.serialize_field("by_severity", &severity_map)?;

        let category_map: HashMap<String, usize> = self
            .by_category
            .iter()
            .map(|(k, v)| (format!("{:?}", k), *v))
            .collect();
        state.serialize_field("by_category", &category_map)?;

        state.serialize_field("by_tool", &self.by_tool)?;
        state.end()
    }
}

impl Clone for TestStatistics {
    fn clone(&self) -> Self {
        Self {
            total_tests: self.total_tests,
            passed: self.passed,
            failed: self.failed,
            skipped: self.skipped,
            vulnerabilities_found: self.vulnerabilities_found,
            by_severity: self.by_severity.clone(),
            by_category: self.by_category.clone(),
            by_tool: self.by_tool.clone(),
        }
    }
}

impl Serialize for ToolStats {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ToolStats", 3)?;
        state.serialize_field("tests_run", &self.tests_run)?;
        state.serialize_field("vulnerabilities", &self.vulnerabilities)?;
        state.serialize_field(
            "average_response_time_ms",
            &self.average_response_time.as_millis(),
        )?;
        state.end()
    }
}

impl Clone for ToolStats {
    fn clone(&self) -> Self {
        Self {
            tests_run: self.tests_run,
            vulnerabilities: self.vulnerabilities,
            average_response_time: self.average_response_time,
        }
    }
}

impl Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Severity::Low => "Low",
            Severity::Medium => "Medium",
            Severity::High => "High",
            Severity::Critical => "Critical",
        })
    }
}
