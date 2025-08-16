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

## Task List 
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

---

### Set 3: Example Reorganization (Day 5-6)

#### Task 7.3.1: Example Audit and Categorization
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: COMPLETED ✅
**Assigned To**: Documentation Team

**Description**: Comprehensive audit of all 156+ examples to categorize by audience, scope, and purpose.

**Implementation Steps**:
1. [x] **Discovery and Inventory** (1 hour):
   - [x] List all examples in `/examples/`: `find examples -name "*.lua" -o -name "*.rs" | sort`
   - [x] List per-crate examples: `find llmspell-*/examples -name "*.rs" | sort`
   - [x] Find test examples: `grep -r "fn main()" llmspell-*/tests/ | grep -v "test fn"`
   - [x] Document example count by location and type
   - [x] Create master inventory spreadsheet

2. [x] **Categorization** (1.5 hours):
   - [x] Tag each example by audience: Script Users, Rust Developers, System Integrators
   - [x] Tag by scope: Learning, Integration, Production
   - [x] Tag by feature area: agents, tools, workflows, hooks, events, state
   - [x] Identify duplicate examples covering same functionality
   - [x] Note examples that serve multiple purposes

3. [x] **Gap Analysis** (1 hour):
   - [x] Identify missing getting-started examples
   - [x] Find feature areas lacking examples
   - [x] Note missing error handling examples
   - [x] List needed deployment/production examples
   - [x] Document testing pattern gaps

4. [x] **Quality Assessment** (30 min):
   - [x] Check which examples are broken/outdated
   - [x] Verify examples against current APIs
   - [x] Test example runnability
   - [x] Note examples needing updates

**Deliverables**:
- [x] Example inventory spreadsheet with categorization (INVENTORY.md, inventory.csv)
- [x] Gap analysis report (GAP_ANALYSIS.md)
- [x] Migration priority list (MIGRATION_PLAN.md)
- [x] Quality issues list (in GAP_ANALYSIS.md)

**Acceptance Criteria**:
- [x] All 125 examples inventoried (94 Lua, 31 Rust)
- [x] Each example categorized by audience and scope
- [x] Duplicates identified (~15% duplicative)
- [x] Gaps documented (critical gaps in getting started, error handling, production)
- [x] Migration plan created (phased approach with priorities)

---

#### Task 7.3.2: Example Directory Structure Creation
**Priority**: HIGH
**Estimated Time**: 2 hours
**Status**: COMPLETED ✅
**Assigned To**: Documentation Team
**Dependencies**: Task 7.3.1

**Description**: Create new example directory structure organized by audience and learning path.

**Implementation Steps**:
1. [x] **Create Directory Hierarchy** (30 min):
   ```bash
   mkdir -p examples/script-users/{getting-started,features,cookbook,applications}
   mkdir -p examples/rust-developers/{getting-started,api-usage,patterns,extensions}
   mkdir -p examples/tests-as-examples/{integration,benchmarks}
   ```

2. [x] **Create Navigation Structure** (1 hour):
   - [x] Create README.md in each directory explaining its purpose
   - [x] Add example metadata template
   - [x] Create cross-reference index
   - [x] Add learning path guides

3. [x] **Establish Standards** (30 min):
   - [x] Define metadata header format
   - [x] Create example template files
   - [x] Document naming conventions
   - [x] Set up example testing framework

**Files to Create**:
- `examples/README.md` - Main navigation guide ✅
- `examples/script-users/README.md` - Script user guide ✅
- `examples/script-users/getting-started/README.md` - Learning path ✅
- `examples/rust-developers/README.md` - Developer guide ✅
- `examples/STANDARDS.md` - Example standards document ✅

**Acceptance Criteria**:
- [x] Complete directory structure created
- [x] Navigation READMEs in place
- [x] Standards documented
- [x] Templates created
- [x] Testing framework ready

---

#### Task 7.3.3: Core Example Migration
**Priority**: HIGH
**Estimated Time**: 8 hours
**Status**: IN PROGRESS
**Assigned To**: Documentation Team
**Dependencies**: Task 7.3.2

**Description**: Systematically migrate, test, and validate all examples with proper metadata and structure.

**Migration Methodology**: Test → Document → Add Metadata → Update Paths → Move → Retest

