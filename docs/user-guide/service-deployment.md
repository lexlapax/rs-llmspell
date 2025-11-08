# Service Deployment Guide

**Version**: 0.9.0
**Last Updated**: December 2024

> **🚀 Production Deployment**: Deploy LLMSpell kernel as a system service with systemd (Linux) or launchd (macOS).

**🔗 Navigation**: [← User Guide](README.md) | [Configuration →](configuration.md) | [Troubleshooting →](troubleshooting.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Deployment Modes](#deployment-modes)
3. [systemd Deployment (Linux)](#systemd-deployment-linux)
4. [launchd Deployment (macOS)](#launchd-deployment-macos)
5. [Configuration](#configuration)
6. [Managing Services](#managing-services)
7. [Monitoring & Logging](#monitoring--logging)
8. [Security Best Practices](#security-best-practices)
9. [Troubleshooting](#troubleshooting)
10. [Programmatic Deployment](#programmatic-deployment)
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

**🔗 Next Steps**: [IDE Integration →](ide-integration.md) | [API Reference →](api/README.md)