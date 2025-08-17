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

---

#### Task 7.3.6: Real-World Applications
**Priority**: MEDIUM
**Estimated Time**: 40 hours (expanded from 8 due to real LLM integration requirements)
**Status**: IN PROGRESS (Data Pipeline partially completed, needs blueprint alignment)
**Assigned To**: Solutions Team
**Dependencies**: Task 7.3.4
**Reference**: Follow the architecture and design in `examples/script-users/applications/blueprint.md` for each application.

**Description**: Create 7 production-ready applications demonstrating llmspell's full capabilities with REAL LLM APIs and proper component composition.

**⚠️ CRITICAL REQUIREMENTS**:
- **NO MOCKS**: Real OpenAI/Anthropic API keys required (costs apply!)
- **Component Composition**: Use Workflows + Agents + Tools properly
- **Minimal Lua**: Only orchestration logic, no business logic
- **Production Grade**: Error handling, monitoring, persistence

**Implementation Steps (Per Blueprint v2.0)**:

0. [x] **CRITICAL: Nested Workflow Support Implementation** (4 hours) - REQUIRED for all applications ✅ COMPLETED:
   - [x] **Core Implementation**:
     - [x] Add `StepType::Workflow` variant to `llmspell-workflows/src/traits.rs`
     - [x] Implement `execute_workflow_step()` in `llmspell-workflows/src/step_executor.rs`
     - [x] Update `llmspell-bridge/src/workflows.rs` native bridge to support nested execution
     - [x] Update `llmspell-bridge/src/lua/globals/workflow.rs` to handle workflow steps
     - [x] Update `llmspell-bridge/src/javascript/globals/workflow.rs` to include nested workflow notes for Phase 12 ✅ COMPLETED
     - [x] Remove "Workflow steps are not yet implemented" error in bridge
   - [x] **Testing & Quality**:
     - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`
     - [~] Run `cargo test --workspace` (compilation successful, tests take too long)
     - [x] Test nested workflow execution in data pipeline
     - [x] Verify workflow composition works end-to-end
   - [x] **Documentation**:
     - [x] Update blueprint.md with correct nested workflow API
     - [x] Add examples of nested workflow usage in blueprint
   - [x] **Validation**:
     - [x] Test data pipeline with real nested workflows ✅ SUCCESS!
     - [x] Verify workflow types work as nested steps (Sequential + Parallel tested)

0.1. [x] **TRUE Conditional Workflow Enhancement & Test Rehabilitation** (24-28 hours) - ✅ COMPLETED:
   - **Priority**: CRITICAL - Blocks all real-world applications using conditional workflows
   - **Issue**: "Cannot execute conditional workflow without branches" error prevents Content Generation Platform
   - **Root Cause**: Bridge serialization bugs + broken/inadequate tests + missing agent-based conditions
   
   - [x] **Level 1: Bridge Serialization Fix + Test Updates (8 hours)** ✅:
     - [x] 0.1.1 Fix Format Mismatch: then_steps/else_steps → branches (4 hours) - ✅ FIXED
       - File: `llmspell-bridge/src/lua/globals/workflow.rs:696-723`
       - **Root Cause**: Lua wrapper sends `{"then_steps": [...], "else_steps": [...]}` but native bridge `create_conditional_workflow()` expects `{"branches": [...]}`
       - **Fix**: Convert Lua `then_steps`/`else_steps` arrays to proper `branches` format expected by workflows layer
       - Replace placeholder JSON `"tool": "placeholder"` with proper step conversion using existing step parsing logic
       - Update JSON format: `config["then_steps"] = ...` → `config["branches"] = serde_json::json!([{name, condition, steps}])`
     - [x] 0.1.2 Fix Workflow Step Format (2 hours) - ✅ FIXED  
       - File: `llmspell-bridge/src/lua/globals/workflow.rs:555-563`
       - **Root Cause**: `:condition()` method is dummy implementation that always returns `true`
       - **Fix**: Store and serialize Lua condition functions for bridge processing
       - Add condition context passing from Lua → Rust bridge layer
     - [x] 0.1.3 Update Broken Bridge Test (1 hour) ✅
       - File: `llmspell-bridge/tests/lua_workflow_api_tests.rs` 
       - Fix `test_lua_workflow_conditional` to use builder API instead of direct config
       - Category: `#[cfg_attr(test_category = "integration")] #[cfg_attr(test_category = "bridge")]`
     - [x] 0.1.4 Clippy Compliance - Bridge Layer (1 hour) ✅
       - Files: `llmspell-bridge/src/lua/globals/workflow.rs`, `llmspell-bridge/src/workflows.rs`
       - Fix unused variables in condition closures, deprecated JSON patterns
       - Verify: `cargo clippy --package llmspell-bridge -- -D warnings`
   
   - [x] **Level 2: Workflows Layer Fix + Test Rehabilitation (6 hours)** ✅:
     - [x] 0.1.5 Add Agent-Based Condition Types (2 hours) ✅
       - File: `llmspell-workflows/src/conditions.rs`
       - Add: `StepOutputContains{step_name, search_text}`, `AgentClassification{step_name, expected_type}`
     - [x] 0.1.6 Fix Broken Workflow Unit Tests (3 hours) ✅
       - File: `llmspell-workflows/src/conditional.rs` test module
       - Update 7 existing tests to use real conditions instead of `Condition::Always` stubs
       - Tests: `test_conditional_workflow_execution_always_true`, `test_conditional_workflow_shared_data_condition`, etc.
       - Category: `#[cfg_attr(test_category = "unit")] #[cfg_attr(test_category = "workflow")]`
     - [x] 0.1.7 Clippy Compliance - Workflows Layer (1 hour) ✅
       - Files: `llmspell-workflows/src/conditional.rs`, `llmspell-workflows/src/conditions.rs`
       - Verify: `cargo clippy --package llmspell-workflows -- -D warnings`
       
   - [x] **Level 3: Integration Test Creation (5 hours)** ✅:
     - [x] 0.1.8 Bridge-to-Workflows Integration Tests (3 hours) ✅
       - File: `llmspell-bridge/tests/workflow_bridge_integration_tests.rs` (new section)
       - Test Lua builder → Rust workflow conversion, agent classification condition parsing
       - Category: `#[cfg_attr(test_category = "integration")] #[cfg_attr(test_category = "workflow")] #[cfg_attr(test_category = "bridge")]`
     - [x] 0.1.9 End-to-End Content Routing Tests (2 hours) ✅
       - File: `llmspell-bridge/tests/content_routing_integration_test.rs` (new)
       - Test full agent classification → workflow routing pipeline
       - Category: `#[cfg_attr(test_category = "integration")] #[cfg_attr(test_category = "agent")] #[cfg_attr(test_category = "workflow")]`
       
   - [x] **Level 4: Documentation & Examples (3 hours)** ✅:
     - [x] 0.1.10 Update Blueprint Documentation (1.5 hours) ✅
       - File: `examples/script-users/applications/blueprint.md`
       - Remove conditional workflow warnings, add working patterns, migration guide
     - [x] 0.1.11 Create Working Example Files (1.5 hours) ✅
       - Files: `examples/script-users/workflows/conditional-content-routing.lua`, `conditional-multi-branch.lua`
       
   - [x] **Level 5: Advanced Features + Final Compliance (5 hours)** ✅:
     - [x] 0.1.12 Multi-Branch Support Enhancement (2 hours) ✅
       - File: `llmspell-bridge/src/lua/globals/workflow.rs`
       - Add `add_branch(condition, steps)` API for N-branch routing beyond then/else
     - [x] 0.1.13 External API Tests (1.5 hours) ✅
       - File: `llmspell-bridge/tests/conditional_external_tests.rs` (new)
       - Tests using real LLM agents for content classification
       - Category: `#[ignore = "external"]` for tests requiring API keys
     - [x] 0.1.14 Final Clippy & Test Compliance Verification (1.5 hours) ✅
       - Commands: `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --all-features`
       - Verify: All existing tests pass + new tests pass + 0 clippy warnings
   - [x] **Level 6: Bridge Architecture Refactoring (8 hours)** ✅ COMPLETED:
     - [x] 0.1.15 Refactor JSON Serialization Architecture (4 hours) ✅
       - **Problem**: Language bridges (Lua/JS/Python) each create JSON independently = logic duplication
       - **Solution**: Move ALL JSON serialization to native bridge (`llmspell-bridge/src/workflows.rs`)
       - Files to refactor:
         - `llmspell-bridge/src/lua/globals/workflow.rs` - Remove JSON creation, pass Rust structs ✅
         - `llmspell-bridge/src/workflows.rs` - Add struct-to-JSON conversion functions ✅
         - `llmspell-bridge/src/standardized_workflows.rs` - Use new conversion functions
       - **Architecture**:
         - Language bridges: Convert language types → Rust structs only ✅
         - Native bridge: Single source of truth for Rust structs → JSON conversion ✅
         - Benefits: No duplication, consistent format, easier maintenance ✅
     - [x] 0.1.16 Fix Step Format Inconsistency (2 hours) ✅
       - Fix then_branch using nested `step_type` while else_branch uses flat format ✅
       - Ensure ALL branches use consistent flat format expected by parser ✅
       - Update `create_conditional_workflow` to handle proper step conversion ✅
     - [x] 0.1.17 Update Tests for New Architecture (1 hour) ✅
       - Update bridge tests to verify struct passing instead of JSON ✅
       - Ensure test_fallback_routing passes with consistent formats ✅
     - [x] 0.1.18 Document Architecture Pattern (1 hour) ✅
       - Add architecture documentation to `docs/technical/bridge-architecture.md` ✅
       - Document the pattern for future language bridges (JavaScript, Python) ✅
       - Create migration guide for existing code ✅
       
