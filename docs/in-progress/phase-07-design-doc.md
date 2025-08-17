# Phase 7: API Consistency and Documentation Polish - Design Document

**Version**: 1.0  
**Date**: December 2024  
**Status**: Planning Phase  
**Phase**: 7 (API Consistency and Documentation Polish)  
**Timeline**: Weeks 23-27 (5 weeks)  
**Priority**: CRITICAL (Pre-1.0 Release Requirement)  
**Dependencies**: Phase 6 Session Management ✅ COMPLETE  
**Scope**: 40 tasks across API standardization, documentation, and example reorganization

> **📋 Pre-1.0 Polish Guide**: This document provides complete specifications for standardizing all public APIs, reorganizing examples, and polishing documentation to establish a stable 1.0 release foundation for rs-llmspell.

---

## Phase Overview

### Goal
Standardize all public APIs across the codebase, reorganize 156+ examples by audience, ensure comprehensive documentation coverage, and establish a clean, consistent API surface for the 1.0 release.

### Core Principles
- **Clean Break Approach**: No backward compatibility requirements - APIs changed in place for 1.0 stability
- **Test-First Foundation**: Implement test categorization before API changes to avoid rework
- **Audience-Driven Examples**: Reorganize examples by target audience (Script Users, Rust Developers, System Integrators)
- **Consistency Over Convenience**: Uniform patterns across all APIs, even if it means more typing
- **Documentation as Code**: 100% rustdoc coverage enforced by CI
- **Progressive Learning**: Examples organized from simple to complex with clear learning paths

### Success Criteria
- [ ] All APIs follow consistent naming patterns (get_*, set_*, *Manager suffix)
- [ ] 100% builder pattern adoption for all configuration objects
- [ ] Test suite properly categorized (unit <5s, integration <30s, external isolated)
- [ ] 156+ examples reorganized by audience with metadata headers
- [ ] Hook execution working consistently across all crates
- [ ] Complete rustdoc coverage for all public APIs
- [ ] User and developer documentation standardized
- [ ] All breaking changes documented with migration guide
- [ ] Example testing framework with CI integration
- [ ] Zero API inconsistencies detected by linting tools

---

## 1. API Consistency Specifications

### 1.1 Naming Convention Standards

**Method Naming Patterns:**
```rust
// Accessor patterns
fn get_<property>(&self) -> &T           // Immutable access
fn get_<property>_mut(&mut self) -> &mut T   // Mutable access
fn set_<property>(&mut self, value: T)   // Property setter
fn take_<property>(self) -> T            // Ownership transfer

// Collection patterns
fn list_<items>(&self) -> Vec<T>         // Return all items
fn add_<item>(&mut self, item: T)        // Add single item
fn remove_<item>(&mut self, id: &Id)      // Remove by identifier
fn clear_<items>(&mut self)              // Remove all items

// Lifecycle patterns
fn create_<resource>(...) -> Result<T>    // Resource creation
fn destroy_<resource>(&mut self, id: &Id) // Resource cleanup
fn initialize(&mut self) -> Result<()>    // Lazy initialization
fn shutdown(&mut self) -> Result<()>      // Graceful shutdown

// Query patterns
fn find_<item>(&self, predicate: P) -> Option<&T>
fn filter_<items>(&self, predicate: P) -> Vec<&T>
fn contains_<item>(&self, item: &T) -> bool
fn is_<state>(&self) -> bool              // State checking
```

**Service Naming Migration:**
```rust
// OLD: *Service suffix
pub struct AgentService { ... }
pub struct ToolService { ... }

// NEW: *Manager suffix
pub struct AgentManager { ... }
pub struct ToolManager { ... }
```

**Builder Pattern Standardization:**
```rust
// Every configuration struct MUST have a builder
pub struct SessionManagerConfig {
    max_active_sessions: usize,
    default_timeout: Duration,
    storage_path: PathBuf,
}

impl SessionManagerConfig {
    pub fn builder() -> SessionManagerConfigBuilder {
        SessionManagerConfigBuilder::default()
    }
}

// Builder provides fluent interface
let config = SessionManagerConfig::builder()
    .max_active_sessions(100)
    .default_timeout(Duration::from_secs(3600))
    .storage_path("/var/lib/llmspell/sessions")
    .build()?;
```

### 1.2 Workflow-Agent Integration

