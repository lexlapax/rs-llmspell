//! ABOUTME: Common test fixtures and data for consistent testing
//! ABOUTME: Provides pre-configured test objects and sample data

//! Test fixtures and common test data.
//!
//! This module provides pre-configured test objects and sample data
//! that can be used across different test suites for consistency.
//!
//! # Examples
//!
//! ```rust
//! use llmspell_testing::fixtures::{
//!     sample_component_metadata,
//!     sample_agent_input,
//!     sample_workflow_steps,
//! };
//!
//! // Use pre-configured test data
//! let metadata = sample_component_metadata();
//! assert_eq!(metadata.name, "test-component");
//!
//! let input = sample_agent_input();
//! assert_eq!(input.text, "Test prompt");
//! ```

use llmspell_core::{
    execution_context::ExecutionContext,
    traits::{
        agent::{AgentConfig, ConversationMessage},
        tool::ToolSchema,
        workflow::{RetryPolicy, WorkflowConfig, WorkflowStep},
    },
    types::AgentInput,
    ComponentId, ComponentMetadata, Version,
};

#[cfg(test)]
#[cfg_attr(test_category = "testing")]
use llmspell_core::traits::agent::MessageRole;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Sample ComponentMetadata for testing
pub fn sample_component_metadata() -> ComponentMetadata {
    let mut metadata = ComponentMetadata::new(
        "test-component".to_string(),
        "A test component for unit testing".to_string(),
    );
    metadata.version = Version {
        major: 1,
        minor: 0,
        patch: 0,
    };
    metadata
}

/// Sample ComponentMetadata variants for different scenarios
pub fn component_metadata_variants() -> Vec<ComponentMetadata> {
    vec![
        // Minimal metadata
        ComponentMetadata::new(
            "minimal-component".to_string(),
            "Minimal test component".to_string(),
        ),
        // Full metadata
        sample_component_metadata(),
        // Version 2.0 component
        {
            let mut metadata = ComponentMetadata::new(
                "v2-component".to_string(),
                "Version 2 test component".to_string(),
            );
            metadata.version = Version {
                major: 2,
                minor: 0,
                patch: 0,
            };
            metadata
        },
    ]
}

/// Sample AgentInput for testing
pub fn sample_agent_input() -> AgentInput {
    AgentInput::text("Test prompt")
        .with_parameter("user_id", json!("test-user"))
        .with_parameter("session", json!("test-session"))
}

/// Sample AgentInput variants
pub fn agent_input_variants() -> Vec<AgentInput> {
    vec![
        // Simple input
        AgentInput::text("Simple prompt"),
        // Input with context
        sample_agent_input(),
        // Complex input
        AgentInput::text("Complex prompt with lots of detail")
            .with_parameter("history", json!(["previous", "messages"]))
            .with_parameter("temperature", json!(0.7))
            .with_parameter("max_tokens", json!(100))
            .with_parameter("priority", json!("high"))
            .with_parameter("timestamp", json!(1234567890)),
    ]
}

/// Sample ExecutionContext for testing
pub fn sample_execution_context() -> ExecutionContext {
    ExecutionContext::with_conversation("test-session-123".to_string())
        .with_data("user_id".to_string(), json!("test-user"))
        .with_data("LLMSPELL_ENV".to_string(), json!("test"))
}

/// Sample conversation for Agent testing
pub fn sample_conversation() -> Vec<ConversationMessage> {
    vec![
        ConversationMessage::system("You are a helpful assistant for testing.".to_string()),
        ConversationMessage::user("Hello, how are you?".to_string()),
        ConversationMessage::assistant(
            "I'm doing well, thank you! How can I help you today?".to_string(),
        ),
        ConversationMessage::user("Can you help me test something?".to_string()),
        ConversationMessage::assistant("Of course! I'd be happy to help you test.".to_string()),
    ]
}

