# Phase 10: Service Integration & IDE Connectivity - Design Document

**Version**: 2.0 (Refactored for Single-Binary Daemon Architecture)
**Date**: October 2025
**Status**: Design Ready
**Phase**: 10 (Service Integration & IDE Connectivity)
**Timeline**: Weeks 33-36 (20 working days)
**Priority**: HIGH (Critical for Developer Experience and External Tool Integration)
**Dependencies**: Phase 9 Kernel Infrastructure ✅
**Architecture Document**: `docs/technical/master-architecture-vision.md`
**Implementation Phases**: `docs/in-progress/implementation-phases.md`
**CLI Architecture**: `docs/technical/cli-command-architecture.md`

> **📋 Service Integration Foundation**: This phase enhances the kernel with daemon mode capabilities, enabling multi-protocol server operation (Jupyter, DAP, LSP, REPL) while maintaining a single-binary architecture with proper Unix daemon behavior.

---

## Phase Overview

### Goal
Enhance `llmspell-kernel` with daemon mode capabilities and multi-protocol server support, transforming llmspell from a CLI tool into a proper Unix service that IDEs, notebooks, and other tools can connect to. The kernel operates as a single process that can run either embedded (foreground) or daemon (background) mode.

This phase includes comprehensive example applications demonstrating production service capabilities: **Kernel Fleet Manager** for orchestrating multiple kernel instances and **Development Environment Service** for IDE integration with code intelligence.

### Core Principles
- **Single Binary**: `llmspell` is the only executable, no separate service binary
- **Daemon Mode**: Proper Unix daemon with double-fork, setsid, I/O redirection
- **Signal Handling**: Unix signals convert to Jupyter protocol messages
- **Multi-Protocol**: Single kernel process serves Jupyter, DAP, LSP, REPL
- **Process Management**: PID files, connection files, kernel discovery
- **Logging Architecture**: stdout/stderr redirection, rotating logs, syslog
- **Production-Ready**: systemd/launchd integration with Type=forking
- **IDE-Native**: First-class support for VS Code, Jupyter Lab, vim/neovim
- **Performance Critical**: <5ms message handling, <20ms debug stepping
- **Security-First**: TLS, authentication, session isolation

### Implementation Strategy

**Architecture**:
- Enhance `llmspell-kernel` with daemon and multi-protocol capabilities
- No separate service crate - all functionality in kernel module
- Use `llmspell kernel start --daemon` for service mode
- Protocol servers run within kernel process, sharing event loop

**Key Design Decisions**:
1. **Integrated Architecture**: Protocol servers embedded in kernel
2. **Unix Daemon Pattern**: Double-fork, setsid, detach from TTY
3. **Signal-to-Message Bridge**: Convert signals to protocol messages
4. **Shared Runtime**: Single tokio runtime for all protocols
5. **Connection Discovery**: Jupyter-compatible connection files

### Success Criteria
- [ ] `llmspell kernel start --daemon` properly daemonizes
- [ ] Process detaches from TTY with double-fork technique
- [ ] Signals (SIGTERM, SIGINT) convert to Jupyter messages
- [ ] stdout/stderr redirect to rotating log files
- [ ] Jupyter Lab connects via ZeroMQ using connection file
- [ ] VS Code debugging works with <20ms stepping
- [ ] Multiple clients connect simultaneously
- [ ] PID file prevents multiple instances
- [ ] systemd/launchd manages kernel lifecycle
- [ ] Performance targets met (<5ms message handling)
- [ ] Example applications demonstrate production service capabilities
- [ ] Fleet manager orchestrates multiple kernel instances
- [ ] Dev service provides functional IDE integration

---

## 1. Unix Daemon Implementation

### 1.1 Daemon Mode Architecture

The kernel implements proper Unix daemon behavior using the double-fork technique:

```rust
// llmspell-kernel/src/daemon.rs
use nix::unistd::{fork, ForkResult, setsid, chdir, close};
use nix::sys::signal::{signal, SigHandler, Signal};
use nix::sys::stat::{umask, Mode};
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::Path;
use std::process;
use tracing::info;

/// Unix daemon implementation for kernel service mode
pub struct DaemonManager {
    /// PID file path for preventing multiple instances
    pid_file: PathBuf,

    /// Log file for stdout/stderr redirection
    log_file: PathBuf,

    /// Whether to run in foreground (for debugging)
    foreground: bool,
}

impl DaemonManager {
    pub fn new(pid_file: PathBuf, log_file: PathBuf, foreground: bool) -> Self {
        Self {
            pid_file,
            log_file,
            foreground,
        }
    }

    /// Daemonize the process using double-fork technique
    pub fn daemonize(&self) -> Result<()> {
        if self.foreground {
            info!("Running in foreground mode (--foreground flag)");
            return Ok(());
        }

        // Check if already running
        self.check_pid_file()?;

        // First fork - parent exits, child continues
        match unsafe { fork()? } {
            ForkResult::Parent { .. } => {
                // Parent process exits successfully
                process::exit(0);
            }
            ForkResult::Child => {}
        }

        // Create new session, become session leader
        setsid()?;

        // Ignore SIGHUP (terminal hangup)
        unsafe {
            signal(Signal::SIGHUP, SigHandler::SigIgn)?;
        }

        // Second fork - ensures we can't reacquire controlling terminal
        match unsafe { fork()? } {
            ForkResult::Parent { .. } => {
                // First child exits
                process::exit(0);
            }
            ForkResult::Child => {}
        }

        // Now we're the grandchild process - the actual daemon

        // Change working directory to root
        chdir("/")?;

        // Set file creation mask
        umask(Mode::from_bits_truncate(0o027));

        // Redirect stdin/stdout/stderr
        self.redirect_io()?;

        // Write PID file
        self.write_pid_file()?;

        info!("Daemon started with PID {}", process::id());
        Ok(())
    }

    /// Redirect stdin/stdout/stderr for daemon mode
    fn redirect_io(&self) -> Result<()> {
        // Close stdin and redirect to /dev/null
        let dev_null = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?;

        // Redirect stdin (fd 0)
        unsafe {
            close(0)?;
            let _ = File::from_raw_fd(dev_null.as_raw_fd());
        }

        // Open log file for stdout/stderr
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        // Redirect stdout (fd 1)
        unsafe {
            close(1)?;
            let _ = File::from_raw_fd(log.as_raw_fd());
        }

        // Redirect stderr (fd 2)
        unsafe {
            close(2)?;
            let _ = File::from_raw_fd(log.try_clone()?.as_raw_fd());
        }

        Ok(())
    }

    /// Write PID file to prevent multiple instances
    fn write_pid_file(&self) -> Result<()> {
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.pid_file)?;

        writeln!(file, "{}", process::id())?;
        Ok(())
    }

    /// Check if daemon is already running
    fn check_pid_file(&self) -> Result<()> {
        if self.pid_file.exists() {
            let pid = std::fs::read_to_string(&self.pid_file)?
                .trim()
                .parse::<i32>()?;

            // Check if process is still running
            match nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                None,
            ) {
                Ok(_) => {
                    return Err(anyhow!(
                        "Daemon already running with PID {}",
                        pid
                    ));
                }
                Err(_) => {
                    // Process not running, remove stale PID file
                    std::fs::remove_file(&self.pid_file)?;
                }
            }
        }
        Ok(())
    }

    /// Clean up on daemon exit
    pub fn cleanup(&self) -> Result<()> {
        if self.pid_file.exists() {
            std::fs::remove_file(&self.pid_file)?;
        }
        Ok(())
    }
}
```

## 2. Signal Handling Architecture

### 2.1 Signal-to-Message Bridge

Convert Unix signals to Jupyter protocol messages for graceful shutdown:

```rust
// llmspell-kernel/src/signals.rs
use nix::sys::signal::{SigSet, Signal};
use nix::sys::signalfd::{SignalFd, SfdFlags};
use tokio::sync::mpsc;
use llmspell_kernel::{JupyterMessage, ControlChannel};

/// Signal handler that converts Unix signals to protocol messages
pub struct SignalBridge {
    /// Signal file descriptor for async signal handling
    signal_fd: SignalFd,

    /// Channel to send shutdown/interrupt messages
    control_tx: mpsc::Sender<JupyterMessage>,

    /// Shutdown sequence state
    shutdown_state: Arc<Mutex<ShutdownState>>,
}

#[derive(Debug, Clone, Copy)]
enum ShutdownState {
    Running,
    Interrupting,      // SIGINT received
    ShuttingDown,      // SIGTERM received
    ForceShutdown,     // Second SIGTERM
}

impl SignalBridge {
    pub fn new(control_tx: mpsc::Sender<JupyterMessage>) -> Result<Self> {
        // Block signals for signalfd
        let mut mask = SigSet::empty();
        mask.add(Signal::SIGTERM);
        mask.add(Signal::SIGINT);
        mask.add(Signal::SIGHUP);
        mask.add(Signal::SIGUSR1);
        mask.thread_block()?;

        // Create signalfd for async handling
        let signal_fd = SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK)?;

        Ok(Self {
            signal_fd,
            control_tx,
            shutdown_state: Arc::new(Mutex::new(ShutdownState::Running)),
        })
    }

    /// Start signal handling loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Wait for signal
            let signal_info = self.signal_fd.read_signal()?;

            if let Some(info) = signal_info {
                match Signal::try_from(info.ssi_signo as i32)? {
                    Signal::SIGTERM => {
                        self.handle_sigterm().await?;
                    }
                    Signal::SIGINT => {
                        self.handle_sigint().await?;
                    }
                    Signal::SIGHUP => {
                        self.handle_sighup().await?;
                    }
                    Signal::SIGUSR1 => {
                        self.handle_sigusr1().await?;
                    }
                    _ => {}
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Handle SIGTERM - graceful shutdown
    async fn handle_sigterm(&mut self) -> Result<()> {
        let mut state = self.shutdown_state.lock().unwrap();

        match *state {
            ShutdownState::Running => {
                info!("SIGTERM received, initiating graceful shutdown");
                *state = ShutdownState::ShuttingDown;

                // Send interrupt first to stop execution
                self.send_interrupt_request().await?;

                // Wait briefly for interrupt to complete
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Send shutdown request
                self.send_shutdown_request().await?;
            }
            ShutdownState::ShuttingDown => {
                warn!("Second SIGTERM, forcing shutdown");
                *state = ShutdownState::ForceShutdown;
                process::exit(1);
            }
            _ => {}
        }

        Ok(())
    }

    /// Send shutdown_request on control channel
    async fn send_shutdown_request(&mut self) -> Result<()> {
        let msg = JupyterMessage {
            header: MessageHeader {
                msg_type: "shutdown_request".to_string(),
                msg_id: uuid::Uuid::new_v4().to_string(),
                session: String::new(),
                username: "kernel".to_string(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: HashMap::new(),
            content: json!({"restart": false}),
        };

        self.control_tx.send(msg).await?;
        Ok(())
    }
}
```

## 3. Kernel Service Architecture

### 3.1 Enhanced Kernel with Protocol Servers

The kernel process hosts all protocol servers internally:

```rust
// llmspell-kernel/src/service.rs
use llmspell_kernel::{IntegratedKernel, KernelConfig};
use dashmap::DashMap;
use std::sync::Arc;

/// Enhanced kernel that runs protocol servers in daemon mode
pub struct KernelService {
    /// The core kernel instance
    kernel: Arc<IntegratedKernel>,

    /// Daemon manager for process control
    daemon: Option<DaemonManager>,

    /// Signal bridge for signal handling
    signal_bridge: Option<SignalBridge>,

    /// Active protocol servers
    servers: DashMap<String, Arc<dyn ProtocolServer>>,

    /// Client connection registry
    clients: Arc<ClientRegistry>,

    /// Service configuration
    config: KernelServiceConfig,

    /// Connection file path for Jupyter
    connection_file: PathBuf,

    /// Log configuration
    pub logging: LogConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Protocol-specific configurations
    pub jupyter: JupyterConfig,
    pub dap: DAPConfig,
    pub lsp: LSPConfig,
    pub repl: REPLConfig,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Health check configuration
    pub health: HealthConfig,

    /// Metrics export configuration
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterConfig {
    pub enabled: bool,

    /// ZeroMQ binding ports
    pub ports: JupyterPorts,

    /// Connection file path for Jupyter discovery
    pub connection_file: PathBuf,

    /// HMAC key for message signing
    pub key: String,

    /// IP to bind to
    pub ip: String,

    /// Transport (tcp or ipc)
    pub transport: String,

    /// Kernel display name
    pub kernel_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterPorts {
    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS for remote connections
    pub tls_enabled: bool,

    /// TLS certificate path
    pub cert_path: Option<PathBuf>,

    /// TLS key path
    pub key_path: Option<PathBuf>,

    /// Authentication method (none, token, oauth2)
    pub auth_method: AuthMethod,

    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,

    /// IP whitelist
    pub ip_whitelist: Vec<String>,

    /// Enable audit logging
    pub audit_log: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum concurrent clients
    pub max_clients: usize,

    /// Maximum memory usage (MB)
    pub max_memory_mb: usize,

    /// Maximum CPU percentage
    pub max_cpu_percent: f32,

    /// Session timeout (seconds)
    pub session_timeout_secs: u64,

    /// Maximum message size (bytes)
    pub max_message_size: usize,
}
```

}

impl KernelService {
    pub async fn new(config: KernelServiceConfig) -> Result<Self> {
        // Create kernel instance
        let kernel = Arc::new(IntegratedKernel::new(config.kernel_config).await?);

        // Setup daemon if requested
        let daemon = if config.daemon {
            let dm = DaemonManager::new(
                config.pid_file.clone(),
                config.log_file.clone(),
                config.foreground,
            );
            dm.daemonize()?;
            Some(dm)
        } else {
            None
        };

        // Setup signal handling
        let (control_tx, control_rx) = mpsc::channel(100);
        let signal_bridge = if !config.foreground {
            Some(SignalBridge::new(control_tx)?)
        } else {
            None
        };

        // Create client registry
        let clients = Arc::new(ClientRegistry::new());

        Ok(Self {
            kernel,
            daemon,
            signal_bridge,
            servers: DashMap::new(),
            clients,
            config,
            connection_file: PathBuf::new(),
        })
    }

    /// Start all configured protocol servers
    pub async fn start(&mut self) -> Result<()> {
        // Start signal handler if configured
        if let Some(mut bridge) = self.signal_bridge.take() {
            tokio::spawn(async move {
                if let Err(e) = bridge.run().await {
                    error!("Signal handler error: {}", e);
                }
            });
        }

        // Start Jupyter server if enabled
        if self.config.jupyter_enabled {
            self.start_jupyter_server().await?;
        }

        // Start DAP server if enabled
        if self.config.dap_enabled {
            self.start_dap_server().await?;
        }

        // Start LSP server if enabled
        if self.config.lsp_enabled {
            self.start_lsp_server().await?;
        }

        // Start REPL service if enabled
        if self.config.repl_enabled {
            self.start_repl_service().await?;
        }

        // Write connection file for discovery
        self.write_connection_file()?;

        info!("Kernel service started with PID {}", process::id());
        info!("Connection file: {}", self.connection_file.display());

        // Keep running until shutdown
        self.run_event_loop().await
    }

    /// Write Jupyter connection file for client discovery
    fn write_connection_file(&mut self) -> Result<()> {
        let connection_info = json!({
            "kernel_id": self.kernel.id(),
            "pid": process::id(),
            "transport": "tcp",
            "ip": self.config.bind_ip,
            "shell_port": self.config.shell_port,
            "iopub_port": self.config.iopub_port,
            "stdin_port": self.config.stdin_port,
            "control_port": self.config.control_port,
            "hb_port": self.config.hb_port,
            "key": self.config.key,
            "signature_scheme": "hmac-sha256",
            "kernel_name": "llmspell",
        });

        let path = self.config.runtime_dir.join(
            format!("kernel-{}.json", self.kernel.id())
        );

        std::fs::write(&path, serde_json::to_string_pretty(&connection_info)?)?;
        self.connection_file = path;

        Ok(())
    }
}
```

### 3.2 CLI Integration - kernel start Command

Enhance the existing `kernel start` command with daemon and protocol options:

```rust
// llmspell-cli/src/cli.rs - Enhanced KernelCommands
#[derive(Subcommand, Debug)]
pub enum KernelCommands {
    /// Start kernel server with optional daemon mode
    #[command(long_about = "Start a kernel server for multi-client execution.

EXAMPLES:
    llmspell kernel start                        # Start in foreground
    llmspell kernel start --daemon               # Start as background daemon
    llmspell kernel start --daemon --port 9555   # Daemon on specific port
    llmspell kernel start --daemon --all         # All protocols enabled
    llmspell kernel start --daemon --jupyter     # Only Jupyter protocol
    llmspell kernel start --daemon --foreground  # Debug daemon behavior")]
    Start {
        /// Port to listen on (shell port, others are +1, +2, etc.)
        #[arg(short, long, default_value = "9555")]
        port: u16,

        /// Run as daemon (background process)
        #[arg(long)]
        daemon: bool,

        /// Stay in foreground (for debugging daemon mode)
        #[arg(long, conflicts_with = "daemon")]
        foreground: bool,

        /// Kernel ID (generated if not provided)
        #[arg(short = 'i', long)]
        id: Option<String>,

        /// Connection file path (for Jupyter discovery)
        #[arg(long)]
        connection_file: Option<PathBuf>,

        /// PID file location (default: /var/run/llmspell.pid)
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Log file for daemon output (default: /var/log/llmspell.log)
        #[arg(long)]
        log_file: Option<PathBuf>,

        /// Enable Jupyter protocol (default: true)
        #[arg(long)]
        jupyter: bool,

        /// Enable Debug Adapter Protocol
        #[arg(long)]
        dap: bool,

        /// Enable Language Server Protocol
        #[arg(long)]
        lsp: bool,

        /// Enable REPL service
        #[arg(long)]
        repl: bool,

        /// Enable all protocols
        #[arg(long)]
        all: bool,

        /// Bind IP address
        #[arg(long, default_value = "127.0.0.1")]
        ip: String,

        /// Enable TLS for remote connections
        #[arg(long)]
        tls: bool,
    },

    // ... other kernel subcommands ...
}
```

---

## 4. Logging Infrastructure for Daemon Mode

### 4.1 Rotating Log System

Daemon mode requires proper logging with rotation:

```rust
// llmspell-kernel/src/logging.rs
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use std::fs::OpenOptions;

/// Logging configuration for daemon mode
pub struct DaemonLogger {
    log_dir: PathBuf,
    log_file: String,
    max_size: u64,
    max_files: usize,
    syslog_enabled: bool,
}

impl DaemonLogger {
    pub fn init(&self) -> Result<()> {
        // Create rolling file appender
        let file_appender = rolling::daily(&self.log_dir, &self.log_file);
        let (non_blocking_appender, _guard) = non_blocking(file_appender);

        // Build subscriber with file and optional syslog
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(
                fmt::layer()
                    .with_writer(non_blocking_appender)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
            );

        // Add syslog layer if enabled
        #[cfg(unix)]
        if self.syslog_enabled {
            use tracing_syslog::Syslog;
            let syslog = Syslog::new(
                "llmspell",
                syslog::Facility::LOG_DAEMON,
                None,
            )?;
            subscriber = subscriber.with(syslog);
        }

        // Set as global subscriber
        tracing::subscriber::set_global_default(subscriber)?;

        // Redirect panic messages to log
        std::panic::set_hook(Box::new(|panic_info| {
            error!("Panic occurred: {}", panic_info);
        }));

        Ok(())
    }

    /// Rotate logs based on size
    pub fn rotate_if_needed(&self) -> Result<()> {
        let log_path = self.log_dir.join(&self.log_file);

        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path)?;

            if metadata.len() > self.max_size {
                // Rotate log files
                for i in (1..self.max_files).rev() {
                    let old = self.log_dir.join(format!("{}.{}", self.log_file, i));
                    let new = self.log_dir.join(format!("{}.{}", self.log_file, i + 1));

                    if old.exists() {
                        if i == self.max_files - 1 {
                            std::fs::remove_file(&old)?;
                        } else {
                            std::fs::rename(&old, &new)?;
                        }
                    }
                }

                // Rotate current to .1
                let rotated = self.log_dir.join(format!("{}.1", self.log_file));
                std::fs::rename(&log_path, &rotated)?;
            }
        }

        Ok(())
    }
}
```

### 4.2 Structured Logging for Protocols

Protocol-specific logging with correlation:

```rust
// llmspell-kernel/src/logging/protocol.rs
use tracing::{info_span, Instrument};
use uuid::Uuid;

/// Protocol-aware logging context
pub struct ProtocolLogger {
    protocol: String,
    session_id: String,
    client_id: Option<String>,
}

impl ProtocolLogger {
    /// Create span for protocol operation
    pub fn span(&self, operation: &str) -> tracing::Span {
        info_span!(
            "protocol",
            protocol = %self.protocol,
            session = %self.session_id,
            client = ?self.client_id,
            operation = %operation,
            request_id = %Uuid::new_v4(),
        )
    }

    /// Log message with protocol context
    pub fn log_message(&self, msg_type: &str, direction: &str) {
        info!(
            protocol = %self.protocol,
            msg_type = %msg_type,
            direction = %direction,
            "Protocol message"
        );
    }
}

/// Log Jupyter message
pub fn log_jupyter_message(msg: &JupyterMessage, direction: &str) {
    info!(
        msg_type = %msg.header.msg_type,
        msg_id = %msg.header.msg_id,
        session = %msg.header.session,
        direction = %direction,
        "Jupyter message"
    );
}

/// Log DAP request/response
pub fn log_dap_message(command: &str, seq: u32, success: bool) {
    info!(
        command = %command,
        seq = seq,
        success = success,
        "DAP message"
    );
}
```

## 5. Jupyter Lab Integration

### 5.1 ZeroMQ Transport Implementation

Complete 5-channel ZeroMQ implementation for Jupyter protocol:

```rust
// llmspell-kernel/src/jupyter/transport.rs
use zmq::{Context, Socket, SocketType};
use llmspell_kernel::JupyterProtocol;
use serde_json::Value;
use std::sync::Arc;

/// ZeroMQ-based Jupyter transport
pub struct ZmqJupyterTransport {
    /// ZMQ context
    context: Context,

    /// Shell channel (REQ-REP)
    shell: Socket,

    /// IOPub channel (PUB)
    iopub: Socket,

    /// StdIn channel (REQ-REP)
    stdin: Socket,

    /// Control channel (REQ-REP)
    control: Socket,

    /// Heartbeat channel (REQ-REP)
    heartbeat: Socket,

    /// Message signing key
    key: String,

    /// Session ID
    session_id: String,
}