**Following Google Agent Development Kit (ADK) Pattern - Core Paradigm Shift:**

**🚨 BREAKING CHANGE**: Workflows transform from "pre-configured batch jobs" to "function-like agents" with no backward compatibility. Current workflows execute `async fn execute(&self) -> Result<WorkflowResult>` but must become agents with `async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>`.

**Workflow Composition & Recursive Patterns:**
```rust
// KEY PATTERN: Workflows can contain other workflows as steps
let complex_workflow = SequentialWorkflow::builder()
    .add_step_tool("file_reader", tool_params)
    .add_step_agent(analysis_agent)           // Agent as step
    .add_step_workflow(preprocessing_workflow) // Workflow as step ← CORE ADK PATTERN
    .add_step_tool("file_writer", output_params)
    .build();

// Workflows are first-class agents
let agent_pool: Vec<Arc<dyn BaseAgent>> = vec![
    Arc::new(chatbot_agent),
    Arc::new(sequential_workflow),  // Workflow IS an agent
    Arc::new(data_analysis_tool),
];
```

**Input/Output Adapter Architecture (Task 7.1.7 Step 4):**
```rust
// Critical adapter layer for type unification
impl BaseAgent for SequentialWorkflow {
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        // 1. Convert AgentInput → WorkflowInput
        let workflow_input = WorkflowInputAdapter::from_agent_input(input)?;
        
        // 2. Execute workflow-specific logic  
        let workflow_output = self.execute_internal(workflow_input).await?;
        
        // 3. Convert WorkflowOutput → AgentOutput
        let agent_output = AgentOutputAdapter::from_workflow_output(workflow_output)?;
        Ok(agent_output)
    }
}

// Input adapter handles parameter extraction
struct WorkflowInputAdapter;
impl WorkflowInputAdapter {
    fn from_agent_input(input: AgentInput) -> Result<WorkflowInput> {
        let workflow_params = input.parameters.get("workflow_config")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        
        Ok(WorkflowInput::new(input.text)
            .with_context("agent_input", serde_json::to_value(input.parameters)?)
            .with_workflow_params(workflow_params))
    }
}

// Output adapter preserves workflow execution details
struct AgentOutputAdapter;
impl AgentOutputAdapter {
    fn from_workflow_output(output: WorkflowOutput) -> Result<AgentOutput> {
        let mut agent_output = AgentOutput::text(
            serde_json::to_string_pretty(&output.output)?
        );
        
        // Preserve workflow execution metadata
        agent_output = agent_output
            .add_metadata("workflow_duration", output.duration.as_millis())
            .add_metadata("steps_executed", output.steps_executed)
            .add_metadata("workflow_success", output.success);
            
        Ok(agent_output)
    }
}
```

**Unified Type System:**
```rust
// All workflow patterns implement both BaseAgent AND Workflow traits
impl BaseAgent for SequentialWorkflow { /* agent interface */ }
impl Workflow for SequentialWorkflow { /* workflow-specific methods */ }

// Workflows are fully interchangeable with agents
enum ComponentType {
    Agent(Arc<dyn BaseAgent>),
    Tool(Arc<dyn Tool>),        // Tool also implements BaseAgent
    Workflow(Arc<dyn Workflow>), // Workflow also implements BaseAgent  
}
```

### 1.3 Workflow Factory Standardization (Task 7.1.8)

