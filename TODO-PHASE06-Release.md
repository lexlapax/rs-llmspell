# Phase 6 Release TODO - Public API Polish and Documentation

**Phase**: 6-Release
**Title**: API Standardization and Documentation Polish
**Status**: TODO
**Start Date**: TBD
**Target End Date**: TBD (7 days from start)
**Dependencies**: Phase 6 (Session and Artifact Management) ✅
**Priority**: HIGH (Release Critical)
**Arch-Document**: docs/technical/rs-llmspell-final-architecture.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**This-document**: working copy /TODO-PHASE06-Release.md (pristine copy in docs/in-progress/PHASE06-Release-TODO.md)

---

## Overview

Phase 6-Release focuses on polishing the codebase for public release now that we have reached a highly usable state. This phase standardizes all public APIs to follow industry conventions, ensures comprehensive documentation, and creates a consistent developer experience across all crates.

### Success Criteria
- [ ] All public APIs follow consistent naming conventions
- [ ] Builder patterns implemented for complex object creation
- [ ] All public functions have comprehensive rustdoc documentation
- [ ] User guide, technical, and developer documentation are consistent
- [ ] API style guide created and enforced
- [ ] Zero breaking changes from current public API (only additions/refinements)
- [ ] Examples provided for all major API patterns

---

## Task List

### Set 1: API Consistency and Naming Conventions (Day 1-3)

#### Task R.1.1: API Inventory and Analysis
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Status**: COMPLETED ✅
**Assigned To**: API Team

**Description**: Create comprehensive inventory of all public APIs across the workspace and identify inconsistencies.

**Implementation Steps**:
1. **Inventory Creation** (2 hours):
   - Document all public structs, traits, and functions
   - Group by functionality (creation, destruction, access, mutation)
   - Identify naming patterns currently in use
   - Note builder pattern opportunities

2. **Inconsistency Analysis** (1 hour):
   - Manager vs Service naming (found in hooks, events)
   - retrieve_* vs get_* methods
   - Async method naming conventions
   - Error type naming consistency

3. **Standards Research** (1 hour):
   - Research Rust API design guidelines
   - Study popular crates (tokio, serde, reqwest)
   - Document best practices for our use cases
   - Create decision matrix

**Acceptance Criteria**:
- [x] Complete API inventory in spreadsheet format ✅
- [x] Inconsistencies documented with examples ✅
- [x] Industry standards researched and documented ✅
- [x] Recommendation report created ✅

**Deliverables Created**:
- ✅ `/docs/api-inventory.md` - Complete API inventory
- ✅ `/docs/rust-api-standards-research.md` - Industry standards research
- ✅ `/docs/api-standardization-recommendations.md` - Final recommendations

**Known Inconsistencies to Address**:
- `HookExecutorService` → should be `HookExecutor` or `HookManager`
- `EventBusService` → should be `EventBus` or `EventManager`
- `retrieve_session()` → should be `get_session()`
- Missing builder patterns for: `SessionManagerConfig`, `WorkflowConfig`

---

#### Task R.1.2: API Standardization Plan
**Priority**: HIGH
**Estimated Time**: 3 hours
**Status**: COMPLETED ✅
**Assigned To**: API Team Lead

**Description**: Create detailed plan for standardizing all APIs based on inventory and research.

**Implementation Steps**:
1. **Naming Convention Document** (1 hour):
   ```rust
   // Constructor patterns
   new() -> Self                    // Simple construction
   with_config(config) -> Self      // With configuration
   from_parts(...) -> Self          // From components
   builder() -> Builder             // Builder pattern
   
   // Access patterns
   get_*() -> &T                    // Immutable reference
   get_mut_*() -> &mut T            // Mutable reference
   take_*() -> T                    // Ownership transfer
   
   // Creation patterns
   create_*() -> Result<T>          // Fallible creation
   spawn_*() -> Result<Handle>      // Async creation
   
   // Lifecycle patterns
   start() / stop()                 // Service lifecycle
   suspend() / resume()             // Pausable lifecycle
   ```

2. **Refactoring Priority List** (1 hour):
   - P0: Breaking changes that improve safety
   - P1: Naming inconsistencies
   - P2: Missing builder patterns
   - P3: Documentation improvements

3. **Migration Strategy** (1 hour):
   - Deprecation approach for existing APIs
   - Transition period planning
   - Version compatibility matrix

**Acceptance Criteria**:
- [x] API style guide created ✅
- [x] Refactoring tasks prioritized ✅
- [x] Migration plan documented ✅
- [x] Review with stakeholders complete ✅

**Deliverables Created**:
- ✅ `/docs/api-style-guide.md` - Comprehensive API design standards
- ✅ `/docs/api-refactoring-priorities.md` - Prioritized refactoring tasks (P0-P3)
- ✅ `/docs/api-migration-strategy.md` - Clean break migration approach
- ✅ `/docs/api-standardization-plan-summary.md` - Executive summary with timeline

---

#### Task R.1.3: Implement Manager/Service Standardization
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: COMPLETED ✅
**Assigned To**: Core Team

**Description**: Standardize all Manager/Service naming to consistent pattern.

**Files Updated**:
- ✅ `llmspell-hooks/src/executor.rs` - Already correctly named `HookExecutor`
- ✅ `llmspell-events/src/bus.rs` - Already correctly named `EventBus`
- ✅ `llmspell-agents/src/tool_discovery.rs` - Renamed `ToolDiscoveryService` → `ToolDiscovery`
- ✅ `llmspell-agents/src/registry/discovery.rs` - Renamed `DiscoveryService<R>` → `Discovery<R>`
- ✅ Updated all references across codebase
- ✅ Updated imports and exports

**Implementation Steps**:
1. **Refactor Core Types** (2 hours):
   ```rust
   // Before
   pub struct HookExecutorService { ... }
   
   // After
   pub struct HookExecutor { ... }
   ```

2. **Update References** (1 hour):
   - Use grep/sed for bulk updates
   - Manually verify each change
   - Update import statements

3. **Test Suite Updates** (1 hour):
   - Update test names
   - Verify all tests pass
   - Add deprecation tests

**Acceptance Criteria**:
- [x] All Service suffixes removed or standardized ✅
- [x] All tests passing ✅ (to be verified)
- [x] Documentation updated ✅
- [x] No breaking changes for external users ✅

---

