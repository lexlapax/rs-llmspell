# Model Control Protocol (MCP) Support Research

## Overview

This document researches the integration of Model Control Protocol (MCP) support into rs-llmspell, enabling agents to consume external MCP tools and expose rs-llmspell tools as MCP servers. MCP provides a standardized way for AI agents to securely access tools and resources across different systems.

## MCP Protocol Overview

### Core Concepts

**Model Control Protocol (MCP)** is a standard protocol that enables AI models and agents to securely interact with external tools, databases, and services. Key characteristics:

- **Client-Server Architecture**: Agents act as MCP clients, tools/services act as MCP servers
- **Capability Discovery**: Dynamic discovery of available tools and their schemas
- **Security**: Built-in authentication, authorization, and sandboxing
- **Transport Agnostic**: Works over HTTP, WebSocket, stdio, or custom transports
- **Schema-Driven**: Tools expose JSON schema definitions for type safety

### MCP Message Flow

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   MCP Client    │       │  MCP Server     │       │   External      │
│  (rs-llmspell)  │       │  (Tool Host)    │       │   Resource      │
└─────────────────┘       └─────────────────┘       └─────────────────┘
         │                          │                          │
         │  1. Connection Setup     │                          │
         │─────────────────────────→│                          │
         │                          │                          │
         │  2. Capability Discovery │                          │
         │─────────────────────────→│                          │
         │  ←─────────────────────── │                          │
         │                          │                          │
         │  3. Tool Invocation      │                          │
         │─────────────────────────→│  ──────────────────────→ │
         │                          │  ←────────────────────── │
         │  ←─────────────────────── │                          │
         │                          │                          │
```

## MCP Client Support in Rs-LLMSpell

### Architecture for MCP Client Integration

**Goal**: Enable rs-llmspell agents to discover and use external MCP tools seamlessly.

```rust
// Core MCP client traits and types
pub trait MCPClient: Send + Sync {
    async fn connect(&mut self, server_uri: &str) -> Result<MCPConnection>;
    async fn discover_capabilities(&self, connection: &MCPConnection) -> Result<MCPCapabilities>;
    async fn list_tools(&self, connection: &MCPConnection) -> Result<Vec<MCPToolDescription>>;
    async fn invoke_tool(&self, connection: &MCPConnection, request: MCPToolRequest) -> Result<MCPToolResponse>;
    async fn disconnect(&mut self, connection: MCPConnection) -> Result<()>;
}

pub struct MCPConnection {
    session_id: String,
    transport: Box<dyn MCPTransport>,
    capabilities: MCPCapabilities,
    auth_context: Option<AuthContext>,
}

pub struct MCPCapabilities {
    supported_transports: Vec<TransportType>,
    supported_auth_methods: Vec<AuthMethod>,
    tool_discovery: bool,
    streaming_support: bool,
    batch_operations: bool,
    resource_subscriptions: bool,
}

pub struct MCPToolDescription {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
    required_permissions: Vec<Permission>,
    rate_limits: Option<RateLimits>,
    cost_info: Option<CostInfo>,
}

// Bridge MCP tools into rs-llmspell tool system
pub struct MCPToolAdapter {
    mcp_client: Box<dyn MCPClient>,
    connection: MCPConnection,
    tool_description: MCPToolDescription,
    performance_monitor: ToolPerformanceMonitor,
}

#[async_trait]
impl Tool for MCPToolAdapter {
    fn name(&self) -> &str {
        &self.tool_description.name
    }
    