impl ZmqJupyterTransport {
    pub fn new(config: &JupyterConfig) -> Result<Self> {
        let context = Context::new();

        // Create and bind shell socket
        let shell = context.socket(SocketType::ROUTER)?;
        shell.bind(&format!("{}://{}:{}",
            config.transport, config.ip, config.ports.shell_port))?;

        // Create and bind iopub socket
        let iopub = context.socket(SocketType::PUB)?;
        iopub.bind(&format!("{}://{}:{}",
            config.transport, config.ip, config.ports.iopub_port))?;

        // Create and bind stdin socket
        let stdin = context.socket(SocketType::ROUTER)?;
        stdin.bind(&format!("{}://{}:{}",
            config.transport, config.ip, config.ports.stdin_port))?;

        // Create and bind control socket
        let control = context.socket(SocketType::ROUTER)?;
        control.bind(&format!("{}://{}:{}",
            config.transport, config.ip, config.ports.control_port))?;

        // Create and bind heartbeat socket
        let heartbeat = context.socket(SocketType::REP)?;
        heartbeat.bind(&format!("{}://{}:{}",
            config.transport, config.ip, config.ports.hb_port))?;

        Ok(Self {
            context,
            shell,
            iopub,
            stdin,
            control,
            heartbeat,
            key: config.key.clone(),
            session_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Receive a message from specified channel
    pub fn recv_message(&self, channel: Channel) -> Result<JupyterMessage> {
        let socket = self.get_socket(channel);

        // Read multipart message
        let mut parts = Vec::new();
        loop {
            let part = socket.recv_bytes(0)?;
            let more = socket.get_rcvmore()?;
            parts.push(part);
            if !more {
                break;
            }
        }

        // Parse Jupyter wire protocol
        self.parse_wire_message(parts)
    }

    /// Send a message to specified channel
    pub fn send_message(&self, channel: Channel, msg: JupyterMessage) -> Result<()> {
        let socket = self.get_socket(channel);

        // Serialize to wire format
        let parts = self.serialize_wire_message(msg)?;

        // Send multipart message
        for (i, part) in parts.iter().enumerate() {
            let flags = if i < parts.len() - 1 { zmq::SNDMORE } else { 0 };
            socket.send(part, flags)?;
        }

        Ok(())
    }

    /// Parse wire protocol message
    fn parse_wire_message(&self, parts: Vec<Vec<u8>>) -> Result<JupyterMessage> {
        // Jupyter wire format:
        // [<IDS>b'<IDS>', HMAC, header, parent_header, metadata, content, buffers...]

        // Find delimiter
        let delim_idx = parts.iter()
            .position(|p| p == b"<IDS>")
            .ok_or_else(|| anyhow!("Invalid message format"))?;

        // Extract components
        let identities = &parts[..delim_idx];
        let signature = &parts[delim_idx + 1];
        let header = serde_json::from_slice(&parts[delim_idx + 2])?;
        let parent_header = serde_json::from_slice(&parts[delim_idx + 3])?;
        let metadata = serde_json::from_slice(&parts[delim_idx + 4])?;
        let content = serde_json::from_slice(&parts[delim_idx + 5])?;
        let buffers = parts[delim_idx + 6..].to_vec();

        // Verify HMAC signature
        self.verify_signature(signature, &parts[delim_idx + 2..])?;

        Ok(JupyterMessage {
            identities: identities.to_vec(),
            header,
            parent_header,
            metadata,
            content,
            buffers,
        })
    }

    /// Start heartbeat loop
    pub async fn heartbeat_loop(&self) -> Result<()> {
        loop {
            // Receive ping
            let ping = self.heartbeat.recv_bytes(0)?;

            // Send pong (echo back)
            self.heartbeat.send(&ping, 0)?;

            // Small yield to prevent busy loop
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
}
```

### 2.2 Jupyter Server Implementation

The Jupyter kernel server that handles protocol messages:

```rust
// llmspell-service/src/jupyter/server.rs
use llmspell_kernel::{IntegratedKernel, ExecutionConfig};
use tokio::task::JoinHandle;

pub struct JupyterServer {
    kernel: Arc<IntegratedKernel>,
    transport: Arc<ZmqJupyterTransport>,
    clients: Arc<ClientRegistry>,
    config: JupyterConfig,
    event_bus: Arc<EventBus>,
    shutdown: Arc<AtomicBool>,
    execution_count: Arc<AtomicUsize>,
}

impl JupyterServer {
    pub async fn new(
        kernel: Arc<IntegratedKernel>,
        clients: Arc<ClientRegistry>,
        config: JupyterConfig,
        event_bus: Arc<EventBus>,
    ) -> Result<Self> {
        let transport = Arc::new(ZmqJupyterTransport::new(&config)?);

        Ok(Self {
            kernel,
            transport,
            clients,
            config,
            event_bus,
            shutdown: Arc::new(AtomicBool::new(false)),
            execution_count: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Start the Jupyter server
    pub async fn start(&self) -> Result<Vec<JoinHandle<()>>> {
        let mut handles = Vec::new();

        // Start shell handler
        handles.push(self.start_shell_handler().await?);

        // Start control handler
        handles.push(self.start_control_handler().await?);

        // Start stdin handler
        handles.push(self.start_stdin_handler().await?);

        // Start heartbeat loop
        handles.push(self.start_heartbeat_loop().await?);

        // Start IOPub publisher
        handles.push(self.start_iopub_publisher().await?);

        Ok(handles)
    }

    /// Handle shell channel messages
    async fn start_shell_handler(&self) -> Result<JoinHandle<()>> {
        let transport = self.transport.clone();
        let kernel = self.kernel.clone();
        let execution_count = self.execution_count.clone();
        let shutdown = self.shutdown.clone();

        let handle = tokio::spawn(async move {
            while !shutdown.load(Ordering::Relaxed) {
                // Receive request
                let msg = match transport.recv_message(Channel::Shell).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Error receiving shell message: {}", e);
                        continue;
                    }
                };

                // Route by message type
                let response = match msg.header.msg_type.as_str() {
                    "execute_request" => {
                        Self::handle_execute_request(
                            kernel.clone(),
                            msg,
                            execution_count.clone(),
                        ).await
                    },
                    "inspect_request" => {
                        Self::handle_inspect_request(kernel.clone(), msg).await
                    },
                    "complete_request" => {
                        Self::handle_complete_request(kernel.clone(), msg).await
                    },
                    "history_request" => {
                        Self::handle_history_request(kernel.clone(), msg).await
                    },
                    "kernel_info_request" => {
                        Self::handle_kernel_info_request(msg).await
                    },
                    "comm_open" => {
                        Self::handle_comm_open(kernel.clone(), msg).await
                    },
                    _ => {
                        warn!("Unknown message type: {}", msg.header.msg_type);
                        continue;
                    }
                };

                // Send response
                if let Err(e) = transport.send_message(Channel::Shell, response).await {
                    error!("Error sending shell response: {}", e);
                }
            }
        });

        Ok(handle)
    }

    /// Handle execute_request
    async fn handle_execute_request(
        kernel: Arc<IntegratedKernel>,
        msg: JupyterMessage,
        execution_count: Arc<AtomicUsize>,
    ) -> JupyterMessage {
        let code = msg.content["code"].as_str().unwrap_or("");
        let silent = msg.content["silent"].as_bool().unwrap_or(false);

        // Update execution count
        let count = if !silent {
            execution_count.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            execution_count.load(Ordering::SeqCst)
        };

        // Create execution config
        let config = ExecutionConfig {
            code: code.to_string(),
            silent,
            store_history: !silent,
            allow_stdin: msg.content["allow_stdin"].as_bool().unwrap_or(true),
            stop_on_error: msg.content["stop_on_error"].as_bool().unwrap_or(false),
        };

        // Execute code
        let result = kernel.execute(config).await;

        // Build reply
        let (status, content) = match result {
            Ok(output) => {
                ("ok", json!({
                    "status": "ok",
                    "execution_count": count,
                    "payload": [],
                    "user_expressions": {}
                }))
            },
            Err(e) => {
                ("error", json!({
                    "status": "error",
                    "ename": "ExecutionError",
                    "evalue": e.to_string(),
                    "traceback": vec![e.to_string()]
                }))
            }
        };

        JupyterMessage::reply(msg, "execute_reply", content)
    }

    /// Write connection file for Jupyter discovery
    pub fn write_connection_file(&self, path: &Path) -> Result<()> {
        let connection_info = json!({
            "shell_port": self.config.ports.shell_port,
            "iopub_port": self.config.ports.iopub_port,
            "stdin_port": self.config.ports.stdin_port,
            "control_port": self.config.ports.control_port,
            "hb_port": self.config.ports.hb_port,
            "ip": self.config.ip,
            "key": self.config.key,
            "transport": self.config.transport,
            "signature_scheme": "hmac-sha256",
            "kernel_name": self.config.kernel_name,
        });

        std::fs::write(path, serde_json::to_string_pretty(&connection_info)?)?;
        Ok(())
    }
}
```

---

## 3. VS Code Integration (DAP)

### 3.1 Debug Adapter Protocol Server

DAP server implementation for VS Code debugging:

```rust
// llmspell-service/src/dap/server.rs
use llmspell_kernel::{DAPBridge, DebugSession};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

pub struct DAPServer {
    kernel: Arc<IntegratedKernel>,
    bridge: Arc<DAPBridge>,
    config: DAPConfig,
    listener: Option<TcpListener>,
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
}

impl DAPServer {
    pub async fn new(
        kernel: Arc<IntegratedKernel>,
        config: DAPConfig,
    ) -> Result<Self> {
        let bridge = Arc::new(DAPBridge::new(kernel.clone()));

        Ok(Self {
            kernel,
            bridge,
            config,
            listener: None,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the DAP server
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        self.listener = Some(TcpListener::bind(&addr).await?);

        info!("DAP server listening on {}", addr);

        // Accept connections
        self.accept_loop().await
    }

    /// Accept and handle client connections
    async fn accept_loop(&self) -> Result<()> {
        let listener = self.listener.as_ref().unwrap();

        loop {
            let (stream, addr) = listener.accept().await?;
            info!("DAP client connected from {}", addr);

            // Spawn handler for this client
            let bridge = self.bridge.clone();
            let sessions = self.sessions.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, bridge, sessions).await {
                    error!("DAP client error: {}", e);
                }
            });
        }
    }

    /// Handle a single DAP client
    async fn handle_client(
        stream: TcpStream,
        bridge: Arc<DAPBridge>,
        sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    ) -> Result<()> {
        let mut framed = Framed::new(stream, DAPCodec::new());

        // Session state
        let mut initialized = false;
        let mut session_id = String::new();

        while let Some(message) = framed.next().await {
            let message = message?;

            // Parse DAP message
            let request: DAPRequest = serde_json::from_str(&message)?;

            // Handle based on command
            let response = match request.command.as_str() {
                "initialize" => {
                    initialized = true;
                    session_id = uuid::Uuid::new_v4().to_string();
                    Self::handle_initialize(&bridge, request).await?
                },
                "launch" if initialized => {
                    Self::handle_launch(&bridge, &session_id, request).await?
                },
                "attach" if initialized => {
                    Self::handle_attach(&bridge, &session_id, request).await?
                },
                "setBreakpoints" => {
                    Self::handle_set_breakpoints(&bridge, request).await?
                },
                "stackTrace" => {
                    Self::handle_stack_trace(&bridge, request).await?
                },
                "scopes" => {
                    Self::handle_scopes(&bridge, request).await?
                },
                "variables" => {
                    Self::handle_variables(&bridge, request).await?
                },
                "continue" => {
                    Self::handle_continue(&bridge, request).await?
                },
                "next" => {
                    Self::handle_next(&bridge, request).await?
                },
                "stepIn" => {
                    Self::handle_step_in(&bridge, request).await?
                },
                "stepOut" => {
                    Self::handle_step_out(&bridge, request).await?
                },
                "pause" => {
                    Self::handle_pause(&bridge, request).await?
                },
                "evaluate" => {
                    Self::handle_evaluate(&bridge, request).await?
                },
                "disconnect" => {
                    Self::handle_disconnect(&bridge, &session_id, sessions.clone()).await?
                },
                _ => {
                    Self::error_response(request.seq, "Unknown command")
                }
            };

            // Send response
            let response_str = serde_json::to_string(&response)?;
            framed.send(response_str).await?;
        }

        Ok(())
    }

    /// Handle initialize request
    async fn handle_initialize(
        bridge: &Arc<DAPBridge>,
        request: DAPRequest,
    ) -> Result<DAPResponse> {
        let capabilities = bridge.get_capabilities();

        Ok(DAPResponse {
            seq: 0,
            request_seq: request.seq,
            success: true,
            command: request.command,
            body: Some(json!({
                "supportsConfigurationDoneRequest": true,
                "supportsFunctionBreakpoints": true,
                "supportsConditionalBreakpoints": true,
                "supportsHitConditionalBreakpoints": true,
                "supportsEvaluateForHovers": true,
                "supportsStepBack": false,
                "supportsSetVariable": true,
                "supportsRestartFrame": false,
                "supportsStepInTargetsRequest": false,
                "supportsGotoTargetsRequest": false,
                "supportsCompletionsRequest": true,
                "supportsModulesRequest": false,
                "supportsExceptionOptions": true,
                "supportsValueFormattingOptions": true,
                "supportsExceptionInfoRequest": true,
                "supportTerminateDebuggee": true,
                "supportsDelayedStackTraceLoading": true,
                "supportsLoadedSourcesRequest": true,
                "supportsLogPoints": false,
                "supportsTerminateThreadsRequest": false,
                "supportsSetExpression": false,
                "supportsDataBreakpoints": false,
                "supportsReadMemoryRequest": false,
                "supportsDisassembleRequest": false,
                "supportsCancelRequest": false,
                "supportsBreakpointLocationsRequest": true,
            })),
            message: None,
        })
    }
}
```

### 3.2 VS Code Extension Integration

Configuration for VS Code to connect to our DAP server:

```json
// .vscode/launch.json example
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "llmspell",
            "request": "launch",
            "name": "Debug Lua Script",
            "program": "${workspaceFolder}/script.lua",
            "stopOnEntry": false,
            "debugServer": 8889,
            "cwd": "${workspaceFolder}",
            "env": {},
            "args": []
        },
        {
            "type": "llmspell",
            "request": "attach",
            "name": "Attach to LLMSpell Kernel",
            "debugServer": 8889,
            "kernelId": "auto"
        }
    ]
}
```

---

## 4. Language Server Protocol (LSP)

### 4.1 LSP Server Implementation

LSP server providing code intelligence for llmspell scripts:

```rust
// llmspell-service/src/lsp/server.rs
use llmspell_kernel::IntegratedKernel;
use tower_lsp::{LspService, Server};
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;

pub struct LSPServer {
    kernel: Arc<IntegratedKernel>,
    config: LSPConfig,
    document_store: Arc<RwLock<DocumentStore>>,
    symbol_index: Arc<RwLock<SymbolIndex>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for LSPServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::Incremental),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        inter_file_dependencies: true,
                        workspace_diagnostics: true,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("LSP server initialized");
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get symbol at position from kernel state
        let symbol = self.kernel.get_symbol_at(&uri, position).await?;

        if let Some(sym) = symbol {
            let hover = Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**{}**\n\nType: `{}`\n\n{}",
                        sym.name,
                        sym.type_info,
                        sym.documentation
                    ),
                }),
                range: Some(sym.range),
            };
            Ok(Some(hover))
        } else {
            Ok(None)
        }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get completions from kernel runtime context
        let completions = self.kernel.get_completions(&uri, position).await?;

        let items: Vec<CompletionItem> = completions
            .into_iter()
            .map(|c| CompletionItem {
                label: c.label,
                kind: Some(match c.kind {
                    CompletionKind::Function => CompletionItemKind::Function,
                    CompletionKind::Variable => CompletionItemKind::Variable,
                    CompletionKind::Module => CompletionItemKind::Module,
                    CompletionKind::Keyword => CompletionItemKind::Keyword,
                    _ => CompletionItemKind::Text,
                }),
                detail: c.detail,
                documentation: c.documentation.map(|d| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: d,
                    })
                }),
                insert_text: Some(c.insert_text),
                insert_text_format: Some(InsertTextFormat::Snippet),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Find definition using kernel's symbol index
        let definition = self.kernel.find_definition(&uri, position).await?;

        if let Some(def) = definition {
            Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: def.uri,
                range: def.range,
            })))
        } else {
            Ok(None)
        }
    }

    async fn diagnostics(
        &self,
        params: DocumentDiagnosticParams,
    ) -> LspResult<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;

        // Get diagnostics from kernel execution
        let diags = self.kernel.get_diagnostics(&uri).await?;

        let items: Vec<Diagnostic> = diags
            .into_iter()
            .map(|d| Diagnostic {
                range: d.range,
                severity: Some(match d.severity {
                    DiagSeverity::Error => DiagnosticSeverity::ERROR,
                    DiagSeverity::Warning => DiagnosticSeverity::WARNING,
                    DiagSeverity::Info => DiagnosticSeverity::INFORMATION,
                    DiagSeverity::Hint => DiagnosticSeverity::HINT,
                }),
                code: d.code.map(|c| NumberOrString::String(c)),
                source: Some("llmspell".to_string()),
                message: d.message,
                related_information: None,
                tags: None,
                data: None,
                code_description: None,
            })
            .collect();

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items,
                    },
                },
            ),
        ))
    }
}
```

---

## 5. Interactive REPL Service

### 5.1 REPL Service Implementation

Enhanced REPL as a service with multi-client support:

```rust
// llmspell-service/src/repl/service.rs
use llmspell_kernel::{IntegratedKernel, REPLSession};
use tokio::net::{TcpListener, UnixListener};

