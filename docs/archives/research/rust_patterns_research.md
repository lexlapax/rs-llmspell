# Rust Patterns Research for Rs-LLMSpell

## Overview

Research findings on existing Rust patterns for agent systems, hook architectures, state management, and plugin systems that can be adapted for rs-llmspell's architecture.

## Table of Contents

1. [Event Hook Systems](#event-hook-systems)
2. [Agent-Based Systems](#agent-based-systems)
3. [Plugin Architecture Patterns](#plugin-architecture-patterns)
4. [Async State Management](#async-state-management)
5. [Observer Pattern Implementations](#observer-pattern-implementations)
6. [Performance Considerations](#performance-considerations)
7. [Recommendations for Rs-LLMSpell](#recommendations-for-rs-llmspell)

## Event Hook Systems

### Trait-Based Event Design

**Pattern**: Define event hooks as traits with default no-op implementations

```rust
// From mattgathu.dev research
pub trait Events {
    fn on_connect(&self, host: &str, port: i32) {}
    fn on_error(&self, err: &str) {}
    fn on_disconnect(&self) {}
    // Default implementations allow selective hook implementation
}

// Hook registration pattern
pub struct EventManager {
    hooks: Vec<Box<dyn Events>>,
}

impl EventManager {
    pub fn add_events_hook<E: Events + 'static>(&mut self, hook: E) {
        self.hooks.push(Box::new(hook));
    }
    
    pub fn trigger_connect(&self, host: &str, port: i32) {
        for hook in &self.hooks {
            hook.on_connect(host, port);
        }
    }
}
```

**Key Benefits**:
- Generic, type-agnostic event handlers
- Selective hook implementation via default methods
- Dynamic dispatch with `Box<dyn Trait>`

**Adaptation for Rs-LLMSpell**:
```rust
pub trait AgentHook: Send + Sync {
    fn on_agent_start(&self, agent_id: &str, context: &HookContext) {}
    fn on_agent_complete(&self, agent_id: &str, result: &AgentResponse) {}
    fn on_tool_execute(&self, tool_name: &str, input: &ToolInput) {}
    fn on_state_change(&self, old_state: &AgentState, new_state: &AgentState) {}
}
```

### Lifecycle Hook Pattern

**Pattern**: Pre/post hooks for operation lifecycle management

```rust
// From Apollo Router research
pub trait Plugin {
    // Plugin lifecycle
    async fn on_startup(&self) -> Result<(), PluginError> { Ok(()) }
    async fn on_shutdown(&self) -> Result<(), PluginError> { Ok(()) }
    
    // Request lifecycle hooks
    async fn on_request_pre(&self, req: &Request) -> Result<(), PluginError> { Ok(()) }
    async fn on_request_post(&self, resp: &Response) -> Result<(), PluginError> { Ok(()) }
}

// Hook execution with error handling
impl PluginManager {
    async fn execute_pre_hooks(&self, req: &Request) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.on_request_pre(req).await?;
        }
        Ok(())
    }
}
```

**Rs-LLMSpell Implementation**:
```rust
pub enum HookPoint {
    PreAgentExecution,
    PostAgentExecution,
    PreToolCall,
    PostToolCall,
    PreStateUpdate,
    PostStateUpdate,
}

#[async_trait]
pub trait Hook: Send + Sync {
    async fn execute(&self, point: HookPoint, context: &HookContext) 
        -> Result<HookResult, HookError>;
    
    fn hook_points(&self) -> Vec<HookPoint>;
    fn priority(&self) -> u32; // Higher numbers execute first
}
```

## Agent-Based Systems

### Agent Trait Pattern

**Pattern**: Core agent struct with focused responsibilities

```rust
// From Shuttle AI agents research
pub struct Agent {
    pub system: String,        // System prompt/role definition
    pub model: String,         // LLM model identifier
    pub history: Vec<Message>, // Conversation history
}

impl Agent {
    pub async fn prompt(&mut self, input: String) -> Result<String, AgentError> {
        // Add user message to history
        self.history.push(Message::user(input));
        
        // Call LLM with full context
        let response = self.llm_call().await?;
        
        // Add response to history
        self.history.push(Message::assistant(response.clone()));
        
        Ok(response)
    }
}
```

**Key Principles**:
- Single responsibility per agent
- Immutable system prompts for consistent behavior
- Conversation history management
- Error handling with `Result<T, E>`

### Entity-Component-System (ECS) Pattern

**Pattern**: Flexible agent composition using ECS architecture

```rust
// From agent-based models research
pub struct Agent {
    pub entity_id: EntityId,
    pub components: HashMap<ComponentType, Box<dyn Component>>,
}

pub trait Component: Send + Sync {
    fn update(&mut self, delta_time: f64, world: &World);
    fn component_type(&self) -> ComponentType;
}

// Example components
pub struct LLMComponent {
    pub model: String,
    pub temperature: f32,
}

pub struct ToolComponent {
    pub available_tools: Vec<String>,
}

pub struct StateComponent {
    pub variables: HashMap<String, Value>,
}
```

**Benefits for Rs-LLMSpell**:
- Modular agent capabilities
- Dynamic component composition
- Separation of concerns
- Easy testing and modification

## Plugin Architecture Patterns

### Service Composition Pattern

**Pattern**: Tower-rs inspired service composition

```rust
// From Apollo Router research
use tower::{Service, ServiceBuilder, Layer};

pub struct PluginLayer<T> {
    plugin: Arc<T>,
}

impl<T: Plugin> Layer<Service> for PluginLayer<T> {
    type Service = PluginService<Service, T>;
    
    fn layer(&self, service: Service) -> Self::Service {
        PluginService {
            inner: service,
            plugin: Arc::clone(&self.plugin),
        }
    }
}

// Service composition
let service = ServiceBuilder::new()
    .layer(AuthPlugin::layer())
    .layer(LoggingPlugin::layer())
    .layer(MetricsPlugin::layer())
    .service(core_service);
```

**Rs-LLMSpell Adaptation**:
```rust
pub struct AgentService {
    base_agent: Arc<dyn BaseAgent>,
    middleware: Vec<Arc<dyn AgentMiddleware>>,
}

#[async_trait]
pub trait AgentMiddleware: Send + Sync {
    async fn process_request(&self, input: &str, next: Next<'_>) 
        -> Result<AgentResponse, AgentError>;
}

// Middleware composition
impl AgentService {
    pub fn builder() -> AgentServiceBuilder {
        AgentServiceBuilder::new()
            .with_middleware(LoggingMiddleware::new())
            .with_middleware(MetricsMiddleware::new())
            .with_middleware(ValidationMiddleware::new())
    }
}
```

### Configuration-Driven Plugin System

**Pattern**: YAML/JSON configuration with schema validation

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub settings: serde_json::Value,
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    config: PluginRegistry,
}

impl PluginManager {
    pub fn from_config(config_path: &Path) -> Result<Self, ConfigError> {
        let config: PluginRegistry = serde_yaml::from_reader(
            File::open(config_path)?
        )?;
        
        let mut manager = Self::new();
        for plugin_config in config.plugins {
            manager.register_plugin(plugin_config)?;
        }
        
        Ok(manager)
    }
}
```

## Async State Management

### Arc<Mutex<T>> vs Tokio Async Primitives

**Standard Pattern**: Arc<Mutex<T>> for CPU-bound operations

```rust
use std::sync::{Arc, Mutex};

pub struct SharedState {
    data: Arc<Mutex<HashMap<String, Value>>>,
}

impl SharedState {
    pub fn update<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&mut HashMap<String, Value>) -> R 
    {
        let mut data = self.data.lock().unwrap();
        f(&mut *data)
    }
}
```

**Async Pattern**: Tokio primitives for I/O-bound operations

```rust
use tokio::sync::{RwLock, Mutex as AsyncMutex};

pub struct AsyncState {
    data: Arc<RwLock<HashMap<String, Value>>>,
}

impl AsyncState {
    pub async fn read<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&HashMap<String, Value>) -> R 
    {
        let data = self.data.read().await;
        f(&*data)
    }
    
    pub async fn write<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&mut HashMap<String, Value>) -> R 
    {
        let mut data = self.data.write().await;
        f(&mut *data)
    }
}
```

**Best Practices**:
1. Use `std::sync::Mutex` for short, CPU-bound critical sections
2. Use `tokio::sync::RwLock` for read-heavy, longer operations
3. Minimize lock scope to prevent blocking async tasks
4. Consider message-passing patterns for high contention

### Message Passing Alternative

**Pattern**: Actor-like message passing for state management

```rust
use tokio::sync::mpsc;

pub enum StateMessage {
    Get { key: String, response: oneshot::Sender<Option<Value>> },
    Set { key: String, value: Value },
    Update { key: String, updater: Box<dyn FnOnce(&mut Value) + Send> },
}

pub struct StateActor {
    receiver: mpsc::Receiver<StateMessage>,
    state: HashMap<String, Value>,
}

impl StateActor {
    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                StateMessage::Get { key, response } => {
                    let value = self.state.get(&key).cloned();
                    let _ = response.send(value);
                }
                StateMessage::Set { key, value } => {
                    self.state.insert(key, value);
                }
                StateMessage::Update { key, updater } => {
                    if let Some(value) = self.state.get_mut(&key) {
                        updater(value);
                    }
                }
            }
        }
    }
}
```

## Observer Pattern Implementations

### Dynamic Dispatch Observer

**Pattern**: Trait objects with shared ownership

```rust
pub trait Observer: Send + Sync {
    fn on_notify(&self, event: &Event);
}

