//! Integration tests for Ollama provider
//!
//! Requires: OLLAMA_AVAILABLE=1 and Ollama server running (`ollama serve`)
//! Run with: OLLAMA_AVAILABLE=1 cargo test --package llmspell-providers --test ollama_integration_test

use llmspell_core::types::AgentInput;
use llmspell_providers::{create_ollama_provider, ProviderConfig};

/// Check if Ollama tests should run
fn should_run_ollama_tests() -> bool {
    std::env::var("OLLAMA_AVAILABLE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

#[tokio::test]
async fn test_ollama_provider_creation() {
    // Always run (no Ollama server needed for creation)
    println!("\n=== Ollama Test: Provider Creation ===\n");

    let config = ProviderConfig::new_with_type("ollama", "local", "llama3.1:8b");
    let provider = create_ollama_provider(config);

    assert!(provider.is_ok(), "Provider creation should succeed");
    let provider = provider.unwrap();
    assert_eq!(provider.name(), "ollama");

    println!("✅ Provider created successfully: {}", provider.name());
}

#[tokio::test]
async fn test_ollama_list_models() {
    if !should_run_ollama_tests() {
        println!("Skipping Ollama test (set OLLAMA_AVAILABLE=1 to run)");
        return;
    }

    println!("\n=== Ollama Test: List Models ===\n");

    let config = ProviderConfig::new_with_type("ollama", "local", "llama3.1:8b");
    let provider = create_ollama_provider(config).expect("Provider creation failed");

    if let Some(local) = provider.as_local() {
        let models = local.list_local_models().await.expect("List models failed");
        assert!(!models.is_empty(), "Should list models");

        println!("Found {} models:", models.len());
        for model in models.iter().take(5) {
            println!("  - {}", model.id);
        }

        println!("\n✅ Listed {} models successfully", models.len());
    } else {
        panic!("Provider should implement LocalProviderInstance");
    }
}

#[tokio::test]
async fn test_ollama_inference() {
    if !should_run_ollama_tests() {
        println!("Skipping Ollama test (set OLLAMA_AVAILABLE=1 to run)");
        return;
    }

    println!("\n=== Ollama Test: Inference ===\n");

    // Use smallest available model for faster testing
    let config = ProviderConfig::new_with_type("ollama", "local", "deepseek-r1:1.5b");
    let provider = create_ollama_provider(config).expect("Provider creation failed");

    let input = AgentInput::text("Write a one-sentence haiku about Rust programming.");
    println!("Prompt: {}", input.text);
    println!("Generating response...\n");

    let start = std::time::Instant::now();
    let output = provider.complete(&input).await;
    let duration = start.elapsed();

    assert!(
        output.is_ok(),
        "Inference should succeed: {:?}",
        output.err()
    );

    let output = output.unwrap();
    assert!(!output.text.is_empty(), "Should generate text");

    println!("Response: {}", output.text);
    println!("Duration: {:.2}s", duration.as_secs_f64());
    println!("\n✅ Inference completed successfully");
}

#[tokio::test]
async fn test_ollama_model_info() {
    if !should_run_ollama_tests() {
        println!("Skipping Ollama test (set OLLAMA_AVAILABLE=1 to run)");
        return;
    }

    println!("\n=== Ollama Test: Model Info ===\n");

    let config = ProviderConfig::new_with_type("ollama", "local", "llama3.1:8b");
    let provider = create_ollama_provider(config).expect("Provider creation failed");

    if let Some(local) = provider.as_local() {
        let info = local
            .model_info("llama3.1:8b")
            .await
            .expect("Model info failed");

        println!("Model: {}", info.id);
        println!(
            "Size: {} bytes ({:.2} GB)",
            info.size_bytes,
            info.size_bytes as f64 / 1_000_000_000.0
        );

        if let Some(params) = &info.parameter_count {
            println!("Parameters: {}", params);
        }

        assert_eq!(info.id, "llama3.1:8b");

        println!("\n✅ Model info retrieved successfully");
    } else {
        panic!("Provider should implement LocalProviderInstance");
    }
}

#[tokio::test]
async fn test_ollama_health_check() {
    if !should_run_ollama_tests() {
        println!("Skipping Ollama test (set OLLAMA_AVAILABLE=1 to run)");
        return;
    }

    println!("\n=== Ollama Test: Health Check ===\n");

    let config = ProviderConfig::new_with_type("ollama", "local", "llama3.1:8b");
    let provider = create_ollama_provider(config).expect("Provider creation failed");

    if let Some(local) = provider.as_local() {
        let health = local.health_check().await.expect("Health check failed");

        println!("Health: {:?}", health);

        // Health check should return some status
        println!("\n✅ Health check completed successfully");
    } else {
        panic!("Provider should implement LocalProviderInstance");
    }
}
