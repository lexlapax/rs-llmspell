//! ABOUTME: CLI argument parsing and command structures
//! ABOUTME: Defines the command-line interface with multi-engine support

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use tracing::Level;

/// Trace level for logging output
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TraceLevel {
    /// No trace output
    Off,
    /// Error level only
    Error,
    /// Warning level and above
    Warn,
    /// Info level and above (default)
    Info,
    /// Debug level and above
    Debug,
    /// Trace level (most verbose)
    Trace,
}

impl From<TraceLevel> for Level {
    fn from(trace_level: TraceLevel) -> Self {
        match trace_level {
            TraceLevel::Off => Level::ERROR, // There's no "OFF" level in tracing
            TraceLevel::Error => Level::ERROR,
            TraceLevel::Warn => Level::WARN,
            TraceLevel::Info => Level::INFO,
            TraceLevel::Debug => Level::DEBUG,
            TraceLevel::Trace => Level::TRACE,
        }
    }
}

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

    /// Set trace level for logging output (off|error|warn|info|debug|trace)
    #[arg(long, global = true, value_enum)]
    pub trace: Option<TraceLevel>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Option<Commands>,
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
            ScriptEngine::Javascript => "Coming in Phase 19",
            ScriptEngine::Python => "Coming in Phase 22",
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

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long)]
        connect: Option<String>,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,

        /// RAG profile to use (e.g., "production", "development")
        #[arg(long, value_name = "PROFILE")]
        rag_profile: Option<String>,

        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Execute inline script
    Exec {
        /// Script code to execute
        #[arg(value_name = "CODE")]
        code: String,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long)]
        connect: Option<String>,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,

        /// RAG profile to use (e.g., "production", "development")
        #[arg(long, value_name = "PROFILE")]
        rag_profile: Option<String>,
    },

    /// Start interactive REPL
    Repl {
        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long)]
        connect: Option<String>,

        /// History file path
        #[arg(long)]
        history: Option<PathBuf>,

        /// RAG profile to use (e.g., "production", "development")
        #[arg(long, value_name = "PROFILE")]
        rag_profile: Option<String>,
    },

    /// Available Providers
    Providers {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Show engine information
    Info {
        /// Show all engines (including unavailable)
        #[arg(long)]
        all: bool,
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

    /// Manage persistent state
    State {
        #[command(subcommand)]
        command: StateCommands,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long)]
        connect: Option<String>,
    },

    /// Manage sessions and replay
    Session {
        #[command(subcommand)]
        command: SessionCommands,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long)]
        connect: Option<String>,
    },

    /// Manage RAG (Retrieval-Augmented Generation) system
    Rag {
        #[command(subcommand)]
        command: RagCommands,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long)]
        connect: Option<String>,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Debug a script with interactive debugging
    Debug {
        /// Script to debug
        script: PathBuf,

        /// Set breakpoints (format: file:line)
        #[arg(long)]
        break_at: Vec<String>,

        /// DAP server port
        #[arg(long)]
        port: Option<u16>,

        /// RAG profile to use (e.g., "production", "development")
        #[arg(long, value_name = "PROFILE")]
        rag_profile: Option<String>,

        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Manage kernel servers
    Kernel {
        #[command(subcommand)]
        command: KernelCommands,
    },
}

/// Kernel management subcommands
#[derive(Subcommand, Debug)]
pub enum KernelCommands {
    /// Start a kernel server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "9555")]
        port: u16,

        /// Run as daemon (background process)
        #[arg(long)]
        daemon: bool,

        /// Kernel ID (generated if not provided)
        #[arg(short = 'i', long)]
        id: Option<String>,

        /// Connection file path (for Jupyter discovery)
        #[arg(short = 'f', long)]
        connection_file: Option<PathBuf>,
    },

    /// Stop a running kernel
    Stop {
        /// Kernel ID to stop (if not provided, stops all kernels)
        id: Option<String>,
    },

    /// Show kernel status
    Status {
        /// Kernel ID for detailed status (if not provided, lists all kernels)
        id: Option<String>,
    },

    /// Connect to an existing kernel
    Connect {
        /// Kernel address (e.g., "localhost:9555" or "/path/to/connection.json")
        /// If not provided, uses the last successful connection
        address: Option<String>,
    },
}