pub struct EventSystem {
    observers: Arc<Mutex<Vec<Arc<dyn Observer>>>>,
}

impl EventSystem {
    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }
    
    pub fn notify(&self, event: &Event) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.on_notify(event);
        }
    }
}
```

**Weak Reference Pattern**: Avoiding memory leaks

```rust
use std::sync::Weak;

pub struct EventSystem {
    observers: Arc<Mutex<Vec<Weak<dyn Observer>>>>,
}

impl EventSystem {
    pub fn notify(&self, event: &Event) {
        let mut observers = self.observers.lock().unwrap();
        
        // Clean up dead observers and notify live ones
        observers.retain(|weak_observer| {
            if let Some(observer) = weak_observer.upgrade() {
                observer.on_notify(event);
                true // Keep observer
            } else {
                false // Remove dead observer
            }
        });
    }
}
```

### Async Observer Pattern

**Pattern**: Non-blocking async notifications

```rust
#[async_trait]
pub trait AsyncObserver: Send + Sync {
    async fn on_notify(&self, event: &Event) -> Result<(), ObserverError>;
}

pub struct AsyncEventSystem {
    observers: Arc<RwLock<Vec<Arc<dyn AsyncObserver>>>>,
}

impl AsyncEventSystem {
    pub async fn notify(&self, event: &Event) {
        let observers = self.observers.read().await;
        
        // Execute all observers concurrently
        let futures: Vec<_> = observers.iter()
            .map(|observer| observer.on_notify(event))
            .collect();
            
        // Wait for all observers to complete
        let results = futures::future::join_all(futures).await;
        
        // Handle any errors
        for result in results {
            if let Err(e) = result {
                eprintln!("Observer error: {}", e);
            }
        }
    }
}
```

## Performance Considerations

### Lock Contention Optimization

**Pattern**: Mutex sharding for high-concurrency scenarios

```rust
pub struct ShardedState {
    shards: Vec<Arc<Mutex<HashMap<String, Value>>>>,
    shard_count: usize,
}