    fn description(&self) -> &str {
        &self.tool_description.description
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        self.tool_description.input_schema.clone()
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolOutput> {
        // Convert rs-llmspell tool params to MCP format
        let mcp_request = MCPToolRequest {
            tool_name: self.tool_description.name.clone(),
            parameters: params,
            context: MCPExecutionContext {
                session_id: self.connection.session_id.clone(),
                request_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
            },
        };
        
        // Execute via MCP client
        let start_time = Instant::now();
        let mcp_response = self.mcp_client.invoke_tool(&self.connection, mcp_request).await?;
        let duration = start_time.elapsed();
        
        // Record performance metrics
        self.performance_monitor.record_execution(&self.tool_description.name, duration, &mcp_response);
        
        // Convert MCP response to rs-llmspell format
        Ok(ToolOutput {
            content: mcp_response.result,
            metadata: HashMap::from([
                ("mcp_session_id".to_string(), Value::String(self.connection.session_id.clone())),
                ("mcp_server_uri".to_string(), Value::String(mcp_response.server_info.uri)),
                ("execution_time_ms".to_string(), Value::Number(duration.as_millis().into())),
                ("mcp_version".to_string(), Value::String(mcp_response.protocol_version)),
            ]),
        })
    }
}

// MCP client manager for handling multiple connections
pub struct MCPClientManager {
    clients: HashMap<String, Box<dyn MCPClient>>,
    connections: HashMap<String, MCPConnection>,
    discovery_cache: Arc<RwLock<HashMap<String, Vec<MCPToolDescription>>>>,
    connection_pool: MCPConnectionPool,
    security_manager: MCPSecurityManager,
}

impl MCPClientManager {
    pub async fn register_mcp_server(&mut self, server_config: MCPServerConfig) -> Result<String> {
        let server_id = format!("mcp_server_{}", Uuid::new_v4());
        
        // Create client for this server
        let mut client = match server_config.transport {
            TransportType::HTTP => Box::new(HTTPMCPClient::new(server_config.clone())),
            TransportType::WebSocket => Box::new(WebSocketMCPClient::new(server_config.clone())),
            TransportType::Stdio => Box::new(StdioMCPClient::new(server_config.clone())),
        };
        
        // Establish connection
        let connection = client.connect(&server_config.uri).await?;
        
        // Discover available tools
        let tools = client.list_tools(&connection).await?;
        
        // Cache discovered tools
        {
            let mut cache = self.discovery_cache.write().await;
            cache.insert(server_id.clone(), tools.clone());
        }
        
        // Store client and connection
        self.clients.insert(server_id.clone(), client);
        self.connections.insert(server_id.clone(), connection);
        
        info!("Registered MCP server {} with {} tools", server_id, tools.len());
        Ok(server_id)
    }
    
    pub async fn create_tool_adapters(&self, server_id: &str) -> Result<Vec<Box<dyn Tool>>> {
        let tools = {
            let cache = self.discovery_cache.read().await;
            cache.get(server_id)
                .ok_or_else(|| anyhow!("Server not found: {}", server_id))?
                .clone()
        };
        
        let client = self.clients.get(server_id)
            .ok_or_else(|| anyhow!("Client not found: {}", server_id))?;
            
        let connection = self.connections.get(server_id)
            .ok_or_else(|| anyhow!("Connection not found: {}", server_id))?;
        
        let mut adapters: Vec<Box<dyn Tool>> = Vec::new();
        
        for tool_desc in tools {
            let adapter = MCPToolAdapter {
                mcp_client: dyn_clone::clone_box(&**client),
                connection: connection.clone(),
                tool_description: tool_desc,
                performance_monitor: ToolPerformanceMonitor::new(),
            };
            
            adapters.push(Box::new(adapter));
        }
        
        Ok(adapters)
    }
}

// Transport implementations
#[async_trait]
pub trait MCPTransport: Send + Sync {
    async fn send_message(&mut self, message: MCPMessage) -> Result<()>;
    async fn receive_message(&mut self) -> Result<MCPMessage>;
    async fn close(&mut self) -> Result<()>;
}

pub struct HTTPMCPClient {
    http_client: reqwest::Client,
    base_url: String,
    auth_header: Option<String>,
}

#[async_trait]
impl MCPClient for HTTPMCPClient {
    async fn connect(&mut self, server_uri: &str) -> Result<MCPConnection> {
        let connection_request = MCPConnectionRequest {
            protocol_version: "1.0".to_string(),
            client_info: MCPClientInfo {
                name: "rs-llmspell".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: MCPClientCapabilities {
                tool_invocation: true,
                resource_subscriptions: false,
                batch_operations: true,
            },
        };
        
        let response = self.http_client
            .post(&format!("{}/mcp/connect", server_uri))
            .json(&connection_request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow!("MCP connection failed: {}", response.status()));
        }
        
        let connection_response: MCPConnectionResponse = response.json().await?;
        
        Ok(MCPConnection {
            session_id: connection_response.session_id,
            transport: Box::new(HTTPTransport::new(self.http_client.clone(), server_uri.to_string())),
            capabilities: connection_response.server_capabilities,
            auth_context: None,
        })
    }
    
