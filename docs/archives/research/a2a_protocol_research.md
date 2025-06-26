# Agent to Agent (A2A) Protocol Research

## Overview

This document researches Agent to Agent (A2A) protocol support for rs-llmspell, enabling agents to discover, communicate with, and delegate tasks to other agents across different systems. A2A protocols facilitate multi-agent collaboration, distributed intelligence, and agent handoff patterns at scale.

## A2A Protocol Concepts

### Core Principles

**Agent to Agent (A2A) Protocol** enables autonomous agents to:
- **Discover** other agents and their capabilities
- **Negotiate** task delegation and collaboration
- **Execute** distributed workflows across agent networks
- **Monitor** and manage inter-agent communication
- **Handle** failures and agent unavailability gracefully

### A2A vs MCP Comparison

| Aspect | MCP (Model Control Protocol) | A2A (Agent to Agent Protocol) |
|--------|------------------------------|--------------------------------|
| **Focus** | Tools and resources | Agents and intelligence |
| **Granularity** | Function-level | Agent-level |
| **State** | Stateless operations | Stateful conversations |
| **Complexity** | Simple request/response | Complex negotiation |
| **Use Case** | Tool integration | Agent collaboration |

### A2A Communication Patterns

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   Agent A       │       │   Agent B       │       │   Agent C       │
│  (rs-llmspell)  │       │  (External)     │       │  (External)     │
└─────────────────┘       └─────────────────┘       └─────────────────┘
         │                          │                          │
         │  1. Discover Agents      │                          │
         │─────────────────────────→│                          │
         │  ←─────────────────────── │                          │
         │                          │                          │
         │  2. Capability Query     │                          │
         │─────────────────────────→│                          │
         │  ←─────────────────────── │                          │
         │                          │                          │
         │  3. Task Delegation      │                          │
         │─────────────────────────→│                          │
         │                          │  4. Sub-task to Agent C  │
         │                          │─────────────────────────→│
         │                          │  ←─────────────────────── │
         │  ←─────────────────────── │                          │
         │                          │                          │
         │  5. Result Aggregation   │                          │
         │─────────────────────────→│                          │
         │  ←─────────────────────── │                          │
```

## A2A Client Support in Rs-LLMSpell

### Architecture for A2A Client Integration

**Goal**: Enable rs-llmspell agents to discover and collaborate with external agents.

```rust
// Core A2A client traits and types
pub trait A2AClient: Send + Sync {
    async fn discover_agents(&self, discovery_config: DiscoveryConfig) -> Result<Vec<AgentDescriptor>>;
    async fn establish_connection(&mut self, agent_id: &str) -> Result<A2AConnection>;
    async fn negotiate_capabilities(&self, connection: &A2AConnection) -> Result<CapabilityNegotiation>;
    async fn delegate_task(&self, connection: &A2AConnection, task: TaskDelegation) -> Result<TaskResult>;
    async fn start_conversation(&self, connection: &A2AConnection, conversation: ConversationRequest) -> Result<ConversationSession>;
    async fn monitor_agent_health(&self, agent_id: &str) -> Result<AgentHealth>;
}

pub struct A2AConnection {
    session_id: String,
    agent_id: String,
    transport: Box<dyn A2ATransport>,
    capabilities: NegotiatedCapabilities,
    auth_context: AuthContext,
    heartbeat_interval: Duration,
    last_activity: Instant,
}

pub struct AgentDescriptor {
    id: String,
    name: String,
    description: String,
    version: String,
    endpoint: String,
    capabilities: AgentCapabilities,
    specializations: Vec<String>,
    availability: AgentAvailability,
    reputation: AgentReputation,
    cost_model: Option<CostModel>,
}

pub struct AgentCapabilities {
    supported_tasks: Vec<TaskType>,
    input_formats: Vec<String>,
    output_formats: Vec<String>,
    conversation_modes: Vec<ConversationMode>,
    batch_processing: bool,
    streaming_support: bool,
    multi_turn_conversations: bool,
    state_persistence: bool,
}

// A2A agent registry and discovery
pub struct A2AAgentRegistry {
    discovery_services: Vec<Box<dyn DiscoveryService>>,
    agent_cache: Arc<RwLock<HashMap<String, CachedAgentInfo>>>,
    connection_pool: A2AConnectionPool,
    reputation_system: ReputationSystem,
    load_balancer: Box<dyn AgentLoadBalancer>,
}

