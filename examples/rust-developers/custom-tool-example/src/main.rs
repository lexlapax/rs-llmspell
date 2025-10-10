//! ============================================================
//! LLMSPELL RUST DEVELOPERS SHOWCASE
//! ============================================================
//! Example ID: 01 - Custom Tool v1.0.0
//! Complexity Level: BEGINNER
//! Real-World Use Case: Creating domain-specific tools for text processing
//!
//! Purpose: Demonstrates how to create custom tools using BaseAgent + Tool traits
//! Architecture: BaseAgent for execution, Tool for schema and categorization
//! Crates Showcased: llmspell-core (BaseAgent, Tool, ComponentMetadata)
//! Key Features:
//!   • BaseAgent implementation with execute_impl
//!   • Tool trait with schema, category, security level
//!   • Parameter validation and error handling
//!   • JSON input/output processing
//!
//! Prerequisites:
//!   • Rust 1.70+, tokio async runtime
//!
//! HOW TO RUN:
//! ```bash
//! cd examples/rust-developers/getting-started/custom-tool-example
//! cargo build
//! cargo run
//! ```
//!
//! EXPECTED OUTPUT:
//! Tool registration, parameter validation, execution demos
//!
//! Time to Complete: <3 seconds compilation + execution
//! ============================================================

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::tool::{ParameterDef, ParameterType, SecurityLevel, ToolCategory, ToolSchema},
    types::{AgentInput, AgentOutput},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Tool,
};
use serde_json::{json, Value};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Step 1: Define a custom tool - Text Analyzer
#[derive(Debug)]
struct TextAnalyzerTool {
    metadata: ComponentMetadata,
}

impl TextAnalyzerTool {
    fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "text-analyzer".to_string(),
                "Analyzes text for word count, character count, and sentiment".to_string(),
            ),
        }
    }

    fn analyze_text(&self, text: &str, operation: &str) -> Value {
        match operation {
            "word_count" => {
                let count = text.split_whitespace().count();
                json!({
                    "operation": "word_count",
                    "result": count,
                    "text_sample": text.chars().take(50).collect::<String>()
                })
            }
            "char_count" => {
                let count = text.chars().count();
                json!({
                    "operation": "char_count",
                    "result": count,
                    "text_sample": text.chars().take(50).collect::<String>()
                })
            }
            "line_count" => {
                let count = text.lines().count();
                json!({
                    "operation": "line_count",
                    "result": count,
                    "text_sample": text.chars().take(50).collect::<String>()
                })
            }
            "sentiment" => {
                // Simple sentiment analysis
                let positive_words = [
                    "good",
                    "great",
                    "excellent",
                    "amazing",
                    "wonderful",
                    "fantastic",
                ];
                let negative_words = ["bad", "terrible", "awful", "horrible", "disappointing"];

                let text_lower = text.to_lowercase();
                let pos_count = positive_words
                    .iter()
                    .filter(|word| text_lower.contains(*word))
                    .count();
                let neg_count = negative_words
                    .iter()
                    .filter(|word| text_lower.contains(*word))
                    .count();

                let sentiment = if pos_count > neg_count {
                    "positive"
                } else if neg_count > pos_count {
                    "negative"
                } else {
                    "neutral"
                };

                json!({
                    "operation": "sentiment",
                    "result": sentiment,
                    "positive_signals": pos_count,
                    "negative_signals": neg_count,
                    "text_sample": text.chars().take(50).collect::<String>()
                })
            }
            _ => {
                json!({
                    "error": format!("Unknown operation: {}", operation),
                    "available_operations": ["word_count", "char_count", "line_count", "sentiment"]
                })
            }
        }
    }
}