    async fn invoke_tool(&self, connection: &MCPConnection, request: MCPToolRequest) -> Result<MCPToolResponse> {
        let url = format!("{}/mcp/tools/{}/invoke", self.base_url, request.tool_name);
        
        let mut req_builder = self.http_client.post(&url).json(&request);
        
        if let Some(auth_header) = &self.auth_header {
            req_builder = req_builder.header("Authorization", auth_header);
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("MCP tool invocation failed: {}", error_text));
        }
        
        let tool_response: MCPToolResponse = response.json().await?;
        Ok(tool_response)
    }
}
```

### Usage Examples

#### Lua Integration Example
```lua
-- Configure MCP client manager
local mcp_manager = MCPClientManager.new({
    connection_timeout = 30,
    request_timeout = 60,
    max_connections_per_server = 5,
    security_policy = "strict"
})

-- Register external MCP servers
local weather_server = mcp_manager:register_mcp_server({
    name = "weather_service",
    uri = "https://weather-mcp.example.com",
    transport = "http",
    auth = {
        type = "api_key",
        key = config.weather_api_key
    },
    capabilities = {"tool_invocation", "streaming"}
})

local database_server = mcp_manager:register_mcp_server({
    name = "database_service", 
    uri = "ws://localhost:8080/mcp",
    transport = "websocket",
    auth = {
        type = "oauth2",
        client_id = config.db_client_id,
        client_secret = config.db_client_secret
    }
})

-- Create agent with MCP tools
local research_agent = Agent.new({
    system_prompt = "You are a research assistant with access to external tools",
    tools = {
        -- Built-in rs-llmspell tools
        WebSearch.new(),
        FileProcessor.new(),
        
        -- External MCP tools (automatically discovered)
        table.unpack(mcp_manager:create_tool_adapters(weather_server)),
        table.unpack(mcp_manager:create_tool_adapters(database_server))
    }
})

-- Agent can now use external MCP tools seamlessly
local result = research_agent:chat([[
    Research climate change impacts on agriculture. Use the weather service to get 
    historical climate data, and query the database for agricultural yield statistics.
    Provide a comprehensive analysis with charts.
]])

print("Research completed with MCP tools:")
print("Tools used:", table.concat(result.metadata.tools_used, ", "))
print("MCP servers contacted:", table.concat(result.metadata.mcp_servers, ", "))
```

#### JavaScript Integration Example
```javascript
// Configure MCP integration
const mcpManager = new MCPClientManager({
    connectionTimeout: 30000,
    requestTimeout: 60000,
    maxConnectionsPerServer: 5,
    retryPolicy: {
        maxRetries: 3,
        backoffStrategy: 'exponential'
    }
});

// Register multiple MCP servers
await mcpManager.registerMCPServer({
    name: 'slack_workspace',
    uri: 'https://slack-mcp.company.com',
    transport: 'http',
    auth: {
        type: 'bearer_token',
        token: process.env.SLACK_MCP_TOKEN
    }
});

await mcpManager.registerMCPServer({
    name: 'github_org',
    uri: 'wss://github-mcp.company.com/ws',
    transport: 'websocket', 
    auth: {
        type: 'github_app',
        appId: process.env.GITHUB_APP_ID,
        privateKey: process.env.GITHUB_PRIVATE_KEY
    }
});

// Create agent with mixed built-in and MCP tools
const devAgent = new Agent({
    systemPrompt: "You are a development assistant",
    tools: [
        // Built-in tools
        new CodeAnalyzer(),
        new FileSystem({ allowedPaths: ['./src', './tests'] }),
        
        // External MCP tools
        ...(await mcpManager.createToolAdapters('slack_workspace')),
        ...(await mcpManager.createToolAdapters('github_org'))
    ]
});

// Agent uses both local and remote tools
const response = await devAgent.chat(`
    Check the latest GitHub issues assigned to me, analyze the code files 
    mentioned in the issues, and post a summary to our team Slack channel.
`);

console.log('Development task completed');
console.log('Local tools used:', response.metadata.localTools);
console.log('MCP tools used:', response.metadata.mcpTools);
```

## MCP Server Support in Rs-LLMSpell

### Architecture for Exposing Rs-LLMSpell Tools as MCP Servers

**Goal**: Enable external systems to discover and use rs-llmspell tools via MCP protocol.

```rust
// MCP server implementation
pub struct MCPServer {
    tool_registry: Arc<RwLock<ToolRegistry>>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    server_config: MCPServerConfig,
    connection_manager: ConnectionManager,
    security_manager: SecurityManager,
    performance_monitor: ServerPerformanceMonitor,
}

