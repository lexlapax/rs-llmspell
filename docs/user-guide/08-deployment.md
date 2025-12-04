# Service Deployment Guide

**Version**: 0.13.x (Phase 13)
**Last Updated**: December 2025

> **🚀 Production Deployment**: Deploy LLMSpell kernel as a system service with systemd (Linux) or launchd (macOS). Use builtin profiles for zero-config deployment.

**🔗 Navigation**: [← User Guide](README.md) | [Profile Guide →](profile-layers-guide.md) | [Configuration →](03-configuration.md) | [Troubleshooting →](10-troubleshooting.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Profile Selection by Environment](#profile-selection-by-environment)
3. [Deployment Modes](#deployment-modes)
4. [systemd Deployment (Linux)](#systemd-deployment-linux)
5. [launchd Deployment (macOS)](#launchd-deployment-macos)
6. [Configuration](#configuration)
7. [Managing Services](#managing-services)
8. [Monitoring & Logging](#monitoring--logging)
9. [Security Best Practices](#security-best-practices)
10. [Troubleshooting](#troubleshooting)
11. [Programmatic Deployment](#programmatic-deployment)
    - [Architecture Overview](#architecture-overview)
    - [Basic Embedded Service](#basic-embedded-service)
    - [HTTP Service Example](#http-service-example)
    - [Systemd Service for Custom Binary](#systemd-service-for-custom-binary)
    - [Docker Deployment](#docker-deployment)
    - [Infrastructure Module Access](#infrastructure-module-access)
    - [Production Configuration](#production-configuration)
    - [Comparison: CLI vs Programmatic](#comparison-cli-vs-programmatic)

---

## Quick Start

Install LLMSpell kernel as a service with auto-detection:

```bash
# Auto-detect platform and install as user service
./target/release/llmspell kernel install-service

# Install as system service with custom port
./target/release/llmspell kernel install-service --system --port 9600

# Enable and start immediately
./target/release/llmspell kernel install-service --enable --start
```

## Profile Selection by Environment

Use **builtin profiles** for zero-config deployment. See [Profile Guide](profile-layers-guide.md) for full details.

### Development
```bash
# Quick iteration with debug logging
llmspell -p development run script.lua

# Memory system debugging
llmspell -p memory-development run script.lua

# RAG development with verbose output
llmspell -p rag-dev run script.lua
```

### Staging
```bash
# PostgreSQL backend validation
export LLMSPELL_POSTGRES_URL="postgresql://user:pass@host:5432/llmspell_staging"
llmspell -p postgres-prod run script.lua

# Production RAG settings test
llmspell -p rag-prod run script.lua
```

### Production
```bash
# PostgreSQL with production settings (recommended)
export LLMSPELL_POSTGRES_URL="postgresql://user:pass@host:5432/llmspell_prod"
llmspell -p postgres-prod run script.lua

# Local LLM production (Ollama)
ollama serve
llmspell -p full-local-ollama run script.lua

# Cloud LLM production
llmspell -p openai-prod run script.lua  # or claude-prod, gemini-prod
```

### Profile Quick Reference

| Environment | Profile | Prerequisites |
|-------------|---------|---------------|
| Development | `development` | API keys |
| Memory Dev | `memory-development` | API keys |
| RAG Dev | `rag-dev` | API keys |
| Staging | `postgres-prod` | PostgreSQL + API keys |
| Production (Cloud) | `openai-prod`, `claude-prod` | API keys |
| Production (Local) | `full-local-ollama` | Ollama + models |
| Production (PG) | `postgres-prod` | PostgreSQL + API keys |

## Deployment Modes

### User Service
- Runs under your user account
- Starts when you log in
- Access to your home directory and environment
- Recommended for development and single-user systems

### System Service
- Runs as dedicated service user
- Starts at boot
- Isolated from user sessions
- Recommended for production servers

### Daemon Mode
- Background process with detached TTY
- PID file management
- Signal handling for graceful shutdown
- Automatic log rotation support

### Programmatic Deployment (Phase 13b.16)
- **Direct kernel API** - bypass CLI entirely for services
- **Infrastructure module** - ScriptRuntime creates all 9 components
- Embedded in Rust applications
- Custom service architectures
- Recommended for library integrations and custom servers

**Use Case**: Building a custom LLM service, web server, or embedded application that needs scriptable AI capabilities without CLI dependencies.

**Architecture**: Service → Kernel API → ScriptRuntime → Infrastructure (9 components)

See [Programmatic Deployment](#programmatic-deployment) section below for complete implementation examples.

## systemd Deployment (Linux)

### Installation

```bash
# Install as user service
./target/release/llmspell kernel install-service --service-type systemd

# Install as system service
sudo ./target/release/llmspell kernel install-service --system --service-type systemd

# Custom configuration
./target/release/llmspell kernel install-service \
  --name my-llmspell \
  --port 9600 \
  --log-file /var/log/llmspell/kernel.log \
  --pid-file /var/run/llmspell/kernel.pid
```

### Generated Service File

The installation creates a service file at:
- User service: `~/.config/systemd/user/llmspell-kernel.service`
- System service: `/etc/systemd/system/llmspell-kernel.service`

Example service file:

```ini
[Unit]
Description=LLMSpell Kernel Service
After=network.target

[Service]
Type=forking
ExecStart=/usr/local/bin/llmspell kernel start --daemon --port 9555 --log-file /var/log/llmspell/kernel.log --pid-file /var/run/llmspell/kernel.pid
ExecStop=/usr/local/bin/llmspell kernel stop --pid-file /var/run/llmspell/kernel.pid
PIDFile=/var/run/llmspell/kernel.pid
Restart=on-failure
RestartSec=10

# Security hardening
PrivateTmp=yes
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/log/llmspell /var/run/llmspell

[Install]
WantedBy=multi-user.target
```

### Management Commands

```bash
# Enable service to start at boot
systemctl --user enable llmspell-kernel  # User service
sudo systemctl enable llmspell-kernel    # System service

# Start/stop/restart
systemctl --user start llmspell-kernel
systemctl --user stop llmspell-kernel
systemctl --user restart llmspell-kernel

# Check status
systemctl --user status llmspell-kernel

# View logs
journalctl --user -u llmspell-kernel -f
```

## launchd Deployment (macOS)

### Installation

```bash
# Install as user agent
./target/release/llmspell kernel install-service --service-type launchd

# Install as system daemon
sudo ./target/release/llmspell kernel install-service --system --service-type launchd

# With custom settings
./target/release/llmspell kernel install-service \
  --name com.example.llmspell \
  --port 9600 \
  --enable \
  --start
```

### Generated Plist File

The installation creates a plist file at:
- User agent: `~/Library/LaunchAgents/com.llmspell.kernel.plist`
- System daemon: `/Library/LaunchDaemons/com.llmspell.kernel.plist`

Example plist:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.llmspell.kernel</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/llmspell</string>
        <string>kernel</string>
        <string>start</string>
        <string>--daemon</string>
        <string>--port</string>
        <string>9555</string>
        <string>--log-file</string>
        <string>/usr/local/var/log/llmspell/kernel.log</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/llmspell/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/llmspell/stderr.log</string>
</dict>
</plist>
```

### Management Commands

```bash
# Load/unload service
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist
launchctl unload ~/Library/LaunchAgents/com.llmspell.kernel.plist

# Start/stop
launchctl start com.llmspell.kernel
launchctl stop com.llmspell.kernel

# Check status
launchctl list | grep llmspell

# View logs
tail -f /usr/local/var/log/llmspell/kernel.log
```

## Configuration

### Service Configuration File

Create a dedicated configuration file for the service:

```toml
# /etc/llmspell/kernel.toml (Linux)
# /usr/local/etc/llmspell/kernel.toml (macOS)

[kernel]
port = 9555
connection_file = "/var/lib/llmspell/kernel.json"
idle_timeout = 0  # Never timeout for service

[daemon]
daemonize = true
pid_file = "/var/run/llmspell/kernel.pid"
working_dir = "/var/lib/llmspell"
umask = 0o077  # Restrictive permissions

[logging]
log_file = "/var/log/llmspell/kernel.log"
log_level = "info"
max_size_mb = 100
max_backups = 5
compress = true

[security]
hmac_key = "your-secret-key-here"
allowed_origins = ["localhost", "127.0.0.1"]
max_message_size_mb = 10
rate_limit_per_minute = 100
```

### Environment Variables

Set environment variables in the service file:

```bash
# systemd environment file: /etc/default/llmspell-kernel
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
RUST_LOG=info
LLMSPELL_CONFIG=/etc/llmspell/kernel.toml
```

## Managing Services

### Start/Stop Operations

```bash
# Start kernel daemon manually
./target/release/llmspell kernel start --daemon --port 9555

# Check status
./target/release/llmspell kernel status

# Stop by PID file
./target/release/llmspell kernel stop --pid-file /var/run/llmspell/kernel.pid

# Stop by kernel ID
./target/release/llmspell kernel stop abc123
```

### Signal Handling

The daemon responds to standard Unix signals:

- `SIGTERM`: Graceful shutdown
- `SIGINT`: Graceful shutdown (Ctrl+C)
- `SIGHUP`: Reload configuration
- `SIGUSR1`: Dump statistics
- `SIGUSR2`: Toggle debug logging

```bash
# Graceful reload
kill -HUP $(cat /var/run/llmspell/kernel.pid)

# Get statistics
kill -USR1 $(cat /var/run/llmspell/kernel.pid)
```

## Monitoring & Logging

### Health Checks

```bash
# Built-in health endpoint
curl http://localhost:9555/health

# Detailed metrics
curl http://localhost:9555/metrics
```

### Log Management

```bash
# View real-time logs
tail -f /var/log/llmspell/kernel.log

# Rotate logs manually
logrotate -f /etc/logrotate.d/llmspell-kernel
```

Example logrotate configuration:

```
/var/log/llmspell/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 llmspell llmspell
    postrotate
        kill -USR1 $(cat /var/run/llmspell/kernel.pid) 2>/dev/null || true
    endscript
}
```

## Security Best Practices

### 1. Run as Dedicated User

```bash
# Create service user
sudo useradd -r -s /bin/false -d /var/lib/llmspell llmspell

# Set ownership
sudo chown -R llmspell:llmspell /var/lib/llmspell
sudo chown -R llmspell:llmspell /var/log/llmspell
```

### 2. File Permissions

```bash
# Restrictive permissions
chmod 600 /etc/llmspell/kernel.toml      # Config file
chmod 644 /var/run/llmspell/kernel.pid   # PID file
chmod 640 /var/log/llmspell/kernel.log   # Log file
```

### 3. Network Security

```toml
[security]
# Enable HMAC authentication
hmac_key = "$(openssl rand -base64 32)"

# Restrict origins
allowed_origins = ["localhost"]

# Enable TLS (requires certificates)
tls_enabled = true
tls_cert = "/etc/llmspell/cert.pem"
tls_key = "/etc/llmspell/key.pem"
```

### 4. Resource Limits

```ini
# systemd resource limits
[Service]
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
CPUQuota=200%
```

## Troubleshooting

### Service Won't Start

1. Check permissions:
```bash
ls -la /var/run/llmspell/
ls -la /var/log/llmspell/
```

2. Check port availability:
```bash
lsof -i :9555
netstat -an | grep 9555
```

3. Verify configuration:
```bash
./target/release/llmspell kernel start --dry-run
```

### Connection Issues

1. Check if service is running:
```bash
systemctl --user status llmspell-kernel
ps aux | grep llmspell
```

2. Test connectivity:
```bash
nc -zv localhost 9555
curl http://localhost:9555/health
```

3. Check firewall:
```bash
sudo iptables -L -n | grep 9555
sudo ufw status | grep 9555
```

### Performance Issues

1. Monitor resource usage:
```bash
systemctl --user status llmspell-kernel
top -p $(cat /var/run/llmspell/kernel.pid)
```

2. Check logs for warnings:
```bash
grep WARN /var/log/llmspell/kernel.log
grep ERROR /var/log/llmspell/kernel.log
```

3. Enable debug logging:
```bash
kill -USR2 $(cat /var/run/llmspell/kernel.pid)
```

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Address already in use" | Port conflict | Change port or stop conflicting service |
| "Permission denied" | Insufficient privileges | Check file permissions and user |
| "Connection refused" | Service not running | Start service and check logs |
| "Too many open files" | ulimit too low | Increase file descriptor limit |
| "Cannot create PID file" | Directory doesn't exist | Create directory with proper permissions |

## Programmatic Deployment

**Phase 13b.16**: Deploy kernel directly via Rust API without CLI dependency.

### Architecture Overview

```
Your Service
    ↓
Kernel API (llmspell_kernel::api)
    ↓
ScriptRuntime::new() [Phase 13b.16.3]
    ↓
Infrastructure::from_config() [Phase 13b.16]
    ↓
9 Components: ProviderManager, StateManager, SessionManager,
              RAG, MemoryManager, ToolRegistry, AgentRegistry,
              WorkflowFactory, ComponentRegistry
```

**Key Benefits**:
- Zero CLI dependencies in production
- Embedded in custom Rust applications
- Full control over lifecycle and configuration
- Same Infrastructure module pattern as CLI
- Direct kernel message protocol access

### Basic Embedded Service

Minimal service with embedded kernel:

```rust
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::start_embedded_kernel_with_executor;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = LLMSpellConfig::load_from_file("config.toml").await?;

    // Phase 13b.16.3: ScriptRuntime creates ALL infrastructure
    let script_executor = Arc::new(
        ScriptRuntime::new(config.clone()).await?
    ) as Arc<dyn llmspell_core::traits::script_executor::ScriptExecutor>;

    // Start embedded kernel
    let mut kernel_handle = start_embedded_kernel_with_executor(
        config.clone(),
        script_executor,
    ).await?;

    // Execute script
    let result = kernel_handle.execute(r#"
        local result = Agent.query("What is Rust?")
        return result
    "#).await?;

    println!("Result: {}", result);

    Ok(())
}
```

**Code References**:
- llmspell-kernel/src/api.rs:1093-1096 (start_embedded_kernel_with_executor)
- llmspell-bridge/src/lib.rs (ScriptRuntime::new)
- llmspell-bridge/src/infrastructure.rs (Infrastructure::from_config)

### HTTP Service Example

Build a REST API service with LLMSpell kernel:

```rust
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelHandle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    kernel: Arc<Mutex<KernelHandle>>,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    code: String,
}

#[derive(Serialize)]
struct ExecuteResponse {
    result: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize kernel
    let config = LLMSpellConfig::from_file("config.toml")?;
    let script_executor = Arc::new(ScriptRuntime::new(config.clone()).await?)
        as Arc<dyn llmspell_core::traits::script_executor::ScriptExecutor>;

    let kernel_handle = start_embedded_kernel_with_executor(
        config,
        script_executor,
    ).await?;

    let state = AppState {
        kernel: Arc::new(Mutex::new(kernel_handle)),
    };

    // Build router
    let app = Router::new()
        .route("/execute", post(execute_handler))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn execute_handler(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Json<ExecuteResponse> {
    let mut kernel = state.kernel.lock().await;

    match kernel.execute(&req.code).await {
        Ok(result) => Json(ExecuteResponse { result }),
        Err(e) => Json(ExecuteResponse {
            result: format!("Error: {}", e),
        }),
    }
}
```

**Dependencies** (Cargo.toml):
```toml
[dependencies]
llmspell-bridge = { path = "../llmspell-bridge" }
llmspell-config = { path = "../llmspell-config" }
llmspell-kernel = { path = "../llmspell-kernel" }
llmspell-core = { path = "../llmspell-core" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
```

### Systemd Service for Custom Binary

Deploy your programmatic service with systemd:

```ini
[Unit]
Description=Custom LLMSpell HTTP Service
After=network.target

[Service]
Type=simple
User=llmspell
Group=llmspell
WorkingDirectory=/opt/llmspell-service
ExecStart=/opt/llmspell-service/my-service
Restart=on-failure
RestartSec=10

# Environment
Environment="RUST_LOG=info"
Environment="LLMSPELL_CONFIG=/etc/llmspell/config.toml"

# Security hardening
PrivateTmp=yes
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/llmspell /var/lib/llmspell

[Install]
WantedBy=multi-user.target
```

**Installation**:
```bash
# Build release binary
cargo build --release

# Copy binary
sudo cp target/release/my-service /opt/llmspell-service/

# Install systemd service
sudo cp my-service.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable my-service
sudo systemctl start my-service

# Check status
sudo systemctl status my-service
```

### Docker Deployment

Dockerfile for programmatic service:

```dockerfile
FROM rust:1.83 AS builder

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release --bin my-service

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 llmspell

# Copy binary and config
COPY --from=builder /app/target/release/my-service /usr/local/bin/
COPY config.toml /etc/llmspell/config.toml

# Set ownership
RUN chown -R llmspell:llmspell /etc/llmspell

USER llmspell
WORKDIR /home/llmspell

# Expose HTTP port
EXPOSE 3000

CMD ["/usr/local/bin/my-service"]
```

**docker-compose.yml** with PostgreSQL:

```yaml
services:
  postgres:
    image: ghcr.io/tensorchord/vchord-postgres:pg18-v0.5.3
    container_name: llmspell_postgres
    environment:
      POSTGRES_DB: llmspell_prod
      POSTGRES_USER: llmspell
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U llmspell"]
      interval: 10s
      timeout: 5s
      retries: 5

  llmspell-service:
    build: .
    container_name: llmspell_service
    ports:
      - "3000:3000"
    environment:
      RUST_LOG: info
      LLMSPELL_CONFIG: /etc/llmspell/config.toml
      OPENAI_API_KEY: ${OPENAI_API_KEY}
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
    volumes:
      - ./config.toml:/etc/llmspell/config.toml:ro
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  postgres_data:
```

**Configuration** (config.toml):
```toml
[storage]
backend = "postgres"

[storage.postgres]
url = "postgresql://llmspell:${POSTGRES_PASSWORD}@postgres:5432/llmspell_prod"
pool_size = 20
auto_migrate = true
enforce_tenant_isolation = true

[providers.openai]
enabled = true
api_key_env = "OPENAI_API_KEY"

[providers.anthropic]
enabled = true
api_key_env = "ANTHROPIC_API_KEY"
```

**Deployment**:
```bash
# Build and start
docker-compose up -d

# Check logs
docker-compose logs -f llmspell-service

# Scale service
docker-compose up -d --scale llmspell-service=3

# Health check
curl http://localhost:3000/execute -d '{"code":"return 42"}'
```

### Infrastructure Module Access

Access components created by Infrastructure module:

```rust
use llmspell_bridge::infrastructure::Infrastructure;
use llmspell_config::LLMSpellConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = LLMSpellConfig::from_file("config.toml")?;

    // Create infrastructure (9 components)
    let infrastructure = Infrastructure::from_config(&config).await?;

    // Access individual components
    let provider_manager = infrastructure.provider_manager();
    let state_manager = infrastructure.state_manager();
    let session_manager = infrastructure.session_manager();

    // Optional components (if enabled in config)
    if let Some(rag) = infrastructure.rag() {
        println!("RAG enabled");
    }

    if let Some(memory_manager) = infrastructure.memory_manager() {
        println!("Memory enabled");
    }

    // Tool and agent registries
    let tool_registry = infrastructure.tool_registry();
    let agent_registry = infrastructure.agent_registry();

    // Workflow and component factories
    let workflow_factory = infrastructure.workflow_factory();
    let component_registry = infrastructure.component_registry();

    Ok(())
}
```

**Code References**:
- llmspell-bridge/src/infrastructure.rs:154-248 (Infrastructure::from_config)
- llmspell-bridge/src/infrastructure.rs:250-310 (accessor methods)

### Production Configuration

Optimized configuration for programmatic services:

```toml
[kernel]
# Disable CLI-specific features
idle_timeout = 0  # Never timeout in service mode

[daemon]
daemonize = false  # systemd/Docker handles this
pid_file = "/var/run/llmspell/service.pid"

[logging]
log_level = "info"
log_file = "/var/log/llmspell/service.log"
max_size_mb = 100
max_backups = 10
compress = true

[storage]
backend = "postgres"  # Production storage

[storage.postgres]
url = "${DATABASE_URL}"  # From environment
pool_size = 20  # Formula: (CPU × 2) + 1
enforce_tenant_isolation = true
auto_migrate = false  # Run migrations separately

[providers.openai]
enabled = true
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4-turbo"
timeout_secs = 300

[memory]
enable_memory = true
enable_rag = true

[memory.episodic.hnsw]
m = 16  # Bi-directional links
ef_construction = 128  # Build quality
ef_search = 40  # Query speed vs recall
```

**Environment Variables** (.env):
```bash
DATABASE_URL=postgresql://user:pass@postgres:5432/llmspell_prod
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
RUST_LOG=info,llmspell=debug
```

### Comparison: CLI vs Programmatic

| Aspect | CLI Deployment | Programmatic Deployment |
|--------|---------------|------------------------|
| **Use Case** | Standalone kernel server | Embedded in Rust app |
| **Entry Point** | `llmspell kernel start` | `start_embedded_kernel_with_executor()` |
| **Dependencies** | llmspell CLI binary | Crate dependencies only |
| **Infrastructure** | Created by CLI → ScriptRuntime | Created by your code → ScriptRuntime |
| **Lifecycle** | Managed by systemd/launchd | Managed by your application |
| **Configuration** | CLI flags + config file | Config file or builder pattern |
| **Ideal For** | General-purpose kernel | Custom services, web APIs |

**When to Use Programmatic**:
- Building a web service with LLM capabilities
- Embedding in larger Rust application
- Custom server architectures
- Library integration
- Fine-grained control over lifecycle

**When to Use CLI**:
- Standalone kernel server
- Multi-client execution
- Development and testing
- IDE integration (Jupyter, VS Code)

---

**🔗 Next Steps**: [Security Guide →](09-security.md) | [Troubleshooting →](10-troubleshooting.md) | [Lua API Reference](appendix/lua-api-reference.md)

---

## IDE Integration

**Version**: 0.9.0
**Last Updated**: December 2024

> **🔌 IDE Integration**: Connect LLMSpell kernel with VS Code, Jupyter Lab, vim/neovim, and other development environments.

**🔗 Navigation**: [← Deployment](README.md#08-deployment) | [Security →](09-security.md) | [Lua API](appendix/lua-api-reference.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Kernel Connection Basics](#kernel-connection-basics)
3. [VS Code Setup](#vs-code-setup)
4. [Jupyter Lab Integration](#jupyter-lab-integration)
5. [vim/neovim Setup](#vimneovim-setup)
6. [Connection File Format](#connection-file-format)
7. [Debug Adapter Protocol (DAP)](#debug-adapter-protocol-dap)
8. [Common Workflows](#common-workflows)
9. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Start LLMSpell Kernel

```bash
# Start kernel with connection file for IDE discovery
./target/release/llmspell kernel start --connection-file ~/.llmspell/kernel.json

# Start in background with specific port
./target/release/llmspell kernel start --daemon --port 9555 --connection-file ~/.llmspell/kernel.json

# Check kernel is running
./target/release/llmspell kernel status
```

### Connect from IDE

**VS Code**: Install Jupyter extension, use "Connect to Existing Jupyter Server"
**Jupyter Lab**: Use connection file with `--existing` flag
**vim/neovim**: Configure LSP client with kernel connection

## Kernel Connection Basics

LLMSpell kernel implements the Jupyter protocol with 5 ZeroMQ channels:
- **Shell**: Execute requests and replies
- **IOPub**: Broadcast outputs and status
- **Stdin**: Input requests (prompts)
- **Control**: Control commands (shutdown, interrupt)
- **Heartbeat**: Connection liveness monitoring

The kernel can run in two modes:
1. **Embedded**: Spawns in background thread (default)
2. **External**: Connects to standalone kernel process

## VS Code Setup

### Prerequisites

1. Install VS Code extensions:
   - **Jupyter** (Microsoft)
   - **Python** (Microsoft) - for notebook support
   - Optional: **CodeLLDB** for debugging

### Connect to LLMSpell Kernel

#### Method 1: Using Connection File

1. Start LLMSpell kernel:
```bash
./target/release/llmspell kernel start \
  --connection-file ~/.llmspell/kernel.json \
  --daemon
```

2. In VS Code:
   - Open Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
   - Run: "Jupyter: Specify Jupyter Server for Connections"
   - Select: "Existing: Specify the URI of an existing server"
   - Enter the connection file path

3. Create/Open a notebook:
   - Create new file with `.ipynb` extension
   - Select "LLMSpell" as kernel

#### Method 2: Manual Connection

1. Get connection info:
```bash
cat ~/.llmspell/kernel.json
```

2. In VS Code settings.json:
```json
{
  "jupyter.kernels.trusted": [
    "/path/to/.llmspell/kernel.json"
  ],
  "jupyter.jupyterServerType": "remote",
  "jupyter.jupyterServer.uriList": [
    {
      "name": "LLMSpell Kernel",
      "uri": "http://localhost:9555/?token=your-token"
    }
  ]
}
```

### Debug Adapter Protocol (DAP) Setup

1. Create `.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "llmspell",
      "request": "attach",
      "name": "Attach to LLMSpell",
      "host": "localhost",
      "port": 9556,
      "pathMappings": [
        {
          "localRoot": "${workspaceFolder}",
          "remoteRoot": "/workspace"
        }
      ]
    }
  ]
}
```

2. Start debugging:
   - Set breakpoints in Lua/JavaScript files
   - Press F5 to start debugging
   - Use debug console for REPL

### Features in VS Code

- **IntelliSense**: Code completion for Lua/JavaScript
- **Variable Explorer**: View variables in notebook interface
- **Debugging**: Breakpoints, stepping, watch expressions
- **Output Streaming**: Real-time execution output
- **Markdown Support**: Rich text cells in notebooks

## Jupyter Lab Integration

### Installation

```bash
# Install Jupyter Lab if needed
pip install jupyterlab

# Optional: Install kernel spec globally
mkdir -p ~/.local/share/jupyter/kernels/llmspell
```

### Create Kernel Spec

Create `~/.local/share/jupyter/kernels/llmspell/kernel.json`:

```json
{
  "display_name": "LLMSpell",
  "language": "lua",
  "argv": [
    "/usr/local/bin/llmspell",
    "kernel",
    "start",
    "--connection-file",
    "{connection_file}"
  ],
  "metadata": {
    "debugger": true
  }
}
```

### Connect to Running Kernel

```bash
# Start LLMSpell kernel
./target/release/llmspell kernel start \
  --connection-file ~/.llmspell/kernel.json \
  --daemon

# Connect Jupyter Lab
jupyter lab --existing ~/.llmspell/kernel.json
```

### Use in Notebooks

```python
# In Jupyter notebook cell (Lua syntax)
%%lua
-- LLMSpell Lua code
local agent = Agent.builder()
  :model("openai/gpt-4o-mini")
  :build()

local result = agent:execute({
  prompt = "Explain quantum computing"
})

print(result.content)
```

### Features in Jupyter Lab

- **Multi-language Support**: Lua, JavaScript, Python (via bridge)
- **Rich Output**: HTML, images, plots
- **Interactive Widgets**: ipywidgets compatible
- **Variable Inspector**: Built-in variable explorer
- **Code Completion**: Tab completion support
- **Magic Commands**: Custom %%lua, %%js magics

## vim/neovim Setup

### LSP Configuration (Neovim)

1. Install required plugins:
```vim
" Using packer.nvim
use 'neovim/nvim-lspconfig'
use 'hrsh7th/nvim-cmp'
use 'hrsh7th/cmp-nvim-lsp'
use 'L3MON4D3/LuaSnip'
```

2. Configure LLMSpell LSP in `init.lua`:
```lua
local lspconfig = require('lspconfig')

-- Custom LLMSpell LSP config
lspconfig.llmspell = {
  default_config = {
    cmd = {
      'llmspell', 'kernel', 'start',
      '--lsp',
      '--port', '9557'
    },
    filetypes = {'lua', 'javascript'},
    root_dir = lspconfig.util.root_pattern('.llmspell', '.git'),
    settings = {
      llmspell = {
        enableSnippets = true,
        enableDebug = true
      }
    }
  }
}

lspconfig.llmspell.setup{
  on_attach = function(client, bufnr)
    -- Enable completion
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

    -- Keybindings
    local opts = { noremap=true, silent=true, buffer=bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
  end,
  capabilities = require('cmp_nvim_lsp').default_capabilities()
}
```

### REPL Integration

1. Install vim-slime or similar:
```vim
use 'jpalardy/vim-slime'
```

2. Configure REPL connection:
```vim
" .vimrc or init.vim
let g:slime_target = "tmux"
let g:slime_default_config = {
  \ 'socket_name': get(split($TMUX, ','), 0),
  \ 'target_pane': '{last}'
\ }

" Custom command for LLMSpell REPL
command! LLMSpellREPL :terminal llmspell repl --kernel localhost:9555
```

3. Keybindings for REPL:
```vim
" Send selection to REPL
xmap <leader>s <Plug>SlimeRegionSend
" Send paragraph to REPL
nmap <leader>s <Plug>SlimeParagraphSend
" Send cell to REPL
nmap <leader>c <Plug>SlimeConfig
```

### DAP Debugging (Neovim)

1. Install nvim-dap:
```vim
use 'mfussenegger/nvim-dap'
use 'rcarriga/nvim-dap-ui'
```

2. Configure DAP for LLMSpell:
```lua
local dap = require('dap')

dap.adapters.llmspell = {
  type = 'server',
  host = 'localhost',
  port = 9556
}

dap.configurations.lua = {
  {
    type = 'llmspell',
    request = 'launch',
    name = 'Launch LLMSpell Script',
    program = '${file}',
    cwd = '${workspaceFolder}',
    stopOnEntry = false
  }
}
```

## Connection File Format

The kernel creates a connection file with ZeroMQ endpoints:

```json
{
  "transport": "tcp",
  "ip": "127.0.0.1",
  "shell_port": 50510,
  "iopub_port": 50511,
  "stdin_port": 50512,
  "control_port": 50513,
  "hb_port": 50514,
  "key": "a0b1c2d3-e4f5-6789-abcd-ef0123456789",
  "signature_scheme": "hmac-sha256",
  "kernel_name": "llmspell"
}
```

### Security

- **HMAC Authentication**: Messages signed with `key`
- **Local Only**: Default binding to localhost
- **Port Range**: Configurable port allocation
- **Token Auth**: Optional token for HTTP endpoints

## Debug Adapter Protocol (DAP)

LLMSpell implements DAP for IDE debugging integration:

### Supported Features

- **Breakpoints**: Line and conditional
- **Stepping**: Step in/over/out, continue
- **Variables**: Locals, globals, watch expressions
- **Call Stack**: Full stack trace
- **REPL**: Debug console evaluation
- **Source Maps**: Accurate file:line mapping

### DAP Commands

```lua
-- In debug console
:break 10           -- Set breakpoint at line 10
:watch myVar        -- Add watch expression
:locals             -- Show local variables
:stack              -- Show call stack
:continue           -- Resume execution
:step               -- Step into
:next               -- Step over
:finish             -- Step out
```

## Common Workflows

### Interactive Development

1. Start kernel in development mode:
```bash
./target/release/llmspell kernel start \
  --connection-file ~/.llmspell/kernel.json \
  --idle-timeout 0 \
  --trace debug
```

2. Connect IDE of choice
3. Create notebook or script
4. Iterate with hot-reload:
```lua
-- Reload modules
package.loaded['mymodule'] = nil
local mymodule = require('mymodule')
```

### Production Debugging

1. Connect to production kernel:
```bash
# Get connection info from production
ssh prod-server cat /var/lib/llmspell/kernel.json > local-kernel.json

# Connect locally
./target/release/llmspell kernel connect --connection-file local-kernel.json
```

2. Attach debugger:
```bash
# In IDE, use remote debugging with SSH tunneling
ssh -L 9556:localhost:9556 prod-server
```

### Multi-Client Collaboration

1. Start kernel with high client limit:
```bash
./target/release/llmspell kernel start \
  --max-clients 50 \
  --connection-file /shared/kernel.json
```

2. Share connection file with team
3. Multiple developers connect simultaneously
4. Shared state and debugging sessions

## Troubleshooting

### Connection Issues

**Issue**: "Could not connect to kernel"
```bash
# Check kernel is running
./target/release/llmspell kernel status

# Check ports are open
lsof -i :9555

# Test ZeroMQ connectivity
nc -zv localhost 50510-50514
```

**Issue**: "Authentication failed"
```bash
# Verify HMAC key matches
cat ~/.llmspell/kernel.json | jq .key

# Regenerate connection file
rm ~/.llmspell/kernel.json
./target/release/llmspell kernel start --connection-file ~/.llmspell/kernel.json
```

### VS Code Issues

**Issue**: "No kernel specs found"
```bash
# Install kernel spec manually
jupyter kernelspec install --user ~/.local/share/jupyter/kernels/llmspell

# List installed kernels
jupyter kernelspec list
```

**Issue**: "IntelliSense not working"
```json
// settings.json
{
  "jupyter.enableExtendedKernelCompletions": true,
  "jupyter.enableCellCodeLens": true
}
```

### Jupyter Lab Issues

**Issue**: "Kernel keeps disconnecting"
```bash
# Increase timeout
./target/release/llmspell kernel start --idle-timeout 0

# Check for memory issues
top -p $(pgrep llmspell)
```

**Issue**: "Variables not showing"
```python
# Enable variable inspector
%config InlineBackend.figure_format = 'retina'
%load_ext autoreload
%autoreload 2
```

### vim/neovim Issues

**Issue**: "LSP not attaching"
```vim
:LspInfo  " Check LSP status
:LspLog   " View LSP logs
:checkhealth lsp  " Diagnose issues
```

**Issue**: "Completion not working"
```lua
-- Check capabilities
:lua print(vim.inspect(vim.lsp.get_active_clients()))

-- Force completion
:lua vim.lsp.buf.completion()
```

### Performance Tips

1. **Use connection pooling**: Reuse kernel connections
2. **Enable caching**: Cache frequently used completions
3. **Limit concurrent operations**: Set max-clients appropriately
4. **Monitor resources**: Watch memory/CPU usage
5. **Use local kernels**: Reduce network latency

---

**🔗 Next Steps**: [Security Guide →](09-security.md) | [Troubleshooting →](10-troubleshooting.md) | [Examples →](../../examples/README.md)