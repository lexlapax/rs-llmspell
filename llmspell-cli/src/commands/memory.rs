//! Memory command implementation - sends memory requests to kernel
//!
//! This module provides CLI commands for memory operations (episodic and semantic).
//! All memory logic is executed in the kernel which has MemoryManager access.

use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{info, instrument, trace};

use crate::cli::{MemoryCommands, OutputFormat};
use crate::execution_context::ExecutionContext;
use crate::output::OutputFormatter;
use llmspell_config::LLMSpellConfig;

/// Handle memory management commands by sending requests to kernel
#[instrument(skip(runtime_config), fields(command_type))]
pub async fn handle_memory_command(
    command: MemoryCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling memory command");

    // Resolve execution context (connects to kernel or creates embedded)
    let context = ExecutionContext::resolve(
        None, // No connect string
        None, // No port
        None, // No daemon config
        runtime_config.clone(),
    )
    .await?;

    match context {
        ExecutionContext::Embedded { handle, config } => {
            trace!("Using embedded context");
            handle_memory_embedded(command, handle, config, output_format).await
        }
        ExecutionContext::Connected { handle, address } => {
            trace!("Using connected context at address: {}", address);
            handle_memory_remote(command, handle, address, output_format).await
        }
    }
}

/// Enum to abstract over KernelHandle and ClientHandle
enum MemoryHandle {
    Kernel(Box<llmspell_kernel::api::KernelHandle>),
    Client(llmspell_kernel::api::ClientHandle),
}

impl MemoryHandle {
    async fn send_memory_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match self {
            MemoryHandle::Kernel(handle) => handle.send_memory_request(content).await,
            MemoryHandle::Client(handle) => handle.send_memory_request(content).await,
        }
    }
}

/// Handle memory commands in embedded mode (kernel in same process)
async fn handle_memory_embedded(
    command: MemoryCommands,
    handle: Box<llmspell_kernel::api::KernelHandle>,
    _config: Box<LLMSpellConfig>,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling memory command in embedded mode");

    // Use enum wrapper for consistency with remote handler
    let memory_handle = MemoryHandle::Kernel(handle);
    handle_memory_with_ops(command, memory_handle, output_format).await
}

/// Handle memory commands in remote mode (connected to external kernel)
async fn handle_memory_remote(
    command: MemoryCommands,
    handle: llmspell_kernel::api::ClientHandle,
    _address: String,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling memory command in remote mode");

    // Use enum to unify KernelHandle and ClientHandle
    let memory_handle = MemoryHandle::Client(handle);
    handle_memory_with_ops(command, memory_handle, output_format).await
}

