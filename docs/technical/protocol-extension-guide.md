# Protocol Extension Guide

## Adding New Protocols to UnifiedProtocolEngine

This guide provides step-by-step instructions for extending the UnifiedProtocolEngine to support new protocols like MCP (Model Context Protocol), LSP (Language Server Protocol), DAP (Debug Adapter Protocol), and A2A (Agent-to-Agent).

## Step 1: Define Protocol Types

Add your new protocol to the `ProtocolType` enum:

```rust
// In llmspell-engine/src/engine.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    LRP, // Lua REPL Protocol
    LDP, // Lua Debug Protocol  
    MCP, // Model Context Protocol
    LSP, // Language Server Protocol
    DAP, // Debug Adapter Protocol
    A2A, // Agent-to-Agent Protocol
}
```

## Step 2: Define Protocol Messages

Create protocol-specific message types in the protocol module:

```rust
// In llmspell-engine/src/protocol/mod.rs
pub mod mcp;  // New module

// In llmspell-engine/src/protocol/mcp.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPRequest {
    Initialize { 
        capabilities: MCPCapabilities,
        client_info: ClientInfo,
    },
    ListModels,
    GenerateText { 
        model: String, 
        prompt: String, 
        parameters: GenerationParameters,
    },
    // Add more MCP-specific requests
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPResponse {
    Initialize(MCPCapabilities),
    ModelList(Vec<ModelInfo>),
    TextGeneration(GenerationResult),
    Error { code: i32, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCapabilities {
    pub text_generation: bool,
    pub model_selection: bool,
    pub streaming: bool,
}
```

## Step 3: Extend UniversalMessage

Update the `MessageContent` enum to handle your protocol:

```rust
// In llmspell-engine/src/engine.rs
#[derive(Debug, Clone, PartialEq)]
pub enum MessageContent {
    // Existing
    Request { method: String, params: serde_json::Value },
    Response { data: serde_json::Value },
    Notification { event: String, data: serde_json::Value },
    Error { code: i32, message: String },
    
    // New protocol-specific variants
    MCPRequest(MCPRequest),
    MCPResponse(MCPResponse),
}
```

## Step 4: Create Protocol Adapter

Implement conversion between your protocol and UniversalMessage:

```rust
// In llmspell-engine/src/engine.rs
impl UnifiedProtocolEngine {
    fn convert_mcp_to_universal(&self, msg: ProtocolMessage) -> Result<UniversalMessage> {
        let content = match serde_json::from_value::<MCPRequest>(msg.content)? {
            MCPRequest::Initialize { capabilities, client_info } => {
                MessageContent::MCPRequest(MCPRequest::Initialize { capabilities, client_info })
            },
            MCPRequest::ListModels => {
                MessageContent::MCPRequest(MCPRequest::ListModels)
            },
            MCPRequest::GenerateText { model, prompt, parameters } => {
                MessageContent::MCPRequest(MCPRequest::GenerateText { model, prompt, parameters })
            },
        };

        Ok(UniversalMessage {
            id: msg.msg_id,
            protocol: ProtocolType::MCP,
            channel: self.determine_channel(&msg)?,
            content,
            metadata: HashMap::new(),
        })
    }
    
    fn convert_universal_to_mcp(&self, msg: UniversalMessage) -> Result<ProtocolMessage> {
        let content = match msg.content {
            MessageContent::MCPResponse(response) => serde_json::to_value(response)?,
            MessageContent::Error { code, message } => {
                serde_json::to_value(MCPResponse::Error { code, message })?
            },
            _ => return Err(EngineError::InvalidMessageContent),
        };

        Ok(ProtocolMessage {
            msg_id: msg.id,
            msg_type: MessageType::Response,
            channel: msg.channel.to_string(),
            content,
        })
    }
}
```

## Step 5: Register Protocol Handler

Update the protocol detection and routing logic:

```rust
// In llmspell-engine/src/engine.rs
impl UnifiedProtocolEngine {
    async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        // Protocol detection (could be enhanced with handshake)
        let protocol = self.detect_protocol(&stream).await?;
        
        match protocol {
            ProtocolType::LRP => self.handle_lrp_connection(stream).await,
            ProtocolType::LDP => self.handle_ldp_connection(stream).await,
            ProtocolType::MCP => self.handle_mcp_connection(stream).await, // New
            // Add other protocols...
        }
    }
    
    async fn handle_mcp_connection(&self, stream: TcpStream) -> Result<()> {
        let mut framed = Framed::new(stream, LRPCodec::new());
        
        while let Some(frame) = framed.next().await {
            let msg = frame?;
            let universal = self.convert_mcp_to_universal(msg)?;
            
            // Route through MessageProcessor
            let response = self.processor.process_message(universal).await?;
            let mcp_response = self.convert_universal_to_mcp(response)?;
            
            framed.send(mcp_response).await?;
        }
        
        Ok(())
    }
}
```

## Step 6: Implement Business Logic

Extend your MessageProcessor to handle the new protocol:

