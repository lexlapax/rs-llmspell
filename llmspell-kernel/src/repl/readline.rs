//! Readline support for REPL with history navigation
//!
//! This module provides readline functionality with arrow key navigation,
//! history search, and tab completion support.

use crate::repl::state::ReplState;
use anyhow::{anyhow, Result};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// REPL readline interface with history support
pub struct ReplReadline {
    editor: Editor<ReplHelper, rustyline::history::FileHistory>,
    state: Arc<RwLock<ReplState>>,
}

impl ReplReadline {
    /// Create a new readline interface with history support
    ///
    /// # Errors
    ///
    /// Returns an error if the readline editor cannot be created
    pub async fn new(state: Arc<RwLock<ReplState>>) -> Result<Self> {
        // Configure editor with full readline features
        let config = rustyline::Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)? // Avoid duplicate entries
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(rustyline::EditMode::Emacs) // Enables Ctrl+R, Ctrl+A, Ctrl+E, Ctrl+W
            .auto_add_history(true) // Automatically add commands to history
            .max_history_size(10000)? // Large history buffer
            .build();

        // Create editor with config
        let mut editor =
            Editor::<ReplHelper, rustyline::history::FileHistory>::with_config(config)?;

        // Set helper for completion and hints
        editor.set_helper(Some(ReplHelper::new()));

        // Load existing history into rustyline
        let repl_state = state.read().await;
        for entry in repl_state.history.entries() {
            let _ = editor.add_history_entry(entry);
        }
        drop(repl_state); // Release read lock

        Ok(Self { editor, state })
    }

    /// Read a line with the given prompt
    ///
    /// # Errors
    ///
    /// Returns an error if the readline operation fails or is interrupted
    pub async fn readline(&mut self, prompt: &str) -> Result<String> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                // Add to rustyline history
                let _ = self.editor.add_history_entry(&line);

                // Add to SessionHistory
                let mut repl_state = self.state.write().await;
                repl_state.history.add(line.clone());
                drop(repl_state); // Release write lock

                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                debug!("Readline interrupted (Ctrl-C)");
                Err(anyhow!("Interrupted"))
            }
            Err(ReadlineError::Eof) => {
                debug!("Readline EOF (Ctrl-D)");
                Err(anyhow!("EOF"))
            }
            Err(e) => {
                warn!("Readline error: {}", e);
                Err(anyhow!("Readline error: {}", e))
            }
        }
    }

    /// Save history to file
    ///
    /// # Errors
    ///
    /// Returns an error if the history file cannot be saved
    pub fn save_history(&mut self, path: &std::path::Path) -> Result<()> {
        self.editor.save_history(path)?;
        Ok(())
    }

    /// Load history from file with corruption recovery
    ///
    /// # Errors
    ///
    /// Returns an error if the history file cannot be loaded
    pub fn load_history(&mut self, path: &std::path::Path) -> Result<()> {
        if path.exists() {
            match self.editor.load_history(path) {
                Ok(()) => {
                    debug!("Loaded history from {:?}", path);
                }
                Err(e) => {
                    warn!(
                        "Failed to load history from {:?}: {}. Creating backup and starting fresh.",
                        path, e
                    );

                    // Create backup of corrupted history
                    let backup_path = path.with_extension("corrupt.backup");
                    if let Err(backup_err) = std::fs::copy(path, &backup_path) {
                        warn!("Failed to backup corrupted history: {}", backup_err);
                    } else {
                        debug!("Backed up corrupted history to {:?}", backup_path);
                    }

                    // Remove corrupted file and start fresh
                    if let Err(remove_err) = std::fs::remove_file(path) {
                        warn!("Failed to remove corrupted history file: {}", remove_err);
                    }

                    // Continue with empty history - not a fatal error
                }
            }
        }
        Ok(())
    }
}

/// Helper for readline with completion and hints
pub struct ReplHelper {
    commands: Vec<String>,
}

