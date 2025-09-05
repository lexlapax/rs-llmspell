//! ABOUTME: Exec command implementation for inline script execution
//! ABOUTME: Executes script code provided directly on the command line

use crate::cli::{OutputFormat, ScriptEngine};
use crate::output::format_output;
use anyhow::Result;
use llmspell_bridge::engine::{ScriptMetadata, ScriptOutput};
use llmspell_config::LLMSpellConfig;

/// Execute inline script code
pub async fn execute_inline_script(
    code: String,
    _engine: ScriptEngine, // Engine selection handled by kernel
    runtime_config: LLMSpellConfig,
    connect: Option<String>, // Connection string for external kernel
    _stream: bool,           // Streaming handled differently in kernel mode
    debug_mode: bool,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::debug!(
        "[9.8.2] execute_inline_script - starting with code: {}",
        code
    );

    // Unified execution path - kernel handles debug vs non-debug internally
    let mut runtime_config = runtime_config;
    if debug_mode {
        runtime_config.debug.enabled = true;
        runtime_config.debug.mode = "interactive".to_string();
    }

    tracing::debug!("[9.8.2] execute_inline_script - creating kernel connection");
    // Create kernel connection instead of direct runtime
    let mut kernel = super::create_kernel_connection(runtime_config, connect).await?;

    tracing::debug!("[9.8.2] execute_inline_script - executing code via kernel");
    // Execute code via kernel
    let result = kernel.execute(&code).await?;
    tracing::debug!(
        "[9.8.2] execute_inline_script - received result: {:?}",
        result
    );

    // Create ScriptOutput from kernel result
    let script_output = ScriptOutput {
        output: serde_json::Value::String(result),
        console_output: vec![], // TODO: Get console output from kernel
        metadata: ScriptMetadata {
            engine: "kernel".to_string(),
            execution_time_ms: 0, // TODO: Get timing from kernel
            memory_usage_bytes: None,
            warnings: vec![],
        },
    };

    // Format and display the result
    println!("{}", format_output(&script_output, output_format)?);

    Ok(())
}
