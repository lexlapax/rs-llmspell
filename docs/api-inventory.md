# rs-llmspell Public API Inventory

**Generated**: August 1, 2025
**Purpose**: Complete inventory of all public APIs for standardization review

## Overview

This document catalogs all public APIs across the rs-llmspell workspace, organized by crate and categorized by function type.

---

## Core Crates

### llmspell-core

#### Traits
```rust
// Base trait hierarchy
pub trait BaseAgent: Send + Sync
pub trait Agent: BaseAgent
pub trait Tool: BaseAgent + ToolCapabilities
pub trait Workflow: Send + Sync

// Capabilities
pub trait ToolCapabilities
pub trait ConversationCapabilities
pub trait ReActCapabilities
```

#### Structs
```rust
pub struct AgentCapabilities
pub struct ProviderConfig
pub struct ToolInfo
pub struct ToolParameter
```

#### Enums
```rust
pub enum AgentType
pub enum ParameterType
pub enum LLMSpellError
```

#### Functions/Methods
- Constructor patterns:
  - `BaseAgent::id()`, `name()`, `description()` (trait methods)
  - `Agent::query()`, `stream_query()` (async trait methods)
  - `Tool::execute()` (async trait method)
  - `Workflow::execute()` (async trait method)

---

### llmspell-agents

#### Structs
```rust
pub struct DefaultAgent
pub struct DefaultAgentFactory
pub struct AgentFactory
pub struct AgentRegistry
```

#### Factory Methods
```rust
// AgentFactory
pub fn new() -> Self
pub fn with_registry(registry: Arc<AgentRegistry>) -> Self
pub async fn create_agent(config: AgentConfig) -> Result<Arc<dyn Agent>>
pub fn register_factory(agent_type: &str, factory: Arc<dyn AgentFactoryTrait>)

// DefaultAgentFactory  
pub fn new() -> Self
pub async fn create(&self, config: AgentConfig) -> Result<Box<dyn Agent>>
```

#### Registry Methods
```rust
// AgentRegistry
pub fn new() -> Self
pub fn register(agent_type: &str, factory: Arc<dyn AgentFactoryTrait>)
pub fn get(agent_type: &str) -> Option<Arc<dyn AgentFactoryTrait>>
pub fn list() -> Vec<String>
```

---

### llmspell-tools

#### Tool Implementations (150+ tools)
All follow pattern: `pub struct *Tool` implementing `Tool` trait

Examples:
```rust
pub struct FileReadTool
pub struct FileWriteTool
pub struct HttpGetTool
pub struct JsonParseTool
pub struct RegexMatchTool
// ... 145+ more
```

#### Common Tool Methods
```rust
// All tools implement
impl Tool for *Tool {
    async fn execute(&self, params: Value) -> Result<Value>
}

// Most tools have
impl *Tool {
    pub fn new() -> Self
}
```

---

### llmspell-workflows

#### Workflow Types
```rust
pub struct SequentialWorkflow
pub struct ParallelWorkflow
pub struct ConditionalWorkflow
pub struct LoopWorkflow
pub struct MapReduceWorkflow
```

#### Builder Pattern
```rust
pub struct WorkflowBuilder
impl WorkflowBuilder {
    pub fn new(name: String) -> Self
    pub fn add_step(mut self, step: WorkflowStep) -> Self
    pub fn with_timeout(mut self, timeout: Duration) -> Self
    pub fn build(self) -> Result<Box<dyn Workflow>>
}
```

---

## Infrastructure Crates

### llmspell-storage

#### Traits
```rust
pub trait StorageBackend: Send + Sync
pub trait StorageFactory
```

#### Implementations
```rust
pub struct FileSystemBackend
pub struct MemoryBackend
pub struct S3Backend

// Methods
impl StorageBackend for * {
    async fn read(&self, key: &str) -> Result<Vec<u8>>
    async fn write(&self, key: &str, data: &[u8]) -> Result<()>
    async fn delete(&self, key: &str) -> Result<()>
    async fn exists(&self, key: &str) -> Result<bool>
    async fn list(&self, prefix: &str) -> Result<Vec<String>>
}
```

---

### llmspell-hooks

#### Core Types
```rust
pub struct HookRegistry
pub struct HookExecutor  // Note: Currently HookExecutorService
pub struct HookContext
```

#### Enums
```rust
pub enum HookPoint
pub enum HookPriority
```

#### Methods
```rust
// HookRegistry
pub fn new() -> Self
pub fn register(point: HookPoint, hook: Box<dyn Hook>)
pub fn unregister(point: HookPoint, id: &str)
pub fn get_hooks(point: HookPoint) -> Vec<Arc<dyn Hook>>

// HookExecutor (currently HookExecutorService)
pub fn new() -> Self
pub async fn execute(point: HookPoint, context: HookContext) -> Result<()>
```

---

### llmspell-events

#### Core Types
```rust
pub struct EventBus  // Note: Currently EventBusService
pub struct UniversalEvent
pub struct EventSubscription
```

#### Methods
```rust
// EventBus (currently EventBusService)
pub fn new() -> Self
pub async fn publish(event: UniversalEvent) -> Result<()>
pub fn subscribe(filter: EventFilter) -> EventSubscription
pub fn unsubscribe(subscription_id: &str)
```

---

### llmspell-state-persistence

#### Core Types
```rust
pub struct StateManager
pub struct StateEntry
pub struct MigrationPlan
```

#### Enums
```rust
pub enum StateScope
pub enum StatePersistence
```

#### Methods
```rust
// StateManager
pub async fn new() -> Result<Self>
pub async fn get(scope: StateScope, key: &str) -> Result<Option<Value>>
pub async fn set(scope: StateScope, key: &str, value: Value) -> Result<()>
pub async fn delete(scope: StateScope, key: &str) -> Result<()>
pub async fn list(scope: StateScope) -> Result<Vec<String>>
```