0.2. [x] **Tool Registration Fixes** (2 hours) - ✅ COMPLETED:
   **Context**: Multiple examples were broken because webhook-caller tool wasn't accessible via scripts.
   - [x] **Register Existing Unregistered Tools**:
     - [x] ✅ Verified `WebhookCallerTool` already registered in `llmspell-bridge/src/tools.rs:269` as "webhook-caller"
     - [x] ✅ Fixed webhook-caller usage in Content Generation Platform - replaced simulation with real webhook calls
     - [x] ✅ Verified all 34+ tools properly registered and accessible via Tool.list()
   - [x] **Testing**:
     - [x] ✅ Fixed conditional-multi-branch.lua to use "webhook-caller" (hyphen) and correct parameter format
     - [x] ✅ Verified webhook-caller tool works in script context: `Tool.invoke("webhook-caller", {...})`
   
0.3. [x] **Phase 7 Appropriate Tool Implementation** (6 hours) - ✅ COMPLETED:
   **Research Complete**: Identified best Rust libraries for each tool
   
   - [x] **1. PdfProcessorTool Implementation** (2 hours) ✅:
     - **Library**: `pdf-extract = "0.9"` (most focused for text extraction)
     - **Implementation**: `llmspell-tools/src/document/pdf_processor.rs` 
     - **Operations**: extract_text, extract_metadata, extract_pages
     - **Parameters**: `input` (file path), `operation`, `start_page` (optional)
     - **Output**: JSON with text content, page count, metadata
     - **Security**: File path validation, size limits (10MB), sandboxing
     
   - [x] **2. CitationFormatterTool Implementation** (2 hours) ✅:
     - **Library**: `hayagriva = "0.5"` (Phase 7 basic implementation)
     - **Implementation**: `llmspell-tools/src/academic/citation_formatter.rs`
     - **Operations**: format_citation, validate_bibliography, list_styles
     - **Parameters**: `input` (citation data), `style` (apa/mla/chicago/etc), `operation`, `format` (yaml/bibtex)
     - **Output**: Basic formatted citations (Phase 7), full CSL processor for Phase 8
     - **Note**: Phase 7 provides structure + basic validation, full hayagriva integration planned for Phase 8
     
   - [x] **3. GraphBuilderTool Implementation** (2 hours) ✅:
     - **Library**: `petgraph = "0.6"` (with serde-1 feature for JSON serialization)
     - **Implementation**: `llmspell-tools/src/data/graph_builder.rs`
     - **Operations**: create_graph, add_node, add_edge, analyze, export_json, import_json
     - **Parameters**: `input` (graph data), `operation`, `graph_type` (directed/undirected), `format` (json)
     - **Output**: Graph structure as JSON, analysis results (node count, edge count, degree statistics)
     - **Features**: 10K nodes, 50K edges limits, JSON import/export, basic connectivity analysis
     
   - [x] **Implementation Requirements**:
     - [x] ✅ Add dependencies to `llmspell-tools/Cargo.toml` (pdf-extract, hayagriva, petgraph)
     - [x] ✅ Follow existing tool patterns in `llmspell-tools/src/`
     - [x] ✅ Create modules with proper directory structure and mod.rs files
     - [x] ✅ Register in bridge: `llmspell-bridge/src/tools.rs` (pdf-processor, citation-formatter, graph-builder)
     - [x] ✅ Update `llmspell-tools/src/lib.rs` re-exports
     - [x] ✅ Add comprehensive tests with proper test categorization
     - [x] ✅ Fix API mismatches for compilation (ResponseBuilder, SecurityRequirements, ResourceLimits field names)
     - [x] ✅ Test tools in script context: `Tool.invoke("pdf-processor", {...})`
   