pub struct MCPServerConfig {
    bind_address: String,
    port: u16,
    transport_types: Vec<TransportType>,
    auth_methods: Vec<AuthMethod>,
    rate_limits: RateLimitConfig,
    cors_settings: CorsConfig,
    tls_config: Option<TlsConfig>,
}

impl MCPServer {
    pub async fn start(&mut self) -> Result<()> {
        // Start HTTP server for MCP over HTTP
        if self.server_config.transport_types.contains(&TransportType::HTTP) {
            self.start_http_server().await?;
        }
        
        // Start WebSocket server for MCP over WebSocket
        if self.server_config.transport_types.contains(&TransportType::WebSocket) {
            self.start_websocket_server().await?;
        }
        
        // Start stdio server for MCP over stdio
        if self.server_config.transport_types.contains(&TransportType::Stdio) {
            self.start_stdio_server().await?;
        }
        
        info!("MCP server started on {}:{}", self.server_config.bind_address, self.server_config.port);
        Ok(())
    }
    
    async fn handle_connection_request(&self, request: MCPConnectionRequest) -> Result<MCPConnectionResponse> {
        // Authenticate client
        let auth_result = self.security_manager.authenticate(&request).await?;
        
        // Create session
        let session = MCPSession {
            id: Uuid::new_v4().to_string(),
            client_info: request.client_info,
            auth_context: auth_result,
            created_at: Utc::now(),
            capabilities: self.negotiate_capabilities(&request.capabilities),
        };
        
        // Register session
        self.connection_manager.register_session(session.clone()).await?;
        
        Ok(MCPConnectionResponse {
            session_id: session.id,
            server_capabilities: MCPServerCapabilities {
                tool_discovery: true,
                tool_invocation: true,
                agent_delegation: true,
                streaming_results: true,
                batch_operations: true,
            },
            protocol_version: "1.0".to_string(),
        })
    }
    
    async fn handle_list_tools_request(&self, session_id: &str) -> Result<MCPListToolsResponse> {
        // Verify session
        let session = self.connection_manager.get_session(session_id).await?;
        
        // Get available tools based on permissions
        let available_tools = {
            let registry = self.tool_registry.read().await;
            registry.list_tools_for_session(&session).await?
        };
        
        // Convert to MCP format
        let mcp_tools = available_tools.into_iter()
            .map(|tool| self.convert_tool_to_mcp_description(tool))
            .collect();
            
        Ok(MCPListToolsResponse {
            tools: mcp_tools,
            total_count: mcp_tools.len(),
        })
    }
    
    async fn handle_tool_invocation(&self, request: MCPToolInvocationRequest) -> Result<MCPToolInvocationResponse> {
        // Verify session and permissions
        let session = self.connection_manager.get_session(&request.session_id).await?;
        self.security_manager.authorize_tool_access(&session, &request.tool_name).await?;
        
        // Get tool instance
        let tool = {
            let registry = self.tool_registry.read().await;
            registry.get_tool(&request.tool_name)
                .ok_or_else(|| anyhow!("Tool not found: {}", request.tool_name))?
                .clone()
        };
        
        // Rate limiting check
        self.security_manager.check_rate_limit(&session, &request.tool_name).await?;
        
        // Execute tool
        let start_time = Instant::now();
        let execution_result = tool.execute(request.parameters).await;
        let duration = start_time.elapsed();
        
        // Record metrics
        self.performance_monitor.record_tool_execution(&request.tool_name, duration, &execution_result);
        
        match execution_result {
            Ok(output) => Ok(MCPToolInvocationResponse {
                success: true,
                result: output.content,
                metadata: Some(MCPExecutionMetadata {
                    execution_time_ms: duration.as_millis() as u64,
                    tool_version: output.metadata.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    session_id: request.session_id,
                }),
                error: None,
            }),
            Err(error) => Ok(MCPToolInvocationResponse {
                success: false,
                result: serde_json::Value::Null,
                metadata: None,
                error: Some(MCPError {
                    code: "TOOL_EXECUTION_FAILED".to_string(),
                    message: error.to_string(),
                    details: None,
                }),
            })
        }
    }
    
    fn convert_tool_to_mcp_description(&self, tool: &dyn Tool) -> MCPToolDescription {
        MCPToolDescription {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            input_schema: tool.parameters_schema(),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "content": {"type": "object"},
                    "metadata": {"type": "object"}
                }
            }),
            required_permissions: self.extract_tool_permissions(tool),
            rate_limits: self.get_tool_rate_limits(tool.name()),
            cost_info: self.get_tool_cost_info(tool.name()),
        }
    }
}