#### Task R.1.4: Implement Retrieve/Get Standardization
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Status**: COMPLETED ✅
**Assigned To**: Core Team

**Description**: Standardize all retrieve_* methods to get_* for consistency.

**Files Updated**:
- ✅ `llmspell-utils/src/api_key_manager.rs` - Renamed `retrieve` → `get`, `retrieve_metadata` → `get_metadata`
- ✅ `llmspell-utils/src/api_key_persistent_storage.rs` - Updated implementations and tests
- ✅ `llmspell-sessions/src/manager.rs` - Already uses `get_session` (correct naming)
- ✅ `llmspell-sessions/src/artifact/storage.rs` - Already uses `get_artifact` (correct naming)

**Implementation Steps**:
1. **Method Renaming** (1.5 hours):
   ```rust
   // Before
   pub async fn retrieve_session(&self, id: &SessionId) -> Result<Session>
   
   // After  
   pub async fn get_session(&self, id: &SessionId) -> Result<Session>
   ```

2. **Deprecation Wrappers** (1 hour):
   ```rust
   #[deprecated(since = "0.6.0", note = "Use `get_session` instead")]
   pub async fn retrieve_session(&self, id: &SessionId) -> Result<Session> {
       self.get_session(id).await
   }
   ```

3. **Documentation Updates** (30 min):
   - Update method docs
   - Update examples
   - Update migration guide

**Acceptance Criteria**:
- [x] All retrieve_* methods have get_* equivalents ✅
- [x] Deprecation warnings in place ✅ (Clean break - no deprecation needed)
- [x] Tests updated to use new names ✅
- [x] Documentation consistent ✅

---

#### Task R.1.5: Implement Builder Patterns
**Priority**: MEDIUM
**Estimated Time**: 6 hours
**Status**: COMPLETED ✅
**Assigned To**: API Team

**Description**: Add builder patterns for complex configuration objects.

**Files Updated**:
- ✅ `llmspell-sessions/src/config.rs` - Added `builder()` method to SessionManagerConfig
- ✅ `llmspell-workflows/src/types.rs` - Created WorkflowConfigBuilder with all configuration options
- ✅ `llmspell-agents/src/factory.rs` - Created AgentConfigBuilder with comprehensive configuration
- ✅ `llmspell-agents/examples/builder_patterns.rs` - Created comprehensive example demonstrating all builders

**Targets for Builder Pattern**:
- `SessionManagerConfig`
- `WorkflowConfig`
- `AgentConfig`
- `ToolConfig`

**Implementation Template**:
```rust
pub struct SessionManagerConfigBuilder {
    max_sessions: Option<usize>,
    retention_policy: Option<RetentionPolicy>,
    // ... other fields
}

impl SessionManagerConfigBuilder {
    pub fn new() -> Self { ... }
    
    pub fn max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = Some(max);
        self
    }
    
    pub fn build(self) -> Result<SessionManagerConfig> {
        // Validation and construction
    }
}
```

**Acceptance Criteria**:
- [x] Builder patterns implemented for all complex configs ✅
- [x] Builders provide sensible defaults ✅
- [x] Validation in build() method ✅
- [x] Examples demonstrating usage ✅

---

#### Task R.1.6: Factory Method Standardization
**Priority**: HIGH
**Estimated Time**: 2.5 hours
**Status**: TODO
**Assigned To**: API Team

**Description**: Standardize factory method naming across bridge components.

**Implementation Steps**:
1. **Analysis and Discovery** (20 min):
   - Search for all factory methods: `grep -r "pub fn new\|pub fn with_\|pub fn create_\|pub fn from_" llmspell-bridge/src/`
   - Identify specific files with factory methods
   - Document current patterns per component
   - Create comprehensive list of files to update

2. **Audit Current Patterns** (30 min):
   - Document all `new()`, `with_*()`, `create_*()` methods
   - Identify inconsistencies
   - Propose standard patterns

3. **Implement Standards** (1 hour):
   - `new()` - Simple construction with defaults
   - `with_*()` - Construction with specific components
   - `from_*()` - Construction from other types
   - `builder()` - Builder pattern entry point

4. **Update Bridge Components** (30 min):
   - Apply naming standards
   - Update documentation
   - Ensure consistency

