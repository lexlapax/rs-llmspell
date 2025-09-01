use llmspell_engine::{LRPRequest, LRPResponse, ProtocolClient};

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ProtocolClient::connect("127.0.0.1:9555").await?;
    println!("Connected");

    // Request 1
    println!("Sending KernelInfoRequest...");
    let _response = client.send_lrp_request(LRPRequest::KernelInfoRequest).await?;
    println!("Got KernelInfoResponse");

    // Request 2  
    println!("Sending ExecuteRequest...");
    let _response = client.send_lrp_request(LRPRequest::ExecuteRequest {
        code: "return 42".to_string(),
        silent: false,
        store_history: true,
        user_expressions: None,
        allow_stdin: false, 
        stop_on_error: true,
    }).await?;
    println!("Got ExecuteResponse");

    Ok(())
}