//! Tool command implementation - sends tool requests to kernel
//!
//! This module provides CLI commands for tool operations.
//! All tool logic is executed in the kernel which has ComponentRegistry access.

use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{info, instrument, trace, warn};

use crate::cli::{OutputFormat, ToolCommands};
use crate::execution_context::ExecutionContext;
use crate::output::OutputFormatter;
use llmspell_config::LLMSpellConfig;

/// Handle tool management commands by sending requests to kernel
#[instrument(skip(runtime_config), fields(command_type))]
pub async fn handle_tool_command(
    command: ToolCommands,
    source: String,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling tool command with source: {}", source);

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
            // For embedded mode, send tool requests to embedded kernel
            handle_tool_embedded(command, handle, config, output_format).await
        }
        ExecutionContext::Connected { handle, address } => {
            trace!("Using connected context at address: {}", address);
            // For connected mode, send tool requests to remote kernel
            handle_tool_remote(command, handle, address, output_format).await
        }
    }
}

/// Handle tool commands in embedded mode (kernel in same process)
async fn handle_tool_embedded(
    command: ToolCommands,
    mut handle: Box<llmspell_kernel::api::KernelHandle>,
    _config: Box<LLMSpellConfig>,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handle tool embedded with command: {:?}", command);
    trace!("Handling tool command in embedded mode");

    match command {
        ToolCommands::List { category, format } => {
            info!("Listing tools via kernel message protocol");

            // Create tool_request message for list command
            let request_content = json!({
                "command": "list",
                "category": category,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Extract tools from response
            let tools = response
                .get("tools")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(format.unwrap_or(output_format));
            formatter.print_tool_list(&tools)?;
            Ok(())
        }

        ToolCommands::Info { name, show_schema } => {
            info!("Getting info for tool: {} via kernel", name);

            // Create tool_request message for info command
            let request_content = json!({
                "command": "info",
                "name": name,
                "show_schema": show_schema,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Tool info error: {}", error));
            }

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ToolCommands::Invoke {
            name,
            params,
            stream,
        } => {
            info!("Invoking tool: {} via kernel", name);
            trace!("Parameters: {:?}", params);

            if stream {
                warn!("Streaming not yet implemented");
            }

            // Create tool_request message for invoke command
            let request_content = json!({
                "command": "invoke",
                "name": name,
                "params": params,
                "stream": stream,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Tool invocation error: {}", error));
            }

            // Format and display the result
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ToolCommands::Search { query, category } => {
            info!("Searching tools with query: {:?} via kernel", query);

            // Create tool_request message for search command
            let request_content = json!({
                "command": "search",
                "query": query,
                "category": category,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Extract matching tools from response
            let matches = response
                .get("matches")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(output_format);
            formatter.print_tool_list(&matches)?;
            Ok(())
        }

        ToolCommands::Test { name, verbose } => {
            info!("Testing tool: {} via kernel", name);

            // Create tool_request message for test command
            let request_content = json!({
                "command": "test",
                "name": name,
                "verbose": verbose,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Tool test error: {}", error));
            }

            // Display test results
            if verbose {
                if let Some(details) = response.get("details") {
                    println!("Test details: {}", details);
                }
            }

            if response
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                println!("✓ Tool '{}' test successful", name);
            } else {
                let message = response
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Test failed");
                println!("✗ Tool '{}' test failed: {}", name, message);
            }
            Ok(())
        }
    }
}

/// Handle tool commands in connected mode (remote kernel)
async fn handle_tool_remote(
    command: ToolCommands,
    mut handle: llmspell_kernel::api::ClientHandle,
    address: String,
    output_format: OutputFormat,
) -> Result<()> {
    trace!("Handling tool command in connected mode to {}", address);

    match command {
        ToolCommands::List { category, format } => {
            info!("Listing tools via remote kernel");

            // Create tool_request message for list command
            let request_content = json!({
                "command": "list",
                "category": category,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Extract tools from response
            let tools = response
                .get("tools")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(format.unwrap_or(output_format));
            formatter.print_tool_list(&tools)?;
            Ok(())
        }

        ToolCommands::Info { name, show_schema } => {
            info!("Getting info for tool: {} via remote kernel", name);

            // Create tool_request message for info command
            let request_content = json!({
                "command": "info",
                "name": name,
                "show_schema": show_schema,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Tool info error: {}", error));
            }

            // Format and display the response
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ToolCommands::Invoke {
            name,
            params,
            stream,
        } => {
            info!("Invoking tool: {} via remote kernel", name);
            trace!("Parameters: {:?}", params);

            if stream {
                warn!("Streaming not yet implemented");
            }

            // Create tool_request message for invoke command
            let request_content = json!({
                "command": "invoke",
                "name": name,
                "params": params,
                "stream": stream,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Tool invocation error: {}", error));
            }

            // Format and display the result
            let formatter = OutputFormatter::new(output_format);
            formatter.print_json(&response)?;
            Ok(())
        }

        ToolCommands::Search { query, category } => {
            info!("Searching tools with query: {:?} via remote kernel", query);

            // Create tool_request message for search command
            let request_content = json!({
                "command": "search",
                "query": query,
                "category": category,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Extract matching tools from response
            let matches = response
                .get("matches")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            // Format output
            let formatter = OutputFormatter::new(output_format);
            formatter.print_tool_list(&matches)?;
            Ok(())
        }

        ToolCommands::Test { name, verbose } => {
            info!("Testing tool: {} via remote kernel", name);

            // Create tool_request message for test command
            let request_content = json!({
                "command": "test",
                "name": name,
                "verbose": verbose,
            });

            // Send request to kernel and wait for response
            let response = handle.send_tool_request(request_content).await?;

            // Check for error in response
            if let Some(error) = response.get("error") {
                return Err(anyhow!("Tool test error: {}", error));
            }

            // Display test results
            if verbose {
                if let Some(details) = response.get("details") {
                    println!("Test details: {}", details);
                }
            }

            if response
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                println!("✓ Tool '{}' test successful", name);
            } else {
                let message = response
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Test failed");
                println!("✗ Tool '{}' test failed: {}", name, message);
            }
            Ok(())
        }
    }
}