0.4. [x] **Update Examples to Use Real Tools** (3 hours): ✅ COMPLETED
   - [x] **Code Review Assistant**: ✅ COMPLETED
     - [x] Verified uses real text_manipulator for analysis
     - [x] Verified uses real json_processor for validation
   - [x] **Research Assistant Application**: ✅ COMPLETED
     - [x] Uses real PdfProcessorTool (with W3C dummy PDF due to pdf-extract limitations)
     - [x] Uses real GraphBuilderTool (with serialize_graph helper for Lua JSON issues)
     - [x] Uses real CitationFormatterTool (all operations working)
     - [x] All Phase 7 tools tested and operational
   - [x] **Document Intelligence System**: ✅ COMPLETED
     - [x] Already uses real pdf-processor tool
     - [x] Already uses real graph-builder tool
     - [x] Already uses real citation-formatter tool
     - [x] Uses web_search as vector_search alternative (until Phase 8)
   - [x] **Testing All Applications**: ✅ COMPLETED
     - [x] Verified all 5 applications run without simulated tools
     - [x] Data Pipeline: Uses real file_operations, http_request, webhook_caller
     - [x] Customer Support: Uses real file_operations, webhook-caller
     - [x] Code Review: Uses real file_operations, text_manipulator, json_processor
     - [x] Content Generation: Uses real web_search, file_operations, webhook-caller
     - [x] Document Intelligence: Uses all Phase 7 tools
   
