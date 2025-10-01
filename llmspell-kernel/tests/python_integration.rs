//! Integration test wrapper for Python-based Jupyter DAP tests.
//!
//! This test runs the Python test suite that validates DAP functionality
//! through real Jupyter protocol interactions. The Python tests use jupyter_client
//! to connect to a subprocess-managed llmspell daemon.
//!
//! REQUIREMENTS:
//! - Python 3 with pytest and jupyter_client installed
//! - A built llmspell binary in target/debug/ or target/release/
//! - The tests will automatically start a kernel daemon if needed
//! - Requires network access for ZMQ communication
//! - May take 30+ seconds to complete all DAP tests
//!
//! RUNNING THE TESTS:
//! - To skip Python tests during development:
//!   cargo test --features skip-python-tests
//!
//! - To run only Python tests with output:
//!   cargo test python_jupyter_integration -- --nocapture
//!
//! - To run directly (for debugging):
//!   cd tests/python && python3 -m pytest test_dap*.py -v
//!
//! These tests are marked as #[ignore] by default since they require
//! special setup and take significant time to run.

#[cfg(not(feature = "skip-python-tests"))]
use std::process::Command;

/// Run Python integration tests for Jupyter DAP functionality.
///
/// This test:
/// 1. Ensures llmspell is built (requires `cargo build` to have been run)
/// 2. Runs the Python test suite via shell script
/// 3. Each Python test starts its own kernel daemon if needed
/// 4. Reports success/failure based on Python test results
///
/// The Python tests validate:
/// - DAP initialization and capabilities
/// - Breakpoint setting and hitting
/// - Stepping operations (over, in, out)
/// - Variable inspection
/// - Performance benchmarks
/// - Multiple simultaneous debug sessions
///
/// This test is ignored by default. To run:
/// cargo test python_jupyter_integration -- --ignored --nocapture
#[test]
#[ignore = "Requires Python environment, built binary, and takes 30+ seconds"]
#[cfg(not(feature = "skip-python-tests"))]
fn test_python_jupyter_integration() {
    // Determine if we're in verbose mode
    let verbose =
        std::env::var("RUST_LOG").is_ok() || std::env::args().any(|arg| arg == "--nocapture");

    // Build the command - script is in root tests/scripts directory
    // When run via cargo test, we're in the package directory
    let script_path = if std::path::Path::new("../tests/scripts/run_python_tests.sh").exists() {
        "../tests/scripts/run_python_tests.sh"
    } else if std::path::Path::new("tests/scripts/run_python_tests.sh").exists() {
        "tests/scripts/run_python_tests.sh"
    } else {
        panic!("Cannot find run_python_tests.sh script");
    };

    let mut cmd = Command::new("bash");
    cmd.arg(script_path);

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
             2. Run the Python tests directly: ../tests/scripts/run_python_tests.sh --verbose\n\
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
/// Checks for Python 3 and pip availability.
#[test]
#[ignore = "Part of Python integration test suite"]
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
        .args(["-m", "pip", "--version"])
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
///
/// Verifies the test infrastructure is in place.
#[test]
#[ignore = "Part of Python integration test suite"]
#[cfg(not(feature = "skip-python-tests"))]
fn test_python_test_script_exists() {
    // Try both possible locations
    let script_path = if std::path::Path::new("../tests/scripts/run_python_tests.sh").exists() {
        "../tests/scripts/run_python_tests.sh"
    } else if std::path::Path::new("tests/scripts/run_python_tests.sh").exists() {
        "tests/scripts/run_python_tests.sh"
    } else {
        ""
    };

    // Check if file exists
    if script_path.is_empty() {
        panic!(
            "Python test script not found at tests/scripts/run_python_tests.sh or ../tests/scripts/run_python_tests.sh.\n\
             Ensure you're running tests from the project root."
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
