//! ABOUTME: CLI argument parsing and command structures
//! ABOUTME: Defines the command-line interface with multi-engine support

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Command-line interface for LLMSpell
#[derive(Parser, Debug)]
#[command(name = "llmspell")]
#[command(version)]
#[command(about = "LLMSpell - Scriptable LLM interactions")]
#[command(propagate_version = true)]
pub struct Cli {
    /// Script engine to use
    #[arg(long, value_enum, default_value = "lua", env = "LLMSPELL_ENGINE")]
    pub engine: ScriptEngine,

    /// Configuration file path
    #[arg(short, long, env = "LLMSPELL_CONFIG")]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available script engines
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ScriptEngine {
    /// Lua 5.4 engine
    Lua,
    /// JavaScript engine (future)
    #[clap(alias = "js")]
    Javascript,
    /// Python engine (future)
    Python,
}

impl ScriptEngine {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScriptEngine::Lua => "lua",
            ScriptEngine::Javascript => "javascript",
            ScriptEngine::Python => "python",
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            ScriptEngine::Lua => true,         // Available in Phase 1
            ScriptEngine::Javascript => false, // Phase 5
            ScriptEngine::Python => false,     // Phase 9
        }
    }

    pub fn availability_message(&self) -> &'static str {
        match self {
            ScriptEngine::Lua => "Available",
            ScriptEngine::Javascript => "Coming in Phase 5",
            ScriptEngine::Python => "Coming in Phase 9",
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// Pretty-printed output
    Pretty,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a script file
    Run {
        /// Script file to execute
        script: PathBuf,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,

        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Execute inline script
    Exec {
        /// Script code to execute
        #[arg(value_name = "CODE")]
        code: String,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,
    },

    /// Start interactive REPL
    Repl {
        /// History file path
        #[arg(long)]
        history: Option<PathBuf>,
    },

    /// Available Providers
    Providers {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Validate configuration
    Validate {
        /// Configuration file to validate
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Show engine information
    Info {
        /// Show all engines (including unavailable)
        #[arg(long)]
        all: bool,
    },

    /// Initialize configuration file
    Init {
        /// Output path for configuration file
        #[arg(short, long, default_value = "llmspell.toml")]
        output: PathBuf,

        /// Force overwrite existing file
        #[arg(short, long)]
        force: bool,
    },

    /// Manage API keys for external services
    #[command(subcommand)]
    Keys(crate::commands::keys::KeysSubcommand),

    /// Backup and restore state data
    Backup(crate::commands::backup::BackupCommand),
}

impl Cli {
    /// Validate the selected engine is available
    pub fn validate_engine(&self) -> anyhow::Result<()> {
        if !self.engine.is_available() {
            anyhow::bail!(
                "Script engine '{}' is not available yet. {}",
                self.engine.as_str(),
                self.engine.availability_message()
            );
        }
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path(&self) -> Option<PathBuf> {
        self.config.clone().or_else(|| {
            // Try default locations
            let home = dirs::home_dir()?;
            let config_path = home.join(".llmspell").join("config.toml");
            if config_path.exists() {
                Some(config_path)
            } else {
                None
            }
        })
    }
}
