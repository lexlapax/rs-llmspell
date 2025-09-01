use llmspell_engine::{LRPRequest, LRPResponse, ProtocolClient};
use std::time::Duration;
use tokio::time::timeout;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let kernel_addr = "127.0.0.1:9555";
    debug!("Connecting to kernel at {}", kernel_addr);

    let client = ProtocolClient::connect(kernel_addr).await?;
    debug!("Connected to kernel");

    // Test 1: KernelInfoRequest
    debug!("Sending KernelInfoRequest");
    let response = timeout(
        Duration::from_secs(5),
        client.send_lrp_request(LRPRequest::KernelInfoRequest),
    ).await??;
    debug!("Got KernelInfoResponse: {:?}", response);

    // Test 2: ExecuteRequest  
    debug!("Sending ExecuteRequest");
    let response = timeout(
        Duration::from_secs(5),
        client.send_lrp_request(LRPRequest::ExecuteRequest {
            code: "return 42".to_string(),
            silent: false,
            store_history: true, 
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        }),
    ).await??;
    debug!("Got ExecuteResponse: {:?}", response);

    Ok(())
}