//! ABOUTME: Kernel server command implementation
//! ABOUTME: Manages Jupyter protocol kernel servers for external clients

use crate::cli::{KernelCommands, OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    ConnectionInfo, JupyterClient, JupyterKernel, JupyterProtocol, KernelDiscovery, ZmqTransport,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Path to store the last successful connection
fn last_connection_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".llmspell")
        .join("last_kernel.json")
}

/// Save the last successful connection for reuse
async fn save_last_connection(info: &ConnectionInfo) -> Result<()> {
    let path = last_connection_file();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Save minimal info needed to reconnect
    let last_connection = serde_json::json!({
        "kernel_id": info.kernel_id,
        "ip": info.ip,
        "port": info.shell_port,
        "saved_at": chrono::Utc::now().to_rfc3339()
    });

    tokio::fs::write(&path, serde_json::to_string_pretty(&last_connection)?).await?;
    tracing::debug!("Saved last connection to {}", path.display());
    Ok(())
}

/// Load the last successful connection
async fn load_last_connection() -> Result<ConnectionInfo> {
    let path = last_connection_file();

    if !path.exists() {
        anyhow::bail!("No previous connection found. Please specify a kernel address.");
    }

    let content = tokio::fs::read_to_string(&path).await?;
    let last_conn: serde_json::Value = serde_json::from_str(&content)?;

    // Try to find the kernel by ID first
    if let Some(kernel_id) = last_conn["kernel_id"].as_str() {
        let discovery = KernelDiscovery::new();
        if let Some(info) = discovery.find_kernel(kernel_id).await? {
            return Ok(info);
        }
    }

    // Fall back to reconstructing from saved IP and port
    let kernel_id = last_conn["kernel_id"]
        .as_str()
        .unwrap_or("last-connection")
        .to_string();
    let ip = last_conn["ip"].as_str().unwrap_or("127.0.0.1").to_string();
    let port = last_conn["port"].as_u64().unwrap_or(9555) as u16;

    Ok(ConnectionInfo::new(kernel_id, ip, port))
}

/// Format a duration in human-readable form
fn format_duration(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds();
    if seconds < 60 {
        format!("{} seconds", seconds)
    } else if seconds < 3600 {
        format!("{} minutes", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours", seconds / 3600)
    } else {
        format!("{} days", seconds / 86400)
    }
}

/// Handle kernel subcommands
pub async fn handle_kernel_command(
    command: KernelCommands,
    engine: ScriptEngine,
    config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        KernelCommands::Start {
            port,
            daemon,
            id,
            connection_file,
        } => start_kernel(engine, port, daemon, id, connection_file, config).await,
        KernelCommands::Stop { id } => stop_kernel(id, output_format).await,
        KernelCommands::Status { id } => show_kernel_status(id, output_format).await,
        KernelCommands::Connect { address } => connect_to_kernel(address, output_format).await,
    }
}

/// Start the kernel server
async fn start_kernel(
    engine: ScriptEngine,
    port: u16,
    daemon: bool,
    id: Option<String>,
    connection_file: Option<PathBuf>,
    config: LLMSpellConfig,
) -> Result<()> {
    println!("Starting LLMSpell kernel server...");
    println!("Engine: {}", engine.as_str());
    println!("Port: {}", port);
    println!("Daemon mode: {}", daemon);

    // Generate kernel ID
    let kernel_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    println!("Kernel ID: {}", kernel_id);

    // Create connection info with simple constructor
    let mut connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    // Add process management info
    connection_info.pid = Some(std::process::id());
    connection_info.started_at = Some(chrono::Utc::now());

    // Save connection file for discovery
    let conn_file = if let Some(file) = connection_file {
        file
    } else {
        // Use the standard connection file path
        connection_info.connection_file_path()
    };

    // Ensure directory exists
    if let Some(parent) = conn_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write connection info to file
    connection_info.write_connection_file().await?;
    println!("Connection file: {}", conn_file.display());

    // Set the default engine in the config based on CLI selection
    let mut config = config;
    config.default_engine = engine.as_str().to_string();

    // Create shared config
    let config = Arc::new(config);

    // Create transport and protocol
    let transport = ZmqTransport::new()?;
    let protocol = JupyterProtocol::new(connection_info.clone());

    // Create and run the kernel
    let mut kernel = JupyterKernel::new(kernel_id.clone(), config, transport, protocol).await?;

    println!("\nKernel server started successfully!");
    println!("Press Ctrl+C to shutdown\n");

    // Run the kernel (blocks until shutdown)
    kernel.serve().await?;

    // Cleanup connection file
    let _ = std::fs::remove_file(&conn_file);

    Ok(())
}

