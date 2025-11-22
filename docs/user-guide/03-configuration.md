# Configuration Guide

**Version**: 0.9.0
**Last Updated**: December 2024

> **📋 Quick Reference**: Complete configuration guide for LLMSpell kernel architecture including providers, daemon mode, protocols, and services.

**🔗 Navigation**: [← User Guide](README.md) | [Core Concepts](concepts.md) | [Service Deployment](service-deployment.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Builtin Profiles](#builtin-profiles) ⭐ **NEW - Zero-Config**
3. [Configuration Files](#configuration-files)
4. [Kernel Configuration](#kernel-configuration) ⭐ **Phase 9-10**
5. [Daemon & Service Settings](#daemon--service-settings) ⭐ **Phase 10**
6. [Protocol Configuration](#protocol-configuration) ⭐ **Phase 9-10**
7. [Debug & IDE Integration](#debug--ide-integration) ⭐ **Phase 9**
8. [Logging & Monitoring](#logging--monitoring) ⭐ **Phase 9-10**
9. [LLM Providers](#llm-providers)
10. [RAG Configuration](#rag-configuration) ⭐ **Phase 8**
11. [Multi-Tenancy](#multi-tenancy) ⭐ **Phase 8**
12. [State & Sessions](#state--sessions)
13. [Infrastructure Module](#infrastructure-module-phase-13b16) ⭐ **Phase 13b.16**
14. [Storage Backend Configuration](#storage-backend-configuration-phase-13b) ⭐ **Phase 13b**
15. [Security Settings](#security-settings)
16. [Tool Configuration](#tool-configuration)
17. [External API Setup](#external-api-setup)
18. [Deployment Profiles](#deployment-profiles)
19. [Environment Variables](#environment-variables)
20. [Troubleshooting](#troubleshooting)

---

## Quick Start

**Recommended**: Use builtin profiles for zero-config startup:

```bash
# Set LLM provider API key (for agent examples)
export OPENAI_API_KEY="sk-..."

# Run with builtin profile (no config file needed)
./target/release/llmspell -p minimal run script.lua        # Tools only
./target/release/llmspell -p providers run script.lua      # With LLM agents
./target/release/llmspell -p rag-dev run script.lua        # With RAG
./target/release/llmspell -p development run script.lua    # Full debug mode

# Optional: Start kernel in service mode
./target/release/llmspell kernel start --port 9555

# Advanced: Use custom configuration file
./target/release/llmspell -c config.toml run script.lua

# Debugging with trace flag (Phase 9)
./target/release/llmspell --trace debug -p development run script.lua
```

See [Builtin Profiles](#builtin-profiles) below for complete list of 20 available profiles, or [Profile Layers Guide](profile-layers-guide.md) for comprehensive documentation.

---

## Builtin Profiles

**Version 0.14.0**: LLMSpell includes **20 builtin configuration profiles** powered by a composable **4-layer architecture**. Profiles can be used directly by name, or composed from individual layers for custom configurations.

### Three Ways to Use Profiles

**1. Single preset name** (backward compatible):
```bash
llmspell -p minimal run script.lua
```

**2. Explicit preset path**:
```bash
llmspell -p presets/rag-dev run script.lua
```

**3. Multi-layer composition** ⭐ NEW:
```bash
llmspell -p bases/cli,features/rag,envs/dev,backends/sqlite run script.lua
```

The multi-layer syntax uses a **4-layer architecture**:
- **bases/** - Deployment mode (cli, daemon, embedded, testing)
- **features/** - Capabilities (minimal, llm, rag, memory, state, full, local)
- **envs/** - Environment tuning (dev, staging, prod, perf)
- **backends/** - Storage backend (memory, sqlite, postgres)

### Quick Reference

| Use Case | Preset | Composition Equivalent |
|----------|--------|----------------------|
| **Tools only** | `minimal` | `bases/cli,features/minimal` |
| **Agent dev** | `development` | `bases/cli,features/llm,envs/dev` |
| **RAG dev** | `rag-dev` | `bases/cli,features/rag,envs/dev,backends/sqlite` |
| **Production RAG** | `rag-prod` | `bases/cli,features/rag,envs/prod,backends/sqlite` |
| **Memory system** | `memory` | `bases/cli,features/memory,envs/dev,backends/sqlite` |
| **Full stack** | `gemini-prod` | `bases/cli,features/full,envs/prod,backends/sqlite` |
| **Local offline** | `full-local-ollama` | `bases/cli,features/full,envs/dev,backends/sqlite` |
| **Production daemon** | `daemon-prod` | `bases/daemon,features/full,envs/prod,backends/postgres` |

### Core Profiles (Backward Compatible)

**minimal** - Tools and workflows only, no LLM providers
```bash
llmspell -p minimal run script.lua
```
**What's enabled**: Tool execution, basic workflows
**What's disabled**: LLM providers, RAG, memory, graph

---

**development** - Full development environment with debug logging
```bash
llmspell -p development run script.lua
```
**What's enabled**: All LLM providers, debug logging, in-memory state
**What's disabled**: Persistence, RAG, memory system

---

**providers** - Simple OpenAI + Anthropic + Gemini agent setup
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
llmspell -p providers run script.lua
```
**What's enabled**: All cloud LLM providers
**What's disabled**: Persistence, RAG, memory

---

**state** - State persistence with memory backend
```bash
llmspell -p state run script.lua
```
**What's enabled**: State persistence, sessions, hooks, events
**What's disabled**: LLM providers, RAG, disk persistence

---

**sessions** - Full session management (state + hooks + events)
```bash
llmspell -p sessions run script.lua
```
**What's enabled**: State persistence (SQLite), sessions, hooks, events, artifacts

---

### Local LLM Profiles

**ollama** - Local LLM via Ollama backend
```bash
llmspell -p ollama run script.lua  # Requires: ollama serve
```
**What's enabled**: Ollama provider, Candle provider, SQLite persistence

---

**candle** - Local LLM via Candle backend
```bash
llmspell -p candle run script.lua
```
**What's enabled**: Candle provider, Ollama provider, SQLite persistence

---

**full-local-ollama** ⭐ NEW - Complete local stack (Ollama + all features)
```bash
llmspell -p full-local-ollama run offline-app.lua
```
**What's enabled**: Graph, RAG, memory, context, Ollama default provider, SQLite

---

### RAG Profiles (Phase 13)

**rag-dev** - RAG development with debug logging
```bash
export OPENAI_API_KEY="sk-..."  # For embeddings
llmspell -p rag-dev run doc-search.lua
```
**What's enabled**: Vector storage (HNSW), embeddings, debug logging, SQLite

---

**rag-prod** - RAG production settings
```bash
llmspell -p rag-prod run knowledge-base.lua
```
**What's enabled**: Vector storage, production logging (warn), SQLite

---

**rag-perf** - RAG performance tuning
```bash
llmspell -p rag-perf run benchmark.lua
```
**What's enabled**: Vector storage, minimal logging (error), optimized settings

---

### Memory & Advanced Profiles (Phase 13)

**memory** ⭐ NEW - Adaptive memory system
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p memory run chatbot-with-memory.lua
```
**What's enabled**: 3-tier memory system, debug logging, SQLite persistence

---

**research** ⭐ NEW - Full features with trace logging
```bash
llmspell -p research run experiment.lua 2>&1 | tee research.log
```
**What's enabled**: Graph, RAG, memory, context, trace-level logging, SQLite

---

### Production Profiles

**gemini-prod** ⭐ NEW - Full stack with Gemini
```bash
export GEMINI_API_KEY="..."
llmspell -p gemini-prod run app.lua
```
**What's enabled**: Graph, RAG, memory, context, Gemini default provider

---

**openai-prod** ⭐ NEW - Full stack with OpenAI
```bash
export OPENAI_API_KEY="sk-..."
llmspell -p openai-prod run app.lua
```
**What's enabled**: Graph, RAG, memory, context, OpenAI default provider

---

**claude-prod** ⭐ NEW - Full stack with Claude/Anthropic
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
llmspell -p claude-prod run app.lua
```
**What's enabled**: Graph, RAG, memory, context, Anthropic default provider

---

### Daemon Profiles

**daemon-dev** ⭐ NEW - Daemon mode development
```bash
llmspell -p daemon-dev kernel start --port 9555
```
**What's enabled**: Everything, high concurrency (100), debug logging, SQLite

---

**daemon-prod** ⭐ NEW - Production daemon with PostgreSQL
```bash
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell -p daemon-prod kernel start --port 9555
```
**What's enabled**: Everything, high concurrency (100), production logging, PostgreSQL

---

**postgres-prod** ⭐ NEW - Alias for daemon-prod
```bash
llmspell -p postgres-prod kernel start
```
**What's enabled**: Same as daemon-prod

---

### Default Profile

**default** - Minimal CLI setup (same as `minimal`)
```bash
llmspell run script.lua  # Uses 'default' profile automatically
```

### Profile Precedence

Configuration is resolved in this order (later overrides earlier):

1. Built-in defaults
2. **Builtin profile** (`-p profile-name`) ⭐ NEW
3. System config: `/etc/llmspell/config.toml`
4. User config: `~/.config/llmspell/config.toml`
5. Project config: `./llmspell.toml`
6. CLI specified: `-c custom.toml`
7. Environment variables
8. Command-line arguments

**Note**: Builtin profiles override defaults but can be overridden by config files, environment variables, or command-line arguments.

### When to Use Custom Configs

Use custom configuration files instead of builtin profiles when you need:
- Unique resource limits beyond profile defaults
- Multi-tenant isolation policies
- Application-specific security settings
- Advanced migration or backup strategies
- Custom provider endpoints

See [Configuration Files](#configuration-files) section below for custom config syntax.

---

## Configuration Files

### Main Configuration Structure

```toml
# config.toml - Complete configuration example with Phase 9-10 features

# Kernel configuration (Phase 9-10)
[kernel]
port = 9555
connection_file = "/var/lib/llmspell/kernel.json"
kernel_id = "auto"  # auto-generated if not specified
hmac_key = "your-secret-key"  # For message signing
idle_timeout = 0  # 0 = never timeout
max_clients = 100
max_message_size_mb = 10

# Daemon configuration (Phase 10)
[daemon]
daemonize = false  # Set to true for production
pid_file = "/var/run/llmspell/kernel.pid"
working_dir = "/"
umask = 0o027
close_stdin = true

# Logging configuration (Phase 9-10)
[logging]
log_file = "/var/log/llmspell/kernel.log"
log_level = "info"  # trace, debug, info, warn, error
max_size_mb = 100
max_backups = 5
compress = true
enable_syslog = false  # Unix only

# Main runtime configuration
[runtime]
max_concurrent_scripts = 10
script_timeout_seconds = 300
enable_streaming = true
use_global_io_runtime = true  # Phase 9 - prevents "dispatch task is gone"

# Provider configuration
[providers]
default_provider = "openai"

[providers.providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o-mini"
temperature = 0.7
max_tokens = 2000

# RAG (Retrieval-Augmented Generation) - Phase 8
[rag]
enabled = false  # Enable for RAG functionality
multi_tenant = false

[rag.vector_storage]
dimensions = 384  # 384, 768, 1536, 3072
backend = "hnsw"
persistence_path = "./data/rag/vectors"
max_memory_mb = 500

[rag.vector_storage.hnsw]
m = 16
ef_construction = 200
ef_search = 50
max_elements = 1000000
metric = "cosine"

# Security configuration
[runtime.security]
allow_file_access = false
allow_network_access = true
allow_process_spawn = false
max_memory_bytes = 50000000

# State persistence
[runtime.state_persistence]
enabled = true
backend_type = "memory"
migration_enabled = false
backup_enabled = false

# Events system
[events]
enabled = true
buffer_size = 10000
emit_timing_events = true
```

### Configuration Hierarchy

See [Profile Precedence](#profile-precedence) in the Builtin Profiles section above for complete configuration resolution order.

---

## Kernel Configuration

Phase 9-10 introduced the integrated kernel architecture with comprehensive configuration options.

### Basic Kernel Settings

```toml
[kernel]
# Network settings
port = 9555                               # Base port (other channels are +1, +2, etc)
ip = "127.0.0.1"                         # Bind IP address
transport = "tcp"                        # tcp or ipc

# Connection management
connection_file = "/var/lib/llmspell/kernel.json"  # Jupyter connection file
kernel_id = "auto"                       # auto-generated if not specified
hmac_key = "your-secret-key"            # Message signing key

# Resource limits
idle_timeout = 0                          # Seconds (0 = never timeout)
max_clients = 100                        # Maximum concurrent clients
max_message_size_mb = 10                # Maximum message size
max_memory_mb = 2048                    # Memory limit
max_execution_time_secs = 300          # Per-execution timeout

# Protocol support
enable_jupyter = true                    # Enable Jupyter protocol
enable_dap = false                       # Enable Debug Adapter Protocol
enable_lsp = false                      # Enable Language Server Protocol
enable_repl = true                      # Enable REPL service
```

### Execution Configuration

```toml
[kernel.execution]
max_history = 1000                       # Command history size
execution_timeout_secs = 300            # Per-execution timeout
monitor_agents = true                   # Track agent performance
track_performance = true                # Enable performance metrics
enable_tracing = false                  # Enable execution tracing

# Health monitoring
[kernel.health]
enable_health_checks = true
health_check_interval = 30              # Seconds
memory_threshold_mb = 1500              # Warning threshold
cpu_threshold_percent = 80              # Warning threshold
restart_on_failure = true               # Auto-restart on health failure
```

### Transport Configuration

```toml
# ZeroMQ transport settings (Jupyter)
[kernel.transport.zeromq]
shell_port = 9555
iopub_port = 9556
stdin_port = 9557
control_port = 9558
hb_port = 9559
send_timeout_ms = 1000
recv_timeout_ms = 1000
max_message_size = 104857600  # 100MB

# WebSocket transport (future)
[kernel.transport.websocket]
port = 9560
path = "/ws"
max_frame_size = 65536
compression = true

# In-process transport (embedded mode)
[kernel.transport.inprocess]
buffer_size = 1048576  # 1MB
use_channels = true
```

---

## Daemon & Service Settings

Phase 10 added proper Unix daemon support with systemd/launchd integration.

### Daemon Configuration

```toml
[daemon]
daemonize = true                         # Run as daemon
pid_file = "/var/run/llmspell/kernel.pid"  # PID file location
working_dir = "/"                        # Working directory
umask = 0o027                           # File creation mask
close_stdin = true                      # Close stdin in daemon mode
detach_tty = true                       # Detach from TTY

# Double-fork settings
double_fork = true                       # Use double-fork technique
become_session_leader = true            # setsid()

# Log redirection
[daemon.logging]
stdout_path = "/var/log/llmspell/stdout.log"
stderr_path = "/var/log/llmspell/stderr.log"
rotate_size_mb = 100
rotate_count = 5
compress_rotated = true
```

### Service Management

```toml
[service]
# systemd settings (Linux)
systemd_type = "forking"                # forking or simple
restart_policy = "on-failure"           # always, on-failure, no
restart_sec = 10                        # Seconds between restarts
watchdog_sec = 0                        # Systemd watchdog (0=disabled)

# Security hardening
private_tmp = true
no_new_privileges = true
protect_system = "strict"
protect_home = "read-only"
read_write_paths = ["/var/log/llmspell", "/var/lib/llmspell"]

# launchd settings (macOS)
launchd_label = "com.llmspell.kernel"
run_at_load = true
keep_alive = true
throttle_interval = 10                  # Seconds between restart attempts

# Windows service (future)
windows_service_name = "LLMSpellKernel"
windows_display_name = "LLMSpell Kernel Service"
windows_start_type = "automatic"        # automatic, manual, disabled
```

### Signal Configuration

```toml
[signals]
# Signal handling behavior
handle_sigterm = true                   # Graceful shutdown
handle_sighup = true                    # Reload configuration
handle_sigusr1 = true                   # Dump statistics
handle_sigusr2 = true                   # Toggle debug logging
handle_sigint = true                    # Interrupt execution

# Shutdown behavior
graceful_shutdown_timeout = 30          # Seconds to wait for clean shutdown
force_shutdown_on_second_signal = true  # Force exit on second SIGTERM
save_state_on_shutdown = true          # Persist state before exit
notify_clients_on_shutdown = true      # Send shutdown message to clients
```

---

## Protocol Configuration

Phase 9-10 added support for multiple protocols through the kernel.

### Jupyter Protocol

```toml
[protocols.jupyter]
enabled = true
signature_scheme = "hmac-sha256"
kernel_name = "llmspell"
language = "lua"
display_name = "LLMSpell"
codemirror_mode = "lua"

# Message handling
[protocols.jupyter.messages]
max_iopub_queue = 100
max_stdin_queue = 10
broadcast_iopub = true
validate_signatures = true

# Execution
[protocols.jupyter.execution]
store_history = true
silent_execution_reply = false
stop_on_error = true
```

### Debug Adapter Protocol (DAP)

```toml
[protocols.dap]
enabled = false
port = 9556
wait_for_client = false
break_on_entry = false
break_on_error = false

# Capabilities
[protocols.dap.capabilities]
supports_configuration_done_request = true
supports_function_breakpoints = true
supports_conditional_breakpoints = true
supports_evaluate_for_hovers = true
supports_step_back = false
supports_restart_frame = false
supports_exception_info_request = true
supports_terminate_request = true

# Debugging
[protocols.dap.debugging]
show_return_value = true
all_threads_stopped = true
lines_start_at_1 = true
columns_start_at_1 = true
```

### Language Server Protocol (LSP)

```toml
[protocols.lsp]
enabled = false
port = 9557
root_uri = "file:///workspace"

# Capabilities
[protocols.lsp.capabilities]
completion_provider = true
hover_provider = true
signature_help_provider = true
definition_provider = true
references_provider = true
document_symbol_provider = true
workspace_symbol_provider = true
code_action_provider = true
```

### REPL Service

```toml
[protocols.repl]
enabled = true
port = 9558
history_file = "~/.llmspell/repl_history"
history_size = 1000
multiline_enabled = true
auto_indent = true
syntax_highlighting = true

# Prompt customization
[protocols.repl.prompt]
primary = "llmspell> "
secondary = "      > "
error_prefix = "Error: "
```

---

## Debug & IDE Integration

Phase 9 added comprehensive debugging support with DAP and IDE integration.

### Debug Configuration

```toml
[debug]
enable_dap = true                        # Enable Debug Adapter Protocol
dap_port = 9556                         # DAP server port
wait_for_debugger = false               # Wait for debugger to attach
break_on_error = false                  # Auto-break on errors
break_on_entry = false                  # Break at script start

# Source mapping
[debug.source_map]
enable = true
map_script_paths = true                 # Map script paths to source files
source_root = "/workspace"              # Source root directory
strip_prefix = ""                       # Path prefix to strip
add_prefix = ""                         # Path prefix to add
```

### IDE Integration

```toml
[ide]
# VS Code integration
vscode_extension = "llmspell.llmspell-debug"
launch_config_template = true           # Generate launch.json template
attach_config_template = true           # Generate attach config

# Jupyter integration
jupyter_kernel_name = "llmspell"
jupyter_display_name = "LLMSpell"
jupyter_language = "lua"
jupyter_codemirror_mode = "lua"
jupyter_file_extension = ".lua"

# IntelliJ/JetBrains integration
intellij_plugin = "com.llmspell.idea"
intellij_run_config = true

# Vim/Neovim integration
vim_plugin = "llmspell.nvim"
nvim_lsp_config = true
```

### Performance Profiling

```toml
[profiling]
enable_profiling = false                # Enable performance profiling
profile_output = "/var/log/llmspell/profile.json"
sample_rate = 100                       # Samples per second
track_allocations = false               # Track memory allocations
track_cpu_time = true                  # Track CPU usage
track_io_time = true                   # Track I/O operations
flamegraph_enabled = false             # Generate flamegraphs
```

---

## Logging & Monitoring

Enhanced logging and monitoring for Phase 9-10 kernel architecture.

### Logging Configuration

```toml
[logging]
# File logging
log_file = "/var/log/llmspell/kernel.log"
log_level = "info"                      # off, error, warn, info, debug, trace
max_size_mb = 100
max_backups = 5
max_age_days = 30
compress = true
json_format = false                     # Use JSON structured logging

# Console logging
console_enabled = true
console_level = "warn"
console_color = true

# Syslog (Unix only)
syslog_enabled = false
syslog_facility = "daemon"
syslog_tag = "llmspell"

# Rotation
[logging.rotation]
strategy = "size"                       # size, daily, hourly
size_limit = "100MB"
time_format = "%Y-%m-%d"
keep_files = 5
compress_old = true
```

### Metrics & Monitoring

```toml
[monitoring]
# Metrics collection
enable_metrics = true
metrics_port = 9559
metrics_path = "/metrics"
export_format = "prometheus"            # prometheus, json, statsd

# Health checks
[monitoring.health]
enable_health = true
health_port = 9559
health_path = "/health"
liveness_path = "/healthz"
readiness_path = "/ready"

# Performance monitoring
[monitoring.performance]
track_latency = true
track_throughput = true
track_errors = true
percentiles = [0.5, 0.9, 0.95, 0.99]
window_size_secs = 60

# Resource monitoring
[monitoring.resources]
track_memory = true
track_cpu = true
track_disk_io = true
track_network_io = true
sample_interval_secs = 10
```

### Event Correlation

```toml
[events]
# Event system (Phase 9)
enabled = true
buffer_size = 10000
emit_timing_events = true
emit_state_events = true
emit_protocol_events = true

# Correlation
[events.correlation]
enable_correlation = true
correlation_timeout_secs = 300
max_correlation_depth = 10
track_causation = true
```

---

## LLM Providers

### Provider/Model Syntax

Use hierarchical naming: `provider/model`

```lua
local agent = Agent.builder()
    :model("openai/gpt-4")  -- Provider/model format
    :build()
```

### OpenAI

**Models:**
- `openai/gpt-4` - Most capable
- `openai/gpt-4-turbo` - Faster GPT-4
- `openai/gpt-4o` - Optimized variant
- `openai/gpt-4o-mini` - Smaller, faster
- `openai/gpt-3.5-turbo` - Cost-effective

**Configuration:**
```toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4o-mini"
organization_id = "org-..."  # Optional

[providers.openai.defaults]
temperature = 0.7
max_tokens = 2000
```

### Anthropic

**Models:**
- `anthropic/claude-3-opus` - Most capable
- `anthropic/claude-3-sonnet` - Balanced
- `anthropic/claude-3-haiku` - Fast
- `anthropic/claude-2.1` - Previous gen
- `anthropic/claude-instant` - Fastest

**Configuration:**
```toml
[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"
default_model = "claude-3-sonnet"
version = "2024-02-15"

[providers.anthropic.defaults]
max_tokens = 4000
temperature = 0.5
```

### Ollama (Local)

**Models:**
- `ollama/llama2` - Llama 2
- `ollama/mistral` - Mistral
- `ollama/codellama` - Code Llama
- `ollama/mixtral` - Mixtral

**Configuration:**
```toml
[providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama2"
timeout = 300  # Seconds

[providers.ollama.defaults]
temperature = 0.8
num_predict = 2048
```

### Groq

**Models:**
- `groq/llama3-70b` - Llama 3 70B
- `groq/mixtral-8x7b` - Mixtral
- `groq/gemma-7b` - Gemma

**Configuration:**
```toml
[providers.groq]
api_key = "${GROQ_API_KEY}"
base_url = "https://api.groq.com/openai/v1"
default_model = "llama3-70b"
```

### Custom Providers

```toml
[providers.custom]
api_key = "${CUSTOM_API_KEY}"
base_url = "https://your-api.com"
auth_header = "X-API-Key"  # Or "Authorization"
auth_prefix = "Bearer"      # For Authorization header
default_model = "your-model"

[providers.custom.headers]
"X-Custom-Header" = "value"
"X-API-Version" = "2024-01"
```

---

## RAG Configuration ⭐ **Phase 8**

LLMSpell includes comprehensive RAG (Retrieval-Augmented Generation) capabilities with HNSW vector storage, multi-tenant isolation, and cost optimization.

### Basic RAG Setup

```toml
[rag]
enabled = true
multi_tenant = false  # Enable for tenant isolation

# Vector storage configuration
[rag.vector_storage]
dimensions = 768      # 384, 768, 1536, 3072 supported
backend = "hnsw"      # Only HNSW supported currently
persistence_path = "./data/rag/vectors"
max_memory_mb = 1024

# HNSW algorithm parameters
[rag.vector_storage.hnsw]
m = 16                      # Connections per node
ef_construction = 200       # Build-time search width
ef_search = 100            # Query-time search width
max_elements = 1000000     # Maximum vectors
metric = "cosine"          # cosine, euclidean, inner_product
allow_replace_deleted = true
num_threads = 4
```

### Embedding Configuration

```toml
[rag.embedding]
default_provider = "openai"  # Provider for embeddings
cache_enabled = true         # 70% cost reduction
cache_size = 20000          # Cached embeddings
cache_ttl_seconds = 3600    # 1 hour cache
batch_size = 32             # Batch processing
timeout_seconds = 30
max_retries = 3
```

### Document Chunking

```toml
[rag.chunking]
strategy = "sliding_window"  # sliding_window, semantic, sentence
chunk_size = 512            # Tokens per chunk
overlap = 64               # Overlap between chunks
max_chunk_size = 2048      # Hard limit
min_chunk_size = 100       # Quality threshold
```

### RAG Caching (70% Cost Reduction)

```toml
[rag.cache]
# Search result caching
search_cache_enabled = true
search_cache_size = 5000
search_cache_ttl_seconds = 600

# Document caching
document_cache_enabled = true
document_cache_size_mb = 200
```

### HNSW Optimization Profiles

**Small Dataset (<10K vectors):**
```toml
[rag.vector_storage.hnsw]
m = 12
ef_construction = 100
ef_search = 50
max_elements = 10000
```

**Large Dataset (100K-1M vectors):**
```toml
[rag.vector_storage.hnsw]
m = 32
ef_construction = 400
ef_search = 200
max_elements = 1000000
num_threads = 4
```

**Speed Optimized:**
```toml
[rag.vector_storage.hnsw]
m = 8
ef_construction = 50
ef_search = 25
```

**Accuracy Optimized:**
```toml
[rag.vector_storage.hnsw]
m = 48
ef_construction = 500
ef_search = 300
```

### Session Collections

For conversational memory:

```toml
[rag.sessions]
enable_session_collections = true
session_collection_ttl = 3600  # 1 hour
max_session_vectors = 1000
auto_cleanup = true
```

### Supported Vector Dimensions

| Dimensions | Model Example | Use Case |
|------------|---------------|----------|
| 384 | all-MiniLM-L6-v2 | Fast, small memory |
| 768 | all-mpnet-base-v2 | Balanced |
| 1536 | text-embedding-3-small | OpenAI standard |
| 3072 | text-embedding-3-large | Maximum accuracy |

---

## Multi-Tenancy ⭐ **Phase 8**

Complete tenant isolation for RAG and state data.

### Basic Multi-Tenant Setup

```toml
[rag]
enabled = true
multi_tenant = true  # IMPORTANT: Enable tenant isolation

[rag.multi_tenant_settings]
max_vectors_per_tenant = 100000
tenant_ttl_hours = 168        # 7 days retention
auto_cleanup = true
strict_isolation = true       # No cross-tenant access
max_concurrent_operations = 10
rate_limit_per_minute = 100
```

### Tenant Resource Quotas

```toml
[tenancy.quotas]
# Storage limits per tenant
max_storage_mb = 1000
max_vectors = 50000
max_collections = 10

# Compute limits
max_queries_per_minute = 100
max_ingestion_rate = 50      # documents per minute
max_concurrent_operations = 5

# Cost control
max_embedding_tokens_per_day = 100000
billing_enabled = true
```

### Tenant Lifecycle

```toml
[tenancy.lifecycle]
default_retention_days = 30
auto_suspend_inactive_days = 7
purge_deleted_after_days = 90
backup_before_deletion = true
```

---

## State & Sessions

### State Persistence

```toml
[runtime.state_persistence]
enabled = true
backend_type = "sqlite"      # memory, sqlite
migration_enabled = true
backup_enabled = true
backup_on_migration = true
schema_directory = "./schemas"
max_state_size_bytes = 10000000

[runtime.state_persistence.backup]
backup_dir = "./backups"
compression_enabled = true
compression_type = "zstd"
compression_level = 3
incremental_enabled = true
max_backups = 10
max_backup_age = 2592000     # 30 days
```

### Session Management

```toml
[runtime.sessions]
enabled = true
max_sessions = 100
max_artifacts_per_session = 1000
artifact_compression_threshold = 10240  # 10KB
session_timeout_seconds = 3600
storage_backend = "memory"   # memory, sqlite, postgres
```

---

## Infrastructure Module (Phase 13b.16)

### Overview

The Infrastructure module provides **unified component creation** for ScriptRuntime. All 9 core components are created from configuration via `Infrastructure::from_config()`:

1. **ProviderManager** - LLM provider access
2. **StateManager** - State persistence
3. **SessionManager** - Session + artifact management
4. **RAG** - Retrieval-Augmented Generation (optional)
5. **MemoryManager** - Adaptive memory system (optional)
6. **ToolRegistry** - Tool management
7. **AgentRegistry** - Agent factories
8. **WorkflowFactory** - Workflow execution
9. **ComponentRegistry** - Script access layer

### Single Creation Path

```rust
use llmspell_bridge::infrastructure::Infrastructure;
use llmspell_config::LLMSpellConfig;

let config = LLMSpellConfig::from_file("config.toml")?;
let infrastructure = Infrastructure::from_config(&config).await?;

// All 9 components available
let provider_manager = infrastructure.provider_manager.clone();
let rag = infrastructure.rag.clone(); // Option<Arc<...>>
let memory = infrastructure.memory_manager.clone(); // Option<Arc<...>>
```

### Conditional Component Creation

**RAG and Memory are created only if enabled in config:**

```toml
[rag]
enabled = true  # Creates MultiTenantRAG component

[memory]
enabled = true  # Creates DefaultMemoryManager component
```

If `enabled = false` or section missing:
- `infrastructure.rag` = `None`
- `infrastructure.memory_manager` = `None`

### Backend Selection

**Global backend** (applies to all 10 storage components):

```toml
[storage]
backend = "postgres"  # "memory", "sqlite", or "postgres"
```

**Per-component overrides** (advanced):

```toml
[storage]
backend = "postgres"  # Default for all

[storage.memory]
backend = "memory"    # Override: use in-memory for episodic memory (testing)

[storage.state]
backend = "sqlite"    # Override: use sqlite for agent state (embedded)
```

**10 Storage Components:**
1. Vector embeddings (vector_embeddings_{384,768,1536,3072})
2. Temporal graph (entities, relationships)
3. Procedural memory (procedural_memory)
4. Agent state (agent_states)
5. Workflow states (workflow_states)
6. Sessions (sessions)
7. Artifacts (artifacts + artifact_content)
8. Event log (event_log)
9. Hook history (hook_history)
10. API keys (api_keys)

---

## Storage Backend Configuration (Phase 13b)

### Backend Types

**memory** - In-memory storage (testing only)
```toml
[storage]
backend = "memory"
```
- **Use case**: Testing, development, CI/CD
- **Pros**: Fastest, no setup
- **Cons**: Data lost on restart, no persistence

**sqlite** - Embedded database
```toml
[storage]
backend = "sqlite"

[storage.sqlite]
path = "./data/llmspell.db"
```
- **Use case**: Embedded deployments, single-user applications, development
- **Pros**: No external dependencies, ACID transactions, vector search (vectorlite-rs), bi-temporal graph
- **Cons**: Single-process only (file locking), limited concurrency

**postgres** - PostgreSQL 18 + VectorChord
```toml
[storage]
backend = "postgres"

[storage.postgres]
url = "postgresql://llmspell_app:pass@localhost:5432/llmspell_prod"
pool_size = 20
pool_timeout_secs = 30
idle_timeout_secs = 600
max_lifetime_secs = 1800
default_tenant_id = "default"
enforce_tenant_isolation = true
auto_migrate = false
```
- **Use case**: Production, multi-tenant, high-concurrency
- **Pros**: ACID, RLS isolation, vector search (HNSW), scalable
- **Cons**: Requires PostgreSQL 18 + VectorChord setup

**See**: [PostgreSQL Setup Guide](storage/postgresql-setup.md) for installation

### PostgreSQL Configuration

**Connection settings:**
```toml
[storage.postgres]
# Connection URL (required)
url = "postgresql://llmspell_app:password@localhost:5432/llmspell_prod"

# Connection pool
pool_size = 20              # Max connections (formula: CPU × 2 + 1)
pool_timeout_secs = 30      # Timeout acquiring connection
idle_timeout_secs = 600     # Close idle connections (10 min)
max_lifetime_secs = 1800    # Recycle connections (30 min)
```

**Multi-tenancy:**
```toml
[storage.postgres]
# Tenant isolation via Row-Level Security (RLS)
default_tenant_id = "default"
enforce_tenant_isolation = true  # Enable RLS policies
tenant_id_pattern = "^[a-z0-9-]{3,255}$"  # Validation regex
```

**Migrations:**
```toml
[storage.postgres]
auto_migrate = false        # Run migrations on startup
migration_timeout_secs = 300
```

**Component-specific settings:**
```toml
[storage.postgres.vector_embeddings]
# HNSW index parameters (per-dimension table)
hnsw_m = 16                # Graph connectivity (default: 16)
hnsw_ef_construction = 128 # Build-time search depth (default: 128)

[storage.postgres.event_log]
# Event log partitioning
partition_strategy = "monthly"  # "daily", "weekly", "monthly"
retention_days = 365            # Purge partitions older than 1 year

[storage.postgres.artifacts]
# Large object storage
compression_threshold_bytes = 1048576  # Compress artifacts >1 MB
```

**Performance targets:**
- **Vector search**: <5ms p95 (10K vectors, k=10)
- **RLS overhead**: <5% (4.9% validated)
- **Event ingestion**: 10K events/sec sustained
- **Connection pool**: 100+ concurrent connections

**See**:
- [Schema Reference](storage/schema-reference.md) - 15 tables documented
- [Performance Tuning](storage/performance-tuning.md) - HNSW optimization
- [Backup/Restore](storage/backup-restore.md) - Disaster recovery

### SQLite Configuration

```toml
[storage.sqlite]
# Database path
path = "./data/llmspell.db"
```

**SQLite Configuration:**
- Managed by libsql with optimal defaults
- ACID transactions, WAL mode
- vectorlite-rs extension for HNSW vector search
- Bi-temporal graph with recursive CTEs

### Hybrid Backend Configuration

**Use different backends per component:**

```toml
[storage]
backend = "postgres"  # Default

[storage.memory]
backend = "memory"    # Episodic memory in RAM (fast, testing)

[storage.state]
backend = "sqlite"    # Agent state in embedded DB (no external deps)

[storage.events]
backend = "postgres"  # Events in PostgreSQL (durability, partitions)
```

**Use cases:**
- **Development**: `memory` for fast iteration
- **Embedded**: `sqlite` for zero-dependency deployments
- **Production**: `postgres` for durability and scale

---

## Security Settings

> **📚 Complete Security Guide**: See [Security & Permissions Guide](security-and-permissions.md) for comprehensive coverage of security levels, sandbox configuration, permission troubleshooting, and common scenarios. This section shows configuration syntax - the guide explains when and how to use it.

### Authentication

```toml
[security.authentication]
require_api_key = true
api_key_min_length = 32
api_key_rotation_days = 90
session_timeout_minutes = 30
max_failed_attempts = 5
lockout_duration_minutes = 15

[security.authentication.api_keys]
hash_algorithm = "argon2"
rate_limit_per_key = 1000  # per hour
```

### Tool Permissions & Sandboxing

> **⚠️ Schema Change**: Tool permissions are now configured via `[tools.*]` sections, not `[security.sandboxing]`. See [Security & Permissions Guide](security-and-permissions.md) for migration details.

```toml
# File System Access
[tools.file_operations]
enabled = true
allowed_paths = [
    "/tmp",              # Safe scratch directory
    "/workspace",        # Your project directory
    "/data"              # Data directory
]
max_file_size = 50000000  # 50MB in bytes
atomic_writes = true
max_depth = 10  # Directory traversal depth
allowed_extensions = []  # Empty = all allowed except blocked
blocked_extensions = ["exe", "dll", "so", "dylib"]
validate_file_types = true

# Network Access - Web Search Tool
[tools.web_search]
rate_limit_per_minute = 30
allowed_domains = [
    "api.openai.com",
    "*.anthropic.com",  # Wildcard for subdomains
    "github.com"
]
blocked_domains = []
max_results = 10
timeout_seconds = 30

# Network Access - HTTP Request Tool
[tools.http_request]
allowed_hosts = [
    "api.example.com",
    "*.trusted.com"
]
blocked_hosts = ["localhost", "127.0.0.1", "0.0.0.0"]  # SSRF prevention
max_request_size = 10000000  # 10MB
timeout_seconds = 30
max_redirects = 5

# Process Execution
[tools.system]
allow_process_execution = false  # ⚠️ Set true to enable (security critical)
allowed_commands = "echo,cat,ls,pwd,date,whoami"  # Comma-separated allowlist
# Blocked by default: rm, sudo, chmod, chown, curl, wget, ssh, scp, python, sh, bash
command_timeout_seconds = 30
max_output_size = 1000000  # 1MB
allowed_env_vars = "HOME,PATH,LANG"  # Comma-separated
```

### Rate Limiting

```toml
[security.rate_limiting]
enabled = true
window_size_secs = 60
max_requests_per_window = 100
max_burst = 20
cleanup_interval_secs = 300

[security.rate_limiting.endpoints]
"/execute" = 50
"/kernel/start" = 10
"/rag/search" = 100
```

### Audit Logging

```toml
[security.audit]
enabled = true
log_file = "/var/log/llmspell/audit.log"
log_authentication = true
log_authorization = true
log_data_access = true
log_configuration_changes = true
include_request_body = false
include_response_body = false
```

---

## Tool Configuration

### Tool Discovery

```toml
[tools]
auto_discover = true
tool_directories = ["./tools", "~/.llmspell/tools"]
reload_on_change = false
cache_metadata = true

[tools.categories]
enabled = ["filesystem", "web", "api", "data", "system"]
disabled = []
```

### Tool Security

```toml
[tools.security]
default_level = "restricted"  # safe, restricted, privileged
require_approval = false
sandbox_tools = true

# Per-tool security levels (optional - defaults to default_level)
[tools.permissions]
"file-operations" = "restricted"    # Requires explicit paths in [tools.file_operations]
"http-request" = "restricted"       # Requires explicit hosts in [tools.http_request]
"process-executor" = "restricted"   # Requires explicit commands in [tools.system]
"web-search" = "restricted"         # Requires explicit domains in [tools.web_search]
"calculator" = "safe"               # Pure computation, no external access
"text-manipulator" = "safe"         # Pure computation, no external access
"hash-calculator" = "safe"          # Pure computation, no external access

# Security levels explained:
# - safe: No file/network/process access (pure computation)
# - restricted: Requires explicit allowlists in [tools.*] sections above
# - privileged: Full system access (requires security review - avoid)
#
# See docs/user-guide/security-and-permissions.md for complete guide
```

### Tool Timeouts

```toml
[tools.timeouts]
default = 30  # seconds
"web-scraper" = 60
"api-caller" = 45
"database-query" = 120
```

---

## External API Setup

### API Keys

```toml
[apis]
# Weather API
weather_api_key = "${WEATHER_API_KEY}"
weather_base_url = "https://api.weather.com"

# News API
news_api_key = "${NEWS_API_KEY}"
news_base_url = "https://newsapi.org"

# Custom APIs
[apis.custom]
endpoint = "https://api.example.com"
auth_type = "bearer"  # bearer, api_key, basic
auth_value = "${CUSTOM_API_TOKEN}"
```

### Webhook Configuration

```toml
[webhooks]
enabled = true
timeout = 30
retry_count = 3
retry_delay = 5

[webhooks.endpoints]
"on_execution_complete" = "https://example.com/webhook"
"on_error" = "https://example.com/error-webhook"
```

---

## Deployment Profiles

### Development Profile

```toml
[profiles.development]
extends = "default"

[profiles.development.kernel]
port = 9555
idle_timeout = 3600  # 1 hour timeout
max_clients = 10
enable_dap = true
enable_lsp = true

[profiles.development.daemon]
daemonize = false  # Run in foreground

[profiles.development.logging]
log_level = "debug"
log_file = "./llmspell.log"
console_enabled = true

[profiles.development.debug]
enable_dap = true
break_on_error = true
wait_for_debugger = false

[profiles.development.security]
sandboxing_enabled = false
rate_limiting_enabled = false
```

### Production Profile

```toml
[profiles.production]
extends = "default"

[profiles.production.kernel]
port = 9555
idle_timeout = 0  # Never timeout
max_clients = 1000
max_memory_mb = 8192
enable_jupyter = true
enable_dap = false
enable_lsp = false

[profiles.production.daemon]
daemonize = true
pid_file = "/var/run/llmspell/kernel.pid"
umask = 0o077  # Restrictive permissions

[profiles.production.logging]
log_level = "info"
log_file = "/var/log/llmspell/kernel.log"
max_size_mb = 500
max_backups = 10
syslog_enabled = true
json_format = true

[profiles.production.security]
sandboxing_enabled = true
require_authentication = true
enable_tls = true
audit_log = true
rate_limiting_enabled = true
```

### Service Profile

```toml
[profiles.service]
extends = "production"

[profiles.service.service]
systemd_type = "forking"
restart_policy = "always"
private_tmp = true
no_new_privileges = true
protect_system = "strict"

[profiles.service.signals]
handle_sigterm = true
handle_sighup = true
graceful_shutdown_timeout = 60

[profiles.service.monitoring]
enable_metrics = true
enable_health = true
export_format = "prometheus"
```

### Fleet Profile (Multiple Kernels)

```toml
[profiles.fleet]
extends = "service"

[profiles.fleet.fleet]
max_kernels = 10
port_range_start = 9555
port_range_end = 9655
auto_spawn = true
load_balance = true
health_check_interval = 30

[profiles.fleet.fleet.scaling]
min_kernels = 2
max_kernels = 10
scale_up_threshold = 80  # CPU %
scale_down_threshold = 20
scale_cooldown_secs = 300
```

---

## Environment Variables

### Kernel & Runtime Variables

```bash
# Kernel configuration (Phase 9-10)
export LLMSPELL_KERNEL_PORT="9555"
export LLMSPELL_KERNEL_ID="my-kernel"
export LLMSPELL_CONNECTION_FILE="/var/lib/llmspell/kernel.json"
export LLMSPELL_HMAC_KEY="secret-key"

# Daemon settings (Phase 10)
export LLMSPELL_DAEMON="true"
export LLMSPELL_PID_FILE="/var/run/llmspell/kernel.pid"
export LLMSPELL_LOG_FILE="/var/log/llmspell/kernel.log"
export LLMSPELL_WORKING_DIR="/"

# Runtime settings
export RUST_LOG="info"  # trace, debug, info, warn, error
export LLMSPELL_CONFIG="/etc/llmspell/config.toml"
export LLMSPELL_LOG_LEVEL="info"
export LLMSPELL_DEBUG="false"

# Protocol settings (Phase 9-10)
export LLMSPELL_ENABLE_JUPYTER="true"
export LLMSPELL_ENABLE_DAP="false"
export LLMSPELL_ENABLE_LSP="false"
export LLMSPELL_ENABLE_REPL="true"
```

### Security & Permissions Variables

Override security settings without modifying config.toml. Essential for CI/CD, Docker, and quick testing.

```bash
# Runtime security (master switches)
export LLMSPELL_ALLOW_FILE_ACCESS="true"          # Enable file system access
export LLMSPELL_ALLOW_NETWORK_ACCESS="false"      # Disable network access
export LLMSPELL_ALLOW_PROCESS_SPAWN="true"        # Enable process spawning

# File operations
export LLMSPELL_TOOLS_ALLOWED_PATHS="/tmp,/workspace,/data"
export LLMSPELL_TOOLS_MAX_FILE_SIZE="104857600"   # 100MB in bytes
export LLMSPELL_TOOLS_BLOCKED_EXTENSIONS="exe,dll,so,dylib"
export LLMSPELL_TOOLS_MAX_DEPTH="10"              # Directory traversal depth
export LLMSPELL_TOOLS_FILE_ENABLED="true"
export LLMSPELL_TOOLS_FOLLOW_SYMLINKS="false"
export LLMSPELL_TOOLS_CHECK_MIME="false"          # Validate MIME types

# Web search
export LLMSPELL_TOOLS_WEB_ALLOWED_DOMAINS="*.openai.com,github.com,*.anthropic.com"
export LLMSPELL_TOOLS_WEB_BLOCKED_DOMAINS="spam.com,malware.com"
export LLMSPELL_TOOLS_WEB_RATE_LIMIT="100"        # Requests per minute
export LLMSPELL_TOOLS_WEB_ENABLED="true"
export LLMSPELL_TOOLS_WEB_MAX_RESULTS="10"

# HTTP requests
export LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS="api.example.com,*.company.com"
export LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS="localhost,127.0.0.1,169.254.169.254"  # SSRF protection
export LLMSPELL_TOOLS_HTTP_TIMEOUT="60"           # Seconds
export LLMSPELL_TOOLS_HTTP_MAX_REDIRECTS="5"
export LLMSPELL_TOOLS_HTTP_MAX_SIZE="10485760"    # 10MB in bytes
export LLMSPELL_TOOLS_HTTP_VERIFY_SSL="true"
export LLMSPELL_TOOLS_HTTP_USER_AGENT="LLMSpell/0.9.0"
export LLMSPELL_TOOLS_HTTP_ENABLED="true"

# System/process execution
export LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC="true"
export LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS="echo,cat,ls,pwd,git,python3"
export LLMSPELL_TOOLS_SYSTEM_BLOCKED_COMMANDS="rm,sudo,chmod"
export LLMSPELL_TOOLS_SYSTEM_TIMEOUT="60"         # Seconds
export LLMSPELL_TOOLS_SYSTEM_MAX_OUTPUT="1048576" # 1MB in bytes
export LLMSPELL_TOOLS_SYSTEM_ALLOWED_ENV="HOME,PATH,LANG,USER"
export LLMSPELL_TOOLS_SYSTEM_WORKING_DIR="/workspace"
export LLMSPELL_TOOLS_SYSTEM_ENABLED="true"

# Network configuration
export LLMSPELL_TOOLS_NETWORK_TIMEOUT="30"        # Seconds
export LLMSPELL_TOOLS_NETWORK_RETRIES="3"
export LLMSPELL_TOOLS_NETWORK_VERIFY_SSL="true"

# State persistence
export LLMSPELL_STATE_ENABLED="false"
export LLMSPELL_STATE_PATH=".llmspell/state"
export LLMSPELL_STATE_AUTO_SAVE="true"
export LLMSPELL_STATE_AUTO_LOAD="true"
```

#### Environment Variable to Config Mapping

| Environment Variable | Config Path | Default | Description |
|---------------------|-------------|---------|-------------|
| `LLMSPELL_ALLOW_FILE_ACCESS` | `runtime.security.allow_file_access` | `false` | Master switch for file system access |
| `LLMSPELL_ALLOW_NETWORK_ACCESS` | `runtime.security.allow_network_access` | `true` | Master switch for network access |
| `LLMSPELL_ALLOW_PROCESS_SPAWN` | `runtime.security.allow_process_spawn` | `false` | Master switch for process spawning |
| `LLMSPELL_TOOLS_ALLOWED_PATHS` | `tools.file_operations.allowed_paths` | - | Comma-separated allowed paths |
| `LLMSPELL_TOOLS_MAX_FILE_SIZE` | `tools.file_operations.max_file_size` | `50000000` | Max file size (50MB) |
| `LLMSPELL_TOOLS_BLOCKED_EXTENSIONS` | `tools.file_operations.blocked_extensions` | `exe,dll,so` | Blocked file extensions |
| `LLMSPELL_TOOLS_MAX_DEPTH` | `tools.file_operations.max_depth` | `10` | Max directory depth |
| `LLMSPELL_TOOLS_WEB_ALLOWED_DOMAINS` | `tools.web_search.allowed_domains` | - | Comma-separated allowed domains |
| `LLMSPELL_TOOLS_WEB_RATE_LIMIT` | `tools.web_search.rate_limit_per_minute` | `30` | Rate limit per minute |
| `LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS` | `tools.http_request.allowed_hosts` | - | Comma-separated allowed hosts |
| `LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS` | `tools.http_request.blocked_hosts` | `localhost,127.0.0.1` | Blocked hosts (SSRF) |
| `LLMSPELL_TOOLS_HTTP_TIMEOUT` | `tools.http_request.timeout_seconds` | `30` | Request timeout |
| `LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC` | `tools.system.allow_process_execution` | `false` | Enable process execution |
| `LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS` | `tools.system.allowed_commands` | `ls,cat,echo,pwd` | Allowed commands |

**Complete list**: See `llmspell-config/src/env_registry.rs` for all 50+ registered variables.

#### Common Configuration Patterns

**CI/CD Testing (Permissive)**:
```bash
# GitHub Actions, GitLab CI, etc.
export LLMSPELL_ALLOW_FILE_ACCESS="true"
export LLMSPELL_ALLOW_NETWORK_ACCESS="true"
export LLMSPELL_TOOLS_ALLOWED_PATHS="/workspace,/tmp"
export LLMSPELL_TOOLS_SYSTEM_ALLOWED_COMMANDS="git,echo,cat,ls,python3"
./target/release/llmspell run test-suite.lua
```

**Production Docker (Restricted)**:
```dockerfile
FROM rust:latest
ENV LLMSPELL_ALLOW_FILE_ACCESS=false
ENV LLMSPELL_ALLOW_NETWORK_ACCESS=true
ENV LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS=api.internal.company.com
ENV LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS=localhost,127.0.0.1,169.254.169.254
CMD ["./target/release/llmspell", "kernel", "start"]
```

**Development (Relaxed)**:
```bash
# Local development with debugging
export LLMSPELL_ALLOW_FILE_ACCESS="true"
export LLMSPELL_ALLOW_NETWORK_ACCESS="true"
export LLMSPELL_TOOLS_ALLOWED_PATHS="/Users/dev/workspace,/tmp"
export LLMSPELL_TOOLS_SYSTEM_ALLOW_PROCESS_EXEC="true"
export RUST_LOG="debug"
./target/release/llmspell run script.lua
```

**Single Command Override**:
```bash
# Override for one execution only
LLMSPELL_ALLOW_FILE_ACCESS=true ./target/release/llmspell run script.lua
```

**systemd Service**:
```ini
[Service]
Environment="LLMSPELL_ALLOW_FILE_ACCESS=false"
Environment="LLMSPELL_ALLOW_NETWORK_ACCESS=true"
Environment="LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS=api.internal.company.com"
Environment="LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS=localhost,127.0.0.1,169.254.169.254"
ExecStart=/usr/local/bin/llmspell kernel start
```

**Docker Compose**:
```yaml
services:
  llmspell:
    image: llmspell:latest
    environment:
      LLMSPELL_ALLOW_FILE_ACCESS: "false"
      LLMSPELL_ALLOW_NETWORK_ACCESS: "true"
      LLMSPELL_TOOLS_HTTP_ALLOWED_HOSTS: "api.example.com,*.company.com"
      LLMSPELL_TOOLS_HTTP_BLOCKED_HOSTS: "localhost,127.0.0.1,169.254.169.254"
      LLMSPELL_TOOLS_WEB_ALLOWED_DOMAINS: "*.openai.com,*.anthropic.com"
```

**See also**: [Security & Permissions Guide](security-and-permissions.md) for detailed security configuration and troubleshooting.

---

### Provider Variables

```bash
# LLM Providers
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GROQ_API_KEY="gsk_..."
export COHERE_API_KEY="..."

# Local Model Endpoints
export OLLAMA_BASE_URL="http://localhost:11434"
export LLAMA_CPP_URL="http://localhost:8080"
```

### RAG Variables (Phase 8)

```bash
# RAG Configuration
export LLMSPELL_RAG_ENABLED="true"
export LLMSPELL_RAG_MULTI_TENANT="false"
export LLMSPELL_RAG_DIMENSIONS="768"
export LLMSPELL_RAG_BACKEND="hnsw"
export LLMSPELL_RAG_PERSISTENCE_PATH="/var/lib/llmspell/rag"
export LLMSPELL_RAG_MAX_MEMORY_MB="1024"

# HNSW Configuration
export LLMSPELL_HNSW_M="16"
export LLMSPELL_HNSW_EF_CONSTRUCTION="200"
export LLMSPELL_HNSW_EF_SEARCH="50"
export LLMSPELL_HNSW_MAX_ELEMENTS="1000000"
export LLMSPELL_HNSW_METRIC="cosine"

# Embedding Configuration
export LLMSPELL_EMBEDDING_PROVIDER="openai"
export LLMSPELL_EMBEDDING_MODEL="text-embedding-3-small"
export LLMSPELL_EMBEDDING_CACHE="true"
export LLMSPELL_EMBEDDING_BATCH_SIZE="32"

# Multi-tenancy
export LLMSPELL_TENANT_ID="default"
export LLMSPELL_TENANT_ISOLATION="strict"
export LLMSPELL_TENANT_MAX_VECTORS="100000"
```

### Service Variables (Phase 10)

```bash
# systemd/launchd
export LLMSPELL_SERVICE_TYPE="systemd"  # systemd, launchd, windows
export LLMSPELL_SERVICE_NAME="llmspell-kernel"
export LLMSPELL_SERVICE_RESTART="on-failure"
export LLMSPELL_SERVICE_USER="llmspell"
export LLMSPELL_SERVICE_GROUP="llmspell"

# Signal handling
export LLMSPELL_HANDLE_SIGTERM="true"
export LLMSPELL_HANDLE_SIGHUP="true"
export LLMSPELL_HANDLE_SIGUSR1="true"
export LLMSPELL_HANDLE_SIGUSR2="true"
export LLMSPELL_GRACEFUL_SHUTDOWN_TIMEOUT="30"
```

### Monitoring Variables

```bash
# Metrics & Health
export LLMSPELL_METRICS_ENABLED="true"
export LLMSPELL_METRICS_PORT="9559"
export LLMSPELL_HEALTH_ENABLED="true"
export LLMSPELL_HEALTH_PORT="9559"

# Performance
export LLMSPELL_PROFILE_ENABLED="false"
export LLMSPELL_PROFILE_OUTPUT="/var/log/llmspell/profile.json"
export LLMSPELL_TRACE_ENABLED="false"
```

### Precedence

Environment variables override config file settings:
1. Command-line arguments (highest)
2. Environment variables
3. Config file
4. Defaults (lowest)

---

## Troubleshooting

### Config Loading Issues

```bash
# Validate configuration
./target/release/llmspell validate -c config.toml

# Show effective configuration
./target/release/llmspell config show

# Test specific profile
./target/release/llmspell --profile production validate
```

### Common Problems

**Config not found:**
```bash
# Check search paths
./target/release/llmspell config paths

# Use explicit path
./target/release/llmspell -c /absolute/path/config.toml run script.lua
```

**Invalid TOML:**
```bash
# Validate TOML syntax
toml-cli validate config.toml

# Check for typos in section names
grep -E '^\[' config.toml
```

**Environment variables not working:**
```bash
# Debug environment
env | grep LLMSPELL

# Export correctly
export LLMSPELL_CONFIG="/path/to/config.toml"  # Not just assignment
```

**Builtin profile not found:**
```bash
# List available builtin profiles
llmspell -p list  # Shows: minimal, development, providers, state, sessions, ollama, candle, rag-dev, rag-prod, rag-perf

# Use builtin profile
llmspell -p providers run script.lua

# For custom profiles (deprecated - use builtin or custom configs)
./target/release/llmspell --profile production run script.lua
```

**Need providers but getting errors:**
```bash
# Set API keys first
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Use providers builtin profile
llmspell -p providers run script.lua
```

**Need RAG but getting "RAG not available":**
```bash
# Set embedding API key
export OPENAI_API_KEY="sk-..."

# Use RAG builtin profile
llmspell -p rag-dev run script.lua      # Development
llmspell -p rag-prod run script.lua     # Production
```

**Kernel config issues:**
```bash
# Test kernel configuration
./target/release/llmspell kernel start --dry-run

# Verify ports available
netstat -an | grep 9555
```

### Debug Commands

```bash
# Show all configuration sources
./target/release/llmspell config debug

# Test configuration merge
./target/release/llmspell config test

# Export effective configuration
./target/release/llmspell config export > effective.toml

# Trace configuration loading
RUST_LOG=llmspell_config=trace ./target/release/llmspell run script.lua
```

---

## See Also

- [Core Concepts](concepts.md) - Understanding kernel architecture
- [Service Deployment](service-deployment.md) - Production deployment
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
- [Getting Started](getting-started.md) - Quick start guide
- [Lua API Reference](appendix/lua-api-reference.md) - Complete Lua scripting API
- [Rust API Reference](../developer-guide/reference/) - Complete Rust extension API