//! # CLI Argument Parsing and Command Structures - Phase 9.4.4 Complete Restructure
//!
//! This module implements the complete CLI architecture as specified in
//! `docs/technical/cli-command-architecture.md`. It provides a professional command-line
//! interface with hierarchical subcommands, consistent output formatting, and comprehensive
//! argument validation.
//!
//! ## Architecture Highlights
//!
//! - **Global Flags**: `--config`, `--trace`, `--output` available on all commands
//! - **RAG Profile Integration**: Single `--rag-profile` flag replaces 20+ individual flags
//! - **Dual-Mode Support**: Automatic kernel context resolution (embedded vs connected)
//! - **Professional Output**: Consistent JSON/YAML/Text formatting across all commands
//! - **Contextual Help**: Command-specific help with examples and usage patterns
//!
//! ## Command Hierarchy
//!
//! ```text
//! llmspell [GLOBAL_FLAGS] <COMMAND>
//! ├── run <script> [args...]                       # Execute scripts with streaming support
//! ├── exec <code>                                  # Execute inline code
//! ├── repl [--history-file]                        # Interactive REPL sessions
//! ├── debug <script> [debug-flags]                 # Interactive debugging with DAP
//! ├── kernel {start|stop|status|connect|install-service}  # Kernel lifecycle management
//! ├── session {list|show|replay|delete}            # Session management
//! ├── config {init|validate|show}                  # Configuration management
//! ├── keys {add|list|remove}                       # API key management
//! ├── state {show|clear|export|import}             # State persistence
//! ├── app {list|info|run|search}                   # Discover and run applications
//! └── backup {create|restore|list|delete}          # Backup operations
//! ```
//!
//! ## Usage Examples
//!
//! ```bash
//! # Execute a Lua script with JSON output
//! llmspell --output json run script.lua arg1 arg2
//!
//! # Start REPL with specific RAG profile
//! llmspell --rag-profile research-assistant repl
//!
//! # Connect to remote kernel for debugging
//! llmspell --connect localhost:9572 debug script.lua --break-at main.lua:10
//!
//! # Manage configuration with YAML output
//! llmspell --output yaml config show --section rag
//! ```

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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

impl From<TraceLevel> for tracing::Level {
    fn from(trace_level: TraceLevel) -> Self {
        match trace_level {
            TraceLevel::Off => tracing::Level::ERROR, // No "OFF" level in tracing
            TraceLevel::Error => tracing::Level::ERROR,
            TraceLevel::Warn => tracing::Level::WARN,
            TraceLevel::Info => tracing::Level::INFO,
            TraceLevel::Debug => tracing::Level::DEBUG,
            TraceLevel::Trace => tracing::Level::TRACE,
        }
    }
}

/// Command-line interface for LLMSpell - Professional Architecture
#[derive(Parser, Debug)]
#[command(name = "llmspell")]
#[command(version)]
#[command(about = "LLMSpell - Scriptable LLM interactions")]
#[command(
    long_about = "LLMSpell provides scriptable LLM interactions through Lua, JavaScript, and Python engines.

EXAMPLES:
    llmspell run script.lua                    # Execute a Lua script
    llmspell exec \"print('hello')\"             # Execute inline code
    llmspell repl                              # Start interactive REPL
    llmspell debug script.lua --break-at main.lua:10  # Debug with breakpoints

    llmspell kernel start --port 9555         # Start kernel server
    llmspell state show --kernel localhost:9555  # Show state from remote kernel
    llmspell config init --force               # Initialize configuration

For more help on specific commands:
    llmspell <command> --help"
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Configuration file (GLOBAL)
    #[arg(short = 'c', long, global = true, env = "LLMSPELL_CONFIG")]
    pub config: Option<PathBuf>,

    /// Trace level (replaces --debug/--verbose)
    #[arg(long, global = true, value_enum, default_value = "warn")]
    pub trace: TraceLevel,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "text")]
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
            ScriptEngine::Javascript => false, // Phase 15
            ScriptEngine::Python => false,     // Phase 22
        }
    }

    pub fn availability_message(&self) -> &'static str {
        match self {
            ScriptEngine::Lua => "Available",
            ScriptEngine::Javascript => "Coming in Phase 15",
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
    /// YAML output
    Yaml,
    /// Pretty-printed output
    Pretty,
}