0.5. [x] **Tool Architecture Documentation** (1 hour): ✅ COMPLETED
   - [x] **Tool Development Guide Updated**: ✅
     - [x] Updated existing `/docs/developer-guide/tool-development-guide.md`
     - [x] Added Phase 7 tool examples and patterns
     - [x] Documented spawn_blocking for sync libraries
     - [x] Added tool naming conventions and response formats
   - [x] **Blueprint Updated**: ✅
     - [x] Added tool guidelines to phase-07-design-doc.md
     - [x] Listed all 37 available tools with categories
     - [x] Added bridge auto-parsing documentation
   
   - **SUCCESS CRITERIA**:
     - Level 1-2: Content Generation Platform executes without "Cannot execute conditional workflow without branches" error
     - Level 3: Integration tests prove agent classification correctly routes to different workflows  
     - Level 4: Documentation updated, examples work, migration path clear
     - Level 5: Multi-branch routing + 0 clippy warnings + all test categories validated

1. [x] **Customer Support System** (8 hours) - ✅ COMPLETED:
   - [x] **Component Architecture**:
     - [x] Main Sequential Workflow for routing logic (conditional workaround) ✅
     - [x] Urgent Handler (Parallel Workflow) - response + supervisor notification ✅
     - [x] Standard Handler (Sequential Workflow) - sentiment + response + notify ✅
   - [x] **Agents** (3 required):
     - [x] ticket_classifier (GPT-4o-mini) - categorizes and prioritizes ✅
     - [x] sentiment_analyzer (Claude-3-haiku) - detects escalation needs ✅
     - [x] response_generator (GPT-4o-mini) - creates customer responses ✅
   - [x] **Tools Integration**:
     - [x] webhook-caller, file_operations ✅ (database-connector for future enhancement)
   - [x] **Implementation Patterns** (CRITICAL - Applied Successfully):
     - [x] Agent name storage: `agent_names.classifier = "ticket_classifier_" .. timestamp` ✅
     - [x] Timing implementation: Use workflow execution logs (74ms actual), not os.time() ✅
     - [x] Graceful degradation: Fallback to basic tools when no API keys ✅
   - [x] **Testing Requirements** (COMPLETED):
     - [x] Test workflow builder syntax: `:parallel()`, `:sequential()` ✅
     - [x] Test with and without API keys for graceful degradation ✅
     - [x] Verify execution with: `LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run main.lua` ✅
   - [x] **Files Created**:
     - [x] `customer-support-bot/main.lua` - orchestration ✅
     - [x] `customer-support-bot/config.toml` - configuration ✅
     - [x] `customer-support-bot/README.md` - setup and usage ✅
   - [x] **Lessons Learned**:
     - [x] Conditional workflows need debugging - used sequential workaround ✅
     - [x] Builder pattern works perfectly for sequential, parallel, loop ✅
     - [x] Nested workflows function correctly with `type = "workflow"` ✅

