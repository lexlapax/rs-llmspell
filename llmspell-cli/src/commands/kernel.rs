//! ABOUTME: Kernel server command implementation
//! ABOUTME: Starts a Jupyter protocol kernel server for external clients

use crate::cli::ScriptEngine;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{ConnectionInfo, JupyterKernel, JupyterProtocol, ZmqTransport};
use std::path::PathBuf;
use std::sync::Arc;

/// Start the kernel server
pub async fn start_kernel(
    engine: ScriptEngine,
    port: u16,
    id: Option<String>,
    connection_file: Option<PathBuf>,
    config: LLMSpellConfig,
) -> Result<()> {
    println!("Starting LLMSpell kernel server...");
    println!("Engine: {}", engine.as_str());
    println!("Port: {}", port);
    
    // Generate kernel ID
    let kernel_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    println!("Kernel ID: {}", kernel_id);
    
    // Create connection info with simple constructor
    let connection_info = ConnectionInfo::new(
        kernel_id.clone(),
        "127.0.0.1".to_string(),
        port
    );
    
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
    let mut kernel = JupyterKernel::new(
        kernel_id.clone(),
        config,
        transport,
        protocol,
    ).await?;
    
    println!("\nKernel server started successfully!");
    println!("Press Ctrl+C to shutdown\n");
    
    // Run the kernel (blocks until shutdown)
    kernel.serve().await?;
    
    // Cleanup connection file
    let _ = std::fs::remove_file(&conn_file);
    
    Ok(())
}