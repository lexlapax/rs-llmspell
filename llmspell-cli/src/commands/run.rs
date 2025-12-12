//! ABOUTME: Run command implementation for executing script files
//! ABOUTME: Handles script execution with streaming and output formatting

use crate::cli::{OutputFormat, ScriptEngine};
use crate::execution_context::ExecutionContext;
use anyhow::Result;
use llmspell_kernel::api::{ClientHandle, KernelHandle};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

/// Parse script arguments into a HashMap
/// Supports three formats:
/// - Positional: arg1 arg2 arg3 -> {"1": "arg1", "2": "arg2", "3": "arg3"}
/// - Named: --key value --flag true -> {"key": "value", "flag": "true"}
/// - Mixed: pos1 --named value pos2 -> {"1": "pos1", "named": "value", "2": "pos2"}
fn parse_script_args(args: Vec<String>, script_path: &Path) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let mut positional_index = 1;
    let mut i = 0;

    // Add script name as arg[0] for Lua compatibility
    if let Some(script_name) = script_path.file_name() {
        parsed.insert("0".to_string(), script_name.to_string_lossy().to_string());
    }

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            // Named argument
            let key = arg.trim_start_matches("--");

            // Check if there's a value following this flag
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                // --key value format
                parsed.insert(key.to_string(), args[i + 1].clone());
                i += 2;
            } else {
                // --flag format (boolean flag without value)
                parsed.insert(key.to_string(), "true".to_string());
                i += 1;
            }
        } else {
            // Positional argument
            parsed.insert(positional_index.to_string(), arg.clone());
            positional_index += 1;
            i += 1;
        }
    }

    parsed
}

/// Execute a script file
pub async fn execute_script_file(
    script_path: PathBuf,
    engine: ScriptEngine,
    context: ExecutionContext,
    stream: bool,
    args: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Validate script file exists
    if !script_path.exists() {
        anyhow::bail!("Script file not found: {}", script_path.display());
    }

    // Validate the engine is available
    crate::commands::validate_engine(engine)?;

    info!(
        "Executing script with {} engine: {}",
        engine.as_str(),
        script_path.display()
    );

    // Read script content
    let script_content = fs::read_to_string(&script_path).await?;

    // Parse script arguments
    let parsed_args = parse_script_args(args, &script_path);
    if !parsed_args.is_empty() {
        debug!("Parsed script arguments: {:?}", parsed_args);
    }

    // Execute based on context type
    match context {
        ExecutionContext::Embedded { handle, config: _ } => {
            // Use embedded kernel for execution
            execute_script_embedded(*handle, &script_content, parsed_args, stream, output_format)
                .await?;
        }
        ExecutionContext::Connected { handle, address: _ } => {
            // Use connected kernel for execution
            execute_script_connected(handle, &script_content, parsed_args, stream, output_format)
                .await?;
        }
    }

    Ok(())
}

/// Execute script using embedded kernel
async fn execute_script_embedded(
    handle: KernelHandle,
    script_content: &str,
    args: HashMap<String, String>,
    stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
    // Pass script arguments to the execution context
    if !args.is_empty() {
        debug!(
            "Script arguments will be available in script context: {:?}",
            args
        );
    }

    // For embedded mode, we need to directly execute using the kernel
    // The KernelHandle.execute() method requires the kernel to be running,
    // which would cause a deadlock. Instead, we'll get the kernel and execute directly (Direct mode).
    let mut kernel = handle.into_kernel()?;

    // Execute the script with arguments
    let result = kernel
        .execute_direct_with_args(script_content, args.clone())
        .await?;

    // Format and display the output based on the requested format
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mode": "embedded",
                    "script_length": script_content.len(),
                    "args_count": args.len(),
                    "streaming": stream,
                    "result": result
                })
            );
        }
        _ => {
            // For plain text output, the result is already printed via IOPub
            // Just show completion status
            if stream {
                debug!("Script execution completed with streaming");
            } else {
                debug!("Script execution completed");
            }
        }
    }
    Ok(())
}

/// Execute script using connected kernel
async fn execute_script_connected(
    mut handle: ClientHandle,
    script_content: &str,
    args: HashMap<String, String>,
    _stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
    debug!(
        "Executing script on connected kernel (length: {} chars, args: {})",
        script_content.len(),
        args.len()
    );

    // TODO: Inject args into the script context before execution
    // For now, we just execute the script without args support in connected mode
    if !args.is_empty() {
        debug!("Warning: Script arguments not yet supported in connected mode");
    }

    // Execute the script on the remote kernel
    let result = handle.execute(script_content).await?;

    // Format and display the output based on the requested format
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mode": "connected",
                    "script_length": script_content.len(),
                    "args_count": args.len(),
                    "result": result
                })
            );
        }
        _ => {
            // For plain text output, the result is already printed via IOPub
            // Just show completion status
            debug!("Script execution completed via connected kernel");
        }
    }

    Ok(())
}