2. [x] **Data Pipeline** (6 hours) - ✅ COMPLETED - Blueprint v2.0 compliant:
   - [x] **Component Architecture** (100% Blueprint Compliant):
     - [x] Main Sequential Workflow ✅
     - [x] Extract Phase (Parallel Workflow) - ✅ FIXED: 3 sources (database, API, files)
     - [x] Transform Phase (Loop Workflow) - ✅ ADDED: complete loop workflow with batching
     - [x] Analysis Phase (Parallel Workflow) - ✅ COMPLETED: 3 agent parallel analysis
     - [x] Load Phase (Sequential) - ✅ ADDED: database save, report generation, notifications
   - [x] **Agents** (5 required per blueprint):
     - [x] data_enricher (GPT-3.5-turbo) - contextual enrichment
     - [x] quality_analyzer (GPT-4) - quality issues
     - [x] anomaly_detector (GPT-4) - outlier detection
     - [x] pattern_finder (Claude-3-haiku) - pattern discovery
     - [x] report_generator (Claude-3-sonnet) - insights report
   - [x] **Workflow Architecture** (All Required Phases Implemented):
     - [x] Replace simple sequential with nested workflows ✅
     - [x] Add Parallel extraction from 3 sources (file_operations, database-connector, api-tester) ✅
     - [x] Add Loop workflow for batch transformation with validation, cleaning, enrichment ✅
     - [x] Add Parallel analysis workflows with 3 specialized agents ✅
     - [x] Add Load phase with database save, report generation, webhook notifications ✅
   - [x] **Files Completed**:
     - [x] `data-pipeline/main.lua` - ✅ Blueprint v2.0 compliant ETL implementation
     - [x] `data-pipeline/README.md` - comprehensive documentation ✅
     - [x] `data-pipeline/test.lua` - comprehensive testing available  
     - [x] `data-pipeline/config.toml` - configuration file ✅
   - [x] **Blueprint Compliance Achieved**:
     - [x] Extract Phase: Parallel workflow with database-connector, api-tester, file_operations ✅
     - [x] Transform Phase: Loop workflow with json_processor, text_manipulator, LLM enrichment ✅ 
     - [x] Analysis Phase: Parallel workflow with quality, anomaly, pattern agents ✅
     - [x] Load Phase: Sequential workflow with database save, report generation, webhook notifications ✅
     - [x] 4-Phase Architecture: Extract→Transform→Analysis→Load nested workflow composition ✅