pub trait DiscoveryService: Send + Sync {
    async fn discover_agents(&self, query: DiscoveryQuery) -> Result<Vec<AgentDescriptor>>;
    fn discovery_method(&self) -> DiscoveryMethod;
}

#[derive(Clone)]
pub enum DiscoveryMethod {
    ServiceRegistry { endpoint: String },
    Multicast { network_interface: String },
    DHT { bootstrap_nodes: Vec<String> },
    Gossip { known_peers: Vec<String> },
    Static { agent_configs: Vec<StaticAgentConfig> },
}

impl A2AAgentRegistry {
    pub async fn discover_agents_by_capability(&self, capability: &str) -> Result<Vec<AgentDescriptor>> {
        let mut all_agents = Vec::new();
        
        // Query all discovery services
        for discovery_service in &self.discovery_services {
            let query = DiscoveryQuery {
                capability_filter: Some(capability.to_string()),
                availability_filter: Some(AgentAvailability::Available),
                reputation_threshold: Some(0.7),
                max_results: Some(50),
            };
            
            match discovery_service.discover_agents(query).await {
                Ok(agents) => all_agents.extend(agents),
                Err(e) => warn!("Discovery service failed: {}", e),
            }
        }
        
        // Deduplicate and rank agents
        let unique_agents = self.deduplicate_agents(all_agents);
        let ranked_agents = self.rank_agents_by_suitability(unique_agents, capability).await?;
        
        Ok(ranked_agents)
    }
    
    async fn rank_agents_by_suitability(&self, agents: Vec<AgentDescriptor>, capability: &str) -> Result<Vec<AgentDescriptor>> {
        let mut scored_agents = Vec::new();
        
        for agent in agents {
            let suitability_score = self.calculate_suitability_score(&agent, capability).await?;
            scored_agents.push((agent, suitability_score));
        }
        
        // Sort by suitability score (highest first)
        scored_agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(scored_agents.into_iter().map(|(agent, _)| agent).collect())
    }
    
    async fn calculate_suitability_score(&self, agent: &AgentDescriptor, capability: &str) -> Result<f64> {
        let capability_match = if agent.capabilities.supported_tasks.iter()
            .any(|task| task.name.contains(capability)) { 1.0 } else { 0.0 };
            
        let reputation_score = agent.reputation.overall_score;
        let availability_score = match agent.availability {
            AgentAvailability::Available => 1.0,
            AgentAvailability::Busy => 0.5,
            AgentAvailability::Unavailable => 0.0,
        };
        
        let cost_score = match &agent.cost_model {
            Some(cost) => 1.0 / (1.0 + cost.base_cost), // Lower cost = higher score
            None => 1.0,
        };
        
        Ok(capability_match * 0.4 + reputation_score * 0.3 + availability_score * 0.2 + cost_score * 0.1)
    }
}

// A2A task delegation
pub struct TaskDelegation {
    task_id: String,
    task_type: TaskType,
    input_data: serde_json::Value,
    requirements: TaskRequirements,
    constraints: TaskConstraints,
    callback_config: Option<CallbackConfig>,
}

pub struct TaskRequirements {
    max_execution_time: Duration,
    required_quality_score: f64,
    preferred_agent_specializations: Vec<String>,
    input_format: String,
    output_format: String,
}

pub struct TaskConstraints {
    max_cost: Option<f64>,
    geographic_restrictions: Vec<String>,
    data_residency_requirements: Vec<String>,
    security_clearance_level: Option<String>,
}

// A2A agent wrapper for external agents
pub struct A2AAgentProxy {
    agent_descriptor: AgentDescriptor,
    connection: Option<A2AConnection>,
    client: Box<dyn A2AClient>,
    performance_monitor: AgentPerformanceMonitor,
    fallback_strategy: FallbackStrategy,
}

#[async_trait]
impl Agent for A2AAgentProxy {
    fn id(&self) -> &str {
        &self.agent_descriptor.id
    }
    
