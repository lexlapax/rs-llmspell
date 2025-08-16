// ABOUTME: Example demonstrating how to create custom tools for LLMSpell
// ABOUTME: Shows tool implementation, registration, and usage from Lua scripts

use anyhow::{Context, Result};
use async_trait::async_trait;
use llmspell_bridge::{lua::LuaEngine, ScriptEngine};
use llmspell_core::{Tool, ToolInput, ToolOutput, ToolRegistry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Step 1: Define a custom tool - Word Counter
#[derive(Debug, Clone)]
struct WordCounterTool;

#[async_trait]
impl Tool for WordCounterTool {
    fn name(&self) -> &str {
        "word_counter"
    }

    fn description(&self) -> &str {
        "Counts words, characters, and lines in text"
    }

    fn parameters(&self) -> Vec<(&str, &str)> {
        vec![
            ("input", "The text to analyze"),
            ("operation", "count_words, count_chars, count_lines, or full_stats"),
        ]
    }

    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        // Extract parameters
        let text = input
            .get("input")
            .and_then(|v| v.as_str())
            .context("Missing 'input' parameter")?;

        let operation = input
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("full_stats");

        // Perform the requested operation
        let result = match operation {
            "count_words" => {
                let word_count = text.split_whitespace().count();
                json!({
                    "operation": "count_words",
                    "count": word_count,
                    "success": true
                })
            }
            "count_chars" => {
                let char_count = text.chars().count();
                json!({
                    "operation": "count_chars",
                    "count": char_count,
                    "success": true
                })
            }
            "count_lines" => {
                let line_count = text.lines().count();
                json!({
                    "operation": "count_lines",
                    "count": line_count,
                    "success": true
                })
            }
            "full_stats" | _ => {
                let word_count = text.split_whitespace().count();
                let char_count = text.chars().count();
                let line_count = text.lines().count();
                let char_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();

                json!({
                    "operation": "full_stats",
                    "stats": {
                        "words": word_count,
                        "characters": char_count,
                        "characters_no_spaces": char_no_spaces,
                        "lines": line_count,
                        "average_word_length": if word_count > 0 {
                            char_no_spaces as f64 / word_count as f64
                        } else {
                            0.0
                        }
                    },
                    "success": true
                })
            }
        };

        Ok(ToolOutput::from_json(result))
    }
}

// Step 2: Define another custom tool - Math Calculator
#[derive(Debug, Clone)]
struct MathCalculatorTool;

#[async_trait]
impl Tool for MathCalculatorTool {
    fn name(&self) -> &str {
        "math_calculator"
    }

    fn description(&self) -> &str {
        "Performs basic math operations"
    }

    fn parameters(&self) -> Vec<(&str, &str)> {
        vec![
            ("operation", "add, subtract, multiply, divide, power, sqrt"),
            ("a", "First number"),
            ("b", "Second number (not needed for sqrt)"),
        ]
    }

    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        let operation = input
            .get("operation")
            .and_then(|v| v.as_str())
            .context("Missing 'operation' parameter")?;

        let a = input
            .get("a")
            .and_then(|v| v.as_f64())
            .context("Missing or invalid 'a' parameter")?;

        let result = match operation {
            "add" => {
                let b = input
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .context("Missing 'b' parameter for addition")?;
                json!({
                    "operation": "add",
                    "result": a + b,
                    "expression": format!("{} + {} = {}", a, b, a + b),
                    "success": true
                })
            }
            "subtract" => {
                let b = input
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .context("Missing 'b' parameter for subtraction")?;
                json!({
                    "operation": "subtract",
                    "result": a - b,
                    "expression": format!("{} - {} = {}", a, b, a - b),
                    "success": true
                })
            }
            "multiply" => {
                let b = input
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .context("Missing 'b' parameter for multiplication")?;
                json!({
                    "operation": "multiply",
                    "result": a * b,
                    "expression": format!("{} × {} = {}", a, b, a * b),
                    "success": true
                })
            }
            "divide" => {
                let b = input
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .context("Missing 'b' parameter for division")?;
                
                if b == 0.0 {
                    json!({
                        "operation": "divide",
                        "error": "Division by zero",
                        "success": false
                    })
                } else {
                    json!({
                        "operation": "divide",
                        "result": a / b,
                        "expression": format!("{} ÷ {} = {}", a, b, a / b),
                        "success": true
                    })
                }
            }
            "power" => {
                let b = input
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .context("Missing 'b' parameter for power")?;
                json!({
                    "operation": "power",
                    "result": a.powf(b),
                    "expression": format!("{}^{} = {}", a, b, a.powf(b)),
                    "success": true
                })
            }
            "sqrt" => {
                if a < 0.0 {
                    json!({
                        "operation": "sqrt",
                        "error": "Cannot take square root of negative number",
                        "success": false
                    })
                } else {
                    json!({
                        "operation": "sqrt",
                        "result": a.sqrt(),
                        "expression": format!("√{} = {}", a, a.sqrt()),
                        "success": true
                    })
                }
            }
            _ => json!({
                "error": format!("Unknown operation: {}", operation),
                "success": false
            }),
        };

