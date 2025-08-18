# Phase 7 TODO - API Consistency and Standardization

**Phase**: 7
**Title**: Refactoring for API Consistency and Standardization Across Entire Codebase
**Status**: TODO
**Start Date**: TBD
**Target End Date**: TBD (7 days from start)
**Dependencies**: Phase 6 Release (Session and Artifact Management) ✅
**Priority**: HIGH (Release Critical)
**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-07-design-doc.md
**Testing Guide**: docs/developer-guid/test-development-guide.md
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE07-TODO.md)

---

## Overview

Phase 7 focuses on comprehensive refactoring to achieve API consistency and standardization across the entire codebase. After completing Phase 6 Release, we identified the need for systematic standardization of all APIs, configuration patterns, naming conventions, and architectural patterns. This phase establishes the foundation for a stable 1.0 release by creating unified patterns across all crates, components, and script interfaces. We've already completed 5 core API standardization tasks (1.1-1.5), providing a strong foundation for the remaining work.

### Success Criteria
- [ ] All public APIs follow consistent naming conventions
- [ ] Builder patterns implemented for complex object creation
- [ ] All public functions have comprehensive rustdoc documentation
- [ ] User guide, technical, and developer documentation are consistent
- [ ] API style guide created and enforced
- [ ] Clean API breaks to establish stable patterns (no backward compatibility cruft)
- [ ] Examples provided for all major API patterns

---

## Task List Summary
**for completed tasks see `/TODO-DONE.md`**
### Set 1: API Consistency and Naming Conventions (Day 1-3)
#### Task 7.1.1: API Inventory and Analysis
#### Task 7.1.2: API Standardization Plan
#### Task 7.1.3: Implement Manager/Service Standardization
#### Task 7.1.4: Implement Retrieve/Get Standardization
#### Task 7.1.5: Implement Builder Patterns
#### Task 7.1.6: Comprehensive Test Organization and Categorization Refactoring
#### Task 7.1.7: Workflow-Agent Trait Integration
#### Task 7.1.8: Workflow Factory and Executor Standardization
#### Task 7.1.9: Workflow Config Builder Standardization  
#### Task 7.1.10: Workflow Bridge API Standardization
#### Task 7.1.11: Workflow Script API Naming Standardization
#### Task 7.1.12: Factory Method Standardization
#### Task 7.1.13: Core Bridge Config Builder Usage
#### Task 7.1.14: Bridge-Specific Config Builders
#### Task 7.1.15: Infrastructure Config Builders
#### Task 7.1.16: Script Engine Config Builders
#### Task 7.1.17: Bridge Discovery Pattern Unification
#### Task 7.1.18: Bridge Tool API Standardization
#### Task 7.1.19: Provider and Session API Standardization
#### Task 7.1.20: State and Storage API Standardization ✅
#### Task 7.1.21: Hook and Event API Unification ✅
#### Task 7.1.22: Script API Naming Standardization  
#### Task 7.1.23: Configuration Builder Exposure in Script APIs
#### Task 7.1.24: Hook Execution Standardization
#### Task 7.1.25: Fix Test Infrastructure Failures Across All Crates
#### Task 7.1.26: Fix all fixable clippy errors across all crates
#### Task 7.2.1: Core Crate Documentation
#### Task 7.2.2: Infrastructure Crate Documentation
#### Task 7.2.3: Bridge and Scripting Documentation
#### Task 7.3.1: Example Audit and Categorization
#### Task 7.3.2: Example Directory Structure Creation
#### Task 7.3.3: Core Example Migration
## 🎉 TASK 7.3.2 + 7.3.3 COMPLETED SUCCESSFULLY ✅
#### Task 7.3.4: Getting Started Experience
#### Task 7.3.5: Cookbook and Patterns
#### Task 7.3.6: Real-World Applications
**CRITICAL ISSUES DISCOVERED**: Wait for 7.3.8 to be done (look at `TODO-DONE.md` for details)
- **Workflows return metadata, not content** - `result.data` contains branch info, not generated outputs
- **No actual LLM integration** - Despite API keys set, agents don't call LLMs
- **Only 2/7 files created** - Missing ux-design.json, architecture.json, frontend/backend code, deployment.yaml
- **Executes in 262ms** - Impossibly fast for real LLM generation
- **File writing code added but unused** - Workflow results are empty
#### Task 7.3.7: Configuration Architecture Redesign and Tool Security Enhancement

---

#### Task 7.3.8: State-Based Workflow Output Implementation (Google ADK Pattern)
**Priority**: CRITICAL
**Estimated Time**: 12 hours
**Status**: IN PROGRESS
**Assigned To**: Core Team
**Dependencies**: Task 7.3.7 (llmspell-config)

**Description**: Implement Google ADK-style state-based workflow outputs where workflows automatically write outputs to state instead of returning them directly. This provides memory efficiency, natural persistence, and industry-standard patterns for agent composition.

