//! # CLI Argument Parsing and Command Structures - Phase 9.4.4 Complete Restructure
//!
//! This module implements the complete CLI architecture as specified in
//! `docs/technical/cli-command-architecture.md`. It provides a professional command-line
//! interface with hierarchical subcommands, consistent output formatting, and comprehensive
//! argument validation.
//!
//! ## Architecture Highlights
//!
//! - **Global Flags**: `--config`, `--profile`, `--trace`, `--output` available on all commands
//! - **Unified Profile System**: Single `--profile` / `-p` flag for all builtin configurations
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
//! llmspell -p rag-dev repl
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
#[command(version)] // Default version for --version flag
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

    /// Built-in configuration profile (GLOBAL)
    ///
    /// Supports three syntax forms:
    ///   1. Single preset:      llmspell -p minimal
    ///   2. Explicit preset:    llmspell -p presets/rag-dev
    ///   3. Multi-layer:        llmspell -p bases/cli,features/rag,envs/dev
    ///
    /// Available presets (20 total):
    ///
    /// Backward Compatible (12):
    ///   minimal              - Tools only, no LLM features
    ///   development          - Dev environment with cloud LLM providers
    ///   providers            - All LLM providers (OpenAI, Anthropic, Gemini, Ollama, Candle)
    ///   state                - State persistence + sessions
    ///   sessions             - Session management with artifacts
    ///   ollama               - Local Ollama models
    ///   candle               - Local Candle ML models
    ///   memory               - Adaptive memory system
    ///   rag-dev              - RAG development with trace logging
    ///   rag-prod             - RAG production with SQLite
    ///   rag-perf             - RAG performance tuned
    ///   default              - Minimal CLI setup
    ///
    /// New Combinations (8):
    ///   postgres-prod        - Production PostgreSQL backend
    ///   daemon-dev           - Daemon mode development
    ///   daemon-prod          - Daemon mode production
    ///   gemini-prod          - Full Phase 13 stack + Gemini
    ///   openai-prod          - Full Phase 13 stack + OpenAI
    ///   claude-prod          - Full Phase 13 stack + Claude/Anthropic
    ///   full-local-ollama    - Complete local stack (Ollama + SQLite)
    ///   research             - Full features + trace logging
    ///
    /// Multi-layer composition uses 4 layer types:
    ///   bases/*      - Deployment modes (cli, daemon, embedded, testing)
    ///   features/*   - Capabilities (minimal, llm, rag, memory, full)
    ///   envs/*       - Tuning (dev, staging, prod, perf)
    ///   backends/*   - Storage (memory, sqlite, postgres)
    ///
    /// Use 'llmspell config list-profiles' for detailed information.
    ///
    /// Precedence: --profile > -c > discovery > default
    #[arg(short = 'p', long, global = true)]
    pub profile: Option<String>,

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
    llmspell -p rag-prod run ml.lua            # Use production RAG profile
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

        /// Script arguments
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Execute inline code
    #[command(long_about = "Execute code directly from the command line.

EXAMPLES:
    llmspell exec \"print('hello world')\"      # Execute Lua code
    llmspell exec \"console.log('test')\" --engine javascript  # Execute JavaScript
    llmspell -p development exec \"agent.query('What is 2+2?')\"  # Use development profile
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
    },

    /// Start interactive REPL
    #[command(long_about = "Start an interactive Read-Eval-Print Loop for scripting.

EXAMPLES:
    llmspell repl                              # Start Lua REPL
    llmspell repl --engine javascript         # Start JavaScript REPL
    llmspell repl --history ~/.llmspell_history  # Use custom history file
    llmspell repl --connect localhost:9555    # Connect to remote kernel
    llmspell -p rag-prod repl                  # Use production RAG profile")]
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

    /// Tool management and direct invocation
    #[command(
        long_about = "Manage and execute tools directly via kernel communication.