    fn name(&self) -> &str {
        &self.agent_descriptor.name
    }
    
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput> {
        // Ensure connection is established
        if self.connection.is_none() {
            let connection = self.client.establish_connection(&self.agent_descriptor.id).await?;
            self.connection = Some(connection);
        }
        
        let connection = self.connection.as_ref().unwrap();
        
        // Create task delegation
        let task = TaskDelegation {
            task_id: Uuid::new_v4().to_string(),
            task_type: TaskType::General,
            input_data: serde_json::to_value(&input)?,
            requirements: TaskRequirements {
                max_execution_time: Duration::from_secs(120),
                required_quality_score: 0.8,
                preferred_agent_specializations: vec![],
                input_format: "json".to_string(),
                output_format: "json".to_string(),
            },
            constraints: TaskConstraints {
                max_cost: Some(1.0),
                geographic_restrictions: vec![],
                data_residency_requirements: vec![],
                security_clearance_level: None,
            },
            callback_config: None,
        };
        
        // Execute task delegation
        let start_time = Instant::now();
        let result = match self.client.delegate_task(connection, task).await {
            Ok(task_result) => {
                let duration = start_time.elapsed();
                self.performance_monitor.record_success(&self.agent_descriptor.id, duration);
                
                AgentOutput {
                    content: task_result.output,
                    metadata: HashMap::from([
                        ("external_agent_id".to_string(), Value::String(self.agent_descriptor.id.clone())),
                        ("execution_time_ms".to_string(), Value::Number(duration.as_millis().into())),
                        ("task_id".to_string(), Value::String(task_result.task_id)),
                        ("agent_version".to_string(), Value::String(self.agent_descriptor.version.clone())),
                    ]),
                }
            },
            Err(error) => {
                self.performance_monitor.record_failure(&self.agent_descriptor.id, &error);
                
                // Apply fallback strategy
                match &self.fallback_strategy {
                    FallbackStrategy::RetryWithBackoff { max_retries, base_delay } => {
                        return self.retry_with_backoff(input, *max_retries, *base_delay).await;
                    },
                    FallbackStrategy::UseAlternativeAgent { alternative_agents } => {
                        return self.try_alternative_agents(input, alternative_agents).await;
                    },
                    FallbackStrategy::Fail => {
                        return Err(error);
                    }
                }
            }
        };
        
        Ok(result)
    }
    
    async fn chat(&mut self, message: &str) -> Result<String> {
        let input = AgentInput {
            message: message.to_string(),
            context: HashMap::new(),
        };
        
        let output = self.execute(input).await?;
        Ok(output.content.as_str().unwrap_or("").to_string())
    }
}

// Conversation management for multi-turn interactions
pub struct A2AConversationManager {
    active_conversations: HashMap<String, ConversationSession>,
    conversation_store: Box<dyn ConversationStore>,
    session_timeout: Duration,
}

pub struct ConversationSession {
    session_id: String,
    participants: Vec<String>, // Agent IDs
    conversation_history: Vec<ConversationTurn>,
    state: ConversationState,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}

pub struct ConversationTurn {
    turn_id: String,
    agent_id: String,
    message: String,
    timestamp: DateTime<Utc>,
    message_type: MessageType,
    metadata: HashMap<String, Value>,
}

