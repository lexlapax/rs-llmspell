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
#### Task 7.3.4: Getting Started Experience
#### Task 7.3.5: Cookbook and Patterns
#### Task 7.3.6: Real-World Applications
#### Task 7.3.7: Configuration Architecture Redesign and Tool Security Enhancement
#### Task 7.3.8: State-Based Workflow Output Implementation (Google ADK Pattern)
#### Task 7.3.9: Mandatory Sandbox Architecture (Security Critical) ✅ COMPLETED
#### Task 7.3.10: WebApp Creator Complete Rebuild (Production-Ready)
#### Task 7.3.11: Performance Metrics Documentation ✅ COMPLETED (2025-08-22)

---

#### Task 7.3.12: Universal → Professional Application Progression Implementation
**Priority**: HIGH
**Estimated Time**: 13.5 days (full implementation)
**Status**: IN_PROGRESS
**Assigned To**: Core Team
**Dependencies**: Phase 7 Infrastructure (complete)

**Description**: Transform existing applications into a universal → professional progression using renaming strategy and complexity adjustment to demonstrate Phase 7 infrastructure through natural problem evolution.

**Current State Analysis**:
- **Working Applications**: 2/7 (code-review-assistant ✅, webapp-creator ✅)
- **Applications Requiring Transformation**: 5/7 (rename + adjust complexity)
- **Phase 7 Infrastructure Available**: All crates ready for progressive integration
- **Architecture Strategy**: Renaming existing apps (no backward compatibility constraints)

**Architecture Overview**:
- **Progression Model**: Universal → Professional (2 → 20 agents across 6 complexity layers)
- **Transformation Strategy**: Rename existing applications + adjust complexity (no backward compatibility)
- **Crate Integration**: Incremental Phase 7 infrastructure introduction per layer
- **Validation Approach**: Universal appeal testing (Layer 1-2) → professional adoption (Layer 5-6)

**Implementation Phases**:

##### 7.3.12.1: Foundation Reset** (0.5 days) ✅ COMPLETED
- [x] **Architecture Documentation**:
  - [x] Map existing app capabilities to target transformations
  - [x] Define agent reduction/expansion strategies per app
  - [x] Create incremental crate integration plan
  - [x] Design validation framework for universal appeal

##### 7.3.12.2: Universal Layer Implementation** (3 days)

