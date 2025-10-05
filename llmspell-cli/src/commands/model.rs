//! Model command implementation - sends model requests to kernel
//!
//! This module provides CLI commands for local model operations.
//! All model logic is executed in the kernel which has provider access.

use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{info, instrument, trace};

use crate::cli::{ModelCommands, OutputFormat};
use crate::execution_context::ExecutionContext;
use crate::output::OutputFormatter;
use llmspell_config::LLMSpellConfig;

/// Handle model management commands by sending requests to kernel
#[instrument(skip(runtime_config), fields(command_type))]
pub async fn handle_model_command(
    command: ModelCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling model command");

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
            // For embedded mode, send model requests to embedded kernel
            handle_model_embedded(command, handle, config, output_format).await
        }
        ExecutionContext::Connected { handle, address } => {
            trace!("Using connected context at address: {}", address);
            // For connected mode, send model requests to remote kernel
            handle_model_remote(command, handle, address, output_format).await
        }
    }
}

/// Handle model commands in embedded mode (kernel in same process)
async fn handle_model_embedded(
    command: ModelCommands,
    mut handle: Box<llmspell_kernel::api::KernelHandle>,
    _config: Box<LLMSpellConfig>,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling model command in embedded mode");

    match command {
        ModelCommands::List {
            backend,
            verbose,
            format,
        } => {
            info!("Listing models via kernel message protocol");

            // Create model_request message for list command
            let request_content = json!({
                "command": "list",
                "backend": backend,
                "verbose": verbose,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Extract models from response
            let models = response
                .get("models")
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(format.unwrap_or(output_format));
            if verbose {
                formatter.print_json(&json!({ "models": models, "count": models.len() }))?;
            } else {
                // Simple list format
                for model in models {
                    if let Some(id) = model.get("id").and_then(|v| v.as_str()) {
                        println!("{}", id);
                    }
                }
            }
            Ok(())
        }

        ModelCommands::Pull {
            model,
            force,
            quantization,
        } => {
            info!("Pulling model: {} via kernel", model);

            // Create model_request message for pull command
            let request_content = json!({
                "command": "pull",
                "model": model,
                "force": force,
                "quantization": quantization,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model pull error: {}", error));
            }

            // Display success message
            if let Some(status) = response.get("status").and_then(|v| v.as_str()) {
                println!("{}", status);
            } else {
                println!("Model pulled successfully");
            }

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ModelCommands::Remove { model, yes } => {
            info!("Removing model: {} via kernel", model);

            // Confirm deletion unless --yes flag is set
            if !yes {
                print!("Remove model '{}'? [y/N]: ", model);
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            // Create model_request message for remove command
            let request_content = json!({
                "command": "remove",
                "model": model,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model remove error: {}", error));
            }

            println!("Model removed successfully");
            Ok(())
        }

        ModelCommands::Info { model } => {
            info!("Getting info for model: {} via kernel", model);

            // Create model_request message for info command
            let request_content = json!({
                "command": "info",
                "model": model,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model info error: {}", error));
            }

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ModelCommands::Available {
            backend,
            recommended,
        } => {
            info!("Listing available models via kernel");

            // Create model_request message for available command
            let request_content = json!({
                "command": "available",
                "backend": backend,
                "recommended": recommended,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Extract available models from response
            let models = response
                .get("models")
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&json!({ "models": models, "count": models.len() }))?;
            Ok(())
        }

        ModelCommands::Status => {
            info!("Checking backend status via kernel");

            // Create model_request message for status command
            let request_content = json!({
                "command": "status",
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ModelCommands::InstallOllama => {
            info!("Installing Ollama binary");

            // Create model_request message for install-ollama command
            let request_content = json!({
                "command": "install-ollama",
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Ollama installation error: {}", error));
            }

            println!("Ollama installed successfully");
            Ok(())
        }
    }
}

/// Handle model commands in connected mode (remote kernel)
async fn handle_model_remote(
    command: ModelCommands,
    mut handle: llmspell_kernel::api::ClientHandle,
    address: String,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling model command in connected mode to {}", address);

    match command {
        ModelCommands::List {
            backend,
            verbose,
            format,
        } => {
            info!("Listing models via remote kernel");

            // Create model_request message for list command
            let request_content = json!({
                "command": "list",
                "backend": backend,
                "verbose": verbose,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Extract models from response
            let models = response
                .get("models")
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(format.unwrap_or(output_format));
            if verbose {
                formatter.print_json(&json!({ "models": models, "count": models.len() }))?;
            } else {
                // Simple list format
                for model in models {
                    if let Some(id) = model.get("id").and_then(|v| v.as_str()) {
                        println!("{}", id);
                    }
                }
            }
            Ok(())
        }

        ModelCommands::Pull {
            model,
            force,
            quantization,
        } => {
            info!("Pulling model: {} via remote kernel", model);

            // Create model_request message for pull command
            let request_content = json!({
                "command": "pull",
                "model": model,
                "force": force,
                "quantization": quantization,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model pull error: {}", error));
            }

            // Display success message
            if let Some(status) = response.get("status").and_then(|v| v.as_str()) {
                println!("{}", status);
            } else {
                println!("Model pulled successfully");
            }

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ModelCommands::Remove { model, yes } => {
            info!("Removing model: {} via remote kernel", model);

            // Confirm deletion unless --yes flag is set
            if !yes {
                print!("Remove model '{}'? [y/N]: ", model);
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            // Create model_request message for remove command
            let request_content = json!({
                "command": "remove",
                "model": model,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model remove error: {}", error));
            }

            println!("Model removed successfully");
            Ok(())
        }

        ModelCommands::Info { model } => {
            info!("Getting info for model: {} via remote kernel", model);

            // Create model_request message for info command
            let request_content = json!({
                "command": "info",
                "model": model,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Model info error: {}", error));
            }

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ModelCommands::Available {
            backend,
            recommended,
        } => {
            info!("Listing available models via remote kernel");

            // Create model_request message for available command
            let request_content = json!({
                "command": "available",
                "backend": backend,
                "recommended": recommended,
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Extract available models from response
            let models = response
                .get("models")
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&json!({ "models": models, "count": models.len() }))?;
            Ok(())
        }

        ModelCommands::Status => {
            info!("Checking backend status via remote kernel");

            // Create model_request message for status command
            let request_content = json!({
                "command": "status",
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ModelCommands::InstallOllama => {
            info!("Installing Ollama binary via remote kernel");

            // Create model_request message for install-ollama command
            let request_content = json!({
                "command": "install-ollama",
            });

            // Send request to kernel and wait for response
            let response = handle.send_model_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Ollama installation error: {}", error));
            }

            println!("Ollama installed successfully");
            Ok(())
        }
    }
}
