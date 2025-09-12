//! Session management commands
//!
//! Provides CLI commands for managing sessions using the SessionManager
//! from llmspell-sessions crate.

use crate::cli::{ExportFormat, OutputFormat, SessionCommands};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_sessions::{
    CreateSessionOptions, SessionId, SessionManager, SessionManagerConfig, SessionQuery,
};
use std::path::PathBuf;
use std::str::FromStr;

/// Handle session management commands
pub async fn handle_session_command(
    command: SessionCommands,
    config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    // Create session manager using the configuration
    let session_config = SessionManagerConfig::default();

    // Create required dependencies for SessionManager
    let state_manager = std::sync::Arc::new(
        llmspell_state_persistence::StateManager::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create state manager: {}", e))?,
    );

    // Create storage backend based on configuration
    let storage_backend: std::sync::Arc<dyn llmspell_storage::StorageBackend> =
        match config.runtime.sessions.storage_backend.as_str() {
            "sled" => {
                // Use sled backend for persistent storage
                let sled_backend = llmspell_storage::SledBackend::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create sled backend: {}", e))?;
                std::sync::Arc::new(sled_backend)
            }
            "memory" => std::sync::Arc::new(llmspell_storage::MemoryBackend::new()),
            _ => {
                // Default to memory backend for unknown types
                std::sync::Arc::new(llmspell_storage::MemoryBackend::new())
            }
        };

    let hook_registry = std::sync::Arc::new(llmspell_hooks::registry::HookRegistry::new());
    let hook_executor = std::sync::Arc::new(llmspell_hooks::executor::HookExecutor::new());
    let event_bus = std::sync::Arc::new(llmspell_events::bus::EventBus::new());

    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        session_config,
    )?;

    match command {
        SessionCommands::Create { name, description } => {
            create_session(&session_manager, name, description, output_format).await
        }
        SessionCommands::List { detailed } => {
            list_sessions(&session_manager, detailed, output_format).await
        }
        SessionCommands::Show { id } => show_session(&session_manager, id, output_format).await,
        SessionCommands::Replay {
            id,
            from_step,
            to_step,
        } => replay_session(&session_manager, id, from_step, to_step, output_format).await,
        SessionCommands::Delete { id, all } => delete_session(&session_manager, id, all).await,
        SessionCommands::Export { id, file, format } => {
            export_session(&session_manager, id, file, format).await
        }
    }
}

/// Create a new session
async fn create_session(
    session_manager: &SessionManager,
    name: String,
    description: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    let options = CreateSessionOptions::builder()
        .name(name.clone())
        .description(
            description
                .clone()
                .unwrap_or_else(|| format!("Session {}", name)),
        )
        .build();

    let session_id = session_manager.create_session(options).await?;

    match output_format {
        OutputFormat::Json => {
            let result = serde_json::json!({
                "id": session_id.to_string(),
                "name": name,
                "description": description,
                "status": "created"
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("✓ Created session: {}", session_id);
        }
    }

    Ok(())
}

/// List all sessions
async fn list_sessions(
    session_manager: &SessionManager,
    detailed: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let sessions = session_manager
        .list_sessions(SessionQuery::default())
        .await?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&sessions)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            if sessions.is_empty() {
                println!("No sessions found");
            } else {
                println!("Sessions ({} total):", sessions.len());
                for session in sessions {
                    if detailed {
                        println!("\nID: {}", session.id);
                        println!("  Name: {}", session.name.as_deref().unwrap_or("<unnamed>"));
                        println!("  Status: {:?}", session.status);
                        println!("  Created: {}", session.created_at);
                        if let Some(desc) = &session.description {
                            println!("  Description: {}", desc);
                        }
                    } else {
                        let status_str = format!("{:?}", session.status);
                        println!(
                            "  {} - {} ({})",
                            session.id,
                            session.name.as_deref().unwrap_or("<unnamed>"),
                            status_str
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Show session details
async fn show_session(
    session_manager: &SessionManager,
    id: String,
    output_format: OutputFormat,
) -> Result<()> {
    let session_id = SessionId::from_str(&id)?;
    let session = session_manager.get_session(&session_id).await?;

    let metadata = session.metadata.read().await;

    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&*metadata)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("Session Details:");
            println!("  ID: {}", metadata.id);
            println!(
                "  Name: {}",
                metadata.name.as_deref().unwrap_or("<unnamed>")
            );
            println!("  Status: {:?}", metadata.status);
            println!("  Created: {}", metadata.created_at);
            if let Some(desc) = &metadata.description {
                println!("  Description: {}", desc);
            }
            if !metadata.tags.is_empty() {
                println!("  Tags: {}", metadata.tags.join(", "));
            }
        }
    }

    Ok(())
}

/// Replay session history
async fn replay_session(
    session_manager: &SessionManager,
    id: String,
    from_step: Option<usize>,
    to_step: Option<usize>,
    output_format: OutputFormat,
) -> Result<()> {
    let session_id = SessionId::from_str(&id)?;

    // Check if session can be replayed
    if !session_manager.can_replay_session(&session_id).await? {
        anyhow::bail!("Session {} cannot be replayed", session_id);
    }

    // Configure replay
    let mode = llmspell_hooks::replay::ReplayMode::Exact;
    let config = llmspell_sessions::replay::session_adapter::SessionReplayConfig {
        mode,
        target_timestamp: None,
        compare_results: true,
        timeout: std::time::Duration::from_secs(300), // 5 minute timeout
        stop_on_error: true,
        metadata: std::collections::HashMap::new(),
    };

    println!("Replaying session {}...", session_id);
    if let Some(from) = from_step {
        println!("  Starting from step: {}", from);
    }
    if let Some(to) = to_step {
        println!("  Ending at step: {}", to);
    }

    let result = session_manager.replay_session(&session_id, config).await?;

    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("\nReplay completed:");
            println!("  Hooks replayed: {}", result.hooks_replayed);
            println!("  Successful: {}", result.successful_replays);
            println!("  Failed: {}", result.failed_replays);
            println!("  Duration: {:?}", result.total_duration);
        }
    }

    Ok(())
}

/// Delete session
async fn delete_session(session_manager: &SessionManager, id: String, all: bool) -> Result<()> {
    if all {
        // Get all sessions and delete them
        let sessions = session_manager
            .list_sessions(SessionQuery::default())
            .await?;

        let count = sessions.len();
        for session in sessions {
            session_manager.delete_session(&session.id).await?;
        }

        println!("✓ Deleted {} sessions", count);
    } else {
        let session_id = SessionId::from_str(&id)?;
        session_manager.delete_session(&session_id).await?;
        println!("✓ Deleted session: {}", id);
    }

    Ok(())
}

/// Export session to file
async fn export_session(
    session_manager: &SessionManager,
    id: String,
    file: PathBuf,
    format: ExportFormat,
) -> Result<()> {
    let session_id = SessionId::from_str(&id)?;
    let session = session_manager.get_session(&session_id).await?;

    // Get session metadata and timeline for export
    let metadata = session.metadata.read().await.clone();
    let timeline = session_manager.get_session_timeline(&session_id).await?;

    let export_data = serde_json::json!({
        "metadata": metadata,
        "timeline": timeline,
        "config": session.config,
    });

    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&export_data)?,
        ExportFormat::Yaml => serde_yaml::to_string(&export_data)?,
        ExportFormat::Toml => toml::to_string_pretty(&export_data)?,
    };

    tokio::fs::write(&file, content).await?;
    println!("✓ Exported session {} to {}", id, file.display());

    Ok(())
}
