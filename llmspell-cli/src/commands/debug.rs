//! Debug command - forwards debug requests to kernel
//!
//! Since Phase 9.8, all execution happens through the kernel.
//! This is now a thin wrapper that sends debug commands to the kernel.

use crate::cli::{OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// Handle the debug command by forwarding to kernel
pub async fn handle_debug_command(
    script: PathBuf,
    _break_at: Vec<String>,
    _port: Option<u16>,
    _args: Vec<String>,
    _engine: ScriptEngine,
    _config: LLMSpellConfig,
    _output_format: OutputFormat,
) -> Result<()> {
    // Phase 9.8: All execution must go through kernel
    // Debug functionality should be implemented in kernel, not CLI
    anyhow::bail!(
        "Debug command temporarily disabled. Per Phase 9.8, all execution must go through kernel.\n\
         Use the dedicated Debug command: llmspell debug {}",
        script.display()
    )
}
