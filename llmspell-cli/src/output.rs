//! ABOUTME: Output formatting utilities for different output modes
//! ABOUTME: Handles text, JSON, and pretty-printed output formats

use crate::cli::OutputFormat;
use anyhow::Result;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use llmspell_bridge::engine::{ScriptOutput, ScriptStream};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::Mutex;

/// Format script output according to the specified format
pub fn format_output(output: &ScriptOutput, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&output.output)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(&output.output)?),
        OutputFormat::Text => {
            // Simple text representation
            match output.output {
                serde_json::Value::String(ref s) => Ok(s.clone()),
                _ => Ok(output.output.to_string()),
            }
        }
        OutputFormat::Pretty => {
            // Pretty-printed output with metadata
            let mut result = String::new();

            // Add console output if any
            if !output.console_output.is_empty() {
                result.push_str("Console Output:\n");
                for line in &output.console_output {
                    result.push_str(&format!("  {}\n", line));
                }
                result.push('\n');
            }

            result.push_str(&format!(
                "Output: {}\n",
                serde_json::to_string_pretty(&output.output)?
            ));

            // Add metadata
            result.push_str(&format!("Engine: {}\n", output.metadata.engine));
            result.push_str(&format!(
                "Execution time: {}ms\n",
                output.metadata.execution_time_ms
            ));

            if !output.metadata.warnings.is_empty() {
                result.push_str("\nWarnings:\n");
                for warning in &output.metadata.warnings {
                    result.push_str(&format!("  ⚠ {}\n", warning));
                }
            }

            Ok(result)
        }
    }
}

/// Print streaming output with the specified format
pub async fn print_stream(stream: &mut ScriptStream, format: OutputFormat) -> Result<()> {
    // Set up signal handler for graceful shutdown
    let interrupted = Arc::new(Mutex::new(false));
    let interrupted_clone = interrupted.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        *interrupted_clone.lock().await = true;
    });

    match format {
        OutputFormat::Json => print_stream_json(stream, interrupted).await,
        OutputFormat::Yaml => print_stream_yaml(stream, interrupted).await,
        OutputFormat::Text => print_stream_text(stream, false, interrupted).await,
        OutputFormat::Pretty => print_stream_text(stream, true, interrupted).await,
    }
}

/// Print streaming output as JSON
async fn print_stream_json(stream: &mut ScriptStream, interrupted: Arc<Mutex<bool>>) -> Result<()> {
    // Show progress while collecting chunks
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Collecting stream data...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut chunks = Vec::new();
    while let Some(chunk) = stream.stream.next().await {
        // Check if interrupted
        if *interrupted.lock().await {
            spinner.finish_with_message("Stream interrupted by user");
            eprintln!("\n⚠️ Stream interrupted by Ctrl+C");
            break;
        }

        chunks.push(chunk?);
        spinner.set_message(format!("Collected {} chunks", chunks.len()));
    }

    spinner.finish_and_clear();
    println!("{}", serde_json::to_string_pretty(&chunks)?);
    Ok(())
}

/// Print streaming output as YAML
async fn print_stream_yaml(stream: &mut ScriptStream, interrupted: Arc<Mutex<bool>>) -> Result<()> {
    // Show progress while collecting chunks
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Collecting stream data...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut chunks = Vec::new();
    while let Some(chunk) = stream.stream.next().await {
        // Check if interrupted
        if *interrupted.lock().await {
            spinner.finish_with_message("Stream interrupted by user");
            eprintln!("\n⚠️ Stream interrupted by Ctrl+C");
            break;
        }

        chunks.push(chunk?);
        spinner.set_message(format!("Collected {} chunks", chunks.len()));
    }

    spinner.finish_and_clear();
    println!("{}", serde_yaml::to_string(&chunks)?);
    Ok(())
}

/// Print streaming output as text with optional progress indicators
async fn print_stream_text(
    stream: &mut ScriptStream,
    show_progress: bool,
    interrupted: Arc<Mutex<bool>>,
) -> Result<()> {
    use std::io::{self, Write};

    let progress = if show_progress {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Streaming output...");
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let mut chunk_count = 0;
    let mut in_tool_call = false;

    while let Some(chunk) = stream.stream.next().await {
        // Check if interrupted
        if *interrupted.lock().await {
            if let Some(ref pb) = progress {
                pb.finish_with_message("Stream interrupted by user");
            }
            eprintln!("\n⚠️ Stream interrupted by Ctrl+C");
            break;
        }

        let chunk = chunk?;
        chunk_count += 1;

        // Update progress message
        if let Some(ref pb) = progress {
            pb.set_message(format!("Processing chunk {}", chunk_count));
        }

        // Print chunk content based on its type
        match &chunk.content {
            llmspell_core::types::ChunkContent::Text(text) => {
                if in_tool_call && progress.is_some() {
                    // Clear the progress bar before printing text
                    if let Some(ref pb) = progress {
                        pb.suspend(|| {
                            print!("{}", text);
                            io::stdout().flush().ok();
                        });
                    }
                } else {
                    print!("{}", text);
                    io::stdout().flush()?;
                }
            }
            llmspell_core::types::ChunkContent::ToolCallProgress { tool_name, .. } => {
                in_tool_call = true;
                if let Some(ref pb) = progress {
                    pb.suspend(|| {
                        println!("\n🔧 Calling tool: {}...", tool_name);
                    });
                    pb.set_message(format!("Tool: {}", tool_name));
                } else {
                    println!("\n[Calling tool: {}...]", tool_name);
                }
            }
            llmspell_core::types::ChunkContent::ToolCallComplete { tool_name, .. } => {
                in_tool_call = false;
                if let Some(ref pb) = progress {
                    pb.suspend(|| {
                        println!("✓ Tool call complete: {}", tool_name);
                    });
                } else {
                    println!("[Tool call complete: {}]", tool_name);
                }
            }
            llmspell_core::types::ChunkContent::Media { caption, .. } => {
                let media_msg = if let Some(cap) = caption {
                    format!("📎 Media: {}", cap)
                } else {
                    "📎 Media content".to_string()
                };

                if let Some(ref pb) = progress {
                    pb.suspend(|| {
                        println!("\n{}", media_msg);
                    });
                } else {
                    println!("\n[{}]", media_msg);
                }
            }
            llmspell_core::types::ChunkContent::Control(msg) => {
                // Handle control messages
                use llmspell_core::types::ControlMessage;
                use tracing::debug;
                debug!("Control message: {:?}", msg);
                if let Some(ref pb) = progress {
                    match msg {
                        ControlMessage::StreamStart { .. } => {
                            pb.set_message("Stream started");
                        }
                        ControlMessage::StreamEnd { .. } => {
                            pb.set_message("Stream ending...");
                        }
                        ControlMessage::StreamCancelled { reason } => {
                            pb.set_message(format!("Stream cancelled: {}", reason));
                        }
                        ControlMessage::Heartbeat => {
                            // Keep spinner alive
                        }
                        ControlMessage::RateLimit { remaining, .. } => {
                            pb.set_message(format!("Rate limited ({} remaining)", remaining));
                        }
                        ControlMessage::Custom { .. } => {
                            // Custom control messages
                        }
                    }
                }
            }
        }
    }

    // Clean up progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("Stream complete");
        pb.finish_and_clear();
    }

    println!(); // Final newline
    Ok(())
}
