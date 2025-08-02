//! ABOUTME: Test environment setup and configuration helpers
//! ABOUTME: Provides utilities for setting up consistent test environments

//! Environment testing helpers.
//!
//! This module provides utilities for setting up and managing
//! test environments, including temporary directories, environment
//! variables, and test contexts.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::environment_helpers::{
//!     TestEnvironment,
//!     create_test_context,
//!     with_test_env_vars,
//! };
//!
//! # async fn test_example() {
//! // Create a test environment
//! let env = TestEnvironment::new("my-test").await;
//!
//! // Use test environment variables
//! with_test_env_vars(vec![
//!     ("LLMSPELL_TEST", "true"),
//!     ("LLMSPELL_LOG_LEVEL", "debug"),
//! ], || {
//!     // Code that uses environment variables
//! });
//!
//! // Create a test execution context
//! let context = create_test_context();
//! # }
//! ```

use llmspell_core::execution_context::ExecutionContext;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test environment manager
pub struct TestEnvironment {
    name: String,
    temp_dir: TempDir,
    original_env: HashMap<String, Option<String>>,
    test_data_dir: PathBuf,
}

impl TestEnvironment {
    /// Create a new test environment
    pub async fn new(name: &str) -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let test_data_dir = temp_dir.path().join("test_data");
        std::fs::create_dir_all(&test_data_dir).expect("Failed to create test data dir");

        Self {
            name: name.to_string(),
            temp_dir,
            original_env: HashMap::new(),
            test_data_dir,
        }
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get the test data directory path
    pub fn test_data_dir(&self) -> &Path {
        &self.test_data_dir
    }

    /// Create a test file in the environment
    pub fn create_test_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.temp_dir.path().join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        std::fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    /// Create multiple test files
    pub fn create_test_files(&self, files: Vec<(&str, &str)>) -> Vec<PathBuf> {
        files
            .into_iter()
            .map(|(path, content)| self.create_test_file(path, content))
            .collect()
    }

    /// Set an environment variable for this test
    pub fn set_env_var(&mut self, key: &str, value: &str) {
        // Save original value
        self.original_env
            .insert(key.to_string(), std::env::var(key).ok());
        // Set new value
        std::env::set_var(key, value);
    }

    /// Set multiple environment variables
    pub fn set_env_vars(&mut self, vars: Vec<(&str, &str)>) {
        for (key, value) in vars {
            self.set_env_var(key, value);
        }
    }

    /// Create a subdirectory in the test environment
    pub fn create_dir(&self, relative_path: &str) -> PathBuf {
        let dir_path = self.temp_dir.path().join(relative_path);
        std::fs::create_dir_all(&dir_path).expect("Failed to create directory");
        dir_path
    }

    /// Get a path within the test environment
    pub fn path(&self, relative_path: &str) -> PathBuf {
        self.temp_dir.path().join(relative_path)
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Restore original environment variables
        for (key, original_value) in &self.original_env {
            match original_value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

/// Create a test execution context
pub fn create_test_context() -> ExecutionContext {
    ExecutionContext::with_conversation("test-session".to_string())
        .with_data("test_mode".to_string(), json!(true))
        .with_data("user_id".to_string(), json!("test-user"))
        .with_data("LLMSPELL_ENV".to_string(), json!("test"))
}

/// Create a test execution context with custom data
pub fn create_test_context_with_data(data: HashMap<String, serde_json::Value>) -> ExecutionContext {
    let mut context = create_test_context();
    for (key, value) in data {
        context = context.with_data(key, value);
    }
    context
}

/// Run a function with temporary environment variables
pub fn with_test_env_vars<F, R>(vars: Vec<(&str, &str)>, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Save original values
    let original_values: HashMap<String, Option<String>> = vars
        .iter()
        .map(|(key, _)| (key.to_string(), std::env::var(key).ok()))
        .collect();

    // Set test values
    for (key, value) in &vars {
        std::env::set_var(key, value);
    }

    // Run the function
    let result = f();

    // Restore original values
    for (key, original_value) in original_values {
        match original_value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    result
}

/// Common test environment configurations
pub mod configs {
    /// Development environment configuration
    pub fn development_env() -> Vec<(&'static str, &'static str)> {
        vec![
            ("LLMSPELL_ENV", "development"),
            ("LLMSPELL_LOG_LEVEL", "debug"),
            ("LLMSPELL_LOG_FORMAT", "pretty"),
            ("RUST_BACKTRACE", "1"),
        ]
    }

    /// Production-like environment configuration
    pub fn production_env() -> Vec<(&'static str, &'static str)> {
        vec![
            ("LLMSPELL_ENV", "production"),
            ("LLMSPELL_LOG_LEVEL", "info"),
            ("LLMSPELL_LOG_FORMAT", "json"),
            ("RUST_BACKTRACE", "0"),
        ]
    }

    /// CI environment configuration
    pub fn ci_env() -> Vec<(&'static str, &'static str)> {
        vec![
            ("LLMSPELL_ENV", "ci"),
            ("LLMSPELL_LOG_LEVEL", "info"),
            ("LLMSPELL_LOG_FORMAT", "json"),
            ("CI", "true"),
            ("RUST_BACKTRACE", "full"),
        ]
    }

    /// Minimal test environment
    pub fn minimal_env() -> Vec<(&'static str, &'static str)> {
        vec![("LLMSPELL_ENV", "test"), ("LLMSPELL_LOG_LEVEL", "error")]
    }
}

/// Test data generators for environment testing
pub mod test_data {
    /// Generate test configuration files
    pub fn config_files() -> Vec<(&'static str, &'static str)> {
        vec![
            ("config.json", r#"{"version": "1.0.0", "debug": true}"#),
            ("config.yaml", "version: 1.0.0\ndebug: true\n"),
            ("config.toml", "[app]\nversion = \"1.0.0\"\ndebug = true\n"),
            (".env", "APP_VERSION=1.0.0\nDEBUG=true\n"),
        ]
    }

    /// Generate test script files
    pub fn script_files() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "test.py",
                "#!/usr/bin/env python3\nprint('Hello from Python')\n",
            ),
            ("test.sh", "#!/bin/bash\necho 'Hello from Bash'\n"),
            (
                "test.js",
                "#!/usr/bin/env node\nconsole.log('Hello from Node');\n",
            ),
            ("test.lua", "-- Lua test script\nprint('Hello from Lua')\n"),
        ]
    }

    /// Generate test data files
    pub fn data_files() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "data.json",
                r#"[{"id": 1, "name": "Test 1"}, {"id": 2, "name": "Test 2"}]"#,
            ),
            ("data.csv", "id,name\n1,Test 1\n2,Test 2\n"),
            ("data.txt", "Line 1\nLine 2\nLine 3\n"),
            (
                "data.xml",
                "<?xml version=\"1.0\"?>\n<data><item>Test</item></data>\n",
            ),
        ]
    }
}