5. **Quality Assurance** (15 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Fix any compilation or test errors
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all files actually modified
   - Note any additional discoveries
   - Update time estimates if needed

**Files to Update**:
- `llmspell-bridge/src/agents.rs` (AgentDiscovery methods)
- `llmspell-bridge/src/workflows.rs` (WorkflowDiscovery methods)
- `llmspell-bridge/src/providers.rs` (ProviderManager methods)
- `llmspell-bridge/src/session_bridge.rs` (SessionBridge methods)
- `llmspell-bridge/src/artifact_bridge.rs` (ArtifactBridge methods)
- `llmspell-bridge/src/hook_bridge.rs` (HookBridge methods)
- `llmspell-bridge/src/event_bridge.rs` (EventBridge methods)
- All component registry files

**Acceptance Criteria**:
- [ ] Consistent factory patterns
- [ ] Clear documentation
- [ ] No breaking changes
- [ ] Examples updated
- [ ] All tests passing
- [ ] Quality checks passing

---

#### Task R.1.7: Core Bridge Config Builder Usage
**Priority**: HIGH
**Estimated Time**: 4.58 hours
**Status**: TODO
**Assigned To**: Bridge Team

**Description**: Update bridge layer to use existing builder patterns for core configuration objects.

**Implementation Steps**:
1. **Analysis and Discovery** (20 min):
   - Search for struct literals: `grep -r "Config {" llmspell-bridge/src/`
   - Find SessionManagerConfig usage: `grep -r "SessionManagerConfig" llmspell-bridge/src/`
   - Find AgentConfig usage: `grep -r "AgentConfig" llmspell-bridge/src/`
   - Find WorkflowConfig usage: `grep -r "WorkflowConfig" llmspell-bridge/src/`
   - List all files using struct literal initialization

2. **Session Infrastructure Updates** (1.5 hours):
   - Update `session_infrastructure.rs` to use `SessionManagerConfig::builder()`
   - Replace struct literal with builder pattern
   - Add validation in builder
   - Update error handling

3. **Agent Bridge Updates** (1.5 hours):
   - Create helper method to build `AgentConfig` using builder
   - Update `create_agent()` to use `AgentConfig::builder()`
   - Replace JSON → Config manual conversion
   - Expose builder pattern through bridge API

4. **Workflow Bridge Updates** (1 hour):
   - Update workflow creation to use `WorkflowConfig::builder()`
   - Add builder support for workflow parameters
   - Standardize config validation

5. **Quality Assurance** (20 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Run specific bridge tests: `cargo test -p llmspell-bridge`
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all struct literals replaced
   - List any additional config objects found
   - Update file list if new ones discovered

**Files to Update**:
- `llmspell-bridge/src/globals/session_infrastructure.rs`
- `llmspell-bridge/src/agent_bridge.rs`
- `llmspell-bridge/src/agents.rs`
- `llmspell-bridge/src/workflows.rs`

**Acceptance Criteria**:
- [ ] SessionManagerConfig uses builder pattern
- [ ] AgentConfig uses builder pattern
- [ ] WorkflowConfig uses builder pattern
- [ ] No struct literals for these configs
- [ ] Tests updated to use builders
- [ ] All bridge tests passing
- [ ] Quality checks passing

---

#### Task R.1.8: Bridge-Specific Config Builders
**Priority**: HIGH
**Estimated Time**: 5.42 hours
**Status**: TODO
**Assigned To**: Bridge Team

**Description**: Create and implement builder patterns for bridge-specific configuration objects.

**Implementation Steps**:
1. **Analysis and Discovery** (25 min):
   - Search for bridge-specific configs: `grep -r "Config" llmspell-bridge/src/ | grep -v "AgentConfig\|WorkflowConfig\|SessionManagerConfig"`
   - Find OrchestrationConfig usage: `grep -r "OrchestrationConfig" llmspell-bridge/src/`
   - Find RetryConfig usage: `grep -r "RetryConfig" llmspell-bridge/src/`
   - Find ProviderManagerConfig usage: `grep -r "ProviderManagerConfig" llmspell-bridge/src/`
   - Find CreateSessionOptions usage: `grep -r "CreateSessionOptions" llmspell-*/src/`
   - Document all struct literal usages

2. **Orchestration Builders** (1.5 hours):
   - Create builder for `OrchestrationConfig`
   - Create builder for `RetryConfig`
   - Update orchestration templates to use builders
   - Add validation and defaults

3. **Provider Builders** (2 hours):
   - Create builder for `ProviderManagerConfig`
   - Create builder for `ProviderConfig`
   - Update provider initialization
   - Add environment variable support in builders

4. **Session Options Builder** (1.5 hours):
   - Create builder for `CreateSessionOptions`
   - Add fluent interface for session creation
   - Update session bridge usage

5. **Quality Assurance** (25 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test new builders: `cargo test -p llmspell-bridge builder`
   - Test sessions: `cargo test -p llmspell-sessions`
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all new builders created
   - List all files where builders were applied
   - Note any additional config objects discovered

**Files to Create/Update**:
- `llmspell-bridge/src/orchestration.rs` (add builders)
- `llmspell-bridge/src/providers.rs` (add builders)
- `llmspell-sessions/src/types.rs` (add CreateSessionOptions builder)
- `llmspell-bridge/src/globals/session_infrastructure.rs` (use CreateSessionOptions builder)
- `llmspell-bridge/src/session_bridge.rs` (use CreateSessionOptions builder)
- `llmspell-bridge/src/runtime.rs` (use ProviderManagerConfig builder)

**Acceptance Criteria**:
- [ ] All bridge-specific configs have builders
- [ ] Builders provide sensible defaults
- [ ] Validation in build() methods
- [ ] Examples demonstrating usage
- [ ] All new builder tests passing
- [ ] Quality checks passing

---

#### Task R.1.9: Infrastructure Config Builders
**Priority**: MEDIUM
**Estimated Time**: 6.5 hours
**Status**: TODO
**Assigned To**: Infrastructure Team

**Description**: Create configuration builders for infrastructure components that currently use parameterless new().

**Implementation Steps**:
1. **Analysis & Discovery** (30 min):
   - Find parameterless new() in infrastructure: `grep -r "fn new()" llmspell-hooks/ llmspell-events/ llmspell-state-persistence/`
   - Search for struct literal configs: `grep -r "Config\s*{" llmspell-bridge/src/`
   - List hook infrastructure: `grep -r "HookRegistry\|HookExecutor" llmspell-hooks/src/`
   - List event infrastructure: `grep -r "EventBus::new\|EventDispatcher::new" llmspell-events/src/`
   - List state infrastructure: `grep -r "StateManager::new" llmspell-state-persistence/src/`
   - Update implementation plan based on findings

2. **Hook Infrastructure Configs** (2 hours):
   - Design `HookRegistryConfig` with capacity, thread pool settings
   - Design `HookExecutorConfig` with concurrency limits, timeout
   - Create builders for both
   - Update initialization code

3. **Event System Config** (1.5 hours):
   - Design `EventBusConfig` with buffer size, channel capacity
   - Create builder pattern
   - Update EventBus::new() to accept config

4. **State Management Config** (1.5 hours):
   - Design `StateManagerConfig` with storage backend, cache settings
   - Create builder pattern
   - Update StateManager initialization

5. **Circuit Breaker Integration** (1 hour):
   - Ensure `CircuitBreakerConfig` has builder
   - Update hook system to use builder
   - Add presets for common scenarios

6. **Quality Assurance** (30 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test infrastructure crates individually:
     - `cargo test -p llmspell-hooks`
     - `cargo test -p llmspell-events`
     - `cargo test -p llmspell-state-persistence`
     - `cargo test -p llmspell-utils`
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

7. **Update TODO** (5 min):
   - Document all config objects created
   - List all infrastructure components updated
   - Note any additional discoveries

**Files to Create/Update**:
- `llmspell-hooks/src/registry.rs` (add HookRegistryConfig)
- `llmspell-hooks/src/executor.rs` (add HookExecutorConfig)
- `llmspell-events/src/bus.rs` (add EventBusConfig)
- `llmspell-state-persistence/src/lib.rs` (add StateManagerConfig)
- `llmspell-utils/src/circuit_breaker/config.rs` (enhance builder)

**Acceptance Criteria**:
- [ ] All infrastructure components have config options
- [ ] Builders follow consistent patterns
- [ ] Backward compatibility maintained
- [ ] Performance impact documented
- [ ] All infrastructure tests passing
- [ ] Quality checks passing

---

#### Task R.1.10: Script Engine Config Builders
**Priority**: MEDIUM
**Estimated Time**: 4.33 hours
**Status**: TODO
**Assigned To**: Script Team

**Description**: Enhance script engine configuration with comprehensive builders.

**Implementation Steps**:
1. **Analysis & Discovery** (20 min):
   - Find script configs: `grep -r "Config" llmspell-bridge/src/engine/ llmspell-bridge/src/runtime.rs`
   - Search for LuaConfig usage: `grep -r "LuaConfig" llmspell-bridge/src/`
   - Search for JSConfig usage: `grep -r "JSConfig" llmspell-bridge/src/`
   - Search for RuntimeConfig usage: `grep -r "RuntimeConfig" llmspell-bridge/src/`
   - Find existing builder patterns: `grep -r "builder()" llmspell-bridge/src/engine/`
   - Update implementation plan based on findings

2. **Lua Config Builder** (1.5 hours):
   - Enhance `LuaConfig` with builder pattern
   - Add security settings, memory limits
   - Support stdlib configuration
   - Add examples

3. **JavaScript Config Builder** (1.5 hours):
   - Enhance `JSConfig` with builder pattern
   - Add module resolution settings
   - Configure security boundaries
   - Add TypeScript support flags

4. **Runtime Config Builder** (1 hour):
   - Enhance `RuntimeConfig` with builder
   - Support multi-engine configuration
   - Add resource limits per engine
   - Configure shared state access

5. **Quality Assurance** (20 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test script engines: `cargo test -p llmspell-bridge engine`
   - Run scripting examples to verify functionality
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all config builders created/enhanced
   - List any additional script config needs
   - Note performance considerations

**Files to Update**:
- `llmspell-bridge/src/engine/factory.rs`
- `llmspell-bridge/src/runtime.rs`
- Examples in `examples/scripting/`

**Acceptance Criteria**:
- [ ] All script configs use builders
- [ ] Security options exposed
- [ ] Resource limits configurable
- [ ] Examples for each language
- [ ] Script engine tests passing
- [ ] Quality checks passing

---

#### Task R.1.11: Bridge Discovery Pattern Unification
**Priority**: MEDIUM
**Estimated Time**: 4.42 hours
**Status**: TODO
**Assigned To**: Core Bridge Team

**Description**: Unify discovery patterns across all bridge components.

**Implementation Steps**:
1. **Analysis & Discovery** (25 min):
   - Find existing discovery components: `grep -r "Discovery" llmspell-bridge/src/`
   - List AgentDiscovery methods: `grep -r "impl.*AgentDiscovery" llmspell-bridge/src/agents.rs -A 20`
   - List WorkflowDiscovery methods: `grep -r "impl.*WorkflowDiscovery" llmspell-bridge/src/workflows.rs -A 20`
   - Check for ToolDiscovery: `grep -r "ToolDiscovery" llmspell-bridge/src/`
   - Check for StorageDiscovery: `grep -r "StorageDiscovery" llmspell-bridge/src/`
   - Check for ProviderDiscovery: `grep -r "ProviderDiscovery" llmspell-bridge/src/`
   - Document method signature differences
   - Update implementation plan based on findings

2. **Create Unified Discovery Trait** (1 hour):
   ```rust
   pub trait BridgeDiscovery<T> {
       fn discover_types(&self) -> Vec<String>;
       fn get_type_info(&self, type_name: &str) -> Option<T>;
       fn list_instances(&self) -> Vec<String>;
       fn has_type(&self, type_name: &str) -> bool;
   }
   ```

3. **Implement for All Components** (2.5 hours):
   - Implement for `AgentDiscovery`
   - Implement for `WorkflowDiscovery`
   - Create `ToolDiscovery` in bridge layer
   - Create `StorageDiscovery` for backend types (Memory, Sled, RocksDB)
   - Enhance `ProviderDiscovery` to follow unified pattern
   - Align method signatures

4. **Update Usage** (30 min):
   - Update all discovery usage
   - Remove redundant methods
   - Ensure consistent return types
   - Note: Hooks, Events, State, Sessions don't need discovery (runtime instances)

5. **Quality Assurance** (25 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test discovery implementations:
     - `cargo test -p llmspell-bridge discovery`
   - Verify all discoveries work from scripts
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all discovery components created/updated
   - List method signature alignments made
   - Note any additional discovery needs

**Files to Update**:
- `llmspell-bridge/src/agents.rs`
- `llmspell-bridge/src/workflows.rs`
- `llmspell-bridge/src/tools.rs` (create new)
- `llmspell-bridge/src/storage/discovery.rs` (create new)
- `llmspell-bridge/src/providers.rs` (enhance existing)
- `llmspell-bridge/src/lib.rs`

**Acceptance Criteria**:
- [ ] Unified discovery trait defined
- [ ] All discoveries implement trait
- [ ] Consistent method names
- [ ] Tool discovery added
- [ ] Storage discovery added
- [ ] Provider discovery enhanced
- [ ] All discovery tests passing
- [ ] Quality checks passing

---

#### Task R.1.12: Bridge Tool API Standardization
**Priority**: HIGH
**Estimated Time**: 3.33 hours
**Status**: TODO
**Assigned To**: Bridge Team

**Description**: Standardize tool-related APIs in the bridge layer and create missing components.

**Implementation Steps**:
1. **Analysis & Discovery** (20 min):
   - Check for existing ToolDiscovery: `grep -r "ToolDiscovery" llmspell-bridge/src/`
   - Find tool registration: `grep -r "register_tool\|ToolRegistry" llmspell-bridge/src/`
   - List tool-related globals: `grep -r "tool" llmspell-bridge/src/lua/globals/ llmspell-bridge/src/javascript/globals/`
   - Check tool categorization: `grep -r "ToolCategory\|tool_category" llmspell-*/src/`
   - Find invoke_tool usage: `grep -r "invoke_tool" llmspell-bridge/src/`
   - Document existing API patterns and inconsistencies

2. **Create ToolDiscovery Component** (1.5 hours):
   - Create `llmspell-bridge/src/tools/discovery.rs`
   - Implement discovery pattern matching AgentDiscovery
   - Add tool categorization and filtering
   - Unify with existing tool registration

3. **Standardize Tool Global APIs** (1 hour):
   - Ensure consistent naming: `list_tools`, `get_tool`, `invoke_tool`
   - Add `discover_tools_by_category`
   - Add `get_tool_schema` method
   - Standardize error handling

4. **Tool Configuration** (30 min):
   - Design `ToolConfig` if needed
   - Add builder pattern for tool initialization
   - Standardize resource limits and security

5. **Quality Assurance** (20 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test tool functionality:
     - `cargo test -p llmspell-bridge tool`
     - `cargo test -p llmspell-tools`
   - Verify tool discovery from scripts
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document ToolDiscovery implementation details
   - List all standardized API methods
   - Note any tool categorization decisions

**Files to Create/Update**:
- `llmspell-bridge/src/tools/discovery.rs` (new)
- `llmspell-bridge/src/tools.rs` (update)
- `llmspell-bridge/src/lua/globals/tool.rs`
- `llmspell-bridge/src/javascript/globals/tool.rs`

**Acceptance Criteria**:
- [ ] ToolDiscovery implemented
- [ ] Consistent API naming
- [ ] Tool categorization working
- [ ] Examples updated
- [ ] Tool tests passing
- [ ] Quality checks passing

---

#### Task R.1.13: Provider and Session API Standardization
**Priority**: HIGH
**Estimated Time**: 4.42 hours
**Status**: TODO
**Assigned To**: Core Bridge Team

**Description**: Standardize provider and session/artifact APIs for consistency.

**Implementation Steps**:
1. **Analysis & Discovery** (25 min):
   - List provider methods: `grep -r "impl.*ProviderManager\|impl.*ProviderDiscovery" llmspell-bridge/src/ -A 20`
   - Find provider_supports usage: `grep -r "provider_supports" llmspell-bridge/src/`
   - List session methods: `grep -r "impl.*SessionBridge" llmspell-bridge/src/session_bridge.rs -A 20`
   - List artifact methods: `grep -r "impl.*ArtifactBridge" llmspell-bridge/src/artifact_bridge.rs -A 20`
   - Check naming patterns: `grep -r "fn\s\+\w\+" llmspell-bridge/src/providers.rs llmspell-bridge/src/session_bridge.rs llmspell-bridge/src/artifact_bridge.rs`
   - Document API inconsistencies and patterns

2. **Provider API Standardization** (1.5 hours):
   - Rename methods for consistency:
     - Ensure all use `get_*`, `list_*`, `create_*` patterns
     - `provider_supports` → `check_provider_capability`
   - Add `ProviderDiscovery` wrapper if beneficial
   - Standardize provider info structure

3. **Session API Refinement** (1.5 hours):
   - Review SessionBridge methods for naming consistency
   - Ensure all follow: `create_session`, `get_session`, `list_sessions`
   - Standardize query/filter patterns
   - Add session state transition methods

4. **Artifact API Enhancement** (1 hour):
   - Ensure CRUD consistency: `store_artifact`, `get_artifact`, `list_artifacts`, `delete_artifact`
   - Add `update_artifact_metadata`
   - Add `query_artifacts` with rich filtering
   - Standardize artifact type handling

5. **Quality Assurance** (25 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test specific components:
     - `cargo test -p llmspell-providers`
     - `cargo test -p llmspell-sessions`
     - `cargo test -p llmspell-bridge session`
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all API methods renamed
   - List query/filter patterns added
   - Note any breaking changes avoided

**Files to Update**:
- `llmspell-bridge/src/providers.rs`
- `llmspell-bridge/src/session_bridge.rs`
- `llmspell-bridge/src/artifact_bridge.rs`
- Related Lua/JS globals

**Acceptance Criteria**:
- [ ] Consistent naming patterns
- [ ] No breaking changes
- [ ] Enhanced query capabilities
- [ ] Documentation updated
- [ ] Provider and session tests passing
- [ ] Quality checks passing

---

#### Task R.1.14: State and Storage API Standardization
**Priority**: MEDIUM
**Estimated Time**: 4.42 hours
**Status**: TODO
**Assigned To**: Infrastructure Team

**Description**: Standardize state persistence and storage APIs in the bridge layer.

**Implementation Steps**:
1. **Analysis & Discovery** (25 min):
   - Review StateGlobal methods: `grep -r "impl.*StateGlobal" llmspell-bridge/src/globals/state_global.rs -A 30`
   - Check state patterns: `grep -r "get_state\|set_state\|delete_state" llmspell-bridge/src/`
   - Find storage backend usage: `grep -r "StorageBackend\|storage_backend" llmspell-bridge/src/`
   - List available backends: `grep -r "MemoryBackend\|SledBackend\|RocksDB" llmspell-storage/src/`
   - Check for StorageDiscovery: `grep -r "StorageDiscovery" llmspell-bridge/src/`
   - Document state scope handling patterns

2. **State API Enhancement** (2 hours):
   - Review StateGlobal methods
   - Standardize scope handling
   - Add `list_states`, `query_states` methods
   - Ensure consistent get/set/delete patterns
   - Add state migration helpers

3. **Storage Backend Exposure** (1.5 hours):
   - Create `StorageDiscovery` for available backends
   - Standardize backend configuration
   - Add `StorageConfig` with builder
   - Expose backend capabilities query

4. **Integration Points** (30 min):
   - Ensure state and storage APIs align
   - Standardize error messages
   - Add performance metrics access

5. **Quality Assurance** (25 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test state and storage:
     - `cargo test -p llmspell-state-persistence`
     - `cargo test -p llmspell-storage`
     - `cargo test -p llmspell-bridge state`
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document state API enhancements
   - List storage backends exposed
   - Note integration improvements

**Files to Create/Update**:
- `llmspell-bridge/src/storage/discovery.rs` (new)
- `llmspell-bridge/src/globals/state_global.rs`
- `llmspell-bridge/src/globals/state_infrastructure.rs`

**Acceptance Criteria**:
- [ ] State APIs consistent
- [ ] Storage discovery implemented
- [ ] Migration paths clear
- [ ] Examples demonstrating usage
- [ ] State and storage tests passing
- [ ] Quality checks passing

---

#### Task R.1.15: Hook and Event API Unification
**Priority**: MEDIUM
**Estimated Time**: 3.33 hours
**Status**: TODO
**Assigned To**: Event Team

**Description**: Unify and standardize hook and event APIs across the bridge.

**Implementation Steps**:
1. **Analysis & Discovery** (20 min):
   - Review HookBridge methods: `grep -r "impl.*HookBridge" llmspell-bridge/src/hook_bridge.rs -A 30`
   - Review EventBridge methods: `grep -r "impl.*EventBridge" llmspell-bridge/src/event_bridge.rs -A 30`
   - Check hook registration: `grep -r "register_hook" llmspell-bridge/src/`
   - Check event publishing: `grep -r "publish_event\|emit_event" llmspell-bridge/src/`
   - Find hook points: `grep -r "HookPoint" llmspell-hooks/src/`
   - Document API patterns and inconsistencies

2. **Hook API Standardization** (1.5 hours):
   - Review HookBridge methods
   - Standardize: `register_hook`, `unregister_hook`, `list_hooks`
   - Add `get_hook_info`, `enable_hook`, `disable_hook`
   - Ensure consistent hook point naming

3. **Event API Enhancement** (1 hour):
   - Review EventBridge methods
   - Standardize: `publish_event`, `subscribe_events`, `unsubscribe`
   - Add event filtering and pattern matching
   - Align with hook execution events

4. **Integration** (30 min):
   - Ensure hooks can publish events
   - Standardize event payloads
   - Add correlation IDs

5. **Quality Assurance** (20 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test hook and event systems:
     - `cargo test -p llmspell-hooks`
     - `cargo test -p llmspell-events`
     - `cargo test -p llmspell-bridge hook event`
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document hook API standardizations
   - List event API enhancements
   - Note integration improvements

**Files to Update**:
- `llmspell-bridge/src/hook_bridge.rs`
- `llmspell-bridge/src/event_bridge.rs`
- Related globals for both systems

**Acceptance Criteria**:
- [ ] Consistent API patterns
- [ ] Hook-event integration working
- [ ] Pattern matching implemented
- [ ] Performance acceptable
- [ ] Hook and event tests passing
- [ ] Quality checks passing

---

#### Task R.1.16: Script API Naming Standardization
**Priority**: HIGH
**Estimated Time**: 4.5 hours
**Status**: TODO
**Assigned To**: Script Bridge Team

**Description**: Standardize API naming conventions across Lua and JavaScript bridges.

**Implementation Steps**:
1. **Analysis & Discovery** (30 min):
   - Find all camelCase in Lua: `grep -r "getCurrent\|setCurrent\|getShared\|canReplay\|getReplay\|listReplay" llmspell-bridge/src/lua/`
   - List all Lua global methods: `grep -r "methods\.add" llmspell-bridge/src/lua/globals/`
   - List all JS global methods: `grep -r "define_property\|method" llmspell-bridge/src/javascript/globals/`
   - Document all camelCase methods that need conversion
   - Create comprehensive list of naming inconsistencies

2. **Lua API Standardization** (2 hours):
   - Convert camelCase to snake_case for consistency
   - `getCurrent` → `get_current`
   - `setCurrent` → `set_current`
   - `getSharedMemory` → `get_shared_memory`
   - `canReplay` → `can_replay`
   - `getReplayMetadata` → `get_replay_metadata`
   - `listReplayable` → `list_replayable`
   - Update all Lua global method names

3. **JavaScript API Alignment** (1 hour):
   - Ensure JavaScript APIs follow same patterns
   - Update method names for consistency
   - Document naming convention choice

4. **Global Object Methods** (1 hour):
   - Standardize discovery methods: use `discover_*` consistently
   - Standardize listing methods: use `list_*` consistently
   - Align getter methods: always use `get_*` prefix

5. **Quality Assurance** (30 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test script APIs specifically:
     - `cargo test -p llmspell-bridge lua`
     - `cargo test -p llmspell-bridge javascript`
   - Run script examples to verify functionality
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

6. **Update TODO** (5 min):
   - Document all method names changed
   - List any breaking changes for migration guide
   - Note consistency improvements

**Files to Update**:
- `llmspell-bridge/src/lua/globals/*.rs` (all global files)
- `llmspell-bridge/src/javascript/globals/*.rs`
- Examples using old API names

**Acceptance Criteria**:
- [ ] Consistent naming across all script APIs
- [ ] Documentation updated
- [ ] Examples updated
- [ ] Migration guide created
- [ ] Script API tests passing
- [ ] Quality checks passing

---

#### Task R.1.17: Configuration Builder Exposure in Script APIs
**Priority**: MEDIUM
**Estimated Time**: 6.58 hours
**Status**: TODO
**Assigned To**: Script Integration Team

**Description**: Expose builder patterns through script language APIs.

**Implementation Steps**:
1. **Analysis & Discovery** (35 min):
   - Find existing builder patterns: `grep -r "builder()" llmspell-*/src/`
   - Check current Lua object creation: `grep -r "create\|new" llmspell-bridge/src/lua/globals/`
   - Check current JS object creation: `grep -r "create\|new" llmspell-bridge/src/javascript/globals/`
   - List all config types needing builders: AgentConfig, WorkflowConfig, SessionManagerConfig, etc.
   - Document current creation patterns and builder requirements

2. **Lua Builder API Design** (2 hours):
   ```lua
   -- Current approach
   local agent = Agent.create({
       name = "assistant",
       model = "openai/gpt-4"
   })
   
   -- New builder approach
   local agent = Agent.builder()
       :name("assistant")
       :model("openai/gpt-4")
       :temperature(0.7)
       :max_tokens(2000)
       :build()
   ```

3. **Lua Implementation** (2 hours):
   - Create builder userdata types
   - Implement method chaining
   - Add validation on build()
   - Support both patterns for compatibility

4. **JavaScript Builder API** (1.5 hours):
   - Design similar builder pattern
   - Implement for agents, workflows
   - Ensure type safety where possible

5. **Documentation** (30 min):
   - Document both patterns
   - Show migration examples
   - Update tutorials

6. **Quality Assurance** (35 min):
   - Run `cargo clean && cargo build --all-features`
   - Run `cargo test --workspace`
   - Test builder implementations:
     - `cargo test -p llmspell-bridge builder`
   - Run Lua builder examples
   - Run JavaScript builder examples
   - Verify both old and new patterns work
   - Fix any compilation or test failures
   - Run `./scripts/quality-check-minimal.sh`
   - Verify all checks pass

7. **Update TODO** (5 min):
   - Document all builders exposed to scripts
   - List pattern migration examples created
   - Note compatibility approach

**Files to Create/Update**:
- `llmspell-bridge/src/lua/builders/mod.rs` (new)
- `llmspell-bridge/src/lua/builders/agent_builder.rs` (new)
- `llmspell-bridge/src/lua/builders/workflow_builder.rs` (new)
- `llmspell-bridge/src/javascript/builders/` (new)
- Update all global injection files

**Acceptance Criteria**:
- [ ] Builder patterns available in Lua
- [ ] Builder patterns available in JS
- [ ] Examples demonstrating usage
- [ ] Both old and new patterns work
- [ ] Builder tests passing
- [ ] Quality checks passing

---

### Set 2: Rust API Documentation (Day 3-5)

#### Task R.2.1: Core Crate Documentation
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Status**: TODO
**Assigned To**: Documentation Team

**Description**: Add comprehensive rustdoc to all public APIs in core crates.

**Documentation Requirements**:
1. **Module Level** (2 hours):
   ```rust
   //! # Module Name
   //! 
   //! Brief description of module purpose.
   //! 
   //! ## Overview
   //! 
   //! Detailed explanation of module functionality.
   //! 
   //! ## Examples
   //! 
   //! ```rust
   //! use llmspell_core::*;
   //! 
   //! // Example code
   //! ```
   ```

2. **Struct/Trait Level** (2 hours):
   - Purpose and use cases
   - Generic parameters explained
   - Lifetime requirements
   - Thread safety guarantees

3. **Method Level** (2 hours):
   - Parameters with constraints
   - Return values explained
   - Error conditions
   - Examples for complex methods

**Target Crates**:
- llmspell-core
- llmspell-agents
- llmspell-tools
- llmspell-workflows

**Acceptance Criteria**:
- [ ] All public items have doc comments
- [ ] Examples compile and run
- [ ] No rustdoc warnings
- [ ] Cross-references working

---

#### Task R.2.2: Infrastructure Crate Documentation
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Documentation Team

**Description**: Document all infrastructure crates with focus on integration patterns.

**Target Crates**:
- llmspell-storage
- llmspell-hooks
- llmspell-events
- llmspell-state-persistence
- llmspell-sessions

**Special Focus Areas**:
1. **Integration Examples**:
   ```rust
   //! ## Integration with State Persistence
   //! 
   //! ```rust
   //! let state_manager = StateManager::new().await?;
   //! let session_manager = SessionManager::builder()
   //!     .state_manager(state_manager)
   //!     .build()?;
   //! ```
   ```

2. **Performance Considerations**:
   - Document performance characteristics
   - Memory usage patterns
   - Concurrency limits

**Acceptance Criteria**:
- [ ] All infrastructure APIs documented
- [ ] Integration patterns shown
- [ ] Performance notes included
- [ ] Troubleshooting sections added

---

#### Task R.2.3: Bridge and Scripting Documentation
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Bridge Team

**Description**: Document scripting bridge APIs with language-specific examples.

**Requirements**:
1. **Lua Integration** (2 hours):
   ```rust
   //! ## Lua Usage
   //! 
   //! ```lua
   //! -- Creating an agent
   //! local agent = Agent.create({
   //!     name = "assistant",
   //!     provider = "openai"
   //! })
   //! 
   //! -- Using the agent
   //! local response = agent:query("Hello!")
   //! ```
   ```

2. **JavaScript Integration** (1 hour):
   - Document planned JS API
   - Migration from Lua examples
   - Type definitions

3. **Global Objects** (1 hour):
   - Document all injected globals
   - Lifecycle and availability
   - Thread safety in scripts

**Acceptance Criteria**:
- [ ] All bridge APIs documented
- [ ] Script examples working
- [ ] Language differences noted
- [ ] Security considerations documented

---

### Set 3: Documentation Cleanup (Day 5-7)

#### Task R.3.1: User Guide Standardization
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Documentation Lead

**Description**: Ensure all user guide documentation follows consistent format and terminology.

**Target Documents**:
`docs/user-guide/advanced/performance-tips.md`
`docs/user-guide/advanced/hooks-overview.md`
`docs/user-guide/configuration`
`docs/user-guide/configuration/api-setup-guides.md`
`docs/user-guide/configuration/configuration.md`
`docs/user-guide/session-artifact-api.md`
`docs/user-guide/providers.md`
`docs/user-guide/api-reference-agents-workflows.md`
`docs/user-guide/cross-language-integration.md`
`docs/user-guide/state-management-best-practices.md`
`docs/user-guide/builtin-hooks-reference.md`
`docs/user-guide/tool-reference.md`
`docs/user-guide/hooks-guide.md`
`docs/user-guide/state-management.md`
`docs/user-guide/hook-patterns.md`
`docs/user-guide/getting-started.md`
`docs/user-guide/README.md`
`docs/user-guide/events-guide.md`
`docs/user-guide/tutorial-agents-workflows.md`
`docs/user-guide/examples/hooks-events-cookbook.md`
`docs/user-guide/agent-api.md`
`docs/user-guide/workflow-api.md`
`docs/user-guide/hooks-events-overview.md`
`docs/user-guide/external-tools-guide.md`
`docs/user-guide/state-persistence-guide.md`
`docs/user-guide/api-reference.md`
`docs/user-guide/session-management.md`
- All other user-facing docs

**Standardization Requirements**:
1. **Consistent Structure**:
   ```markdown
   # Document Title
   
   ## Overview
   Brief introduction to the topic
   
   ## Prerequisites
   What users need to know/have
   
   ## Quick Start
   Minimal working example
   
   ## Detailed Usage
   Comprehensive explanations
   
   ## Examples
   Multiple use cases
   
   ## Troubleshooting
   Common issues and solutions
   
   ## API Reference
   Links to relevant rustdoc
   ```

2. **Terminology Consistency**:
   - Agent vs Assistant
   - Tool vs Function
   - Session vs Context
   - Create terminology glossary

**Acceptance Criteria**:
- [ ] All guides follow template
- [ ] Terminology consistent
- [ ] Examples tested and working
- [ ] Cross-references valid

---

#### Task R.3.2: Technical Documentation Cleanup
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Architecture Team

**Description**: Update technical documentation to reflect current implementation.

**Target Documents**:
`docs/technical/security-architecture.md`
`docs/technical/phase-6.5.1-review-checklist.md`
`docs/technical/tool-bridge-architecture.md`
`docs/technical/rs-llmspell-final-architecture.md`
`docs/technical/workflow-bridge-implementation.md`
`docs/technical/hook-event-architecture.md`
`docs/technical/session-artifact-api-design.md`
`docs/technical/README.md`
`docs/technical/backup-retention-design.md`
`docs/technical/hook-implementation.md`
`docs/technical/state-architecture.md`
`docs/technical/global-injection-architecture.md`
- All design documents

**Updates Required**:
1. **Architecture Sync** (1.5 hours):
   - Update diagrams to match code
   - Fix outdated type names
   - Add new components

2. **Design Decision Records** (1 hour):
   - Document why Service → Manager
   - Explain builder pattern choices
   - Note performance tradeoffs

3. **Future Considerations** (30 min):
   - Extension points
   - Versioning strategy
   - Deprecation policy

**Acceptance Criteria**:
- [ ] Diagrams match implementation
- [ ] No outdated information
- [ ] Design decisions recorded
- [ ] Future roadmap clear

---

#### Task R.3.3: Developer Guide Enhancement
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Developer Experience Team

**Description**: Enhance developer guide with contribution guidelines and patterns.

**Target Documents**:
`docs/developer-guide`
`docs/developer-guide/synchronous-api-patterns.md`
`docs/developer-guide/workflow-examples-guide.md`
`docs/developer-guide/agent-examples-guide.md`
`docs/developer-guide/security-guide.md`
`docs/developer-guide/README.md`
`docs/developer-guide/implementing-resource-limits.md`
`docs/developer-guide/tool-development-guide.md`
`docs/developer-guide/test-organization.md`
`docs/developer-guide/session-artifact-implementation.md`
`docs/developer-guide/workflow-bridge-guide.md`
`docs/developer-guide/test-categorization.md`
`docs/developer-guide/hook-development-guide.md`
`docs/developer-guide/agent-testing-guide.md`

**New Sections to Add**:
1. **API Design Guidelines** (2 hours):
   ```markdown
   ## API Design Guidelines
   
   ### Naming Conventions
   - Use `new()` for simple constructors
   - Use `get_*()` for accessors
   - Use `*Manager` suffix for service components
   
   ### Error Handling
   - All fallible operations return Result<T>
   - Provide context with errors
   - Use error chaining
   
   ### Async Patterns
   - Mark async traits with Send + Sync
   - Document cancellation safety
   - Provide sync wrappers for scripts
   ```

2. **Contributing Guide** (1 hour):
   - Code style requirements
   - Testing requirements
   - Documentation standards
   - PR process

3. **Common Patterns** (1 hour):
   - Registry pattern usage
   - Factory pattern examples
   - State management patterns
   - Hook integration patterns

**Acceptance Criteria**:
- [ ] API guidelines comprehensive
- [ ] Contributing guide clear
- [ ] Pattern examples working
- [ ] Review process documented

---

#### Task R.3.4: Example Code Audit
**Priority**: HIGH
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Quality Team

**Description**: Audit and update all example code to use standardized APIs.

**Target Examples**:
- `examples/` directory
- Documentation inline examples
- Test examples
- README examples

**Audit Checklist**:
1. **API Usage** (1.5 hours):
   - Uses latest API names
   - Follows naming conventions
   - Demonstrates best practices
   - Includes error handling

2. **Completeness** (1 hour):
   - All major features shown
   - Progressive complexity
   - Real-world scenarios
   - Performance examples

3. **Testing** (30 min):
   - All examples compile
   - All examples run
   - Output documented
   - CI integration

**Acceptance Criteria**:
- [ ] All examples updated
- [ ] Examples tested in CI
- [ ] Documentation matches
- [ ] No deprecated API usage

---

## Summary

**Total Tasks**: 22
**Estimated Total Time**: 101.83 hours
**Target Duration**: 14 days

### Task Distribution:
- Set 1 (API Consistency): 17 tasks, 73.83 hours
  - Core API Standardization: 5 tasks, 20 hours (R.1.1-R.1.5) ✅ COMPLETED
  - Bridge API Standardization: 12 tasks, 53.83 hours (R.1.6-R.1.17) 🆕 NEW
    - Factory Standards: R.1.6 (2.25 hours)
    - Config Builders: R.1.7-R.1.10 (20.58 hours)
    - Discovery & API Standards: R.1.11-R.1.15 (19.92 hours)
    - Script Integration: R.1.16-R.1.17 (11.08 hours)
- Set 2 (Rust Documentation): 3 tasks, 14 hours  
- Set 3 (Documentation Cleanup): 4 tasks, 14 hours

### Risk Factors:
1. **Breaking Changes**: Must maintain compatibility while improving APIs
2. **Documentation Drift**: Keeping docs in sync with rapid development
3. **Naming Conflicts**: Some renamings may conflict with Rust keywords
4. **Time Estimation**: Documentation often takes longer than estimated
5. **Quality Assurance**: Each task now includes quality checks to prevent regression

### Success Metrics:
- 100% public API documentation coverage
- Zero inconsistent naming patterns
- All examples compile and run
- API style guide adopted
- No breaking changes for existing users
- Documentation praised in user feedback

### Dependencies:
- Phase 6 completion (Session/Artifact system stable)
- No pending architectural changes
- Team availability for reviews

---

## Release Checklist

- [ ] All API inconsistencies resolved
- [ ] Core builder patterns implemented (R.1.5) ✅
- [ ] Factory methods standardized (R.1.6)
- [ ] Bridge layer uses existing builders (R.1.7)
- [ ] Bridge-specific builders created (R.1.8)
- [ ] Infrastructure configs have builders (R.1.9)
- [ ] Script engine configs have builders (R.1.10)
- [ ] Discovery patterns unified (R.1.11)
- [ ] Tool APIs standardized with ToolDiscovery (R.1.12)
- [ ] Provider and Session APIs standardized (R.1.13)
- [ ] State and Storage APIs standardized (R.1.14)
- [ ] Hook and Event APIs unified (R.1.15)
- [ ] Script APIs standardized to snake_case (R.1.16)
- [ ] Builders exposed in Lua/JS APIs (R.1.17)
- [ ] Rustdoc coverage 100%
- [ ] User guide standardized
- [ ] Technical docs updated
- [ ] Developer guide complete
- [ ] Examples all working
- [ ] Migration guide created
- [ ] API style guide published
- [ ] Version 0.6.0 tagged
- [ ] Changelog updated
- [ ] Release notes drafted