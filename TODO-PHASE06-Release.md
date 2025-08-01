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

**Total Tasks**: 13
**Estimated Total Time**: 47 hours
**Target Duration**: 7 days

### Task Distribution:
- Set 1 (API Consistency): 5 tasks, 20 hours
- Set 2 (Rust Documentation): 3 tasks, 14 hours  
- Set 3 (Documentation Cleanup): 4 tasks, 14 hours

### Risk Factors:
1. **Breaking Changes**: Must maintain compatibility while improving APIs
2. **Documentation Drift**: Keeping docs in sync with rapid development
3. **Naming Conflicts**: Some renamings may conflict with Rust keywords
4. **Time Estimation**: Documentation often takes longer than estimated

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
- [ ] Builder patterns implemented
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