Tools are executed in the kernel process which has access to the ComponentRegistry.
The CLI sends tool requests to the kernel and displays the results.

EXAMPLES:
    llmspell tool list                         # List all available tools
    llmspell tool list --category filesystem   # List filesystem tools
    llmspell tool info calculator              # Show tool details and schema
    llmspell tool invoke calculator --params '{\"expression\":\"2+2\"}'  # Execute tool
    llmspell tool search \"file\" \"web\"          # Search tools by keywords
    llmspell tool test calculator --verbose    # Test tool with examples"
    )]
    Tool {
        #[command(subcommand)]
        command: ToolCommands,

        /// Tool source (future: local|mcp:\<server\>|a2a:\<node\>)
        #[arg(long, default_value = "local", hide = true)]
        source: String,
    },

    /// Manage local LLM models (Ollama and Candle)
    #[command(long_about = "Manage local LLM models with Ollama and Candle backends.

Local models provide inference without cloud dependencies. Ollama manages GGUF models
while Candle supports native Rust inference.

EXAMPLES:
    llmspell model list                        # List all local models
    llmspell model list --backend ollama       # List only Ollama models
    llmspell model pull llama3.1:8b@ollama     # Download Llama 3.1 8B via Ollama
    llmspell model status                      # Check backend health
    llmspell model info llama3.1:8b            # Show model details")]
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },

    /// Manage and execute templates
    #[command(
        long_about = "Discover, configure, and execute production-ready AI agent templates.

Templates provide turn-key solutions for common workflows by combining agents, tools,
RAG, and workflows into pre-configured patterns. Available templates include:
- Research Assistant (4-phase: gather → ingest → synthesize → validate)
- Interactive Chat (session-based conversation)
- Data Analysis (stats + visualization)
- Code Generator (spec → impl → test)
- Document Processor (PDF/OCR + transformation)
- Workflow Orchestrator (custom patterns)

EXAMPLES:
    llmspell template list                       # List all available templates
    llmspell template list --category Research   # Filter by category
    llmspell template info research-assistant    # Show template details
    llmspell template info research-assistant --show-schema  # Include parameter schema
    llmspell template exec research-assistant --param topic=\"Rust async\" --param max_sources=15
    llmspell template search \"research\" \"citations\"  # Search templates
    llmspell template schema research-assistant  # Show JSON schema"
    )]
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },

    /// Memory management operations
    #[command(long_about = "Manage episodic and semantic memory systems.

Memory operations enable persistent conversation history (episodic) and knowledge graph
management (semantic). The system automatically consolidates episodic memories into
structured semantic knowledge.

EXAMPLES:
    llmspell memory add session-1 user \"What is Rust?\"  # Add memory entry
    llmspell memory search \"async programming\"            # Search memories
    llmspell memory query \"Rust types\"                    # Query knowledge graph
    llmspell memory stats                                  # Show statistics
    llmspell memory consolidate --session-id session-1     # Trigger consolidation")]
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Context assembly operations
    #[command(
        long_about = "Assemble context for LLM prompts using retrieval strategies.

Context assembly intelligently combines episodic memory (conversation history) and
semantic memory (knowledge graph) to build relevant context within token budgets.

Strategies:
  - hybrid:   Combines episodic and semantic (recommended)
  - episodic: Conversation history only
  - semantic: Knowledge graph entities only

EXAMPLES:
    llmspell context assemble \"What is Rust?\"            # Assemble context
    llmspell context assemble \"async\" --strategy episodic  # Use specific strategy
    llmspell context strategies                           # List available strategies
    llmspell context analyze \"memory systems\" --budget 2000  # Analyze token usage"
    )]
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },

    /// Storage migration and management operations
    #[command(
        long_about = "Manage storage migrations between backends (SQLite, PostgreSQL).

Migration operations enable safe data migration with plan-based workflow, validation,
and rollback capabilities. Phase 1 supports SQLite→PostgreSQL for Agent State, Workflow
State, and Sessions.