```rust
// In your kernel/processor implementation
impl MessageProcessor for KernelMessageProcessor {
    async fn process_message(&self, msg: UniversalMessage) -> Result<UniversalMessage> {
        match (msg.protocol, msg.content) {
            // Existing LRP/LDP handling...
            
            (ProtocolType::MCP, MessageContent::MCPRequest(req)) => {
                let response = self.handle_mcp_request(req).await?;
                Ok(UniversalMessage {
                    id: msg.id,
                    protocol: ProtocolType::MCP,
                    channel: msg.channel,
                    content: MessageContent::MCPResponse(response),
                    metadata: msg.metadata,
                })
            },
            
            _ => Err(ProcessorError::UnsupportedMessage),
        }
    }
    
    async fn handle_mcp_request(&self, req: MCPRequest) -> Result<MCPResponse> {
        match req {
            MCPRequest::Initialize { capabilities, client_info } => {
                // Initialize MCP session
                let server_capabilities = self.get_mcp_capabilities();
                Ok(MCPResponse::Initialize(server_capabilities))
            },
            
            MCPRequest::ListModels => {
                let models = self.get_available_models().await?;
                Ok(MCPResponse::ModelList(models))
            },
            
            MCPRequest::GenerateText { model, prompt, parameters } => {
                let result = self.generate_text(model, prompt, parameters).await?;
                Ok(MCPResponse::TextGeneration(result))
            },
        }
    }
}
```

## Step 7: Update Channel Routing

Configure appropriate routing strategy for your protocol:

```rust
// In your engine initialization
impl UnifiedProtocolEngine {
    pub fn new() -> Self {
        let mut engine = Self { /* ... */ };
        
        // Configure routing strategies
        engine.router.set_strategy(ChannelType::Shell, RoutingStrategy::Direct);
        engine.router.set_strategy(ChannelType::IOPub, RoutingStrategy::Broadcast);
        
        // MCP might use load balancing for model requests
        engine.router.set_strategy(ChannelType::Custom("mcp".to_string()), 
                                 RoutingStrategy::LoadBalanced);
        
        engine
    }
}
```

## Step 8: Add Protocol Tests

Create comprehensive tests for your protocol integration:

```rust
// In llmspell-engine/tests/mcp_integration_test.rs
#[tokio::test]
async fn test_mcp_initialize_handshake() {
    let engine = UnifiedProtocolEngine::new();
    
    let init_msg = UniversalMessage {
        id: "test-1".to_string(),
        protocol: ProtocolType::MCP,
        channel: ChannelType::Shell,
        content: MessageContent::MCPRequest(MCPRequest::Initialize {
            capabilities: MCPCapabilities::default(),
            client_info: ClientInfo::default(),
        }),
        metadata: HashMap::new(),
    };
    
    let response = engine.process_message(init_msg).await.unwrap();
    
    match response.content {
        MessageContent::MCPResponse(MCPResponse::Initialize(caps)) => {
            assert!(caps.text_generation);
            // Assert other capabilities
        },
        _ => panic!("Expected Initialize response"),
    }
}

#[tokio::test]
async fn test_mcp_text_generation() {
    // Test text generation request/response cycle
}

#[tokio::test] 
async fn test_mcp_error_handling() {
    // Test error scenarios and recovery
}
```

## Step 9: Update Documentation

Add protocol-specific documentation:

```rust
// In your protocol module
/// Model Context Protocol (MCP) implementation
/// 
/// MCP enables communication with AI models for text generation,
/// completion, and other AI-powered capabilities.
/// 
/// # Channel Usage
/// - Shell: Model requests and responses
/// - IOPub: Model training/fine-tuning progress
/// - Control: Model lifecycle management
/// 
/// # Message Flow
/// ```
/// Client → Initialize → Server
/// Client → ListModels → Server  
/// Client → GenerateText → Server
/// ```
```

## Best Practices

### Protocol Design
1. **Use typed messages**: Define clear request/response types
2. **Version compatibility**: Include protocol version in handshake
3. **Error handling**: Define comprehensive error codes and messages
4. **Async operations**: Support streaming/long-running operations

### Performance Considerations
1. **Message size**: Keep messages reasonably sized for network efficiency
2. **Connection reuse**: Design for persistent connections where possible
3. **Batching**: Support batch operations for related requests
4. **Caching**: Cache frequently requested data (models, capabilities)

### Testing Strategy
1. **Unit tests**: Test message conversion and business logic
2. **Integration tests**: Test full protocol flows
3. **Load tests**: Verify performance under concurrent load
4. **Compatibility tests**: Test with real protocol clients

### Error Recovery
1. **Graceful degradation**: Handle partial failures gracefully
2. **Retry logic**: Implement exponential backoff for transient errors
3. **Circuit breakers**: Prevent cascading failures
4. **Monitoring**: Add metrics and logging for observability

## Example: Complete LSP Integration

Here's a abbreviated example of adding Language Server Protocol support:

```rust
// Protocol definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LSPRequest {
    Initialize(InitializeParams),
    TextDocumentDidOpen(DidOpenTextDocumentParams),
    TextDocumentCompletion(CompletionParams),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)] 
pub enum LSPResponse {
    Initialize(InitializeResult),
    Completion(CompletionList),
    Error { code: i32, message: String },
}

// Business logic implementation
impl KernelMessageProcessor {
    async fn handle_lsp_request(&self, req: LSPRequest) -> Result<LSPResponse> {
        match req {
            LSPRequest::Initialize(params) => {
                let capabilities = ServerCapabilities {
                    completion_provider: Some(CompletionOptions::default()),
                    // Configure other capabilities
                };
                Ok(LSPResponse::Initialize(InitializeResult { capabilities }))
            },
            
            LSPRequest::TextDocumentCompletion(params) => {
                let completions = self.get_completions(params).await?;
                Ok(LSPResponse::Completion(completions))
            },
            
            // Handle other LSP requests...
        }
    }
}
```

This guide provides the foundation for extending the UnifiedProtocolEngine with any new protocol. The key is maintaining the separation between protocol handling (adapters) and business logic (MessageProcessor) while leveraging the unified routing and channel abstraction.