/// Test environment assertions
pub mod assertions {
    use super::*;

    /// Assert that a file exists in the test environment
    pub fn assert_file_exists(env: &TestEnvironment, relative_path: &str) {
        let path = env.path(relative_path);
        assert!(
            path.exists(),
            "Expected file to exist at: {}",
            path.display()
        );
    }

    /// Assert that a directory exists in the test environment
    pub fn assert_dir_exists(env: &TestEnvironment, relative_path: &str) {
        let path = env.path(relative_path);
        assert!(
            path.exists() && path.is_dir(),
            "Expected directory to exist at: {}",
            path.display()
        );
    }

    /// Assert file contents match
    pub fn assert_file_contents(env: &TestEnvironment, relative_path: &str, expected: &str) {
        let path = env.path(relative_path);
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
        assert_eq!(
            contents,
            expected,
            "File contents mismatch for: {}",
            path.display()
        );
    }

    /// Assert environment variable is set
    pub fn assert_env_var(key: &str, expected: &str) {
        let value =
            std::env::var(key).unwrap_or_else(|_| panic!("Environment variable not set: {}", key));
        assert_eq!(
            value, expected,
            "Environment variable {} has unexpected value",
            key
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_environment() {
        let mut env = TestEnvironment::new("test-env").await;

        // Test file creation
        let file_path = env.create_test_file("test.txt", "Hello, World!");
        assert!(file_path.exists());

        // Test directory creation
        let dir_path = env.create_dir("test_dir");
        assert!(dir_path.exists() && dir_path.is_dir());

        // Test environment variables
        env.set_env_var("TEST_VAR", "test_value");
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
    }

    #[test]
    fn test_with_env_vars() {
        // Original value
        std::env::set_var("TEST_VAR", "original");

        let result = with_test_env_vars(vec![("TEST_VAR", "temporary")], || {
            std::env::var("TEST_VAR").unwrap()
        });

        assert_eq!(result, "temporary");
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "original");
    }

    #[test]
    fn test_create_context() {
        let context = create_test_context();
        assert!(context.data.contains_key("test_mode"));
        assert_eq!(context.data["test_mode"], json!(true));

        let custom_data = HashMap::from([("custom_key".to_string(), json!("custom_value"))]);
        let context_with_data = create_test_context_with_data(custom_data);
        assert!(context_with_data.data.contains_key("custom_key"));
    }

    #[tokio::test]
    async fn test_multiple_files() {
        let env = TestEnvironment::new("multi-file-test").await;

        let files = env.create_test_files(vec![
            ("file1.txt", "Content 1"),
            ("dir/file2.txt", "Content 2"),
            ("dir/subdir/file3.txt", "Content 3"),
        ]);

        assert_eq!(files.len(), 3);
        for file in files {
            assert!(file.exists());
        }
    }
}