        Ok(ToolOutput::from_json(result))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== Creating Custom Tools for LLMSpell ===\n");

    // Step 3: Create and register custom tools
    println!("1. Creating custom tools...");
    let word_counter = Arc::new(WordCounterTool) as Arc<dyn Tool>;
    let math_calculator = Arc::new(MathCalculatorTool) as Arc<dyn Tool>;
    println!("   ✅ Created WordCounterTool");
    println!("   ✅ Created MathCalculatorTool");

    // Step 4: Register tools with the registry
    println!("\n2. Registering tools...");
    let registry = ToolRegistry::new();
    registry.register(word_counter.clone()).await?;
    registry.register(math_calculator.clone()).await?;
    
    // Also register standard tools for comparison
    llmspell_tools::register_all_tools(&registry).await?;
    
    let tool_count = registry.list_tools().await.len();
    println!("   ✅ Registered {} tools total (2 custom + standard tools)", tool_count);

    // Step 5: Create a Lua engine with our custom tools
    println!("\n3. Creating Lua engine with custom tools...");
    let mut engine = LuaEngine::new()?;
    engine.initialize().await?;
    println!("   ✅ Engine initialized");

    // Step 6: Test custom tools from Lua
    println!("\n4. Testing custom tools from Lua...");
    
    let test_script = r#"
        print("\n   Testing WordCounterTool:")
        print("   " .. string.rep("-", 40))
        
        local text = "Hello world! This is a test.\nIt has two lines."
        
        -- Test full stats
        local result = Tool.invoke("word_counter", {
            input = text,
            operation = "full_stats"
        })
        
        if result and result.stats then
            print("   Text: '" .. text .. "'")
            print("   Words: " .. tostring(result.stats.words))
            print("   Characters: " .. tostring(result.stats.characters))
            print("   Lines: " .. tostring(result.stats.lines))
            print("   Avg word length: " .. string.format("%.2f", result.stats.average_word_length))
        end
        
        print("\n   Testing MathCalculatorTool:")
        print("   " .. string.rep("-", 40))
        
        -- Test various operations
        local operations = {
            {op = "add", a = 10, b = 5},
            {op = "multiply", a = 7, b = 8},
            {op = "divide", a = 100, b = 4},
            {op = "sqrt", a = 16},
            {op = "power", a = 2, b = 8},
        }
        
        for _, test in ipairs(operations) do
            local calc_result = Tool.invoke("math_calculator", test)
            if calc_result and calc_result.expression then
                print("   " .. calc_result.expression)
            end
        end
        
        -- Test error handling
        print("\n   Testing error handling:")
        local div_zero = Tool.invoke("math_calculator", {
            operation = "divide",
            a = 10,
            b = 0
        })
        if div_zero and div_zero.error then
            print("   ✅ Caught error: " .. div_zero.error)
        end
        
        return "Custom tools tested successfully"
    "#;

    let result = engine.execute(test_script).await?;
    if let Some(output) = result {
        println!("\n   Script result: {}", output);
    }

    // Step 7: Direct tool invocation from Rust
    println!("\n5. Direct tool invocation from Rust...");
    
    let input = json!({
        "input": "The quick brown fox jumps over the lazy dog.",
        "operation": "count_words"
    });
    
    let word_result = word_counter.invoke(input).await?;
    println!("   Word count result: {}", word_result.to_json());

    // Step 8: Show tool discovery
    println!("\n6. Tool discovery...");
    let all_tools = registry.list_tools().await;
    println!("   Custom tools registered:");
    for tool in &all_tools {
        if tool == "word_counter" || tool == "math_calculator" {
            println!("   - {}", tool);
        }
    }

    println!("\n✅ Successfully created and registered custom tools!");
    println!("\n💡 Key Concepts:");
    println!("   - Implement the Tool trait for custom functionality");
    println!("   - Register tools with ToolRegistry");
    println!("   - Tools are automatically available in scripts");
    println!("   - Handle errors gracefully in tool implementations");
    println!("   - Tools can be invoked from both Rust and scripts");
    
    println!("\n📚 Next Steps:");
    println!("   - Create tools that integrate with your systems");
    println!("   - Add async operations in tool implementations");
    println!("   - Implement resource limits and security checks");
    println!("   - Create tool wrappers for existing libraries");

    Ok(())
}