// Agent-as-Tool MCP exposure
pub struct MCPAgentWrapper {
    agent: Box<dyn Agent>,
    agent_config: AgentMCPConfig,
}

pub struct AgentMCPConfig {
    expose_as_tool: bool,
    tool_name: String,
    tool_description: String,
    input_schema: serde_json::Value,
    required_permissions: Vec<Permission>,
    execution_timeout: Duration,
}

impl MCPAgentWrapper {
    pub fn new(agent: Box<dyn Agent>, config: AgentMCPConfig) -> Self {
        Self {
            agent,
            agent_config: config,
        }
    }
    
    pub fn to_mcp_tool_description(&self) -> MCPToolDescription {
        MCPToolDescription {
            name: self.agent_config.tool_name.clone(),
            description: self.agent_config.tool_description.clone(),
            input_schema: self.agent_config.input_schema.clone(),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "response": {"type": "string"},
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "agent_id": {"type": "string"},
                            "execution_time_ms": {"type": "number"},
                            "tools_used": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        }
                    }
                }
            }),
            required_permissions: self.agent_config.required_permissions.clone(),
            rate_limits: Some(RateLimits {
                requests_per_minute: 60,
                requests_per_hour: 1000,
            }),
            cost_info: Some(CostInfo {
                base_cost: 0.01,
                unit: "request".to_string(),
                currency: "USD".to_string(),
            }),
        }
    }
    
    pub async fn execute_as_mcp_tool(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        // Extract message from MCP parameters
        let message = params.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'message' parameter"))?;
            
        let context = params.get("context").cloned().unwrap_or(json!({}));
        
        // Execute agent
        let start_time = Instant::now();
        let response = self.agent.chat(message).await?;
        let duration = start_time.elapsed();
        
        // Format response for MCP
        Ok(json!({
            "response": response,
            "metadata": {
                "agent_id": self.agent.id(),
                "execution_time_ms": duration.as_millis(),
                "tools_used": self.agent.last_execution_metadata()
                    .map(|m| m.tools_used)
                    .unwrap_or_default()
            }
        }))
    }
}
```

### Usage Examples for MCP Server

#### Exposing Built-in Tools via MCP
```lua
-- Configure MCP server to expose rs-llmspell tools
local mcp_server = MCPServer.new({
    bind_address = "0.0.0.0",
    port = 8080,
    transport_types = {"http", "websocket"},
    
    auth_methods = {
        {
            type = "api_key",
            validation_endpoint = "https://auth.company.com/validate"
        },
        {
            type = "oauth2",
            provider = "company_oauth"
        }
    },
    
    rate_limits = {
        requests_per_minute = 1000,
        requests_per_hour = 10000,
        burst_size = 50
    }
})

-- Register tools to expose via MCP
mcp_server:register_tool("web_search", WebSearchTool.new({
    api_key = config.search_api_key,
    max_results = 10
}), {
    required_permissions = {"web_access"},
    rate_limit_override = {
        requests_per_minute = 100
    }
})

mcp_server:register_tool("document_analyzer", DocumentAnalyzerTool.new(), {
    required_permissions = {"document_processing"},
    cost_per_request = 0.05
})

mcp_server:register_tool("code_generator", CodeGeneratorTool.new({
    languages = {"rust", "python", "javascript"}
}), {
    required_permissions = {"code_generation"},
    execution_timeout = 30
})

-- Expose specialized agents as tools
local research_agent = ResearchAgent.new({
    tools = {WebSearchTool.new(), DocumentAnalyzerTool.new()},
    system_prompt = "You are a specialized research assistant"
})

mcp_server:register_agent_as_tool(research_agent, {
    tool_name = "research_assistant",
    tool_description = "Specialized agent for research tasks",
    input_schema = {
        type = "object",
        properties = {
            message = {type = "string", description = "Research query"},
            context = {type = "object", description = "Additional context"}
        },
        required = {"message"}
    },
    required_permissions = {"research_access"}
})