**Architecture Decision (Option A Selected)**:

After analyzing the codebase, we've chosen to make state a first-class citizen by:
1. Adding a `StateAccess` trait to `llmspell-core` (no direct dependency on persistence)
2. Adding `state: Option<Arc<dyn StateAccess>>` to `ExecutionContext`
3. All components (workflows, agents, tools) access state through context
4. `llmspell-state-persistence` provides the concrete implementation

**Rationale**:
- State is fundamental to component communication (like Google ADK, Temporal, Airflow)
- Available to ALL components via ExecutionContext (already passed to all execute methods)
- Clean architecture: trait in core, implementation in state-persistence
- Clear distinction: `shared_memory` for transient data, `state` for persistent data
- Non-breaking: existing code works with `state: None`
- Avoids circular dependencies while making state universally accessible

**Architecture Goals**:
- State as first-class citizen available to all components
- Workflows write outputs directly to state during execution
- WorkflowResult contains only execution metadata (not data)
- Consistent pattern across all workflow types
- Scripts access outputs via State global
- Memory efficient for large outputs
- Natural workflow composition through shared state

**Implementation Steps**:

1. [x] **Core State Infrastructure** (2 hours): ✅ COMPLETED
   - [x] Create `llmspell-core/src/traits/state.rs` with `StateAccess` trait:
     ```rust
     #[async_trait]
     pub trait StateAccess: Send + Sync {
         async fn read(&self, key: &str) -> Result<Option<Value>>;
         async fn write(&self, key: &str, value: Value) -> Result<()>;
         async fn delete(&self, key: &str) -> Result<bool>;
         async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
     }
     ```
   - [x] Modify `ExecutionContext` to include state:
     ```rust
     pub struct ExecutionContext {
         // ... existing fields
         pub shared_memory: SharedMemory,      // Transient (exists)
         pub state: Option<Arc<dyn StateAccess>>, // Persistent (NEW)
     }
     ```
   - [x] Update `ExecutionContextBuilder` with `.state()` method
   - [x] Export StateAccess trait from core/traits/mod.rs
   - [x] Run `cargo clippy -- -D warnings` after each change
   - [x] Fixed dyn-compatibility issue by removing generic parameter from `write_batch`

2. [x] **Create Unified WorkflowResult** (1 hour): ✅ COMPLETED
   - [x] Create `llmspell-workflows/src/result.rs` with unified result:
     ```rust
     pub struct WorkflowResult {
         pub execution_id: String,
         pub workflow_type: WorkflowType,
         pub workflow_name: String,
         pub success: bool,
         pub summary: String,
         pub state_keys: Vec<String>,  // Keys written to state
         pub steps_executed: usize,
         pub steps_failed: usize,
         pub duration: Duration,
         pub error: Option<WorkflowError>,
     }
     ```
   - [x] Add methods for constructing success/failure results
   - [x] Export from lib.rs
   - [x] Remove old result types after all workflows updated (will be done after workflows are updated)
   - [x] Added WorkflowError enum for workflow-specific errors
   - [x] Added PartiallyCompleted status to WorkflowStatus enum
   - [x] Fixed all pattern matching for new status variant

3. [x] **Sequential Workflow State Integration** (1.5 hours): ✅ COMPLETED
   - [x] Modify `llmspell-workflows/src/sequential.rs`:
     - [x] Added new `execute_with_state()` method that takes ExecutionContext
     - [x] Access state through `context.state` in execution
     - [x] Write step outputs: `context.state.write("workflow:{id}:{step}", output)`
     - [x] Return unified `WorkflowResult` with state keys
     - [x] Handle case when `context.state` is None (fallback behavior)
   - [x] Updated `execute()` method to use state when available
   - [x] Maintained backward compatibility with legacy `execute_workflow()`
   - [x] Ensure ZERO clippy warnings

4. [x] **Parallel Workflow State Integration** (1.5 hours): ✅ COMPLETED
   - [x] Modify `llmspell-workflows/src/parallel.rs`:
     - [x] Added new `execute_with_state()` method for state-based execution
     - [x] Write branch outputs through context.state
     - [x] Keys: `workflow:{id}:branch_{branch_name}:{step_name}`
     - [x] Handle concurrent writes (using Arc<Mutex> for thread-safe collection)
     - [x] Return unified `WorkflowResult`
   - [x] Updated `execute()` method to use state when available
   - [x] Maintained backward compatibility with legacy `execute_workflow()`
   - [x] Ensure ZERO clippy warnings

