//! ============================================================
//! LLMSPELL RUST DEVELOPERS SHOWCASE
//! ============================================================
//! Example ID: 03 - Async Patterns v1.0.0
//! Complexity Level: ADVANCED
//! Real-World Use Case: Concurrent tool execution, streaming operations, timeouts and async composition
//!
//! Purpose: Demonstrates async patterns with LLMSpell tools and agents
//! Architecture: BaseAgent + Tool traits with async execution patterns
//! Crates Showcased: llmspell-core (BaseAgent, Tool), tokio, futures
//! Key Features:
//!   • Concurrent tool execution with tokio::join!
//!   • Streaming operations with futures::stream
//!   • Timeout handling with tokio::time
//!   • Select patterns for racing operations
//!   • Async pipeline construction
//!
//! Prerequisites:
//!   • Rust 1.70+, tokio async runtime, futures crate
//!
//! HOW TO RUN:
//! ```bash
//! cd examples/rust-developers/getting-started/async-patterns-example
//! cargo build
//! cargo run
//! ```
//!
//! EXPECTED OUTPUT:
//! Demonstrations of concurrent execution, streaming, timeouts, and pipelines
//!
//! Time to Complete: <10 seconds compilation + execution
//! ============================================================

use anyhow::Result;
use async_trait::async_trait;
use futures::{future, stream, StreamExt};
use llmspell_core::{
    traits::tool::{ParameterDef, ParameterType, SecurityLevel, ToolCategory, ToolSchema},
    types::{AgentInput, AgentOutput},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Tool,
};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::{join, select, time};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Example: Async tool that simulates a slow operation
#[derive(Debug, Clone)]
struct SlowTool {
    metadata: ComponentMetadata,
    delay_ms: u64,
}

impl SlowTool {
    fn new(name: String, delay_ms: u64) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.clone(),
                format!("Async tool that simulates {}ms delay", delay_ms),
            ),
            delay_ms,
        }
    }
}

#[async_trait]
impl BaseAgent for SlowTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Extract input data
        let data = input
            .parameters
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or(&input.text);

        info!(
            "SlowTool {} starting {}ms operation with data: {}",
            self.metadata.name, self.delay_ms, data
        );

        // Simulate async work
        let start = Instant::now();
        time::sleep(Duration::from_millis(self.delay_ms)).await;
        let actual_duration = start.elapsed();

        let response = json!({
            "tool": self.metadata.name,
            "processed": format!("Processed: {}", data),
            "expected_duration_ms": self.delay_ms,
            "actual_duration_ms": actual_duration.as_millis(),
            "success": true
        });

        Ok(AgentOutput::text(response.to_string()))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        // Accept any input for this demo tool
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        let error_response = json!({
            "tool": self.metadata.name,
            "error": error.to_string(),
            "success": false
        });

        Ok(AgentOutput::text(error_response.to_string()))
    }
}

#[async_trait]
impl Tool for SlowTool {
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
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input data to process".to_string(),
            required: false,
            default: Some(json!("default_data")),
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

    println!("=== Async Patterns with LLMSpell ===\n");

    // Run all async pattern examples
    concurrent_tools().await?;
    streaming_operations().await?;
    timeout_patterns().await?;
    select_patterns().await?;
    pipeline_pattern().await?;

    println!("\n✅ All async patterns demonstrated!");
    println!("\n💡 Key Async Concepts Demonstrated:");
    println!("   - tokio::join! for concurrent execution");
    println!("   - futures::stream for streaming operations");
    println!("   - tokio::time::timeout for operation timeouts");
    println!("   - tokio::select! for first-to-complete patterns");
    println!("   - Async pipelines for sequential processing");
    println!("   - BaseAgent + Tool trait implementations");

    Ok(())
}

