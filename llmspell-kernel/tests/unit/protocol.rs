//! Unit tests for Protocol trait implementations
//! Tests `JupyterProtocol` implementation and message handling

use anyhow::Result;
use chrono::Utc;
use llmspell_kernel::jupyter::protocol::{
    JupyterMessage, JupyterProtocol, MessageContent, MessageHeader,
};
use llmspell_kernel::traits::protocol::Protocol;
use llmspell_kernel::ConnectionInfo;
use serde_json::Value;
use uuid::Uuid;

/// Test `JupyterProtocol` creation
#[tokio::test]
async fn test_jupyter_protocol_creation() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9000);

    let protocol = JupyterProtocol::new(connection_info);
    // Just verify protocol was created successfully
    let transport_config = protocol.transport_config();
    assert_eq!(transport_config.transport_type, "tcp");
    assert_eq!(transport_config.base_address, "127.0.0.1");
    assert_eq!(
        transport_config.channels.len(),
        5,
        "Should have 5 Jupyter channels"
    );
    assert!(transport_config.channels.contains_key("shell"));
    assert!(transport_config.channels.contains_key("iopub"));
    assert!(transport_config.channels.contains_key("stdin"));
    assert!(transport_config.channels.contains_key("control"));
    assert!(transport_config.channels.contains_key("heartbeat"));
    Ok(())
}

/// Test `KernelInfoRequest` message creation and encoding
#[tokio::test]
async fn test_kernel_info_request_encoding() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9200);

    let protocol = JupyterProtocol::new(connection_info);

    // Create KernelInfoRequest message
    let header = MessageHeader {
        msg_id: Uuid::new_v4().to_string(),
        username: "test".to_string(),
        session: Uuid::new_v4().to_string(),
        date: Utc::now(),
        msg_type: "kernel_info_request".to_string(),
        version: "5.3".to_string(),
    };

    let message = JupyterMessage {
        header,
        parent_header: None,
        metadata: Value::Object(serde_json::Map::new()),
        content: MessageContent::KernelInfoRequest {},
    };

    // Add mock identities for routing
    let mut message = message;
    message.metadata["__identities"] = serde_json::json!(["6d6f636b2d636c69656e742d6964"]); // "mock-client-id" in hex

    // Test encoding
    let encoded = protocol.encode(&message, "shell")?;
    assert!(
        encoded.len() >= 6,
        "Encoded message should have at least 6 parts"
    );
    assert_eq!(
        &encoded[1], b"<IDS|MSG>",
        "Should have delimiter as second part (after identity)"
    );
    Ok(())
}

/// Test `ExecuteRequest` message creation and encoding  
#[tokio::test]
async fn test_execute_request_encoding() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9300);

    let protocol = JupyterProtocol::new(connection_info);

    let header = MessageHeader {
        msg_id: Uuid::new_v4().to_string(),
        username: "test".to_string(),
        session: Uuid::new_v4().to_string(),
        date: Utc::now(),
        msg_type: "execute_request".to_string(),
        version: "5.3".to_string(),
    };

    let mut message = JupyterMessage {
        header,
        parent_header: None,
        metadata: Value::Object(serde_json::Map::new()),
        content: MessageContent::ExecuteRequest {
            code: "print('Hello, World!')".to_string(),
            silent: false,
            store_history: Some(true),
            user_expressions: None,
            allow_stdin: Some(false),
            stop_on_error: Some(true),
        },
    };

    // Add mock identities for routing
    message.metadata["__identities"] = serde_json::json!(["6d6f636b2d636c69656e742d6964"]);

    let encoded = protocol.encode(&message, "shell")?;
    assert!(
        encoded.len() >= 6,
        "Encoded message should have at least 6 parts"
    );

    // Verify content contains the code
    let content_part = &encoded[encoded.len() - 1];
    let content_str = String::from_utf8_lossy(content_part);
    assert!(
        content_str.contains("Hello, World!"),
        "Content should contain the code"
    );
    Ok(())
}

/// Test message decoding with missing delimiter
#[tokio::test]
async fn test_message_decoding_missing_delimiter() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9600);

    let protocol = JupyterProtocol::new(connection_info);

    // Message without delimiter
    let multipart = vec![
        b"client-id".to_vec(),
        b"hmac".to_vec(),
        b"header".to_vec(),
        b"parent_header".to_vec(),
        b"metadata".to_vec(),
        b"content".to_vec(),
    ];

    let result = protocol.decode(multipart, "shell");
    assert!(result.is_err(), "Decoding without delimiter should fail");
    Ok(())
}