/// Stop a running kernel
async fn stop_kernel(id: Option<String>, output_format: OutputFormat) -> Result<()> {
    let discovery = KernelDiscovery::new();

    match id {
        Some(kernel_id) => {
            // Stop specific kernel
            println!("Stopping kernel: {}", kernel_id);

            // Find the kernel connection info
            if let Some(info) = discovery.find_kernel(&kernel_id).await? {
                // Check if kernel is alive
                if KernelDiscovery::is_kernel_alive(&info).await? {
                    // Create client and send proper shutdown request
                    let transport = ZmqTransport::new()?;
                    let protocol = JupyterProtocol::new(info.clone());

                    match JupyterClient::connect(transport, protocol, info.clone()).await {
                        Ok(mut client) => {
                            // Send shutdown_request with 5 second timeout
                            match timeout(Duration::from_secs(5), client.shutdown(false)).await {
                                Ok(Ok(_)) => {
                                    if matches!(
                                        output_format,
                                        OutputFormat::Json | OutputFormat::Pretty
                                    ) {
                                        println!(
                                            "{}",
                                            serde_json::json!({
                                                "action": "stop",
                                                "kernel_id": kernel_id,
                                                "status": "shutdown_gracefully"
                                            })
                                        );
                                    } else {
                                        println!("Kernel {} shutdown gracefully", kernel_id);
                                    }
                                    // Clean up connection file after successful shutdown
                                    info.remove_connection_file().await?;
                                }
                                Ok(Err(e)) => {
                                    eprintln!("Shutdown request failed: {}", e);
                                    // Try to force kill if we have PID
                                    if let Some(pid) = info.pid {
                                        eprintln!("Attempting to force kill process {}", pid);
                                        #[cfg(unix)]
                                        {
                                            use std::process::Command;
                                            let _ = Command::new("kill")
                                                .args(["-TERM", &pid.to_string()])
                                                .output();
                                        }
                                    }
                                    // Remove connection file
                                    info.remove_connection_file().await?;

                                    if matches!(
                                        output_format,
                                        OutputFormat::Json | OutputFormat::Pretty
                                    ) {
                                        println!(
                                            "{}",
                                            serde_json::json!({
                                                "action": "stop",
                                                "kernel_id": kernel_id,
                                                "status": "shutdown_failed",
                                                "error": e.to_string()
                                            })
                                        );
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Shutdown request timed out after 5 seconds");
                                    // Try to force kill if we have PID
                                    if let Some(pid) = info.pid {
                                        eprintln!("Attempting to force kill process {}", pid);
                                        #[cfg(unix)]
                                        {
                                            use std::process::Command;
                                            let _ = Command::new("kill")
                                                .args(["-TERM", &pid.to_string()])
                                                .output();
                                        }
                                    }
                                    // Force cleanup
                                    info.remove_connection_file().await?;

                                    if matches!(
                                        output_format,
                                        OutputFormat::Json | OutputFormat::Pretty
                                    ) {
                                        println!(
                                            "{}",
                                            serde_json::json!({
                                                "action": "stop",
                                                "kernel_id": kernel_id,
                                                "status": "shutdown_timeout"
                                            })
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Could not connect to send shutdown, kernel might be stuck
                            eprintln!("Could not connect to kernel: {}", e);
                            // Clean up connection file anyway
                            info.remove_connection_file().await?;

                            if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                                println!(
                                    "{}",
                                    serde_json::json!({
                                        "action": "stop",
                                        "kernel_id": kernel_id,
                                        "status": "connection_failed",
                                        "cleaned_up": true
                                    })
                                );
                            }
                        }
                    }
                } else {
                    // Kernel is already dead, clean up connection file
                    info.remove_connection_file().await?;

                    if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                        println!(
                            "{}",
                            serde_json::json!({
                                "action": "stop",
                                "kernel_id": kernel_id,
                                "status": "already_stopped"
                            })
                        );
                    } else {
                        println!(
                            "Kernel {} was already stopped. Cleaned up connection file.",
                            kernel_id
                        );
                    }
                }
            } else {
                anyhow::bail!("Kernel {} not found", kernel_id);
            }
        }
        None => {
            // Stop all kernels with proper shutdown protocol
            println!("Stopping all kernels...");

            let kernels = discovery.discover_kernels().await?;
            let mut graceful_count = 0;
            let mut forced_count = 0;
            let mut already_stopped = 0;

            for info in kernels {
                if KernelDiscovery::is_kernel_alive(&info).await? {
                    // Try graceful shutdown first
                    let transport = ZmqTransport::new()?;
                    let protocol = JupyterProtocol::new(info.clone());

                    match JupyterClient::connect(transport, protocol, info.clone()).await {
                        Ok(mut client) => {
                            match timeout(Duration::from_secs(2), client.shutdown(false)).await {
                                Ok(Ok(_)) => {
                                    graceful_count += 1;
                                    println!("  {} - Shutdown gracefully", info.kernel_id);
                                }
                                _ => {
                                    forced_count += 1;
                                    println!("  {} - Forced cleanup", info.kernel_id);
                                }
                            }
                        }
                        Err(_) => {
                            forced_count += 1;
                            println!("  {} - Connection failed, cleaned up", info.kernel_id);
                        }
                    }
                    info.remove_connection_file().await?;
                } else {
                    info.remove_connection_file().await?;
                    already_stopped += 1;
                }
            }

            if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                println!(
                    "{}",
                    serde_json::json!({
                        "action": "stop_all",
                        "graceful_shutdown": graceful_count,
                        "forced_cleanup": forced_count,
                        "already_stopped": already_stopped,
                        "total": graceful_count + forced_count + already_stopped
                    })
                );
            } else {
                println!("\nSummary:");
                println!("  Graceful shutdowns: {}", graceful_count);
                println!("  Forced cleanups: {}", forced_count);
                println!("  Already stopped: {}", already_stopped);
            }
        }
    }

    Ok(())
}

/// Show kernel status
async fn show_kernel_status(id: Option<String>, output_format: OutputFormat) -> Result<()> {
    let discovery = KernelDiscovery::new();

    match id {
        Some(kernel_id) => {
            // Show detailed status for specific kernel
            if let Some(info) = discovery.find_kernel(&kernel_id).await? {
                let is_alive = KernelDiscovery::is_kernel_alive(&info).await?;
                let process_alive = info.is_process_alive();

                if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                    let mut json = serde_json::json!({
                        "kernel_id": info.kernel_id,
                        "status": if is_alive { "running" } else { "stopped" },
                        "transport": info.transport,
                        "ip": info.ip,
                        "shell_port": info.shell_port,
                        "iopub_port": info.iopub_port,
                        "stdin_port": info.stdin_port,
                        "control_port": info.control_port,
                        "hb_port": info.hb_port,
                        "connection_file": info.connection_file_path().display().to_string()
                    });

                    if let Some(pid) = info.pid {
                        json.as_object_mut()
                            .unwrap()
                            .insert("pid".to_string(), serde_json::json!(pid));
                        json.as_object_mut().unwrap().insert(
                            "process_alive".to_string(),
                            serde_json::json!(process_alive),
                        );
                    }

                    if let Some(started_at) = info.started_at {
                        let uptime = chrono::Utc::now() - started_at;
                        json.as_object_mut().unwrap().insert(
                            "started_at".to_string(),
                            serde_json::json!(started_at.to_rfc3339()),
                        );
                        json.as_object_mut().unwrap().insert(
                            "uptime_seconds".to_string(),
                            serde_json::json!(uptime.num_seconds()),
                        );
                    }

                    println!("{}", json);
                } else {
                    println!("Kernel ID: {}", info.kernel_id);
                    println!(
                        "Status: {}",
                        if is_alive {
                            "Running"
                        } else {
                            "Stopped (stale connection file)"
                        }
                    );

                    if let Some(pid) = info.pid {
                        println!(
                            "Process ID: {} ({})",
                            pid,
                            if process_alive { "alive" } else { "dead" }
                        );
                    }

                    if let Some(started_at) = info.started_at {
                        let uptime = chrono::Utc::now() - started_at;
                        println!(
                            "Started: {} ({} ago)",
                            started_at.format("%Y-%m-%d %H:%M:%S UTC"),
                            format_duration(uptime)
                        );
                    }

                    println!("Transport: {}", info.transport);
                    println!("Address: {}:{}", info.ip, info.shell_port);
                    println!("Ports:");
                    println!("  Shell: {}", info.shell_port);
                    println!("  IOPub: {}", info.iopub_port);
                    println!("  Stdin: {}", info.stdin_port);
                    println!("  Control: {}", info.control_port);
                    println!("  Heartbeat: {}", info.hb_port);
                    println!("Connection file: {}", info.connection_file_path().display());
                }
            } else {
                anyhow::bail!("Kernel {} not found", kernel_id);
            }
        }
        None => {
            // List all kernels
            let kernels = discovery.discover_kernels().await?;

            if kernels.is_empty() {
                if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                    println!(
                        "{}",
                        serde_json::json!({
                            "kernels": []
                        })
                    );
                } else {
                    println!("No kernels found");
                }
            } else {
                let mut kernel_list = Vec::new();

                for info in kernels {
                    let is_alive = KernelDiscovery::is_kernel_alive(&info).await?;

                    if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                        kernel_list.push(serde_json::json!({
                            "kernel_id": info.kernel_id,
                            "status": if is_alive { "running" } else { "stopped" },
                            "address": format!("{}:{}", info.ip, info.shell_port)
                        }));
                    } else {
                        println!(
                            "  {} - {} [{}:{}]",
                            info.kernel_id,
                            if is_alive { "Running" } else { "Stopped" },
                            info.ip,
                            info.shell_port
                        );
                    }
                }

                if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                    println!(
                        "{}",
                        serde_json::json!({
                            "kernels": kernel_list
                        })
                    );
                }
            }
        }
    }

    Ok(())
}