**Unified Workflow Creation Patterns:**
```rust
// Standardized workflow factory interface
pub trait WorkflowFactory {
    /// Create workflow from type string and configuration
    fn create_workflow(workflow_type: &str, config: WorkflowConfig) -> Result<Arc<dyn Workflow>>;
    
    /// List all available workflow types
    fn list_workflow_types() -> Vec<String>;
    
    /// Create workflow from predefined template
    fn create_from_template(template_name: &str) -> Result<Arc<dyn Workflow>>;
    
    /// Register custom workflow patterns
    fn register_workflow_type<T: Workflow + 'static>(type_name: &str);
}

// Default implementation for all standard patterns
pub struct DefaultWorkflowFactory;
impl WorkflowFactory for DefaultWorkflowFactory {
    fn create_workflow(workflow_type: &str, config: WorkflowConfig) -> Result<Arc<dyn Workflow>> {
        match workflow_type {
            "sequential" => Ok(Arc::new(SequentialWorkflow::new("default".to_string(), config))),
            "parallel" => Ok(Arc::new(ParallelWorkflow::new("default".to_string(), config))),
            "conditional" => Ok(Arc::new(ConditionalWorkflow::new("default".to_string(), config))),
            "loop" => Ok(Arc::new(LoopWorkflow::new("default".to_string(), config))),
            _ => Err(LLMSpellError::Component { 
                message: format!("Unknown workflow type: {}", workflow_type) 
            })
        }
    }
}

// Workflow executor for execution management
pub trait WorkflowExecutor {
    /// Execute workflow with input and return output
    async fn execute_workflow(
        &self, 
        workflow: Arc<dyn Workflow>, 
        input: WorkflowInput
    ) -> Result<WorkflowOutput>;
    
    /// Execute with cancellation support
    async fn execute_with_cancellation(
        &self,
        workflow: Arc<dyn Workflow>,
        input: WorkflowInput,
        cancellation_token: CancellationToken,
    ) -> Result<WorkflowOutput>;
    
    /// Get execution metrics and monitoring
    fn get_execution_metrics(&self) -> ExecutionMetrics;
}
```

### 1.4 Pattern-Specific Config Builders (Task 7.1.9)

**All Workflow Configurations MUST Use Builder Pattern:**
```rust
// Core WorkflowConfig builder (enhanced)
let base_config = WorkflowConfig::builder()
    .max_execution_time(Duration::from_secs(300))
    .default_step_timeout(Duration::from_secs(30))
    .max_retry_attempts(3)
    .retry_delay_ms(1000)
    .exponential_backoff(true)
    .error_strategy(ErrorStrategy::FailFast)
    .build()?;

// Pattern-specific config builders
let parallel_config = ParallelConfig::builder()
    .max_concurrency(4)
    .fail_fast(true)
    .timeout(Duration::from_secs(60))
    .continue_on_optional_failure(true)
    .build()?;

let conditional_config = ConditionalConfig::builder()
    .execute_all_matching(false)
    .execute_default_on_no_match(true)
    .max_branches_to_evaluate(100)
    .condition_evaluation_timeout_ms(1000)
    .short_circuit_evaluation(true)
    .build()?;

let loop_config = LoopConfig::builder()
    .max_iterations(1000)
    .break_on_error(true)
    .iteration_delay_ms(0)
    .result_aggregation(ResultAggregation::CollectAll)
    .build()?;

let sequential_config = SequentialConfig::builder()
    .continue_on_step_failure(false)
    .step_isolation_level(IsolationLevel::Shared)
    .checkpoint_frequency(CheckpointFrequency::EveryStep)
    .build()?;

// Preset configurations for common use cases
let fast_config = WorkflowConfig::fast()      // Optimized for speed
    .with_parallel_config(ParallelConfig::max_throughput())
    .build()?;
    
let robust_config = WorkflowConfig::robust()  // Optimized for reliability
    .with_retry_strategy(RetryStrategy::ExponentialBackoff { max_attempts: 5 })
    .build()?;
```

### 1.5 Discovery Pattern Unification

**Unified Discovery Trait:**
```rust
pub trait BridgeDiscovery<T: ComponentInfo> {
    /// List all available component types
    fn discover_types(&self) -> Vec<String>;
    
    /// Get detailed information about a type
    fn get_type_info(&self, type_name: &str) -> Option<T>;
    
    /// List all active instances
    fn list_instances(&self) -> Vec<ComponentId>;
    
    /// Check if a type is available
    fn has_type(&self, type_name: &str) -> bool;
    
    /// Query components by capabilities
    fn query_by_capability(&self, capability: &str) -> Vec<String>;
}

// Implement for all component types
impl BridgeDiscovery<AgentInfo> for AgentDiscovery { ... }
impl BridgeDiscovery<WorkflowInfo> for WorkflowDiscovery { ... }
impl BridgeDiscovery<ToolInfo> for ToolDiscovery { ... }
impl BridgeDiscovery<StorageInfo> for StorageDiscovery { ... }
```

### 1.4 Script API Standardization

**Lua/JavaScript Naming Consistency:**
```lua
-- OLD: Mixed camelCase and snake_case
agent:getMetrics()
Session.getCurrent()
workflow.listTypes()

-- NEW: Consistent snake_case
agent:get_metrics()
Session.get_current()
workflow.list_types()

-- Factory methods remain descriptive
Workflow.sequential()  -- Creates sequential workflow
Workflow.parallel()    -- Creates parallel workflow
Agent.create()         -- Creates agent
```

