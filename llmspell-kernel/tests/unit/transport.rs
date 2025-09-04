//! Unit tests for Transport trait implementations
//! Tests `ZmqTransport` implementation and transport layer functionality

use anyhow::Result;
use llmspell_kernel::traits::transport::{ChannelConfig, Transport, TransportConfig};
use llmspell_kernel::transport::zeromq::ZmqTransport;
use std::collections::HashMap;

/// Test `ZmqTransport` creation and basic initialization
#[tokio::test]
async fn test_zmq_transport_creation() -> Result<()> {
    let transport = ZmqTransport::new()?;
    // ZmqTransport should be created successfully
    assert!(
        transport.channels().is_empty(),
        "New transport should have no channels initially"
    );
    Ok(())
}

/// Test transport binding with valid configuration
#[tokio::test]
async fn test_transport_bind_valid_config() -> Result<()> {
    let mut transport = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([
            (
                "shell".to_string(),
                ChannelConfig {
                    endpoint: "9001".to_string(),
                    pattern: "router".to_string(),
                },
            ),
            (
                "iopub".to_string(),
                ChannelConfig {
                    endpoint: "9002".to_string(),
                    pattern: "pub".to_string(),
                },
            ),
        ]),
    };

    let result = transport.bind(&config).await;
    assert!(
        result.is_ok(),
        "Transport should bind successfully with valid config"
    );

    // Verify channels are available
    assert!(transport.has_channel("shell"), "Should have shell channel");
    assert!(transport.has_channel("iopub"), "Should have iopub channel");
    assert_eq!(transport.channels().len(), 2, "Should have 2 channels");
    Ok(())
}

/// Test transport binding with port conflict
#[tokio::test]
async fn test_transport_bind_port_conflict() -> Result<()> {
    let mut transport1 = ZmqTransport::new()?;
    let mut transport2 = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([(
            "shell".to_string(),
            ChannelConfig {
                endpoint: "9101".to_string(),
                pattern: "router".to_string(),
            },
        )]),
    };

    // First bind should succeed
    transport1.bind(&config).await?;

    // Second bind to same port should fail
    let result = transport2.bind(&config).await;
    assert!(result.is_err(), "Second bind to same port should fail");
    Ok(())
}

/// Test heartbeat functionality
#[tokio::test]
async fn test_heartbeat() -> Result<()> {
    let mut transport = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([(
            "heartbeat".to_string(),
            ChannelConfig {
                endpoint: "9301".to_string(),
                pattern: "rep".to_string(),
            },
        )]),
    };

    transport.bind(&config).await?;

    // Test heartbeat - should return false when no heartbeat message received
    let heartbeat_result = transport.heartbeat().await;
    assert!(heartbeat_result.is_ok(), "Heartbeat should not error");
    assert!(
        !heartbeat_result.unwrap(),
        "Heartbeat should return false when no message"
    );
    Ok(())
}

/// Test receiving from non-existent channel
#[tokio::test]
async fn test_recv_invalid_channel() -> Result<()> {
    let mut transport = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([(
            "shell".to_string(),
            ChannelConfig {
                endpoint: "9401".to_string(),
                pattern: "router".to_string(),
            },
        )]),
    };

    transport.bind(&config).await?;

    // Try to receive from non-existent channel
    let result = transport.recv("nonexistent").await;
    assert!(
        result.is_err(),
        "Receiving from invalid channel should fail"
    );
    Ok(())
}

/// Test send to non-existent channel
#[tokio::test]
async fn test_send_invalid_channel() -> Result<()> {
    let mut transport = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([(
            "shell".to_string(),
            ChannelConfig {
                endpoint: "9501".to_string(),
                pattern: "router".to_string(),
            },
        )]),
    };

    transport.bind(&config).await?;

    // Try to send to non-existent channel
    let test_message = vec![b"test".to_vec()];
    let result = transport.send("nonexistent", test_message).await;
    assert!(result.is_err(), "Sending to invalid channel should fail");
    Ok(())
}

/// Test channel existence checks
#[tokio::test]
async fn test_channel_existence() -> Result<()> {
    let mut transport = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([
            (
                "shell".to_string(),
                ChannelConfig {
                    endpoint: "9601".to_string(),
                    pattern: "router".to_string(),
                },
            ),
            (
                "iopub".to_string(),
                ChannelConfig {
                    endpoint: "9602".to_string(),
                    pattern: "pub".to_string(),
                },
            ),
        ]),
    };

    transport.bind(&config).await?;

    // Test channel existence
    assert!(transport.has_channel("shell"), "Should have shell channel");
    assert!(transport.has_channel("iopub"), "Should have iopub channel");
    assert!(
        !transport.has_channel("nonexistent"),
        "Should not have nonexistent channel"
    );

    let channels = transport.channels();
    assert_eq!(channels.len(), 2, "Should report 2 channels");
    assert!(
        channels.contains(&"shell".to_string()),
        "Should include shell"
    );
    assert!(
        channels.contains(&"iopub".to_string()),
        "Should include iopub"
    );
    Ok(())
}

/// Test recv returns None when no message available
#[tokio::test]
async fn test_recv_no_message() -> Result<()> {
    let mut transport = ZmqTransport::new()?;

    let config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: HashMap::from([(
            "shell".to_string(),
            ChannelConfig {
                endpoint: "9701".to_string(),
                pattern: "router".to_string(),
            },
        )]),
    };

    transport.bind(&config).await?;

    // Try to receive - should return None since no message available
    let result = transport.recv("shell").await?;
    assert!(
        result.is_none(),
        "Should return None when no message available"
    );
    Ok(())
}