impl ReplHelper {
    /// Create a new REPL helper
    pub fn new() -> Self {
        Self {
            commands: vec![
                // Meta commands
                ".help".to_string(),
                ".exit".to_string(),
                ".quit".to_string(),
                ".q".to_string(),
                ".clear".to_string(),
                ".cls".to_string(),
                ".save".to_string(),
                ".load".to_string(),
                ".history".to_string(),
                ".hist".to_string(),
                ".clear-history".to_string(),
                ".variables".to_string(),
                ".vars".to_string(),
                ".set".to_string(),
                ".unset".to_string(),
                ".cd".to_string(),
                ".pwd".to_string(),
                ".ls".to_string(),
                ".info".to_string(),
                ".reset".to_string(),
                ".run".to_string(),
                ".perf".to_string(),
                // Debug commands
                "break".to_string(),
                "b".to_string(),
                "delete".to_string(),
                "d".to_string(),
                "list".to_string(),
                "l".to_string(),
                "step".to_string(),
                "s".to_string(),
                "next".to_string(),
                "n".to_string(),
                "finish".to_string(),
                "f".to_string(),
                "continue".to_string(),
                "c".to_string(),
                "locals".to_string(),
                "backtrace".to_string(),
                "bt".to_string(),
                "where".to_string(),
                "frame".to_string(),
                "print".to_string(),
                "p".to_string(),
                "watch".to_string(),
                "unwatch".to_string(),
                // Debug command prefix
                "db:".to_string(),
            ],
        }
    }
}

impl Default for ReplHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut candidates = Vec::new();

        // Get the word being completed
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |i| i + 1);
        let word = &line[start..pos];

        // Complete commands
        if word.starts_with('.') || word.starts_with("db:") || start == 0 {
            for cmd in &self.commands {
                if cmd.starts_with(word) {
                    candidates.push(Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    });
                }
            }
        }

        // Complete file paths for certain commands
        if line.starts_with(".load ") || line.starts_with(".save ") || line.starts_with(".run ") {
            // Get the partial path to complete
            let partial = &line[start..pos];

            // Determine directory and file prefix
            let (dir_path, file_prefix) = if let Some(slash_pos) = partial.rfind('/') {
                let dir = &partial[..=slash_pos];
                let prefix = &partial[slash_pos + 1..];
                (dir.to_string(), prefix.to_string())
            } else {
                ("./".to_string(), partial.to_string())
            };

            // Try to read the directory
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        // Skip hidden files unless user is explicitly looking for them
                        if file_name.starts_with('.') && !file_prefix.starts_with('.') {
                            continue;
                        }

                        // Check if the file name starts with the prefix
                        if file_name.starts_with(&file_prefix) {
                            let full_path = if dir_path == "./" {
                                file_name.to_string()
                            } else {
                                format!("{dir_path}{file_name}")
                            };

                            // Add trailing slash for directories
                            let display_path = if entry.path().is_dir() {
                                format!("{full_path}/")
                            } else {
                                full_path.clone()
                            };

                            candidates.push(Pair {
                                display: display_path.clone(),
                                replacement: display_path,
                            });
                        }
                    }
                }
            }
        }

        Ok((start, candidates))
    }
}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        // Find commands that start with the current line
        for cmd in &self.commands {
            if cmd.starts_with(line) && cmd != line {
                return Some(cmd[line.len()..].to_string());
            }
        }

        None
    }
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // For now, return the line as-is
        // Could add syntax highlighting later
        Cow::Borrowed(line)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Make hints dimmer (would need ANSI codes in a real implementation)
        Cow::Borrowed(hint)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

impl Validator for ReplHelper {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> Result<rustyline::validate::ValidationResult, ReadlineError> {
        // No validation for now - multi-line will be handled separately
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_creation() {
        let helper = ReplHelper::new();
        assert!(!helper.commands.is_empty());
        assert!(helper.commands.contains(&".help".to_string()));
    }

    #[test]
    fn test_completion() {
        let helper = ReplHelper::new();
        let history = rustyline::history::MemHistory::new();
        let ctx = rustyline::Context::new(&history);

        // Test command completion
        let (start, candidates) = helper.complete(".hel", 4, &ctx).unwrap();
        assert_eq!(start, 0);
        assert!(candidates.iter().any(|p| p.display == ".help"));

        // Test debug command completion
        let (start, candidates) = helper.complete("db:", 3, &ctx).unwrap();
        assert_eq!(start, 0);
        assert!(candidates.iter().any(|p| p.display == "db:"));
    }
}