5. [x] **Conditional & Loop Workflow State Integration** (COMPLETED):
   - [x] Modified `llmspell-workflows/src/conditional.rs`:
     - [x] Added `execute_with_state()` method for state-based execution
     - [x] Write outputs through context.state
     - [x] Keys: `workflow:{id}:branch_{name}:{step}`
     - [x] Updated execute() to use state when available
   - [x] Modified `llmspell-workflows/src/loop.rs`:
     - [x] Added `execute_with_state()` method with iteration tracking
     - [x] Keys: `workflow:{id}:iteration_{n}:{step}`
     - [x] Aggregation results: `workflow:{id}:aggregated`
     - [x] Handles break conditions and aggregation strategies
   - [x] Fixed compilation errors (ComponentId, ConditionEvaluationContext)
   - [x] Maintained backward compatibility with legacy execution
   - [x] Achieved zero clippy warnings

6. [x] **Bridge StateAccess Implementation & Configuration** (2.5 hours) - ✅ COMPLETED:
   - [x] Update `llmspell-config/src/lib.rs` defaults:
     - Changed `CoreStateFlags::enabled` default from `false` to `true`
     - Set default backend to in-memory for immediate usage
   - [x] Create `llmspell-bridge/src/state_adapter.rs`:
     - Implemented `StateManagerAdapter` wrapping `StateManager`
     - Maps `StateManager` operations to `StateAccess` trait
     - Handles scoping (Global, Agent, Workflow, Tool)
   - [x] Update workflow execution to use state:
     - Created `create_execution_context_with_state()` helper
     - All workflow executors use state-enabled contexts
     - Workflows use `BaseAgent` interface with state support
   - [x] Fixed implementation details:
     - JSON deserialization for `AgentInput` using `serde_json::from_value`
     - Error conversion from `anyhow::Error` to `LLMSpellError`
     - Code formatting and clippy compliance

7. [ ] **Bridge Globals Update for State Architecture** (4 hours):
   
   **Rationale**: The script-exposed globals (State, Workflow, Agent, Tool) are currently 
   disconnected from the new state architecture. They use StateManager directly instead of 
   the StateAccess trait, and don't propagate state through ExecutionContext. This step 
   aligns ALL globals with the state-based workflow architecture for consistency.
   
   **A. Update StateGlobal to use StateAccess trait**:
   - [ ] Modify `llmspell-bridge/src/globals/state_global.rs`:
     - [ ] Replace direct StateManager usage with StateAccess trait
     - [ ] Use StateManagerAdapter for state operations
     - [ ] Maintain backward compatibility with fallback_state
   - [ ] Update `llmspell-bridge/src/lua/globals/state.rs`:
     - [ ] Use StateAccess methods instead of StateManager directly
     - [ ] Simplify scope handling (adapter handles it)
   
   **B. Update GlobalContext for state propagation**:
   - [ ] Modify `llmspell-bridge/src/globals/types.rs`:
     ```rust
     pub struct GlobalContext {
         pub registry: Arc<ComponentRegistry>,
         pub providers: Arc<ProviderManager>,
         pub state_access: Option<Arc<dyn StateAccess>>, // NEW
         pub bridge_refs: Arc<RwLock<HashMap<String, Arc<dyn Any>>>>,
     }
     ```
   - [ ] Update GlobalContext construction to include state from config
   
   **C. Update Workflow Global for state-based outputs**:
   - [ ] Modify `llmspell-bridge/src/lua/globals/workflow.rs`:
     ```lua
     -- Add helper methods to workflow instances
     workflow:get_output(step_name)      -- Gets from state
     workflow:get_all_outputs()          -- Gets all workflow outputs  
     workflow:list_outputs()              -- Lists available output keys
     workflow:clear_outputs()             -- Cleans up state
     workflow:get_execution_id()         -- Gets workflow execution ID
     ```
   - [ ] Implement state-aware workflow execution:
     - [ ] Pass ExecutionContext with state to workflows
     - [ ] Handle WorkflowResult with state outputs
     - [ ] Provide access to outputs via state keys
   
   **D. Update Agent Global for state context**:
   - [ ] Modify `llmspell-bridge/src/lua/globals/agent.rs`:
     - [ ] Update `AgentBridge::execute_agent()` calls to pass ExecutionContext with state
       (Currently passing `None` at lines 34, 50, 70)
     - [ ] Create state-enabled ExecutionContext using StateManagerAdapter
     - [ ] Allow agents to access workflow outputs from state
     - [ ] Enable agents to write results to state
   
   **E. Update Tool Global for state access**:
   - [ ] Modify `llmspell-bridge/src/lua/globals/tool.rs`:
     - [ ] Pass state through tool execution context
     - [ ] Allow tools to read/write state for data sharing
   
   **F. Create Lua/JavaScript helpers**:
   - [ ] Add State helper methods:
     ```lua
     State.workflow_get(workflow_id, step_name)  -- Shortcut for workflow outputs
     State.workflow_list(workflow_id)            -- List workflow output keys
     State.agent_get(agent_id, key)              -- Agent-scoped state
     State.tool_get(tool_id, key)                -- Tool-scoped state
     ```
   - [ ] Add JavaScript equivalents for all Lua helpers
   - [ ] Document all helper functions with examples
   
   **G. Add Environment Variable Support**:
   - [ ] Extend `llmspell-config/src/lib.rs` `apply_env_overrides()` method:
     ```rust
     // State persistence overrides
     if let Ok(enabled) = env::var("LLMSPELL_STATE_PERSISTENCE_ENABLED") {
         self.runtime.state_persistence.flags.core.enabled = enabled.parse()?;
     }
     if let Ok(backend) = env::var("LLMSPELL_STATE_PERSISTENCE_BACKEND") {
         self.runtime.state_persistence.backend_type = backend;
     }
     if let Ok(path) = env::var("LLMSPELL_STATE_PERSISTENCE_PATH") {
         // Apply to sled/rocksdb configs if applicable
     }
     ```
   - [ ] Handle backend-specific configuration from env vars
   - [ ] Add validation for env var values
   - [ ] Document environment variables in configuration guide