### 1.5 Hook Execution Fix

**Standardized Hook Execution Pattern:**
```rust
// NOT THIS (found in tools/workflows):
async fn execute_hook_phase<T>(...) -> Result<Option<T>> {
    // TODO: Get hooks from registry
    tokio::time::sleep(Duration::from_millis(1)).await;
    Ok(None)
}

// THIS (proper implementation):
async fn execute_hook_phase<T>(...) -> Result<Option<T>> {
    let hooks = self.hook_registry
        .get_hooks(hook_point)
        .await?;
        
    let mut hook_context = HookContext::new()
        .with_component_id(self.id())
        .with_data(data);
        
    let results = self.hook_executor
        .execute_hooks(&hooks, &mut hook_context)
        .await?;
        
    Ok(results.into_modified_data())
}
```

---

## 2. Test Architecture Specifications

### 2.1 Test Categorization System

**Category Attributes:**
```rust
// Unit tests - fast, isolated
#[test]
#[cfg_attr(test_category = "unit")]
fn test_agent_creation() { ... }

// Integration tests - cross-component
#[test]
#[cfg_attr(test_category = "integration")]
fn test_agent_tool_interaction() { ... }

// External dependency tests
#[test]
#[cfg_attr(test_category = "external")]
#[ignore] // Run only with --ignored flag
fn test_openai_api_integration() { ... }

// Component-specific categories
#[test]
#[cfg_attr(test_category = "agent")]
#[cfg_attr(test_category = "unit")]
fn test_agent_lifecycle() { ... }
```

**Test Execution Profiles:**
```bash
# Fast feedback loop (<35 seconds)
cargo test --features unit-tests,integration-tests

# Complete test suite
cargo test --features all-tests

# External tests only (requires credentials)
cargo test --features external-tests -- --ignored

# Component-specific tests
cargo test --features agent-tests
```

### 2.2 Test Infrastructure

**Leverage llmspell-testing Crate:**
```rust
use llmspell_testing::{
    // Mocks
    mocks::{MockBaseAgent, MockTool, MockProvider},
    
    // Fixtures
    fixtures::{load_fixture, test_agent_config},
    
    // Generators for property testing
    generators::{agent_id_strategy, workflow_config_strategy},
    
    // Test runners
    runners::{CategoryTestRunner, TestCategory},
};

// Unified test setup
#[test_case]
async fn test_with_category() {
    let runner = CategoryTestRunner::new()
        .with_category(TestCategory::Unit)
        .with_timeout(Duration::from_secs(5));
        
    runner.run_async(async {
        // Test implementation
    }).await;
}
```

---

## 3. Example Reorganization Specifications

### 3.1 Directory Structure

**Audience-Based Organization:**
```
examples/
├── README.md                    # Navigation guide with learning paths
├── STANDARDS.md                # Example standards and metadata format
│
├── script-users/               # For Lua/JS users
│   ├── getting-started/        
│   │   ├── 01-hello-world/
│   │   │   ├── hello.lua
│   │   │   ├── README.md
│   │   │   └── expected-output.txt
│   │   ├── 02-first-tool/
│   │   ├── 03-simple-agent/
│   │   └── ...
│   ├── features/              
│   │   ├── agents/
│   │   ├── tools/
│   │   └── workflows/
│   ├── cookbook/              
│   │   ├── error-handling/
│   │   ├── retry-patterns/
│   │   └── performance/
│   └── applications/          
│       ├── research-assistant/
│       ├── data-pipeline/
│       └── monitoring-system/
│
├── rust-developers/           # For Rust library users
│   ├── getting-started/
│   │   ├── 01-embed-llmspell/
│   │   │   ├── Cargo.toml
│   │   │   ├── src/main.rs
│   │   │   └── README.md
│   │   └── ...
│   ├── api-usage/
│   ├── patterns/
│   └── extensions/
│
└── deployment/               # For production users
    ├── docker/
    ├── kubernetes/
    └── monitoring/
```

### 3.2 Example Metadata Standard