impl A2AConversationManager {
    pub async fn start_multi_agent_conversation(&mut self, participants: Vec<String>, initial_prompt: String) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        
        let session = ConversationSession {
            session_id: session_id.clone(),
            participants: participants.clone(),
            conversation_history: vec![],
            state: ConversationState::Active,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        
        // Add initial turn
        let initial_turn = ConversationTurn {
            turn_id: Uuid::new_v4().to_string(),
            agent_id: "system".to_string(),
            message: initial_prompt,
            timestamp: Utc::now(),
            message_type: MessageType::System,
            metadata: HashMap::new(),
        };
        
        session.conversation_history.push(initial_turn);
        self.active_conversations.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    pub async fn add_conversation_turn(&mut self, session_id: &str, agent_id: &str, message: String) -> Result<()> {
        let session = self.active_conversations.get_mut(session_id)
            .ok_or_else(|| anyhow!("Conversation session not found: {}", session_id))?;
            
        let turn = ConversationTurn {
            turn_id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            message,
            timestamp: Utc::now(),
            message_type: MessageType::Agent,
            metadata: HashMap::new(),
        };
        
        session.conversation_history.push(turn);
        session.last_activity = Utc::now();
        
        // Persist conversation
        self.conversation_store.save_conversation(session).await?;
        
        Ok(())
    }
}
```

### Usage Examples

#### Lua A2A Client Example
```lua
-- Configure A2A agent registry with multiple discovery methods
local a2a_registry = A2AAgentRegistry.new({
    discovery_services = {
        ServiceRegistryDiscovery.new({
            endpoint = "https://agent-registry.company.com",
            api_key = config.registry_api_key
        }),
        
        MulticastDiscovery.new({
            network_interface = "eth0",
            multicast_group = "224.0.0.251",
            port = 5353
        }),
        
        StaticDiscovery.new({
            agent_configs = {
                {
                    id = "code_review_agent",
                    endpoint = "https://code-review.company.com/a2a",
                    capabilities = {"code_analysis", "security_review", "style_check"}
                },
                {
                    id = "documentation_agent", 
                    endpoint = "wss://docs.company.com/a2a",
                    capabilities = {"documentation_generation", "api_docs", "user_guides"}
                }
            }
        })
    },
    
    reputation_system = ReputationSystem.new({
        initial_score = 0.5,
        decay_factor = 0.1,
        min_interactions = 5
    })
})

-- Create agent that can delegate to external agents
local development_orchestrator = Agent.new({
    system_prompt = "You coordinate development tasks across multiple specialist agents",
    a2a_registry = a2a_registry,
    
    fallback_strategy = FallbackStrategy.UseAlternativeAgent({
        alternative_agents = {"backup_code_agent", "general_dev_agent"}
    })
})

-- Agent can discover and delegate to external agents
local development_result = development_orchestrator:chat([[
    I need help with a new feature development:
    1. Review the code for security issues
    2. Generate comprehensive documentation
    3. Create unit tests
    4. Perform code style analysis
    
    Please delegate these tasks to appropriate specialist agents.
]])

print("Development coordination completed:")
print("External agents used:", table.concat(development_result.metadata.external_agents, ", "))
print("Total execution time:", development_result.metadata.total_time)
```

#### JavaScript A2A Client Example
```javascript
// Advanced A2A client with conversation management
const a2aClient = new A2AClient({
    discoveryServices: [
        new ConsulDiscovery({
            consulEndpoint: 'http://consul.company.com:8500',
            serviceName: 'ai-agents'
        }),
        
        new KubernetesDiscovery({
            namespace: 'ai-agents',
            labelSelector: 'app=ai-agent'
        }),
        
        new EurekaDiscovery({
            eurekaEndpoint: 'http://eureka.company.com:8761'
        })
    ],
    
    connectionPool: {
        maxConnectionsPerAgent: 3,
        connectionTimeout: 30000,
        heartbeatInterval: 10000
    },
    
    loadBalancing: 'round_robin',
    circuitBreaker: {
        failureThreshold: 5,
        resetTimeout: 60000
    }
});

// Create orchestrator agent with A2A capabilities
const projectManager = new Agent({
    systemPrompt: "You are a project manager that coordinates work across teams",
    a2aClient,
    
    tools: [
        new TaskPlannerTool(),
        new ResourceAllocatorTool()
    ]
});

// Multi-agent project coordination
const project = await projectManager.execute({
    message: `
        Plan and execute a new microservice development project:
        - Backend API development (delegate to backend team agent)
        - Frontend UI development (delegate to frontend team agent)  
        - DevOps setup (delegate to infrastructure team agent)
        - QA testing (delegate to QA team agent)
        
        Coordinate timeline and dependencies between teams.
    `,
    
    project_requirements: {
        deadline: '2025-03-01',
        budget: 50000,
        team_size: 12,
        technology_stack: ['Node.js', 'React', 'Docker', 'AWS']
    }
});

console.log('Project coordination completed');
console.log('Teams involved:', project.metadata.teams_coordinated);
console.log('Timeline:', project.metadata.project_timeline);

// Multi-turn conversation example
const conversationId = await a2aClient.startMultiAgentConversation({
    participants: [
        'backend_team_lead',
        'frontend_team_lead', 
        'devops_engineer',
        'qa_lead'
    ],
    initialPrompt: 'Lets plan the microservice architecture for the new project'
});

// Each agent contributes to the conversation
await a2aClient.addConversationTurn(conversationId, 'backend_team_lead', 
    'I suggest we use a microservices pattern with REST APIs and event sourcing');

await a2aClient.addConversationTurn(conversationId, 'frontend_team_lead',
    'For the frontend, I recommend a microfrontend approach to match the backend architecture');

const conversationSummary = await a2aClient.summarizeConversation(conversationId);
console.log('Architecture decisions:', conversationSummary.decisions);
```

## A2A Server Support in Rs-LLMSpell

### Architecture for Exposing Agents via A2A

**Goal**: Enable external systems to discover, connect to, and collaborate with rs-llmspell agents.

```rust
// A2A server implementation
pub struct A2AServer {
    agent_registry: Arc<RwLock<LocalAgentRegistry>>,
    workflow_registry: Arc<RwLock<WorkflowRegistry>>,
    server_config: A2AServerConfig,
    connection_manager: A2AConnectionManager,
    discovery_beacon: DiscoveryBeacon,
    reputation_tracker: ReputationTracker,
}

pub struct A2AServerConfig {
    bind_address: SocketAddr,
    supported_transports: Vec<TransportType>,
    discovery_config: DiscoveryConfig,
    auth_config: AuthConfig,
    rate_limits: RateLimitConfig,
    security_policy: SecurityPolicy,
}

pub struct LocalAgentRegistry {
    agents: HashMap<String, RegisteredAgent>,
    workflows: HashMap<String, RegisteredWorkflow>,
    capabilities_index: CapabilitiesIndex,
}

pub struct RegisteredAgent {
    agent: Box<dyn Agent>,
    descriptor: AgentDescriptor,
    access_policy: AccessPolicy,
    usage_stats: AgentUsageStats,
    health_monitor: AgentHealthMonitor,
}

impl A2AServer {
    pub async fn start(&mut self) -> Result<()> {
        // Start discovery beacon
        self.discovery_beacon.start_advertising().await?;
        
        // Start connection handlers
        if self.server_config.supported_transports.contains(&TransportType::HTTP) {
            self.start_http_handler().await?;
        }
        
        if self.server_config.supported_transports.contains(&TransportType::WebSocket) {
            self.start_websocket_handler().await?;
        }
        
        // Start health monitoring
        self.start_health_monitoring().await?;
        
        info!("A2A server started on {}", self.server_config.bind_address);
        Ok(())
    }
    