pub struct REPLService {
    kernel: Arc<IntegratedKernel>,
    config: REPLConfig,
    sessions: Arc<RwLock<HashMap<String, REPLSession>>>,
}

impl REPLService {
    pub async fn new(
        kernel: Arc<IntegratedKernel>,
        config: REPLConfig,
    ) -> Result<Self> {
        Ok(Self {
            kernel,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start REPL service
    pub async fn start(&self) -> Result<()> {
        match &self.config.transport {
            REPLTransport::TCP { host, port } => {
                self.start_tcp_service(host, *port).await
            },
            REPLTransport::Unix { path } => {
                self.start_unix_service(path).await
            },
            REPLTransport::WebSocket { host, port } => {
                self.start_websocket_service(host, *port).await
            },
        }
    }

    /// Start TCP REPL service
    async fn start_tcp_service(&self, host: &str, port: u16) -> Result<()> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;

        info!("REPL service listening on {}", addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            info!("REPL client connected from {}", addr);

            let kernel = self.kernel.clone();
            let sessions = self.sessions.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_tcp_client(stream, kernel, sessions).await {
                    error!("REPL client error: {}", e);
                }
            });
        }
    }

    /// Handle REPL client session
    async fn handle_tcp_client(
        mut stream: TcpStream,
        kernel: Arc<IntegratedKernel>,
        sessions: Arc<RwLock<HashMap<String, REPLSession>>>,
    ) -> Result<()> {
        // Create session
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = REPLSession::new(session_id.clone(), kernel.clone());
        sessions.write().await.insert(session_id.clone(), session);

        // Send welcome message
        let welcome = format!(
            "LLMSpell REPL v{}\nSession ID: {}\nType .help for commands\n\n> ",
            env!("CARGO_PKG_VERSION"),
            session_id
        );
        stream.write_all(welcome.as_bytes()).await?;

        // REPL loop
        let mut buffer = String::new();
        let mut reader = BufReader::new(&mut stream);

        loop {
            buffer.clear();

            // Read line
            match reader.read_line(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(_) => {},
                Err(e) => {
                    error!("Read error: {}", e);
                    break;
                }
            }

            let input = buffer.trim();

            // Handle special commands
            if input.starts_with('.') {
                let output = self.handle_repl_command(&session_id, input).await?;
                stream.write_all(output.as_bytes()).await?;
                stream.write_all(b"\n> ").await?;
                continue;
            }

            // Execute code
            let result = {
                let sessions = sessions.read().await;
                let session = sessions.get(&session_id).unwrap();
                session.execute(input).await
            };

            // Send output
            match result {
                Ok(output) => {
                    if !output.is_empty() {
                        stream.write_all(output.as_bytes()).await?;
                        stream.write_all(b"\n").await?;
                    }
                },
                Err(e) => {
                    let error = format!("Error: {}\n", e);
                    stream.write_all(error.as_bytes()).await?;
                }
            }

            // Send prompt
            stream.write_all(b"> ").await?;
            stream.flush().await?;
        }

        // Cleanup session
        sessions.write().await.remove(&session_id);

        Ok(())
    }

    /// Handle REPL meta-commands
    async fn handle_repl_command(&self, session_id: &str, cmd: &str) -> Result<String> {
        match cmd {
            ".help" => Ok(Self::help_text()),
            ".exit" | ".quit" => Err(anyhow!("exit")),
            ".clear" => {
                // Clear session state
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(session_id) {
                    session.clear().await?;
                }
                Ok("Session cleared".to_string())
            },
            ".save" => {
                // Save session history
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(session_id) {
                    let path = session.save_history().await?;
                    Ok(format!("History saved to {}", path.display()))
                } else {
                    Ok("Session not found".to_string())
                }
            },
            ".load" => {
                // Load session history
                Ok("Load not yet implemented".to_string())
            },
            ".vars" | ".locals" => {
                // Show local variables
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(session_id) {
                    let vars = session.get_variables().await?;
                    Ok(format!("Variables:\n{}", vars))
                } else {
                    Ok("Session not found".to_string())
                }
            },
            cmd if cmd.starts_with(".watch") => {
                // Add watch expression
                let expr = cmd.strip_prefix(".watch").unwrap().trim();
                Ok(format!("Watching: {}", expr))
            },
            _ => Ok(format!("Unknown command: {}", cmd)),
        }
    }
}
```

---

## 6. Service Deployment & Management

### 6.1 Systemd Service Unit

Systemd service configuration for Linux:

```ini
# /etc/systemd/system/llmspell.service
[Unit]
Description=LLMSpell Multi-Protocol Service
After=network.target

[Service]
Type=notify
ExecStart=/usr/local/bin/llmspell serve --config /etc/llmspell/service.toml --all
ExecReload=/bin/kill -HUP $MAINPID
KillMode=mixed
KillSignal=SIGTERM
Restart=on-failure
RestartSec=5s
User=llmspell
Group=llmspell

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/llmspell /var/log/llmspell

# Resource limits
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
CPUQuota=200%

# Environment
Environment="LLMSPELL_LOG_LEVEL=info"
Environment="LLMSPELL_CONFIG_DIR=/etc/llmspell"