#[async_trait]
impl BaseAgent for TextAnalyzerTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Extract parameters from input
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing 'parameters' in input".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
            LLMSpellError::Validation {
                message: "Missing or invalid 'text' parameter".to_string(),
                field: Some("text".to_string()),
            }
        })?;

        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("word_count");

        info!("Analyzing text with operation: {}", operation);

        // Perform analysis
        let result = self.analyze_text(text, operation);

        Ok(AgentOutput::text(result.to_string()))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if let Some(params) = input.parameters.get("parameters") {
            if params
                .get("text")
                .and_then(|v| v.as_str())
                .is_none_or(|s| s.is_empty())
            {
                return Err(LLMSpellError::Validation {
                    message: "Text parameter cannot be empty".to_string(),
                    field: Some("text".to_string()),
                });
            }
        } else {
            return Err(LLMSpellError::Validation {
                message: "Missing parameters object".to_string(),
                field: Some("parameters".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Err(error)
    }
}

#[async_trait]
impl Tool for TextAnalyzerTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Analysis
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "text-analyzer".to_string(),
            "Analyzes text for various metrics".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "text".to_string(),
            param_type: ParameterType::String,
            description: "The text to analyze".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Analysis operation: word_count, char_count, line_count, sentiment"
                .to_string(),
            required: false,
            default: Some(json!("word_count")),
        })
        .with_returns(ParameterType::Object)
    }
}

// Step 2: Define another custom tool - Math Calculator
#[derive(Debug)]
struct MathCalculatorTool {
    metadata: ComponentMetadata,
}

impl MathCalculatorTool {
    fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "math-calculator".to_string(),
                "Performs basic mathematical operations".to_string(),
            ),
        }
    }

    fn calculate(&self, operation: &str, a: f64, b: Option<f64>) -> Value {
        match operation {
            "add" => {
                if let Some(b) = b {
                    json!({
                        "operation": "add",
                        "result": a + b,
                        "expression": format!("{} + {} = {}", a, b, a + b)
                    })
                } else {
                    json!({"error": "Addition requires two numbers"})
                }
            }
            "subtract" => {
                if let Some(b) = b {
                    json!({
                        "operation": "subtract",
                        "result": a - b,
                        "expression": format!("{} - {} = {}", a, b, a - b)
                    })
                } else {
                    json!({"error": "Subtraction requires two numbers"})
                }
            }
            "multiply" => {
                if let Some(b) = b {
                    json!({
                        "operation": "multiply",
                        "result": a * b,
                        "expression": format!("{} × {} = {}", a, b, a * b)
                    })
                } else {
                    json!({"error": "Multiplication requires two numbers"})
                }
            }
            "divide" => {
                if let Some(b) = b {
                    if b == 0.0 {
                        json!({"error": "Cannot divide by zero"})
                    } else {
                        json!({
                            "operation": "divide",
                            "result": a / b,
                            "expression": format!("{} ÷ {} = {}", a, b, a / b)
                        })
                    }
                } else {
                    json!({"error": "Division requires two numbers"})
                }
            }
            "sqrt" => {
                if a < 0.0 {
                    json!({"error": "Cannot take square root of negative number"})
                } else {
                    json!({
                        "operation": "sqrt",
                        "result": a.sqrt(),
                        "expression": format!("√{} = {}", a, a.sqrt())
                    })
                }
            }
            _ => {
                json!({
                    "error": format!("Unknown operation: {}", operation),
                    "available_operations": ["add", "subtract", "multiply", "divide", "sqrt"]
                })
            }
        }
    }
}