**01. file-organizer/** (Universal: "My files are a complete mess")
- **SOURCE**: document-intelligence/ → RENAME + REDUCE 8→3 agents
- **Agents**: file_scanner, category_classifier, organization_suggester
- **Workflows**: Simple sequential (scan → classify → organize)  
- **Crates**: llmspell-core, llmspell-agents, basic llmspell-bridge
- **Tools**: file_operations, text_manipulator only
- **Universal Problem**: File chaos (every computer user experiences this)

**Implementation Tasks**:
- [ ] **file-organizer/ Transformation**:
  - [ ] Rename document-intelligence/ → file-organizer/
  - [ ] **AGENT MERGES**: 
    - [ ] `text_extractor` + `metadata_analyzer` → `file_scanner` (content scanning + metadata extraction)
    - [ ] `content_classifier` + `quality_assessor` → `category_classifier` (file categorization)
    - [ ] `insight_generator` → `organization_suggester` (folder/structure suggestions)
    - [ ] **REMOVE**: `anomaly_detector`, `pattern_finder`, `relationship_mapper` (too complex)
  - [ ] **WORKFLOW SIMPLIFICATION**: 8 nested workflows → 1 simple sequential (scan → classify → organize)
  - [ ] **CRITICAL - REMOVE STATE**: Strip all State.get() patterns (too complex for universal users)
  - [ ] **CRATE REDUCTION**: Strip to core only (`llmspell-core`, `llmspell-agents`, `llmspell-bridge`)
  - [ ] **TOOL REDUCTION**: Keep `file_operations` only, remove document processing tools
  - [ ] **UNIVERSAL TESTING**: Apply validation framework - target <5 min file organization

**02. research-collector/** (Universal: "I need to research this thoroughly")  
- **SOURCE**: research-assistant/ → RENAME + REDUCE 11→2 agents
- **Agents**: search_agent, synthesis_agent
- **Workflows**: Parallel search + sequential synthesis
- **Crates**: + llmspell-tools (web_search), basic parallel workflows
- **Tools**: web_search, text_manipulator, basic http_request
- **Universal Problem**: Information gathering (everyone researches purchases, health, travel)

**Implementation Tasks**:
- [ ] **research-collector/ Transformation**:
  - [ ] Rename research-assistant/ → research-collector/
  - [ ] **AGENT MERGES**:
    - [ ] `academic_searcher` + `web_searcher` + `search_orchestrator` → `search_agent` (unified search)
    - [ ] `document_analyzer` + `synthesis_agent` + `quality_reviewer` + `fact_checker` + `bias_detector` + `recommendation_engine` + `report_generator` → `synthesis_agent` (simple synthesis)
    - [ ] **REMOVE**: `citation_formatter` (academic complexity)
  - [ ] **WORKFLOW SIMPLIFICATION**: 6 sequential workflows → 2 workflows (parallel search + sequential synthesis)
  - [ ] **NO STATE PERSISTENCE**: Keep minimal - immediate results only
  - [ ] **CRATE ADDITION**: + `llmspell-tools` (web_search integration)
  - [ ] **TOOL INTEGRATION**: `web_search`, `file_operations` for basic result storage
  - [ ] **UNIVERSAL TESTING**: Apply validation framework - vacation planning, health research, product comparison scenarios

##### 7.3.12.3: Power User Transition** (2 days)

**03. content-creator/** (Power User: "Creating content takes forever")
- **SOURCE**: content-generation-platform/ → RENAME + REDUCE 7→4 agents  
- **Agents**: content_planner, content_writer, content_editor, content_formatter
- **Workflows**: Conditional logic (planning → writing → quality-based editing → formatting)
- **Crates**: + llmspell-workflows (conditional), basic state management
- **Tools**: text_manipulator, template_engine, json_processor
- **Power User Problem**: Content creation productivity (bloggers, creators, professionals)

**Implementation Tasks**:
- [ ] **content-creator/ Transformation**:
  - [ ] Rename content-generation-platform/ → content-creator/
  - [ ] **AGENT CHANGES**:
    - [ ] Keep: `content_strategist` → `content_planner`, `content_writer` → `content_writer`, `editor_agent` → `content_editor`
    - [ ] Combine: `quality_assurance` functionality into `content_formatter` (final formatting + basic QA)
    - [ ] **REMOVE**: `seo_optimizer` + `social_media_formatter` (platform complexity → individual productivity focus)
  - [ ] **WORKFLOW ENHANCEMENT**: Add conditional logic (Plan → Write → **IF** quality check fails **THEN** re-edit **ELSE** format)
  - [ ] **CRATE INTRODUCTION**: `llmspell-workflows` (conditional workflows)
  - [ ] **STATE INTRODUCTION**: Basic state management for quality control decisions
  - [ ] **TOOL ADDITION**: + `template_engine`, `json_processor` for content formatting
  - [ ] **POWER USER TESTING**: Content creators see productivity gains with quality control

##### 7.3.12.4: Business Integration** (2 days)

**04. communication-manager/** (Business: "Managing business communications is overwhelming")
- **SOURCE**: customer-support-bot/ → RENAME + EXPAND 3→5 agents
- **Agents**: comm_classifier, sentiment_analyzer, response_generator, schedule_coordinator, tracking_agent
- **Workflows**: Nested workflows, state management, session persistence
- **Crates**: + llmspell-state-persistence, llmspell-sessions, llmspell-events (basic)
- **Tools**: webhook_caller, email_sender, file_operations, text_manipulator
- **Business Problem**: Communication scaling (small business owners, freelancers, consultants)

**Implementation Tasks**:  
- [ ] **communication-manager/ Transformation**:
  - [ ] Rename customer-support-bot/ → communication-manager/
  - [ ] **AGENT EXPANSION** (UNUSUAL - 3→5 agents):
    - [ ] Keep: `ticket_classifier` → `comm_classifier`, `sentiment_analyzer` → `sentiment_analyzer`, `response_generator` → `response_generator`
    - [ ] **ADD**: `schedule_coordinator` (meeting/follow-up scheduling), `tracking_agent` (communication thread tracking)
  - [ ] **SCOPE BROADENING**: From support tickets → ALL business communications
  - [ ] **WORKFLOW ARCHITECTURE**: Nested workflows (Communication classification → sentiment analysis → response generation → scheduling → tracking)
  - [ ] **CRATE ADDITIONS**: `llmspell-state-persistence` (conversation threads), `llmspell-sessions` (client interaction history), `llmspell-events` (basic notifications)
  - [ ] **TOOL INTEGRATION**: + `email_sender`, `webhook_caller` for external integration
  - [ ] **STATE USAGE**: Persistent thread tracking, client interaction history
  - [ ] **BUSINESS TESTING**: Small businesses can automate communication workflows

##### 7.3.12.5: Professional Mastery** (3 days)

**05. process-orchestrator/** (Professional: "Complex processes need intelligent automation")  
- **SOURCE**: data-pipeline/ (5 agents) + workflow-hub/ (4 agents) → MERGE + OPTIMIZE to 7 agents
- **Agents**: process_coordinator, data_transformer, quality_monitor, workflow_optimizer, error_resolver, system_monitor, report_generator
- **Workflows**: Loop workflows, nested orchestration, monitoring, error handling
- **Crates**: + llmspell-workflows (loop), llmspell-hooks, llmspell-events (advanced), full monitoring
- **Tools**: Complete tool integration (file_operations, json_processor, http_request, webhook_caller, system_monitor)
- **Professional Problem**: Enterprise process automation (DevOps teams, operations managers)

**06. code-review-assistant/** (Professional: "Code quality at scale") ✅ WORKING
- **SOURCE**: STANDARDIZE existing app (already correctly positioned)
- **Agents**: code_analyzer, review_generator, report_formatter (3 agents)
- **Workflows**: Sequential professional workflow with structured output
- **Crates**: Professional development tools integration
- **Professional Problem**: Development team efficiency (engineering teams, managers)

**Implementation Tasks**:
- [ ] **process-orchestrator/ Creation**:
  - [ ] Merge data-pipeline/ + workflow-hub/ → process-orchestrator/
  - [ ] **AGENT OPTIMIZATION** (9→7 merger):
    - [ ] **From data-pipeline**: `data_enricher` + `quality_analyzer` → `data_transformer` (unified data processing)
    - [ ] **From data-pipeline**: Keep `anomaly_detector` → `quality_monitor`, `report_generator` → `report_generator`
    - [ ] **From workflow-hub**: `workflow_optimizer` + `error_resolver` → `workflow_optimizer` (unified optimization)
    - [ ] **From workflow-hub**: Keep `dependency_analyzer` → `process_coordinator`, `workflow_generator` → `system_monitor`
    - [ ] **REMOVE**: `pattern_finder` (data-pipeline) - covered by quality_monitor
  - [ ] **CAPABILITY FUSION**: ETL capabilities + workflow orchestration = comprehensive process automation
  - [ ] **CRATE INTEGRATION**: `llmspell-workflows` (loop), `llmspell-hooks` (monitoring), `llmspell-events` (advanced)
  - [ ] **TOOL INTEGRATION**: Complete integration (`system_monitor`, `json_processor`, `http_request`, `webhook_caller`)
  - [ ] **ADVANCED FEATURES**: Loop workflows, hook monitoring, event correlation, error recovery
  - [ ] **PROFESSIONAL TESTING**: DevOps teams adopt for production automation

- [ ] **code-review-assistant/ Standardization**:
  - [ ] **AGENT CONSOLIDATION**: Combine `security_reviewer`, `performance_reviewer`, `style_reviewer`, `architecture_reviewer`, `test_reviewer`, `documentation_reviewer`, `general_code_reviewer` → `code_analyzer`, keep `review_generator` + `report_formatter`
  - [ ] **MAINTAIN PROFESSIONAL COMPLEXITY**: Already at appropriate professional layer complexity
  - [ ] **PROGRESSIVE CONTEXT**: Add headers showing skill building path from Layer 1-5
  - [ ] **DOCUMENTATION UPDATES**: Position as professional automation achievement
  - [ ] **WORKFLOW VALIDATION**: Ensure professional-grade structured workflow demonstration

##### 7.3.12.6: Expert Showcase** (1 day)

**07. webapp-creator/** (Expert: "Build applications with AI") ✅ WORKING
- **SOURCE**: STANDARDIZE existing app (already correctly positioned)  
- **Agents**: Complete 20-agent orchestration (architecture, UI, backend, database, deployment)
- **Workflows**: Master-level nested orchestration with complex state management
- **Crates**: Complete llmspell ecosystem at maximum complexity
- **Expert Problem**: Full-stack development automation (senior developers, architects, CTOs)

**Implementation Tasks**:
- [ ] **webapp-creator/ Standardization**:
  - [ ] **MAINTAIN ALL 21 AGENTS**: Represents peak complexity, no reduction required
  - [ ] **CRATE SHOWCASE**: Demonstrate complete `llmspell` ecosystem integration
  - [ ] **PROGRESSIVE CONTEXT**: Add headers showing full journey completion from Layer 1-6
  - [ ] **EXPERT POSITIONING**: Position as ultimate achievement of AI automation mastery
  - [ ] **SESSIONS + ARTIFACTS**: Validate `llmspell-sessions` with artifact management for iterative development
  - [ ] **ADVANCED STATE**: Complex state management with migration capabilities demonstration
  - [ ] **EXPERT VALIDATION**: Expert developers can generate full applications

##### 7.3.12.7: Integration & Validation** (2 days)
- [ ] **Cross-Application Integration**:
  - [ ] **CRATE INTEGRATION DEPENDENCIES**: Validate Layer 1-2 State removal, Layer 3 conditional workflows, Layer 4 sessions/events, Layer 5-6 hooks/advanced features
  - [ ] **LEARNING PATH VALIDATION**: Ensure natural progression Layer 1 → Layer 6
  - [ ] **PERFORMANCE OPTIMIZATION**: Complexity should not impact basic use cases
  - [ ] **REGRESSION TESTING**: Ensure previous layer simplicity is maintained

- [ ] **Universal Appeal Validation Framework Implementation**:
  - [ ] **PHASE 1 - INTERNAL TESTING**: Developer team tests with non-technical family members
  - [ ] **USABILITY SCENARIOS**: 
    - [ ] file-organizer: 2000+ downloads folder test, business document organization, student research files
    - [ ] research-collector: vacation planning, health research, product comparison
  - [ ] **SUCCESS METRICS VALIDATION**:
    - [ ] >80% non-technical user task completion
    - [ ] >70% likelihood of recommendation to others
    - [ ] <10% abandonment due to complexity
    - [ ] <3 minutes average time to first value
  - [ ] **COMPLEXITY BOUNDARY TESTING**: Monitor for technical questions, configuration needs, premature abandonment
  - [ ] **PHASE 2 - CONTROLLED USER TESTING**: 10-15 users per application across persona categories (tech-naive, digital natives, small business, students, remote workers)
  - [ ] **NATURAL PROGRESSION VALIDATION**: Users completing Layer 1-2 naturally want Layer 3 capabilities

**Incremental Crate Integration Strategy**:

**Layer 1-2 (Universal)**: 
- Core crates only: llmspell-core, llmspell-agents, llmspell-bridge
- Basic workflows: sequential, simple parallel
- Essential tools: file_operations, text_manipulator, web_search

**Layer 3 (Power User)**:
- Add: llmspell-workflows (conditional)
- Add: Basic state management
- Add: template_engine, json_processor

**Layer 4 (Business)**:
- Add: llmspell-state-persistence, llmspell-sessions  
- Add: llmspell-events (basic)
- Add: Nested workflows
- Add: webhook_caller, email_sender

**Layer 5-6 (Professional/Expert)**:
- Add: llmspell-workflows (loop), llmspell-hooks
- Add: llmspell-events (advanced), full monitoring
- Add: Complete tool ecosystem
- Add: Complex state management, session artifacts

**Success Criteria**:
- [ ] All 7 applications run without errors with expected output
- [ ] Universal → professional progression clearly demonstrated (2 → 20 agents)
- [ ] Universal appeal validated through user testing (Layer 1-2)
- [ ] Progressive complexity builds naturally without educational jumps
- [ ] Phase 7 infrastructure fully leveraged across all layers
- [ ] Architectural diversity showcased (sequential → parallel → conditional → nested → loop)
- [ ] Real-world problems solved at every layer
- [ ] Learning curve validated from computer user → AI automation expert

---

#### Task 7.3.13: Example Documentation Integration
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Status**: TODO
**Assigned To**: Documentation Team
**Dependencies**: Task 7.3.12

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

#### Task 7.4.1: rs-llmspell browseable api documentation 
**Priority**: HIGH
**Estimated Time**: 4 hours
**Status**: TODO
**Assigned To**: Documentation Lead

**Description**: Ensure a complete set of coherent apis documentation are created for rust and lua. they should be under `docs/user-guide/api/rust/` and `docs/user-guide/api/lua`. 


#### Task 7.4.2: User Guide Standardization
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

#### Task 7.4.3: Technical Documentation Cleanup
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

#### Task 7.4.4: Developer Guide Enhancement
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

#### Task 7.4.5: Example Code Audit
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