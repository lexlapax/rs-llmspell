//! ABOUTME: Exec command implementation for inline script execution
//! ABOUTME: Executes script code provided directly on the command line

use crate::cli::{ScriptEngine, OutputFormat};
use crate::output::{format_output, print_stream};
use llmspell_bridge::RuntimeConfig;
use anyhow::Result;

/// Execute inline script code
pub async fn execute_inline_script(
    code: String,
    engine: ScriptEngine,
    runtime_config: RuntimeConfig,
    stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
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