/// Sample AgentConfig for testing
pub fn sample_agent_config() -> AgentConfig {
    AgentConfig {
        max_conversation_length: Some(100),
        system_prompt: Some("You are a test assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(500),
    }
}

/// Sample ToolSchema for testing
pub fn sample_tool_schema() -> ToolSchema {
    use llmspell_core::traits::tool::{ParameterDef, ParameterType};

    ToolSchema::new(
        "process_data".to_string(),
        "Process data with various options".to_string(),
    )
    .with_parameter(ParameterDef {
        name: "input".to_string(),
        param_type: ParameterType::String,
        description: "The input to process".to_string(),
        required: true,
        default: None,
    })
    .with_parameter(ParameterDef {
        name: "format".to_string(),
        param_type: ParameterType::String,
        description: "Output format".to_string(),
        required: false,
        default: Some(json!("json")),
    })
    .with_parameter(ParameterDef {
        name: "verbose".to_string(),
        param_type: ParameterType::Boolean,
        description: "Verbose output".to_string(),
        required: false,
        default: Some(json!(false)),
    })
    .with_returns(ParameterType::Object)
}

/// Sample workflow steps for testing
pub fn sample_workflow_steps() -> Vec<WorkflowStep> {
    let step1_id = ComponentId::from_name("step-1");
    let step2_id = ComponentId::from_name("step-2");
    let step3_id = ComponentId::from_name("step-3");

    vec![
        WorkflowStep {
            id: step1_id,
            name: "Initialize".to_string(),
            component_id: ComponentId::from_name("init-agent"),
            dependencies: vec![],
            retry_policy: Some(RetryPolicy::default()),
            timeout: Some(Duration::from_secs(30)),
        },
        WorkflowStep {
            id: step2_id,
            name: "Process Data".to_string(),
            component_id: ComponentId::from_name("processor-agent"),
            dependencies: vec![step1_id],
            retry_policy: Some(RetryPolicy {
                max_attempts: 5,
                backoff_seconds: 2,
                exponential_backoff: true,
            }),
            timeout: Some(Duration::from_secs(120)),
        },
        WorkflowStep {
            id: step3_id,
            name: "Generate Report".to_string(),
            component_id: ComponentId::from_name("reporter-agent"),
            dependencies: vec![step2_id],
            retry_policy: None,
            timeout: Some(Duration::from_secs(60)),
        },
    ]
}

/// Sample WorkflowConfig for testing
pub fn sample_workflow_config() -> WorkflowConfig {
    WorkflowConfig {
        max_parallel: Some(3),
        continue_on_error: false,
        timeout: Some(Duration::from_secs(600)),
    }
}

/// Create test error scenarios
pub fn error_scenarios() -> Vec<llmspell_core::LLMSpellError> {
    use llmspell_core::LLMSpellError;

    vec![
        // Component error
        LLMSpellError::Component {
            message: "Component initialization failed".to_string(),
            source: None,
        },
        // Validation error
        LLMSpellError::Validation {
            message: "Invalid input format".to_string(),
            field: Some("email".to_string()),
        },
        // Network error (retryable)
        LLMSpellError::Network {
            message: "Connection timeout".to_string(),
            source: None,
        },
        // Storage error
        LLMSpellError::Storage {
            message: "Database connection failed".to_string(),
            operation: Some("read".to_string()),
            source: None,
        },
    ]
}

/// Create a test environment setup
pub fn setup_test_environment() -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    env.insert("LLMSPELL_ENV".to_string(), "test".to_string());
    env.insert("LLMSPELL_LOG_LEVEL".to_string(), "debug".to_string());
    env.insert("LLMSPELL_LOG_FORMAT".to_string(), "json".to_string());
    env
}

// Fixture loading utilities

/// Get the path to the fixtures directory
pub fn fixtures_dir() -> PathBuf {
    // Try multiple strategies to find the fixtures directory
    let possible_paths = vec![
        // Use CARGO_MANIFEST_DIR if available (most reliable)
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|dir| {
                let manifest_path = PathBuf::from(dir);
                // Check if we're in llmspell-testing crate
                if manifest_path.ends_with("llmspell-testing") {
                    manifest_path.join("fixtures")
                } else {
                    // We're in another crate, go up to workspace root
                    manifest_path
                        .parent()
                        .map(|p| p.join("llmspell-testing/fixtures"))
                        .unwrap_or_else(|| manifest_path.join("fixtures"))
                }
            })
            .unwrap_or_default(),
        // When running from the crate root
        PathBuf::from("fixtures"),
        // When running from workspace root
        PathBuf::from("llmspell-testing/fixtures"),
        // When running tests from within the crate
        PathBuf::from("../fixtures"),
        // Look for workspace root marker
        std::env::current_dir()
            .ok()
            .and_then(|mut dir| {
                // Look for Cargo.toml with workspace marker
                loop {
                    let cargo_path = dir.join("Cargo.toml");
                    if cargo_path.exists() {
                        // Check if this is the workspace root
                        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                            if content.contains("[workspace]") {
                                return Some(dir.join("llmspell-testing/fixtures"));
                            }
                        }
                    }
                    if !dir.pop() {
                        break;
                    }
                }
                None
            })
            .unwrap_or_default(),
    ];

    for path in possible_paths {
        if path.exists() && path.is_dir() {
            return path;
        }
    }

    // Fallback: assume we're in the crate directory
    PathBuf::from("fixtures")
}

