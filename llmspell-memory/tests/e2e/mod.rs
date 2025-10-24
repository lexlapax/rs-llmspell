//! End-to-end tests for memory consolidation with real LLM
//!
//! These tests require a running Ollama instance with llama3.2:3b model.
//! Set `OLLAMA_HOST` environment variable (default: <http://localhost:11434>).
//! Tests will skip gracefully if Ollama is unavailable.

pub mod helpers;

use std::env;
use std::time::Duration;

/// Check if Ollama is available for testing
///
/// Checks `OLLAMA_HOST` environment variable (default: <http://localhost:11434>).
/// Attempts to connect and verify the service is responsive.
///
/// # Returns
///
/// `true` if Ollama is available, `false` otherwise
pub async fn check_ollama_available() -> bool {
    let ollama_host =
        env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());

    // Try to connect to Ollama API
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    // Check /api/tags endpoint to verify Ollama is running
    match client.get(format!("{ollama_host}/api/tags")).send().await {
        Ok(response) if response.status().is_success() => {
            eprintln!("✓ Ollama available at {ollama_host}");
            true
        }
        _ => {
            eprintln!(
                "✗ Ollama unavailable at {ollama_host} - skipping E2E tests"
            );
            eprintln!("  To run E2E tests: Start Ollama and ensure llama3.2:3b model is available");
            eprintln!("  Set OLLAMA_HOST env var if using non-default host");
            false
        }
    }
}

/// Get Ollama host from environment
pub fn get_ollama_host() -> String {
    env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string())
}