impl ShardedState {
    pub fn new(shard_count: usize) -> Self {
        let shards = (0..shard_count)
            .map(|_| Arc::new(Mutex::new(HashMap::new())))
            .collect();
            
        Self { shards, shard_count }
    }
    
    fn get_shard(&self, key: &str) -> &Arc<Mutex<HashMap<String, Value>>> {
        let hash = self.hash_key(key);
        let shard_index = hash % self.shard_count;
        &self.shards[shard_index]
    }
    
    pub fn get(&self, key: &str) -> Option<Value> {
        let shard = self.get_shard(key);
        let data = shard.lock().unwrap();
        data.get(key).cloned()
    }
}
```

### Avoiding Async Mutex Overhead

**Pattern**: Dedicated task for state management

```rust
pub struct StateManager {
    sender: mpsc::Sender<StateCommand>,
}

enum StateCommand {
    Get { key: String, response: oneshot::Sender<Option<Value>> },
    Set { key: String, value: Value },
}

impl StateManager {
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::channel(1000);
        
        tokio::spawn(async move {
            let mut state = HashMap::new();
            let mut receiver = receiver;
            
            while let Some(cmd) = receiver.recv().await {
                match cmd {
                    StateCommand::Get { key, response } => {
                        let value = state.get(&key).cloned();
                        let _ = response.send(value);
                    }
                    StateCommand::Set { key, value } => {
                        state.insert(key, value);
                    }
                }
            }
        });
        