/// Get the path to a fixture file
pub fn fixture_path(relative_path: impl AsRef<Path>) -> PathBuf {
    fixtures_dir().join(relative_path)
}

/// Load a text fixture file
pub fn load_fixture_text(relative_path: impl AsRef<Path>) -> Result<String, std::io::Error> {
    let path = fixture_path(relative_path);
    std::fs::read_to_string(path)
}

/// Load a JSON fixture file
pub fn load_fixture_json(
    relative_path: impl AsRef<Path>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let content = load_fixture_text(relative_path)?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

/// Load a Lua fixture file
pub fn load_fixture_lua(filename: impl AsRef<Path>) -> Result<String, std::io::Error> {
    load_fixture_text(Path::new("lua").join(filename))
}

/// Load test data from the data directory
pub fn load_test_data(filename: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
    let path = fixture_path(Path::new("data").join(filename));
    std::fs::read(path)
}

/// Create a temporary test file in the fixtures directory
pub fn create_temp_fixture(filename: &str, content: &str) -> Result<PathBuf, std::io::Error> {
    let temp_dir = fixtures_dir().join("temp");
    std::fs::create_dir_all(&temp_dir)?;
    let path = temp_dir.join(filename);
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Clean up temporary test files
pub fn cleanup_temp_fixtures() -> Result<(), std::io::Error> {
    let temp_dir = fixtures_dir().join("temp");
    if temp_dir.exists() {
        std::fs::remove_dir_all(temp_dir)?;
    }
    Ok(())
}

/// List all files in a fixture subdirectory
pub fn list_fixture_files(subdir: impl AsRef<Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    let dir = fixtures_dir().join(subdir);
    let mut files = Vec::new();

    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
#[cfg_attr(test_category = "testing")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_sample_fixtures() {
        // Test metadata fixture
        let metadata = sample_component_metadata();
        assert_eq!(metadata.name, "test-component");
        assert_eq!(metadata.version.major, 1);

        // Test input fixture
        let input = sample_agent_input();
        assert_eq!(input.text, "Test prompt");
        assert!(!input.parameters.is_empty());

        // Test conversation fixture
        let conversation = sample_conversation();
        assert_eq!(conversation.len(), 5);
        assert_eq!(conversation[0].role, MessageRole::System);

        // Test workflow steps
        let steps = sample_workflow_steps();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[1].dependencies.len(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_fixture_variants() {
        let metadata_variants = component_metadata_variants();
        assert_eq!(metadata_variants.len(), 3);

        let input_variants = agent_input_variants();
        assert_eq!(input_variants.len(), 3);

        let errors = error_scenarios();
        assert!(errors.len() >= 4);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_fixtures_dir() {
        let dir = fixtures_dir();
        // Should return a path (even if it doesn't exist yet)
        assert!(!dir.as_os_str().is_empty());

        // The directory should exist in our test environment
        assert!(
            dir.exists(),
            "Fixtures directory should exist at: {:?}",
            dir
        );
        assert!(
            dir.is_dir(),
            "Fixtures path should be a directory: {:?}",
            dir
        );

        // Should contain expected subdirectories
        let lua_dir = dir.join("lua");
        let data_dir = dir.join("data");
        assert!(
            lua_dir.exists(),
            "Expected lua subdirectory at: {:?}",
            lua_dir
        );
        assert!(
            data_dir.exists(),
            "Expected data subdirectory at: {:?}",
            data_dir
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_fixture_path() {
        let path = fixture_path("test.txt");
        assert!(path.ends_with("test.txt"));
        assert!(path.to_string_lossy().contains("fixtures"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_load_fixture_json() {
        // Test with existing migration test data
        let result = load_fixture_json("data/migration_test_cases/v1_to_v2_user_schema.json");
        if result.is_ok() {
            let json = result.unwrap();
            assert!(json.is_object());
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_temp_fixtures() {
        // Create a temp fixture
        let content = "test content";
        let result = create_temp_fixture("test_temp.txt", content);

        if let Ok(path) = result {
            // Verify it was created
            assert!(path.exists());

            // Read it back
            let read_content = std::fs::read_to_string(&path).unwrap();
            assert_eq!(read_content, content);

            // Clean up
            let _ = cleanup_temp_fixtures();

            // Verify cleanup worked
            assert!(!path.exists());
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_list_fixture_files() {
        let lua_files = list_fixture_files("lua");
        if let Ok(files) = lua_files {
            // We know we have lua fixtures
            if !files.is_empty() {
                assert!(files
                    .iter()
                    .any(|p| p.extension().and_then(|s| s.to_str()) == Some("lua")));
            }
        }
    }
}
