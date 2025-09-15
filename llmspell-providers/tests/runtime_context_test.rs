//! Integration test for provider runtime context fixes
//! Tests that providers survive 60+ second operations without "dispatch task is gone" errors

use llmspell_core::types::{AgentInput, MediaType};
use llmspell_providers::{
    abstraction::{ProviderConfig, ProviderInstance},
    rig::RigProvider,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
#[ignore] // Ignore by default since this is a long-running test
async fn test_provider_runtime_context_60_seconds() {
    println!("Starting 60+ second provider runtime context test");

    // Create a test provider configuration (no real API key needed for runtime testing)
    let config = ProviderConfig {
        name: "test-provider".to_string(),
        provider_type: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        api_key: None, // No API key needed for runtime context testing
        endpoint: None,
        timeout_secs: Some(30),
        max_retries: Some(2),
        custom_config: HashMap::new(),
    };

    // Create provider instance - this should work even without API key for runtime testing
    let provider = RigProvider::new(config).expect("Failed to create provider");

    let start_time = Instant::now();
    let test_duration = Duration::from_secs(65); // Test for 65 seconds

    println!(
        "Running provider operations for {} seconds...",
        test_duration.as_secs()
    );

    let mut iteration = 0;
    while start_time.elapsed() < test_duration {
        iteration += 1;
        let iteration_start = Instant::now();

        println!(
            "Iteration {}: Testing provider HTTP client context...",
            iteration
        );

        // Create test input
        let test_input = AgentInput {
            text: format!("Test prompt #{} - This tests runtime context", iteration),
            media: vec![],
            context: None,
            parameters: HashMap::new(),
            output_modalities: vec![MediaType::Text],
        };

        // Test provider completion - this exercises the HTTP client with global runtime
        // We expect this to fail due to no API key, but the important thing is that it
        // doesn't fail with "dispatch task is gone" error
        let result = provider.complete(&test_input).await;

        match result {
            Ok(output) => {
                println!(
                    "Iteration {} completed: {} chars, duration: {}ms",
                    iteration,
                    output.text.len(),
                    iteration_start.elapsed().as_millis()
                );
            }
            Err(e) => {
                // Expected to fail without API key, but should NOT be "dispatch task is gone"
                let error_msg = e.to_string();
                println!(
                    "Iteration {} failed as expected (no API key): {}",
                    iteration, error_msg
                );

                // Assert that we don't get the specific runtime context error
                assert!(
                    !error_msg.contains("dispatch task is gone"),
                    "Runtime context error detected: {}",
                    error_msg
                );
                assert!(
                    !error_msg.contains("runtime has been dropped"),
                    "Runtime dropped error detected: {}",
                    error_msg
                );
            }
        }

        // Report progress every 20 seconds
        let elapsed = start_time.elapsed();
        if elapsed.as_secs() % 20 == 0 && elapsed.as_secs() > 0 {
            println!(
                "Progress: {:.1}% complete, {} iterations in {}s",
                (elapsed.as_secs_f64() / test_duration.as_secs_f64()) * 100.0,
                iteration,
                elapsed.as_secs()
            );
        }

        // Small delay between iterations
        sleep(Duration::from_millis(200)).await;
    }

    let total_elapsed = start_time.elapsed();
    println!("Provider runtime context test completed successfully!");
    println!("Total duration: {:.2}s", total_elapsed.as_secs_f64());
    println!("Total iterations: {}", iteration);
    println!(
        "Average iteration time: {:.2}ms",
        total_elapsed.as_millis() as f64 / iteration as f64
    );

    // Report provider metrics
    println!(
        "Provider total requests: {}",
        provider.total_requests_made()
    );
    println!("Provider total tokens: {}", provider.total_tokens_used());
    println!(
        "Provider total cost: {:.2}¢",
        provider.total_cost_cents() as f64 / 100.0
    );

    println!("✅ No 'dispatch task is gone' errors - runtime context fix validated!");

    // Ensure we ran for at least 60 seconds
    assert!(
        total_elapsed >= Duration::from_secs(60),
        "Test should run for at least 60 seconds, but only ran for {:.2}s",
        total_elapsed.as_secs_f64()
    );
}