/// Service type for install-service command
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ServiceType {
    /// systemd (Linux)
    Systemd,
    /// launchd (macOS)
    Launchd,
    /// Auto-detect based on platform
    Auto,
}

/// Primary execution commands and subcommand groups
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a script file
    #[command(long_about = "Execute a script file with the specified engine.

EXAMPLES:
    llmspell run script.lua                    # Execute Lua script
    llmspell run script.lua -- arg1 arg2      # Pass arguments to script
    llmspell run script.js --engine javascript # Execute JavaScript script
    llmspell run ml.lua --rag-profile production  # Use production RAG profile
    llmspell run script.lua --connect localhost:9555  # Execute on remote kernel
    llmspell run script.lua --stream           # Enable streaming output")]
    Run {
        /// Script file to execute
        script: PathBuf,

        /// Script engine to use
        #[arg(long, value_enum, default_value = "lua", env = "LLMSPELL_ENGINE")]
        engine: ScriptEngine,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long, value_name = "ADDRESS")]
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

    /// Execute inline code
    #[command(long_about = "Execute code directly from the command line.

EXAMPLES:
    llmspell exec \"print('hello world')\"      # Execute Lua code
    llmspell exec \"console.log('test')\" --engine javascript  # Execute JavaScript
    llmspell exec \"agent.query('What is 2+2?')\"  # Use LLM agent
    llmspell exec \"print('test')\" --connect localhost:9555  # Execute on remote kernel
    llmspell exec \"process_data()\" --stream   # Enable streaming output")]
    Exec {
        /// Code to execute
        #[arg(value_name = "CODE")]
        code: String,

        /// Script engine to use
        #[arg(long, value_enum, default_value = "lua", env = "LLMSPELL_ENGINE")]
        engine: ScriptEngine,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long, value_name = "ADDRESS")]
        connect: Option<String>,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,

        /// RAG profile to use (e.g., "production", "development")
        #[arg(long, value_name = "PROFILE")]
        rag_profile: Option<String>,
    },

    /// Start interactive REPL
    #[command(long_about = "Start an interactive Read-Eval-Print Loop for scripting.

EXAMPLES:
    llmspell repl                              # Start Lua REPL
    llmspell repl --engine javascript         # Start JavaScript REPL
    llmspell repl --history ~/.llmspell_history  # Use custom history file
    llmspell repl --connect localhost:9555    # Connect to remote kernel
    llmspell repl --rag-profile production    # Use production RAG profile")]
    Repl {
        /// Script engine to use
        #[arg(long, value_enum, default_value = "lua", env = "LLMSPELL_ENGINE")]
        engine: ScriptEngine,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long, value_name = "ADDRESS")]
        connect: Option<String>,

        /// History file path
        #[arg(long)]
        history: Option<PathBuf>,

        /// RAG profile to use (e.g., "production", "development")
        #[arg(long, value_name = "PROFILE")]
        rag_profile: Option<String>,
    },

    /// Debug a script with interactive debugging
    #[command(
        long_about = "Debug a script with breakpoints, watch expressions, and step mode.

EXAMPLES:
    llmspell debug script.lua --break-at main.lua:10  # Set breakpoint at line 10
    llmspell debug app.lua --watch \"state.counter\" --step  # Watch variable and step mode
    llmspell debug test.lua --break-at test.lua:5 --break-at lib.lua:20  # Multiple breakpoints
    llmspell debug remote.lua --port 9555     # Start DAP server for IDE attachment
    llmspell debug script.lua --connect localhost:9555  # Debug on remote kernel"
    )]
    Debug {
        /// Script to debug
        script: PathBuf,

        /// Script engine to use
        #[arg(long, value_enum, default_value = "lua", env = "LLMSPELL_ENGINE")]
        engine: ScriptEngine,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long, value_name = "ADDRESS")]
        connect: Option<String>,

        /// Set breakpoints (format: file:line)
        #[arg(long)]
        break_at: Vec<String>,

        /// Watch expressions (repeatable)
        #[arg(long)]
        watch: Vec<String>,

        /// Start in step mode
        #[arg(long)]
        step: bool,

        /// DAP server port for IDE attachment
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
    #[command(long_about = "Manage kernel processes for multi-client execution.