**Required Metadata Header:**
```lua
-- Example: Simple Agent Creation
-- Audience: Script Users
-- Level: Beginner
-- Requires: llmspell v0.6+
-- Topics: agents, providers, queries
-- Related: 02-first-tool, 04-agent-composition
-- Output: Text response from agent

--[[
Description:
This example demonstrates creating a simple agent and making
a basic query. It shows proper error handling and response
processing.
--]]

-- Example code follows...
```

### 3.3 Progressive Learning Paths

**Script Users Path:**
1. **Hello World** → Basic setup and verification
2. **First Tool** → Using a single tool
3. **Simple Agent** → Creating and querying an agent
4. **Basic Workflow** → Sequential task execution
5. **State Persistence** → Saving and loading state
6. **Error Handling** → Proper error management
7. **Multi-Agent** → Coordinating multiple agents
8. **Production App** → Complete application

**Rust Developers Path:**
1. **Embed LLMSpell** → Library integration
2. **Custom Tool** → Implementing Tool trait
3. **Custom Agent** → Extending BaseAgent
4. **Testing Components** → Unit and integration tests
5. **Async Patterns** → Advanced async usage
6. **Custom Provider** → Provider implementation
7. **Performance Tuning** → Optimization techniques

---

## 4. Documentation Specifications

### 4.1 Rustdoc Standards

**Module Documentation:**
```rust
//! # Module Name
//! 
//! Brief one-line description of the module's purpose.
//! 
//! ## Overview
//! 
//! Detailed explanation of what this module provides and why it exists.
//! This section should give readers enough context to understand when
//! and how to use this module.
//! 
//! ## Examples
//! 
//! ```rust
//! use llmspell_tools::JsonProcessorTool;
//! 
//! let tool = JsonProcessorTool::new();
//! let result = tool.process(input)?;
//! ```
//! 
//! ## Performance Considerations
//! 
//! Note any performance implications or optimization opportunities.
//! 
//! ## Security Considerations
//! 
//! Document any security implications or requirements.

/// Brief one-line description of the item.
/// 
/// Detailed explanation including:
/// - Purpose and use cases
/// - Parameter constraints
/// - Return value semantics
/// - Error conditions
/// 
/// # Arguments
/// 
/// * `param1` - Description with constraints
/// * `param2` - Description with valid ranges
/// 
/// # Returns
/// 
/// Description of return value and its meaning.
/// 
/// # Errors
/// 
/// * `ErrorType::Variant1` - When this happens
/// * `ErrorType::Variant2` - When that happens
/// 
/// # Examples
/// 
/// ```rust
/// let result = function(param1, param2)?;
/// assert_eq!(result, expected);
/// ```
/// 
/// # Panics
/// 
/// Describes any panic conditions (should be rare).
/// 
/// # Safety
/// 
/// For unsafe functions, explains safety requirements.
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

### 4.2 User Guide Standardization

**Consistent Document Structure:**
```markdown
# Document Title

## Overview
Brief introduction explaining what this guide covers and who it's for.

## Prerequisites
- Required knowledge
- Required software
- Required configuration

## Quick Start
Minimal example to get users started quickly.

## Core Concepts
Explanation of fundamental concepts needed to understand the topic.

## Step-by-Step Guide
Detailed walkthrough of common tasks.

## Advanced Usage
More complex scenarios and optimizations.

## Examples
Multiple working examples with explanations.

## API Reference
Links to relevant rustdoc pages.

## Troubleshooting
Common issues and their solutions.

## Related Topics
Links to other relevant documentation.
```

### 4.3 Migration Guide

**Breaking Changes Documentation:**
```markdown
# Phase 7 Breaking Changes

## Service → Manager Renaming
**Old**: `AgentService`, `ToolService`
**New**: `AgentManager`, `ToolManager`
**Migration**: Simple find/replace

## Struct Literal → Builder Pattern
**Old**: 
```rust
let config = SessionManagerConfig {
    max_sessions: 100,
    timeout: Duration::from_secs(3600),
};
```
**New**:
```rust
let config = SessionManagerConfig::builder()
    .max_sessions(100)
    .timeout(Duration::from_secs(3600))
    .build()?;
```
**Migration**: Use provided migration tool or manual conversion

## Script API Changes
**Old**: `agent:getMetrics()`
**New**: `agent:get_metrics()`
**Migration**: Update all script files
```

---

## 5. Implementation Strategy

