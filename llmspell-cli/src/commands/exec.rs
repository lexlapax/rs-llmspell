//! Inline code execution commands
//! Professional inline script execution with streaming support

use crate::cli::{OutputFormat, ScriptEngine};
use crate::execution_context::ExecutionContext;
use anyhow::Result;
use tracing::info;

/// Execute inline code
pub async fn execute_inline_script(
    code: String,
    engine: ScriptEngine,
    context: ExecutionContext,
    stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
    crate::commands::validate_engine(engine)?;

    info!(
        "Executing inline code (engine: {:?}, stream: {})",
        engine, stream
    );

    match context {
        ExecutionContext::Embedded { handle, .. } => {
            // Execute inline code using embedded kernel
            execute_code_embedded(*handle, &code, engine, stream, output_format).await?;
        }
        ExecutionContext::Connected { handle, .. } => {
            // Execute code on connected kernel
            execute_code_connected(handle, &code, engine, stream, output_format).await?;
        }
    }

    Ok(())
}

/// Execute code using embedded kernel
async fn execute_code_embedded(
    handle: llmspell_kernel::api::KernelHandle,
    code: &str,
    _engine: ScriptEngine,
    _stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
    use std::collections::HashMap;
    use tracing::debug;

    // Get the kernel from the handle and execute the code (Direct mode)
    let mut kernel = handle.into_kernel()?;
    let result = kernel
        .execute_direct_with_args(code, HashMap::new())
        .await?;

    // Format and display the output based on the requested format
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "executed",
                    "mode": "embedded",
                    "code_length": code.len(),
                    "result": result
                })
            );
        }
        _ => {
            // For plain text output, the result is already printed via IOPub
            // Just show completion status
            debug!("Code execution completed");
        }
    }

    Ok(())
}

/// Execute code using connected kernel
async fn execute_code_connected(
    mut handle: llmspell_kernel::api::ClientHandle,
    code: &str,
    _engine: ScriptEngine,
    _stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
    use tracing::debug;

    // Execute the code on the remote kernel
    let result = handle.execute(code).await?;

    // Format and display the output based on the requested format
    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "executed",
                    "mode": "connected",
                    "code_length": code.len(),
                    "result": result
                })
            );
        }
        _ => {
            // For plain text output, the result is already printed via IOPub
            // Just show completion status
            debug!("Code execution completed via connected kernel");
        }
    }

    Ok(())
}
