//! ABOUTME: REPL command implementation for interactive scripting
//! ABOUTME: Provides an interactive read-eval-print loop

use crate::cli::ScriptEngine;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// Start an interactive REPL session
pub async fn start_repl(
    engine: ScriptEngine,
    _runtime_config: LLMSpellConfig,
    _history_file: Option<PathBuf>,
) -> Result<()> {
    println!("LLMSpell REPL - {} engine", engine.as_str());
    println!("Type 'exit' or press Ctrl+D to quit");
    println!();

    // TODO: Implement full REPL in Phase 2
    anyhow::bail!("REPL mode not implemented yet (coming in Phase 2)")
}