### 5.1 Task Execution Order

**Phase 1: Foundation (Week 1)**
1. Task 1.6: Test Organization Foundation
   - Categorize all 175+ test files
   - Set up test execution profiles
   - Establish CI integration

**Phase 2: Core Standardization (Weeks 2-3)**
2. Tasks 1.7-1.12: Workflow-Agent Integration (BREAKING PARADIGM SHIFT)
   - **7.1.7**: Workflow-Agent Trait Integration (8 hours) - Core BaseAgent implementation
   - **7.1.8**: Workflow Factory Standardization (4.5 hours) - WorkflowFactory/WorkflowExecutor
   - **7.1.9**: Workflow Config Builder Standardization (3.5 hours) - Pattern-specific builders  
   - **7.1.10**: Workflow Bridge API Standardization (4 hours) - Bridge method consistency
   - **7.1.11**: Workflow Script API Naming (3 hours) - snake_case conversion
   - **7.1.12**: Factory Method Standardization (2.58 hours) - Non-workflow factory methods
   - **Total**: 25.58 hours of workflow transformation work
   
3. Tasks 1.12-1.24: Bridge API Standardization  
   - Factory method naming
   - Config builder adoption
   - Discovery unification
   - Hook execution fixes

**Phase 3: Documentation (Week 4)**
4. Tasks 2.1-2.3: Rust API Documentation
   - 100% rustdoc coverage
   - Example documentation
   - Cross-references

5. Tasks 4.1-4.4: Documentation Cleanup
   - User guide standardization
   - Technical doc updates
   - Developer guide enhancement

**Phase 4: Examples (Week 5)**
6. Tasks 3.1-3.8: Example Reorganization
   - Audit and categorization
   - Directory restructuring
   - Progressive learning paths
   - Testing framework

**Phase 5: Verification**
7. Task 5.1: Final Test Verification
   - Ensure no uncategorized tests
   - Verify all changes integrated
   - Final quality check

### 5.2 Quality Gates

**Per-Task Quality Checks:**
- Compile without warnings
- All tests passing
- Documentation complete
- Examples working
- No regression in performance
- Migration guide updated

**Phase-Wide Checks:**
- API consistency linter passing
- Test categorization complete
- Documentation builds clean
- Example test suite passing
- Breaking changes documented

---

## 6. Architecture Decisions

### Decision 1: Clean Break Approach
**Context**: Pre-1.0 phase allows breaking changes
**Decision**: No backward compatibility layers
**Rationale**: 
- Cleaner codebase for 1.0
- No technical debt from compatibility
- Clear migration path documented
**Consequences**: Users must migrate, but get cleaner API

### Decision 2: Test-First Foundation
**Context**: 175+ tests mostly uncategorized
**Decision**: Fix test categorization before API changes
**Rationale**:
- Avoid recategorizing new tests
- Establish fast feedback loop
- Enable selective test execution
**Consequences**: Initial time investment, but faster development

### Decision 3: Workflow as Agent (BREAKING PARADIGM SHIFT)
**Context**: Workflows were "batch job" style components separate from agents
**Decision**: Follow Google ADK pattern - workflows implement BaseAgent and become function-like agents
**Rationale**:
- **Unified Type System**: Workflows can be used anywhere agents are expected
- **Recursive Composition**: Workflows can contain other workflows as steps (Google ADK core pattern)
- **Consistent Execution Interface**: All components use AgentInput/AgentOutput (no more workflow-specific types)
- **Factory Standardization**: Workflows created through same patterns as agents and tools
- **Script API Unification**: Same interface in Lua/JavaScript regardless of component type
**Breaking Changes**:
- Workflow execution: `execute() -> WorkflowResult` → `execute(AgentInput, ExecutionContext) -> AgentOutput`
- No backward compatibility for existing workflow APIs
- Input/output adapter layer required for type conversion
- All workflow configs must use builder patterns
**Consequences**: 
- Major implementation effort across 6 tasks (7.1.7-7.1.12)
- Complete re-architecture of workflow execution paradigm
- Enables powerful composition patterns unavailable in current architecture

### Decision 4: Audience-Based Examples
**Context**: 156+ examples mixed together
**Decision**: Reorganize by target audience
**Rationale**:
- Better discoverability
- Clear learning paths
- Reduced confusion
**Consequences**: Migration effort, but improved UX