**Implementation Steps**:

1. [ ] **Phase 0: Validate Already-Moved Files** (30 min): IN PROGRESS
   - [ ] Test 6 files moved without proper validation:
     - [ ] `00-hello-world.lua` - Test output matches expected
     - [ ] `comprehensive-demo.lua` - Test with API keys
     - [ ] `provider-info.lua` - Test provider detection
     - [ ] `streaming-responses.lua` - Test streaming capability
     - [ ] `multimodal.lua` - Test multimodal features
     - [ ] `performance-validation.rs` - Test benchmark runs
   - [ ] Add metadata headers to these files
   - [ ] Document any issues found

2. [ ] **Phase 1: Baseline Testing Matrix** (1 hour)
   - [ ] Test ALL remaining examples in CURRENT locations
   - [ ] Create testing categories:
     - **No Dependencies**: Can run immediately
     - **Config Required**: Need specific .toml files
     - **API Keys Required**: Need OPENAI_API_KEY or ANTHROPIC_API_KEY
     - **Special Setup**: Need database, network services, etc.
     - **Test Files**: Should go to tests-as-examples/
     - **Broken/Outdated**: Need fixes before migration
   - [ ] Document expected output for each working example
   - [ ] Create `TESTING_MATRIX.md` with results

3. [ ] **Phase 2: Systematic Migration by Category** (4 hours)
   
   **Process for EACH file**:
   1. Verify baseline test result from Phase 1
   2. Add metadata header BEFORE moving:
      ```lua
      -- Example: [Name]
      -- Purpose: [What it demonstrates]
      -- Prerequisites: [API keys, configs needed]
      -- Expected Output: [What should happen]
      -- Version: 0.7.0
      -- Tags: [relevant tags]
      ```
   3. Update internal paths/requires for new structure
   4. Move to appropriate new location
   5. Test in new location - verify produces expected output
   6. Mark status: ✅ Working | ⚠️ Needs Fix | ❌ Blocked
   
   **Migration Groups** (based on testing requirements):
   
   - [ ] **Group A: No Dependencies** (test & migrate first):
     - Tools that work without configs/APIs (filesystem, utility, system)
     - Basic examples (hello world variants)
     - Debug/diagnostic scripts
   
   - [ ] **Group B: Config Required**:
     - State persistence examples (need state-enabled.toml)
     - Backup examples (need backup-enabled.toml)
     - Migration examples (need migration-enabled.toml)
     - Session examples (need session-enabled.toml)
   
   - [ ] **Group C: API Keys Required**:
     - All agent examples (need OPENAI_API_KEY or ANTHROPIC_API_KEY)
     - AI-powered applications
     - Examples using LLM features
   
   - [ ] **Group D: Test Files** (go to tests-as-examples/):
     - Files with "test_" prefix
     - Benchmark files (*-benchmark.lua, *-performance.lua)
     - Test runners (run-*.lua)
     - Validation scripts
   
4. [ ] **Phase 3: Handle Special Cases** (1 hour)
   - [ ] **Shell Scripts**:
     - [ ] Migrate and update paths in shell scripts:
       - `run-all-agent-examples.sh` → Update for new structure
       - `run-all-tools-examples.sh` → Update for new structure
       - `run-workflow-examples.sh` → Update for new structure
       - `run-all-lua-examples.sh` → Update for new structure
     - [ ] Create NEW runner scripts for organized structure
   
   - [ ] **Config Files**:
     - [ ] Move all .toml configs to `script-users/configs/`
     - [ ] Ensure configs work from new location
   
   - [ ] **Duplicates to Remove**:
     - [ ] Identify and remove duplicate examples (see MIGRATION_PLAN.md)
     - [ ] Document which files were removed and why

5. [ ] **Phase 4: Create Missing Examples** (1 hour)
   - [ ] Create `script-users/getting-started/01-first-tool.lua` (from tools-showcase.lua)
   - [ ] Create `script-users/getting-started/02-first-agent.lua` (from agent-simple.lua)
   - [ ] Create `script-users/getting-started/03-first-workflow.lua` (from workflow-basics-sequential.lua)
   - [ ] Create `script-users/getting-started/04-save-state.lua` (from basic_persistence.lua)
   - [ ] Create `script-users/getting-started/05-handle-errors.lua` (NEW - no existing example)

