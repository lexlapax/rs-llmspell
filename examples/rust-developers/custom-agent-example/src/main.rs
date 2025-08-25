//! ============================================================
//! LLMSPELL RUST DEVELOPERS SHOWCASE
//! ============================================================
//! Example ID: 02 - Custom Agent v1.0.0
//! Complexity Level: INTERMEDIATE
//! Real-World Use Case: Building domain-specific agents with different personalities and capabilities
//! 
//! Purpose: Demonstrates custom BaseAgent implementation with different behaviors
//! Architecture: BaseAgent for execution foundation, custom logic for specialized responses
//! Crates Showcased: llmspell-core (BaseAgent, ComponentMetadata, AgentInput/Output)
//! Key Features:
//!   • Custom BaseAgent implementations with different personalities
//!   • Parameter processing and JSON response formatting
//!   • Error handling and input validation
//!   • Tool integration patterns
//!
//! Prerequisites:
//!   • Rust 1.70+, tokio async runtime
//!
//! HOW TO RUN:
//! ```bash
//! cd examples/rust-developers/getting-started/custom-agent-example
//! cargo build
//! cargo run
//! ```
//!
//! EXPECTED OUTPUT:
//! Agent creation, personality-based responses, tool integration demos
//!
//! Time to Complete: <5 seconds compilation + execution
//! ============================================================

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    ComponentMetadata, ExecutionContext, LLMSpellError, BaseAgent, 
    types::{AgentInput, AgentOutput}
};
use serde_json::json;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Step 1: Define a custom agent that answers questions with a specific personality
#[derive(Debug, Clone)]
struct PersonalityAgent {
    metadata: ComponentMetadata,
    personality: String,
    enthusiasm_level: u8, // 1-10
}

impl PersonalityAgent {
    fn new(name: String, personality: String, enthusiasm_level: u8) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                format!("{}_agent", name.to_lowercase().replace(" ", "_")),
                format!("Personality agent with {} persona", personality),
            ),
            personality,
            enthusiasm_level: enthusiasm_level.min(10).max(1),
        }
    }

    fn format_response(&self, base_response: &str) -> String {
        // Add personality flair to responses
        let exclamation_marks = "!".repeat(self.enthusiasm_level as usize / 3);
        let prefix = match self.personality.as_str() {
            "pirate" => format!("Arrr, matey{} ", exclamation_marks),
            "robot" => "BEEP BOOP. PROCESSING... ".to_string(),
            "wizard" => format!("*waves wand mystically* "),
            "cowboy" => format!("Well, howdy partner{} ", exclamation_marks),
            _ => "".to_string(),
        };

        let suffix = match self.personality.as_str() {
            "pirate" => " Yo ho ho!",
            "robot" => " END TRANSMISSION.",
            "wizard" => " *sparkles appear*",
            "cowboy" => " Yeehaw!",
            _ => "",
        };

        format!("{}{}{}", prefix, base_response, suffix)
    }
}

#[async_trait]
impl BaseAgent for PersonalityAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Extract text from input - either from direct text or parameters
        let text = if !input.text.is_empty() {
            input.text.as_str()
        } else {
            input.parameters.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("Hello!")
        };

        info!("PersonalityAgent ({}) processing: {}", self.personality, text);

        // Simple response logic based on input
        let base_response = if text.to_lowercase().contains("hello") {
            "Greetings to you!"
        } else if text.to_lowercase().contains("how are you") {
            "I'm doing wonderfully, thank you for asking!"
        } else if text.to_lowercase().contains("help") {
            "I'm here to assist you with anything you need!"
        } else if text.to_lowercase().contains("joke") {
            "Why don't scientists trust atoms? Because they make up everything!"
        } else {
            "That's an interesting thought!"
        };

        let formatted_response = self.format_response(base_response);
        
        // Create structured output with metadata
        let mut output = AgentOutput::text(formatted_response);
        output.metadata.extra.insert("personality".to_string(), json!(self.personality));
        output.metadata.extra.insert("enthusiasm_level".to_string(), json!(self.enthusiasm_level));
        output.metadata.extra.insert("agent_type".to_string(), json!("personality"));
        
        Ok(output)
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        // Accept either text directly or text parameter
        let has_text = !input.text.is_empty() || 
            input.parameters.get("text")
                .and_then(|v| v.as_str())
                .map_or(false, |s| !s.is_empty());
                
        if !has_text {
            return Err(LLMSpellError::Validation {
                message: "Either input.text or parameters.text must be provided".to_string(),
                field: Some("text".to_string()),
            });
        }
        
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        // Provide a personality-appropriate error response
        let error_response = match self.personality.as_str() {
            "pirate" => format!("Arrr, something went wrong: {}", error),
            "robot" => format!("ERROR DETECTED. {}", error),
            "wizard" => format!("*spells fizzle* Magic failed: {}", error),
            "cowboy" => format!("Well partner, we hit a snag: {}", error),
            _ => format!("Sorry, I encountered an error: {}", error),
        };
        
        Ok(AgentOutput::text(error_response))
    }
}