---

### llmspell-sessions

#### Core Types
```rust
pub struct SessionManager
pub struct Session
pub struct SessionId
pub struct ArtifactManager
pub struct Artifact
pub struct ArtifactId
```

#### Config Types
```rust
pub struct SessionManagerConfig  // Needs builder
pub struct CreateSessionOptions
pub struct SessionConfig
```

#### Methods
```rust
// SessionManager
pub fn new(...) -> Result<Self>  // Complex params
pub async fn create_session(options: CreateSessionOptions) -> Result<SessionId>
pub async fn get_session(id: &SessionId) -> Result<Session>  // Note: retrieve_session also exists
pub async fn suspend_session(id: &SessionId) -> Result<()>
pub async fn resume_session(id: &SessionId) -> Result<()>
pub async fn complete_session(id: &SessionId) -> Result<()>

// Session
pub async fn id() -> SessionId
pub async fn status() -> SessionStatus
pub async fn suspend() -> Result<()>
pub async fn resume() -> Result<()>
```

---

## Bridge and Utility Crates

### llmspell-bridge

#### Runtime Types
```rust
pub struct ScriptRuntime
pub struct RuntimeBuilder
```

#### Global Objects
```rust
pub trait GlobalObject
pub struct AgentGlobal
pub struct ToolGlobal
pub struct SessionGlobal
pub struct ArtifactGlobal
pub struct StateGlobal
```

#### Bridge Traits
```rust
pub trait ScriptEngineBridge
pub trait LanguageRuntime
```

---

### llmspell-providers

#### Provider Types
```rust
pub struct OpenAIProvider
pub struct AnthropicProvider
pub struct OllamaProvider
pub struct MockProvider
```

#### Common Interface
```rust
impl Provider for * {
    async fn complete(request: CompletionRequest) -> Result<CompletionResponse>
    async fn stream_complete(request: CompletionRequest) -> Result<CompletionStream>
}
```

---

### llmspell-config

#### Config Management
```rust
pub struct ConfigManager  // Currently stub
pub struct ConfigLoader
```

---

### llmspell-security

#### Security Types
```rust
pub struct SecurityManager
pub struct SecurityContext
pub struct PermissionSet
```

---

### llmspell-utils

#### Utility Functions
```rust
// Path utilities
pub fn expand_tilde(path: &str) -> PathBuf
pub fn ensure_directory(path: &Path) -> Result<()>

// String utilities  
pub fn truncate_string(s: &str, max_len: usize) -> String

// JSON utilities
pub fn json_to_string_safe(value: &Value) -> String
```

---

## Naming Pattern Analysis

### Consistent Patterns ✅
1. **Tool suffix**: All tools end with `Tool`
2. **Manager suffix**: Most services use `Manager`
3. **Config suffix**: Configuration types use `Config`
4. **Builder suffix**: Builder patterns use `Builder`
5. **Registry suffix**: Component registries use `Registry`
6. **Global suffix**: Script globals use `Global`

### Inconsistencies Found ⚠️

#### 1. Service vs Manager
- ❌ `HookExecutorService` → should be `HookExecutor` or `HookManager`
- ❌ `EventBusService` → should be `EventBus` or `EventManager`

#### 2. Method Naming
- ❌ `retrieve_session()` vs `get_session()` - inconsistent accessor naming
- ❌ `with_registry()` vs `from_registry()` - inconsistent constructor naming

#### 3. Async Method Patterns
- ✅ Most async methods properly marked
- ⚠️ Some missing explicit `Send + Sync` bounds in trait definitions

#### 4. Constructor Patterns
- ✅ Simple types use `new()`
- ⚠️ Complex types lack builder patterns:
  - `SessionManagerConfig`
  - `WorkflowConfig`
  - `AgentConfig`

---

## Functionality Grouping

### Creation/Construction
- `new()` - Simple construction (120+ occurrences)
- `with_*()` - Parameterized construction (15+ variants)
- `from_*()` - Conversion construction (8+ variants)
- `builder()` - Builder pattern initiation (3 occurrences)
- `create_*()` - Factory creation (12+ variants)

### Access/Retrieval
- `get()`, `get_*()` - Immutable access (50+ variants)
- `retrieve_*()` - Alternative getter (should standardize)
- `list()`, `list_*()` - Collection access (20+ variants)
- `find_*()` - Search operations (5+ variants)

### Mutation
- `set()`, `set_*()` - Value updates (15+ variants)
- `add_*()` - Collection additions (10+ variants)
- `register()` - Component registration (8+ variants)
- `update_*()` - State updates (12+ variants)

### Lifecycle
- `start()`, `stop()` - Service lifecycle (4 pairs)
- `suspend()`, `resume()` - Pausable lifecycle (3 pairs)
- `init()`, `cleanup()` - Initialization lifecycle (2 pairs)

### Async Operations
- `*_async()` suffix - Rarely used (Rust convention is implicit)
- `await` points - Properly marked throughout

---

## Builder Pattern Opportunities

### High Priority (Complex Construction)
1. `SessionManagerConfig` - 8+ fields
2. `WorkflowConfig` - 6+ fields  
3. `AgentConfig` - 10+ fields
4. `StateManagerConfig` - 5+ fields

### Medium Priority (Moderate Complexity)
1. `ToolConfig` - 4+ fields
2. `HookConfig` - 4+ fields
3. `EventFilter` - 5+ fields

### Low Priority (Simple Construction)
1. `SecurityContext` - 3 fields
2. `ProviderConfig` - 3 fields

---

## Next Steps

1. Create API style guide based on findings
2. Plan refactoring tasks by priority
3. Design migration strategy for breaking changes
4. Document deprecation approach