    pub async fn register_agent(&mut self, agent: Box<dyn Agent>, config: AgentRegistrationConfig) -> Result<String> {
        let agent_id = agent.id().to_string();
        
        // Create agent descriptor
        let descriptor = AgentDescriptor {
            id: agent_id.clone(),
            name: config.name.unwrap_or_else(|| agent.name().to_string()),
            description: config.description,
            version: config.version.unwrap_or_else(|| "1.0.0".to_string()),
            endpoint: format!("{}/agents/{}", self.server_config.public_endpoint, agent_id),
            capabilities: self.extract_agent_capabilities(&*agent).await?,
            specializations: config.specializations,
            availability: AgentAvailability::Available,
            reputation: AgentReputation::new(),
            cost_model: config.cost_model,
        };
        
        // Create registered agent
        let registered_agent = RegisteredAgent {
            agent,
            descriptor: descriptor.clone(),
            access_policy: config.access_policy.unwrap_or_default(),
            usage_stats: AgentUsageStats::new(),
            health_monitor: AgentHealthMonitor::new(),
        };
        
        // Add to registry
        {
            let mut registry = self.agent_registry.write().await;
            registry.agents.insert(agent_id.clone(), registered_agent);
            registry.capabilities_index.index_agent(&descriptor).await?;
        }
        
        // Update discovery beacon
        self.discovery_beacon.update_agent_list().await?;
        
        info!("Registered agent {} for A2A access", agent_id);
        Ok(agent_id)
    }
    
    async fn handle_agent_discovery_request(&self, request: DiscoveryRequest) -> Result<DiscoveryResponse> {
        let registry = self.agent_registry.read().await;
        
        // Filter agents based on discovery criteria
        let matching_agents = registry.capabilities_index
            .find_agents_by_criteria(&request.criteria).await?;
            
        // Apply access control
        let accessible_agents = matching_agents.into_iter()
            .filter(|agent| self.check_discovery_access(&request.requester, agent))
            .collect();
            
        Ok(DiscoveryResponse {
            agents: accessible_agents,
            total_count: registry.agents.len(),
            server_info: ServerInfo {
                name: "rs-llmspell".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                capabilities: self.get_server_capabilities(),
            },
        })
    }
    