EXAMPLES:
    llmspell kernel start --port 9555 --daemon  # Start kernel server
    llmspell kernel status                       # List all running kernels
    llmspell kernel status abc123                # Show detailed status
    llmspell kernel stop abc123                 # Stop specific kernel
    llmspell kernel connect localhost:9555      # Connect to external kernel")]
    Kernel {
        #[command(subcommand)]
        command: KernelCommands,
    },

    /// Manage persistent state
    #[command(long_about = "Manage persistent state across script executions.

EXAMPLES:
    llmspell state show --kernel abc123 --scope global    # Show state from running kernel
    llmspell state show --config production.toml --scope session  # Show state from config file
    llmspell state clear --key user.preferences           # Clear specific key
    llmspell state export --kernel abc123 > state.json    # Export state
    llmspell state import --kernel abc123 < backup.json   # Import state")]
    State {
        #[command(subcommand)]
        command: StateCommands,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long, value_name = "ADDRESS")]
        connect: Option<String>,

        /// Use specific kernel by ID
        #[arg(long, value_name = "ID")]
        kernel: Option<String>,
    },

    /// Manage sessions and replay
    #[command(long_about = "Manage execution sessions with replay capabilities.

EXAMPLES:
    llmspell session list --kernel abc123         # List sessions from specific kernel
    llmspell session show sess_xyz --kernel abc123  # Show session details
    llmspell session replay sess_xyz --kernel abc123  # Replay session
    llmspell session delete old_session --kernel abc123  # Delete old session")]
    Session {
        #[command(subcommand)]
        command: SessionCommands,

        /// Connect to external kernel (e.g., "localhost:9555" or "/path/to/connection.json")
        #[arg(long, value_name = "ADDRESS")]
        connect: Option<String>,

        /// Use specific kernel by ID
        #[arg(long, value_name = "ID")]
        kernel: Option<String>,
    },

    /// Configuration management
    #[command(long_about = "Manage LLMSpell configuration files.

EXAMPLES:
    llmspell config init --force               # Initialize configuration
    llmspell config validate --file custom.toml  # Validate configuration
    llmspell config show --section rag        # Show specific config section")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// API key management
    #[command(long_about = "Manage API keys for LLM providers.

EXAMPLES:
    llmspell keys add openai sk-1234567890     # Add OpenAI API key
    llmspell keys list                         # List configured providers
    llmspell keys remove anthropic             # Remove Anthropic API key")]
    Keys {
        #[command(subcommand)]
        command: KeysCommands,
    },

    /// Backup and restore operations
    #[command(long_about = "Create and restore backups of your LLMSpell data.

EXAMPLES:
    llmspell backup create --output my_backup.tar.gz  # Create backup
    llmspell backup restore my_backup.tar.gz          # Restore from backup
    llmspell backup list                               # List available backups
    llmspell backup delete backup_001                  # Delete old backup")]
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },

    /// Manage and run applications
    #[command(
        long_about = "Discover, list, and run applications with filesystem-based discovery.

EXAMPLES:
    llmspell app list                          # List all available applications
    llmspell app info file-organizer           # Show app metadata and details
    llmspell app run file-organizer            # Run file organizer app
    llmspell app run research-collector -- --verbose  # Run with app arguments
    llmspell app search --tag productivity     # Search apps by tag
    llmspell app search --complexity Simple    # Search apps by complexity"
    )]
    App {
        #[command(subcommand)]
        command: AppCommands,

        /// Additional search paths for applications
        #[arg(long, value_name = "PATH", action = clap::ArgAction::Append)]
        search_path: Vec<String>,
    },
}

