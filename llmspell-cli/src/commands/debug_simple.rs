//! Debug command implementation - simplified stub version
//!
//! Temporarily disabled.
//! Will be reimplemented using kernel's Jupyter debug protocol.

use crate::cli::{OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// Handle the debug command - temporarily disabled
pub async fn handle_debug_command(
    script: PathBuf,
    _args: Vec<String>,
    _engine: ScriptEngine,
    _config: LLMSpellConfig,
    _output_format: OutputFormat,
) -> Result<()> {
    anyhow::bail!(
        "Debug command temporarily disabled during protocol migration.\n\
        Script: {}\n\
        Please use 'llmspell run' with kernel debugging instead.",
        script.display()
    )
}