/// Connect to an existing kernel
async fn connect_to_kernel(address: Option<String>, output_format: OutputFormat) -> Result<()> {
    // If no address provided, try to use the last connection
    let address = match address {
        Some(addr) => addr,
        None => {
            println!("No address provided, using last successful connection...");
            let last_info = load_last_connection().await?;
            // Return the kernel ID to continue with normal flow
            last_info.kernel_id
        }
    };

    // Parse address - could be:
    // 1. kernel ID (just the ID)
    // 2. host:port (e.g., "localhost:9555")
    // 3. path to connection file (e.g., "/path/to/connection.json")

    let info = if address.contains(':') && !address.contains('/') {
        // Looks like host:port - need to discover the kernel to get its key
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid address format. Expected host:port");
        }

        let host = parts[0];
        let port: u16 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid port number"))?;

        // Must discover the kernel to get the connection info with HMAC key
        let discovery = KernelDiscovery::new();
        let kernels = discovery.discover_kernels().await?;

        // Find kernel with matching port
        let matching_kernel = kernels.iter().find(|k| {
            k.shell_port == port && (k.ip == host || (host == "localhost" && k.ip == "127.0.0.1"))
        });

        match matching_kernel {
            Some(kernel_info) => {
                println!(
                    "Found kernel {} at {}:{}",
                    kernel_info.kernel_id, host, port
                );
                kernel_info.clone()
            }
            None => {
                anyhow::bail!(
                    "Could not find kernel at {}:{}. Kernels require HMAC authentication.\n\
                    Use 'llmspell kernel status' to list available kernels, then connect using:\n\
                    - Kernel ID: llmspell kernel connect <kernel-id>\n\
                    - Connection file: llmspell kernel connect /path/to/connection.json",
                    host,
                    port
                );
            }
        }
    } else if address.contains('/') || address.ends_with(".json") {
        // Looks like a file path
        let path = PathBuf::from(&address);
        ConnectionInfo::read_connection_file(&path).await?
    } else {
        // Assume it's a kernel ID
        let discovery = KernelDiscovery::new();
        discovery
            .find_kernel(&address)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Kernel {} not found", address))?
    };

    // First check if kernel is alive via heartbeat
    if !KernelDiscovery::is_kernel_alive(&info).await? {
        anyhow::bail!("Kernel at {} is not responding to heartbeat", address);
    }

    // Create actual client connection to verify protocol communication
    let transport = ZmqTransport::new()?;
    let protocol = JupyterProtocol::new(info.clone());

    println!("Connecting to kernel {}...", info.kernel_id);

    match JupyterClient::connect(transport, protocol, info.clone()).await {
        Ok(mut client) => {
            // Save connection immediately after successful ZMQ connection
            // (even if kernel_info might fail later)
            if let Err(e) = save_last_connection(&info).await {
                tracing::warn!("Failed to save last connection: {}", e);
            }

            // Verify connection with kernel_info_request
            match timeout(Duration::from_secs(3), client.kernel_info()).await {
                Ok(Ok(kernel_info_content)) => {
                    // Parse kernel info from MessageContent
                    if let llmspell_kernel::jupyter::protocol::MessageContent::KernelInfoReply {
                        protocol_version,
                        implementation,
                        implementation_version,
                        language_info,
                        banner,
                        ..
                    } = kernel_info_content
                    {
                        if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "status": "connected",
                                    "kernel_id": info.kernel_id,
                                    "protocol_version": protocol_version,
                                    "implementation": implementation,
                                    "implementation_version": implementation_version,
                                    "language": language_info.name,
                                    "language_version": language_info.version,
                                    "channels": {
                                        "shell": format!("{}:{}", info.ip, info.shell_port),
                                        "iopub": format!("{}:{}", info.ip, info.iopub_port),
                                        "stdin": format!("{}:{}", info.ip, info.stdin_port),
                                        "control": format!("{}:{}", info.ip, info.control_port),
                                        "heartbeat": format!("{}:{}", info.ip, info.hb_port),
                                    },
                                    "connection_file": info.connection_file_path().display().to_string()
                                })
                            );
                        } else {
                            println!("\n✅ Successfully connected to kernel: {}", info.kernel_id);
                            println!("\nKernel Information:");
                            println!("  Protocol: {}", protocol_version);
                            println!(
                                "  Implementation: {} v{}",
                                implementation, implementation_version
                            );
                            println!(
                                "  Language: {} v{}",
                                language_info.name, language_info.version
                            );
                            if !banner.is_empty() {
                                println!("  Banner: {}", banner);
                            }
                            println!("\nConnection Details:");
                            println!("  Address: {}:{}", info.ip, info.shell_port);
                            println!("  All 5 ZMQ channels connected and verified");
                            println!("\nTo use this kernel, run commands with:");
                            println!(
                                "  llmspell exec --connect {} 'your code here'",
                                info.kernel_id
                            );
                            println!("  llmspell run --connect {} script.lua", info.kernel_id);
                            println!("  llmspell repl --connect {}", info.kernel_id);
                            println!("\nTo reconnect later: llmspell kernel connect");
                        }
                    } else {
                        anyhow::bail!("Unexpected response from kernel_info_request");
                    }
                }
                Ok(Err(e)) => {
                    anyhow::bail!("Failed to get kernel info: {}", e);
                }
                Err(_) => {
                    anyhow::bail!("Kernel info request timed out after 3 seconds");
                }
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to connect to kernel: {}", e);
        }
    }

    Ok(())
}