/// Generic handler that works with MemoryHandle enum
async fn handle_memory_with_ops(
    command: MemoryCommands,
    mut handle: MemoryHandle,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        MemoryCommands::Add {
            session_id,
            role,
            content,
            metadata,
        } => {
            info!("Adding episodic memory entry via kernel");

            let metadata_value = if let Some(meta_str) = metadata {
                serde_json::from_str(&meta_str)
                    .map_err(|e| anyhow!("Invalid metadata JSON: {}", e))?
            } else {
                json!({})
            };

            let request_content = json!({
                "command": "add",
                "session_id": session_id,
                "role": role,
                "content": content,
                "metadata": metadata_value,
            });

            let response = handle.send_memory_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Memory add error: {}", error));
            }

            let formatter = OutputFormatter::new(output_format);

            match output_format {
                OutputFormat::Json => {
                    formatter.print_json(&response)?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if let Some(entry_id) = response.get("entry_id").and_then(|v| v.as_str()) {
                        println!("✓ Added memory entry: {}", entry_id);
                    } else {
                        println!("✓ Memory entry added successfully");
                    }
                }
            }

            Ok(())
        }

        MemoryCommands::Search {
            query,
            session_id,
            limit,
            format,
        } => {
            info!("Searching episodic memory via kernel");

            let request_content = json!({
                "command": "search",
                "query": query,
                "session_id": session_id,
                "limit": limit,
            });

            let response = handle.send_memory_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Memory search error: {}", error));
            }

            let results = response
                .as_array()
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&json!({"results": results}))?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if results.is_empty() {
                        println!("No memory entries found matching query: {}", query);
                    } else {
                        println!("\nMemory Search Results: \"{}\"", query);
                        println!("{}", "=".repeat(80));
                        println!("Found {} entries", results.len());
                        println!();

                        for (idx, entry) in results.iter().enumerate() {
                            println!("--- Entry {} ---", idx + 1);

                            if let Some(id) = entry.get("id").and_then(|v| v.as_str()) {
                                println!("ID:        {}", id);
                            }
                            if let Some(session) = entry.get("session_id").and_then(|v| v.as_str())
                            {
                                println!("Session:   {}", session);
                            }
                            if let Some(role) = entry.get("role").and_then(|v| v.as_str()) {
                                println!("Role:      {}", role);
                            }
                            if let Some(timestamp) = entry.get("timestamp").and_then(|v| v.as_str())
                            {
                                println!("Time:      {}", timestamp);
                            }
                            if let Some(content) = entry.get("content").and_then(|v| v.as_str()) {
                                let display_content = if content.len() > 200 {
                                    format!("{}...", &content[..200])
                                } else {
                                    content.to_string()
                                };
                                println!("Content:   {}", display_content);
                            }
                            println!();
                        }
                    }
                }
            }

            Ok(())
        }

        MemoryCommands::Query {
            query,
            limit,
            format,
        } => {
            info!("Querying semantic knowledge graph via kernel");

            let request_content = json!({
                "command": "query",
                "query": query,
                "limit": limit,
            });

            let response = handle.send_memory_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Memory query error: {}", error));
            }

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&response)?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    if let Some(message) = response.get("message") {
                        println!("{}", message.as_str().unwrap_or("Query completed"));
                    }

                    if let Some(entities) = response.get("entities").and_then(|v| v.as_array()) {
                        if entities.is_empty() {
                            println!("No entities found matching query: {}", query);
                        } else {
                            println!("\nSemantic Memory Query: \"{}\"", query);
                            println!("{}", "=".repeat(80));
                            println!("Found {} entities\n", entities.len());

                            for (idx, entity) in entities.iter().enumerate() {
                                println!("--- Entity {} ---", idx + 1);

                                if let Some(name) = entity.get("name").and_then(|v| v.as_str()) {
                                    println!("Name:      {}", name);
                                }
                                if let Some(entity_type) =
                                    entity.get("entity_type").and_then(|v| v.as_str())
                                {
                                    println!("Type:      {}", entity_type);
                                }
                                println!();
                            }
                        }
                    }
                }
            }

            Ok(())
        }

        MemoryCommands::Stats { format } => {
            info!("Getting memory statistics via kernel");

            let request_content = json!({
                "command": "stats",
            });

            let response = handle.send_memory_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Memory stats error: {}", error));
            }

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&response)?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("\nMemory System Statistics");
                    println!("{}", "=".repeat(80));

                    if let Some(episodic) = response.get("episodic") {
                        println!("\nEpisodic Memory:");
                        if let Some(sessions) = episodic
                            .get("sessions_with_unprocessed")
                            .and_then(|v| v.as_u64())
                        {
                            println!("  Sessions with unprocessed entries: {}", sessions);
                        }
                        if let Some(session_list) =
                            episodic.get("sessions").and_then(|v| v.as_array())
                        {
                            if !session_list.is_empty() {
                                println!("  Active sessions:");
                                for session in session_list {
                                    if let Some(sid) = session.as_str() {
                                        println!("    - {}", sid);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(semantic) = response.get("semantic") {
                        println!("\nSemantic Memory:");
                        if let Some(message) = semantic.get("message") {
                            println!("  {}", message.as_str().unwrap_or(""));
                        }
                    }

                    if let Some(consolidation) = response.get("consolidation") {
                        println!("\nConsolidation:");
                        if let Some(enabled) =
                            consolidation.get("enabled").and_then(|v| v.as_bool())
                        {
                            println!("  Enabled: {}", if enabled { "yes" } else { "no" });
                        }
                    }

                    println!();
                }
            }

            Ok(())
        }

        MemoryCommands::Consolidate { session_id, force } => {
            info!("Triggering memory consolidation via kernel");

            let request_content = json!({
                "command": "consolidate",
                "session_id": session_id,
                "force": force,
            });

            let response = handle.send_memory_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Memory consolidate error: {}", error));
            }

            let formatter = OutputFormatter::new(output_format);

            match output_format {
                OutputFormat::Json => {
                    formatter.print_json(&response)?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("\nMemory Consolidation Results");
                    println!("{}", "=".repeat(80));

                    if let Some(entries_processed) =
                        response.get("entries_processed").and_then(|v| v.as_u64())
                    {
                        println!("Entries processed:    {}", entries_processed);
                    }
                    if let Some(entities_added) =
                        response.get("entities_added").and_then(|v| v.as_u64())
                    {
                        println!("Entities added:       {}", entities_added);
                    }
                    if let Some(entities_updated) =
                        response.get("entities_updated").and_then(|v| v.as_u64())
                    {
                        println!("Entities updated:     {}", entities_updated);
                    }
                    if let Some(entities_deleted) =
                        response.get("entities_deleted").and_then(|v| v.as_u64())
                    {
                        println!("Entities deleted:     {}", entities_deleted);
                    }
                    if let Some(entries_skipped) =
                        response.get("entries_skipped").and_then(|v| v.as_u64())
                    {
                        println!("Entries skipped:      {}", entries_skipped);
                    }
                    if let Some(entries_failed) =
                        response.get("entries_failed").and_then(|v| v.as_u64())
                    {
                        println!("Entries failed:       {}", entries_failed);
                    }
                    if let Some(duration_ms) = response.get("duration_ms").and_then(|v| v.as_u64())
                    {
                        println!("Duration:             {}ms", duration_ms);
                    }

                    println!("\n✓ Consolidation completed");
                }
            }

            Ok(())
        }
    }
}