Workflow:
  1. Generate plan: Analyze source data and create migration plan
  2. Review plan:   Inspect YAML plan for correctness
  3. Dry-run:       Validate migration without data modification
  4. Execute:       Perform actual migration with progress tracking
  5. Validate:      Verify data integrity post-migration

EXAMPLES:
    llmspell storage migrate plan --from sqlite --to postgres \\
      --components agent_state,workflow_state,sessions \\
      --output migration-plan.toml

    llmspell storage migrate execute --plan migration-plan.toml --dry-run

    llmspell storage migrate execute --plan migration-plan.toml

    llmspell storage info --backend sqlite

    llmspell storage validate --backend postgres --components agent_state"
    )]
    Storage {
        #[command(subcommand)]
        command: StorageCommands,
    },

    /// Display version information
    #[command(long_about = "Display detailed version and build information.

EXAMPLES:
    llmspell version                    # Show version information
    llmspell version --verbose          # Show detailed build information
    llmspell version --short            # Show just the version number
    llmspell version --client           # Show client version only
    llmspell version --output json      # Output as JSON")]
    Version(crate::commands::version::VersionCommand),
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

/// Tool management subcommands
#[derive(Subcommand, Debug)]
pub enum ToolCommands {
    /// List available tools with filtering
    #[command(
        long_about = "List all tools registered in the kernel's ComponentRegistry.

EXAMPLES:
    llmspell tool list                         # List all tools
    llmspell tool list --category filesystem   # Filter by category
    llmspell tool list --format json           # Output in JSON format"
    )]
    List {
        /// Filter by tool category
        #[arg(long)]
        category: Option<String>, // Will be parsed to ToolCategory in handler

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Show detailed tool information
    #[command(
        long_about = "Display detailed information about a specific tool including schema.

EXAMPLES:
    llmspell tool info calculator              # Show calculator tool details
    llmspell tool info file_operations --show-schema  # Include input/output schema"
    )]
    Info {
        /// Tool name to show information for
        name: String,

        /// Show detailed input/output schema
        #[arg(long)]
        show_schema: bool,
    },

    /// Invoke tool directly with parameters
    #[command(
        long_about = "Execute a tool directly by sending a request to the kernel.

The kernel accesses the ComponentRegistry and executes the tool with proper context.

EXAMPLES:
    llmspell tool invoke calculator --params '{\"expression\":\"sqrt(16)\"}'
    llmspell tool invoke web_scraper --params '{\"url\":\"example.com\"}' --stream
    llmspell tool invoke file_operations --params '{\"operation\":\"list\",\"path\":\"/tmp\"}'"
    )]
    Invoke {
        /// Tool name to invoke
        name: String,

        /// Parameters as JSON object
        #[arg(long, value_parser = parse_json_value)]
        params: serde_json::Value,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,
    },

    /// Search tools by capability/keywords
    #[command(
        long_about = "Search for tools by keywords, capabilities, or descriptions.

EXAMPLES:
    llmspell tool search \"file\"                # Search for file-related tools
    llmspell tool search \"web\" \"api\"           # Search for web or API tools
    llmspell tool search \"json\" --category data  # Search with category filter"
    )]
    Search {
        /// Search keywords (can specify multiple)
        query: Vec<String>,

        /// Filter by tool category
        #[arg(long)]
        category: Option<String>,
    },

    /// Test tool with example inputs
    #[command(long_about = "Test a tool using its built-in example cases.

Tools provide test cases that demonstrate their functionality.

EXAMPLES:
    llmspell tool test calculator              # Run calculator tests
    llmspell tool test file_operations --verbose  # Show detailed test output")]
    Test {
        /// Tool name to test
        name: String,

        /// Show detailed test output
        #[arg(long)]
        verbose: bool,
    },
}