// Step 2: Create a specialized agent for mathematical operations
#[derive(Debug, Clone)]
struct MathAgent {
    metadata: ComponentMetadata,
    precision: u8, // decimal places for results
}

impl MathAgent {
    fn new(precision: u8) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "math_agent".to_string(),
                "Specialized agent for mathematical operations".to_string(),
            ),
            precision: precision.min(10),
        }
    }
    
    fn perform_calculation(&self, operation: &str, a: f64, b: Option<f64>) -> Result<f64, String> {
        match operation.to_lowercase().as_str() {
            "add" => {
                if let Some(b) = b {
                    Ok(a + b)
                } else {
                    Err("Addition requires two numbers".to_string())
                }
            }
            "subtract" => {
                if let Some(b) = b {
                    Ok(a - b)
                } else {
                    Err("Subtraction requires two numbers".to_string())
                }
            }
            "multiply" => {
                if let Some(b) = b {
                    Ok(a * b)
                } else {
                    Err("Multiplication requires two numbers".to_string())
                }
            }
            "divide" => {
                if let Some(b) = b {
                    if b == 0.0 {
                        Err("Cannot divide by zero".to_string())
                    } else {
                        Ok(a / b)
                    }
                } else {
                    Err("Division requires two numbers".to_string())
                }
            }
            "square" => Ok(a * a),
            "sqrt" => {
                if a < 0.0 {
                    Err("Cannot take square root of negative number".to_string())
                } else {
                    Ok(a.sqrt())
                }
            }
            "abs" => Ok(a.abs()),
            _ => Err(format!("Unknown operation: {}", operation)),
        }
    }
}