8. [ ] **Testing Suite** (1.5 hours):
   - [ ] Create mock StateAccess for testing in `llmspell-workflows/src/test_utils.rs`
   - [ ] Create integration test `llmspell-bridge/tests/state_workflow_integration.rs`:
     - [ ] Test sequential workflow with state outputs
     - [ ] Test parallel workflow with concurrent state writes
     - [ ] Test workflow without state (None case)
     - [ ] Test large output handling (>10MB)
   - [ ] Performance benchmarks for state-based vs direct returns
   - [ ] Memory usage comparison tests
   - [ ] Run with `SKIP_SLOW_TESTS=false cargo test`

9. [ ] **Update Example Applications** (2 hours):
   - [ ] Update `webapp-creator/main.lua` to use state-based outputs:
     ```lua
     local result = main_workflow:execute({})
     if result.success then
         -- Access outputs from state using helper
         local ux_design = main_workflow:get_output("ux_design_phase")
         -- Or directly via State global
         local frontend = State.get("workflow:" .. result.execution_id .. ":frontend_phase")
         -- Write to files...
     end
     ```
   - [ ] Update other applications similarly:
     - [ ] `content-generation-platform/main.lua`
     - [ ] `code-review-assistant/main.lua`
     - [ ] `data-pipeline/main.lua`
     - [ ] `document-intelligence/main.lua`
     - [ ] `research-assistant/main.lua`
     - [ ] `customer-support-bot/main.lua`
     - [ ] `workflow-hub/main.lua`
   - [ ] Update cookbook example `multi-agent-coordination.lua`
   - [ ] Test each application: `./examples/run-all-applications.sh`

10. [ ] **Documentation & Migration Guide** (1 hour):
    - [ ] Create `/docs/technical/state-based-workflows.md`:
      - [ ] Architecture decision and rationale
      - [ ] StateAccess trait design
      - [ ] State key naming conventions
      - [ ] Migration guide from direct returns
      - [ ] Performance considerations
      - [ ] Best practices for large outputs
    - [ ] Update `/docs/user-guide/workflows.md` with state-based examples
    - [ ] Add migration notes to CHANGELOG.md
    - [ ] Document backward compatibility (state: None case)

**Implementation Order**:
1. Core State Infrastructure (Step 1) - Add StateAccess trait and update ExecutionContext
2. Create Unified WorkflowResult (Step 2) - Single result type for all workflows
3. Sequential Workflow (Step 3) - Simplest case first
4. Parallel/Conditional/Loop (Steps 4-5) - More complex patterns
5. Bridge Implementation (Step 6) - Connect state-persistence to workflows
6. **Bridge Globals Update (Step 7)** - Critical architectural alignment:
   - All script-exposed globals must use the new state architecture
   - StateGlobal must use StateAccess trait, not StateManager directly
   - GlobalContext needs state propagation for all globals
   - Workflow/Agent/Tool globals need state-aware execution
7. Testing & Validation (Step 8) - Ensure correctness
8. Update Applications (Step 9) - Real-world validation
9. Documentation (Step 10) - Complete the work

**Quality Requirements**:
- [ ] ZERO clippy warnings: `cargo clippy -- -D warnings`
- [ ] All tests passing: `cargo test --workspace --all-features`
- [ ] No performance regression: Benchmark before/after
- [ ] Memory usage improved for large outputs
- [ ] Documentation complete with examples
- [ ] All 9 example applications working with new pattern

**Acceptance Criteria**:
- [ ] StateAccess trait added to core with clean abstraction
- [ ] ExecutionContext includes optional state access
- [ ] Workflows write outputs to state when available
- [ ] WorkflowResult contains only metadata, not actual data
- [ ] State keys follow consistent naming pattern
- [ ] Scripts can access outputs via State global or helpers
- [ ] Backward compatible: works without state (None case)
- [ ] All example applications updated and functional
- [ ] Performance benchmarks show improvement for large data
- [ ] Migration guide helps users update existing code

