//! ============================================================
//! LLMSPELL RUST DEVELOPERS SHOWCASE
//! ============================================================
//! Example ID: 04 - Extension Pattern v1.0.0
//! Complexity Level: ADVANCED
//! Real-World Use Case: Creating extensible tools with plugin architecture
//!
//! Purpose: Demonstrates extension patterns for building modular, pluggable tools
//! Architecture: Plugin trait system, dynamic extension loading, registry patterns
//! Crates Showcased: llmspell-core (`BaseAgent`, Tool traits), plugin architecture
//! Key Features:
//!   • Extension trait definitions
//!   • Plugin registration and discovery
//!   • Modular tool composition
//!   • Runtime extension loading
//!
//! Prerequisites:
//!   • Rust 1.70+, tokio async runtime, understanding of trait objects
//!
//! HOW TO RUN:
//! ```bash
//! cd examples/rust-developers/getting-started/extension-pattern-example
//! cargo build
//! cargo run
//! ```
//!
//! EXPECTED OUTPUT:
//! Extension registration, plugin discovery, modular tool execution
//!
//! Time to Complete: <5 seconds compilation + execution
//! ============================================================

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::tool::{ParameterDef, ParameterType, SecurityLevel, ToolCategory, ToolSchema},
    types::{AgentInput, AgentOutput},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Tool,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Core extension trait for pluggable functionality
#[async_trait]
pub trait Extension: Send + Sync + std::fmt::Debug {
    /// Get extension identifier
    fn id(&self) -> &str;

    /// Get extension version
    fn version(&self) -> &str;

    /// Get extension description
    fn description(&self) -> &str;

    /// Initialize the extension (called once during registration)
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    async fn initialize(&self) -> Result<(), String>;

    /// Execute extension with parameters
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails or parameters are invalid
    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String>;

    /// Get supported operations for this extension
    fn supported_operations(&self) -> Vec<String>;
}

// Extension registry for managing plugins
#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    extensions: HashMap<String, Arc<dyn Extension>>,
}

impl ExtensionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    /// Register a new extension
    ///
    /// # Errors
    ///
    /// Returns an error if extension initialization fails or ID conflicts
    pub async fn register(&mut self, extension: Arc<dyn Extension>) -> Result<(), String> {
        let id = extension.id().to_string();

        // Initialize the extension
        extension.initialize().await?;

        // Check for conflicts
        if self.extensions.contains_key(&id) {
            return Err(format!("Extension with id '{id}' already registered"));
        }

        self.extensions.insert(id.clone(), extension);
        info!("Registered extension: {}", id);
        Ok(())
    }

    /// Get extension by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Extension>> {
        self.extensions.get(id)
    }

    /// List all registered extensions
    #[must_use]
    pub fn list(&self) -> Vec<&Arc<dyn Extension>> {
        self.extensions.values().collect()
    }

    /// Get extension IDs
    #[must_use]
    pub fn extension_ids(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
}

// Extensible tool that uses registered extensions
#[derive(Debug)]
pub struct ExtensibleTool {
    metadata: ComponentMetadata,
    registry: Arc<ExtensionRegistry>,
}

impl ExtensibleTool {
    #[must_use]
    pub fn new(name: String, registry: Arc<ExtensionRegistry>) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name,
                "Tool that executes operations through registered extensions".to_string(),
            ),
            registry,
        }
    }
}

