//! Debug execution support for run command
//!
//! Phase 9.8: Debug mode should be implemented in kernel, not CLI

use crate::cli::OutputFormat;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// Execute script in debug mode via kernel connection
pub async fn execute_script_debug(
    _script_content: String,
    script_path: PathBuf,
    _runtime_config: LLMSpellConfig,
    _args: Vec<String>,
    _output_format: OutputFormat,
) -> Result<()> {
    // Phase 9.8: All execution must go through kernel
    // Debug functionality should be implemented in kernel, not CLI
    anyhow::bail!(
        "Debug mode temporarily disabled. Per Phase 9.8, all execution must go through kernel.\n\
         Debug functionality will be implemented in kernel. Use: llmspell run {}",
        script_path.display()
    )
}