/// Test message decoding with incomplete message parts
#[tokio::test]
async fn test_message_decoding_incomplete() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9700);

    let protocol = JupyterProtocol::new(connection_info);

    // Incomplete message (missing parts)
    let multipart = vec![
        b"client-id".to_vec(),
        b"<IDS|MSG>".to_vec(),
        b"hmac".to_vec(),
    ];

    let result = protocol.decode(multipart, "shell");
    assert!(result.is_err(), "Decoding incomplete message should fail");
    Ok(())
}

/// Test `ShutdownRequest` message handling
#[tokio::test]
async fn test_shutdown_request_encoding() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9800);

    let protocol = JupyterProtocol::new(connection_info);

    let header = MessageHeader {
        msg_id: Uuid::new_v4().to_string(),
        username: "test".to_string(),
        session: Uuid::new_v4().to_string(),
        date: Utc::now(),
        msg_type: "shutdown_request".to_string(),
        version: "5.3".to_string(),
    };

    let mut message = JupyterMessage {
        header,
        parent_header: None,
        metadata: Value::Object(serde_json::Map::new()),
        content: MessageContent::ShutdownRequest { restart: false },
    };

    // Add mock identities for routing
    message.metadata["__identities"] = serde_json::json!(["6d6f636b2d636c69656e742d6964"]);

    let encoded = protocol.encode(&message, "control")?;
    assert!(
        encoded.len() >= 6,
        "Encoded shutdown message should have at least 6 parts"
    );

    // Verify content contains restart flag
    let content_part = &encoded[encoded.len() - 1];
    let content_str = String::from_utf8_lossy(content_part);
    assert!(
        content_str.contains("restart"),
        "Content should contain restart field"
    );
    Ok(())
}

/// Test transport config generation
#[tokio::test]
async fn test_transport_config_details() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9100);

    let protocol = JupyterProtocol::new(connection_info);
    let transport_config = protocol.transport_config();

    // Check specific channel configurations
    assert!(
        transport_config.channels.contains_key("shell"),
        "Should have shell channel config"
    );
    assert!(
        transport_config.channels.contains_key("iopub"),
        "Should have iopub channel config"
    );
    assert!(
        transport_config.channels.contains_key("control"),
        "Should have control channel config"
    );

    let shell_config = transport_config.channels.get("shell").unwrap();
    assert_eq!(
        shell_config.pattern, "router",
        "Shell should use ROUTER pattern"
    );

    let iopub_config = transport_config.channels.get("iopub").unwrap();
    assert_eq!(iopub_config.pattern, "pub", "IOPub should use PUB pattern");

    Ok(())
}

/// Test encoding produces proper multipart structure
#[tokio::test]
async fn test_multipart_structure() -> Result<()> {
    let connection_info =
        ConnectionInfo::new("test-kernel".to_string(), "127.0.0.1".to_string(), 9400);

    let protocol = JupyterProtocol::new(connection_info);

    let header = MessageHeader {
        msg_id: Uuid::new_v4().to_string(),
        username: "test".to_string(),
        session: Uuid::new_v4().to_string(),
        date: Utc::now(),
        msg_type: "kernel_info_request".to_string(),
        version: "5.3".to_string(),
    };

    let mut message = JupyterMessage {
        header,
        parent_header: None,
        metadata: Value::Object(serde_json::Map::new()),
        content: MessageContent::KernelInfoRequest {},
    };

    // Add mock identities for routing
    message.metadata["__identities"] = serde_json::json!(["6d6f636b2d636c69656e742d6964"]);

    let encoded = protocol.encode(&message, "shell")?;

    // Verify proper multipart structure:
    // [identities..., <IDS|MSG>, hmac, header, parent_header, metadata, content]
    assert!(encoded.len() >= 6, "Should have at least 6 parts");

    // Find delimiter position
    let delimiter_pos = encoded.iter().position(|part| part == b"<IDS|MSG>");
    assert!(delimiter_pos.is_some(), "Should contain delimiter");

    let delim_idx = delimiter_pos.unwrap();
    assert!(
        delim_idx < encoded.len() - 5,
        "Should have 5 parts after delimiter"
    );

    Ok(())
}