    async fn handle_task_delegation(&self, request: TaskDelegationRequest) -> Result<TaskDelegationResponse> {
        // Validate request
        self.validate_delegation_request(&request).await?;
        
        // Get target agent
        let agent = {
            let registry = self.agent_registry.read().await;
            registry.agents.get(&request.target_agent_id)
                .ok_or_else(|| anyhow!("Agent not found: {}", request.target_agent_id))?
                .agent.clone() // This would need proper cloning/sharing
        };
        
        // Check access permissions
        self.check_delegation_permissions(&request).await?;
        
        // Apply rate limiting
        self.check_rate_limits(&request.requester_id, &request.target_agent_id).await?;
        
        // Execute task
        let start_time = Instant::now();
        let execution_result = self.execute_delegated_task(agent, &request).await;
        let duration = start_time.elapsed();
        
        // Record usage statistics
        self.record_delegation_usage(&request, &execution_result, duration).await?;
        
        match execution_result {
            Ok(result) => Ok(TaskDelegationResponse {
                task_id: request.task_id,
                success: true,
                output: result.content,
                metadata: TaskExecutionMetadata {
                    execution_time_ms: duration.as_millis() as u64,
                    agent_id: request.target_agent_id,
                    cost: self.calculate_execution_cost(&request, duration),
                },
                error: None,
            }),
            Err(error) => Ok(TaskDelegationResponse {
                task_id: request.task_id,
                success: false,
                output: serde_json::Value::Null,
                metadata: TaskExecutionMetadata {
                    execution_time_ms: duration.as_millis() as u64,
                    agent_id: request.target_agent_id,
                    cost: 0.0,
                },
                error: Some(A2AError {
                    code: "EXECUTION_FAILED".to_string(),
                    message: error.to_string(),
                    details: None,
                }),
            })
        }
    }
}

// Workflow exposure via A2A
pub struct A2AWorkflowWrapper {
    workflow: Box<dyn Workflow>,
    workflow_config: WorkflowA2AConfig,
}

pub struct WorkflowA2AConfig {
    expose_as_agent: bool,
    agent_name: String,
    agent_description: String,
    input_schema: serde_json::Value,
    access_control: AccessControl,
    execution_limits: ExecutionLimits,
}

impl A2AWorkflowWrapper {
    pub fn to_agent_descriptor(&self) -> AgentDescriptor {
        AgentDescriptor {
            id: format!("workflow_{}", self.workflow.id()),
            name: self.workflow_config.agent_name.clone(),
            description: self.workflow_config.agent_description.clone(),
            version: "1.0.0".to_string(),
            endpoint: format!("/workflows/{}", self.workflow.id()),
            capabilities: AgentCapabilities {
                supported_tasks: vec![TaskType::Workflow],
                input_formats: vec!["json".to_string()],
                output_formats: vec!["json".to_string()],
                conversation_modes: vec![ConversationMode::SingleTurn],
                batch_processing: false,
                streaming_support: true,
                multi_turn_conversations: false,
                state_persistence: true,
            },
            specializations: vec!["workflow_execution".to_string()],
            availability: AgentAvailability::Available,
            reputation: AgentReputation::new(),
            cost_model: Some(CostModel {
                base_cost: 0.10,
                unit: "execution".to_string(),
                currency: "USD".to_string(),
            }),
        }
    }
    
    pub async fn execute_via_a2a(&mut self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Convert A2A input to workflow input
        let workflow_input = WorkflowInput::from_json(input)?;
        
        // Execute workflow
        let start_time = Instant::now();
        let workflow_output = self.workflow.run(workflow_input).await?;
        let duration = start_time.elapsed();
        
        // Convert to A2A output format
        Ok(json!({
            "result": workflow_output.result,
            "metadata": {
                "workflow_id": self.workflow.id(),
                "execution_time_ms": duration.as_millis(),
                "steps_executed": workflow_output.steps_executed,
                "workflow_type": self.workflow.workflow_type()
            }
        }))
    }
}

// Discovery beacon for agent advertisement
pub struct DiscoveryBeacon {
    beacon_config: BeaconConfig,
    agent_advertisements: Arc<RwLock<Vec<AgentAdvertisement>>>,
    multicast_sender: Option<UdpSocket>,
    service_registry_client: Option<Box<dyn ServiceRegistryClient>>,
}

impl DiscoveryBeacon {
    pub async fn start_advertising(&mut self) -> Result<()> {
        // Start multicast advertising
        if self.beacon_config.multicast_enabled {
            self.start_multicast_beacon().await?;
        }
        
        // Register with service registries
        if let Some(registry_client) = &self.service_registry_client {
            self.register_with_service_registry(registry_client).await?;
        }
        
        // Start periodic advertisement updates
        self.start_periodic_updates().await?;
        
        Ok(())
    }
    