#[async_trait]
impl BaseAgent for MathAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Parse parameters from input
        let operation = input.parameters.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'operation' parameter".to_string(),
                field: Some("operation".to_string()),
            })?;
            
        let a = input.parameters.get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing or invalid 'a' parameter".to_string(),
                field: Some("a".to_string()),
            })?;
            
        let b = input.parameters.get("b").and_then(|v| v.as_f64());
        
        info!("MathAgent performing {} with a={}, b={:?}", operation, a, b);
        
        // Perform calculation
        match self.perform_calculation(operation, a, b) {
            Ok(result) => {
                // Format result with specified precision
                let formatted_result = format!("{:.precision$}", result, precision = self.precision as usize);
                
                let response_text = match operation {
                    "add" => format!("{} + {} = {}", a, b.unwrap_or(0.0), formatted_result),
                    "subtract" => format!("{} - {} = {}", a, b.unwrap_or(0.0), formatted_result),
                    "multiply" => format!("{} × {} = {}", a, b.unwrap_or(0.0), formatted_result),
                    "divide" => format!("{} ÷ {} = {}", a, b.unwrap_or(0.0), formatted_result),
                    "square" => format!("{}² = {}", a, formatted_result),
                    "sqrt" => format!("√{} = {}", a, formatted_result),
                    "abs" => format!("|{}| = {}", a, formatted_result),
                    _ => format!("{}({}) = {}", operation, a, formatted_result),
                };
                
                let mut output = AgentOutput::text(response_text);
                output.metadata.extra.insert("operation".to_string(), json!(operation));
                output.metadata.extra.insert("result".to_string(), json!(result));
                output.metadata.extra.insert("precision".to_string(), json!(self.precision));
                
                Ok(output)
            }
            Err(error_msg) => {
                Err(LLMSpellError::Component {
                    message: error_msg,
                    source: None,
                })
            }
        }
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        // Check required parameters
        if !input.parameters.contains_key("operation") {
            return Err(LLMSpellError::Validation {
                message: "Missing 'operation' parameter".to_string(),
                field: Some("operation".to_string()),
            });
        }
        
        if !input.parameters.contains_key("a") {
            return Err(LLMSpellError::Validation {
                message: "Missing 'a' parameter".to_string(),
                field: Some("a".to_string()),
            });
        }
        
        // Validate operation type
        let operation = input.parameters.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let valid_operations = ["add", "subtract", "multiply", "divide", "square", "sqrt", "abs"];
        if !valid_operations.contains(&operation) {
            return Err(LLMSpellError::Validation {
                message: format!("Invalid operation '{}'. Valid operations: {:?}", operation, valid_operations),
                field: Some("operation".to_string()),
            });
        }
        
        // Check if two-operand operations have 'b' parameter
        let two_operand_ops = ["add", "subtract", "multiply", "divide"];
        if two_operand_ops.contains(&operation) && !input.parameters.contains_key("b") {
            return Err(LLMSpellError::Validation {
                message: format!("Operation '{}' requires 'b' parameter", operation),
                field: Some("b".to_string()),
            });
        }
        
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        let error_message = format!("Mathematical calculation failed: {}", error);
        Ok(AgentOutput::text(error_message))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== Creating Custom Agents ===\n");

    // Step 3: Create personality agents
    println!("1. Creating personality agents...");
    
    let pirate_agent = PersonalityAgent::new(
        "Captain Blackbeard".to_string(),
        "pirate".to_string(),
        8,
    );

    let robot_agent = PersonalityAgent::new(
        "Unit-7734".to_string(),
        "robot".to_string(),
        3,
    );

    let wizard_agent = PersonalityAgent::new(
        "Merlin".to_string(),
        "wizard".to_string(),
        6,
    );

    println!("   ✅ Created pirate agent: {}", pirate_agent.metadata().name);
    println!("   ✅ Created robot agent: {}", robot_agent.metadata().name);
    println!("   ✅ Created wizard agent: {}", wizard_agent.metadata().name);

    // Step 4: Test personality agents
    println!("\n2. Testing personality agents...");
    
    let test_messages = vec![
        "Hello there!",
        "How are you today?",
        "Can you help me?",
        "Tell me a joke!",
    ];

    let agents = vec![&pirate_agent, &robot_agent, &wizard_agent];
    
    for agent in &agents {
        println!("\n   Agent: {}", agent.metadata().name);
        println!("   {}", "-".repeat(40));
        
        for msg in &test_messages {
            let input = AgentInput::text(*msg);
            let context = ExecutionContext::new();
            
            match agent.execute_impl(input, context).await {
                Ok(response) => {
                    println!("   You: {}", msg);
                    println!("   {}: {}", agent.metadata().name, response.text);
                }
                Err(e) => {
                    println!("   You: {}", msg);
                    println!("   Error: {}", e);
                }
            }
        }
    }

    // Step 5: Create specialized math agent
    println!("\n3. Creating specialized math agent...");
    
    let math_agent = MathAgent::new(3); // 3 decimal places precision
    
    println!("   ✅ Created math agent: {}", math_agent.metadata().name);
    println!("   ✅ Math agent precision: {} decimal places", math_agent.precision);

    // Step 6: Test math agent
    println!("\n4. Testing math agent...");
    
    let math_tests = vec![
        ("add", 15.5, Some(7.2)),
        ("multiply", 3.14159, Some(2.0)),
        ("divide", 100.0, Some(3.0)),
        ("sqrt", 16.0, None),
        ("square", 5.5, None),
        ("abs", -42.7, None),
    ];

    println!("   Agent: {}", math_agent.metadata().name);
    println!("   {}", "-".repeat(50));
    
    for (operation, a, b) in &math_tests {
        let mut input = AgentInput::text("calculate")
            .with_parameter("operation", json!(operation))
            .with_parameter("a", json!(a));
            
        if let Some(b_val) = b {
            input = input.with_parameter("b", json!(b_val));
        }
        
        let context = ExecutionContext::new();
        
        match math_agent.execute_impl(input, context).await {
            Ok(response) => {
                println!("   Operation: {}", response.text);
                if let Some(result) = response.metadata.extra.get("result") {
                    println!("   [Result: {}]", result);
                }
            }
            Err(e) => {
                println!("   Operation {} failed: {}", operation, e);
            }
        }
    }
    
    // Test validation
    println!("\n   Testing validation...");
    let invalid_input = AgentInput::text("test");
    match math_agent.validate_input(&invalid_input).await {
        Ok(_) => println!("   ❌ Validation should have failed"),
        Err(e) => println!("   ✅ Validation correctly failed: {}", e),
    }

    // Step 7: Demonstrate error handling
    println!("\n5. Testing error handling...");
    
    // Test divide by zero
    let error_input = AgentInput::text("calculate")
        .with_parameter("operation", json!("divide"))
        .with_parameter("a", json!(10.0))
        .with_parameter("b", json!(0.0));
        
    let context = ExecutionContext::new();
    match math_agent.execute_impl(error_input, context).await {
        Ok(response) => println!("   ❌ Should have failed: {}", response.text),
        Err(e) => {
            println!("   ✅ Correctly caught error: {}", e);
            
            // Test error handling
            match math_agent.handle_error(e).await {
                Ok(error_response) => {
                    println!("   ✅ Error handled gracefully: {}", error_response.text);
                }
                Err(e2) => println!("   ❌ Error handling failed: {}", e2),
            }
        }
    }
    
    // Step 8: Advanced agent patterns  
    println!("\n6. Advanced agent patterns demonstrated:");
    println!("   ✅ Custom personality agents with different response styles");
    println!("   ✅ Specialized math agent with parameter validation");
    println!("   ✅ Structured input/output with metadata");
    println!("   ✅ Comprehensive error handling and validation");
    println!("   ✅ BaseAgent trait implementation patterns");

    println!("\n✅ Successfully created and tested custom agents!");
    println!("\n💡 Key Concepts Demonstrated:");
    println!("   - BaseAgent trait implementation with execute_impl, validate_input, handle_error");
    println!("   - ComponentMetadata for agent identification and description");
    println!("   - AgentInput/AgentOutput for structured communication");
    println!("   - Parameter extraction and validation patterns");
    println!("   - Error handling with graceful degradation");
    println!("   - Custom agent personalities and specialized behaviors");
    println!("   - Metadata enrichment in agent outputs");
    
    println!("\n📚 Next Steps:");
    println!("   - Add streaming support with stream_execute()");
    println!("   - Implement multimodal content processing");
    println!("   - Add state persistence across executions");
    println!("   - Integrate with external AI services");
    println!("   - Build agent composition and chaining patterns");

    Ok(())
}