/// Model management subcommands
#[derive(Subcommand, Debug)]
pub enum ModelCommands {
    /// List installed local models
    #[command(
        long_about = "List all models installed locally with optional filtering.

EXAMPLES:
    llmspell model list                        # List all models
    llmspell model list --backend ollama       # List only Ollama models
    llmspell model list --verbose              # Show sizes and dates
    llmspell model list --format json          # Output in JSON format"
    )]
    List {
        /// Filter by backend (ollama, candle, or all)
        #[arg(long, default_value = "all")]
        backend: String,

        /// Show verbose output with sizes and dates
        #[arg(long, short)]
        verbose: bool,

        /// Output format override
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Download a model
    #[command(long_about = "Download a model from the specified backend.

Model specifications follow the format: model:variant@backend
- model: Base model name (e.g., llama3.1, mistral, phi3)
- variant: Model variant/size (e.g., 8b, 7b, 13b)
- backend: Backend to use (ollama or candle)

EXAMPLES:
    llmspell model available                   # List models from backend libraries
    llmspell model pull llama3.1:8b@ollama     # Download Llama 3.1 8B via Ollama
    llmspell model pull mistral:7b@candle      # Download Mistral 7B via Candle
    llmspell model pull phi3@ollama --force    # Force re-download

Browse models online:
  Ollama:  https://ollama.com/library
  Candle:  https://huggingface.co/models?pipeline_tag=text-generation")]
    Pull {
        /// Model specification (e.g., \"llama3.1:8b@ollama\")
        model: String,

        /// Force re-download even if exists
        #[arg(long, short)]
        force: bool,

        /// Quantization level for Candle models
        #[arg(long, default_value = "Q4_K_M")]
        quantization: String,
    },

    /// Remove a model
    #[command(long_about = "Remove a local model to free disk space.

EXAMPLES:
    llmspell model remove llama3.1:8b          # Remove Llama 3.1 8B
    llmspell model remove mistral:7b --yes     # Skip confirmation")]
    Remove {
        /// Model identifier
        model: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Show model information
    #[command(long_about = "Display detailed information about a specific model.

EXAMPLES:
    llmspell model info llama3.1:8b            # Show Llama 3.1 info
    llmspell model info phi3 --format json     # JSON output")]
    Info {
        /// Model identifier
        model: String,
    },

    /// List available models from library
    #[command(
        long_about = "List models available for download from backend libraries.

EXAMPLES:
    llmspell model available                   # List all available models
    llmspell model available --backend ollama  # List Ollama library
    llmspell model available --recommended     # Show only recommended"
    )]
    Available {
        /// Backend to query (ollama or candle)
        #[arg(long)]
        backend: Option<String>,

        /// Show only recommended models
        #[arg(long)]
        recommended: bool,
    },

    /// Check local LLM installation status
    #[command(long_about = "Check health and status of local LLM backends.

Shows whether Ollama and Candle backends are available, their versions,
and the number of models installed.

EXAMPLES:
    llmspell model status                      # Check all backends
    llmspell model status --format json        # JSON output")]
    Status,

    /// Install Ollama binary (macOS and Linux only)
    #[command(long_about = "Download and install the Ollama binary.

This command downloads the official Ollama installer and sets up the binary.
macOS and Linux only.

EXAMPLES:
    llmspell model install-ollama              # Install Ollama")]
    InstallOllama,
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

    /// List available builtin profiles
    #[command(
        long_about = "Display all available builtin configuration profiles with detailed metadata.

