//! Test multiple requests on the same TCP connection

use llmspell_engine::{LRPRequest, LRPResponse, ProtocolClient};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
#[ignore = "Requires running kernel"]
async fn test_multiple_requests_same_connection() -> Result<(), Box<dyn std::error::Error>> {
    let kernel_addr = "127.0.0.1:9555";

    println!("Connecting to kernel at {kernel_addr}");
    let client = ProtocolClient::connect(kernel_addr).await?;
    println!("✓ Connected!");

    // Send 5 kernel info requests in a row
    for i in 1..=5 {
        println!("\nRequest {i}: Sending kernel info request...");

        let response = timeout(
            Duration::from_secs(5),
            client.send_lrp_request(LRPRequest::KernelInfoRequest),
        )
        .await??;

        if let LRPResponse::KernelInfoReply { implementation, .. } = response {
            println!("✓ Response {i}: Got reply from {implementation}");
        } else {
            println!("✗ Request {i} failed");
            return Err("Unexpected response".into());
        }

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!("\n✓ All 5 requests succeeded on the same connection!");
    Ok(())
}
