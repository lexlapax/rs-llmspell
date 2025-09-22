//! REPL state management
//!
//! Manages REPL session state including history, variables, and debug context.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

/// Maximum history entries to keep in memory
const MAX_HISTORY_SIZE: usize = 1000;

/// REPL session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplState {
    /// Current working directory
    pub working_dir: PathBuf,
    /// Session variables
    pub variables: HashMap<String, String>,
    /// Command history
    pub history: SessionHistory,
    /// Debug breakpoints
    pub breakpoints: Vec<Breakpoint>,
    /// Current debug context
    pub debug_context: Option<DebugContext>,
}

impl ReplState {
    /// Create new REPL state
    pub fn new() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_default(),
            variables: HashMap::new(),
            history: SessionHistory::new(),
            breakpoints: Vec::new(),
            debug_context: None,
        }
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.push(breakpoint);
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: usize) -> Option<Breakpoint> {
        if id < self.breakpoints.len() {
            Some(self.breakpoints.remove(id))
        } else {
            None
        }
    }

    /// List all breakpoints
    pub fn list_breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    /// Set debug context
    pub fn set_debug_context(&mut self, context: DebugContext) {
        self.debug_context = Some(context);
    }

    /// Clear debug context
    pub fn clear_debug_context(&mut self) {
        self.debug_context = None;
    }
}

impl Default for ReplState {
    fn default() -> Self {
        Self::new()
    }
}

/// Session command history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    entries: VecDeque<HistoryEntry>,
    current_index: Option<usize>,
}

impl SessionHistory {
    /// Create new history
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            current_index: None,
        }
    }

    /// Add command to history
    pub fn add(&mut self, command: String) {
        // Don't add duplicate consecutive commands
        if self.entries.back().map(|e| &e.command) != Some(&command) {
            let entry = HistoryEntry {
                command,
                timestamp: chrono::Utc::now(),
            };
            self.entries.push_back(entry);

            // Limit history size
            while self.entries.len() > MAX_HISTORY_SIZE {
                self.entries.pop_front();
            }
        }
        self.current_index = None;
    }

    /// Get previous command
    pub fn previous(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        let index = match self.current_index {
            None => self.entries.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(_) => return None,
        };

        self.current_index = Some(index);
        self.entries.get(index).map(|e| e.command.as_str())
    }

    /// Get next command
    pub fn next_command(&mut self) -> Option<&str> {
        match self.current_index {
            Some(i) if i < self.entries.len() - 1 => {
                self.current_index = Some(i + 1);
                self.entries.get(i + 1).map(|e| e.command.as_str())
            }
            Some(_) => {
                self.current_index = None;
                None
            }
            None => None,
        }
    }

    /// Get all history entries
    pub fn entries(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.command.clone()).collect()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
    }

    /// Save history to file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = self.entries().join("\n");
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load history from file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn load_from_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            for line in content.lines() {
                if !line.trim().is_empty() {
                    self.add(line.to_string());
                }
            }
        }
        Ok(())
    }
}

impl Default for SessionHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// History entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    command: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Debug breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Breakpoint ID
    pub id: usize,
    /// File path
    pub file: Option<String>,
    /// Line number
    pub line: usize,
    /// Condition (optional)
    pub condition: Option<String>,
    /// Hit count
    pub hit_count: usize,
    /// Is enabled
    pub enabled: bool,
}

impl Breakpoint {
    /// Create new breakpoint
    pub fn new(id: usize, line: usize) -> Self {
        Self {
            id,
            file: None,
            line,
            condition: None,
            hit_count: 0,
            enabled: true,
        }
    }

    /// Set file
    #[must_use]
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    /// Set condition
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }
}

/// Debug context when paused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugContext {
    /// Current stack frames
    pub stack_frames: Vec<StackFrame>,
    /// Current frame index
    pub current_frame: usize,
    /// Local variables
    pub locals: HashMap<String, String>,
    /// Pause reason
    pub pause_reason: PauseReason,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame ID
    pub id: usize,
    /// Function name
    pub name: String,
    /// Source file
    pub file: Option<String>,
    /// Line number
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
}

/// Reason for debug pause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PauseReason {
    /// Hit a breakpoint
    Breakpoint(usize),
    /// Step command
    Step,
    /// Exception occurred
    Exception(String),
    /// Manual pause
    Pause,
}
