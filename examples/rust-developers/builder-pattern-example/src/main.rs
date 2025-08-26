//! ============================================================
//! LLMSPELL RUST DEVELOPERS SHOWCASE
//! ============================================================
//! Example ID: 05 - Builder Pattern v1.0.0
//! Complexity Level: INTERMEDIATE
//! Real-World Use Case: Fluent APIs for constructing complex tools and agents
//!
//! Purpose: Demonstrates builder patterns for creating configurable tools and agents
//! Architecture: Builder pattern, fluent interfaces, configuration validation
//! Crates Showcased: llmspell-core (`BaseAgent`, `Tool` traits), builder patterns
//! Key Features:
//!   • Fluent builder APIs for tool construction
//!   • Validation during build process
//!   • Optional and required configuration
//!   • Method chaining for clean syntax
//!
//! Prerequisites:
//!   • Rust 1.70+, tokio async runtime, understanding of builder pattern
//!
//! HOW TO RUN:
//! ```bash
//! cd examples/rust-developers/getting-started/builder-pattern-example
//! cargo build
//! cargo run
//! ```
//!
//! EXPECTED OUTPUT:
//! Builder pattern demonstrations with tool and agent construction
//!
//! Time to Complete: <5 seconds compilation + execution
//! ============================================================

// Allow nursery lint group false positives for const fn on mutable self methods
#![allow(clippy::missing_const_for_fn)]

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::tool::{ParameterDef, ParameterType, SecurityLevel, ToolCategory, ToolSchema},
    types::{AgentInput, AgentOutput},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Tool,
};
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Configuration structure for our tool
#[derive(Debug, Clone)]
pub struct ToolConfig {
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub cache_enabled: bool,
    pub log_level: String,
    pub custom_settings: HashMap<String, String>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            timeout_ms: Some(5000),
            max_retries: Some(3),
            cache_enabled: true,
            log_level: "info".to_string(),
            custom_settings: HashMap::new(),
        }
    }
}

// Configurable tool that demonstrates builder pattern
#[derive(Debug)]
pub struct ConfigurableTool {
    metadata: ComponentMetadata,
    config: ToolConfig,
    processors: Vec<String>,
}

impl ConfigurableTool {
    /// Create a builder for this tool
    #[must_use]
    pub fn builder(name: String) -> ConfigurableToolBuilder {
        ConfigurableToolBuilder::new(name)
    }

    /// Process input with configured settings
    fn process_with_config(
        &self,
        input: &str,
        operation: &str,
    ) -> Result<serde_json::Value, String> {
        info!(
            "Processing '{}' with operation '{}' using config: {:?}",
            input, operation, self.config
        );

        // Simulate processing based on configuration
        let start = std::time::Instant::now();

        // Apply timeout if configured
        if let Some(timeout) = self.config.timeout_ms {
            if timeout < 1000 {
                return Err("Simulated timeout - operation took too long".to_string());
            }
        }

        let result = match operation {
            "process" => {
                let mut result_data = json!({
                    "input": input,
                    "operation": operation,
                    "processed": format!("Processed: {}", input),
                    "duration_ms": start.elapsed().as_millis(),
                    "cache_enabled": self.config.cache_enabled,
                    "log_level": self.config.log_level
                });

                // Apply processors
                for processor in &self.processors {
                    match processor.as_str() {
                        "uppercase" => {
                            if let Some(processed) = result_data.get_mut("processed") {
                                *processed = json!(processed.as_str().unwrap_or("").to_uppercase());
                            }
                        }
                        "reverse" => {
                            if let Some(processed) = result_data.get_mut("processed") {
                                *processed = json!(processed
                                    .as_str()
                                    .unwrap_or("")
                                    .chars()
                                    .rev()
                                    .collect::<String>());
                            }
                        }
                        "length" => {
                            result_data["length"] = json!(input.len());
                        }
                        _ => {
                            result_data["unknown_processor"] = json!(processor);
                        }
                    }
                }

                result_data
            }
            "analyze" => {
                json!({
                    "analysis": {
                        "word_count": input.split_whitespace().count(),
                        "char_count": input.len(),
                        "config_timeout": self.config.timeout_ms,
                        "config_retries": self.config.max_retries,
                        "processors_count": self.processors.len()
                    }
                })
            }
            _ => {
                return Err(format!("Unknown operation: {operation}"));
            }
        };

        Ok(result)
    }
}

// Builder for ConfigurableTool
#[derive(Debug)]
pub struct ConfigurableToolBuilder {
    name: String,
    description: Option<String>,
    config: ToolConfig,
    processors: Vec<String>,
}