[Install]
WantedBy=multi-user.target
```

### 6.2 Launchd Service Plist (macOS)

Launchd configuration for macOS:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.llmspell.service</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/llmspell</string>
        <string>serve</string>
        <string>--config</string>
        <string>/usr/local/etc/llmspell/service.toml</string>
        <string>--all</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>

    <key>StandardOutPath</key>
    <string>/usr/local/var/log/llmspell/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/llmspell/stderr.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>LLMSPELL_LOG_LEVEL</key>
        <string>info</string>
    </dict>

    <key>SoftResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
    </dict>

    <key>HardResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
    </dict>
</dict>
</plist>
```

### 6.3 Health Monitoring

Health check and monitoring implementation:

```rust
// llmspell-service/src/health.rs
use std::time::{Duration, Instant};

pub struct HealthMonitor {
    checks: Vec<Box<dyn HealthCheck>>,
    status: Arc<RwLock<HealthStatus>>,
    config: HealthConfig,
}

impl HealthMonitor {
    pub async fn start_monitoring(
        &self,
        servers: DashMap<String, Box<dyn ProtocolServer>>,
    ) {
        let check_interval = Duration::from_secs(self.config.check_interval_secs);

        loop {
            tokio::time::sleep(check_interval).await;

            let mut status = HealthStatus {
                healthy: true,
                timestamp: Instant::now(),
                checks: HashMap::new(),
            };

            // Check each protocol server
            for server in servers.iter() {
                let name = server.key().clone();
                let result = server.value().health_check().await;

                status.checks.insert(name.clone(), result.clone());
                if !result.healthy {
                    status.healthy = false;
                    warn!("Health check failed for {}: {}", name, result.message);
                }
            }

            // Check system resources
            let mem_result = self.check_memory().await;
            status.checks.insert("memory".to_string(), mem_result.clone());
            if !mem_result.healthy {
                status.healthy = false;
            }

            let cpu_result = self.check_cpu().await;
            status.checks.insert("cpu".to_string(), cpu_result.clone());
            if !cpu_result.healthy {
                status.healthy = false;
            }

            // Update status
            *self.status.write().await = status;

            // Trigger alerts if unhealthy
            if !status.healthy && self.config.enable_alerts {
                self.send_alert(&status).await;
            }
        }
    }

    /// HTTP health endpoint handler
    pub async fn health_endpoint(&self) -> impl warp::Reply {
        let status = self.status.read().await;

        let code = if status.healthy {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };

        warp::reply::with_status(
            warp::reply::json(&*status),
            code,
        )
    }
}
```

---

## 7. Multi-Client Architecture & Security

### 7.1 Client Registry and Session Management

Managing multiple concurrent client connections:

```rust
// llmspell-service/src/clients.rs
use std::net::SocketAddr;
use uuid::Uuid;

pub struct ClientRegistry {
    clients: Arc<DashMap<ClientId, ClientInfo>>,
    sessions: Arc<DashMap<SessionId, ClientSession>>,
    limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: ClientId,
    pub protocol: Protocol,
    pub address: SocketAddr,
    pub connected_at: Instant,
    pub last_activity: Arc<RwLock<Instant>>,
    pub session_id: Option<SessionId>,
    pub authenticated: bool,
    pub permissions: Permissions,
}

#[derive(Debug, Clone)]
pub struct ClientSession {
    pub id: SessionId,
    pub client_ids: Vec<ClientId>,
    pub kernel_context: KernelContext,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
    pub state: Arc<RwLock<SessionState>>,
}

impl ClientRegistry {
    /// Register a new client connection
    pub async fn register_client(
        &self,
        protocol: Protocol,
        address: SocketAddr,
    ) -> Result<ClientId> {
        // Check connection limits
        if self.clients.len() >= self.limits.max_clients {
            return Err(anyhow!("Maximum client limit reached"));
        }

        let client_id = ClientId(Uuid::new_v4());
        let client_info = ClientInfo {
            id: client_id.clone(),
            protocol,
            address,
            connected_at: Instant::now(),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            session_id: None,
            authenticated: false,
            permissions: Permissions::default(),
        };

        self.clients.insert(client_id.clone(), client_info);

        info!("Client {} registered from {}", client_id, address);
        Ok(client_id)
    }

    /// Authenticate a client
    pub async fn authenticate_client(
        &self,
        client_id: &ClientId,
        credentials: Credentials,
    ) -> Result<()> {
        // Validate credentials
        let permissions = self.validate_credentials(credentials).await?;

        // Update client info
        if let Some(mut client) = self.clients.get_mut(client_id) {
            client.authenticated = true;
            client.permissions = permissions;
        } else {
            return Err(anyhow!("Client not found"));
        }

        Ok(())
    }

    /// Create or join a session
    pub async fn create_or_join_session(
        &self,
        client_id: &ClientId,
        session_id: Option<SessionId>,
    ) -> Result<SessionId> {
        let sid = session_id.unwrap_or_else(|| SessionId(Uuid::new_v4()));

        // Get or create session
        let session = self.sessions.entry(sid.clone()).or_insert_with(|| {
            ClientSession {
                id: sid.clone(),
                client_ids: Vec::new(),
                kernel_context: KernelContext::new(),
                created_at: Instant::now(),
                expires_at: Some(Instant::now() + Duration::from_secs(self.limits.session_timeout_secs)),
                state: Arc::new(RwLock::new(SessionState::default())),
            }
        });

        // Add client to session
        if !session.client_ids.contains(client_id) {
            session.client_ids.push(client_id.clone());
        }

        // Update client's session
        if let Some(mut client) = self.clients.get_mut(client_id) {
            client.session_id = Some(sid.clone());
        }

        Ok(sid)
    }

    /// Broadcast shutdown to all clients
    pub async fn broadcast_shutdown(&self) {
        for client in self.clients.iter() {
            // Send shutdown message based on protocol
            match client.protocol {
                Protocol::Jupyter => {
                    // Send shutdown_reply
                },
                Protocol::DAP => {
                    // Send terminated event
                },
                Protocol::LSP => {
                    // Send exit notification
                },
                Protocol::REPL => {
                    // Send goodbye message
                },
            }
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let now = Instant::now();
        let expired: Vec<SessionId> = self.sessions
            .iter()
            .filter(|s| {
                s.expires_at.map_or(false, |exp| exp < now)
            })
            .map(|s| s.id.clone())
            .collect();

        for sid in expired {
            self.sessions.remove(&sid);
            info!("Cleaned up expired session {}", sid);
        }
    }
}
```

### 7.2 Security Layer

Security implementation with TLS and authentication:

```rust
// llmspell-service/src/security.rs
use rustls::{ServerConfig, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;

pub struct SecurityLayer {
    config: SecurityConfig,
    tls_acceptor: Option<TlsAcceptor>,
    auth_provider: Box<dyn AuthProvider>,
    audit_logger: Option<AuditLogger>,
}

impl SecurityLayer {
    pub async fn new(config: SecurityConfig) -> Result<Self> {
        // Setup TLS if enabled
        let tls_acceptor = if config.tls_enabled {
            Some(Self::create_tls_acceptor(&config).await?)
        } else {
            None
        };

        // Create auth provider
        let auth_provider: Box<dyn AuthProvider> = match config.auth_method {
            AuthMethod::None => Box::new(NoAuth),
            AuthMethod::Token => Box::new(TokenAuth::new(&config)),
            AuthMethod::OAuth2 => Box::new(OAuth2Auth::new(&config)),
        };

        // Setup audit logger if enabled
        let audit_logger = if config.audit_log {
            Some(AuditLogger::new(&config).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            tls_acceptor,
            auth_provider,
            audit_logger,
        })
    }

    /// Create TLS acceptor
    async fn create_tls_acceptor(config: &SecurityConfig) -> Result<TlsAcceptor> {
        let cert_path = config.cert_path.as_ref()
            .ok_or_else(|| anyhow!("TLS cert path required"))?;
        let key_path = config.key_path.as_ref()
            .ok_or_else(|| anyhow!("TLS key path required"))?;

        // Load certificate
        let cert_file = std::fs::File::open(cert_path)?;
        let mut reader = std::io::BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut reader)?
            .into_iter()
            .map(Certificate)
            .collect::<Vec<_>>();

        // Load private key
        let key_file = std::fs::File::open(key_path)?;
        let mut reader = std::io::BufReader::new(key_file);
        let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
        let key = PrivateKey(keys.into_iter().next()
            .ok_or_else(|| anyhow!("No private key found"))?);

        // Create TLS config
        let tls_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        Ok(TlsAcceptor::from(Arc::new(tls_config)))
    }

    /// Wrap TCP stream with TLS if enabled
    pub async fn wrap_stream<S>(&self, stream: S) -> Result<SecureStream<S>>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        if let Some(acceptor) = &self.tls_acceptor {
            let tls_stream = acceptor.accept(stream).await?;
            Ok(SecureStream::Tls(tls_stream))
        } else {
            Ok(SecureStream::Plain(stream))
        }
    }

    /// Authenticate a connection
    pub async fn authenticate(&self, credentials: Credentials) -> Result<Permissions> {
        let result = self.auth_provider.authenticate(credentials).await?;

        // Log authentication attempt
        if let Some(logger) = &self.audit_logger {
            logger.log_auth(result.clone()).await;
        }

        Ok(result.permissions)
    }

    /// Check if IP is allowed
    pub fn check_ip(&self, addr: &SocketAddr) -> bool {
        if self.config.ip_whitelist.is_empty() {
            true
        } else {
            self.config.ip_whitelist.iter()
                .any(|allowed| addr.ip().to_string() == *allowed)
        }
    }
}
```

---

## 8. Example Applications

### 8.1 Overview

Phase 10's service capabilities enable powerful production applications that leverage daemon mode, signal handling, multi-client support, and IDE integration. These examples demonstrate the evolution from expert-level applications to service-oriented production systems.

### 8.2 Application Architecture Layer

These examples represent **Layer 7: Service & Production Territory**, building upon the existing application progression:

```
Application Progression:
Layer 1-2: Universal (file-organizer, research-collector)
Layer 3: Power User (content-creator)
Layer 4: Business (communication-manager)
Layer 5: Professional (process-orchestrator, code-review)
Layer 6: Expert (webapp-creator)
Layer 7: Service & Production (NEW - Phase 10)
    ├── kernel-fleet-manager/     # Production orchestration
    └── dev-environment-service/  # IDE integration
```

### 8.3 Kernel Fleet Manager

**Purpose**: Production orchestration of multiple kernel instances as services
**Problem Statement**: "Managing multiple AI workloads requires intelligent orchestration"
**Key Features**: Multi-kernel management, load balancing, tenant isolation, health monitoring

