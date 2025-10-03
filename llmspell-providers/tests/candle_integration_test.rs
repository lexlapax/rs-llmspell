//! Integration tests for Candle GGUF provider
//!
//! These tests download real models and perform inference.
//! Run with: RUN_EXPENSIVE_TESTS=1 cargo test --release --test candle_integration_test

use llmspell_core::types::AgentInput;
use llmspell_providers::abstraction::ProviderConfig;
use llmspell_providers::local::candle::{create_candle_provider, HFDownloader, HFModelRepo};
use llmspell_providers::local::ModelSpec;
use std::path::PathBuf;

/// Check if expensive tests should run
fn should_run_expensive_tests() -> bool {
    std::env::var("RUN_EXPENSIVE_TESTS")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get test model directory
fn get_test_model_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    temp_dir.join("llmspell-candle-test-models")
}

/// Download TinyLlama test model if not already present
async fn ensure_test_model() -> anyhow::Result<PathBuf> {
    let model_dir = get_test_model_dir();
    let model_id = "tinyllama:Q4_K_M";
    let model_path = model_dir.join(model_id);

    // Check if already downloaded
    if model_path.exists() {
        println!("Test model already present at {:?}", model_path);
        return Ok(model_path);
    }

    println!("Downloading TinyLlama test model...");

    // Get HF repo info
    let (repo_id, filename) = HFModelRepo::get_repo_info("tinyllama", "Q4_K_M")
        .expect("TinyLlama should be a known model");

    // Download
    let downloader = HFDownloader::new()?;
    let _gguf_path = downloader.download_model(repo_id, &filename, &model_path)?;

    println!("Test model downloaded to {:?}", model_path);
    Ok(model_path)
}

#[tokio::test]
async fn test_candle_provider_creation() {
    // This test always runs (no expensive operations)
    let config = ProviderConfig::new_with_type("candle", "local", "tinyllama:Q4_K_M");

    let provider = create_candle_provider(config);
    assert!(provider.is_ok(), "Provider creation should succeed");

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "candle");
}

#[tokio::test]
async fn test_candle_download_and_inference() {
    if !should_run_expensive_tests() {
        println!("Skipping expensive test (set RUN_EXPENSIVE_TESTS=1 to run)");
        return;
    }

    println!("\n=== Candle Integration Test: Download + Inference ===\n");

    // Setup
    let model_dir = get_test_model_dir();
    std::fs::create_dir_all(&model_dir).expect("Failed to create test directory");

    // Download model
    let model_path = ensure_test_model()
        .await
        .expect("Failed to download test model");

    println!("Model available at: {:?}\n", model_path);

    // Create provider with custom model directory
    let mut config = ProviderConfig::new_with_type("candle", "local", "tinyllama:Q4_K_M");
    config.custom_config.insert(
        "model_directory".to_string(),
        serde_json::Value::String(model_dir.to_string_lossy().to_string()),
    );
    config.custom_config.insert(
        "device".to_string(),
        serde_json::Value::String("auto".to_string()),
    );

    let provider = create_candle_provider(config).expect("Failed to create provider");

    println!("Provider created: {}\n", provider.name());

    // Perform health check
    if let Some(local_provider) = provider.as_local() {
        let health = local_provider
            .health_check()
            .await
            .expect("Health check failed");
        println!("Health check: {:?}\n", health);
    }

    // Test inference
    println!("=== Testing Inference ===\n");

    let input = AgentInput::text("Write a haiku about code.");

    println!("Prompt: {}\n", input.text);
    println!("Generating response...\n");

    let start = std::time::Instant::now();
    let output = provider.complete(&input).await;
    let duration = start.elapsed();

    if let Err(ref e) = output {
        println!("ERROR: {:?}", e);
    }
    assert!(output.is_ok(), "Inference should succeed");

    let output = output.unwrap();
    println!("=== Response ===");
    println!("{}", output.text);
    println!("\n=== Performance ===");
    println!("Total time: {:.2}s", duration.as_secs_f64());

    // Validate output
    assert!(!output.text.is_empty(), "Output should not be empty");
    assert!(
        output.text.len() > 10,
        "Output should contain meaningful text"
    );

    println!("\n=== Test Passed ===\n");
}