impl ConfigurableToolBuilder {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            config: ToolConfig::default(),
            processors: Vec::new(),
        }
    }

    /// Set tool description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set timeout in milliseconds
    #[must_use]
    pub fn timeout(mut self, ms: u64) -> Self {
        self.config.timeout_ms = Some(ms);
        self
    }

    /// Set maximum retry attempts
    #[must_use]
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = Some(retries);
        self
    }

    /// Enable or disable caching
    #[must_use]
    pub fn cache(mut self, enabled: bool) -> Self {
        self.config.cache_enabled = enabled;
        self
    }

    /// Set log level
    #[must_use]
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.config.log_level = level.into();
        self
    }

    /// Add a custom setting
    #[must_use]
    pub fn setting(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.custom_settings.insert(key.into(), value.into());
        self
    }

    /// Add a processor to the pipeline
    #[must_use]
    pub fn add_processor(mut self, processor: impl Into<String>) -> Self {
        self.processors.push(processor.into());
        self
    }

    /// Add multiple processors
    #[must_use]
    pub fn processors(mut self, processors: Vec<String>) -> Self {
        self.processors.extend(processors);
        self
    }

    /// Validate and build the tool
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails:
    /// - Empty name
    /// - Zero timeout
    /// - Excessive retry count
    pub fn build(self) -> Result<ConfigurableTool, String> {
        // Validation
        if self.name.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }

        if let Some(timeout) = self.config.timeout_ms {
            if timeout == 0 {
                return Err("Timeout must be greater than 0".to_string());
            }
        }

        if let Some(retries) = self.config.max_retries {
            if retries > 10 {
                return Err("Maximum retries cannot exceed 10".to_string());
            }
        }

        let description = self.description.unwrap_or_else(|| {
            format!(
                "Configurable tool '{}' with {} processors",
                self.name,
                self.processors.len()
            )
        });

        Ok(ConfigurableTool {
            metadata: ComponentMetadata::new(self.name, description),
            config: self.config,
            processors: self.processors,
        })
    }

    /// Build with default validation (returns Result with `anyhow::Error`)
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails during build
    pub fn try_build(self) -> anyhow::Result<ConfigurableTool> {
        self.build().map_err(|e| anyhow::anyhow!(e))
    }
}

#[async_trait]
impl BaseAgent for ConfigurableTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let operation = input
            .parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("process");

        let data = if input.text.is_empty() {
            input
                .parameters
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("default data")
        } else {
            input.text.as_str()
        };

        info!(
            "ConfigurableTool executing operation '{}' on data: '{}'",
            operation, data
        );

        match self.process_with_config(data, operation) {
            Ok(result) => {
                let response = json!({
                    "tool_name": self.metadata.name,
                    "operation": operation,
                    "config_summary": {
                        "timeout_ms": self.config.timeout_ms,
                        "max_retries": self.config.max_retries,
                        "cache_enabled": self.config.cache_enabled,
                        "log_level": self.config.log_level,
                        "custom_settings_count": self.config.custom_settings.len(),
                        "processors": self.processors
                    },
                    "result": result,
                    "success": true
                });

                Ok(AgentOutput::text(response.to_string()))
            }
            Err(error) => Err(LLMSpellError::Component {
                message: format!("Tool execution failed: {error}"),
                source: None,
            }),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if input.text.is_empty() && !input.parameters.contains_key("data") {
            return Err(LLMSpellError::Validation {
                message: "Either input.text or parameters.data must be provided".to_string(),
                field: Some("data".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        let error_response = json!({
            "tool_name": self.metadata.name,
            "error": error.to_string(),
            "config_info": {
                "timeout_ms": self.config.timeout_ms,
                "max_retries": self.config.max_retries
            },
            "success": false
        });

        Ok(AgentOutput::text(error_response.to_string()))
    }
}

#[async_trait]
impl Tool for ConfigurableTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: 'process' or 'analyze'".to_string(),
            required: false,
            default: Some(json!("process")),
        })
        .with_parameter(ParameterDef {
            name: "data".to_string(),
            param_type: ParameterType::String,
            description: "Data to process (or use input.text)".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }
}

// Demonstration functions
fn demonstrate_basic_builder() -> Result<()> {
    println!("=== Basic Builder Pattern ===");

    // Simple builder usage
    let tool = ConfigurableTool::builder("simple_tool".to_string())
        .description("A simple tool created with builder pattern")
        .timeout(3000)
        .cache(true)
        .build()
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("✅ Built tool: {}", tool.metadata.name);
    println!("   Description: {}", tool.metadata.description);
    println!("   Timeout: {:?}ms", tool.config.timeout_ms);
    println!("   Cache enabled: {}", tool.config.cache_enabled);

    Ok(())
}

async fn demonstrate_fluent_builder() -> Result<()> {
    println!("\n=== Fluent Builder Pattern ===");

    // Complex fluent builder
    let advanced_tool = ConfigurableTool::builder("advanced_processor".to_string())
        .description("Advanced tool with multiple processors")
        .timeout(10000)
        .max_retries(5)
        .cache(false)
        .log_level("debug")
        .setting("custom_option", "value1")
        .setting("feature_flag", "enabled")
        .add_processor("uppercase")
        .add_processor("reverse")
        .add_processor("length")
        .try_build()?;

    println!("✅ Built advanced tool: {}", advanced_tool.metadata.name);
    println!("   Processors: {:?}", advanced_tool.processors);
    println!(
        "   Custom settings: {:?}",
        advanced_tool.config.custom_settings
    );

    // Test the tool
    let input =
        AgentInput::text("Hello Builder Pattern!").with_parameter("operation", json!("process"));
    let context = ExecutionContext::new();

    match advanced_tool.execute_impl(input, context).await {
        Ok(output) => {
            println!("   Execution result length: {} chars", output.text.len());
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                if let Some(result) = parsed.get("result").and_then(|r| r.get("processed")) {
                    println!("   Processed result: {}", result.as_str().unwrap_or(""));
                }
            }
        }
        Err(e) => println!("   ❌ Execution failed: {e}"),
    }

    Ok(())
}

