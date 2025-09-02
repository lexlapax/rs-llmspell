//! ABOUTME: Exec command implementation for inline script execution
//! ABOUTME: Executes script code provided directly on the command line

use crate::cli::{OutputFormat, ScriptEngine};
use crate::output::{format_output, print_stream};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;

/// Execute inline script code
pub async fn execute_inline_script(
    code: String,
    engine: ScriptEngine,
    runtime_config: LLMSpellConfig,
    stream: bool,
    debug_mode: bool,
    output_format: OutputFormat,
) -> Result<()> {
    if debug_mode {
        // Debug execution path using DebugBridge (Bridge Pattern architecture)
        // Create temporary script file for inline code debugging
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new()
            .map_err(|e| anyhow::anyhow!("Failed to create temporary script file: {}", e))?;

        temp_file
            .write_all(code.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write to temporary script file: {}", e))?;

        let temp_path = temp_file.path().to_path_buf();

        // Keep temp file alive during debugging
        let result = super::debug::handle_debug_command(
            temp_path,
            vec![], // No args for inline scripts
            engine,
            runtime_config,
            output_format,
        )
        .await;

        // Temp file automatically deleted when temp_file goes out of scope
        result
    } else {
        // Non-debug execution path (existing implementation)
        // Create runtime for the selected engine
        let runtime = super::create_runtime(engine, runtime_config).await?;

        // Execute script
        if stream && runtime.supports_streaming() {
            // Execute with streaming
            let mut stream = runtime.execute_script_streaming(&code).await?;
            print_stream(&mut stream, output_format).await?;
        } else {
            // Execute without streaming
            let result = runtime.execute_script(&code).await?;
            println!("{}", format_output(&result, output_format)?);
        }

        Ok(())
    }
}
