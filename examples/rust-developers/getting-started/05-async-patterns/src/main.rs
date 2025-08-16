// ABOUTME: Example demonstrating async patterns with LLMSpell components
// ABOUTME: Shows concurrent execution, streaming, and async composition

use anyhow::Result;
use async_trait::async_trait;
use futures::{future, stream, StreamExt};
use llmspell_core::{Tool, ToolInput, ToolOutput, ToolRegistry};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::{join, select, time};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Example: Async tool that simulates a slow operation
#[derive(Debug, Clone)]
struct SlowTool {
    name: String,
    delay_ms: u64,
}

#[async_trait]
impl Tool for SlowTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "A tool that simulates slow async operations"
    }

    fn parameters(&self) -> Vec<(&str, &str)> {
        vec![("input", "Input data to process")]
    }

    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        let data = input
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Simulate async work
        time::sleep(Duration::from_millis(self.delay_ms)).await;

        Ok(ToolOutput::from_json(json!({
            "tool": self.name,
            "processed": format!("Processed: {}", data),
            "duration_ms": self.delay_ms,
            "success": true
        })))
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
    println!("\n💡 Key Async Concepts:");
    println!("   - Use tokio::join! for concurrent execution");
    println!("   - Implement streaming for real-time data");
    println!("   - Apply timeouts to prevent hanging");
    println!("   - Use select! for first-to-complete patterns");
    println!("   - Build async pipelines for data processing");

    Ok(())
}

// Pattern 1: Concurrent Tool Execution
async fn concurrent_tools() -> Result<()> {
    println!("1. Concurrent Tool Execution");
    println!("   " + &"-".repeat(40));

    // Create tools with different delays
    let fast_tool = Arc::new(SlowTool {
        name: "fast_tool".to_string(),
        delay_ms: 100,
    }) as Arc<dyn Tool>;

    let medium_tool = Arc::new(SlowTool {
        name: "medium_tool".to_string(),
        delay_ms: 300,
    }) as Arc<dyn Tool>;

    let slow_tool = Arc::new(SlowTool {
        name: "slow_tool".to_string(),
        delay_ms: 500,
    }) as Arc<dyn Tool>;

    // Sequential execution (for comparison)
    println!("   Sequential execution:");
    let start = Instant::now();
    
    let r1 = fast_tool.invoke(json!({ "input": "data1" })).await?;
    let r2 = medium_tool.invoke(json!({ "input": "data2" })).await?;
    let r3 = slow_tool.invoke(json!({ "input": "data3" })).await?;
    
    let sequential_time = start.elapsed();
    println!("   ⏱️  Sequential time: {:?}", sequential_time);

    // Concurrent execution
    println!("\n   Concurrent execution:");
    let start = Instant::now();
    
    let (r1, r2, r3) = join!(
        fast_tool.invoke(json!({ "input": "data1" })),
        medium_tool.invoke(json!({ "input": "data2" })),
        slow_tool.invoke(json!({ "input": "data3" }))
    );
    
    let concurrent_time = start.elapsed();
    println!("   ⏱️  Concurrent time: {:?}", concurrent_time);
    println!("   🚀 Speedup: {:.2}x", 
             sequential_time.as_millis() as f64 / concurrent_time.as_millis() as f64);

    Ok(())
}