#[async_trait]
impl BaseAgent for MathCalculatorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing 'parameters' in input".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing or invalid 'operation' parameter".to_string(),
                field: Some("operation".to_string()),
            })?;

        let a =
            params
                .get("a")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing or invalid 'a' parameter".to_string(),
                    field: Some("a".to_string()),
                })?;

        let b = params.get("b").and_then(|v| v.as_f64());

        info!(
            "Performing calculation: {} with a={}, b={:?}",
            operation, a, b
        );

        let result = self.calculate(operation, a, b);

        Ok(AgentOutput::text(result.to_string()))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if let Some(params) = input.parameters.get("parameters") {
            if params
                .get("operation")
                .and_then(|v| v.as_str())
                .is_none_or(|s| s.is_empty())
            {
                return Err(LLMSpellError::Validation {
                    message: "Operation parameter cannot be empty".to_string(),
                    field: Some("operation".to_string()),
                });
            }
            if params.get("a").and_then(|v| v.as_f64()).is_none() {
                return Err(LLMSpellError::Validation {
                    message: "Parameter 'a' must be a valid number".to_string(),
                    field: Some("a".to_string()),
                });
            }
        } else {
            return Err(LLMSpellError::Validation {
                message: "Missing parameters object".to_string(),
                field: Some("parameters".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Err(error)
    }
}

#[async_trait]
impl Tool for MathCalculatorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "math-calculator".to_string(),
            "Performs basic mathematical operations".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Math operation: add, subtract, multiply, divide, sqrt".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "a".to_string(),
            param_type: ParameterType::Number,
            description: "First number".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "b".to_string(),
            param_type: ParameterType::Number,
            description: "Second number (not needed for sqrt)".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== Custom Tool Creation Demo ===\n");

    // Step 1: Create custom tools
    println!("1. Creating custom tools...");
    let text_analyzer = TextAnalyzerTool::new();
    let math_calculator = MathCalculatorTool::new();

    println!(
        "   ✅ Created TextAnalyzerTool: {}",
        text_analyzer.metadata().name
    );
    println!(
        "   ✅ Created MathCalculatorTool: {}",
        math_calculator.metadata().name
    );
    println!(
        "   📊 TextAnalyzer - Category: {:?}, Security: {:?}",
        text_analyzer.category(),
        text_analyzer.security_level()
    );
    println!(
        "   🔢 MathCalculator - Category: {:?}, Security: {:?}",
        math_calculator.category(),
        math_calculator.security_level()
    );

    // Step 2: Test parameter validation
    println!("\n2. Testing parameter validation...");

    // Test invalid input
    let invalid_input = AgentInput::text("test");
    // No parameters added - should fail validation

    match text_analyzer.validate_input(&invalid_input).await {
        Ok(_) => println!("   ❌ Validation should have failed"),
        Err(e) => println!("   ✅ Validation correctly failed: {}", e),
    }

    // Step 3: Test text analyzer tool
    println!("\n3. Testing TextAnalyzerTool...");

    let text_input = AgentInput::text("analyze")
        .with_parameter("parameters", json!({
            "text": "This is a wonderful example of great text analysis. The results should be amazing!",
            "operation": "sentiment"
        }));

    let context = ExecutionContext::default();

    match text_analyzer.execute_impl(text_input, context).await {
        Ok(output) => {
            println!("   ✅ Text analysis result:");
            println!("   {}", output.text);
        }
        Err(e) => println!("   ❌ Text analysis failed: {}", e),
    }

    // Test word count
    let word_count_input = AgentInput::text("analyze").with_parameter(
        "parameters",
        json!({
            "text": "Count the words in this sentence please.",
            "operation": "word_count"
        }),
    );

    match text_analyzer
        .execute_impl(word_count_input, ExecutionContext::default())
        .await
    {
        Ok(output) => {
            println!("   ✅ Word count result:");
            println!("   {}", output.text);
        }
        Err(e) => println!("   ❌ Word count failed: {}", e),
    }

    // Step 4: Test math calculator tool
    println!("\n4. Testing MathCalculatorTool...");

    let math_tests = vec![
        ("add", 10.0, Some(5.0)),
        ("multiply", 7.0, Some(8.0)),
        ("divide", 100.0, Some(4.0)),
        ("sqrt", 16.0, None),
        ("divide", 10.0, Some(0.0)), // Test error case
    ];

    for (operation, a, b) in math_tests {
        let mut params = json!({
            "operation": operation,
            "a": a
        });

        if let Some(b_val) = b {
            params["b"] = json!(b_val);
        }

        let math_input = AgentInput::text("calculate").with_parameter("parameters", params);

        match math_calculator
            .execute_impl(math_input, ExecutionContext::default())
            .await
        {
            Ok(output) => {
                println!("   ✅ Math operation '{}': {}", operation, output.text);
            }
            Err(e) => println!("   ❌ Math operation '{}' failed: {}", operation, e),
        }
    }

    // Step 5: Display schemas
    println!("\n5. Tool schemas...");
    println!("   📄 TextAnalyzer schema: {}", text_analyzer.schema().name);
    println!(
        "   📄 MathCalculator schema: {}",
        math_calculator.schema().name
    );

    println!("\n✅ Custom Tool Demo Complete!");
    println!("\n💡 Key Concepts Demonstrated:");
    println!("   - BaseAgent trait implementation with execute_impl");
    println!("   - Tool trait with category, security_level, schema");
    println!("   - Parameter validation and error handling");
    println!("   - AgentInput/AgentOutput processing");
    println!("   - JSON parameter extraction and validation");

    println!("\n📚 Next Steps:");
    println!("   - Integrate tools with LLM agents");
    println!("   - Add tools to registry for script access");
    println!("   - Implement more complex validation logic");
    println!("   - Add resource limits and security checks");

    Ok(())
}