        Self { sender }
    }
}
```

## Recommendations for Rs-LLMSpell

### 1. Hook System Architecture

**Recommended Pattern**: Combine trait-based hooks with async execution

```rust
#[async_trait]
pub trait Hook: Send + Sync {
    fn id(&self) -> &str;
    fn priority(&self) -> u32;
    fn hook_points(&self) -> Vec<HookPoint>;
    
    async fn execute(&self, point: HookPoint, context: &HookContext) 
        -> Result<HookResult, HookError>;
}

pub struct HookManager {
    hooks: Arc<RwLock<BTreeMap<u32, Vec<Arc<dyn Hook>>>>>, // Priority-ordered
}

impl HookManager {
    pub async fn execute_hooks(&self, point: HookPoint, context: &HookContext) 
        -> Result<Vec<HookResult>, HookError> {
        let hooks = self.hooks.read().await;
        let mut results = Vec::new();
        
        // Execute hooks in priority order (highest first)
        for (_, hook_group) in hooks.iter().rev() {
            for hook in hook_group {
                if hook.hook_points().contains(&point) {
                    let result = hook.execute(point.clone(), context).await?;
                    results.push(result);
                    
                    // Stop execution if hook indicates to halt
                    if !result.continue_execution {
                        break;
                    }
                }
            }
        }
        
        Ok(results)
    }
}
```

### 2. State Management Strategy

**Recommended Approach**: Hybrid pattern based on use case

```rust
// For agent state: Use Arc<RwLock<T>> for read-heavy operations
pub struct AgentState {
    variables: Arc<RwLock<HashMap<String, StateValue>>>,
    metadata: Arc<RwLock<StateMetadata>>,
}

// For event system: Use message passing for high throughput
pub struct EventBus {
    sender: mpsc::UnboundedSender<Event>,
}

// For configuration: Use Arc<Mutex<T>> for simple updates
pub struct Config {
    settings: Arc<Mutex<ConfigData>>,
}
```

### 3. Agent Architecture

**Recommended Pattern**: Composition over inheritance with ECS-inspired design

```rust
pub struct BaseAgent {
    id: String,
    capabilities: HashMap<String, Box<dyn Capability>>,
    state: Arc<RwLock<AgentState>>,
    hooks: Arc<HookManager>,
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

pub trait Capability: Send + Sync {
    fn capability_type(&self) -> &str;
    async fn execute(&self, context: &CapabilityContext) -> Result<CapabilityResult, CapabilityError>;
}

// Capabilities can be composed dynamically
impl BaseAgent {
    pub fn with_capability<C: Capability + 'static>(mut self, capability: C) -> Self {
        self.capabilities.insert(
            capability.capability_type().to_string(),
            Box::new(capability)
        );
        self
    }
}
```

### 4. Tool System Design

**Recommended Pattern**: Trait objects with builder pattern

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> &ToolSchema;
    
    async fn execute(&self, input: ToolInput, context: ToolContext) 
        -> Result<ToolResult, ToolError>;
    
    async fn validate(&self, input: &ToolInput) -> Result<(), ToolError>;
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ToolRegistry {
    pub fn builder() -> ToolRegistryBuilder {
        ToolRegistryBuilder::new()
            .with_builtin_tools()
            .with_category_indexing()
    }
}
```

### 5. Error Handling Strategy

**Recommended Pattern**: Hierarchical error types with context

```rust
#[derive(Debug, thiserror::Error)]
pub enum RsLlmSpellError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Hook error: {0}")]
    Hook(#[from] HookError),
    
    #[error("State error: {0}")]
    State(#[from] StateError),
}

// With context preservation
impl From<AgentError> for RsLlmSpellError {
    fn from(err: AgentError) -> Self {
        Self::Agent(err)
    }
}
```

## Key Takeaways

1. **Use trait-based design** for maximum flexibility and composability
2. **Prefer async/await** for I/O-bound operations, std sync primitives for CPU-bound
3. **Implement priority-based hook execution** for predictable behavior
4. **Use message passing** for high-contention scenarios
5. **Design for composition** rather than inheritance
6. **Prioritize error context** and recovery patterns
7. **Consider performance implications** of dynamic dispatch and locking
8. **Implement graceful degradation** when hooks or tools fail

This research provides a solid foundation for implementing robust, performant, and maintainable systems in rs-llmspell.