**Success Metrics**:
- Memory usage reduced by >50% for large workflow outputs
- No performance degradation for small outputs
- Zero breaking changes for workflows without state
- All example applications pass integration tests
- Clean architecture with no circular dependencies

---

#### Task 7.3.9: Example Testing Framework
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Test Team
**Dependencies**: Task 7.3.2

**Description**: Create automated testing for all examples to ensure they remain functional.

**Implementation Steps**:
1. [ ] **Test Infrastructure** (1.5 hours):
   - [ ] Create example test runner
   - [ ] Add example validation
   - [ ] Create test categories

2. [ ] **Test Implementation** (1.5 hours):
   - [ ] Add tests for script examples
   - [ ] Add tests for Rust examples
   - [ ] Test example outputs
   - [ ] Validate metadata

3. [ ] **Automation** (1 hour):
   - [ ] Nightly example testing
   - [ ] PR validation for examples
   - [ ] Performance regression tests
   - [ ] Breaking change detection

**Test Categories**:
- [ ] Compilation/syntax tests
- [ ] Execution tests
- [ ] Output validation
- [ ] Performance tests
- [ ] Integration tests

**Acceptance Criteria**:
- [ ] All examples have tests
- [ ] Nightly runs configured
- [ ] Test reports generated
- [ ] Breaking changes detected

---

#### Task 7.3.10: Example Documentation Integration
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Documentation Team
**Dependencies**: Task 7.3.6

**Description**: Integrate examples into main documentation with proper cross-references.

**Implementation Steps**:
1. [ ] **Documentation Updates** (1.5 hours):
   - [ ] Update user guide with example links
   - [ ] Add examples to API documentation
   - [ ] Create example index
   - [ ] Update getting started guide

2. [ ] **Cross-Reference System** (1 hour):
   - [ ] Link examples from feature docs
   - [ ] Create example search system
   - [ ] Add "See Also" sections
   - [ ] Build example graph

3. [ ] **Discovery Enhancement** (30 min):
   - [ ] Add example finder tool
   - [ ] Create tag-based search
   - [ ] Implement full-text search
   - [ ] Add recommendation system

**Integration Points**:
- [ ] User guide references
- [ ] API documentation
- [ ] Developer guide
- [ ] README files
- [ ] Website/docs site

**Acceptance Criteria**:
- [ ] All docs reference relevant examples
- [ ] Example index created
- [ ] Search system implemented
- [ ] Cross-references complete
- [ ] Discovery tools working

---

### Set 4: Documentation Cleanup (Day 7-9)

#### Task 4.1: rs-llmspell browseable api documentation 
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Documentation Lead

**Description**: Ensure a complete set of coherent apis documentation are created for rust and lua. they should be under `docs/user-guide/api/rust/` and `docs/user-guide/api/lua`. 


#### Task 4.2: User Guide Standardization
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Documentation Lead

**Description**: Ensure all user guide documentation follows consistent format and terminology. Requires Megathink to analyze what we have now vs what we actually need for a very user-friendly user-guide.


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
- [ ] All other user-facing docs



**Standardization Requirements**:
1. [ ] **Consistent Structure**:
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

2. [ ] **Terminology Consistency**:
   - [ ] Agent vs Assistant
   - [ ] Tool vs Function
   - [ ] Session vs Context
   - [ ] Create terminology glossary

**Acceptance Criteria**:
- [ ] All guides follow template
- [ ] Terminology consistent
- [ ] Examples tested and working
- [ ] Cross-references valid

---

#### Task 4.3: Technical Documentation Cleanup
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Architecture Team

**Description**: Update technical documentation to reflect current implementation.  Requires Megathink to analyze what we have now vs what we actually need for a very user-friendly technical-guide which is different from the developer-guide in 4.4 below. Do not modify `docs/technical/master-architecture-vision.md`.

**Target Documents**:
`docs/technical/security-architecture.md`
`docs/technical/phase-6.5.1-review-checklist.md`
`docs/technical/tool-bridge-architecture.md`
`docs/technical/master-architecture-vision.md`
`docs/technical/workflow-bridge-implementation.md`
`docs/technical/hook-event-architecture.md`
`docs/technical/session-artifact-api-design.md`
`docs/technical/README.md`
`docs/technical/backup-retention-design.md`
`docs/technical/hook-implementation.md`
`docs/technical/state-architecture.md`
`docs/technical/global-injection-architecture.md`
- [ ] All design documents

**Updates Required**:
1. [ ] **Architecture Sync** (1.5 hours):
   - [ ] Update diagrams to match code
   - [ ] Fix outdated type names
   - [ ] Add new components

2. [ ] **Design Decision Records** (1 hour):
   - [ ] Document why Service → Manager
   - [ ] Explain builder pattern choices
   - [ ] Note performance tradeoffs