EXAMPLES:
    llmspell config list-profiles                    # List all profiles
    llmspell config list-profiles --detailed         # Show full metadata for each profile
    llmspell config list-profiles --output json      # Output in JSON format"
    )]
    ListProfiles {
        /// Show detailed profile information (use cases, features)
        #[arg(long, short = 'd')]
        detailed: bool,
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

/// Template management subcommands
#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// List available templates
    #[command(long_about = "List all templates with optional category filtering.

EXAMPLES:
    llmspell template list                       # List all templates
    llmspell template list --category Research   # Filter by category
    llmspell template list --format json         # Output as JSON")]
    List {
        /// Filter by template category (Research, Chat, Analysis, CodeGen, Document, Workflow)
        #[arg(long)]
        category: Option<String>,

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Show detailed template information
    #[command(long_about = "Display detailed information about a specific template.

EXAMPLES:
    llmspell template info research-assistant    # Show template details
    llmspell template info research-assistant --show-schema  # Include parameter schema
    llmspell template info interactive-chat --format json    # JSON output")]
    Info {
        /// Template ID to show information for
        name: String,

        /// Show detailed parameter schema
        #[arg(long)]
        show_schema: bool,
    },

    /// Execute template with parameters
    #[command(long_about = "Execute a template with specified parameters.

Parameters can be provided as --param key=value flags. Values are parsed as JSON first,
falling back to strings. Complex values should use JSON syntax.

EXAMPLES:
    llmspell template exec research-assistant --param topic=\"Rust async runtime design\"
    llmspell template exec research-assistant --param topic=\"AI safety\" --param max_sources=20
    llmspell template exec data-analysis --param data_file=\"data.csv\" --param chart_type=\"bar\"
    llmspell template exec research-assistant --param topic=\"Quantum\" --output-dir /tmp/results")]
    Exec {
        /// Template ID to execute
        name: String,

        /// Template parameters in key=value format (repeatable)
        #[arg(long = "param", value_parser = parse_key_val::<String, String>)]
        params: Vec<(String, String)>,

        /// Output directory for artifacts
        #[arg(long, short = 'o')]
        output_dir: Option<std::path::PathBuf>,
    },

    /// Search templates by keywords
    #[command(
        long_about = "Search for templates by keywords in name, description, or tags.

EXAMPLES:
    llmspell template search \"research\"         # Search for research templates
    llmspell template search \"code\" \"generator\" # Multiple keywords
    llmspell template search \"data\" --category Analysis  # Filter by category"
    )]
    Search {
        /// Search keywords (can specify multiple)
        query: Vec<String>,

        /// Filter by template category
        #[arg(long)]
        category: Option<String>,
    },

    /// Show template parameter schema
    #[command(
        long_about = "Display the parameter schema for a template in JSON format.

The schema shows parameter types, defaults, constraints, and validation rules.

EXAMPLES:
    llmspell template schema research-assistant  # Show parameter schema
    llmspell template schema interactive-chat > schema.json  # Save to file"
    )]
    Schema {
        /// Template ID to show schema for
        name: String,
    },
}

/// Memory management subcommands
#[derive(Subcommand, Debug)]
pub enum MemoryCommands {
    /// Add entry to episodic memory
    #[command(long_about = "Add a new entry to episodic memory.

EXAMPLES:
    llmspell memory add session-1 user \"What is Rust?\"
    llmspell memory add session-1 assistant \"Rust is a systems programming language.\"
    llmspell memory add session-1 user \"Tell me more\" --metadata '{\"importance\": 5}'")]
    Add {
        /// Session ID for this memory entry
        session_id: String,

        /// Role (user, assistant, system)
        role: String,

        /// Memory content
        content: String,

        /// Optional metadata as JSON
        #[arg(long)]
        metadata: Option<String>,
    },

    /// Search episodic memory
    #[command(long_about = "Search episodic memory using vector similarity.