#[async_trait]
impl BaseAgent for ExtensibleTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Extract parameters
        let extension_id = input
            .parameters
            .get("extension")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'extension' parameter".to_string(),
                field: Some("extension".to_string()),
            })?;

        let operation = input
            .parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Get the extension
        let extension =
            self.registry
                .get(extension_id)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Extension '{extension_id}' not found"),
                    source: None,
                })?;

        // Prepare parameters for extension
        let mut ext_params = HashMap::new();
        ext_params.insert("operation".to_string(), json!(operation));

        // Pass through all other parameters
        for (key, value) in &input.parameters {
            if key != "extension" && key != "operation" {
                ext_params.insert(key.clone(), value.clone());
            }
        }

        // If no specific parameters, use input text as data
        if ext_params.len() <= 1 && !input.text.is_empty() {
            ext_params.insert("data".to_string(), json!(input.text));
        }

        info!(
            "Executing extension '{}' with operation '{}'",
            extension_id, operation
        );

        // Execute extension
        match extension.execute(&ext_params).await {
            Ok(result) => {
                let response = json!({
                    "extension_id": extension_id,
                    "extension_version": extension.version(),
                    "operation": operation,
                    "result": result,
                    "success": true
                });

                Ok(AgentOutput::text(response.to_string()))
            }
            Err(error) => Err(LLMSpellError::Component {
                message: format!("Extension execution failed: {error}"),
                source: None,
            }),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if !input.parameters.contains_key("extension") {
            return Err(LLMSpellError::Validation {
                message: "Missing required 'extension' parameter".to_string(),
                field: Some("extension".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        let available_extensions: Vec<String> = self.registry.extension_ids();

        let error_response = json!({
            "error": error.to_string(),
            "available_extensions": available_extensions,
            "success": false
        });

        Ok(AgentOutput::text(error_response.to_string()))
    }
}

#[async_trait]
impl Tool for ExtensibleTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        let available_extensions = self.registry.extension_ids();

        ToolSchema::new(
            self.metadata.name.clone(),
            format!(
                "Extensible tool with {} registered extensions",
                available_extensions.len()
            ),
        )
        .with_parameter(ParameterDef {
            name: "extension".to_string(),
            param_type: ParameterType::String,
            description: format!("Extension ID to use. Available: {available_extensions:?}"),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform".to_string(),
            required: false,
            default: Some(json!("default")),
        })
        .with_parameter(ParameterDef {
            name: "data".to_string(),
            param_type: ParameterType::String,
            description: "Data to process".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }
}

// Example extension: Text Processor
#[derive(Debug)]
pub struct TextProcessorExtension;

#[async_trait]
impl Extension for TextProcessorExtension {
    fn id(&self) -> &'static str {
        "text_processor"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Text processing operations like uppercase, lowercase, reverse, etc."
    }

    async fn initialize(&self) -> Result<(), String> {
        info!("Initializing TextProcessorExtension v{}", self.version());
        Ok(())
    }

    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("uppercase");

        let data = params
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'data' parameter for text processing".to_string())?;

        let result = match operation {
            "uppercase" => data.to_uppercase(),
            "lowercase" => data.to_lowercase(),
            "reverse" => data.chars().rev().collect::<String>(),
            "length" => data.len().to_string(),
            "words" => data.split_whitespace().count().to_string(),
            "chars" => data.chars().count().to_string(),
            _ => {
                return Err(format!(
                    "Unknown text operation: '{}'. Supported: {:?}",
                    operation,
                    self.supported_operations()
                ))
            }
        };

        Ok(json!({
            "processed_text": result,
            "original_length": data.len(),
            "operation": operation
        }))
    }

    fn supported_operations(&self) -> Vec<String> {
        vec![
            "uppercase".to_string(),
            "lowercase".to_string(),
            "reverse".to_string(),
            "length".to_string(),
            "words".to_string(),
            "chars".to_string(),
        ]
    }
}

// Example extension: Math Processor
#[derive(Debug)]
pub struct MathProcessorExtension;

#[async_trait]
impl Extension for MathProcessorExtension {
    fn id(&self) -> &'static str {
        "math_processor"
    }

    fn version(&self) -> &'static str {
        "1.2.0"
    }

    fn description(&self) -> &'static str {
        "Mathematical operations on numeric data"
    }

    async fn initialize(&self) -> Result<(), String> {
        info!("Initializing MathProcessorExtension v{}", self.version());
        Ok(())
    }

    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("square");

        let data = params
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'data' parameter for math processing".to_string())?;

        let number: f64 = data
            .parse()
            .map_err(|_| format!("Cannot parse '{data}' as number"))?;

        let result = match operation {
            "square" => number * number,
            "cube" => number * number * number,
            "sqrt" => {
                if number < 0.0 {
                    return Err("Cannot take square root of negative number".to_string());
                }
                number.sqrt()
            }
            "abs" => number.abs(),
            "double" => number * 2.0,
            "half" => number / 2.0,
            _ => {
                return Err(format!(
                    "Unknown math operation: '{}'. Supported: {:?}",
                    operation,
                    self.supported_operations()
                ))
            }
        };

        Ok(json!({
            "result": result,
            "original": number,
            "operation": operation,
            "formatted": format!("{:.3}", result)
        }))
    }

    fn supported_operations(&self) -> Vec<String> {
        vec![
            "square".to_string(),
            "cube".to_string(),
            "sqrt".to_string(),
            "abs".to_string(),
            "double".to_string(),
            "half".to_string(),
        ]
    }
}

/// Initialize the tracing subscriber for logging
fn init_logging() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

/// Set up and register extensions in the registry
async fn setup_extensions(registry: &mut ExtensionRegistry) -> Result<()> {
    println!("\n2. Registering extensions...");

    let text_extension = Arc::new(TextProcessorExtension);
    registry
        .register(text_extension)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register text extension: {e}"))?;

    let math_extension = Arc::new(MathProcessorExtension);
    registry
        .register(math_extension)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register math extension: {e}"))?;

    println!("   ✅ Registered {} extensions", registry.list().len());
    Ok(())
}