3. [ ] **Future Considerations** (30 min):
   - [ ] Extension points
   - [ ] Versioning strategy
   - [ ] Post-1.0 stability commitments

**Acceptance Criteria**:
- [ ] Diagrams match implementation
- [ ] No outdated information
- [ ] Design decisions recorded
- [ ] Future roadmap clear

---

#### Task 4.4: Developer Guide Enhancement
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Developer Experience Team

**Description**: Enhance developer guide with contribution guidelines and patterns. Requires Megathink to analyze what we have now vs what we actually need for a very user-friendly developer-guide which is different from the technical-guide in 4.3 above.

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
1. [ ] **API Design Guidelines** (2 hours):
   ```markdown
   ## API Design Guidelines
   
   ### Naming Conventions
   - [ ] Use `new()` for simple constructors
   - [ ] Use `get_*()` for accessors
   - [ ] Use `*Manager` suffix for service components
   
   ### Error Handling
   - [ ] All fallible operations return Result<T>
   - [ ] Provide context with errors
   - [ ] Use error chaining
   
   ### Async Patterns
   - [ ] Mark async traits with Send + Sync
   - [ ] Document cancellation safety
   - [ ] Provide sync wrappers for scripts
   ```

2. [ ] **Contributing Guide** (1 hour):
   - [ ] Code style requirements
   - [ ] Testing requirements
   - [ ] Documentation standards
   - [ ] PR process

3. [ ] **Common Patterns** (1 hour):
   - [ ] Registry pattern usage
   - [ ] Factory pattern examples
   - [ ] State management patterns
   - [ ] Hook integration patterns

**Acceptance Criteria**:
- [ ] API guidelines comprehensive
- [ ] Contributing guide clear
- [ ] Pattern examples working
- [ ] Review process documented

---

#### Task 4.4: Example Code Audit
**Priority**: HIGH
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Quality Team

**Description**: Audit and update all example code to use standardized APIs.

**Target Examples**:
- `examples/` directory
- [ ] Documentation inline examples
- [ ] Test examples
- [ ] README examples

**Audit Checklist**:
1. [ ] **API Usage** (1.5 hours):
   - [ ] Uses latest API names
   - [ ] Follows naming conventions
   - [ ] Demonstrates best practices
   - [ ] Includes error handling

2. [ ] **Completeness** (1 hour):
   - [ ] All major features shown
   - [ ] Progressive complexity
   - [ ] Real-world scenarios
   - [ ] Performance examples

3. [ ] **Testing** (30 min):
   - [ ] All examples compile
   - [ ] All examples run
   - [ ] Output documented
   - [ ] CI integration

**Acceptance Criteria**:
- [ ] All examples updated
- [ ] Examples tested in CI
- [ ] Documentation matches
- [ ] All APIs use latest patterns

---

## Summary

**Total Tasks**: 40
**Estimated Total Time**: 174.41 hours  
**Target Duration**: 25 days

### Task Distribution:
- **Completed**: 5 tasks (12.5% complete)
- **TODO**: 35 tasks (87.5% remaining)

- [ ] Set 1 (API Consistency): 24 tasks, 104.41 hours
  - [ ] Core API Standardization: 5 tasks, 20 hours (1.1-1.5) ✅ COMPLETED
  - [ ] Test Organization Foundation: 1 task, 8 hours (1.6) 🆕 CRITICAL FOUNDATION
  - [ ] Workflow Standardization: 5 tasks, 23 hours (1.7-1.11) 🆕 NEW
    - Workflow-Agent Integration: 1.7 (8 hours)
    - Factory and Executor Standards: 1.8 (4.5 hours)
    - Config Builder Standards: 1.9 (3.5 hours)
    - Bridge API Standards: 1.10 (4 hours)
    - Script API Standards: 1.11 (3 hours)
  - [ ] Bridge API Standardization: 13 tasks, 53.41 hours (1.12-1.24) 🔄 RENUMBERED & COORDINATED
    - Factory Standards: 1.12 (2.58 hours, excludes workflows)
    - Config Builder Usage: 1.13-1.16 (19.33 hours, excludes workflows)  
    - Discovery & API Standards: 1.17-1.21 (18.42 hours, coordinates with workflows)
    - Script Integration: 1.22-1.23 (10.33 hours, coordinates with 1.11)
    - Hook Architecture Fix: 1.24 (5.5 hours, critical infrastructure fix)
- [ ] Set 2 (Rust Documentation): 3 tasks, 14 hours  
- [ ] Set 3 (Example Reorganization): 8 tasks, 40 hours 🆕 NEW
- [ ] Set 4 (Documentation Cleanup): 4 tasks, 14 hours
- [ ] Set 5 (Test Architecture Verification): 1 task, 2 hours (5.1) 🆕 FINAL CHECK