-- Start the MCP server
mcp_server:start()
print("MCP server running - external systems can now discover and use our tools")
```

#### JavaScript MCP Server Example
```javascript
// Create MCP server exposing rs-llmspell capabilities
const mcpServer = new MCPServer({
    bindAddress: '0.0.0.0',
    port: 8080,
    transportTypes: ['http', 'websocket'],
    
    authMethods: [
        {
            type: 'jwt',
            publicKey: process.env.JWT_PUBLIC_KEY,
            issuer: 'company.com'
        }
    ],
    
    corsSettings: {
        allowedOrigins: ['https://app.company.com'],
        allowedMethods: ['GET', 'POST'],
        allowCredentials: true
    }
});

// Register company-specific tools
await mcpServer.registerTool('customer_lookup', new CustomerLookupTool({
    database: customerDb,
    permissions: ['customer_data_read']
}));

await mcpServer.registerTool('ticket_creator', new TicketCreatorTool({
    jiraConfig: jiraConfig,
    permissions: ['ticket_create']
}));

// Expose sophisticated agents as MCP tools
const supportAgent = new CustomerSupportAgent({
    tools: [
        new CustomerLookupTool(),
        new TicketCreatorTool(),
        new KnowledgeBaseSearchTool()
    ],
    systemPrompt: "You are a customer support specialist"
});

await mcpServer.registerAgentAsTool(supportAgent, {
    toolName: 'customer_support_agent',
    toolDescription: 'AI agent specialized in customer support tasks',
    inputSchema: {
        type: 'object',
        properties: {
            customer_query: { type: 'string' },
            customer_id: { type: 'string' },
            priority: { type: 'string', enum: ['low', 'medium', 'high'] }
        },
        required: ['customer_query']
    },
    requiredPermissions: ['customer_support_access'],
    executionTimeout: 60000
});

// Start server
await mcpServer.start();
console.log('MCP server started - tools available at http://localhost:8080/mcp');

// External systems can now discover available tools:
// GET http://localhost:8080/mcp/tools
// And invoke them:
// POST http://localhost:8080/mcp/tools/customer_support_agent/invoke
```

## MCP Crates and Libraries Research

### Existing Rust MCP Ecosystem

#### Available Crates Analysis

**1. mcp-rs (Hypothetical - Core MCP Protocol)**
```toml
[dependencies]
mcp-rs = "0.1.0"  # Core MCP protocol implementation
```

**Capabilities**:
- Core MCP message types and serialization
- Transport abstractions (HTTP, WebSocket, stdio)
- Client and server building blocks
- Protocol version negotiation

**2. mcp-client (Client-side Implementation)**
```toml
[dependencies]
mcp-client = "0.1.0"  # MCP client implementation
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
```

**Usage**:
```rust
use mcp_client::{MCPClient, ClientConfig};

let client = MCPClient::new(ClientConfig {
    server_uri: "https://tools.example.com/mcp".to_string(),
    auth: AuthConfig::ApiKey("your-api-key".to_string()),
    timeout: Duration::from_secs(30),
});

let connection = client.connect().await?;
let tools = client.list_tools(&connection).await?;
```

**3. mcp-server (Server-side Implementation)**
```toml
[dependencies]
mcp-server = "0.1.0"  # MCP server implementation
axum = "0.7"
tower = "0.4"
tokio = { version = "1.0", features = ["full"] }
```

**Usage**:
```rust
use mcp_server::{MCPServer, ServerConfig, Tool};

let server = MCPServer::new(ServerConfig {
    bind_address: "0.0.0.0:8080".parse()?,
    auth_providers: vec![AuthProvider::ApiKey],
});

server.register_tool("example_tool", ExampleTool::new()).await?;
server.start().await?;
```

**4. mcp-types (Protocol Types and Schemas)**
```toml
[dependencies]
mcp-types = "0.1.0"  # MCP protocol types
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Integration Architecture

```rust
// Rs-LLMSpell MCP integration crate structure
[workspace]
members = [
    "llmspell-mcp",           # Main MCP integration
    "llmspell-mcp-client",    # MCP client functionality
    "llmspell-mcp-server",    # MCP server functionality
    "llmspell-mcp-types",     # MCP-specific types for rs-llmspell
]

// llmspell-mcp/Cargo.toml
[dependencies]
llmspell-core = { path = "../llmspell-core" }
llmspell-tools = { path = "../llmspell-tools" }
llmspell-mcp-client = { path = "../llmspell-mcp-client" }
llmspell-mcp-server = { path = "../llmspell-mcp-server" }
mcp-rs = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
```

This comprehensive MCP support research provides the foundation for integrating rs-llmspell with the broader ecosystem of AI tools and services through standardized protocols.