#[tokio::test]
async fn test_candle_pull_model() {
    if !should_run_expensive_tests() {
        println!("Skipping expensive test (set RUN_EXPENSIVE_TESTS=1 to run)");
        return;
    }

    println!("\n=== Candle Integration Test: Model Pull ===\n");

    // Setup
    let model_dir = get_test_model_dir();
    std::fs::create_dir_all(&model_dir).expect("Failed to create test directory");

    // Create provider
    let mut config = ProviderConfig::new_with_type("candle", "local", "tinyllama:Q4_K_M");
    config.custom_config.insert(
        "model_directory".to_string(),
        serde_json::Value::String(model_dir.to_string_lossy().to_string()),
    );

    let provider = create_candle_provider(config).expect("Failed to create provider");

    // Pull model using provider API
    if let Some(local_provider) = provider.as_local() {
        let spec = ModelSpec {
            model: "tinyllama".to_string(),
            variant: Some("Q4_K_M".to_string()),
            backend: None,
        };

        println!("Pulling model: {:?}\n", spec);

        let progress = local_provider.pull_model(&spec).await;
        assert!(progress.is_ok(), "Model pull should succeed");

        let progress = progress.unwrap();
        println!("Pull progress: {:?}", progress);
        println!("Status: {:?}", progress.status);
        println!("Downloaded: {} bytes", progress.bytes_downloaded);

        // Verify model exists
        let models = local_provider
            .list_local_models()
            .await
            .expect("Failed to list models");

        println!("\nLocal models:");
        for model in &models {
            println!("  - {} ({} bytes)", model.id, model.size_bytes);
        }

        assert!(!models.is_empty(), "Should have at least one model");
    }

    println!("\n=== Test Passed ===\n");
}

#[tokio::test]
async fn test_candle_performance_benchmark() {
    if !should_run_expensive_tests() {
        println!("Skipping expensive test (set RUN_EXPENSIVE_TESTS=1 to run)");
        return;
    }

    println!("\n=== Candle Performance Benchmark ===\n");

    // Download model
    let model_path = ensure_test_model()
        .await
        .expect("Failed to download test model");

    // Create provider
    let model_dir = model_path.parent().unwrap().parent().unwrap();
    let mut config = ProviderConfig::new_with_type("candle", "local", "tinyllama:Q4_K_M");
    config.custom_config.insert(
        "model_directory".to_string(),
        serde_json::Value::String(model_dir.to_string_lossy().to_string()),
    );

    let provider = create_candle_provider(config).expect("Failed to create provider");

    // Test prompts of varying lengths
    let test_cases = vec![
        ("Short prompt", "Hello!", 20),
        ("Medium prompt", "Write a short story about a robot learning to paint.", 50),
        (
            "Long prompt",
            "Explain the concept of machine learning, including supervised learning, unsupervised learning, and reinforcement learning, with examples.",
            100,
        ),
    ];

    println!(
        "Running benchmark with {} test cases...\n",
        test_cases.len()
    );

    for (name, prompt, max_tokens) in test_cases {
        println!("=== {} ===", name);
        println!("Prompt: {}", prompt);

        let mut input = AgentInput::text(prompt);
        input.parameters.insert(
            "max_tokens".to_string(),
            serde_json::Value::Number(max_tokens.into()),
        );

        let start = std::time::Instant::now();
        let result = provider.complete(&input).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Inference should succeed");

        let output = result.unwrap();
        println!("Generated {} chars", output.text.len());
        println!("Time: {:.2}ms", duration.as_millis());
        println!();
    }

    println!("=== Benchmark Complete ===\n");
}

#[tokio::test]
async fn test_candle_model_info() {
    if !should_run_expensive_tests() {
        println!("Skipping expensive test (set RUN_EXPENSIVE_TESTS=1 to run)");
        return;
    }

    // Ensure model is downloaded
    let model_path = ensure_test_model()
        .await
        .expect("Failed to download test model");

    // Create provider
    let model_dir = model_path.parent().unwrap().parent().unwrap();
    let mut config = ProviderConfig::new_with_type("candle", "local", "tinyllama:Q4_K_M");
    config.custom_config.insert(
        "model_directory".to_string(),
        serde_json::Value::String(model_dir.to_string_lossy().to_string()),
    );

    let provider = create_candle_provider(config).expect("Failed to create provider");

    // Get model info
    if let Some(local_provider) = provider.as_local() {
        let info = local_provider
            .model_info("tinyllama:Q4_K_M")
            .await
            .expect("Failed to get model info");

        println!("Model info:");
        println!("  ID: {}", info.id);
        println!("  Backend: {}", info.backend);
        println!("  Size: {} bytes", info.size_bytes);
        println!("  Format: {}", info.format);
        println!("  Quantization: {:?}", info.quantization);

        assert_eq!(info.backend, "candle");
        assert_eq!(info.format, "GGUF");
        assert!(info.size_bytes > 0);
    }
}