6. [ ] **Phase 5: Cleanup and Validation** (1 hour)
   - [ ] Remove empty directories (lua/, configs/, state_persistence/, etc.)
   - [ ] Remove identified duplicates
   - [ ] Final test of all migrated examples:
     - [ ] Getting-started sequence (00-05) works
     - [ ] Feature examples demonstrate features
     - [ ] Cookbook patterns are self-contained
     - [ ] Applications run with proper setup
   - [ ] Update all documentation references to new paths
   - [ ] Create MIGRATION_NOTES.md for users

**Summary of Files Going to tests-as-examples** (21 files total):
- **Benchmarks** (8 files):
  - `agent-simple-benchmark.lua` → benchmarks/
  - `tools-performance.lua` → benchmarks/
  - `event-performance.lua` → benchmarks/
  - `event-statistics.lua` → benchmarks/
  - `run-performance-benchmarks.lua` → benchmarks/
  - `performance_validation.rs` → benchmarks/
- **Integration Tests** (13 files):
  - `test_replay_basic.lua` → integration/
  - `test_replay_minimal.lua` → integration/
  - `test_state_api.lua` → integration/
  - `test_migration_api.lua` → integration/
  - `backup_validation.lua` → integration/
  - `run-all-examples.lua` → integration/
  - `run-integration-demos.lua` → integration/
  - `debug_globals.lua` → integration/

**Quality Assurance**:
- [ ] **Testing Categories Summary**:
  - [ ] Examples that run without dependencies: Test all
  - [ ] Examples needing configs: Test with appropriate configs from script-users/configs/
  - [ ] Examples needing API keys: Test with mock/real keys, ensure graceful failure
  - [ ] Test files: Verify they're in tests-as-examples/ and run as tests
- [ ] **File Integrity**:
  - [ ] All files have metadata headers per STANDARDS.md
  - [ ] Internal paths/requires are updated for new structure
  - [ ] No broken cross-references
  - [ ] No files lost in migration (check count: ~125 total)
- [ ] **Functionality**:
  - [ ] Getting-started examples 00-05 run in sequence
  - [ ] Feature examples demonstrate their features
  - [ ] Cookbook patterns are self-contained
  - [ ] Applications run (with appropriate setup)

**Acceptance Criteria**:
- [ ] All examples migrated to new structure
- [ ] Metadata headers added
- [ ] Duplicates consolidated
- [ ] All examples tested
- [ ] Documentation updated

---

#### Task 7.3.4: Getting Started Experience
**Priority**: CRITICAL
**Estimated Time**: 8 hours
**Status**: TODO
**Assigned To**: Developer Experience Team
**Dependencies**: Task 7.3.3

**Description**: Create progressive getting-started examples for each audience with clear learning paths.

**Implementation Steps**:
1. [ ] **Script Users Path** (3 hours):
   - [ ] `01-hello-world/` - Simplest possible example
   - [ ] `02-first-tool/` - Using a single tool
   - [ ] `03-simple-agent/` - Creating an agent
   - [ ] `04-basic-workflow/` - Simple workflow
   - [ ] `05-state-persistence/` - Saving state
   - [ ] `06-error-handling/` - Handling errors
   - [ ] Create README with learning progression

2. [ ] **Rust Developers Path** (3 hours):
   - [ ] `01-embed-llmspell/` - Basic embedding
   - [ ] `02-custom-tool/` - Creating a tool
   - [ ] `03-custom-agent/` - Building an agent
   - [ ] `04-testing-components/` - Testing patterns
   - [ ] `05-async-patterns/` - Async usage
   - [ ] Create developer learning guide

3. [ ] **Quick Start Guides** (2 hours):
   - [ ] 5-minute quick start for each audience
   - [ ] Copy-paste ready examples
   - [ ] Common task recipes
   - [ ] Troubleshooting guide

**Quality Requirements**:
- [ ] Each example must be self-contained
- [ ] Clear progression in complexity
- [ ] Extensive comments explaining concepts
- [ ] Expected output documented
- [ ] Common errors addressed

**Acceptance Criteria**:
- [ ] Complete learning paths for both audiences
- [ ] All examples tested by newcomers
- [ ] Quick start guides created
- [ ] Troubleshooting documented
- [ ] Feedback incorporated

