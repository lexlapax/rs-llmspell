//! GPU detection validation tests for cross-platform support
//!
//! These tests validate that GPU detection works correctly on all platforms:
//! - macOS: Metal → CPU fallback
//! - Linux: CUDA → CPU fallback
//! - All: CPU always available

use llmspell_providers::local::candle::CandleProvider;
use tempfile::TempDir;

#[test]
fn test_cpu_device_always_available() {
    // CPU should work on all platforms
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "cpu".to_string(),
    );

    assert!(provider.is_ok(), "CPU device should always be available");
    println!("✅ CPU device initialization: OK");
}

#[test]
fn test_auto_device_detection_no_panic() {
    // Auto detection should never panic, even without GPU
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "auto".to_string(),
    );

    assert!(provider.is_ok(), "Auto device detection should not panic");
    println!("✅ Auto device detection: OK (no panic)");
}

#[cfg(target_os = "macos")]
#[test]
fn test_metal_on_macos() {
    // On macOS, Metal request should either succeed or fail gracefully
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "metal".to_string(),
    );

    // Either Metal works or it fails with clear error message
    match provider {
        Ok(_) => println!("✅ Metal device: Available"),
        Err(e) => {
            println!(
                "✅ Metal device: Not available (expected on some macOS): {}",
                e
            );
            assert!(e.to_string().contains("Metal not available"));
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn test_metal_fallback_on_linux() {
    // On Linux, Metal request should fallback to CPU with warning
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "metal".to_string(),
    );

    assert!(provider.is_ok(), "Metal on Linux should fallback to CPU");
    println!("✅ Metal on Linux: Fallback to CPU (expected behavior)");
}

#[cfg(not(target_os = "macos"))]
#[test]
fn test_cuda_detection_on_linux() {
    // On Linux, CUDA may or may not be available
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "cuda".to_string(),
    );

    match provider {
        Ok(_) => println!("✅ CUDA device: Available on this system"),
        Err(e) => {
            println!("✅ CUDA device: Not available (expected if no CUDA): {}", e);
            assert!(e.to_string().contains("CUDA not available"));
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
fn test_cuda_on_macos_fallback() {
    // On macOS, CUDA should fallback to CPU with helpful message
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "cuda".to_string(),
    );

    assert!(provider.is_ok(), "CUDA on macOS should fallback to CPU");
    println!("✅ CUDA on macOS: Fallback to CPU (expected behavior)");
}

#[test]
fn test_invalid_device_fallback() {
    // Invalid device string should fallback to CPU
    let temp_dir = TempDir::new().unwrap();
    let provider = CandleProvider::new(
        "test-model".to_string(),
        Some(temp_dir.path().to_path_buf()),
        "invalid-device".to_string(),
    );

    assert!(provider.is_ok(), "Invalid device should fallback to CPU");
    println!("✅ Invalid device: Fallback to CPU");
}