    async fn start_multicast_beacon(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.join_multicast_v4("224.0.0.251".parse()?, "0.0.0.0".parse()?)?;
        
        let advertisements = Arc::clone(&self.agent_advertisements);
        let beacon_interval = self.beacon_config.beacon_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(beacon_interval);
            
            loop {
                interval.tick().await;
                
                let ads = advertisements.read().await;
                for advertisement in ads.iter() {
                    let beacon_message = BeaconMessage {
                        message_type: BeaconMessageType::Advertisement,
                        agent_info: advertisement.clone(),
                        timestamp: Utc::now(),
                        ttl: beacon_interval.as_secs() * 3, // 3x beacon interval
                    };
                    
                    let message_bytes = serde_json::to_vec(&beacon_message)?;
                    socket.send_to(&message_bytes, "224.0.0.251:5353").await?;
                }
                
                Ok::<(), anyhow::Error>(())
            }
        });
        
        self.multicast_sender = Some(socket);
        Ok(())
    }
}
```

### Usage Examples for A2A Server

#### Lua A2A Server Example
```lua
-- Configure A2A server to expose rs-llmspell agents
local a2a_server = A2AServer.new({
    bind_address = "0.0.0.0:9090",
    supported_transports = {"http", "websocket"},
    
    discovery_config = {
        multicast_enabled = true,
        service_registry = {
            type = "consul",
            endpoint = "http://consul.company.com:8500"
        },
        beacon_interval = 30 -- seconds
    },
    
    auth_config = {
        methods = {"api_key", "mutual_tls"},
        api_key_validation = "https://auth.company.com/validate"
    },
    
    rate_limits = {
        requests_per_agent_per_minute = 100,
        concurrent_executions_per_agent = 5
    }
})

-- Register individual agents for A2A access
local code_review_agent = CodeReviewAgent.new({
    tools = {StaticAnalysisTool.new(), SecurityScannerTool.new()},
    system_prompt = "You are a code review specialist"
})

a2a_server:register_agent(code_review_agent, {
    name = "Code Review Specialist",
    description = "Performs comprehensive code reviews including security and style analysis",
    specializations = {"code_analysis", "security_review", "style_check"},
    access_policy = AccessPolicy.new({
        allowed_organizations = {"internal", "trusted_partners"},
        required_clearance_level = "standard"
    }),
    cost_model = CostModel.new({
        base_cost = 0.05,
        unit = "review",
        currency = "USD"
    })
})

-- Register workflows as agents
local deployment_workflow = Workflow.sequential({
    name = "CI/CD Deployment",
    steps = {
        {agent = "build_agent", action = "compile"},
        {agent = "test_agent", action = "run_tests"},
        {agent = "security_agent", action = "security_scan"},
        {agent = "deploy_agent", action = "deploy"}
    }
})

a2a_server:register_workflow_as_agent(deployment_workflow, {
    agent_name = "Deployment Pipeline",
    agent_description = "Complete CI/CD pipeline for application deployment",
    access_control = AccessControl.new({
        require_admin_approval = true,
        audit_all_executions = true
    })
})

-- Start A2A server
a2a_server:start()
print("A2A server running - agents discoverable by external systems")

-- External systems can now:
-- 1. Discover agents: GET http://localhost:9090/discover
-- 2. Delegate tasks: POST http://localhost:9090/agents/code_review_specialist/delegate
-- 3. Monitor health: GET http://localhost:9090/agents/code_review_specialist/health
```

#### JavaScript A2A Server Example
```javascript
// Enterprise A2A server setup
const a2aServer = new A2AServer({
    bindAddress: '0.0.0.0:9090',
    supportedTransports: ['http', 'websocket', 'grpc'],
    
    discoveryConfig: {
        serviceRegistry: {
            type: 'kubernetes',
            namespace: 'ai-agents',
            serviceAccount: 'a2a-server'
        },
        multicastEnabled: false, // Disabled in enterprise env
        staticRegistration: true
    },
    
    authConfig: {
        methods: ['oauth2', 'service_account'],
        oauth2Provider: 'https://auth.company.com',
        requireTLS: true
    },
    
    monitoring: {
        metricsEnabled: true,
        tracingEnabled: true,
        healthCheckInterval: 30000
    }
});

