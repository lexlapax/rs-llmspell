//! Integration test for TCP communication with kernel

use llmspell_protocol::{LRPRequest, LRPResponse, ProtocolClient};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
#[ignore] // Run with: cargo test -p llmspell-protocol --test kernel_tcp_integration -- --ignored --nocapture
async fn test_kernel_tcp_communication() -> Result<(), Box<dyn std::error::Error>> {
    // For now, hardcode the kernel address
    // In production, this would be read from the connection file
    let kernel_addr = "127.0.0.1:9555";
    
    println!("Connecting to kernel at {}", kernel_addr);
    
    // Create client and connect
    let client = match timeout(
        Duration::from_secs(5), 
        ProtocolClient::connect(kernel_addr)
    ).await {
        Ok(Ok(client)) => client,
        Ok(Err(e)) => {
            eprintln!("Failed to connect to kernel: {}", e);
            eprintln!("Please start a kernel first with: cargo run --bin llmspell-kernel");
            return Ok(()); // Skip test if no kernel is running
        }
        Err(_) => {
            eprintln!("Connection timeout - no kernel running");
            eprintln!("Please start a kernel first with: cargo run --bin llmspell-kernel");
            return Ok(()); // Skip test if timeout
        }
    };
    println!("✓ Connected to kernel!");
    
    // Test 1: Kernel info request
    println!("\nTest 1: Sending kernel info request...");
    let response = timeout(
        Duration::from_secs(5),
        client.send_lrp_request(LRPRequest::KernelInfoRequest)
    ).await??;
    
    match response {
        LRPResponse::KernelInfoReply { implementation, language_info, .. } => {
            println!("✓ Kernel info received:");
            println!("  Implementation: {}", implementation);
            println!("  Language: {}", language_info.name);
        }
        _ => {
            println!("✗ Unexpected response: {:?}", response);
            return Err("Expected KernelInfoReply".into());
        }
    }
    
    // Test 2: Execute request
    println!("\nTest 2: Sending execute request...");
    let response = timeout(
        Duration::from_secs(5),
        client.send_lrp_request(LRPRequest::ExecuteRequest {
            code: "return 'Hello from kernel TCP connection!'".to_string(),
            silent: false,
            store_history: true,
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        })
    ).await??;
    
    match response {
        LRPResponse::ExecuteReply { status, execution_count, .. } => {
            println!("✓ Execute response received:");
            println!("  Status: {}", status);
            println!("  Execution count: {}", execution_count);
        }
        _ => {
            println!("✗ Unexpected response: {:?}", response);
            return Err("Expected ExecuteReply".into());
        }
    }
    
    // Test 3: Complete request
    println!("\nTest 3: Sending complete request...");
    let response = timeout(
        Duration::from_secs(5),
        client.send_lrp_request(LRPRequest::CompleteRequest {
            code: "pri".to_string(),
            cursor_pos: 3,
        })
    ).await??;
    
    match response {
        LRPResponse::CompleteReply { matches, .. } => {
            println!("✓ Complete response received:");
            println!("  Matches: {:?}", matches);
        }
        _ => {
            println!("✗ Unexpected response: {:?}", response);
            return Err("Expected CompleteReply".into());
        }
    }
    
    println!("\n✅ All TCP communication tests passed!");
    
    Ok(())
}