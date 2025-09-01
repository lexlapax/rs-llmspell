#!/usr/bin/env cargo +nightly -Zscript
//! ```cargo
//! [dependencies]
//! llmspell-protocol = { path = "llmspell-protocol" }
//! llmspell-repl = { path = "llmspell-repl" }
//! anyhow = "1"
//! tokio = { version = "1", features = ["full"] }
//! ```

use llmspell_protocol::{ProtocolClient, LRPRequest, ProtocolMessage};
use llmspell_repl::connection::ConnectionInfo;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Read connection file
    let connection_info = ConnectionInfo::read_connection_file(
        "3f407065-30f9-41c1-9bef-9df7774ef7f8"
    ).await?;
    
    println!("Connecting to kernel at {}:{}", 
        connection_info.ip, connection_info.shell_port);
    
    // Create protocol client
    let mut client = ProtocolClient::new(
        format!("{}:{}", connection_info.ip, connection_info.shell_port)
    );
    
    // Connect
    client.connect().await?;
    println!("✓ Connected to kernel!");
    
    // Send kernel info request
    println!("\nSending kernel info request...");
    let request = LRPRequest::KernelInfoRequest;
    let response = client.send_request(
        ProtocolMessage::LRP(request)
    ).await?;
    
    println!("✓ Kernel info response: {:?}", response);
    
    // Send execute request
    println!("\nSending execute request...");
    let request = LRPRequest::ExecuteRequest {
        code: "return 'Hello from kernel TCP connection!'".to_string(),
        silent: false,
        store_history: true,
        user_expressions: None,
        allow_stdin: false,
        stop_on_error: true,
    };
    
    let response = client.send_request(
        ProtocolMessage::LRP(request)
    ).await?;
    
    println!("✓ Execute response: {:?}", response);
    
    println!("\n✅ TCP communication verified successfully!");
    
    Ok(())
}