```lua
-- examples/script-users/applications/kernel-fleet-manager/main.lua
--[[
Kernel Fleet Manager - Production Orchestration Service
Phase 10 Application showcasing daemon mode and service integration

Problem: Organizations need to manage multiple kernel instances for different
teams, projects, and workloads with proper resource allocation and monitoring.
]]

local config = Config.load("fleet-config.toml")

-- Fleet management agents
local fleet_monitor = Agent.new({
    name = "fleet_monitor",
    model = "gpt-4",
    purpose = "Monitor health and performance of kernel fleet"
})

local load_balancer = Agent.new({
    name = "load_balancer",
    model = "gpt-4",
    purpose = "Distribute workloads across available kernels"
})

local resource_manager = Agent.new({
    name = "resource_manager",
    model = "gpt-4",
    purpose = "Manage CPU, memory, and connection limits"
})

local incident_responder = Agent.new({
    name = "incident_responder",
    model = "gpt-4",
    purpose = "Handle failures and recover services"
})

local metrics_collector = Agent.new({
    name = "metrics_collector",
    model = "gpt-3.5-turbo",
    purpose = "Collect and aggregate performance metrics"
})

-- Initialize fleet state
local fleet_state = State.new({
    persistence = "sled",
    scope = "global"
})

-- Signal handling for graceful operations
local function setup_signal_handlers()
    Hook.register("signal.SIGTERM", function()
        print("🛑 Received SIGTERM - initiating graceful shutdown")

        -- Save fleet state
        fleet_state:set("shutdown_time", os.time())
        fleet_state:set("shutdown_reason", "SIGTERM")

        -- Notify all kernels
        local kernels = fleet_state:get("active_kernels") or {}
        for id, kernel in pairs(kernels) do
            print(string.format("  Shutting down kernel %s...", id))
            -- Send shutdown_request via Jupyter protocol
            kernel:shutdown_gracefully()
        end

        -- Wait for confirmations
        os.execute("sleep 5")
        print("✅ Fleet shutdown complete")
    end)

    Hook.register("signal.SIGUSR1", function()
        print("📊 Received SIGUSR1 - dumping fleet statistics")
        local stats = metrics_collector:execute({
            action = "generate_report",
            kernels = fleet_state:get("active_kernels")
        })
        print(stats.output)
    end)
end

-- Kernel lifecycle management
local function manage_kernel_lifecycle(kernel_id, config)
    local kernel_cmd = string.format(
        "llmspell kernel start --daemon --port %d --log-file %s --pid-file %s --idle-timeout %d",
        config.port,
        config.log_file,
        config.pid_file,
        config.idle_timeout or 3600
    )

    -- Start kernel as daemon
    os.execute(kernel_cmd)

    -- Monitor health
    Hook.register("timer.1m", function()
        local health = fleet_monitor:execute({
            action = "check_health",
            kernel_id = kernel_id,
            pid_file = config.pid_file
        })

        if health.status == "unhealthy" then
            incident_responder:execute({
                action = "recover_kernel",
                kernel_id = kernel_id,
                failure = health.reason
            })
        end
    end)
end

-- Load balancing logic
local function route_request(request)
    local available_kernels = fleet_state:get("available_kernels") or {}

    local selected = load_balancer:execute({
        action = "select_kernel",
        request = request,
        kernels = available_kernels,
        strategy = config.load_balancing.strategy -- "round_robin", "least_loaded", "sticky"
    })

    return selected.kernel_id
end

-- Multi-tenant isolation
local function create_tenant_kernel(tenant_id, resource_limits)
    local tenant_config = {
        port = allocate_port(),
        log_file = string.format("/var/log/llmspell/tenant_%s.log", tenant_id),
        pid_file = string.format("/var/run/llmspell/tenant_%s.pid", tenant_id),
        idle_timeout = resource_limits.idle_timeout or 1800,
        max_memory = resource_limits.max_memory or "1GB",
        max_clients = resource_limits.max_clients or 5
    }

    -- Apply resource constraints
    resource_manager:execute({
        action = "apply_limits",
        tenant_id = tenant_id,
        limits = resource_limits
    })

    -- Start isolated kernel
    manage_kernel_lifecycle(tenant_id, tenant_config)

    -- Track in fleet state
    fleet_state:set(string.format("tenant.%s", tenant_id), {
        config = tenant_config,
        created = os.time(),
        resource_limits = resource_limits
    })
end

-- Production monitoring dashboard
local function start_monitoring_dashboard()
    local dashboard = Workflow.new({
        name = "monitoring_dashboard",
        type = "loop"
    })

    dashboard:add_step("collect_metrics", function()
        return metrics_collector:execute({
            action = "collect_all",
            include = {"cpu", "memory", "connections", "requests", "errors"}
        })
    end)

    dashboard:add_step("analyze_trends", function(metrics)
        return fleet_monitor:execute({
            action = "analyze_trends",
            metrics = metrics,
            window = "5m"
        })
    end)

    dashboard:add_step("generate_alerts", function(analysis)
        if analysis.alerts then
            for _, alert in ipairs(analysis.alerts) do
                incident_responder:execute({
                    action = "handle_alert",
                    alert = alert
                })
            end
        end
    end)

    dashboard:run({interval = "30s"})
end

-- Main fleet manager
local function main()
    print("🚀 Kernel Fleet Manager starting...")

    -- Setup signal handlers
    setup_signal_handlers()

    -- Initialize fleet based on config
    local fleet_config = config.fleet or {}

    for name, kernel_config in pairs(fleet_config.kernels) do
        print(string.format("Starting kernel: %s", name))
        manage_kernel_lifecycle(name, kernel_config)
    end

    -- Start monitoring dashboard
    start_monitoring_dashboard()

    -- Multi-tenant support
    if config.multi_tenant.enabled then
        print("🏢 Multi-tenant mode enabled")

        Hook.register("tenant.create", function(tenant_data)
            create_tenant_kernel(
                tenant_data.id,
                tenant_data.resource_limits
            )
        end)
    end

    -- Service registration
    if config.service_discovery.enabled then
        print("📡 Registering with service discovery...")
        -- Register with Consul, etcd, or Kubernetes
    end

    print("✅ Fleet manager ready")
    print(string.format("   Managing %d kernels", #fleet_config.kernels))
    print(string.format("   Load balancing: %s", config.load_balancing.strategy))
    print(string.format("   Monitoring interval: %s", config.monitoring.interval))
end

-- Run as daemon service
if arg and arg[1] == "--daemon" then
    -- This would be handled by Phase 10 daemon infrastructure
    print("Starting in daemon mode...")
    main()
else
    main()
end
```

#### Configuration (fleet-config.toml)

```toml
# Fleet Manager Configuration - Phase 10 Service
[fleet]
name = "production-fleet"
max_kernels = 10
startup_kernels = 3

[[fleet.kernels]]
name = "kernel-01"
port = 9551
log_file = "/var/log/llmspell/kernel-01.log"
pid_file = "/var/run/llmspell/kernel-01.pid"
idle_timeout = 7200
max_clients = 20

[[fleet.kernels]]
name = "kernel-02"
port = 9552
log_file = "/var/log/llmspell/kernel-02.log"
pid_file = "/var/run/llmspell/kernel-02.pid"
idle_timeout = 7200
max_clients = 20

[load_balancing]
strategy = "least_loaded" # round_robin, least_loaded, sticky
health_check_interval = "30s"
failover_enabled = true

[multi_tenant]
enabled = true
isolation_level = "strict"
resource_quotas = true

[monitoring]
interval = "30s"
metrics_retention = "7d"
alert_channels = ["slack", "pagerduty"]

[service_discovery]
enabled = true
provider = "consul" # consul, etcd, kubernetes

[providers.openai]
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"
```

### 8.4 Development Environment Service

**Purpose**: IDE integration service providing code intelligence for llmspell scripts
**Problem Statement**: "Developers need intelligent IDE support for llmspell script development"
**Key Features**: LSP implementation, DAP debugging, code completion, live diagnostics

```lua
-- examples/script-users/applications/dev-environment-service/main.lua
--[[
Development Environment Service - IDE Integration Daemon
Phase 10 Application showcasing LSP and DAP protocol support

Problem: Developers need code completion, debugging, and intelligence
features when writing llmspell scripts in their favorite IDEs.
]]

local config = Config.load("dev-service-config.toml")

-- IDE service agents
local code_analyzer = Agent.new({
    name = "code_analyzer",
    model = "gpt-4",
    purpose = "Analyze llmspell scripts for code intelligence"
})

local completion_provider = Agent.new({
    name = "completion_provider",
    model = "gpt-3.5-turbo",
    purpose = "Generate code completions and snippets"
})

local diagnostic_engine = Agent.new({
    name = "diagnostic_engine",
    model = "gpt-4",
    purpose = "Find issues and suggest fixes"
})

local documentation_generator = Agent.new({
    name = "documentation_generator",
    model = "gpt-3.5-turbo",
    purpose = "Generate hover documentation and signatures"
})

local debug_coordinator = Agent.new({
    name = "debug_coordinator",
    model = "gpt-4",
    purpose = "Coordinate debugging sessions with DAP"
})

-- LSP-like protocol implementation
local lsp_server = {
    capabilities = {
        completionProvider = true,
        hoverProvider = true,
        signatureHelpProvider = true,
        definitionProvider = true,
        referencesProvider = true,
        diagnosticProvider = true,
        codeActionProvider = true
    }
}

-- Initialize service state
local service_state = State.new({
    persistence = "memory", -- Fast access for IDE operations
    scope = "session"
})

-- File watching for live reload
local function setup_file_watchers()
    local watcher = Tool.get("file_watcher")

    watcher:watch({
        paths = config.workspace.paths,
        patterns = {"*.lua", "*.js", "*.py"},
        on_change = function(event)
            print(string.format("📝 File changed: %s", event.path))

            -- Reanalyze file
            local analysis = code_analyzer:execute({
                action = "analyze_file",
                path = event.path,
                content = event.content
            })

            -- Update diagnostics
            local diagnostics = diagnostic_engine:execute({
                action = "check_file",
                analysis = analysis,
                rules = config.linting.rules
            })

            -- Send to IDE
            send_diagnostics(event.path, diagnostics)

            -- Hot reload if enabled
            if config.hot_reload.enabled then
                reload_script(event.path)
            end
        end
    })
end

-- Code completion handler
local function handle_completion_request(params)
    local position = params.position
    local document = params.textDocument

    -- Get context around cursor
    local context = extract_context(document, position)

    -- Generate completions
    local completions = completion_provider:execute({
        action = "generate_completions",
        context = context,
        language = detect_language(document),
        libraries = {"llmspell-core", "llmspell-tools", "llmspell-agents"}
    })

    -- Format as LSP CompletionItems
    local items = {}
    for _, completion in ipairs(completions.suggestions) do
        table.insert(items, {
            label = completion.label,
            kind = completion.kind, -- Method, Function, Variable, etc.
            detail = completion.detail,
            documentation = completion.docs,
            insertText = completion.insert_text,
            insertTextFormat = 2 -- Snippet
        })
    end

    return items
end

-- Hover documentation
local function handle_hover_request(params)
    local position = params.position
    local document = params.textDocument

    local symbol = get_symbol_at_position(document, position)

    local docs = documentation_generator:execute({
        action = "generate_hover",
        symbol = symbol,
        context = document.uri
    })

    return {
        contents = {
            kind = "markdown",
            value = docs.markdown
        },
        range = symbol.range
    }
end

-- Debug adapter protocol support
local function setup_debug_adapter()
    local dap_server = Debug.create_dap_server({
        port = config.dap.port or 9555
    })

    dap_server:on("setBreakpoints", function(args)
        local breakpoints = debug_coordinator:execute({
            action = "set_breakpoints",
            source = args.source,
            lines = args.lines
        })

        return {breakpoints = breakpoints}
    end)

    dap_server:on("launch", function(args)
        local session = debug_coordinator:execute({
            action = "start_session",
            program = args.program,
            args = args.args,
            stopOnEntry = args.stopOnEntry
        })

        service_state:set("debug_session", session)
        return {success = true}
    end)

    dap_server:on("continue", function()
        debug_coordinator:execute({
            action = "continue",
            session = service_state:get("debug_session")
        })
    end)

    dap_server:on("stepOver", function()
        debug_coordinator:execute({
            action = "step_over",
            session = service_state:get("debug_session")
        })
    end)

    dap_server:on("variables", function(args)
        return debug_coordinator:execute({
            action = "get_variables",
            reference = args.variablesReference
        })
    end)

    dap_server:start()
    print(string.format("🐛 DAP server listening on port %d", config.dap.port))
end

-- Multi-client support
local client_sessions = {}

local function handle_client_connection(client_id, connection)
    print(string.format("🔌 Client connected: %s", client_id))

    client_sessions[client_id] = {
        connection = connection,
        workspace = nil,
        capabilities = nil
    }

    -- Initialize client workspace
    connection:on("initialize", function(params)
        client_sessions[client_id].workspace = params.rootUri
        client_sessions[client_id].capabilities = params.capabilities

        -- Index workspace
        code_analyzer:execute({
            action = "index_workspace",
            path = params.rootUri
        })

        return {
            capabilities = lsp_server.capabilities,
            serverInfo = {
                name = "llmspell-language-server",
                version = "0.10.0"
            }
        }
    end)

    -- Handle requests
    connection:on("textDocument/completion", handle_completion_request)
    connection:on("textDocument/hover", handle_hover_request)

    connection:on("textDocument/didChange", function(params)
        -- Live analysis as user types
        local diagnostics = diagnostic_engine:execute({
            action = "incremental_check",
            changes = params.contentChanges
        })

        send_diagnostics(params.textDocument.uri, diagnostics)
    end)
end

-- Signal handling for service mode
local function setup_signal_handlers()
    Hook.register("signal.SIGTERM", function()
        print("🛑 Shutting down development service...")

        -- Notify all connected clients
        for client_id, session in pairs(client_sessions) do
            session.connection:notify("$/serverStopping", {})
        end

        -- Save service state
        service_state:persist()

        print("✅ Service shutdown complete")
    end)

    Hook.register("signal.SIGHUP", function()
        print("♻️ Reloading configuration...")
        config = Config.load("dev-service-config.toml")
        setup_file_watchers() -- Reconfigure watchers
    end)
end

-- Main service
local function main()
    print("🚀 Development Environment Service starting...")

    -- Setup signal handlers
    setup_signal_handlers()

    -- Start file watchers
    setup_file_watchers()

    -- Start debug adapter
    if config.dap.enabled then
        setup_debug_adapter()
    end

    -- Start LSP server
    local lsp_transport = config.lsp.transport or "stdio"

    if lsp_transport == "tcp" then
        local server = Network.create_server({
            port = config.lsp.port or 9556,
            on_connection = handle_client_connection
        })

        server:start()
        print(string.format("📡 LSP server listening on port %d", config.lsp.port))

    elseif lsp_transport == "stdio" then
        -- Standard I/O for VS Code extension
        handle_client_connection("stdio", io)
    end

    -- Register with IDE extension registry
    if config.extension_registry.enabled then
        register_with_extension_registry({
            name = "llmspell-ide-service",
            capabilities = lsp_server.capabilities,
            languages = {"lua", "javascript", "python"},
            version = "0.10.0"
        })
    end

    print("✅ Development service ready")
    print("   Supported languages: Lua, JavaScript, Python")
    print(string.format("   LSP transport: %s", lsp_transport))
    print(string.format("   DAP enabled: %s", config.dap.enabled))
    print(string.format("   Hot reload: %s", config.hot_reload.enabled))
end

-- Service entry point
main()
```