/// Test text processing extension
async fn test_text_extension(tool: &ExtensibleTool) -> Result<()> {
    println!("\n4. Testing text processing extension...");

    let text_input = AgentInput::text("Hello Extension Pattern!")
        .with_parameter("extension", json!("text_processor"))
        .with_parameter("operation", json!("reverse"));

    let context = ExecutionContext::new();
    match tool.execute_impl(text_input, context).await {
        Ok(output) => {
            println!("   Text processing result:");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                println!("   Original: Hello Extension Pattern!");
                if let Some(result) = parsed.get("result").and_then(|r| r.get("processed_text")) {
                    println!("   Reversed: {}", result.as_str().unwrap_or(""));
                }
            }
        }
        Err(e) => println!("   ❌ Text processing failed: {e}"),
    }
    Ok(())
}

/// Test math processing extension
async fn test_math_extension(tool: &ExtensibleTool) -> Result<()> {
    println!("\n5. Testing math processing extension...");

    let math_input = AgentInput::text("7.5")
        .with_parameter("extension", json!("math_processor"))
        .with_parameter("operation", json!("square"));

    let context = ExecutionContext::new();
    match tool.execute_impl(math_input, context).await {
        Ok(output) => {
            println!("   Math processing result:");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                if let Some(result) = parsed.get("result").and_then(|r| r.get("result")) {
                    println!("   7.5² = {}", result.as_f64().unwrap_or(0.0));
                }
            }
        }
        Err(e) => println!("   ❌ Math processing failed: {e}"),
    }
    Ok(())
}

/// Demonstrate extension discovery capabilities
fn demonstrate_extension_discovery(registry: &ExtensionRegistry) {
    println!("\n6. Extension discovery...");

    for extension in registry.list() {
        println!(
            "   📦 Extension: {} v{}",
            extension.id(),
            extension.version()
        );
        println!("       Description: {}", extension.description());
        println!("       Operations: {:?}", extension.supported_operations());
    }
}

/// Test error handling with invalid extension
async fn test_error_handling(tool: &ExtensibleTool) -> Result<()> {
    println!("\n7. Testing error handling...");

    let invalid_input = AgentInput::text("test")
        .with_parameter("extension", json!("nonexistent"))
        .with_parameter("operation", json!("test"));

    let context = ExecutionContext::new();
    match tool.execute_impl(invalid_input, context).await {
        Ok(_) => println!("   ❌ Should have failed"),
        Err(e) => {
            println!("   ✅ Correctly caught error: Extension not found");
            // Test error handling
            match tool.handle_error(e).await {
                Ok(error_output) => {
                    if let Ok(parsed) =
                        serde_json::from_str::<serde_json::Value>(&error_output.text)
                    {
                        if let Some(available) = parsed.get("available_extensions") {
                            println!("   Available extensions: {available}");
                        }
                    }
                }
                Err(e2) => println!("   ❌ Error handling failed: {e2}"),
            }
        }
    }
    Ok(())
}

/// Print summary of demonstrated patterns
fn print_summary() {
    println!("\n✅ Extension Pattern Demo Complete!");
    println!("\n💡 Key Extension Patterns Demonstrated:");
    println!("   - Extension trait definition with async methods");
    println!("   - Plugin registry with registration and discovery");
    println!("   - Extensible tools using registered extensions");
    println!("   - Runtime extension execution and error handling");
    println!("   - Multiple extension types (text, math processors)");
    println!("   - Extension metadata and capability discovery");

    println!("\n📚 Extension Architecture Benefits:");
    println!("   - Modularity: Add functionality without changing core code");
    println!("   - Flexibility: Support different operation types dynamically");
    println!("   - Scalability: Register unlimited extensions at runtime");
    println!("   - Maintainability: Separate concerns into focused extensions");
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    println!("=== LLMSpell Extension Pattern Demo ===\n");

    // Step 1: Create extension registry
    println!("1. Creating extension registry...");
    let mut registry = ExtensionRegistry::new();
    println!("   ✅ Extension registry created");

    // Step 2: Set up extensions
    setup_extensions(&mut registry).await?;

    // Step 3: Create extensible tool
    println!("\n3. Creating extensible tool...");
    let registry_arc = Arc::new(registry);
    let tool = ExtensibleTool::new("extensible_processor".to_string(), registry_arc.clone());
    println!(
        "   ✅ Created extensible tool with {} extensions available",
        registry_arc.list().len()
    );

    // Run tests
    test_text_extension(&tool).await?;
    test_math_extension(&tool).await?;
    demonstrate_extension_discovery(&registry_arc);
    test_error_handling(&tool).await?;

    print_summary();

    Ok(())
}