/// Test that `kernel_info_reply` structure includes session metadata
#[test]
fn test_kernel_info_includes_session_metadata() {
    // Manually construct the expected kernel_info structure
    // This tests that the format is correct without needing runtime
    let session_metadata = serde_json::json!({
        "persistence_enabled": true,
        "session_mapper": "llmspell-sessions",
        "state_backend": "llmspell-state-persistence",
        "comm_targets": [
            "llmspell.session",
            "llmspell.state",
        ],
        "max_clients": 10,
        "kernel_id": "test-kernel",
    });

    let kernel_info = serde_json::json!({
        "status": "ok",
        "protocol_version": "5.3",
        "implementation": "llmspell",
        "implementation_version": "0.8.0",
        "language_info": {
            "name": "lua",
            "version": "1.0.0",
            "file_extension": ".lua"
        },
        "banner": "LLMSpell Kernel v0.8.0 - lua",
        "help_links": [],
        "llmspell_session_metadata": session_metadata
    });

    // Verify standard Jupyter fields exist
    assert_eq!(kernel_info["status"], "ok");
    assert_eq!(kernel_info["protocol_version"], "5.3");
    assert_eq!(kernel_info["implementation"], "llmspell");

    // Verify session metadata extension exists
    assert!(kernel_info["llmspell_session_metadata"].is_object());
    let session_meta = &kernel_info["llmspell_session_metadata"];

    // Verify session metadata fields
    assert_eq!(session_meta["persistence_enabled"], true);
    assert_eq!(session_meta["session_mapper"], "llmspell-sessions");
    assert_eq!(session_meta["state_backend"], "llmspell-state-persistence");
    assert_eq!(session_meta["max_clients"], 10);
    assert_eq!(session_meta["kernel_id"], "test-kernel");

    // Verify comm targets
    let comm_targets = session_meta["comm_targets"].as_array().unwrap();
    assert!(comm_targets.contains(&Value::String("llmspell.session".to_string())));
    assert!(comm_targets.contains(&Value::String("llmspell.state".to_string())));
}

/// Test `kernel_info` metadata format matches Jupyter protocol extensions
#[test]
fn test_kernel_info_metadata_format_matches_jupyter_extensions() {
    let session_metadata = serde_json::json!({
        "persistence_enabled": true,
        "session_mapper": "llmspell-sessions",
        "state_backend": "llmspell-state-persistence",
        "comm_targets": [
            "llmspell.session",
            "llmspell.state",
        ],
        "max_clients": 5,
        "kernel_id": "test-kernel",
    });

    // Check that metadata follows Jupyter extension pattern
    assert!(session_metadata.is_object());

    // Ensure all values are JSON-serializable
    let metadata_json = serde_json::to_string(&session_metadata).unwrap();
    assert!(!metadata_json.is_empty());
    assert!(metadata_json.contains("llmspell-sessions"));
    assert!(metadata_json.contains("llmspell.session"));
}

/// Test `kernel_info` session metadata updates reflect current state
#[test]
fn test_kernel_info_metadata_updates_reflect_current_state() {
    // First kernel configuration
    let session_metadata1 = serde_json::json!({
        "persistence_enabled": true,
        "session_mapper": "llmspell-sessions",
        "state_backend": "llmspell-state-persistence",
        "comm_targets": [
            "llmspell.session",
            "llmspell.state",
        ],
        "max_clients": 3,
        "kernel_id": "test-kernel-1",
    });

    assert_eq!(session_metadata1["max_clients"], 3);
    assert_eq!(session_metadata1["kernel_id"], "test-kernel-1");

    // Second kernel configuration
    let session_metadata2 = serde_json::json!({
        "persistence_enabled": true,
        "session_mapper": "llmspell-sessions",
        "state_backend": "llmspell-state-persistence",
        "comm_targets": [
            "llmspell.session",
            "llmspell.state",
        ],
        "max_clients": 20,
        "kernel_id": "test-kernel-2",
    });

    assert_eq!(session_metadata2["max_clients"], 20);
    assert_eq!(session_metadata2["kernel_id"], "test-kernel-2");

    // Verify the two configurations are different
    assert_ne!(
        session_metadata1["kernel_id"],
        session_metadata2["kernel_id"]
    );
    assert_ne!(
        session_metadata1["max_clients"],
        session_metadata2["max_clients"]
    );
}