EXAMPLES:
    llmspell memory search \"Rust programming\"           # Search all sessions
    llmspell memory search \"async\" --session-id session-1  # Search specific session
    llmspell memory search \"error handling\" --limit 20    # Limit results
    llmspell memory search \"vectors\" --format json        # JSON output")]
    Search {
        /// Search query
        query: String,

        /// Filter by session ID
        #[arg(long)]
        session_id: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Query semantic knowledge graph
    #[command(long_about = "Query the semantic knowledge graph for entities.

EXAMPLES:
    llmspell memory query \"Rust\"                  # Query for Rust entities
    llmspell memory query \"async patterns\" --limit 15  # Limit results
    llmspell memory query \"types\" --format json       # JSON output")]
    Query {
        /// Query text
        query: String,

        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Show memory statistics
    #[command(long_about = "Display memory system statistics.

EXAMPLES:
    llmspell memory stats              # Show all statistics
    llmspell memory stats --format json  # JSON output")]
    Stats {
        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Consolidate episodic to semantic memory
    #[command(
        long_about = "Trigger consolidation of episodic memories into semantic knowledge.

EXAMPLES:
    llmspell memory consolidate                     # Consolidate all sessions
    llmspell memory consolidate --session-id session-1  # Specific session
    llmspell memory consolidate --force             # Force immediate consolidation"
    )]
    Consolidate {
        /// Session ID to consolidate (empty = all sessions)
        #[arg(long)]
        session_id: Option<String>,

        /// Force immediate consolidation
        #[arg(long)]
        force: bool,
    },
}

/// Context assembly subcommands
#[derive(Subcommand, Debug)]
pub enum ContextCommands {
    /// Assemble context for a query
    #[command(long_about = "Assemble context using the specified retrieval strategy.

Strategies:
  - hybrid:   Combines episodic and semantic memory (recommended)
  - episodic: Conversation history only
  - semantic: Knowledge graph entities only

EXAMPLES:
    llmspell context assemble \"What is Rust?\"                     # Use hybrid strategy
    llmspell context assemble \"async\" --strategy episodic          # Episodic only
    llmspell context assemble \"types\" --budget 2000 --session-id session-1  # With budget and session
    llmspell context assemble \"memory\" --format json               # JSON output")]
    Assemble {
        /// Query for context assembly
        query: String,

        /// Retrieval strategy (hybrid, episodic, semantic)
        #[arg(long)]
        strategy: Option<String>,

        /// Token budget for context
        #[arg(long, default_value = "1000")]
        budget: usize,

        /// Filter by session ID
        #[arg(long)]
        session_id: Option<String>,

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// List available context strategies
    #[command(long_about = "List all available context assembly strategies.

EXAMPLES:
    llmspell context strategies              # List strategies
    llmspell context strategies --format json  # JSON output")]
    Strategies {
        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Analyze token usage by strategy
    #[command(long_about = "Analyze estimated token usage for each strategy.

EXAMPLES:
    llmspell context analyze \"Rust async\" --budget 2000  # Analyze strategies
    llmspell context analyze \"memory systems\" --format json  # JSON output")]
    Analyze {
        /// Query for analysis
        query: String,

        /// Token budget
        #[arg(long, default_value = "1000")]
        budget: usize,

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },
}

/// Storage management subcommands
#[derive(Subcommand, Debug)]
pub enum StorageCommands {
    /// Migrate storage data between backends
    #[command(
        long_about = "Migrate data between storage backends with plan-based workflow.

Phase 1 supports SQLite→PostgreSQL for Agent State, Workflow State, and Sessions.

Workflow:
  1. Generate plan with estimated record counts
  2. Review YAML plan file
  3. Execute dry-run for validation
  4. Execute actual migration
  5. Validate data integrity

EXAMPLES:
    llmspell storage migrate plan --from sqlite --to postgres \\
      --components agent_state,workflow_state,sessions \\
      --output migration-plan.toml

    llmspell storage migrate execute --plan migration-plan.toml --dry-run
    llmspell storage migrate execute --plan migration-plan.toml"
    )]
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },

    /// Show backend information
    #[command(
        long_about = "Display storage backend characteristics and configuration.