/// Application management subcommands
#[derive(Subcommand, Debug)]
pub enum AppCommands {
    /// List all available applications
    #[command(long_about = "List all applications discovered in search paths.

EXAMPLES:
    llmspell app list                          # List all apps
    llmspell app list --format json           # List in JSON format")]
    List,

    /// Show detailed information about an application
    #[command(
        long_about = "Show detailed metadata and information about a specific application.

EXAMPLES:
    llmspell app info file-organizer           # Show file-organizer details
    llmspell app info webapp-creator --format json  # Show details in JSON"
    )]
    Info {
        /// Application name to show information for
        #[arg(value_name = "APP")]
        name: String,
    },

    /// Run an application
    #[command(long_about = "Execute an application with optional arguments.

EXAMPLES:
    llmspell app run file-organizer            # Run file organizer
    llmspell app run research-collector -- --verbose  # Run with arguments")]
    Run {
        /// Application name to run
        #[arg(value_name = "APP")]
        name: String,

        /// Application arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Search applications by criteria
    #[command(
        long_about = "Search applications by tags, complexity, or other criteria.

EXAMPLES:
    llmspell app search --tag productivity     # Search by tag
    llmspell app search --complexity Simple    # Search by complexity
    llmspell app search --agents 2             # Search by agent count"
    )]
    Search {
        /// Search by tag
        #[arg(long, value_name = "TAG")]
        tag: Option<String>,

        /// Search by complexity level
        #[arg(long, value_name = "LEVEL")]
        complexity: Option<String>,

        /// Search by number of agents
        #[arg(long, value_name = "COUNT")]
        agents: Option<u32>,
    },
}

/// Kernel management subcommands
#[derive(Subcommand, Debug)]
pub enum KernelCommands {
    /// Start kernel server
    #[command(long_about = "Start a kernel server for multi-client execution.

EXAMPLES:
    llmspell kernel start --port 9555 --daemon  # Start daemon on port 9555
    llmspell kernel start --id my-kernel        # Start with custom ID
    llmspell kernel start --daemon --log-file /var/log/kernel.log  # With logging
    llmspell kernel start --daemon --idle-timeout 7200  # 2 hour idle timeout")]
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

        /// Log file path (for daemon mode)
        #[arg(long)]
        log_file: Option<PathBuf>,

        /// PID file path (for daemon mode)
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Idle timeout in seconds (0 = no timeout)
        #[arg(long, default_value = "3600")]
        idle_timeout: u64,

        /// Maximum concurrent clients
        #[arg(long, default_value = "10")]
        max_clients: usize,

        /// Log rotation size in bytes
        #[arg(long)]
        log_rotate_size: Option<u64>,

        /// Number of rotated log files to keep
        #[arg(long, default_value = "5")]
        log_rotate_count: usize,
    },

    /// Stop kernel by ID or PID file
    #[command(long_about = "Stop a running kernel gracefully.

EXAMPLES:
    llmspell kernel stop --id my-kernel      # Stop by kernel ID
    llmspell kernel stop --pid-file /tmp/kernel.pid  # Stop by PID file
    llmspell kernel stop --all                # Stop all kernels
    llmspell kernel stop --force             # Force kill without graceful shutdown")]
    Stop {
        /// Kernel ID to stop
        #[arg(short, long)]
        id: Option<String>,

        /// PID file path to identify kernel
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Stop all running kernels
        #[arg(long)]
        all: bool,

        /// Force immediate termination (skip graceful shutdown)
        #[arg(long)]
        force: bool,

        /// Timeout in seconds for graceful shutdown
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Don't clean up files after stopping
        #[arg(long)]
        no_cleanup: bool,
    },

    /// Show running kernels or specific kernel details
    #[command(
        long_about = "Display status of running kernels with health and resource metrics.