/// State management subcommands
#[derive(Subcommand, Debug)]
pub enum StateCommands {
    /// Show state value(s)
    Show {
        /// Specific state key to show (if not provided, shows all)
        key: Option<String>,
    },

    /// Clear state value(s)
    Clear {
        /// Specific state key to clear (if not provided, clears all)
        key: Option<String>,
    },

    /// Export state to file
    Export {
        /// Output file path
        file: PathBuf,
        /// Export format
        #[arg(long, value_enum, default_value = "json")]
        format: ExportFormat,
    },

    /// Import state from file
    Import {
        /// Input file path
        file: PathBuf,
        /// Merge with existing state instead of replacing
        #[arg(long)]
        merge: bool,
    },
}

/// Session management subcommands
#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// Create a new session
    Create {
        /// Session name/ID
        name: String,
        /// Session description
        #[arg(long)]
        description: Option<String>,
    },

    /// List all sessions
    List {
        /// Show detailed session information
        #[arg(long)]
        detailed: bool,
    },

    /// Show session details
    Show {
        /// Session ID to show
        id: String,
    },

    /// Replay session history
    Replay {
        /// Session ID to replay
        id: String,
        /// Start from specific step
        #[arg(long)]
        from_step: Option<usize>,
        /// Stop at specific step
        #[arg(long)]
        to_step: Option<usize>,
    },

    /// Delete session
    Delete {
        /// Session ID to delete
        id: String,
        /// Delete all sessions
        #[arg(long)]
        all: bool,
    },

    /// Export session to file
    Export {
        /// Session ID to export
        id: String,
        /// Output file path
        file: PathBuf,
        /// Export format
        #[arg(long, value_enum, default_value = "json")]
        format: ExportFormat,
    },
}

/// RAG system subcommands
#[derive(Subcommand, Debug)]
pub enum RagCommands {
    /// Ingest a document into the RAG system
    Ingest {
        /// Document ID
        id: String,
        /// Document content or file path (if prefixed with @)
        content: String,
        /// Optional metadata as JSON
        #[arg(long)]
        metadata: Option<String>,
        /// Scope for multi-tenant isolation
        #[arg(long)]
        scope: Option<String>,
    },
    /// Search for relevant documents
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Minimum score threshold
        #[arg(long, default_value = "0.5")]
        threshold: f32,
        /// Scope for multi-tenant isolation
        #[arg(long)]
        scope: Option<String>,
    },
    /// Show RAG system statistics
    Stats {
        /// Scope for multi-tenant isolation
        #[arg(long)]
        scope: Option<String>,
    },
    /// Clear RAG data
    Clear {
        /// Scope to clear (defaults to global)
        #[arg(long)]
        scope: Option<String>,
        /// Confirm the clear operation
        #[arg(long)]
        confirm: bool,
    },
    /// Index files or directories
    Index {
        /// Path to index
        path: PathBuf,
        /// Recursively index directories
        #[arg(long)]
        recursive: bool,
        /// File pattern to match (e.g., "*.md")
        #[arg(long)]
        pattern: Option<String>,
        /// Scope for multi-tenant isolation
        #[arg(long)]
        scope: Option<String>,
    },
}

/// Configuration management subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Initialize configuration file
    Init {
        /// Output path for configuration file
        #[arg(short, long, default_value = "llmspell.toml")]
        output: PathBuf,
        /// Force overwrite existing file
        #[arg(short, long)]
        force: bool,
    },

    /// Validate configuration
    Validate {
        /// Configuration file to validate
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Show current configuration
    Show {
        /// Show specific config section
        section: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value = "toml")]
        format: ConfigFormat,
    },
}

/// Export/import format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
}

/// Config format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ConfigFormat {
    /// TOML format (default for configs)
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
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