fn demonstrate_builder_validation() {
    println!("\n=== Builder Validation ===");

    // Test validation - empty name should fail
    match ConfigurableTool::builder(String::new()).build() {
        Ok(_) => println!("   ❌ Should have failed validation"),
        Err(e) => println!("   ✅ Correctly rejected empty name: {e}"),
    }

    // Test validation - zero timeout should fail
    match ConfigurableTool::builder("test".to_string())
        .timeout(0)
        .build()
    {
        Ok(_) => println!("   ❌ Should have failed validation"),
        Err(e) => println!("   ✅ Correctly rejected zero timeout: {e}"),
    }

    // Test validation - too many retries should fail
    match ConfigurableTool::builder("test".to_string())
        .max_retries(15)
        .build()
    {
        Ok(_) => println!("   ❌ Should have failed validation"),
        Err(e) => println!("   ✅ Correctly rejected excessive retries: {e}"),
    }
}

async fn demonstrate_builder_chaining() -> Result<()> {
    println!("\n=== Builder Method Chaining ===");

    // Demonstrate different chaining patterns
    let processors = vec!["uppercase".to_string(), "length".to_string()];

    let chained_tool = ConfigurableTool::builder("chained_tool".to_string())
        .description("Tool built with method chaining")
        .timeout(2000)
        .cache(true)
        .log_level("warn")
        .processors(processors)
        .setting("mode", "production")
        .setting("debug", "false")
        .try_build()?;

    println!("✅ Built chained tool: {}", chained_tool.metadata.name);

    // Test analysis operation
    let input = AgentInput::text("Method chaining is powerful!")
        .with_parameter("operation", json!("analyze"));
    let context = ExecutionContext::new();

    match chained_tool.execute_impl(input, context).await {
        Ok(output) => {
            println!("   Analysis completed successfully");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                if let Some(analysis) = parsed.get("result").and_then(|r| r.get("analysis")) {
                    println!("   Analysis result: {analysis}");
                }
            }
        }
        Err(e) => println!("   ❌ Analysis failed: {e}"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== LLMSpell Builder Pattern Demo ===\n");

    // Run all demonstrations
    demonstrate_basic_builder()?;
    demonstrate_fluent_builder().await?;
    demonstrate_builder_validation();
    demonstrate_builder_chaining().await?;

    println!("\n✅ Builder Pattern Demo Complete!");
    println!("\n💡 Key Builder Pattern Benefits:");
    println!("   - Fluent API: Readable method chaining for configuration");
    println!("   - Validation: Build-time checks for configuration correctness");
    println!("   - Flexibility: Optional parameters with sensible defaults");
    println!("   - Immutability: Builders consume self, preventing misuse");
    println!("   - Type Safety: Compile-time guarantees for required fields");

    println!("\n📚 Builder Pattern Applications:");
    println!("   - Complex object construction with many optional parameters");
    println!("   - Configuration objects that need validation");
    println!("   - APIs that benefit from method chaining");
    println!("   - Objects with interdependent configuration options");

    println!("\n🛠️ Implementation Patterns Demonstrated:");
    println!("   - ConfigurableTool::builder() - entry point");
    println!("   - Method chaining with self consumption");
    println!("   - build() vs try_build() for different error handling");
    println!("   - Validation before object creation");
    println!("   - Default values and optional configuration");

    Ok(())
}
