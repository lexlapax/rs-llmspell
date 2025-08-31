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

    /// Enable debug output
    #[arg(long, global = true)]
    pub debug: bool,

    /// Set debug level (trace, debug, info, warn, error, off)
    #[arg(long, global = true)]
    pub debug_level: Option<String>,

    /// Set debug output format (text, json, json_pretty)
    #[arg(long, global = true)]
    pub debug_format: Option<String>,

    /// Filter debug output by modules (comma-separated, use + to enable, - to disable)
    #[arg(long, global = true)]
    pub debug_modules: Option<String>,

    /// Enable performance profiling
    #[arg(long, global = true)]
    pub debug_perf: bool,

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

        /// Enable debug mode for script execution
        #[arg(long)]
        debug: bool,

        /// Enable RAG functionality (overrides config)
        #[arg(long)]
        rag: bool,

        /// Disable RAG functionality (overrides config)
        #[arg(long)]
        no_rag: bool,

        /// Custom RAG configuration file
        #[arg(long, value_name = "FILE")]
        rag_config: Option<PathBuf>,

        /// Override vector storage dimensions
        #[arg(long, value_name = "SIZE")]
        rag_dims: Option<usize>,

        /// Override vector storage backend (hnsw, mock)
        #[arg(long, value_name = "BACKEND")]
        rag_backend: Option<String>,

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

        /// Enable debug mode for script execution
        #[arg(long)]
        debug: bool,

        /// Enable RAG functionality (overrides config)
        #[arg(long)]
        rag: bool,

        /// Disable RAG functionality (overrides config)
        #[arg(long)]
        no_rag: bool,

        /// Custom RAG configuration file
        #[arg(long, value_name = "FILE")]
        rag_config: Option<PathBuf>,

        /// Override vector storage dimensions
        #[arg(long, value_name = "SIZE")]
        rag_dims: Option<usize>,

        /// Override vector storage backend (hnsw, mock)
        #[arg(long, value_name = "BACKEND")]
        rag_backend: Option<String>,
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

    /// Run example applications
    Apps {
        /// Application to run (file-organizer, research-collector, etc.)
        #[command(subcommand)]
        app: Option<AppsSubcommand>,
    },

    /// Interactive setup for first-time users
    Setup {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Debug a script with interactive debugging
    Debug {
        /// Script to debug
        script: PathBuf,

        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },
}

/// Available example applications
#[derive(Subcommand, Debug)]
pub enum AppsSubcommand {
    /// List all available applications
    List,

    /// Organize messy files with AI categorization
    FileOrganizer {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Research any topic thoroughly
    ResearchCollector {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Create content efficiently
    ContentCreator {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Manage business communications
    CommunicationManager {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Orchestrate complex processes
    ProcessOrchestrator {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Review code for quality and security
    CodeReviewAssistant {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Create complete web applications
    WebappCreator {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Personal knowledge management with semantic search
    KnowledgeBase {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// AI-powered personal productivity assistant
    PersonalAssistant {
        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },
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