// Pattern 1: Concurrent Tool Execution
async fn concurrent_tools() -> Result<()> {
    println!("1. Concurrent Tool Execution");
    println!("   {}", "-".repeat(50));

    // Create tools with different delays
    let fast_tool = SlowTool::new("fast_tool".to_string(), 100);
    let medium_tool = SlowTool::new("medium_tool".to_string(), 300);
    let slow_tool = SlowTool::new("slow_tool".to_string(), 500);

    let context = ExecutionContext::new();

    // Sequential execution (for comparison)
    println!("   Sequential execution:");
    let start = Instant::now();

    let input1 = AgentInput::text("data1").with_parameter("input", json!("data1"));
    let input2 = AgentInput::text("data2").with_parameter("input", json!("data2"));
    let input3 = AgentInput::text("data3").with_parameter("input", json!("data3"));

    let _r1 = fast_tool.execute_impl(input1, context.clone()).await?;
    let _r2 = medium_tool.execute_impl(input2, context.clone()).await?;
    let _r3 = slow_tool.execute_impl(input3, context.clone()).await?;

    let sequential_time = start.elapsed();
    println!("   ⏱️  Sequential time: {:?}", sequential_time);

    // Concurrent execution
    println!("\n   Concurrent execution:");
    let start = Instant::now();

    let input1 = AgentInput::text("data1").with_parameter("input", json!("data1"));
    let input2 = AgentInput::text("data2").with_parameter("input", json!("data2"));
    let input3 = AgentInput::text("data3").with_parameter("input", json!("data3"));

    let (r1, r2, r3) = join!(
        fast_tool.execute_impl(input1, context.clone()),
        medium_tool.execute_impl(input2, context.clone()),
        slow_tool.execute_impl(input3, context.clone())
    );

    // Check results
    let _ = r1?;
    let _ = r2?;
    let _ = r3?;

    let concurrent_time = start.elapsed();
    println!("   ⏱️  Concurrent time: {:?}", concurrent_time);
    println!(
        "   🚀 Speedup: {:.2}x",
        sequential_time.as_millis() as f64 / concurrent_time.as_millis() as f64
    );

    Ok(())
}

// Pattern 2: Streaming Operations
async fn streaming_operations() -> Result<()> {
    println!("\n2. Streaming Operations");
    println!("   {}", "-".repeat(50));

    // Create a stream of data to process
    let data_items = vec!["chunk1", "chunk2", "chunk3", "chunk4", "chunk5"];
    let data_stream = stream::iter(data_items);

    // Process stream items concurrently (up to 3 at a time)
    println!("   Processing stream with concurrency limit of 3:");

    let tool = SlowTool::new("stream_processor".to_string(), 200);
    let context = ExecutionContext::new();

    let results: Vec<Result<AgentOutput, LLMSpellError>> = data_stream
        .map(|chunk| {
            let tool = tool.clone();
            let context = context.clone();
            async move {
                println!("   Processing: {}", chunk);
                let input = AgentInput::text(chunk).with_parameter("input", json!(chunk));
                tool.execute_impl(input, context).await
            }
        })
        .buffer_unordered(3) // Process up to 3 concurrently
        .collect()
        .await;

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    println!(
        "   ✅ Successfully processed {}/{} chunks",
        success_count,
        results.len()
    );

    // Demonstrate async iteration with early termination
    println!("\n   Async iteration with early termination:");
    let mut count_stream = stream::iter(1..=10);

    while let Some(num) = count_stream.next().await {
        println!("   Processing number: {}", num);
        if num >= 5 {
            println!("   Early termination at {}", num);
            break;
        }
    }

    Ok(())
}

// Pattern 3: Timeout Patterns
async fn timeout_patterns() -> Result<()> {
    println!("\n3. Timeout Patterns");
    println!("   {}", "-".repeat(50));

    let slow_tool = SlowTool::new("very_slow_tool".to_string(), 2000);
    let context = ExecutionContext::new();

    // Attempt with timeout
    println!("   Attempting operation with 1-second timeout:");

    let input = AgentInput::text("timeout_test").with_parameter("input", json!("timeout_test"));

    let result = time::timeout(
        Duration::from_secs(1),
        slow_tool.execute_impl(input, context.clone()),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            println!(
                "   ✅ Operation completed: First 100 chars: {}",
                output.text.chars().take(100).collect::<String>()
            );
        }
        Ok(Err(e)) => {
            println!("   ❌ Operation failed: {}", e);
        }
        Err(_) => {
            println!("   ⏱️  Operation timed out after 1 second");
        }
    }

    // Retry with longer timeout
    println!("\n   Retrying with 3-second timeout:");

    let input = AgentInput::text("retry_test").with_parameter("input", json!("retry_test"));

    let result = time::timeout(
        Duration::from_secs(3),
        slow_tool.execute_impl(input, context),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            println!("   ✅ Operation completed successfully");
            println!("   Response length: {} characters", output.text.len());
        }
        Ok(Err(e)) => {
            println!("   ❌ Operation failed: {}", e);
        }
        Err(_) => {
            println!("   ⏱️  Operation timed out again");
        }
    }

    Ok(())
}

