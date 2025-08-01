//! ABOUTME: Example demonstrating how users can store and manage artifacts in sessions
//! ABOUTME: Shows file uploads, metadata management, and artifact retrieval patterns

use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_sessions::{
    types::CreateSessionOptions, ArtifactType, SessionManager, SessionManagerConfig,
};
use llmspell_state_persistence::StateManager;
use llmspell_storage::MemoryBackend;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (optional - requires tracing-subscriber dependency)
    // tracing_subscriber::fmt()
    //     .with_env_filter("llmspell=debug")
    //     .init();

    // Create dependencies
    let state_manager = Arc::new(StateManager::new().await?);
    let storage_backend = Arc::new(MemoryBackend::new());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());

    // Configure session manager with artifact collection enabled
    let mut config = SessionManagerConfig::default();
    config.hook_config.enable_artifact_collection = true;

    // Create session manager
    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        config,
    )?;

    // Create a new session
    let session_options = CreateSessionOptions {
        name: Some("Data Processing Session".to_string()),
        description: Some("Session for processing user datasets".to_string()),
        tags: vec!["data-processing".to_string(), "example".to_string()],
        ..Default::default()
    };
    let session_id = session_manager.create_session(session_options).await?;
    println!("Created session: {}", session_id);

    // Example 1: Store a simple text artifact
    println!("\n--- Example 1: Simple Text Artifact ---");
    let text_content = b"This is my important note that I want to save.".to_vec();
    let text_artifact_id = session_manager
        .store_artifact(
            &session_id,
            ArtifactType::UserInput,
            "my_note.txt".to_string(),
            text_content.clone(),
            None,
        )
        .await?;
    println!("Stored text artifact: {}", text_artifact_id);

    // Retrieve and display the text artifact
    let retrieved_artifact = session_manager
        .get_artifact(&session_id, &text_artifact_id)
        .await?;
    println!("Retrieved artifact: {}", retrieved_artifact.metadata.name);
    println!(
        "Content: {}",
        String::from_utf8_lossy(&retrieved_artifact.get_content()?)
    );

    // Example 2: Store a JSON artifact with metadata
    println!("\n--- Example 2: JSON Artifact with Metadata ---");
    let json_content = serde_json::json!({
        "data": [1, 2, 3, 4, 5],
        "mean": 3.0,
        "stddev": 1.58
    });
    let json_bytes = serde_json::to_vec_pretty(&json_content)?;

    // Create metadata for the artifact
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), serde_json::json!("json"));
    metadata.insert("schema_version".to_string(), serde_json::json!("1.0"));
    metadata.insert(
        "processing_info".to_string(),
        serde_json::json!({
            "algorithm": "basic_stats",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    );

    let json_artifact_id = session_manager
        .store_artifact(
            &session_id,
            ArtifactType::SystemGenerated,
            "statistics.json".to_string(),
            json_bytes,
            Some(metadata),
        )
        .await?;
    println!("Stored JSON artifact: {}", json_artifact_id);

    // Example 3: Store a file from disk
    println!("\n--- Example 3: File Upload ---");
    // Create a temporary file
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join("dataset.csv");
    let csv_content = "name,age,city\nAlice,30,New York\nBob,25,San Francisco\nCarol,35,Chicago\n";
    std::fs::write(&file_path, csv_content)?;

    // Store the file as an artifact
    let file_artifact_id = session_manager
        .store_file_artifact(&session_id, &file_path, ArtifactType::UserInput, None)
        .await?;
    println!("Stored file artifact: {}", file_artifact_id);

    // Example 4: List all artifacts in the session
    println!("\n--- Example 4: List All Artifacts ---");
    let artifacts = session_manager.list_artifacts(&session_id).await?;
    println!("Total artifacts in session: {}", artifacts.len());
    for artifact in &artifacts {
        println!(
            "  - {} ({:?}, {} bytes)",
            artifact.name, artifact.artifact_type, artifact.size
        );
        if !artifact.custom.is_empty() {
            println!("    Custom metadata: {:?}", artifact.custom);
        }
    }

    // Example 5: Query artifacts
    println!("\n--- Example 5: Query Artifacts ---");
    use llmspell_sessions::ArtifactQuery;
    let query = ArtifactQuery {
        session_id: Some(session_id),
        artifact_type: Some(ArtifactType::UserInput),
        ..Default::default()
    };
    let user_artifacts = session_manager.query_artifacts(query).await?;
    println!("Found {} UserInput artifacts", user_artifacts.len());

    // Example 6: Building a knowledge base
    println!("\n--- Example 6: Building a Knowledge Base ---");
    // Store multiple related documents
    let documents = vec![
        (
            "README.md",
            "# Project Documentation\nThis is the main readme.",
        ),
        ("API.md", "# API Reference\nDescribes the API endpoints."),
        (
            "CONTRIBUTING.md",
            "# Contributing Guide\nHow to contribute to this project.",
        ),
    ];

    for (filename, content) in documents {
        let mut doc_metadata = HashMap::new();
        doc_metadata.insert("doc_type".to_string(), serde_json::json!("markdown"));
        doc_metadata.insert("category".to_string(), serde_json::json!("documentation"));

        session_manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                filename.to_string(),
                content.as_bytes().to_vec(),
                Some(doc_metadata),
            )
            .await?;
        println!("Added {} to knowledge base", filename);
    }

    // Example 7: Clean up - delete an artifact
    println!("\n--- Example 7: Delete Artifact ---");
    session_manager
        .delete_artifact(&session_id, &text_artifact_id)
        .await?;
    println!("Deleted artifact: {}", text_artifact_id);

    // Verify deletion
    match session_manager
        .get_artifact(&session_id, &text_artifact_id)
        .await
    {
        Ok(_) => println!("ERROR: Artifact still exists!"),
        Err(_) => println!("Confirmed: Artifact was deleted"),
    }

    // Complete the session
    session_manager.complete_session(&session_id).await?;
    println!("\nSession completed successfully!");

    Ok(())
}
