//! ABOUTME: Run command implementation for executing script files
//! ABOUTME: Handles script execution with streaming and output formatting

use crate::cli::{OutputFormat, ScriptEngine};
use crate::output::format_output;
use anyhow::Result;
use llmspell_bridge::engine::{ScriptMetadata, ScriptOutput};
use llmspell_config::LLMSpellConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Parse script arguments into a HashMap
/// Supports three formats:
/// - Positional: arg1 arg2 arg3 -> {"1": "arg1", "2": "arg2", "3": "arg3"}
/// - Named: --key value --flag true -> {"key": "value", "flag": "true"}
/// - Mixed: pos1 --named value pos2 -> {"1": "pos1", "named": "value", "2": "pos2"}
pub fn parse_script_args(args: Vec<String>, script_path: &Path) -> HashMap<String, String> {
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
    _engine: ScriptEngine, // Engine selection handled by kernel
    runtime_config: LLMSpellConfig,
    _stream: bool, // Streaming handled differently in kernel mode
    args: Vec<String>,
    output_format: OutputFormat,
    debug_mode: bool,
) -> Result<()> {
    // Validate script file exists
    if !script_path.exists() {
        anyhow::bail!("Script file not found: {}", script_path.display());
    }

    // Read script content
    let script_content = fs::read_to_string(&script_path).await?;

    // If debug mode is requested, ensure the config reflects it
    let mut runtime_config = runtime_config;
    if debug_mode {
        runtime_config.debug.enabled = true;
    }

    // Unified execution path via kernel - no longer need separate debug/non-debug paths
    // Parse script arguments
    let parsed_args = parse_script_args(args, &script_path);
    if !parsed_args.is_empty() {
        tracing::debug!("Parsed script arguments: {:?}", parsed_args);
    }

    // Create kernel connection instead of direct runtime
    let mut kernel = super::create_kernel_connection(runtime_config).await?;

    // Execute script via kernel
    // TODO: Add support for script arguments in kernel protocol
    let result = kernel.execute(&script_content).await?;

    // Create ScriptOutput from kernel result
    let script_output = ScriptOutput {
        output: result,
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
