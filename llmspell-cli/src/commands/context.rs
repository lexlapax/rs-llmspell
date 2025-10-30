//! Context command implementation - sends context requests to kernel
//!
//! This module provides CLI commands for context assembly operations.
//! All context logic is executed in the kernel which has MemoryManager access.

use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{info, instrument, trace};

use crate::cli::{ContextCommands, OutputFormat};
use crate::execution_context::ExecutionContext;
use crate::output::OutputFormatter;
use llmspell_config::LLMSpellConfig;

/// Handle context assembly commands by sending requests to kernel
#[instrument(skip(runtime_config), fields(command_type))]
pub async fn handle_context_command(
    command: ContextCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling context command");

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
            handle_context_embedded(command, handle, config, output_format).await
        }
        ExecutionContext::Connected { handle, address } => {
            trace!("Using connected context at address: {}", address);
            handle_context_remote(command, handle, address, output_format).await
        }
    }
}

/// Handle context commands in embedded mode (kernel in same process)
async fn handle_context_embedded(
    command: ContextCommands,
    handle: Box<llmspell_kernel::api::KernelHandle>,
    _config: Box<LLMSpellConfig>,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling context command in embedded mode");

    // Use enum wrapper for consistency with remote handler
    let context_handle = ContextHandle::Kernel(handle);
    handle_context_with_ops(command, context_handle, output_format).await
}

/// Enum to abstract over KernelHandle and ClientHandle
enum ContextHandle {
    Kernel(Box<llmspell_kernel::api::KernelHandle>),
    Client(llmspell_kernel::api::ClientHandle),
}

impl ContextHandle {
    async fn send_context_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match self {
            ContextHandle::Kernel(handle) => handle.send_context_request(content).await,
            ContextHandle::Client(handle) => handle.send_context_request(content).await,
        }
    }
}

/// Handle context commands in remote mode (connected to external kernel)
async fn handle_context_remote(
    command: ContextCommands,
    handle: llmspell_kernel::api::ClientHandle,
    _address: String,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling context command in remote mode");

    // Use enum to unify KernelHandle and ClientHandle
    let context_handle = ContextHandle::Client(handle);
    handle_context_with_ops(command, context_handle, output_format).await
}

/// Generic handler that works with ContextHandle enum
async fn handle_context_with_ops(
    command: ContextCommands,
    mut handle: ContextHandle,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        ContextCommands::Assemble {
            query,
            strategy,
            budget,
            session_id,
            format,
        } => {
            info!("Assembling context via kernel");

            let request_content = json!({
                "command": "assemble",
                "query": query,
                "strategy": strategy.unwrap_or_else(|| "hybrid".to_string()),
                "budget": budget,
                "session_id": session_id,
            });

            let response = handle.send_context_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Context assemble error: {}", error));
            }

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&response)?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("\nContext Assembly Results");
                    println!("{}", "=".repeat(80));

                    if let Some(strategy) = response.get("strategy").and_then(|v| v.as_str()) {
                        println!("Strategy: {}", strategy);
                    }
                    if let Some(total) = response.get("total_chunks").and_then(|v| v.as_u64()) {
                        println!("Chunks:   {}", total);
                    }

                    if let Some(chunks) = response.get("chunks").and_then(|v| v.as_array()) {
                        println!("\n{}", "-".repeat(80));

                        for (idx, chunk) in chunks.iter().enumerate() {
                            println!("\n--- Chunk {} ---", idx + 1);

                            if let Some(chunk_type) = chunk.get("type").and_then(|v| v.as_str()) {
                                println!("Type:    {}", chunk_type);
                            }
                            if let Some(content) = chunk.get("content").and_then(|v| v.as_str()) {
                                let display_content = if content.len() > 300 {
                                    format!("{}...", &content[..300])
                                } else {
                                    content.to_string()
                                };
                                println!("Content: {}", display_content);
                            }
                            if let Some(role) = chunk.get("role").and_then(|v| v.as_str()) {
                                println!("Role:    {}", role);
                            }
                            if let Some(timestamp) = chunk.get("timestamp").and_then(|v| v.as_str())
                            {
                                println!("Time:    {}", timestamp);
                            }
                        }

                        println!();
                    }
                }
            }

            Ok(())
        }

        ContextCommands::Strategies { format } => {
            info!("Listing context strategies via kernel");

            let request_content = json!({
                "command": "strategies",
            });

            let response = handle.send_context_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Context strategies error: {}", error));
            }

            let strategies = response
                .as_array()
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&json!({"strategies": strategies}))?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("\nAvailable Context Strategies");
                    println!("{}", "=".repeat(80));

                    for strategy in strategies {
                        if let Some(name) = strategy.get("name").and_then(|v| v.as_str()) {
                            println!("\n  {}", name);
                            if let Some(desc) = strategy.get("description").and_then(|v| v.as_str())
                            {
                                println!("    {}", desc);
                            }
                        }
                    }

                    println!();
                }
            }

            Ok(())
        }

        ContextCommands::Analyze {
            query,
            budget,
            format,
        } => {
            info!("Analyzing context strategies via kernel");

            let request_content = json!({
                "command": "analyze",
                "query": query,
                "budget": budget,
            });

            let response = handle.send_context_request(request_content).await?;

            if let Some(error) = response.get("error") {
                return Err(anyhow!("Context analyze error: {}", error));
            }

            let analysis = response
                .as_array()
                .ok_or_else(|| anyhow!("Invalid response format"))?;

            let fmt = format.unwrap_or(output_format);
            let formatter = OutputFormatter::new(fmt);

            match fmt {
                OutputFormat::Json => {
                    formatter.print_json(&json!({"analysis": analysis}))?;
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("\nContext Strategy Analysis: \"{}\"", query);
                    println!("{}", "=".repeat(80));
                    println!("Token Budget: {}\n", budget);

                    for strategy in analysis {
                        if let Some(name) = strategy.get("strategy").and_then(|v| v.as_str()) {
                            println!("Strategy: {}", name);
                        }
                        if let Some(tokens) =
                            strategy.get("estimated_tokens").and_then(|v| v.as_u64())
                        {
                            println!("  Estimated tokens: {}", tokens);
                        }
                        if let Some(chunks) = strategy.get("chunks").and_then(|v| v.as_u64()) {
                            println!("  Chunks:           {}", chunks);
                        }
                        if let Some(message) = strategy.get("message").and_then(|v| v.as_str()) {
                            println!("  Note:             {}", message);
                        }
                        println!();
                    }
                }
            }

            Ok(())
        }
    }
}