---

#### Task 7.3.5: Cookbook and Patterns
**Priority**: HIGH
**Estimated Time**: 6 hours
**Status**: TODO
**Assigned To**: Core Team
**Dependencies**: Task 7.3.3

**Description**: Create cookbook-style examples for common patterns and use cases.

**Implementation Steps**:
1. [ ] **Script Cookbook** (3 hours):
   - [ ] Error handling patterns
   - [ ] Retry and timeout strategies
   - [ ] State management patterns
   - [ ] Multi-agent coordination
   - [ ] Tool composition patterns
   - [ ] Performance optimization
   - [ ] Testing script code

2. [ ] **Rust Patterns** (2 hours):
   - [ ] Dependency injection patterns
   - [ ] Custom provider implementation
   - [ ] Storage backend creation
   - [ ] Hook system extensions
   - [ ] Performance profiling
   - [ ] Integration testing

3. [ ] **Production Patterns** (1 hour):
   - [ ] Configuration management
   - [ ] Secret handling
   - [ ] Monitoring integration
   - [ ] Deployment strategies
   - [ ] Security hardening

**Documentation**:
- [ ] Each pattern with problem statement
- [ ] Solution explanation
- [ ] Complete working code
- [ ] When to use/not use
- [ ] Performance implications

**Acceptance Criteria**:
- [ ] 20+ cookbook examples created
- [ ] All patterns documented
- [ ] Examples tested
- [ ] Performance notes included
- [ ] Security considerations documented

---

#### Task 7.3.6: Real-World Applications
**Priority**: MEDIUM
**Estimated Time**: 8 hours
**Status**: TODO
**Assigned To**: Solutions Team
**Dependencies**: Task 7.3.4

**Description**: Enhance and organize real-world application examples that demonstrate production usage.

**Implementation Steps**:
1. [ ] **Enhance Existing Applications** (4 hours):
   - [ ] AI Research Assistant:
     - [ ] Add comprehensive error handling
     - [ ] Include rate limiting
     - [ ] Add state persistence
     - [ ] Document deployment
   - [ ] Data Pipeline:
     - [ ] Add monitoring hooks
     - [ ] Include failure recovery
     - [ ] Add performance tuning
     - [ ] Document scaling
   - [ ] Monitoring System:
     - [ ] Add alerting integration
     - [ ] Include dashboard setup
     - [ ] Add metric collection
     - [ ] Document operations

2. [ ] **Create New Applications** (3 hours):
   - [ ] Customer Support Bot:
     - [ ] Multi-channel support
     - [ ] Context persistence
     - [ ] Escalation workflows
   - [ ] Content Generation System:
     - [ ] Template management
     - [ ] Quality checks
     - [ ] Batch processing
   - [ ] Code Review Assistant:
     - [ ] Git integration
     - [ ] PR analysis
     - [ ] Suggestion generation

3. [ ] **Production Readiness** (1 hour):
   - [ ] Add deployment configurations
   - [ ] Include monitoring setup
   - [ ] Document scaling considerations
   - [ ] Add operational runbooks

**Quality Standards**:
- [ ] Production-quality error handling
- [ ] Comprehensive testing included
- [ ] Performance considerations documented
- [ ] Security best practices followed
- [ ] Operational guidance provided

**Acceptance Criteria**:
- [ ] 6+ complete applications
- [ ] All production-ready
- [ ] Deployment documented
- [ ] Testing included
- [ ] Operations guide created

---

#### Task 7.3.7: Example Testing Framework
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Test Team
**Dependencies**: Task 7.3.2

**Description**: Create automated testing for all examples to ensure they remain functional.

**Implementation Steps**:
1. [ ] **Test Infrastructure** (1.5 hours):
   - [ ] Create example test runner
   - [ ] Set up CI integration
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
- [ ] CI integration complete
- [ ] Nightly runs configured
- [ ] Test reports generated
- [ ] Breaking changes detected

---

#### Task 7.3.8: Example Documentation Integration
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

#### Task 4.1: User Guide Standardization
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

#### Task 4.2: Technical Documentation Cleanup
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Architecture Team

**Description**: Update technical documentation to reflect current implementation.

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

#### Task 4.3: Developer Guide Enhancement
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