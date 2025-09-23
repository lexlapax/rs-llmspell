//! Integration test wrapper for Python-based Jupyter DAP tests.
//!
//! This test runs the Python test suite that validates DAP functionality
//! through real Jupyter protocol interactions. The Python tests use jupyter_client
//! to connect to a subprocess-managed llmspell daemon.
//!
//! To skip Python tests during development, use:
//! cargo test --features skip-python-tests
//!
//! To run only Python tests:
//! cargo test python_jupyter_integration -- --nocapture

#[cfg(not(feature = "skip-python-tests"))]
use std::process::Command;

/// Run Python integration tests for Jupyter DAP functionality.
///
/// This test:
/// 1. Ensures llmspell is built
/// 2. Runs the Python test suite via shell script
/// 3. Reports success/failure based on Python test results
///
/// The Python tests validate:
/// - DAP initialization and capabilities
/// - Breakpoint setting and hitting
/// - Stepping operations (over, in, out)
/// - Variable inspection
/// - Performance benchmarks
/// - Multiple simultaneous debug sessions
#[test]
#[cfg(not(feature = "skip-python-tests"))]
fn test_python_jupyter_integration() {
    // Determine if we're in verbose mode
    let verbose =
        std::env::var("RUST_LOG").is_ok() || std::env::args().any(|arg| arg == "--nocapture");

    // Build the command
    let mut cmd = Command::new("bash");
    cmd.arg("tests/scripts/run_python_tests.sh");

    if verbose {
        cmd.arg("--verbose");
    }

    // Set environment for better error reporting
    cmd.env("RUST_BACKTRACE", "1");

    println!("Running Python integration tests for Jupyter DAP...");

    // Execute the tests
    let output = cmd
        .output()
        .expect("Failed to run Python tests - is bash available?");

    // Convert output to strings
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Always print output in verbose mode or on failure
    if verbose || !output.status.success() {
        if !stdout.is_empty() {
            println!("Python test stdout:\n{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("Python test stderr:\n{}", stderr);
        }
    }

    // Check for success
    if !output.status.success() {
        // Try to extract useful error information
        let exit_code = output.status.code().unwrap_or(-1);

        panic!(
            "Python integration tests failed with exit code {}.\n\
             \n\
             To debug:\n\
             1. Run with --nocapture to see full output\n\
             2. Run the Python tests directly: ./tests/scripts/run_python_tests.sh --verbose\n\
             3. Check if Python 3 and pip are installed\n\
             4. Ensure llmspell builds successfully: cargo build -p llmspell-cli\n\
             \n\
             To skip these tests temporarily, use: cargo test --features skip-python-tests",
            exit_code
        );
    }

    println!("Python integration tests completed successfully");
}

/// Verify Python environment is available for testing.
///
/// This is a separate test to help diagnose setup issues.
#[test]
#[cfg(not(feature = "skip-python-tests"))]
fn test_python_environment_available() {
    // Check if Python 3 is available
    let python_check = Command::new("python3").arg("--version").output();

    match python_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("Python available: {}", version.trim());
        }
        _ => {
            panic!(
                "Python 3 is not available or not in PATH.\n\
                 Python 3 is required to run integration tests.\n\
                 Install Python 3 or use --features skip-python-tests to skip."
            );
        }
    }

    // Check if pip is available
    let pip_check = Command::new("python3")
        .args(&["-m", "pip", "--version"])
        .output();

    match pip_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("Pip available: {}", version.trim());
        }
        _ => {
            panic!(
                "Pip is not available.\n\
                 Pip is required to install test dependencies.\n\
                 Install pip or use --features skip-python-tests to skip."
            );
        }
    }
}

/// Test that the Python test script exists and is executable.
#[test]
#[cfg(not(feature = "skip-python-tests"))]
fn test_python_test_script_exists() {
    let script_path = "tests/scripts/run_python_tests.sh";

    // Check if file exists
    if !std::path::Path::new(script_path).exists() {
        panic!(
            "Python test script not found at {}.\n\
             Ensure you're running tests from the project root.",
            script_path
        );
    }

    // Check if file is executable (on Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script_path).expect("Failed to read test script metadata");
        let permissions = metadata.permissions();

        if permissions.mode() & 0o111 == 0 {
            panic!(
                "Python test script is not executable.\n\
                 Run: chmod +x {}",
                script_path
            );
        }
    }

    println!("Python test script found and ready");
}