### Risk Factors:
1. [ ] **Breaking Changes**: Clean break approach requires updating all calling code
2. [ ] **Documentation Drift**: Keeping docs in sync with rapid development
3. [ ] **Naming Conflicts**: Some renamings may conflict with Rust keywords
4. [ ] **Time Estimation**: Documentation often takes longer than estimated
5. [ ] **Quality Assurance**: Each task now includes quality checks to prevent regression
6. [ ] **No Compatibility Layers**: Must ensure all old patterns are completely removed

### Success Metrics:
- 100% public API documentation coverage
- [ ] Zero inconsistent naming patterns
- [ ] All examples compile and run
- [ ] API style guide adopted
- [ ] Clean, stable API established for 1.0 release
- [ ] Documentation praised in user feedback
- [ ] No compatibility cruft in codebase

### Dependencies:
- [ ] Phase 6 completion (Session/Artifact system stable)
- [ ] No pending architectural changes
- [ ] Team availability for reviews

---

## Release Checklist

- [ ] All API inconsistencies resolved
- [ ] Core builder patterns implemented (1.5) ✅
- [ ] Test organization foundation (1.6)
- [ ] Workflow-Agent trait integration (1.7)
- [ ] Workflow factory and executor standardization (1.8)
- [ ] Workflow config builder standardization (1.9)
- [ ] Workflow bridge API standardization (1.10)
- [ ] Workflow script API naming standardization (1.11)
- [ ] Factory method naming standardized (1.12, excludes workflows)
- [ ] Bridge layer uses existing builders (1.13, excludes workflows)
- [ ] Bridge-specific builders created (1.14)
- [ ] Infrastructure configs have builders (1.15)
- [ ] Script engine configs have builders (1.16)
- [ ] Discovery patterns unified (1.17, coordinates with 1.10)
- [ ] Tool APIs standardized with ToolDiscovery (1.18)
- [ ] Provider APIs standardized (1.19)
- [ ] State and Storage APIs standardized (1.20)
- [ ] Hook and Event APIs unified (1.21)
- [ ] Script APIs standardized to snake_case (1.22, excludes workflows)
- [ ] Builders exposed in Lua/JS APIs (1.23, includes 1.9 workflow builders)
- [ ] Hook execution standardized across all crates (1.24, fixes tools/workflows)
- [ ] Test organization foundation established (1.6, categorize 175+ tests)
- [ ] Examples reorganized and categorized (3.1-3.8)
  - [ ] Example audit completed (3.1)
  - [ ] New directory structure created (3.2)
  - [ ] Examples migrated to new structure (3.3)
  - [ ] Getting started paths created (3.4)
  - [ ] Cookbook patterns documented (3.5)
  - [ ] Real-world applications enhanced (3.6)
  - [ ] Example testing framework created (3.7)
  - [ ] Documentation integration complete (3.8)
- [ ] Test categorization verification completed (5.1, verify all tests categorized)
- [ ] Rustdoc coverage 100%
- [ ] User guide standardized
- [ ] Technical docs updated
- [ ] Developer guide complete
- [ ] Examples all working
- [ ] Breaking changes documented
- [ ] API style guide published
- [ ] Version 0.6.0 tagged
- [ ] Changelog updated
- [ ] Release notes drafted

---

### Set 5: Test Architecture Verification (Critical Infrastructure)

#### Task 7.5.1: Test Categorization Verification and Final Cleanup
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Status**: TODO
**Assigned To**: Test Architecture Team
**Dependencies**: Tasks 7.1.6-7.1.24 (All API tasks completed with test categorization)

**Description**: Final verification pass to ensure all tests are properly categorized after Phase 7 API standardization work. This ensures no uncategorized tests were created during the 18 API tasks.

**Implementation Steps**:
1. [ ] **Test Architecture Analysis** (1 hour):
   - [ ] Audit all 175 integration test files: `find . -name "*.rs" -path "*/tests/*" | wc -l`
   - [ ] Find uncategorized tests: `find . -name "*.rs" -path "*/tests/*" -exec grep -L "cfg_attr.*test_category" {} \;`
   - [ ] Find tests with external dependencies: `find . -name "*.rs" -exec grep -l "reqwest\|tokio::net\|std::net\|url::Url\|api_key\|OPENAI\|ANTHROPIC" {} \;`
   - [ ] Identify duplicate test infrastructure across crates
   - [ ] Map current test distribution by crate and type
   - [ ] Document existing llmspell-testing capabilities

2. [ ] **Test Type Classification** (2 hours):
   **Type 1 - Unit Tests (src/ files)**:
   - [ ] Fast, isolated component tests
   - [ ] No external dependencies
   - [ ] Add `#[cfg_attr(test_category = "unit")]`
   - [ ] Should run in <5 seconds total
   
   **Type 2 - Integration Tests (tests/ files)**:
   - [ ] Cross-component, cross-crate tests
   - [ ] No external dependencies (mocked)
   - [ ] Add `#[cfg_attr(test_category = "integration")]`
   - [ ] Should run in <30 seconds total
   
   **Type 3 - External Dependency Tests**:
   - [ ] API calls, network requests, LLM providers
   - [ ] Add `#[cfg_attr(test_category = "external")]`
   - [ ] Can be slow, require credentials
   - [ ] Should be skipped in CI by default