#### Configuration (dev-service-config.toml)

```toml
# Development Service Configuration - Phase 10 IDE Integration
[workspace]
paths = ["./src", "./examples", "./tests"]
index_on_startup = true

[lsp]
enabled = true
transport = "tcp" # tcp, stdio, pipe
port = 9556

[dap]
enabled = true
port = 9555

[hot_reload]
enabled = true
debounce_ms = 500

[linting]
enabled = true
rules = ["no-unused-vars", "consistent-style", "agent-naming"]

[completion]
max_suggestions = 20
include_snippets = true
smart_imports = true

[extension_registry]
enabled = true
publish_to = "vscode-marketplace"

[providers.openai]
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"
```

### 8.5 Key Phase 10 Features Demonstrated

These example applications showcase the following Phase 10 capabilities:

#### Daemon Mode
- Both applications run as background services using `llmspell kernel start --daemon`
- Proper Unix daemon behavior with PID file management
- stdout/stderr redirection to log files

#### Signal Handling
- **SIGTERM**: Graceful shutdown with state preservation
- **SIGUSR1**: Statistics dump and health reports
- **SIGHUP**: Configuration reload without restart

#### Multi-Client Support
- Fleet manager handles multiple kernel instances
- Dev service manages multiple IDE connections
- Session isolation and resource management

#### Service Integration
- systemd/launchd ready with proper Type=forking
- PID files for process management
- Health monitoring and automatic recovery

#### IDE Protocols
- LSP implementation for code intelligence
- DAP support for debugging
- Real-time diagnostics and completions

#### Production Features
- Resource limits and quotas
- Multi-tenant isolation
- Load balancing strategies
- Metrics collection and alerting
- Service discovery integration

### 8.6 Running the Examples

#### As Standalone Services

```bash
# Start kernel fleet manager
llmspell run examples/script-users/applications/kernel-fleet-manager/main.lua \
  -c examples/script-users/applications/kernel-fleet-manager/config.toml

# Start development environment service
llmspell run examples/script-users/applications/dev-environment-service/main.lua \
  -c examples/script-users/applications/dev-environment-service/config.toml
```

#### As System Services

```bash
# Install fleet manager as systemd service
sudo cp examples/script-users/applications/kernel-fleet-manager/llmspell-fleet.service \
  /etc/systemd/system/
sudo systemctl enable llmspell-fleet
sudo systemctl start llmspell-fleet

# Install dev service as systemd service
sudo cp examples/script-users/applications/dev-environment-service/llmspell-dev.service \
  /etc/systemd/system/
sudo systemctl enable llmspell-dev
sudo systemctl start llmspell-dev
```

#### With Docker

```dockerfile
# Dockerfile for fleet manager
FROM rust:latest
COPY target/release/llmspell /usr/local/bin/
COPY examples/script-users/applications/kernel-fleet-manager/ /app/
WORKDIR /app
CMD ["llmspell", "kernel", "start", "--daemon", "--config", "fleet-config.toml"]
```

### 8.7 Integration with Existing Applications

These service-level applications can orchestrate and enhance the existing application examples:

- **Fleet Manager** can run multiple instances of `webapp-creator` for different teams
- **Dev Service** provides IDE support when developing any of the Layer 1-6 applications
- Both integrate with the Phase 9 kernel architecture for seamless execution

---

## 9. Implementation Strategy

### 9.1 Task Breakdown

Phase 10 implementation tasks organized by week:

**Week 33: Service Infrastructure Foundation**
- Task 10.1.1: Create `llmspell-service` crate structure (4 hours)
- Task 10.1.2: Implement ServiceManager core (8 hours)
- Task 10.1.3: Add `serve` command to CLI (4 hours)
- Task 10.1.4: Implement ClientRegistry (6 hours)
- Task 10.1.5: Create service configuration system (4 hours)
- Task 10.1.6: Write unit tests for service layer (6 hours)

**Week 34: Jupyter Lab & VS Code Integration**
- Task 10.2.1: Complete ZeroMQ transport implementation (8 hours)
- Task 10.2.2: Implement Jupyter server with 5 channels (12 hours)
- Task 10.2.3: DAP server implementation (8 hours)
- Task 10.2.4: VS Code extension configuration (4 hours)
- Task 10.2.5: Integration testing with real IDEs (8 hours)

**Week 35: LSP & REPL Service**
- Task 10.3.1: LSP server implementation (12 hours)
- Task 10.3.2: Symbol indexing and completion (8 hours)
- Task 10.3.3: REPL service with multi-client support (8 hours)
- Task 10.3.4: WebSocket transport for REPL (4 hours)
- Task 10.3.5: Performance optimization (8 hours)

**Week 36: Deployment & Security**
- Task 10.4.1: Systemd/launchd service units (4 hours)
- Task 10.4.2: Health monitoring implementation (6 hours)
- Task 10.4.3: TLS and authentication layer (8 hours)
- Task 10.4.4: Metrics and logging integration (6 hours)
- Task 10.4.5: End-to-end testing including example applications (8 hours)
- Task 10.4.6: Documentation and example application development (8 hours)

### 9.2 Testing Requirements

Comprehensive testing strategy for service layer:

```rust
// llmspell-service/tests/integration_tests.rs

#[tokio::test]
async fn test_multi_protocol_server() {
    // Start service with all protocols
    let config = ServiceConfig::test_config();
    let service = ServiceManager::new(config).await.unwrap();
    service.start().await.unwrap();

    // Connect via Jupyter
    let jupyter_client = JupyterClient::connect("localhost:8888").await.unwrap();
    assert!(jupyter_client.execute("print('test')").await.is_ok());

    // Connect via DAP
    let dap_client = DAPClient::connect("localhost:8889").await.unwrap();
    assert!(dap_client.initialize().await.is_ok());

    // Connect via LSP
    let lsp_client = LSPClient::connect("localhost:8890").await.unwrap();
    assert!(lsp_client.initialize().await.is_ok());

    // Connect via REPL
    let repl_client = REPLClient::connect("localhost:8891").await.unwrap();
    assert!(repl_client.execute("return 1+1").await.unwrap() == "2");
}

#[tokio::test]
async fn test_concurrent_clients() {
    let service = create_test_service().await;

    // Connect 10 clients concurrently
    let mut handles = Vec::new();
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let client = JupyterClient::connect("localhost:8888").await.unwrap();
            client.execute(&format!("return {}", i)).await.unwrap()
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

#[tokio::test]
async fn test_session_persistence() {
    let service = create_test_service().await;

    // Create session
    let client1 = JupyterClient::connect("localhost:8888").await.unwrap();
    let session_id = client1.create_session().await.unwrap();
    client1.execute("x = 42").await.unwrap();

    // Disconnect and reconnect
    drop(client1);
    let client2 = JupyterClient::connect("localhost:8888").await.unwrap();
    client2.join_session(&session_id).await.unwrap();

    // State should persist
    let result = client2.execute("return x").await.unwrap();
    assert_eq!(result, "42");
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let service = create_test_service().await;

    // Connect clients
    let client1 = JupyterClient::connect("localhost:8888").await.unwrap();
    let client2 = DAPClient::connect("localhost:8889").await.unwrap();

    // Trigger shutdown
    service.shutdown().await.unwrap();

    // Clients should receive shutdown notification
    assert!(client1.wait_for_shutdown().await.is_ok());
    assert!(client2.wait_for_shutdown().await.is_ok());
}
```

### 9.3 Performance Targets

Performance requirements and benchmarks:

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Message handling latency | <5ms | P95 latency across 1000 messages |
| Debug stepping latency | <20ms | Time from step command to response |
| LSP completion time | <100ms | Time to generate completions |
| Concurrent clients | 100+ | Max clients before degradation |
| Memory per client | <10MB | RSS growth per connection |
| Startup time | <2s | Time to accept first connection |
| Shutdown time | <5s | Graceful shutdown with 100 clients |
| Message throughput | >10K/sec | Messages processed per second |

### 9.4 Documentation Requirements

Documentation deliverables for Phase 10:

1. **Service Configuration Guide**: Complete TOML configuration reference
2. **IDE Integration Guide**: Setup instructions for VS Code, Jupyter, vim
3. **Deployment Guide**: Production deployment with systemd/launchd
4. **Security Guide**: TLS setup, authentication configuration
5. **API Reference**: Protocol-specific API documentation
6. **Troubleshooting Guide**: Common issues and solutions
7. **Performance Tuning Guide**: Optimization recommendations
8. **Example Applications Guide**: Kernel Fleet Manager and Dev Environment Service
9. **Application Layer 7 Documentation**: Service & Production Territory examples