EXAMPLES:
    llmspell storage info --backend sqlite     # Show SQLite backend info
    llmspell storage info --backend postgres   # Show PostgreSQL backend info"
    )]
    Info {
        /// Backend to show info for (sqlite, postgres)
        #[arg(long)]
        backend: String,
    },

    /// Validate backend data integrity
    #[command(long_about = "Validate data integrity for storage components.

EXAMPLES:
    llmspell storage validate --backend sqlite --components agent_state
    llmspell storage validate --backend postgres --components agent_state,workflow_state")]
    Validate {
        /// Backend to validate (sqlite, postgres)
        #[arg(long)]
        backend: String,

        /// Components to validate (comma-separated)
        #[arg(long)]
        components: String,
    },

    /// Export storage data to JSON file
    #[command(
        long_about = "Export all storage data from PostgreSQL or SQLite to JSON format.

Exports all 10 data types (vectors, graph, patterns, state, workflows, sessions, artifacts, events, hooks).
The JSON export can be imported into any backend for migration or backup.

EXAMPLES:
    llmspell storage export --backend postgres --output backup.json
    llmspell storage export --backend sqlite --output export.json"
    )]
    Export {
        /// Backend to export from (sqlite, postgres)
        #[arg(long)]
        backend: String,

        /// Output JSON file path
        #[arg(long)]
        output: std::path::PathBuf,
    },

    /// Import storage data from JSON file
    #[command(
        long_about = "Import storage data from JSON file into PostgreSQL or SQLite backend.

Imports all data types using transaction safety (rollback on error).
The target backend must have all required migrations applied.

EXAMPLES:
    llmspell storage import --backend sqlite --input backup.json
    llmspell storage import --backend postgres --input export.json"
    )]
    Import {
        /// Backend to import into (sqlite, postgres)
        #[arg(long)]
        backend: String,

        /// Input JSON file path
        #[arg(long)]
        input: std::path::PathBuf,
    },
}

/// Migration action subcommands
#[derive(Subcommand, Debug)]
pub enum MigrateAction {
    /// Generate migration plan
    #[command(
        long_about = "Generate a YAML migration plan with estimated record counts.

The plan includes source/target configs, component list, batch sizes, validation
rules, and rollback metadata. Review the plan before executing.

EXAMPLES:
    llmspell storage migrate plan --from sqlite --to postgres \\
      --components agent_state,workflow_state,sessions \\
      --output migration-plan.toml"
    )]
    Plan {
        /// Source backend (sqlite, postgres)
        #[arg(long)]
        from: String,

        /// Target backend (sqlite, postgres)
        #[arg(long)]
        to: String,

        /// Components to migrate (comma-separated: agent_state,workflow_state,sessions)
        #[arg(long)]
        components: String,

        /// Output file for migration plan
        #[arg(long)]
        output: std::path::PathBuf,
    },

    /// Execute migration from plan
    #[command(long_about = "Execute a migration plan with optional dry-run mode.

Dry-run mode performs validation without data modification. Actual execution
includes progress reporting, validation, and automatic rollback on failure.

EXAMPLES:
    llmspell storage migrate execute --plan migration-plan.toml --dry-run
    llmspell storage migrate execute --plan migration-plan.toml")]
    Execute {
        /// Migration plan file
        #[arg(long)]
        plan: std::path::PathBuf,

        /// Dry run (validation only, no data modification)
        #[arg(long)]
        dry_run: bool,
    },
}

/// Parse key=value pairs from command line
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    U: std::str::FromStr,
    U::Err: std::fmt::Display,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid KEY=value: no '=' found in `{}`", s))?;
    Ok((
        s[..pos]
            .parse()
            .map_err(|e| format!("Invalid key in `{}`: {}", s, e))?,
        s[pos + 1..]
            .parse()
            .map_err(|e| format!("Invalid value in `{}`: {}", s, e))?,
    ))
}

/// Parse JSON value from command line argument
fn parse_json_value(s: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid JSON: {}", e))
}