EXAMPLES:
    llmspell kernel status                    # List all running kernels
    llmspell kernel status --id my-kernel     # Detailed view of specific kernel
    llmspell kernel status --output json      # JSON output for scripting
    llmspell kernel status --watch            # Continuous monitoring"
    )]
    Status {
        /// Kernel ID for detailed status (if not provided, lists all kernels)
        #[arg(short, long)]
        id: Option<String>,

        /// Output format (table, json, yaml, text)
        #[arg(short = 'f', long = "format", default_value = "table")]
        format: String,

        /// Show only kernel IDs (quiet mode)
        #[arg(short, long)]
        quiet: bool,

        /// Watch mode - refresh continuously
        #[arg(short, long)]
        watch: bool,

        /// Refresh interval in seconds (for watch mode)
        #[arg(long, default_value = "5")]
        interval: u64,
    },

    /// Connect to external kernel
    Connect {
        /// Kernel address (e.g., "localhost:9555" or "/path/to/connection.json")
        /// If not provided, uses the last successful connection
        address: Option<String>,
    },

    /// Install kernel as system service
    #[command(long_about = "Generate and install systemd/launchd service files.

EXAMPLES:
    llmspell kernel install-service               # Auto-detect platform, user service
    llmspell kernel install-service --system      # Install as system service
    llmspell kernel install-service --port 9600   # Custom port
    llmspell kernel install-service --name custom # Custom service name")]
    InstallService {
        /// Service type (systemd/launchd/auto)
        #[arg(long, value_enum)]
        service_type: Option<ServiceType>,

        /// Install as system service (default: user service)
        #[arg(long)]
        system: bool,

        /// Service name
        #[arg(long, default_value = "llmspell-kernel")]
        name: String,

        /// Port for kernel
        #[arg(long, default_value = "9555")]
        port: u16,

        /// Kernel ID
        #[arg(long)]
        id: Option<String>,

        /// Log file path
        #[arg(long)]
        log_file: Option<PathBuf>,

        /// PID file path
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Enable service after installation
        #[arg(long)]
        enable: bool,

        /// Start service after installation
        #[arg(long)]
        start: bool,

        /// Override if service already exists
        #[arg(long)]
        force: bool,
    },
}

/// State management subcommands
#[derive(Subcommand, Debug)]
pub enum StateCommands {
    /// Display persisted state
    Show {
        /// Specific state key to show (if not provided, shows all)
        key: Option<String>,

        /// State scope: global|session|workflow|component
        #[arg(long)]
        scope: Option<String>,
    },

    /// Clear state by scope
    Clear {
        /// Specific state key to clear (if not provided, clears all)
        key: Option<String>,

        /// State scope: global|session|workflow|component
        #[arg(long)]
        scope: Option<String>,
    },

    /// Export state to JSON
    Export {
        /// Output file path
        file: PathBuf,

        /// Export format
        #[arg(long, value_enum, default_value = "json")]
        format: ExportFormat,
    },

    /// Import state from JSON
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

    /// Replay a session
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

    /// Delete a session
    Delete {
        /// Session ID to delete
        id: String,

        /// Delete all sessions
        #[arg(long)]
        all: bool,
    },
}

/// Configuration management subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Initialize configuration
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
        file: Option<PathBuf>,
    },

    /// Show configuration
    Show {
        /// Show specific config section
        section: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "toml")]
        format: ConfigFormat,
    },
}

/// API key management subcommands
#[derive(Subcommand, Debug)]
pub enum KeysCommands {
    /// Add API key for provider
    Add {
        /// Provider name
        provider: String,

        /// API key
        key: String,
    },

    /// List configured providers
    List,

    /// Remove API key for provider
    Remove {
        /// Provider name
        provider: String,
    },
}

/// Backup operations subcommands
#[derive(Subcommand, Debug)]
pub enum BackupCommands {
    /// Create backup
    Create {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Restore from backup
    Restore {
        /// Backup file path
        file: PathBuf,
    },

    /// List available backups
    List,

    /// Delete backup
    Delete {
        /// Backup ID
        id: String,
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

impl Cli {
    /// Validate the selected engine is available
    pub fn validate_engine(&self, engine: ScriptEngine) -> anyhow::Result<()> {
        if !engine.is_available() {
            anyhow::bail!(
                "Script engine '{}' is not available yet. {}",
                engine.as_str(),
                engine.availability_message()
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
