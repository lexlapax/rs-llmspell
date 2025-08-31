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
        // Debug execution path using kernel
        // For inline scripts, we'll use a temporary path
        let temp_path = std::path::PathBuf::from("<inline>");
        super::run_debug::execute_script_debug(
            code,
            temp_path,
            runtime_config,
            vec![],
            output_format,
        )
        .await
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
