# LLMSpell CLI Reference

Complete user guide for all llmspell CLI commands.

## Table of Contents

1. [Overview](#overview)
2. [Global Options](#global-options)
3. [Script Execution Commands](#script-execution-commands)
   - [run](#run) - Execute script files
   - [exec](#exec) - Execute inline code
   - [repl](#repl) - Interactive REPL
   - [debug](#debug) - Debug scripts
4. [Kernel Management](#kernel-management)
   - [kernel](#kernel) - Manage kernel servers
5. [State Management](#state-management)
   - [state](#state) - Persistent state operations
   - [session](#session) - Session management
6. [Configuration](#configuration)
   - [config](#config) - Configuration management
   - [keys](#keys) - API key management
   - [backup](#backup) - Backup/restore
   - [storage](#storage) - Storage export/import
7. [Scripting Resources](#scripting-resources)
   - [app](#app) - Application management
   - [tool](#tool) - Tool operations
   - [model](#model) - Model management
   - [template](#template) - Template execution (Phase 12)
8. [Memory & Context (Phase 13)](#memory--context)
   - [memory](#memory) - Memory operations
   - [context](#context) - Context assembly

## Overview

LLMSpell provides scriptable LLM interactions via Lua/JavaScript. The CLI supports:
- Script execution (local or remote kernel)
- Interactive REPL with debug support
- Memory and context management for RAG workflows
- Template-based AI workflows
- State persistence across sessions

### Architecture (Phase 13b.16)

The CLI implements a **kernel-centric dual-mode architecture** with minimal overhead:

**Execution Modes**:
1. **Embedded Mode** (default): In-process kernel with ~12-line initialization
2. **Connected Mode**: Remote kernel connection via TCP client

**Simplified CLI Layer** (llmspell-cli/src/execution_context.rs:136-146, 169-180):
```rust
// Phase 13b.16.3: ScriptRuntime creates ALL infrastructure
let script_executor = Arc::new(
    llmspell_bridge::ScriptRuntime::new(config.clone()).await?
) as Arc<dyn ScriptExecutor>;

let handle = start_embedded_kernel_with_executor(
    config.clone(),
    script_executor,
).await?;
```

**Infrastructure Creation**: ScriptRuntime internally uses `Infrastructure::from_config()` to create all 9 components (ProviderManager, StateManager, SessionManager, RAG, MemoryManager, ToolRegistry, AgentRegistry, WorkflowFactory, ComponentRegistry). CLI has zero direct dependencies on infrastructure.

**Kernel API**: All CLI commands communicate via kernel message protocol:
- Script execution: `execute_request` → `execute_reply`
- Memory operations: `memory_request` → `memory_reply`
- Context assembly: `context_request` → `context_reply`
- Same API for embedded and remote kernels

**Service Deployment**: Daemon mode bypasses CLI entirely - services use kernel API directly (see [Service Deployment](service-deployment.md)).

## Global Options

Available for all commands:

```bash
-c, --config <CONFIG>      Configuration file
-p, --profile <PROFILE>    Built-in profile (minimal, development, providers, etc.)
--trace <LEVEL>            Trace level (off, error, warn, info, debug, trace)
--output <FORMAT>          Output format (text, json, pretty)
-h, --help                 Print help
-V, --version              Print version
```

**Built-in Profiles**:
- `minimal` - Tools only, no LLM providers
- `development` - Dev settings with debug logging
- `providers` - OpenAI + Anthropic setup
- `state` - State persistence with memory backend
- `sessions` - Sessions + state + hooks + events
- `ollama` - Ollama backend configuration
- `candle` - Candle embedded inference
- `rag-dev` - Development RAG (small dims, fast)
- `rag-prod` - Production RAG (reliability, monitoring)
- `rag-perf` - Performance RAG (high memory, cores)

## Script Execution Commands

### run

Execute a script file with the specified engine.

**Usage**:
```bash
llmspell run <SCRIPT> [OPTIONS] [-- <ARGS>...]
```

**Options**:
- `--engine <ENGINE>` - Script engine (lua, javascript, python) [default: lua]
- `--connect <ADDRESS>` - Connect to external kernel (e.g., "localhost:9555")
- `--stream` - Enable streaming output

**Examples**:
```bash
# Execute Lua script
llmspell run script.lua

# Pass arguments to script
llmspell run script.lua -- arg1 arg2

# Use production RAG profile
llmspell -p rag-prod run ml.lua

# Execute on remote kernel
llmspell run script.lua --connect localhost:9555

# Enable streaming output
llmspell run script.lua --stream
```

**Use Cases**:
- Running AI workflow scripts
- Batch processing with LLMs
- Automation tasks
- Data processing pipelines

### exec

Execute code directly from the command line.

**Usage**:
```bash
llmspell exec <CODE> [OPTIONS]
```

**Options**:
- `--engine <ENGINE>` - Script engine (lua, javascript, python) [default: lua]
- `--connect <ADDRESS>` - Connect to external kernel
- `--stream` - Enable streaming output

**Examples**:
```bash
# Execute Lua code
llmspell exec "print('hello world')"

# Use development profile
llmspell -p development exec "Agent.query('What is 2+2?')"

# Execute on remote kernel
llmspell exec "print('test')" --connect localhost:9555

# Query with JSON output
llmspell --output json exec "return Agent.query('Explain async')"
```

**Use Cases**:
- Quick one-liner executions
- Testing LLM queries
- Prototyping
- Shell scripting integration

### repl

Start an interactive REPL session.

**Usage**:
```bash
llmspell repl [OPTIONS]
```

**Options**:
- `--engine <ENGINE>` - Script engine [default: lua]
- `--connect <ADDRESS>` - Connect to external kernel
- `--history <PATH>` - Custom history file path

**Examples**:
```bash
# Start Lua REPL
llmspell repl

# REPL with remote kernel
llmspell repl --connect localhost:9555

# Custom history file
llmspell repl --history ~/.llmspell_history
```

**REPL Commands**:
- `.exit` or `.quit` - Exit REPL
- `.help` - Show help
- `.clear` - Clear screen

**Use Cases**:
- Interactive development
- Exploring API features
- Testing and debugging
- Learning Lua/LLMSpell APIs

### debug

Debug a script with interactive debugging.

**Usage**:
```bash
llmspell debug <SCRIPT> [OPTIONS] [-- <ARGS>...]
```

**Options**:
- `--engine <ENGINE>` - Script engine [default: lua]
- `--connect <ADDRESS>` - Connect to external kernel
- `--break-at <FILE:LINE>` - Set breakpoints (repeatable)
- `--watch <EXPR>` - Watch expressions (repeatable)
- `--step` - Start in step mode
- `--port <PORT>` - DAP server port for IDE attachment

**Examples**:
```bash
# Start debug session
llmspell debug script.lua

# Set breakpoints
llmspell debug script.lua --break-at script.lua:10 --break-at script.lua:25

# Watch variables
llmspell debug script.lua --watch "count" --watch "result"

# Start in step mode
llmspell debug script.lua --step

# Enable DAP for IDE
llmspell debug script.lua --port 9229
```

**Use Cases**:
- Debugging complex workflows
- Inspecting LLM responses
- Finding script errors
- IDE integration (VS Code, etc.)

## Kernel Management

### kernel

Manage kernel processes for multi-client execution.

**Usage**:
```bash
llmspell kernel <SUBCOMMAND>
```

**Subcommands**:
- `start` - Start a kernel server
- `status` - Show kernel status
- `stop` - Stop a kernel
- `list` - List all running kernels
- `connect` - Connect to external kernel

**Examples**:
```bash
# Start kernel server
llmspell kernel start --port 9555 --daemon

# List all running kernels
llmspell kernel list

# Show detailed status
llmspell kernel status abc123

# Stop specific kernel
llmspell kernel stop abc123

# Connect to external kernel
llmspell kernel connect localhost:9555
```

**Use Cases**:
- Multi-client execution
- Long-running services
- Shared kernel across processes
- Remote execution

## State Management

### state

Manage persistent state across script executions.

**Usage**:
```bash
llmspell state <SUBCOMMAND>
```

**Subcommands**:
- `get` - Get state value
- `set` - Set state value
- `delete` - Delete state value
- `list` - List all state keys
- `clear` - Clear all state

**Examples**:
```bash
# Set state value
llmspell state set config.api_key "sk-..."

# Get state value
llmspell state get config.api_key

# List all keys
llmspell state list

# Clear all state
llmspell state clear
```

**Use Cases**:
- Persisting configuration
- Storing credentials
- Maintaining workflow state
- Cross-script data sharing

### session

Manage sessions for conversation history and context.

**Usage**:
```bash
llmspell session <SUBCOMMAND>
```

**Subcommands**:
- `list` - List all sessions
- `create` - Create new session
- `delete` - Delete session
- `show` - Show session details

**Examples**:
```bash
# List all sessions
llmspell session list

# Create new session
llmspell session create --name "research-session"

# Show session details
llmspell session show session-123

# Delete session
llmspell session delete session-123
```

**Use Cases**:
- Managing conversation history
- Organizing research sessions
- Tracking LLM interactions
- Context isolation

## Configuration

### config

Manage configuration files and profiles.

**Usage**:
```bash
llmspell config <SUBCOMMAND>
```

**Subcommands**:
- `list-profiles` - List available profiles
- `show-profile` - Show profile details
- `validate` - Validate config file
- `generate` - Generate sample config

**Examples**:
```bash
# List available profiles
llmspell config list-profiles

# Show profile details
llmspell config show-profile rag-prod

# Validate config file
llmspell config validate --file config.toml

# Generate sample config
llmspell config generate > my-config.toml
```

**Use Cases**:
- Configuration management
- Profile selection
- Config validation
- Template generation

### keys

Manage API keys securely.

**Usage**:
```bash
llmspell keys <SUBCOMMAND>
```

**Subcommands**:
- `set` - Set API key
- `get` - Get API key
- `delete` - Delete API key
- `list` - List configured keys

**Examples**:
```bash
# Set API key
llmspell keys set openai sk-...

# Get API key (masked)
llmspell keys get openai

# List all keys (masked)
llmspell keys list

# Delete key
llmspell keys delete openai
```

**Use Cases**:
- Secure credential storage
- API key management
- Multi-provider setup
- Key rotation

### backup

Backup and restore LLMSpell data.

**Usage**:
```bash
llmspell backup <SUBCOMMAND>
```

**Subcommands**:
- `create` - Create backup
- `restore` - Restore from backup
- `list` - List backups

**Examples**:
```bash
# Create backup
llmspell backup create

# Create named backup
llmspell backup create --name "pre-upgrade"

# List backups
llmspell backup list

# Restore backup
llmspell backup restore backup-20250130.tar.gz
```

**Use Cases**:
- Data protection
- Migration between systems
- Version rollback
- Disaster recovery

### storage

Export and import storage data for PostgreSQL ↔ SQLite migration.

The storage command enables lossless bidirectional migration of all data between PostgreSQL
and SQLite backends. Export creates a JSON file containing all tables (vectors, knowledge
graph, sessions, artifacts, etc.), and import loads this data into the target backend.

**Usage**:
```bash
llmspell storage <SUBCOMMAND>
```

**Subcommands**:
- `export` - Export data to JSON file
- `import` - Import data from JSON file

#### EXPORT - Export storage data to JSON

```bash
llmspell storage export --backend <BACKEND> --output <FILE>
```

**Options**:
- `--backend <BACKEND>` - Source backend (sqlite, postgres)
- `--output <FILE>` - Output JSON file path

**Examples**:
```bash
# Export from SQLite
llmspell storage export --backend sqlite --output export.json

# Export from PostgreSQL (requires DATABASE_URL env var)
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell storage export --backend postgres --output pg-export.json
```

**Configuration**:
- **SQLite**: Uses `persistence_path` from config or defaults to `./storage/llmspell.db`
- **PostgreSQL**: Requires `DATABASE_URL` environment variable

**Export Format**:
- Version: 1.0 (semantic versioning)
- Contains: Vector embeddings, knowledge graph, procedural memory, agent state, KV store, workflow states, sessions, artifacts, event log, hook history
- Encoding: JSON with base64 for binary data (BLOB fields)
- Metadata: Export timestamp, source backend, applied migrations

#### IMPORT - Import storage data from JSON

```bash
llmspell storage import --backend <BACKEND> --input <FILE>
```

**Options**:
- `--backend <BACKEND>` - Target backend (sqlite, postgres)
- `--input <FILE>` - Input JSON file path

**Examples**:
```bash
# Import to SQLite
llmspell storage import --backend sqlite --input export.json

# Import to PostgreSQL (requires DATABASE_URL env var)
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell storage import --backend postgres --input pg-export.json
```

**Import Behavior**:
- Transaction-safe: All-or-nothing import with automatic rollback on errors
- Validates JSON structure before importing
- Reports detailed import statistics by data type
- Supports importing across backend versions (if migrations match)

**PostgreSQL Support**:
PostgreSQL support requires compilation with the `postgres` feature:
```bash
cargo build --features postgres
```

**Use Cases**:
- **PostgreSQL → SQLite Migration**: Move from cloud database to embedded storage
- **SQLite → PostgreSQL Migration**: Scale from embedded to shared database
- **Cross-Environment Migration**: Move data between development/staging/production
- **Data Backup**: Export to JSON for external archival
- **Disaster Recovery**: Restore from JSON to new database instance
- **Multi-Backend Testing**: Verify data consistency across backends

**Common Migration Workflows**:

1. **Development → Production Migration**:
   ```bash
   # On development machine (SQLite)
   llmspell storage export --backend sqlite --output dev-data.json

   # Transfer file to production server
   scp dev-data.json prod-server:/tmp/

   # On production server (PostgreSQL)
   export DATABASE_URL="postgresql://prod-user:pass@localhost/llmspell"
   llmspell storage import --backend postgres --input /tmp/dev-data.json
   ```

2. **Cross-Database Backup**:
   ```bash
   # Export from PostgreSQL
   llmspell storage export --backend postgres --output backup-$(date +%Y%m%d).json

   # Import to SQLite for offline backup
   llmspell storage import --backend sqlite --input backup-20250122.json
   ```

3. **Testing Data Consistency**:
   ```bash
   # Export from source
   llmspell storage export --backend sqlite --output source.json

   # Import to target
   llmspell storage import --backend postgres --input source.json

   # Export from target
   llmspell storage export --backend postgres --output target.json

   # Compare (should be identical except timestamps)
   diff <(jq -S .data source.json) <(jq -S .data target.json)
   ```

**See Also**:
- [Data Migration Guide](11-data-migration.md) - Complete migration workflows
- [Storage Setup](07-storage-setup.md) - Backend configuration
- [PostgreSQL Guide](../technical/postgresql-guide.md) - PostgreSQL-specific details
- [Storage Architecture](../technical/sqlite-vector-storage-architecture.md) - Technical details

## Scripting Resources

### app

Manage and execute embedded applications.

**Usage**:
```bash
llmspell app <SUBCOMMAND>
```

**Subcommands**:
- `list` - List available apps
- `info` - Show app information
- `run` - Run an app

**Examples**:
```bash
# List available apps
llmspell app list

# Show app info
llmspell app info file-organizer

# Run app with parameters
llmspell app run file-organizer --path ~/Documents
```

**Available Apps**:
- `file-organizer` - Organize files by type/date
- `content-creator` - Generate content with AI
- `webapp-creator` - Create web applications

**Use Cases**:
- Quick utilities
- Pre-built workflows
- Learning examples
- Productivity tools

### tool

Manage and execute tools.

**Usage**:
```bash
llmspell tool <SUBCOMMAND>
```

**Subcommands**:
- `list` - List available tools
- `info` - Show tool details
- `exec` - Execute a tool

**Examples**:
```bash
# List available tools
llmspell tool list

# Show tool info
llmspell tool info web_search

# Execute tool
llmspell tool exec web_search --query "Rust programming" --limit 5
```

**Use Cases**:
- Extending LLM capabilities
- Tool-augmented generation
- API integrations
- Custom functionality

### model

Manage LLM models.

**Usage**:
```bash
llmspell model <SUBCOMMAND>
```

**Subcommands**:
- `list` - List available models
- `info` - Show model details
- `test` - Test model connection

**Examples**:
```bash
# List available models
llmspell model list

# Show model details
llmspell model info gpt-4

# Test model connection
llmspell model test gpt-4
```

**Use Cases**:
- Model discovery
- Provider testing
- Configuration verification
- Model comparison

### template

Execute AI workflow templates (Phase 12).

**Usage**:
```bash
llmspell template <SUBCOMMAND>
```

**Subcommands**:
- `list` - List available templates
- `info` - Show template details
- `exec` - Execute a template
- `search` - Search templates by keywords
- `schema` - Show template parameter schema

**Examples**:
```bash
# List available templates
llmspell template list

# Show template info
llmspell template info research-assistant

# Execute template
llmspell template exec research-assistant \
  --param topic="Rust async" \
  --param max_sources=10

# Search templates
llmspell template search "research" "citations"

# Show parameter schema
llmspell template schema research-assistant
```

**Template Categories**:
- **Research**: research-assistant, data-analysis
- **Development**: code-generator, code-review
- **Content**: content-generation, document-processor
- **Productivity**: interactive-chat, workflow-orchestrator
- **Classification**: file-classification, knowledge-management

**Use Cases**:
- Experimental AI workflows
- Standardized processes
- Best-practice implementations
- Rapid prototyping

## Memory & Context (Phase 13)

### memory

Manage episodic and semantic memory systems.

Memory operations enable persistent conversation history (episodic) and knowledge graph
management (semantic). The system automatically consolidates episodic memories into
structured semantic knowledge.

**Architecture Note**: Memory commands use kernel message protocol. The CLI sends
`memory_request` messages to the kernel, which accesses MemoryBridge and returns
results via `memory_reply` messages. Works with both embedded and remote kernels.

**Usage**:
```bash
llmspell memory <SUBCOMMAND>
```

**Subcommands**:
- `add` - Add entry to episodic memory
- `search` - Search episodic memory
- `query` - Query semantic knowledge graph
- `stats` - Show memory statistics
- `consolidate` - Consolidate episodic to semantic memory

#### ADD - Add episodic memory entry

```bash
llmspell memory add <SESSION_ID> <ROLE> <CONTENT> [OPTIONS]
```

**Arguments**:
- `<SESSION_ID>` - Session identifier
- `<ROLE>` - Role (user, assistant, system)
- `<CONTENT>` - Memory content

**Options**:
- `--metadata <JSON>` - Optional metadata as JSON

**Examples**:
```bash
llmspell memory add session-1 user "What is Rust?"
llmspell memory add session-1 assistant "Rust is a systems programming language."
llmspell memory add session-1 user "Tell me more" --metadata '{"importance": 5}'
```

#### SEARCH - Search episodic memory

```bash
llmspell memory search <QUERY> [OPTIONS]
```

**Arguments**:
- `<QUERY>` - Search query

**Options**:
- `--session-id <ID>` - Filter by session ID
- `--limit <N>` - Maximum number of results [default: 10]
- `--format <FORMAT>` - Output format (overrides global format)

**Examples**:
```bash
llmspell memory search "Rust programming"
llmspell memory search "async" --session-id session-1
llmspell memory search "error handling" --limit 20
llmspell memory search "vectors" --format json
```

#### QUERY - Query semantic knowledge graph

```bash
llmspell memory query <QUERY> [OPTIONS]
```

**Arguments**:
- `<QUERY>` - Query text

**Options**:
- `--limit <N>` - Maximum number of results [default: 10]
- `--format <FORMAT>` - Output format (overrides global format)

**Examples**:
```bash
llmspell memory query "Rust"
llmspell memory query "async patterns" --limit 15
llmspell memory query "types" --format json
```

#### STATS - Show memory statistics

```bash
llmspell memory stats [OPTIONS]
```

**Options**:
- `--format <FORMAT>` - Output format (overrides global format)

**Examples**:
```bash
llmspell memory stats
llmspell memory stats --format json
```

**Output includes**:
- Total episodic entries
- Total semantic entities
- Sessions with unprocessed memories
- Storage usage

#### CONSOLIDATE - Consolidate episodic to semantic memory

```bash
llmspell memory consolidate [OPTIONS]
```

**Options**:
- `--session-id <ID>` - Session ID to consolidate (empty = all sessions)
- `--force` - Force immediate consolidation

**Examples**:
```bash
llmspell memory consolidate
llmspell memory consolidate --session-id session-1
llmspell memory consolidate --force
```

**Memory Message Flow**:
1. CLI parses memory command and parameters
2. CLI creates memory_request message with command/params
3. CLI sends via kernel handle (embedded) or connection (remote)
4. Kernel receives on shell channel
5. Kernel.handle_memory_request() processes request
6. Kernel accesses script_executor.memory_bridge()
7. MemoryBridge executes operation (episodic_add, search, etc.)
8. Kernel sends memory_reply with results
9. CLI receives and formats output

**Code References**:
- CLI: `llmspell-cli/src/commands/memory.rs`
- Handler: `llmspell-kernel/src/execution/integrated.rs`
- Bridge: `llmspell-bridge/src/memory_bridge.rs`
- API: `llmspell-kernel/src/api.rs`

**Use Cases**:
- Building conversation context
- Long-term memory for AI agents
- Knowledge accumulation
- Session continuity

### context

Assemble context for LLM prompts using retrieval strategies.

Context assembly intelligently combines episodic memory (conversation history) and
semantic memory (knowledge graph) to build relevant context within token budgets.

**Architecture Note**: Context commands use kernel message protocol. The CLI sends
`context_request` messages to the kernel, which accesses ContextBridge and returns
assembled context via `context_reply` messages.

**Usage**:
```bash
llmspell context <SUBCOMMAND>
```

**Subcommands**:
- `assemble` - Assemble context for a query
- `strategies` - List available context strategies
- `analyze` - Analyze token usage by strategy

#### ASSEMBLE - Assemble context with specified strategy

```bash
llmspell context assemble <QUERY> [OPTIONS]
```

**Arguments**:
- `<QUERY>` - Query for context assembly

**Options**:
- `--strategy <STRATEGY>` - Retrieval strategy [default: hybrid]
  - Options: hybrid, episodic, semantic, rag
- `--budget <N>` - Token budget [default: 1000]
- `--session-id <ID>` - Filter by session ID
- `--format <FORMAT>` - Output format (overrides global format)

**Examples**:
```bash
llmspell context assemble "What is Rust?"
llmspell context assemble "async" --strategy episodic
llmspell context assemble "types" --budget 2000 --session-id session-1
llmspell context assemble "memory" --format json
```

**Strategy Descriptions**:
- `hybrid` - Combines episodic and semantic memory (recommended)
- `episodic` - Conversation history only
- `semantic` - Knowledge graph entities only
- `rag` - Document retrieval only (if RAG enabled)

#### STRATEGIES - List available context strategies

```bash
llmspell context strategies [OPTIONS]
```

**Options**:
- `--format <FORMAT>` - Output format (overrides global format)

**Examples**:
```bash
llmspell context strategies
llmspell context strategies --format json
```

#### ANALYZE - Analyze estimated token usage

```bash
llmspell context analyze <QUERY> [OPTIONS]
```

**Arguments**:
- `<QUERY>` - Query for analysis

**Options**:
- `--budget <N>` - Token budget [default: 1000]
- `--format <FORMAT>` - Output format (overrides global format)

**Examples**:
```bash
llmspell context analyze "Rust async" --budget 2000
llmspell context analyze "memory systems" --format json
```

**Output includes**:
- Estimated tokens per strategy
- Number of chunks retrieved
- Fit within budget status
- Recommendations

**Context Message Flow**:
1. CLI parses context command and parameters
2. CLI creates context_request message with command/params
3. CLI sends via kernel handle (embedded) or connection (remote)
4. Kernel receives on shell channel
5. Kernel.handle_context_request() processes request
6. Kernel accesses script_executor.context_bridge()
7. ContextBridge executes assembly/analysis
8. Kernel sends context_reply with results
9. CLI receives and formats output (chunks, token counts)

**Code References**:
- CLI: `llmspell-cli/src/commands/context.rs`
- Handler: `llmspell-kernel/src/execution/integrated.rs`
- Bridge: `llmspell-bridge/src/context_bridge.rs`
- API: `llmspell-kernel/src/api.rs`

**Use Cases**:
- Optimizing prompt context
- RAG-enhanced queries
- Token budget management
- Strategy comparison

## See Also

- [Configuration Guide](configuration.md) - Detailed configuration options
- [Getting Started](getting-started.md) - Quick start guide
- [Template User Guides](templates/) - Template-specific documentation
- [API Reference](api/) - Lua/JavaScript API documentation
- [Memory Configuration](memory-configuration.md) - Memory system configuration
- [Technical Architecture](../technical/cli-command-architecture.md) - CLI architecture details