3. [ ] **Systematic Test Categorization** (3 hours):
   - [ ] **Phase 1**: Categorize all unit tests in `src/` files
   - [ ] **Phase 2**: Categorize all integration tests in `tests/` directories
   - [ ] **Phase 3**: Identify and isolate external dependency tests
   - [ ] **Phase 4**: Add component-specific categories (agent, tool, workflow, bridge)
   - [ ] **Phase 5**: Add performance/security categories where appropriate
   - [ ] Remove duplicate test infrastructure, use llmspell-testing utilities

4. [ ] **Test Execution Standardization** (1.5 hours):
   - [ ] Update all crates to use unified test runner approach
   - [ ] Create fast test suite: `cargo test --features unit-tests,integration-tests`
   - [ ] Create comprehensive test suite: `cargo test --features all-tests`
   - [ ] Create external test suite: `cargo test --features external-tests`
   - [ ] Update CI to run only fast tests by default
   - [ ] Document test execution patterns

5. [ ] **Test Infrastructure Consolidation** (30 min):
   - [ ] Move common test utilities to llmspell-testing
   - [ ] Remove duplicate mock/fixture code across crates
   - [ ] Standardize test setup patterns
   - [ ] Create common test data generators
   - [ ] Ensure consistent test isolation

6. [ ] **Quality Assurance** (30 min):
   - [ ] Run fast test suite: `./llmspell-testing/scripts/run-fast-tests.sh`
   - [ ] Run integration test suite: `./llmspell-testing/scripts/run-integration-tests.sh`
   - [ ] Verify external tests are properly isolated
   - [ ] Ensure no tests are accidentally ignored
   - [ ] Verify test categorization works correctly
   - [ ] Run `./scripts/quality-check-minimal.sh`
   - [ ] Verify all checks pass

7. [ ] **Update TODO** (10 min):
   - [ ] Document test categorization completion statistics
   - [ ] List any tests that couldn't be categorized
   - [ ] Update developer documentation with new test patterns

**Root Cause Analysis** ✅ **COMPLETED**:
- [x] **175 test files exist** but only ~5% use the categorization system → **536+ files now categorized**
- [x] **21 benchmark files exist** with 0% categorization → **All 21 benchmarks categorized**
- [x] **Advanced llmspell-testing infrastructure** is completely underutilized → **Feature system configured**
- [x] **External dependency tests** mixed with unit tests cause flaky CI → **35 external tests isolated**
- [x] **No standardized test execution** patterns across crates → **Fast/comprehensive/external suites created**
- [x] **Duplicate test infrastructure** instead of shared utilities → **COMPLETED in Step 7 - All 6 phases of systematic duplicate removal**

**Files to Update** ✅ **COMPLETED**:
- [x] All `src/` files with `#[test]` or `#[tokio::test]` (337 unit tests categorized)
- [x] All `tests/` directory files (142 integration, 35 external tests categorized)
- [x] Update `Cargo.toml` files to reference llmspell-testing features (completed)
- [x] Consolidate test utilities into llmspell-testing (Step 6 & 7 - Test Infrastructure Consolidation COMPLETED)
- [x] Update CI configuration to use categorized test execution (cfg_attr syntax issue resolved, feature flags working)

**Expected Outcome**:
- **Fast feedback loop**: Unit + Integration tests run in <35 seconds
- **Reliable CI**: No flaky external dependency failures
- **Developer productivity**: `cargo test --fast` vs `cargo test --all`
- **Clear test separation**: Unit vs Integration vs External clearly defined
- **Unified infrastructure**: All crates use llmspell-testing utilities

**Acceptance Criteria** ✅ **COMPLETED** (with cfg_attr syntax caveat):
- [x] All unit tests properly categorized with `#[cfg_attr(test_category = "unit")]` (337 tests)
- [x] All integration tests properly categorized with `#[cfg_attr(test_category = "integration")]` (142 tests)
- [x] All external dependency tests categorized with `#[cfg_attr(test_category = "external")]` (35 tests)
- [⚠️] Fast test suite runs in <35 seconds (unit + integration) - **blocked by cfg_attr syntax issue**
- [x] External tests properly isolated and skipped in CI by default (feature flags configured)
- [ ] Duplicate test infrastructure removed, unified in llmspell-testing
- [ ] Test execution documented with clear categories
- [ ] CI runs only fast tests, external tests require manual trigger
- [ ] All test categorization tests passing
- [ ] Quality checks passing

---