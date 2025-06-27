//! ABOUTME: Run command implementation for executing script files
//! ABOUTME: Handles script execution with streaming and output formatting

use crate::cli::{ScriptEngine, OutputFormat};
use crate::output::{format_output, print_stream};
use llmspell_bridge::RuntimeConfig;
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

/// Execute a script file
pub async fn execute_script_file(
    script_path: PathBuf,
    engine: ScriptEngine,
    runtime_config: RuntimeConfig,
    stream: bool,
    args: Vec<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Validate script file exists
    if !script_path.exists() {
        anyhow::bail!("Script file not found: {}", script_path.display());
    }

    // Read script content
    let script_content = fs::read_to_string(&script_path).await?;

    // Create runtime for the selected engine
    let runtime = super::create_runtime(engine, runtime_config).await?;

    // TODO: Pass script arguments to the runtime
    if !args.is_empty() {
        tracing::debug!("Script arguments: {:?}", args);
    }

    // Execute script
    if stream && runtime.supports_streaming() {
        // Execute with streaming
        let mut stream = runtime.execute_script_streaming(&script_content).await?;
        print_stream(&mut stream, output_format).await?;
    } else {
        // Execute without streaming
        let result = runtime.execute_script(&script_content).await?;
        println!("{}", format_output(&result, output_format)?);
    }

    Ok(())
}