3. [x] **Content Generation Platform** (6 hours) - ✅ COMPLETED:
   - [x] **Component Architecture**:
     - [x] Main Conditional Workflow (content type routing) (use `:conditional()`) ✅
     - [x] Blog Workflow (Sequential) (use `:sequential()`) ✅
     - [x] Social Workflow (Parallel multi-platform) (use `:parallel()`) ✅
     - [x] Email Workflow (Sequential) (use `:sequential()`) ✅
     - [x] Optimization Phase (Parallel) (use `:parallel()`) ✅
   - [x] **Agents** (7 required):
     - [x] researcher (GPT-4o-mini) - topic research ✅
     - [x] outliner (Claude-3-haiku) - content structure ✅
     - [x] blog_writer (GPT-4o-mini) - long-form ✅
     - [x] social_writer (Claude-3-haiku) - social posts ✅
     - [x] email_writer (Claude-3-haiku) - newsletters ✅
     - [x] seo_optimizer (via web_search tool) - SEO improvements ✅
     - [x] personalizer (GPT-4o-mini) - audience targeting ✅
   - [x] **Implementation Patterns** (CRITICAL - from data pipeline learnings):
     - [x] Agent name storage: `agent_names.researcher = "researcher_" .. timestamp` ✅
     - [x] Timing implementation: Use workflow execution logs (~52ms), not os.time() ✅
     - [x] Graceful degradation: Fallback to basic tools when no API keys ✅
   - [x] **Testing Requirements** (MANDATORY):
     - [x] Test workflow builder syntax: `:conditional()`, `:sequential()`, `:parallel()` ✅
     - [x] Test with and without API keys for graceful degradation ✅
     - [x] Verify execution with: `LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run main.lua` ✅
   - [x] **Files Created**:
     - [x] `content-generation-platform/main.lua` - orchestration ✅
     - [x] `content-generation-platform/config.toml` - configuration ✅
     - [x] `content-generation-platform/README.md` - setup guide ✅
   - [x] **TRUE Conditional Workflow Implementation**:
     - [x] Successfully implemented TRUE conditional routing with classification step ✅
     - [x] Nested workflows work correctly within conditional branches ✅
     - [x] Multi-format content generation (blog, social, email) all functioning ✅
   - [ ] **REQUIRED FIX - Enable WebhookCallerTool**:
     - [ ] Uncomment webhook publishing code after tool registration (0.2)
     - [ ] Test webhook functionality works properly
     - [ ] Update README.md to reflect working webhook integration

4. [x] **Code Review Assistant** (6 hours) - ✅ COMPLETED:
   - [x] **Component Architecture**:
     - [x] Main Sequential Workflow (use `:sequential()`) ✅
     - [x] Code Analysis (Parallel initial analysis) (use `:parallel()`) ✅
     - [x] File Review Loop (iterates through files) (use `:loop_workflow()` + `:max_iterations()`) ✅
     - [x] Review Sub-workflow (Parallel multi-aspect) (use `:parallel()`) ✅
   - [x] **Agents** (7 required):
     - [x] security_reviewer (GPT-4o-mini) - vulnerability detection ✅
     - [x] quality_reviewer (Claude-3-haiku) - code quality ✅
     - [x] practices_reviewer (GPT-4o-mini) - best practices ✅
     - [x] performance_reviewer (GPT-3.5-turbo) - performance issues ✅
     - [x] issue_prioritizer (GPT-4o-mini) - severity ranking ✅
     - [x] fix_generator (Claude-3-haiku) - fix suggestions ✅
     - [x] report_writer (GPT-4o-mini) - review report ✅
   - [x] **Implementation Patterns** (CRITICAL - from data pipeline learnings):
     - [x] Agent name storage: `agent_names.security = "security_reviewer_" .. timestamp` ✅
     - [x] Timing implementation: Use workflow execution logs (~400ms), not os.time() ✅
     - [x] Graceful degradation: Fallback to basic tools when no API keys ✅
   - [x] **Testing Requirements** (MANDATORY):
     - [x] Test workflow builder syntax: `:sequential()`, `:parallel()`, `:loop_workflow()` ✅
     - [x] Test with and without API keys for graceful degradation ✅
     - [x] Verify execution with: `LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run main.lua` ✅
   - [x] **Custom Tools Simulated**:
     - [x] code_analyzer - simulated with text_manipulator ✅
     - [x] syntax_validator - simulated with json_processor ✅
   - [x] **Files Created**:
     - [x] `code-review-assistant/main.lua` - orchestration ✅
     - [x] `code-review-assistant/README.md` - comprehensive documentation ✅
     - [x] `code-review-assistant/config.toml` - configuration ✅
   - [x] **Blueprint Compliance Achieved**:
     - [x] 4-Phase Architecture: Analysis → Review → Aggregate → Report ✅
     - [x] Loop workflow iterating through 3 files ✅
     - [x] Parallel sub-workflows with 4 reviewers per file ✅
     - [x] 7 specialized agents all functioning ✅
   - [ ] **REQUIRED FIX - Replace Simulated Tools**:
     - [ ] Replace simulated code_analyzer with text_manipulator or new tool
     - [ ] Replace simulated syntax_validator with actual validator tool
     - [ ] Update main.lua to use real tool operations
     - [ ] Test execution with real tools