// Pattern 4: Select Patterns (First to Complete)
async fn select_patterns() -> Result<()> {
    println!("\n4. Select Patterns (Race Conditions)");
    println!("   {}", "-".repeat(50));

    // Create tools with different delays
    let tool1 = SlowTool::new("racer1".to_string(), 300);
    let tool2 = SlowTool::new("racer2".to_string(), 250);
    let tool3 = SlowTool::new("racer3".to_string(), 400);

    let context = ExecutionContext::new();

    println!("   Racing 3 tools - first to complete wins:");

    let input1 = AgentInput::text("race").with_parameter("input", json!("race"));
    let input2 = AgentInput::text("race").with_parameter("input", json!("race"));
    let input3 = AgentInput::text("race").with_parameter("input", json!("race"));

    select! {
        result1 = tool1.execute_impl(input1, context.clone()) => {
            match result1 {
                Ok(output) => {
                    println!("   🏆 Tool 1 won!");
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                        if let Some(tool_name) = parsed.get("tool") {
                            println!("   Winner: {}", tool_name);
                        }
                    }
                }
                Err(e) => println!("   ❌ Tool 1 failed: {}", e),
            }
        }
        result2 = tool2.execute_impl(input2, context.clone()) => {
            match result2 {
                Ok(output) => {
                    println!("   🏆 Tool 2 won!");
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                        if let Some(tool_name) = parsed.get("tool") {
                            println!("   Winner: {}", tool_name);
                        }
                    }
                }
                Err(e) => println!("   ❌ Tool 2 failed: {}", e),
            }
        }
        result3 = tool3.execute_impl(input3, context.clone()) => {
            match result3 {
                Ok(output) => {
                    println!("   🏆 Tool 3 won!");
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output.text) {
                        if let Some(tool_name) = parsed.get("tool") {
                            println!("   Winner: {}", tool_name);
                        }
                    }
                }
                Err(e) => println!("   ❌ Tool 3 failed: {}", e),
            }
        }
    }

    Ok(())
}

// Pattern 5: Async Pipeline
async fn pipeline_pattern() -> Result<()> {
    println!("\n5. Async Pipeline Pattern");
    println!("   {}", "-".repeat(50));

    // Create pipeline stages
    let stage1 = SlowTool::new("preprocessor".to_string(), 100);
    let stage2 = SlowTool::new("analyzer".to_string(), 200);
    let stage3 = SlowTool::new("formatter".to_string(), 150);

    println!("   Building 3-stage pipeline:");
    println!("   Stage 1: Preprocessing (100ms)");
    println!("   Stage 2: Analysis (200ms)");
    println!("   Stage 3: Formatting (150ms)");

    // Process multiple items through pipeline
    let items = vec!["item1", "item2", "item3"];
    let context = ExecutionContext::new();

    println!("\n   Processing {} items through pipeline:", items.len());
    let start = Instant::now();

    // Create futures for each item going through the pipeline
    let pipeline_futures = items.into_iter().map(|item| {
        let s1 = stage1.clone();
        let s2 = stage2.clone();
        let s3 = stage3.clone();
        let context = context.clone();

        async move {
            // Stage 1
            let input1 = AgentInput::text(item).with_parameter("input", json!(item));
            let r1 = s1.execute_impl(input1, context.clone()).await?;

            // Extract data from stage 1 output
            let stage1_data =
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&r1.text) {
                    parsed
                        .get("processed")
                        .and_then(|v| v.as_str())
                        .unwrap_or("stage1_default")
                        .to_string()
                } else {
                    "stage1_fallback".to_string()
                };

            // Stage 2
            let input2 = AgentInput::text(&stage1_data).with_parameter("input", json!(stage1_data));
            let r2 = s2.execute_impl(input2, context.clone()).await?;

            // Extract data from stage 2 output
            let stage2_data =
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&r2.text) {
                    parsed
                        .get("processed")
                        .and_then(|v| v.as_str())
                        .unwrap_or("stage2_default")
                        .to_string()
                } else {
                    "stage2_fallback".to_string()
                };

            // Stage 3
            let input3 = AgentInput::text(&stage2_data).with_parameter("input", json!(stage2_data));
            let r3 = s3.execute_impl(input3, context).await?;

            Ok::<_, LLMSpellError>((item, r3))
        }
    });

    // Execute all pipelines concurrently
    let results: Vec<_> = future::try_join_all(pipeline_futures).await?;

    let elapsed = start.elapsed();

    for (item, _result) in results {
        println!("   ✅ Completed pipeline for: {}", item);
    }

    println!("   ⏱️  Total pipeline time: {:?}", elapsed);
    println!("   (Note: Items processed concurrently through all stages)");

    println!("\n   📊 Pipeline Performance Analysis:");
    println!("   - Sequential per item: 450ms (100+200+150)");
    println!("   - 3 items sequential: 1350ms total");
    println!("   - Concurrent pipeline: ~450ms (items overlap in pipeline)");
    println!("   - Actual time: {:?}", elapsed);

    Ok(())
}