// Pattern 2: Streaming Operations
async fn streaming_operations() -> Result<()> {
    println!("\n2. Streaming Operations");
    println!("   " + &"-".repeat(40));

    // Create a stream of data to process
    let data_stream = stream::iter(vec![
        "chunk1",
        "chunk2",
        "chunk3",
        "chunk4",
        "chunk5",
    ]);

    // Process stream items concurrently (up to 3 at a time)
    println!("   Processing stream with concurrency limit of 3:");
    
    let tool = Arc::new(SlowTool {
        name: "stream_processor".to_string(),
        delay_ms: 200,
    });

    let results: Vec<_> = data_stream
        .map(|chunk| {
            let tool = tool.clone();
            async move {
                println!("   Processing: {}", chunk);
                tool.invoke(json!({ "input": chunk })).await
            }
        })
        .buffer_unordered(3) // Process up to 3 concurrently
        .collect()
        .await;

    println!("   ✅ Processed {} chunks", results.len());

    // Demonstrate async iteration
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
    println!("   " + &"-".repeat(40));

    let slow_tool = Arc::new(SlowTool {
        name: "very_slow_tool".to_string(),
        delay_ms: 2000,
    });

    // Attempt with timeout
    println!("   Attempting operation with 1-second timeout:");
    
    let result = time::timeout(
        Duration::from_secs(1),
        slow_tool.invoke(json!({ "input": "timeout_test" }))
    ).await;

    match result {
        Ok(Ok(output)) => {
            println!("   ✅ Operation completed: {:?}", output.to_json());
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
    
    let result = time::timeout(
        Duration::from_secs(3),
        slow_tool.invoke(json!({ "input": "retry_test" }))
    ).await;

    match result {
        Ok(Ok(_)) => {
            println!("   ✅ Operation completed successfully");
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
    println!("   " + &"-".repeat(40));

    // Create tools with random delays
    let tool1 = Arc::new(SlowTool {
        name: "racer1".to_string(),
        delay_ms: 300,
    });

    let tool2 = Arc::new(SlowTool {
        name: "racer2".to_string(),
        delay_ms: 250,
    });

    let tool3 = Arc::new(SlowTool {
        name: "racer3".to_string(),
        delay_ms: 400,
    });

    println!("   Racing 3 tools - first to complete wins:");

    select! {
        result1 = tool1.invoke(json!({ "input": "race" })) => {
            if let Ok(output) = result1 {
                println!("   🏆 Tool 1 won! Result: {:?}", 
                        output.to_json().get("tool"));
            }
        }
        result2 = tool2.invoke(json!({ "input": "race" })) => {
            if let Ok(output) = result2 {
                println!("   🏆 Tool 2 won! Result: {:?}", 
                        output.to_json().get("tool"));
            }
        }
        result3 = tool3.invoke(json!({ "input": "race" })) => {
            if let Ok(output) = result3 {
                println!("   🏆 Tool 3 won! Result: {:?}", 
                        output.to_json().get("tool"));
            }
        }
    }

    Ok(())
}

// Pattern 5: Async Pipeline
async fn pipeline_pattern() -> Result<()> {
    println!("\n5. Async Pipeline Pattern");
    println!("   " + &"-".repeat(40));

    // Create pipeline stages
    let stage1 = Arc::new(SlowTool {
        name: "preprocessor".to_string(),
        delay_ms: 100,
    });

    let stage2 = Arc::new(SlowTool {
        name: "analyzer".to_string(),
        delay_ms: 200,
    });

    let stage3 = Arc::new(SlowTool {
        name: "formatter".to_string(),
        delay_ms: 150,
    });

    println!("   Building 3-stage pipeline:");
    println!("   Stage 1: Preprocessing (100ms)");
    println!("   Stage 2: Analysis (200ms)");
    println!("   Stage 3: Formatting (150ms)");

    // Process multiple items through pipeline
    let items = vec!["item1", "item2", "item3"];
    
    println!("\n   Processing {} items through pipeline:", items.len());
    let start = Instant::now();

    // Create futures for each item going through the pipeline
    let pipeline_futures = items.into_iter().map(|item| {
        let s1 = stage1.clone();
        let s2 = stage2.clone();
        let s3 = stage3.clone();
        
        async move {
            // Stage 1
            let r1 = s1.invoke(json!({ "input": item })).await?;
            let data1 = r1.to_json().get("processed")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            // Stage 2
            let r2 = s2.invoke(json!({ "input": data1 })).await?;
            let data2 = r2.to_json().get("processed")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            // Stage 3
            let r3 = s3.invoke(json!({ "input": data2 })).await?;
            
            Ok::<_, anyhow::Error>((item, r3))
        }
    });

    // Execute all pipelines concurrently
    let results: Vec<_> = future::try_join_all(pipeline_futures).await?;
    
    let elapsed = start.elapsed();
    
    for (item, _result) in results {
        println!("   ✅ Completed pipeline for: {}", item);
    }
    
    println!("   ⏱️  Total pipeline time: {:?}", elapsed);
    println!("   (Note: Items processed concurrently through stages)");

    Ok(())
}