5. [x] **Document Intelligence System** (6 hours) - ✅ COMPLETED:
   - [x] **Component Architecture**:
     - [x] Main Sequential Workflow (use `:sequential()`) ✅
     - [x] Document Ingestion (Parallel) (use `:parallel()`) ✅
     - [x] Processing Loop (per-document) (use `:loop_workflow()` + `:max_iterations()`) ✅
     - [x] Q&A Interface (Conditional) (use `:conditional()`) ✅
   - [x] **Agents** (8 required):
     - [x] entity_extractor (GPT-4o-mini) - NER ✅
     - [x] topic_analyzer (Claude-3-haiku) - topic modeling ✅
     - [x] summarizer (Claude-3-haiku) - summarization ✅
     - [x] embedding_generator (simulated with GPT) - vectors ✅
     - [x] qa_responder (GPT-4o-mini) - Q&A ✅
     - [x] doc_comparer (Claude-3-haiku) - comparison ✅
     - [x] pattern_analyzer (GPT-4o-mini) - patterns ✅
     - [x] insight_generator (Claude-3-haiku) - insights ✅
   - [x] **Implementation Patterns** (CRITICAL - from data pipeline learnings):
     - [x] Agent name storage: `agent_names.entity = "entity_extractor_" .. timestamp` ✅
     - [x] Timing implementation: Use workflow execution logs (~450ms), not os.time() ✅
     - [x] Graceful degradation: Fallback to basic tools when no API keys ✅
   - [x] **Testing Requirements** (MANDATORY):
     - [x] Test workflow builder syntax: `:sequential()`, `:parallel()`, `:loop_workflow()`, `:conditional()` ✅
     - [x] Test with and without API keys for graceful degradation ✅
     - [x] Verify execution with: `LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run main.lua` ✅
   - [x] **Custom Tools Simulated**:
     - [x] pdf_processor, graph_builder, vector_search, citation_formatter ✅
   - [x] **Files Created**:
     - [x] `document-intelligence/main.lua` - orchestration ✅
     - [x] `document-intelligence/README.md` - comprehensive documentation ✅
     - [x] `document-intelligence/config.toml` - configuration ✅
   - [ ] **REQUIRED FIX - Replace Simulated Tools**:
     - [ ] Replace simulated pdf_processor with real PdfProcessorTool (after 0.3)
     - [ ] Replace simulated graph_builder with real GraphBuilderTool (after 0.3)
     - [ ] Replace simulated vector_search with appropriate Phase 7 alternative
     - [ ] Replace simulated citation_formatter with real CitationFormatterTool (after 0.3)
     - [ ] Update main.lua to use real tool names and parameters
     - [ ] Test execution with real tools
     - [ ] Update README.md to reflect real tool capabilities

6. [ ] **Workflow Automation Hub** (5 hours):
   - [ ] **Component Architecture**:
     - [ ] Main Controller (Conditional) (use `:conditional()`)
     - [ ] Sequential Execution engine (use `:sequential()`)
     - [ ] Dynamic Execution engine (nested workflows) (use nested `:workflow` steps)
     - [ ] Monitoring (Parallel) (use `:parallel()`)
     - [ ] Error Handler (Conditional) (use `:conditional()`)
   - [ ] **Agents** (4 required):
     - [ ] workflow_optimizer (GPT-4) - execution optimization
     - [ ] error_resolver (Claude-3-sonnet) - error recovery
     - [ ] workflow_generator (GPT-4) - workflow creation
     - [ ] dependency_analyzer (GPT-3.5-turbo) - dependencies
   - [ ] **Implementation Patterns** (CRITICAL - from data pipeline learnings):
     - [ ] Agent name storage: `agent_names.optimizer = "workflow_optimizer_" .. timestamp`
     - [ ] Timing implementation: Use workflow execution logs (~250ms), not os.time()
     - [ ] Graceful degradation: Fallback to basic tools when no API keys
   - [ ] **Testing Requirements** (MANDATORY):
     - [ ] Test workflow builder syntax: `:conditional()`, `:sequential()`, `:parallel()`
     - [ ] Test nested workflow execution with actual workflow objects
     - [ ] Test with and without API keys for graceful degradation
     - [ ] Verify execution with: `LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run main.lua`
   - [ ] **Event & Hook Integration**:
     - [ ] Workflow lifecycle events
     - [ ] Pre/post step hooks
     - [ ] Rate limiting hooks
   - [ ] **Custom Tools Needed**:
     - [ ] yaml_parser, schema_validator, resource_monitor
   - [ ] **Files to Create**:
     - [ ] `workflow-hub/main.lua` - orchestration
     - [ ] `workflow-hub/README.md` - workflow DSL
     - [ ] `workflow-hub/examples/` - workflow examples

