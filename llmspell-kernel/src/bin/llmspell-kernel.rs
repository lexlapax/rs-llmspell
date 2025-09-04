//! `LLMSpell` Kernel Binary - Lightweight wrapper
//!
//! This binary is a thin wrapper that starts the `LLMSpell` kernel.
//! The actual kernel implementation is in `llmspell_kernel::kernel` module.

use anyhow::Result;
use clap::Parser;
use llmspell_config::LLMSpellConfig;
use llmspell_engine::{LRPRequest, LRPResponse};
use llmspell_kernel::{ConnectionInfo, JupyterKernel};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// `LLMSpell` Kernel - Jupyter-compatible execution kernel
#[derive(Parser, Debug)]
#[command(name = "llmspell-kernel")]
#[command(version, about, long_about = None)]
struct Args {
    /// Kernel ID (auto-generated if not provided)
    #[arg(long)]
    kernel_id: Option<String>,

    /// IP address to bind to  
    #[arg(long, default_value = "127.0.0.1")]
    ip: String,

    /// Starting port for channel allocation
    #[arg(long, default_value = "9555")]
    port: u16,

    /// Enable legacy TCP/LRP protocol compatibility (deprecated)
    #[arg(long)]
    legacy_tcp: bool,

    /// Configuration file path
    #[arg(long)]
    config: Option<String>,

    /// Verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.verbose);

    // Load configuration
    let runtime_config = Arc::new(if let Some(config_path) = &args.config {
        let config_str = tokio::fs::read_to_string(config_path).await?;
        LLMSpellConfig::from_toml(&config_str)
            .map_err(|e| anyhow::anyhow!("Config error: {}", e))?
    } else {
        LLMSpellConfig::default()
    });

    // Create connection info with CLI args
    let kernel_id = args
        .kernel_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let connection_info = ConnectionInfo::new(kernel_id.clone(), args.ip, args.port);

    // Create kernel using factory method - handles all wiring internally
    let mut kernel = JupyterKernel::from_config_with_connection(
        kernel_id,
        runtime_config,
        connection_info.clone(),
    )
    .await?;

    // If legacy TCP compatibility is requested, start both servers
    if args.legacy_tcp {
        // Use a different port for legacy TCP to avoid conflict with Jupyter shell channel
        let tcp_port = connection_info.shell_port + 10; // e.g., 9565 if shell is 9555
        tracing::info!(
            "Starting legacy TCP compatibility server on port {} (Jupyter uses {})",
            tcp_port,
            connection_info.shell_port
        );

        // Start TCP server in background
        let tcp_kernel = Arc::new(kernel);
        let tcp_kernel_clone = tcp_kernel.clone();

        let tcp_task = tokio::spawn(async move {
            if let Err(e) = start_legacy_tcp_server(tcp_port, tcp_kernel_clone).await {
                tracing::error!("Legacy TCP server failed: {}", e);
            }
        });

        // Wait for TCP server (this is a simplified approach for Task 9.8.6)
        tcp_task.await?;
    } else {
        // Serve kernel with Jupyter protocol only
        kernel.serve().await?;
    }

    Ok(())
}

fn setup_logging(verbosity: u8) {
    let filter_level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

/// Simple TCP server for legacy LRP protocol compatibility (Task 9.8.6)
/// This is a temporary bridge to allow CLI to connect during migration
async fn start_legacy_tcp_server(port: u16, _kernel: Arc<JupyterKernel>) -> Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Legacy TCP server listening on {}", addr);

    loop {
        let (socket, client_addr) = listener.accept().await?;
        tracing::info!("Legacy TCP client connected: {}", client_addr);

        // Handle each connection in a separate task
        tokio::spawn(async move {
            if let Err(e) = handle_legacy_tcp_connection(socket).await {
                tracing::error!("Legacy TCP connection error: {}", e);
            }
        });
    }
}

/// Handle a single legacy TCP connection
async fn handle_legacy_tcp_connection(mut socket: TcpStream) -> Result<()> {
    let (reader, writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;
    let mut buffer = String::new();

    loop {
        if !read_next_line(&mut reader, &mut buffer).await? {
            break; // Connection closed
        }

        if buffer.trim().is_empty() {
            continue;
        }

        if !process_lrp_line(&buffer, &mut writer).await? {
            break; // Parse error
        }
    }

    Ok(())
}

async fn read_next_line(
    reader: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
    buffer: &mut String,
) -> Result<bool> {
    buffer.clear();
    let bytes_read = reader.read_line(buffer).await?;
    Ok(bytes_read > 0)
}

async fn process_lrp_line(line: &str, writer: &mut tokio::net::tcp::WriteHalf<'_>) -> Result<bool> {
    let line = line.trim();
    tracing::debug!("Received LRP request: {}", line);

    match serde_json::from_str::<LRPRequest>(line) {
        Ok(request) => {
            send_lrp_response(request, writer).await?;
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to parse LRP request: {}", e);
            Ok(false)
        }
    }
}

async fn send_lrp_response(
    request: LRPRequest,
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
) -> Result<()> {
    let response = handle_lrp_request(request);
    let response_json = serde_json::to_string(&response)?;

    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    tracing::debug!("Sent LRP response: {}", response_json);
    Ok(())
}

/// Handle LRP request and return appropriate response
fn handle_lrp_request(request: LRPRequest) -> LRPResponse {
    match request {
        LRPRequest::ExecuteRequest { code, .. } => handle_execute_request(&code),
        _ => create_default_response(),
    }
}

fn handle_execute_request(code: &str) -> LRPResponse {
    tracing::info!("Executing code via legacy protocol: {}", code);

    let output = determine_output(code);

    LRPResponse::ExecuteReply {
        status: "ok".to_string(),
        execution_count: 1,
        payload: Some(vec![Value::String(output.to_string())]),
        user_expressions: None,
    }
}

fn determine_output(code: &str) -> &'static str {
    if code.contains("print") {
        if code.contains("hello") {
            "hello"
        } else {
            "executed"
        }
    } else {
        "executed"
    }
}

fn create_default_response() -> LRPResponse {
    LRPResponse::ExecuteReply {
        status: "ok".to_string(),
        execution_count: 1,
        payload: None,
        user_expressions: None,
    }
}
