//! Error handling tests for llmspell-memory
//!
//! Tests all 10 `MemoryError` variants, conversions, and error propagation.

use llmspell_memory::prelude::*;
use std::io;

/// Test that all `MemoryError` variants can be instantiated
#[test]
fn test_all_error_variants() {
    // Storage error
    let err = MemoryError::Storage("test storage error".into());
    assert!(err.to_string().contains("Storage error"));

    // VectorSearch error
    let err = MemoryError::VectorSearch("vector search failed".into());
    assert!(err.to_string().contains("Vector search error"));

    // KnowledgeGraph error
    let err = MemoryError::KnowledgeGraph("graph error".into());
    assert!(err.to_string().contains("Knowledge graph error"));

    // Consolidation error
    let err = MemoryError::Consolidation("consolidation failed".into());
    assert!(err.to_string().contains("Consolidation error"));

    // NotFound error
    let err = MemoryError::NotFound("entry not found".into());
    assert!(err.to_string().contains("Entry not found"));

    // InvalidInput error
    let err = MemoryError::InvalidInput("invalid input".into());
    assert!(err.to_string().contains("Invalid input"));

    // Other error
    let err = MemoryError::Other("other error".into());
    assert!(err.to_string().contains("other error"));

    // Serialization error (from serde_json::Error)
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();
    let err = MemoryError::from(json_err);
    assert!(err.to_string().contains("Serialization error"));

    // IO error
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = MemoryError::from(io_err);
    assert!(err.to_string().contains("IO error"));

    // Core error (requires llmspell_core::LLMSpellError)
    // Tested separately in integration tests
}

/// Test From<String> and From<&str> conversions
#[test]
fn test_error_from_conversions() {
    // From<String>
    let err: MemoryError = "error from string".to_string().into();
    assert!(matches!(err, MemoryError::Other(_)));
    assert_eq!(err.to_string(), "error from string");

    // From<&str>
    let err: MemoryError = "error from &str".into();
    assert!(matches!(err, MemoryError::Other(_)));
    assert_eq!(err.to_string(), "error from &str");
}

/// Test error Display messages are correctly formatted
#[test]
fn test_error_display_messages() {
    let test_cases = vec![
        (
            MemoryError::Storage("database connection failed".into()),
            "Storage error: database connection failed",
        ),
        (
            MemoryError::VectorSearch("similarity search timeout".into()),
            "Vector search error: similarity search timeout",
        ),
        (
            MemoryError::KnowledgeGraph("node not found".into()),
            "Knowledge graph error: node not found",
        ),
        (
            MemoryError::Consolidation("LLM extraction failed".into()),
            "Consolidation error: LLM extraction failed",
        ),
        (
            MemoryError::NotFound("entry-123".into()),
            "Entry not found: entry-123",
        ),
        (
            MemoryError::InvalidInput("empty session ID".into()),
            "Invalid input: empty session ID",
        ),
        (
            MemoryError::Other("unexpected error".into()),
            "unexpected error",
        ),
    ];

    for (error, expected_msg) in test_cases {
        assert_eq!(error.to_string(), expected_msg);
    }
}

/// Test error propagation through async boundaries
#[tokio::test]
async fn test_error_propagation() {
    // Define nested function before statements to avoid clippy warning
    async fn nested_get(memory: &InMemoryEpisodicMemory, id: &str) -> Result<EpisodicEntry> {
        memory.get(id).await
    }

    let memory = InMemoryEpisodicMemory::new();

    // Test NotFound error propagation
    let result = memory.get("nonexistent-id").await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, MemoryError::NotFound(_)));
    assert!(err.to_string().contains("Entry not found"));

    // Error should propagate through multiple async layers
    let result = nested_get(&memory, "another-nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MemoryError::NotFound(_)));
}

/// Test `serde_json::Error` conversion to `MemoryError::Serialization`
#[test]
fn test_serialization_error() {
    // Create an invalid JSON string
    let invalid_json = "{\"key\": invalid_value}";

    // Attempt to deserialize
    let result: std::result::Result<EpisodicEntry, serde_json::Error> =
        serde_json::from_str(invalid_json);

    assert!(result.is_err());

    // Convert to MemoryError
    let json_err = result.unwrap_err();
    let memory_err = MemoryError::from(json_err);

    // Verify it's the Serialization variant
    assert!(matches!(memory_err, MemoryError::Serialization(_)));
    assert!(memory_err.to_string().contains("Serialization error"));
}

/// Test core error conversion (`llmspell_core::LLMSpellError` → `MemoryError::Core`)
#[test]
fn test_core_error_conversion() {
    use llmspell_core::LLMSpellError;

    // Create a core error (using Component variant)
    let core_err = LLMSpellError::Component {
        message: "test component error".into(),
        source: None,
    };

    // Convert to MemoryError
    let memory_err = MemoryError::from(core_err);

    // Verify it's the Core variant
    assert!(matches!(memory_err, MemoryError::Core(_)));
    assert!(memory_err.to_string().contains("Core error"));
}