// Register company-specific agents
const customerSupportAgent = new CustomerSupportAgent({
    tools: [
        new CRMIntegrationTool(),
        new TicketManagementTool(),
        new KnowledgeBaseTool()
    ],
    systemPrompt: "You are a customer support specialist"
});

await a2aServer.registerAgent(customerSupportAgent, {
    name: 'Customer Support Specialist',
    description: 'Handles customer inquiries and support tickets',
    specializations: ['customer_service', 'ticket_management', 'escalation'],
    
    accessPolicy: {
        allowedDepartments: ['customer_success', 'support', 'sales'],
        dataClassification: 'internal',
        auditLevel: 'full'
    },
    
    costModel: {
        baseCost: 0.02,
        unit: 'interaction',
        currency: 'USD'
    },
    
    slaRequirements: {
        maxResponseTime: 5000, // ms
        availabilityTarget: 0.995,
        maxConcurrentSessions: 50
    }
});

// Register complex workflows
const incidentResponseWorkflow = new Workflow({
    name: 'Incident Response',
    steps: [
        { name: 'triage', agent: 'triageAgent' },
        { name: 'investigation', agent: 'investigationAgent' },
        { name: 'resolution', agent: 'resolutionAgent' },
        { name: 'postmortem', agent: 'documentationAgent' }
    ]
});

await a2aServer.registerWorkflowAsAgent(incidentResponseWorkflow, {
    agentName: 'Incident Response Coordinator',
    agentDescription: 'Coordinates full incident response lifecycle',
    
    executionLimits: {
        maxExecutionTime: 3600000, // 1 hour
        maxParallelExecutions: 3,
        requireApproval: true
    }
});

// Start server with enterprise features
await a2aServer.start();

console.log('Enterprise A2A server started');
console.log('Registered agents:', await a2aServer.getRegisteredAgents());
console.log('Discovery endpoint: http://localhost:9090/discover');
console.log('Health endpoint: http://localhost:9090/health');

// Usage monitoring
a2aServer.on('agentExecution', (event) => {
    console.log(`Agent ${event.agentId} executed by ${event.requesterId}`);
    console.log(`Duration: ${event.executionTime}ms, Cost: $${event.cost}`);
});

a2aServer.on('discoveryRequest', (event) => {
    console.log(`Discovery request from ${event.requesterId}`);
    console.log(`Criteria: ${JSON.stringify(event.criteria)}`);
});
```

## A2A Crates and Protocol Standards

### Proposed Rust A2A Ecosystem

#### Core A2A Protocol Crates
```toml
# Core A2A protocol specification
[dependencies]
a2a-protocol = "0.1.0"         # Core protocol types and messages
a2a-discovery = "0.1.0"        # Agent discovery mechanisms  
a2a-transport = "0.1.0"        # Transport layer abstractions
a2a-security = "0.1.0"         # Authentication and authorization
a2a-client = "0.1.0"           # A2A client implementation
a2a-server = "0.1.0"           # A2A server implementation
```

#### Integration Architecture
```rust
// Rs-LLMSpell A2A integration crate structure
[workspace]
members = [
    "llmspell-a2a",              # Main A2A integration
    "llmspell-a2a-client",       # A2A client functionality  
    "llmspell-a2a-server",       # A2A server functionality
    "llmspell-a2a-discovery",    # Discovery service implementations
    "llmspell-a2a-transport",    # Transport implementations
]

// llmspell-a2a/Cargo.toml
[dependencies]
llmspell-core = { path = "../llmspell-core" }
llmspell-agents = { path = "../llmspell-agents" }
llmspell-workflows = { path = "../llmspell-workflows" }
a2a-protocol = "0.1.0"
a2a-client = "0.1.0"
a2a-server = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

This comprehensive A2A protocol research provides the foundation for building distributed agent networks where rs-llmspell agents can seamlessly collaborate with agents from other systems, enabling true multi-agent intelligence at scale.