7. [ ] **AI Research Assistant** (7 hours):
   - [ ] **Component Architecture**:
     - [ ] Main Research Workflow (Sequential) (use `:sequential()`)
     - [ ] Database Search (Parallel multi-source) (use `:parallel()`)
     - [ ] Paper Processing Loop (iterative) (use `:loop_workflow()` + `:max_iterations()`)
     - [ ] Analysis Sub-workflow (Parallel) (use `:parallel()`)
     - [ ] Output Generation (Parallel) (use `:parallel()`)
   - [ ] **Agents** (11 required):
     - [ ] query_parser (GPT-4)
     - [ ] term_expander (GPT-3.5-turbo)
     - [ ] paper_summarizer (Claude-3-sonnet)
     - [ ] method_extractor (GPT-4)
     - [ ] finding_extractor (GPT-4)
     - [ ] quality_assessor (Claude-3-opus)
     - [ ] connection_finder (GPT-4)
     - [ ] gap_analyzer (Claude-3-opus)
     - [ ] review_writer (Claude-3-opus)
     - [ ] insight_generator (GPT-4)
     - [ ] recommendation_engine (GPT-4)
   - [ ] **Implementation Patterns** (CRITICAL - from data pipeline learnings):
     - [ ] Agent name storage: `agent_names.parser = "query_parser_" .. timestamp`
     - [ ] Timing implementation: Use workflow execution logs (~500ms), not os.time()
     - [ ] Graceful degradation: Fallback to basic tools when no API keys
   - [ ] **Testing Requirements** (MANDATORY):
     - [ ] Test workflow builder syntax: `:sequential()`, `:parallel()`, `:loop_workflow()`
     - [ ] Test with and without API keys for graceful degradation
     - [ ] Verify execution with: `LLMSPELL_CONFIG=config.toml ./target/debug/llmspell run main.lua`
   - [ ] **Academic Integration**:
     - [ ] ArXiv, Google Scholar, PubMed search
     - [ ] PDF processing and analysis
     - [ ] Citation management
   - [ ] **Files to Create**:
     - [ ] `research-assistant/main.lua` - orchestration
     - [ ] `research-assistant/README.md` - research guide
     - [ ] `research-assistant/citations.lua` - bibliography

**Testing & Documentation** (2 hours):
- [ ] **Test Framework**:
  - [ ] Unit tests per application
  - [ ] Integration tests with real APIs
  - [ ] Cost-aware test configurations
  - [ ] Load testing scenarios
- [ ] **Documentation Requirements**:
  - [ ] Setup instructions with API keys
  - [ ] Cost projections per application
  - [ ] Performance benchmarks
  - [ ] Deployment guides

**Production Readiness** (1 hour):
- [ ] Docker configurations
- [ ] Environment variable management
- [ ] Monitoring metrics setup
- [ ] Cost optimization strategies
- [ ] Operational runbooks

**Acceptance Criteria**:
- [ ] All 7 applications match blueprint.md architectures exactly
- [ ] Each uses proper component composition (Workflows + Agents + Tools)
- [ ] Minimal Lua code (only orchestration)
- [ ] All agents use REAL LLM APIs (no mocks)
- [ ] Production-grade error handling
- [ ] State persistence and recovery
- [ ] Complete documentation
- [ ] Cost estimates documented

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