---

## 7. Risk Analysis

### Risk 1: Scope Creep
**Probability**: High
**Impact**: High
**Mitigation**: 
- Fixed task list in TODO.md
- No new features during phase
- Focus on standardization only

### Risk 2: Breaking User Code
**Probability**: Certain
**Impact**: Medium
**Mitigation**:
- Comprehensive migration guide
- Automated migration tools where possible
- Clear communication in changelog

### Risk 3: Timeline Overrun
**Probability**: Medium
**Impact**: Medium
**Mitigation**:
- Conservative time estimates
- Parallel task execution
- Daily progress tracking

### Risk 4: Incomplete Test Coverage
**Probability**: Low
**Impact**: High
**Mitigation**:
- Test categorization first
- CI enforcement
- Coverage reports

---

## 8. Success Metrics

### Quantitative Metrics
- API consistency: 100% (measured by linter)
- Test categorization: 100% of 175+ files
- Documentation coverage: 100% of public APIs
- Example test pass rate: 100%
- Fast test suite: <35 seconds
- Builder pattern adoption: 100% of configs

### Qualitative Metrics
- Developer feedback on API clarity
- Example usability testing
- Documentation readability scores
- Migration ease feedback
- Code review efficiency

---

## 9. Future Considerations

### Post-1.0 Stability
- Semantic versioning enforcement
- Deprecation cycles established
- API stability guarantees
- Regular but predictable releases

### Extension Points
- Plugin system design
- Third-party tool integration
- Custom provider framework
- Advanced workflow patterns

### Performance Optimization
- API design for zero-copy where possible
- Async-first patterns
- Resource pooling strategies
- Lazy initialization patterns

---

## 10. Tool Development Guidelines (Phase 7 Addition)

### Tool Implementation Standards

#### Naming Conventions
- **Primary data parameter**: Always use `input` (not: text, content, data, expression, query)
- **File paths**: Use `path` for single files, `source_path`/`target_path` for operations
- **Operations**: Always require explicit `operation` parameter for multi-function tools
- **Tool names**: Use hyphens for web tools (e.g., `pdf-processor`, `citation-formatter`), underscores for others

#### Response Format
All tools MUST return responses in this standardized format:
```json
{
  "operation": "operation_name",
  "success": true,
  "result": {...},
  "error": null,
  "metadata": {
    "execution_time_ms": 123,
    "tool_version": "1.0.0"
  }
}
```

#### Security Requirements
- All tools must declare security level (Safe/Restricted/Privileged)
- Input validation is mandatory
- Resource limits must be enforced
- Path traversal prevention for file operations
- Command injection prevention for system operations

#### Performance Targets
- Tool initialization: < 10ms
- Tool execution overhead: < 10ms  
- Use spawn_blocking for synchronous libraries
- Implement timeouts for long operations
- Track resource usage with ResourceTracker

#### Phase 7 Tools Implemented

1. **PDF Processor** (`pdf-processor`)
   - Library: `pdf-extract = "0.9"`
   - Operations: extract_text, get_metadata
   - Key feature: spawn_blocking for sync library
   - Challenge: Complex PDFs require lopdf/pdfium (Phase 21)

2. **Citation Formatter** (`citation-formatter`)
   - Library: `hayagriva = "0.5"`
   - Operations: format_citation, format_bibliography
   - Formats: APA, MLA, Chicago, Harvard
   - Key feature: Complex data validation

3. **Graph Builder** (`graph-builder`)
   - Library: `petgraph = "0.6"`
   - Operations: build_graph, add_node, add_edge, analyze
   - Key feature: Serializable graph structures
   - Support: Directed and undirected graphs

#### Integration with Bridge
The bridge now auto-parses JSON responses from tools for better UX:
```lua
-- Tool responses are automatically parsed
local result = Tool.invoke("pdf-processor", {
    operation = "extract_text",
    input = "/path/to/file.pdf"
})
-- Direct access to parsed fields
print(result.result.text)  -- No need for JSON.parse
```

#### Testing Requirements
- Unit tests with proper categorization
- Integration tests through bridge
- Performance benchmarks
- Security edge case testing
- >90% coverage required

---

This design document provides the complete blueprint for Phase 7 implementation, establishing the foundation for a stable, consistent, and well-documented 1.0 release of rs-llmspell.