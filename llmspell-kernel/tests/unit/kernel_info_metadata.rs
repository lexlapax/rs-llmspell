//! Test that kernel_info_reply includes session metadata
use llmspell_kernel::{GenericKernel, KernelConfig, NullProtocol, NullTransport};
use llmspell_config::LLMSpellConfig;
use serde_json::Value;

#[tokio::test]
async fn test_kernel_info_includes_session_metadata() {
    // Create kernel with null transport/protocol for testing
    let config = KernelConfig {
        kernel_id: Some("test-kernel".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 10,
        auth_enabled: false,
    };
    
    let transport = NullTransport;
    let protocol = NullProtocol;
    
    let kernel = GenericKernel::new(transport, protocol, config)
        .await
        .expect("Failed to create kernel");
    
    // Get kernel info
    let info = kernel.handle_kernel_info();
    
    // Verify standard Jupyter fields exist
    assert_eq!(info["status"], "ok");
    assert_eq!(info["protocol_version"], "5.3");
    assert_eq!(info["implementation"], "llmspell");
    
    // Verify language info
    assert_eq!(info["language_info"]["name"], "lua");
    assert_eq!(info["language_info"]["file_extension"], ".lua");
    
    // Verify session metadata extension exists
    assert!(info["llmspell_session_metadata"].is_object());
    let session_meta = &info["llmspell_session_metadata"];
    
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

#[tokio::test]
async fn test_kernel_info_metadata_format_matches_jupyter_extensions() {
    let config = KernelConfig {
        kernel_id: Some("test-kernel".to_string()),
        engine: "javascript".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 5,
        auth_enabled: true,
    };
    
    let kernel = GenericKernel::new(NullTransport, NullProtocol, config)
        .await
        .expect("Failed to create kernel");
    
    let info = kernel.handle_kernel_info();
    
    // Check that metadata follows Jupyter extension pattern
    assert!(info["llmspell_session_metadata"].is_object());
    
    // Ensure all values are JSON-serializable
    let metadata_json = serde_json::to_string(&info["llmspell_session_metadata"])
        .expect("Session metadata should be JSON-serializable");
    assert!(!metadata_json.is_empty());
}

#[tokio::test]
async fn test_kernel_info_metadata_updates_reflect_current_state() {
    let mut config = KernelConfig {
        kernel_id: Some("test-kernel-1".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 3,
        auth_enabled: false,
    };
    
    let kernel1 = GenericKernel::new(NullTransport, NullProtocol, config.clone())
        .await
        .expect("Failed to create kernel");
    
    let info1 = kernel1.handle_kernel_info();
    assert_eq!(info1["llmspell_session_metadata"]["max_clients"], 3);
    assert_eq!(info1["llmspell_session_metadata"]["kernel_id"], "test-kernel-1");
    
    // Create another kernel with different config
    config.kernel_id = Some("test-kernel-2".to_string());
    config.max_clients = 20;
    
    let kernel2 = GenericKernel::new(NullTransport, NullProtocol, config)
        .await
        .expect("Failed to create kernel");
    
    let info2 = kernel2.handle_kernel_info();
    assert_eq!(info2["llmspell_session_metadata"]["max_clients"], 20);
    assert_eq!(info2["llmspell_session_metadata"]["kernel_id"], "test-kernel-2");
}