---

## 10. Risk Analysis and Mitigation

### Identified Risks

1. **Protocol Compatibility**: Jupyter/DAP/LSP protocol version mismatches
   - **Mitigation**: Implement protocol version negotiation
   - **Mitigation**: Extensive testing with real clients

2. **Performance Degradation**: Multiple clients impacting kernel performance
   - **Mitigation**: Resource pooling and connection limits
   - **Mitigation**: Implement backpressure mechanisms

3. **Security Vulnerabilities**: Remote code execution, privilege escalation
   - **Mitigation**: Mandatory sandboxing for all execution
   - **Mitigation**: Regular security audits

4. **State Synchronization**: Conflicts between concurrent client operations
   - **Mitigation**: Implement proper locking and transaction semantics
   - **Mitigation**: Event sourcing for conflict resolution

5. **Memory Leaks**: Long-running service accumulating memory
   - **Mitigation**: Implement proper resource cleanup
   - **Mitigation**: Regular memory profiling and testing

---

## 11. Success Metrics

### Acceptance Criteria Checklist

- [ ] `llmspell kernel start --daemon` properly daemonizes
- [ ] Jupyter Lab can connect and execute notebooks
- [ ] VS Code can debug with breakpoints and stepping
- [ ] Any LSP client receives completions and diagnostics
- [ ] REPL service handles multiple concurrent sessions
- [ ] Service runs as systemd/launchd daemon reliably
- [ ] Health checks and monitoring endpoints functional
- [ ] TLS encryption works for remote connections
- [ ] Authentication prevents unauthorized access
- [ ] Performance meets all specified targets
- [ ] All integration tests pass
- [ ] Documentation complete and reviewed

### Definition of Done

Phase 10 is complete when:
1. All protocol servers implemented and tested
2. Multi-client support validated with 100+ concurrent connections
3. Security layer prevents unauthorized access
4. Service deployment automated for Linux/macOS
5. Performance targets achieved and validated
6. Documentation enables users to connect from any IDE
7. Integration tests cover all major scenarios
8. Code review completed and approved
9. Performance benchmarks documented
10. User acceptance testing passed

---

## Appendix: Configuration Examples

### A.1 Complete Service Configuration

```toml
# /etc/llmspell/service.toml

[service]
id = "llmspell-prod"
daemon = true
pid_file = "/var/run/llmspell.pid"

[logging]
level = "info"
file = "/var/log/llmspell/service.log"
rotate_size = "100MB"
rotate_count = 10

[security]
tls_enabled = true
cert_path = "/etc/llmspell/certs/server.crt"
key_path = "/etc/llmspell/certs/server.key"
auth_method = "token"
token_file = "/etc/llmspell/tokens.json"
audit_log = true
allowed_origins = ["http://localhost:*", "https://notebook.example.com"]
ip_whitelist = []  # Empty means all IPs allowed

[jupyter]
enabled = true
ip = "0.0.0.0"
transport = "tcp"
kernel_name = "llmspell"
connection_file = "/tmp/llmspell-kernel.json"
key = "your-hmac-key-here"

[jupyter.ports]
shell_port = 8888
iopub_port = 8889
stdin_port = 8890
control_port = 8891
hb_port = 8892

[dap]
enabled = true
host = "0.0.0.0"
port = 9555

[lsp]
enabled = true
host = "0.0.0.0"
port = 9556

[repl]
enabled = true
transport = "tcp"
host = "0.0.0.0"
port = 9557
history_file = "/var/lib/llmspell/repl_history"

[limits]
max_clients = 100
max_memory_mb = 2048
max_cpu_percent = 200.0
session_timeout_secs = 3600
max_message_size = 10485760  # 10MB

[health]
enabled = true
port = 9558
check_interval_secs = 30
enable_alerts = true
alert_webhook = "https://alerts.example.com/webhook"

[metrics]
enabled = true
exporter = "prometheus"
port = 9559
namespace = "llmspell"
```

## 12. Service Deployment & Management

### 10.1 Systemd Service Unit (Linux)

Systemd configuration using Type=forking for proper daemon mode:

```ini
# /etc/systemd/system/llmspell.service
[Unit]
Description=LLMSpell Kernel Service
After=network.target

[Service]
Type=forking
PIDFile=/var/run/llmspell.pid
ExecStart=/usr/local/bin/llmspell kernel start --daemon --all --port 9555 --pid-file /var/run/llmspell.pid --log-file /var/log/llmspell/kernel.log
ExecReload=/bin/kill -HUP $MAINPID
ExecStop=/bin/kill -TERM $MAINPID
KillMode=process
KillSignal=SIGTERM
TimeoutStopSec=30
Restart=on-failure
RestartSec=5s

# User and group
User=llmspell
Group=llmspell

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/llmspell /var/log/llmspell /var/run

# Resource limits
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
CPUQuota=200%

# Environment
Environment="LLMSPELL_LOG_LEVEL=info"
Environment="LLMSPELL_TRACE=info"
Environment="RUST_BACKTRACE=1"

[Install]
WantedBy=multi-user.target
```

### 10.2 Launchd Service Plist (macOS)

Launchd configuration for automatic daemon management:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
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
        <string>--all</string>
        <string>--port</string>
        <string>9555</string>
        <string>--pid-file</string>
        <string>/usr/local/var/run/llmspell.pid</string>
        <string>--log-file</string>
        <string>/usr/local/var/log/llmspell/kernel.log</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
        <key>NetworkState</key>
        <true/>
    </dict>

    <key>StandardOutPath</key>
    <string>/usr/local/var/log/llmspell/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/llmspell/stderr.log</string>

    <key>WorkingDirectory</key>
    <string>/usr/local/var/lib/llmspell</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>LLMSPELL_LOG_LEVEL</key>
        <string>info</string>
        <key>LLMSPELL_TRACE</key>
        <string>info</string>
    </dict>

    <key>SoftResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
        <key>NumberOfProcesses</key>
        <integer>512</integer>
    </dict>

    <key>HardResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
        <key>NumberOfProcesses</key>
        <integer>512</integer>
    </dict>

    <key>ProcessType</key>
    <string>Background</string>
</dict>
</plist>
```

### 10.3 Service Management Commands

Common service management operations:

```bash
# Linux (systemd)
# Start service
sudo systemctl start llmspell

# Enable auto-start on boot
sudo systemctl enable llmspell

# Check status
sudo systemctl status llmspell

# View logs
sudo journalctl -u llmspell -f

# Reload configuration (SIGHUP)
sudo systemctl reload llmspell

# Stop service (graceful)
sudo systemctl stop llmspell

# macOS (launchd)
# Load and start service
sudo launchctl load -w /Library/LaunchDaemons/com.llmspell.kernel.plist

# Check if running
sudo launchctl list | grep llmspell

# Stop service
sudo launchctl stop com.llmspell.kernel

# Unload service
sudo launchctl unload /Library/LaunchDaemons/com.llmspell.kernel.plist

# View logs
tail -f /usr/local/var/log/llmspell/kernel.log
```

## 13. Implementation Tasks

### 13.1 Task Breakdown

Phase 10 implementation organized by week:

**Week 33: Kernel Enhancement & Daemon Implementation**
- Task 10.1.1: Implement DaemonManager with double-fork (8 hours)
- Task 10.1.2: Implement SignalBridge for signal handling (6 hours)
- Task 10.1.3: Enhance KernelService with protocol servers (8 hours)
- Task 10.1.4: Implement daemon logging infrastructure (6 hours)
- Task 10.1.5: Update CLI for enhanced kernel start (4 hours)
- Task 10.1.6: Unit tests for daemon behavior (8 hours)

**Week 34: Jupyter Lab & VS Code Integration**
- Task 10.2.1: Complete ZeroMQ transport in kernel (8 hours)
- Task 10.2.2: Implement Jupyter 5-channel server (12 hours)
- Task 10.2.3: DAP server in kernel module (8 hours)
- Task 10.2.4: VS Code extension configuration (4 hours)
- Task 10.2.5: Integration testing with real IDEs (8 hours)

**Week 35: LSP & REPL Service**
- Task 10.3.1: LSP server in kernel module (12 hours)
- Task 10.3.2: Symbol indexing and completion (8 hours)
- Task 10.3.3: REPL service with multi-client (8 hours)
- Task 10.3.4: Connection file management (4 hours)
- Task 10.3.5: Performance optimization (8 hours)

**Week 36: Deployment & Production**
- Task 10.4.1: Systemd/launchd service units (4 hours)
- Task 10.4.2: PID file and process management (6 hours)
- Task 10.4.3: TLS and authentication layer (8 hours)
- Task 10.4.4: Health monitoring implementation (6 hours)
- Task 10.4.5: End-to-end testing with example applications (8 hours)
- Task 10.4.6: Final documentation and example validation (8 hours)

### 13.2 Testing Requirements

Comprehensive testing for daemon and service modes:

```rust
// llmspell-kernel/tests/daemon_tests.rs

#[test]
fn test_double_fork_daemonization() {
    // Test that process properly detaches from TTY
    // Verify PID file creation
    // Check parent process exits
}

#[test]
fn test_signal_handling() {
    // Send SIGTERM, verify shutdown_request sent
    // Send SIGINT, verify interrupt_request sent
    // Send SIGHUP, verify config reload
}

#[test]
fn test_pid_file_locking() {
    // Attempt to start second instance
    // Verify it fails with "already running"
    // Clean up stale PID file
}

#[test]
fn test_log_rotation() {
    // Write logs exceeding max size
    // Verify rotation occurs
    // Check old logs preserved
}

#[tokio::test]
async fn test_multi_protocol_server() {
    // Start kernel with all protocols
    // Connect via each protocol
    // Verify all work simultaneously
}
```

### 13.3 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Daemon startup time | <2s | Time to write PID file |
| Signal response time | <100ms | SIGTERM to shutdown_request |
| Connection file write | <50ms | Time to write JSON file |
| Protocol server init | <500ms | Per protocol startup |
| Memory overhead | <50MB | Daemon vs embedded mode |
| Log rotation time | <100ms | Time to rotate large log |
| PID file check | <10ms | Check if already running |

## 14. Summary

This Phase 10 design refactors the service architecture to maintain a single-binary approach where:

1. **`llmspell` is the only executable** - no separate service binary
2. **`kernel start --daemon`** enables proper Unix daemon mode
3. **Double-fork technique** properly detaches from TTY
4. **Signals convert to messages** for graceful protocol handling
5. **Logging infrastructure** handles stdout/stderr redirection
6. **systemd/launchd** manage the daemon lifecycle
7. **Protocol servers** run within the kernel process
8. **Connection files** enable Jupyter discovery

The architecture maintains simplicity while providing production-ready daemon capabilities, proper process management, and multi-protocol support for IDE connectivity.

### Example Applications

Phase 10 introduces two production-ready example applications that showcase the new service capabilities:

- **Kernel Fleet Manager**: Production orchestration service managing multiple kernel instances with load balancing, multi-tenant isolation, and health monitoring
- **Development Environment Service**: IDE integration daemon providing LSP code intelligence, DAP debugging support, and real-time diagnostics for llmspell script development

These examples represent the evolution to **Layer 7: Service & Production Territory**, building upon the existing application progression from universal user problems through professional automation to production service deployments.

---

This completes the refactored Phase 10 design document with proper Unix daemon implementation, signal handling, single-binary architecture, and correct service deployment configurations.