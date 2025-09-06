//! ABOUTME: Kernel server command implementation
//! ABOUTME: Manages Jupyter protocol kernel servers for external clients

use crate::cli::{KernelCommands, OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{ConnectionInfo, JupyterKernel, JupyterProtocol, ZmqTransport, KernelDiscovery};
use std::path::PathBuf;
use std::sync::Arc;

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
    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

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
                    // Try to send shutdown via control channel (future implementation)
                    // For now, just remove connection file
                    info.remove_connection_file().await?;
                    
                    if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                        println!("{}", serde_json::json!({
                            "action": "stop",
                            "kernel_id": kernel_id,
                            "status": "stopped"
                        }));
                    } else {
                        println!("Kernel {} stopped successfully", kernel_id);
                    }
                } else {
                    // Kernel is already dead, clean up connection file
                    info.remove_connection_file().await?;
                    
                    if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                        println!("{}", serde_json::json!({
                            "action": "stop",
                            "kernel_id": kernel_id,
                            "status": "already_stopped"
                        }));
                    } else {
                        println!("Kernel {} was already stopped. Cleaned up connection file.", kernel_id);
                    }
                }
            } else {
                anyhow::bail!("Kernel {} not found", kernel_id);
            }
        }
        None => {
            // Stop all kernels
            println!("Stopping all kernels...");
            
            let kernels = discovery.discover_kernels().await?;
            let mut stopped_count = 0;
            let mut already_stopped = 0;
            
            for info in kernels {
                if KernelDiscovery::is_kernel_alive(&info).await? {
                    info.remove_connection_file().await?;
                    stopped_count += 1;
                } else {
                    info.remove_connection_file().await?;
                    already_stopped += 1;
                }
            }
            
            if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                println!("{}", serde_json::json!({
                    "action": "stop_all",
                    "stopped": stopped_count,
                    "already_stopped": already_stopped,
                    "total": stopped_count + already_stopped
                }));
            } else {
                println!("Stopped {} kernels, cleaned up {} stale connections", 
                         stopped_count, already_stopped);
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
                
                if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                    println!("{}", serde_json::json!({
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
                    }));
                } else {
                    println!("Kernel ID: {}", info.kernel_id);
                    println!("Status: {}", if is_alive { "Running" } else { "Stopped (stale connection file)" });
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
                    println!("{}", serde_json::json!({
                        "kernels": []
                    }));
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
                        println!("  {} - {} [{}:{}]", 
                                 info.kernel_id,
                                 if is_alive { "Running" } else { "Stopped" },
                                 info.ip,
                                 info.shell_port);
                    }
                }
                
                if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
                    println!("{}", serde_json::json!({
                        "kernels": kernel_list
                    }));
                }
            }
        }
    }
    
    Ok(())
}

/// Connect to an existing kernel
async fn connect_to_kernel(address: String, output_format: OutputFormat) -> Result<()> {
    // Parse address - could be:
    // 1. kernel ID (just the ID)
    // 2. host:port (e.g., "localhost:9555")
    // 3. path to connection file (e.g., "/path/to/connection.json")
    
    let info = if address.contains(':') && !address.contains('/') {
        // Looks like host:port
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid address format. Expected host:port");
        }
        
        let host = parts[0];
        let port: u16 = parts[1].parse()
            .map_err(|_| anyhow::anyhow!("Invalid port number"))?;
        
        // Create minimal connection info for host:port
        ConnectionInfo::new(
            uuid::Uuid::new_v4().to_string(),
            host.to_string(),
            port
        )
    } else if address.contains('/') || address.ends_with(".json") {
        // Looks like a file path
        let path = PathBuf::from(&address);
        ConnectionInfo::read_connection_file(&path).await?
    } else {
        // Assume it's a kernel ID
        let discovery = KernelDiscovery::new();
        discovery.find_kernel(&address).await?
            .ok_or_else(|| anyhow::anyhow!("Kernel {} not found", address))?
    };
    
    // Test connection
    let is_alive = KernelDiscovery::is_kernel_alive(&info).await?;
    
    if is_alive {
        if matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
            println!("{}", serde_json::json!({
                "action": "connect",
                "kernel_id": info.kernel_id,
                "address": format!("{}:{}", info.ip, info.shell_port),
                "status": "connected",
                "connection_file": info.connection_file_path().display().to_string()
            }));
        } else {
            println!("Successfully connected to kernel: {}", info.kernel_id);
            println!("Address: {}:{}", info.ip, info.shell_port);
            println!("\nTo use this kernel, run commands with:");
            println!("  llmspell exec --connect {} 'your code here'", info.kernel_id);
            println!("  llmspell run --connect {} script.lua", info.kernel_id);
            println!("  llmspell repl --connect {}", info.kernel_id);
        }
    } else {
        anyhow::bail!("Kernel at {} is not responding", address);
    }
    
    Ok(())
}
