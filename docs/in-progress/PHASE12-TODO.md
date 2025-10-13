# Phase 12: Production-Ready AI Agent Templates - TODO List

**Version**: 1.0
**Date**: October 2025
**Status**: Implementation Ready
**Phase**: 12 (Production-Ready AI Agent Templates)
**Timeline**: Weeks 42-43 (10 working days)
**Priority**: CRITICAL (Adoption Baseline - Industry Standard Requirement)
**Dependencies**: Phase 11b Local LLM Cleanup ✅
**Arch-Document**: docs/technical/master-architecture-vision.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-12-design-doc.md
**Current-Architecture**: docs/technical/current-architecture.md
**User-Guide-Docs**: docs/user-guide/*
**Developer-Guide-Docs**: docs/developer-guide/*
**Technical-Docs**: docs/technical/*
**Examples**: examples/*
**Template-Architecture**: docs/technical/template-system-architecture.md (To be created)

> **📋 Actionable Task List**: This document breaks down Phase 12 implementation into specific, measurable tasks for building production-ready AI agent template system solving the "0-day retention problem" with 6 turn-key workflow templates matching industry baseline (LangChain 50+, AutoGen ~10, CrewAI ~15).

---

## Overview

**Goal**: Implement turn-key AI agent templates system enabling immediate layman usability post-installation. Solves critical adoption gap: download → "what do I do?" → abandonment. Templates combine agents, tools, RAG, and LocalLLM into executable solutions accessible via CLI and Lua.

**Success Criteria Summary:**
- [ ] `llmspell-templates` crate compiles without warnings
- [ ] 6 built-in templates implemented and tested
- [ ] CLI commands functional: `template list|info|exec|search|schema`
- [ ] Lua bridge complete: `Template` global (16th global)
- [ ] Template discovery works (by category, by query)
- [ ] Parameter validation with clear error messages
- [ ] Template execution overhead <100ms
- [ ] All tests pass with >90% coverage, >95% API documentation
- [ ] Zero clippy warnings across workspace
- [ ] Examples for all templates (CLI + Lua)

**Strategic Context:**
- **Problem**: Users face "what do I do?" after installation (0-day retention failure)
- **Solution**: 6 production templates provide immediate value + learning by example
- **Industry Requirement**: All competing frameworks ship templates (LangChain 50+, AutoGen ~10, CrewAI ~15)
- **Phase 13 Synergy**: Templates work now, enhanced by memory later (zero breaking changes)

---

## Phase 12.1: Core Infrastructure - Template Trait System (Days 1-2)

### Task 12.1.1: Create llmspell-templates Crate Structure
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Templates Team Lead

**Description**: Create new `llmspell-templates` crate with module structure and dependencies. This is the foundation for end-user workflow templates (distinct from internal `llmspell-agents/src/templates/`).

**Acceptance Criteria:**
- [ ] Crate directory created at `/llmspell-templates`
- [ ] `Cargo.toml` configured with all dependencies
- [ ] Basic module structure in `src/lib.rs`
- [ ] Crate added to workspace members
- [ ] `cargo check -p llmspell-templates` passes

**Implementation Steps:**
1. Create `llmspell-templates/` directory structure
2. Configure `Cargo.toml` with dependencies:
   - `llmspell-core`, `llmspell-utils`, `llmspell-agents`
   - `llmspell-workflows`, `llmspell-tools`, `llmspell-rag`
   - `llmspell-state-persistence`, `llmspell-sessions`
   - `tokio`, `async-trait`, `serde`, `serde_json`, `chrono`
   - `parking_lot`, `lazy_static`, `thiserror`, `anyhow`
3. Create module structure in `src/lib.rs`:
   ```rust
   pub mod core;          // Template trait, metadata
   pub mod registry;      // TemplateRegistry with discovery
   pub mod context;       // ExecutionContext builder
   pub mod params;        // Parameter validation
   pub mod output;        // Output handling, artifacts
   pub mod error;         // Template-specific errors
   pub mod builtin;       // Built-in templates
   pub mod prelude;       // Common imports
   ```
4. Add to workspace in root `Cargo.toml`
5. Run `cargo check -p llmspell-templates`

**Definition of Done:**
- [ ] Crate compiles without errors
- [ ] All module files created (can be empty stubs)
- [ ] Dependencies resolve correctly
- [ ] No clippy warnings: `cargo clippy -p llmspell-templates`

### Task 12.1.2: Define Template Trait and Metadata
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Templates Team

**Description**: Implement core `Template` trait with metadata, schema, validation, and execution. Similar to `BaseAgent` trait but specialized for pre-configured workflow patterns.

**Acceptance Criteria:**
- [ ] `Template` trait with async execute method
- [ ] `TemplateMetadata` struct (id, name, description, category, version, tags)
- [ ] `ConfigSchema` with typed parameters
- [ ] `TemplateParams` key-value store with type-safe getters
- [ ] `TemplateOutput` with results, artifacts, metrics
- [ ] Trait tests compile and pass

**Implementation Steps:**
1. Create `src/core.rs` (250 LOC estimated):
   - Define `Template` trait with:
     - `fn metadata(&self) -> &TemplateMetadata`
     - `fn config_schema(&self) -> ConfigSchema`
     - `async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput>`
     - `fn validate(&self, params: &TemplateParams) -> Result<(), ValidationError>` (default impl)
   - Define `TemplateMetadata` with:
     - `id`, `name`, `description`, `category` (enum), `version`, `author`, `requires` (deps), `tags`
   - Define `TemplateCategory` enum:
     - Research, Chat, Analysis, CodeGen, Document, Workflow, Custom(String)
   - Define `ConfigSchema` with `Vec<ConfigParameter>`
   - Define `TemplateParams` with `HashMap<String, serde_json::Value>`
   - Define `TemplateOutput` with result, artifacts, metadata, metrics
2. Implement parameter types and validation rules:
   - `ParameterType` enum: String, Integer, Float, Boolean, Array, Object, Enum
   - `ParameterValidation` enum: MinLength, MaxLength, Range, Pattern, Custom
3. Write basic trait tests in `tests/core_test.rs`
4. Run `cargo test -p llmspell-templates`

**Definition of Done:**
- [ ] All core types compile without errors
- [ ] Template trait is async-trait compatible
- [ ] Trait object safety verified
- [ ] Basic trait tests pass (5+ tests)
- [ ] Documentation comments complete (>95% coverage)

### Task 12.1.3: Implement ExecutionContext Builder
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: Templates Team

**Description**: Create `ExecutionContext` for providing templates access to agents, tools, RAG, state, sessions from existing infrastructure.

**Acceptance Criteria:**
- [ ] `ExecutionContext` struct with all infrastructure references
- [ ] `ExecutionContextBuilder` with fluent API
- [ ] Integration with existing registries (Agent, Tool, LLM)
- [ ] Session and state scoping support
- [ ] No new dependencies added

**Implementation Steps:**
1. Create `src/context.rs` (150 LOC estimated):
   - Define `ExecutionContext` struct:
     ```rust
     pub struct ExecutionContext {
         pub state: Arc<dyn StateProvider>,
         pub rag_store: Option<Arc<dyn RAGStore>>,
         pub llm_registry: Arc<LLMRegistry>,
         pub tool_registry: Arc<ToolRegistry>,
         pub agent_registry: Arc<AgentRegistry>,
         pub workflow_factory: Arc<dyn WorkflowFactory>,
         pub session_id: Option<String>,
         pub output_dir: Option<PathBuf>,
     }
     ```
   - Implement `ExecutionContextBuilder`:
     - `with_state()`, `with_rag()`, `with_llm_registry()`, etc.
     - `build()` returns `Result<ExecutionContext>`
   - Add accessor methods for all fields
2. Implement `from_config()` for building from `LLMSpellConfig`
3. Write unit tests for builder pattern
4. Verify no circular dependencies with `cargo tree`

**Definition of Done:**
- [ ] ExecutionContext compiles and integrates with existing crates
- [ ] Builder pattern functional
- [ ] No circular dependencies
- [ ] Types are Send + Sync
- [ ] Unit tests pass (8+ tests)

### Task 12.1.4: Implement Template Registry with Discovery
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Templates Team

**Description**: Build global template registry with registration, discovery, and search similar to `ToolRegistry` pattern.

**Acceptance Criteria:**
- [ ] `TemplateRegistry` with thread-safe registration
- [ ] `TEMPLATE_REGISTRY` global lazy_static singleton
- [ ] Category-based discovery working
- [ ] Keyword search across name/description/tags
- [ ] Registry errors defined

**Implementation Steps:**
1. Create `src/registry.rs` (180 LOC estimated):
   - Implement `TemplateRegistry` struct:
     - `templates: RwLock<HashMap<String, Arc<dyn Template>>>`
     - `register()`, `get()`, `discover()`, `search()`, `list_ids()`, `count()`
   - Define `TEMPLATE_REGISTRY` lazy_static:
     ```rust
     lazy_static! {
         pub static ref TEMPLATE_REGISTRY: TemplateRegistry = {
             let registry = TemplateRegistry::new();
             register_builtin_templates(&registry);
             registry
         };
     }
     ```
   - Implement `register_builtin_templates()` (placeholder, will add templates in 12.3-12.4)
2. Implement discovery by category (returns `Vec<TemplateMetadata>`)
3. Implement keyword search (case-insensitive across name/description/tags)
4. Define `RegistryError` enum (DuplicateId, NotFound)
5. Write registry tests (12+ tests)

**Definition of Done:**
- [ ] Registry registration works (success, duplicate ID detection)
- [ ] Get by ID functional
- [ ] Category discovery works
- [ ] Keyword search functional
- [ ] Global registry initializes correctly
- [ ] Thread safety verified
- [ ] Tests pass (12+ tests)

---

## Phase 12.2: CLI Integration (Days 3-4)

### Task 12.2.1: Add Template CLI Command Structure
**Priority**: CRITICAL
**Estimated Time**: 3 hours
**Assignee**: CLI Team Lead

**Description**: Add `template` subcommand to llmspell-cli with 5 subcommands: list, info, exec, search, schema.

**Acceptance Criteria:**
- [ ] `TemplateCommands` enum defined in `cli.rs`
- [ ] Clap integration with comprehensive help text
- [ ] Parameter parsing for `--param key=value` format
- [ ] Output format support (JSON, Pretty, Text)
- [ ] Compiles without warnings

**Implementation Steps:**
1. Update `llmspell-cli/src/cli.rs`:
   - Add `Template { #[command(subcommand)] command: TemplateCommands }` to main enum
   - Define `TemplateCommands` enum with 5 variants:
     - `List { category: Option<String>, format: Option<OutputFormat> }`
     - `Info { name: String, show_schema: bool }`
     - `Exec { name: String, params: Vec<(String, String)>, output: Option<PathBuf> }`
     - `Search { query: Vec<String>, category: Option<String> }`
     - `Schema { name: String }`
2. Add comprehensive long_about descriptions with examples for each subcommand
3. Implement `parse_key_val::<String, String>` for `--param` parsing
4. Run `cargo check -p llmspell-cli`

**Definition of Done:**
- [ ] CLI structure compiles
- [ ] Help text comprehensive with examples
- [ ] Parameter parsing validates
- [ ] No clippy warnings
- [ ] `llmspell template --help` shows all subcommands

### Task 12.2.2: Implement Template List Command
**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: CLI Team

**Description**: Implement `llmspell template list [--category <cat>]` command handler.

**Acceptance Criteria:**
- [ ] Lists all registered templates from TEMPLATE_REGISTRY
- [ ] Category filter works (Research, Chat, Analysis, CodeGen, Document, Workflow)
- [ ] Output formats: JSON, Pretty (table), Text
- [ ] Shows template metadata (name, id, description, category)

**Implementation Steps:**
1. Create `llmspell-cli/src/commands/template.rs` (450 LOC total for all commands)
2. Implement `handle_template_command()` dispatcher
3. Implement `list` handler:
   - Parse category string to `TemplateCategory` enum
   - Call `TEMPLATE_REGISTRY.discover(category)`
   - Format output based on `output_format` (JSON, Pretty, Text)
4. Write integration test for list command
5. Test with `llmspell template list`, `llmspell template list --category Research`

**Definition of Done:**
- [ ] Command executes successfully
- [ ] Category filter works correctly
- [ ] All output formats display properly
- [ ] Integration test passes
- [ ] Performance: <10ms for list operation

### Task 12.2.3: Implement Template Info Command
**Priority**: HIGH
**Estimated Time**: 2 hours
**Assignee**: CLI Team

**Description**: Implement `llmspell template info <name> [--show-schema]` command handler.

**Acceptance Criteria:**
- [ ] Displays detailed template metadata
- [ ] Shows parameter schema when `--show-schema` flag used
- [ ] Output formats: JSON, Pretty (formatted), Text
- [ ] Error handling for template not found

**Implementation Steps:**
1. Implement `info` handler in `template.rs`:
   - Call `TEMPLATE_REGISTRY.get(&name)`
   - Display metadata (name, id, category, version, description, requires, tags)
   - If `--show-schema`, display parameter schema with types, defaults, validation
2. Format pretty output with proper alignment
3. Write integration test for info command
4. Test with `llmspell template info research-assistant --show-schema`

**Definition of Done:**
- [ ] Command displays all metadata correctly
- [ ] Schema display works with proper formatting
- [ ] Error handling for missing template
- [ ] Integration test passes
- [ ] Performance: <5ms for info operation

### Task 12.2.4: Implement Template Exec Command
**Priority**: CRITICAL
**Estimated Time**: 5 hours
**Assignee**: CLI Team Lead

**Description**: Implement `llmspell template exec <name> --param key=value [--output <dir>]` command handler with full template execution.

**Acceptance Criteria:**
- [ ] Parses template parameters from `--param` flags
- [ ] Builds ExecutionContext from runtime config
- [ ] Executes template asynchronously
- [ ] Displays execution metrics (duration, agents, tools, artifacts)
- [ ] Writes artifacts to output directory if specified
- [ ] Handles errors gracefully with user-friendly messages

**Implementation Steps:**
1. Implement `exec` handler in `template.rs`:
   - Parse `--param` key=value pairs into `TemplateParams`
   - Try parsing values as JSON first, fallback to string
   - Build `ExecutionContext::from_config(&runtime_config, output_dir)`
   - Call `template.execute(template_params, context).await`
   - Display metrics: duration_ms, agents_invoked, tools_called, rag_queries
   - List artifacts with paths
   - Display result based on type (Text, Structured, File, Multiple)
2. Implement error handling with context-specific messages:
   - Template not found
   - Missing required parameters
   - Invalid parameter types
   - Execution failures with propagated errors
3. Write integration test executing mock template
4. Test with research-assistant template (will add in 12.3)

**Definition of Done:**
- [ ] Command executes template successfully
- [ ] Parameter parsing handles JSON and strings
- [ ] ExecutionContext builds from config
- [ ] Metrics displayed accurately
- [ ] Artifacts saved to output directory
- [ ] Error messages user-friendly
- [ ] Integration test passes
- [ ] Template execution overhead <100ms (excluding template runtime)

### Task 12.2.5: Implement Template Search and Schema Commands
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: CLI Team

**Description**: Implement `template search <query>` and `template schema <name>` commands.

**Acceptance Criteria:**
- [ ] Search works with multiple keywords
- [ ] Search optionally filters by category
- [ ] Schema outputs valid JSON schema
- [ ] Output formats supported

**Implementation Steps:**
1. Implement `search` handler:
   - Join query words into single string
   - Call `TEMPLATE_REGISTRY.search(&query)`
   - Optionally filter by category
   - Display results with metadata
2. Implement `schema` handler:
   - Get template by name
   - Serialize `config_schema()` to JSON
   - Pretty-print JSON output
3. Write integration tests for both commands
4. Test with various queries

**Definition of Done:**
- [ ] Search finds templates by keywords in name/description/tags
- [ ] Category filter works
- [ ] Schema outputs valid JSON
- [ ] Integration tests pass (4+ tests)
- [ ] Performance: <20ms for search with 6 templates

---

## Phase 12.3: Research Assistant Template (Days 5-6)

### Task 12.3.1: Implement Research Assistant Template Core
**Priority**: CRITICAL
**Estimated Time**: 6 hours
**Assignee**: Research Template Lead

**Description**: Implement the Research Assistant template with 4-phase execution: gather (web search) → ingest (RAG) → synthesize (agent) → validate (agent).

**Acceptance Criteria:**
- [ ] `ResearchAssistantTemplate` struct implements Template trait
- [ ] 4-phase execution pipeline functional
- [ ] Web search tool integration working
- [ ] RAG ingestion working
- [ ] Two agents (synthesizer, validator) coordinated
- [ ] Configurable parameters (topic, max_sources, model, output_format, include_citations)

**Implementation Steps:**
1. Create `src/builtin/research_assistant.rs` (280 LOC):
   - Implement `ResearchAssistantTemplate::new()` with metadata:
     - id: "research-assistant"
     - category: Research
     - requires: ["web-search", "rag", "local-llm"]
     - tags: ["research", "citations", "multi-source", "synthesis"]
   - Implement `config_schema()` with 5 parameters:
     - topic (String, required, MinLength(3))
     - max_sources (Integer, optional, default=10, Range 1-50)
     - model (String, optional, default="ollama/llama3.2:3b")
     - output_format (Enum ["markdown", "json", "html"], default="markdown")
     - include_citations (Boolean, optional, default=true)
2. Implement `execute()` with 4 phases:
   - **Phase 1 (Gather)**: Parallel web search workflow
     - Create parallel workflow with 2 web-search steps
     - Execute and extract documents
   - **Phase 2 (Ingest)**: RAG indexing
     - Get RAG store from context
     - Create session tag
     - Ingest all documents with tag
   - **Phase 3 (Synthesize)**: Agent with RAG retrieval
     - Create synthesizer agent with system prompt
     - Execute with topic query and RAG context
     - Generate synthesis with citations
   - **Phase 4 (Validate)**: Citation validator agent
     - Create validator agent
     - Execute validation on synthesis
     - Generate validation report
3. Implement output formatting (markdown, JSON, HTML)
4. Save artifacts to output directory (synthesis.md, validation.md)
5. Calculate metrics (duration, agents invoked, tools called)

**Definition of Done:**
- [ ] Template executes all 4 phases successfully
- [ ] Web search integration works
- [ ] RAG ingestion and retrieval functional
- [ ] Both agents execute and coordinate
- [ ] All output formats generate correctly
- [ ] Artifacts saved properly
- [ ] Metrics calculated accurately

### Task 12.3.2: Research Assistant Template Testing
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: QA Team

**Description**: Comprehensive testing of Research Assistant template with unit and integration tests.

**Acceptance Criteria:**
- [ ] Unit tests for metadata and schema
- [ ] Integration test with mock web search
- [ ] Integration test with mock RAG store
- [ ] Integration test with mock agents
- [ ] End-to-end test with all components
- [ ] Test coverage >90%

**Implementation Steps:**
1. Create `tests/research_assistant_test.rs`:
   - Test metadata values
   - Test schema parameters and validation
   - Mock web search tool (returns fake documents)
   - Mock RAG store (ingestion and retrieval)
   - Mock agents (synthesizer and validator)
   - End-to-end test with all mocks
2. Test parameter validation:
   - Missing required parameters
   - Invalid enum values
   - Range violations
3. Test error handling:
   - Missing dependencies (web-search, rag)
   - Agent creation failures
   - Tool execution failures
4. Run tests: `cargo test -p llmspell-templates research_assistant`

**Definition of Done:**
- [ ] All unit tests pass (8+ tests)
- [ ] Integration tests pass (4+ tests)
- [ ] Test coverage >90%
- [ ] Error handling tested
- [ ] No flaky tests

### Task 12.3.3: Research Assistant Examples and Documentation
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Documentation Team

**Description**: Create CLI and Lua examples plus comprehensive documentation for Research Assistant template.

**Acceptance Criteria:**
- [ ] CLI example with basic usage
- [ ] CLI example with custom configuration
- [ ] Lua example with basic usage
- [ ] Lua example with custom configuration
- [ ] User guide documentation

**Implementation Steps:**
1. Create `examples/templates/research/`:
   - `cli-basic.sh`: Basic CLI usage
   - `cli-advanced.sh`: Advanced CLI with all parameters
   - `basic.lua`: Basic Lua usage
   - `customized.lua`: Custom configuration in Lua
2. Create `docs/user-guide/templates/research-assistant.md` (250 LOC):
   - Overview and use cases
   - Parameter reference
   - CLI examples
   - Lua examples
   - Output format examples
   - Troubleshooting guide
3. Test all examples execute successfully
4. Run quality-check-fast.sh

**Definition of Done:**
- [ ] All examples execute successfully
- [ ] Documentation comprehensive
- [ ] Examples well-commented
- [ ] User guide helpful
- [ ] Quality checks pass

---

## Phase 12.4: Additional Templates (Days 7-8)

### Task 12.4.1: Implement Interactive Chat Template
**Priority**: HIGH
**Estimated Time**: 4 hours
**Assignee**: Chat Template Lead

**Description**: Implement Interactive Chat template with session-based conversation, tool integration, and memory placeholder for Phase 13.

**Acceptance Criteria:**
- [ ] `InteractiveChatTemplate` implements Template trait
- [ ] Session-based conversation history
- [ ] Optional tool integration (user-configurable)
- [ ] Interactive mode (stdin) + programmatic mode
- [ ] Memory placeholder ready for Phase 13

**Implementation Steps:**
1. Create `src/builtin/interactive_chat.rs` (220 LOC):
   - Metadata: category=Chat, requires=["local-llm"]
   - Schema parameters: model, system_prompt, max_turns, tools (array), enable_memory (boolean)
   - Execute:
     - Create/restore session
     - Load tools from registry
     - Create chat agent with tools
     - Conversation loop (interactive or single-shot)
     - Track conversation history
     - Save session state
2. Implement two modes:
   - Interactive: read from stdin, loop until "exit"/"quit"
   - Programmatic: single message parameter, execute once
3. Add memory placeholder (log warning if enabled)
4. Write tests (8+ tests)
5. Create examples and documentation

**Definition of Done:**
- [ ] Template executes in both modes
- [ ] Session persistence works
- [ ] Tool integration functional
- [ ] Tests pass >90% coverage
- [ ] Examples working

### Task 12.4.2: Implement Data Analysis Template
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Templates Team

**Description**: Implement Data Analysis template with stats agent + visualization agent in sequential workflow.

**Acceptance Criteria:**
- [ ] `DataAnalysisTemplate` implements Template trait
- [ ] Sequential workflow (analyzer → visualizer)
- [ ] Data loading from file
- [ ] Statistical analysis with agent
- [ ] Visualization generation with agent

**Implementation Steps:**
1. Create `src/builtin/data_analysis.rs` (180 LOC):
   - Metadata: category=Analysis, requires=["data-loader", "stats"]
   - Schema: data_file (required), analysis_type (enum), chart_type (enum)
   - Execute:
     - Load data using data-loader tool
     - Create analyzer agent, execute analysis
     - Create visualizer agent, generate charts
     - Save outputs (report + chart)
2. Write tests with mock data
3. Create examples

**Definition of Done:**
- [ ] Sequential workflow functional
- [ ] Both agents coordinate
- [ ] Tests pass
- [ ] Examples working

### Task 12.4.3: Implement Code Generator Template
**Priority**: MEDIUM
**Estimated Time**: 3 hours
**Assignee**: Templates Team

**Description**: Implement Code Generator template with 3-agent chain: spec → impl → test.

**Acceptance Criteria:**
- [ ] `CodeGeneratorTemplate` implements Template trait
- [ ] 3-agent sequential chain
- [ ] Specification agent generates design
- [ ] Implementation agent writes code
- [ ] Test agent writes tests

**Implementation Steps:**
1. Create `src/builtin/code_generator.rs` (200 LOC):
   - Metadata: category=CodeGen, requires=["code-tools", "lint"]
   - Schema: description (required), language (enum), include_tests (boolean)
   - Execute:
     - Spec agent: generate design from description
     - Implementation agent: write code from spec
     - Test agent: generate tests for code
     - Save artifacts (spec.md, code file, test file)
2. Write tests
3. Create examples

**Definition of Done:**
- [ ] 3-agent chain functional
- [ ] Code generation works
- [ ] Tests pass
- [ ] Examples working

### Task 12.4.4: Implement Document Processor and Workflow Orchestrator Templates
**Priority**: MEDIUM
**Estimated Time**: 4 hours
**Assignee**: Templates Team

**Description**: Implement remaining 2 templates to complete 6 total built-in templates.

**Acceptance Criteria:**
- [ ] `DocumentProcessorTemplate` with PDF extraction + transformation
- [ ] `WorkflowOrchestratorTemplate` with custom patterns
- [ ] Both templates tested
- [ ] Examples created

**Implementation Steps:**
1. Create `src/builtin/document_processor.rs` (180 LOC):
   - Metadata: category=Document, requires=["pdf-reader", "ocr"]
   - Parallel workflow for multi-document processing
   - Extractor agent + transformer agent
2. Create `src/builtin/workflow_orchestrator.rs` (150 LOC):
   - Metadata: category=Workflow
   - User-configurable agent/tool composition
   - Custom parallel/sequential patterns
3. Write tests for both (12+ tests total)
4. Create examples for both
5. Update `register_builtin_templates()` to register all 6

**Definition of Done:**
- [ ] Both templates functional
- [ ] All 6 templates registered in TEMPLATE_REGISTRY
- [ ] Tests pass (12+ tests)
- [ ] Examples working

---

## Phase 12.5: Lua Bridge Integration (Day 9)

### Task 12.5.1: Create Template Global Object
**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Assignee**: Bridge Team Lead

**Description**: Implement `Template` global (16th global) for Lua with template discovery and execution.

**Acceptance Criteria:**
- [ ] `TemplateGlobal` implements GlobalObject trait
- [ ] 4 Lua functions: list, info, execute, search
- [ ] Type conversions Lua ↔ Rust working
- [ ] Async execute support
- [ ] Registered in global registry

**Implementation Steps:**
1. Create `llmspell-bridge/src/globals/template_global.rs` (380 LOC):
   - Implement `TemplateGlobal` struct with GlobalObject trait
   - Metadata: name="Template", version="0.12.0", dependencies=["provider_manager"]
2. Implement `inject_template_global(lua, context)`:
   - `Template.list([category])` → table
   - `Template.info(id)` → table (metadata + schema)
   - `Template.execute(id, params)` → async result
   - `Template.search(query)` → table
3. Implement type conversions:
   - `lua_value_to_json()`: LuaValue → serde_json::Value
   - `template_output_to_lua()`: TemplateOutput → LuaTable
   - Handle arrays vs objects correctly
4. Implement `build_execution_context()` from GlobalContext
5. Write Lua integration tests (12+ tests)

**Definition of Done:**
- [ ] Global object compiles
- [ ] All 4 functions work from Lua
- [ ] Type conversions bidirectional
- [ ] Async execute functional
- [ ] Tests pass (12+ tests)

### Task 12.5.2: Register Template Global in Bridge
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Bridge Team

**Description**: Register Template global in bridge initialization, making it the 16th global.

**Acceptance Criteria:**
- [ ] Template global registered in standard registry
- [ ] Injection happens during bridge initialization
- [ ] Dependencies resolved correctly
- [ ] No circular dependencies
- [ ] Global accessible from all scripts

**Implementation Steps:**
1. Update `llmspell-bridge/src/globals/mod.rs`:
   - Add `pub mod template_global;`
   - Create `register_template_global(builder, context)` function
   - Add call in `create_standard_registry()`
2. Verify dependency resolution (needs provider_manager)
3. Test global availability in Lua script
4. Update global count documentation (15 → 16)
5. Run workspace tests

**Definition of Done:**
- [ ] Global registered successfully
- [ ] Available in all Lua scripts
- [ ] Dependencies resolved
- [ ] Tests pass
- [ ] Documentation updated

### Task 12.5.3: Create Lua Template Examples
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Examples Team

**Description**: Create comprehensive Lua examples for all 6 templates demonstrating Template global usage.

**Acceptance Criteria:**
- [ ] Example for each template (6 total)
- [ ] Discovery example (list + search)
- [ ] Schema introspection example
- [ ] Error handling examples
- [ ] All examples execute successfully

**Implementation Steps:**
1. Create `examples/templates/` directory structure:
   - `discovery.lua`: Template.list() and Template.search() usage
   - `research/lua-basic.lua`: Basic research assistant
   - `chat/lua-basic.lua`: Basic interactive chat
   - `analysis/lua-basic.lua`: Basic data analysis
   - `codegen/lua-basic.lua`: Basic code generator
   - `documents/lua-basic.lua`: Basic document processor
   - `orchestration/lua-basic.lua`: Basic workflow orchestrator
2. Add comprehensive comments explaining each API call
3. Test all examples execute successfully
4. Create `examples/templates/README.md` with overview

**Definition of Done:**
- [ ] 7 Lua examples created
- [ ] All examples execute successfully
- [ ] Well-commented and educational
- [ ] README helpful
- [ ] Examples tested with quality-check-fast.sh

---

## Phase 12.6: Testing, Quality, and Release (Day 10)

### Task 12.6.1: Comprehensive Unit Test Suite
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: QA Team

**Description**: Complete unit test coverage for all template system components.

**Acceptance Criteria:**
- [ ] >90% code coverage for llmspell-templates
- [ ] All edge cases covered
- [ ] Mock implementations for external dependencies
- [ ] Tests run in CI
- [ ] No flaky tests

**Implementation Steps:**
1. Review coverage with `cargo tarpaulin -p llmspell-templates`
2. Add missing tests:
   - Core trait tests (Template, TemplateMetadata)
   - Registry tests (registration, discovery, search)
   - Parameter validation tests (all validation types)
   - ExecutionContext builder tests
   - Error handling tests
3. Create `tests/integration_test.rs`:
   - End-to-end template execution with mocks
   - Multi-template workflow test
   - Error propagation test
4. Run full test suite: `cargo test --workspace`

**Definition of Done:**
- [ ] Coverage >90% verified
- [ ] All edge cases tested
- [ ] Integration tests pass
- [ ] CI tests passing
- [ ] Zero flaky tests

### Task 12.6.2: Performance Benchmarks
**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Assignee**: Performance Team

**Description**: Benchmark template system overhead and ensure <100ms target met.

**Acceptance Criteria:**
- [ ] Template list <10ms
- [ ] Template info <5ms
- [ ] Template execute overhead <100ms (excluding template runtime)
- [ ] Parameter validation <5ms
- [ ] Benchmarks reproducible

**Implementation Steps:**
1. Create `benches/template_bench.rs`:
   - Benchmark template list operation
   - Benchmark template info operation
   - Benchmark parameter parsing and validation
   - Benchmark ExecutionContext creation
2. Run benchmarks: `cargo bench -p llmspell-templates`
3. Document results in `PERFORMANCE.md`
4. Create regression test for future runs

**Definition of Done:**
- [ ] All performance targets met
- [ ] Benchmarks reproducible
- [ ] Results documented
- [ ] Regression detection configured

### Task 12.6.3: Quality Gates and Clippy Compliance
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: All Team

**Description**: Ensure all quality gates pass before release.

**Acceptance Criteria:**
- [ ] Zero clippy warnings workspace-wide
- [ ] Format compliance 100%
- [ ] Documentation coverage >95%
- [ ] All examples compile and run
- [ ] Quality check scripts pass

**Implementation Steps:**
1. Run `./scripts/quality/quality-check-fast.sh`
2. Fix all clippy warnings:
   - `cargo clippy --workspace --all-features --all-targets`
3. Fix format issues:
   - `cargo fmt --all`
4. Verify documentation:
   - `cargo doc --workspace --all-features --no-deps`
   - Check for missing doc comments
5. Test all examples:
   - CLI examples in `examples/templates/*/cli-*.sh`
   - Lua examples in `examples/templates/*/*.lua`
6. Run full quality check:
   - `./scripts/quality/quality-check.sh`

**Definition of Done:**
- [ ] Clippy clean (zero warnings)
- [ ] Format compliant
- [ ] Documentation >95%
- [ ] All examples work
- [ ] Full quality check passes

### Task 12.6.4: Documentation Finalization
**Priority**: HIGH
**Estimated Time**: 3 hours
**Assignee**: Documentation Team

**Description**: Complete all documentation including user guides, API docs, and architecture document.

**Acceptance Criteria:**
- [ ] Template system architecture document created
- [ ] User guide complete for all 6 templates
- [ ] API documentation >95% coverage
- [ ] README files helpful
- [ ] Migration guide (if needed)

**Implementation Steps:**
1. Create `docs/technical/template-system-architecture.md`:
   - System overview and design principles
   - Template trait hierarchy
   - Registry and discovery architecture
   - CLI integration architecture
   - Lua bridge architecture
   - Performance considerations
   - Future enhancements (Phase 13 memory)
2. Complete template user guides (6 templates):
   - `docs/user-guide/templates/research-assistant.md` (already done in 12.3.3)
   - `docs/user-guide/templates/interactive-chat.md`
   - `docs/user-guide/templates/data-analysis.md`
   - `docs/user-guide/templates/code-generator.md`
   - `docs/user-guide/templates/document-processor.md`
   - `docs/user-guide/templates/workflow-orchestrator.md`
3. Create `docs/user-guide/templates/README.md`:
   - Template system overview
   - Getting started guide
   - CLI usage examples
   - Lua usage examples
   - Troubleshooting
4. Update `docs/in-progress/implementation-phases.md`:
   - Mark Phase 12 as complete
   - Update Phase 13 dependencies
5. Verify API documentation:
   - `cargo doc --workspace --all-features --no-deps --open`

**Definition of Done:**
- [ ] Architecture document complete
- [ ] All 6 template guides complete
- [ ] Template system README helpful
- [ ] Implementation phases updated
- [ ] API docs >95% coverage

### Task 12.6.5: Release Preparation
**Priority**: CRITICAL
**Estimated Time**: 2 hours
**Assignee**: Release Manager

**Description**: Prepare Phase 12 for release including RELEASE_NOTES and version updates.

**Acceptance Criteria:**
- [ ] `RELEASE_NOTES_v0.12.0.md` created
- [ ] Version bumped to 0.12.0 in all crates
- [ ] CHANGELOG.md updated
- [ ] Git tags prepared
- [ ] Phase handoff document ready

**Implementation Steps:**
1. Create `RELEASE_NOTES_v0.12.0.md`:
   - Executive summary
   - New features (6 templates, CLI, Lua API)
   - Breaking changes (none expected)
   - Performance improvements
   - Documentation improvements
   - Migration guide (if needed)
2. Update version to 0.12.0:
   - `llmspell-templates/Cargo.toml`
   - `llmspell-cli/Cargo.toml` (template command)
   - `llmspell-bridge/Cargo.toml` (Template global)
3. Update `CHANGELOG.md` with Phase 12 changes
4. Create `PHASE12_HANDOFF_PACKAGE.md`:
   - Architecture overview
   - Implementation summary
   - Known limitations
   - Phase 13 integration points
   - Performance baselines
5. Run final validation

**Definition of Done:**
- [ ] Release notes comprehensive
- [ ] Version updates complete
- [ ] CHANGELOG updated
- [ ] Handoff package ready
- [ ] Ready for git tag and release

---

## Final Validation Checklist

### Quality Gates
- [ ] All crates compile without warnings
- [ ] Clippy passes with zero warnings: `cargo clippy --workspace --all-features --all-targets`
- [ ] Format compliance: `cargo fmt --all --check`
- [ ] Tests pass: `cargo test --workspace --all-features`
- [ ] Documentation builds: `cargo doc --workspace --all-features --no-deps`
- [ ] All examples run successfully (CLI + Lua)
- [ ] Benchmarks meet targets

### Feature Validation
- [ ] 6 built-in templates implemented and tested
- [ ] Template trait system functional
- [ ] Registry with discovery and search working
- [ ] CLI commands functional (list, info, exec, search, schema)
- [ ] Lua bridge complete (Template global functional)
- [ ] Parameter validation with clear errors
- [ ] Artifact generation working
- [ ] ExecutionContext integration with all infrastructure

### Performance Validation
- [ ] Template list: <10ms
- [ ] Template info: <5ms
- [ ] Template execute overhead: <100ms (excluding template runtime)
- [ ] Parameter validation: <5ms
- [ ] Registry search: <20ms for 6 templates
- [ ] Memory overhead: <10MB for registry

### Documentation Validation
- [ ] API docs coverage >95%
- [ ] Architecture docs complete
- [ ] User guides comprehensive (6 templates)
- [ ] Template system README helpful
- [ ] CLI help text complete
- [ ] Lua examples working
- [ ] Migration guide (if needed)

### Integration Validation
- [ ] Templates use existing agents infrastructure
- [ ] Templates use existing tools infrastructure
- [ ] Templates use existing RAG infrastructure
- [ ] Templates use existing LocalLLM infrastructure
- [ ] Templates use existing state/session infrastructure
- [ ] CLI integration seamless
- [ ] Lua bridge integration seamless
- [ ] No circular dependencies

### Phase 13 Readiness
- [ ] Memory placeholders in templates
- [ ] No breaking changes planned for memory integration
- [ ] Template trait extensible for memory
- [ ] ExecutionContext ready for memory manager
- [ ] Templates designed for .enable_memory() enhancement

---

## Risk Mitigation

### Technical Risks

1. **Template Execution Complexity**
   - **Risk**: Complex templates may have hidden bugs in multi-agent coordination
   - **Mitigation**: Comprehensive testing with mocks, gradual rollout starting with Research Assistant
   - **Impact**: Low - well-tested infrastructure (agents, workflows)

2. **Parameter Validation Edge Cases**
   - **Risk**: JSON parsing edge cases may cause runtime errors
   - **Mitigation**: Exhaustive validation tests, fallback to string parsing
   - **Impact**: Low - validation happens before execution

3. **Lua Type Conversion Issues**
   - **Risk**: Complex nested structures may not convert correctly Lua ↔ Rust
   - **Mitigation**: Test with complex parameter types, array/object heuristics
   - **Impact**: Medium - affects usability

4. **Performance Overhead**
   - **Risk**: Template abstraction layer adds latency
   - **Mitigation**: Benchmark all operations, lazy initialization, Arc for shared state
   - **Impact**: Low - <100ms target conservative

### Schedule Risks

1. **Research Assistant Complexity**
   - **Risk**: 4-phase template may take longer than 6 hours
   - **Mitigation**: Use existing workflow patterns, parallelize testing
   - **Impact**: Low - can extend Day 6 if needed

2. **Template Count Ambition**
   - **Risk**: 6 templates in 10 days may be aggressive
   - **Mitigation**: Prioritize Research + Chat (Days 5-7), others can be simpler
   - **Impact**: Medium - can reduce to 4 templates if needed

3. **Integration Testing Time**
   - **Risk**: End-to-end testing may reveal late issues
   - **Mitigation**: Test incrementally during development, use mocks early
   - **Impact**: Low - buffer in Day 10

4. **Documentation Scope**
   - **Risk**: Comprehensive docs for 6 templates is time-consuming
   - **Mitigation**: Template structure similar, reuse documentation patterns
   - **Impact**: Low - can parallelize with implementation

---

## Notes and Decisions Log

### Architectural Decisions

- **Decision**: Create separate `llmspell-templates` crate vs extending `llmspell-agents`
  - **Rationale**: Clear separation of concerns - templates are end-user facing workflow patterns, not internal infrastructure
  - **Impact**: New crate to maintain, but better organization

- **Decision**: Template trait uses async execution like BaseAgent
  - **Rationale**: Templates orchestrate async agents/tools, need async context
  - **Impact**: Consistent with existing patterns

- **Decision**: Templates leverage 100% existing infrastructure (no new dependencies)
  - **Rationale**: Phase 12 is about UX, not new capabilities - reuse agents/tools/RAG/LocalLLM
  - **Impact**: Faster implementation, no dependency bloat

- **Decision**: CLI-first design with Lua bridge as secondary interface
  - **Rationale**: Direct execution via CLI maximizes accessibility (no scripting required)
  - **Impact**: More CLI code, but better user experience

- **Decision**: 6 templates match industry distribution (40% Research, 30% Chat, 15% CodeGen, 10% Data, 5% Workflow)
  - **Rationale**: Competitive analysis of LangChain, AutoGen, CrewAI usage patterns
  - **Impact**: Balanced coverage of use cases

- **Decision**: Template global is 16th global (after LocalLLM)
  - **Rationale**: Logical grouping with other high-level abstractions (Agent, Workflow, Tool)
  - **Impact**: Consistent global namespace

### Implementation Notes

- Research Assistant is most complex template (4 phases) - prioritize for Day 5-6
- Interactive Chat needs session management - leverage existing llmspell-sessions
- Templates designed for Phase 13 memory enhancement (enable_memory placeholder)
- Parameter validation errors must be user-friendly (not technical stack traces)
- Template execution should log progress for long-running operations
- Artifact management reuses session artifact storage patterns

### Dependencies - ZERO NEW EXTERNAL CRATES

All dependencies are internal workspace crates:
- `llmspell-core` - Core traits
- `llmspell-utils` - Shared utilities
- `llmspell-agents` - Agent infrastructure
- `llmspell-workflows` - Workflow patterns
- `llmspell-tools` - Tool ecosystem (26+ tools)
- `llmspell-rag` - RAG infrastructure (Phase 8)
- `llmspell-state-persistence` - State management
- `llmspell-sessions` - Session lifecycle
- Standard Rust crates already in use: `tokio`, `async-trait`, `serde`, `parking_lot`, `lazy_static`, `thiserror`, `anyhow`

### Breaking Changes

**NONE** - Phase 12 is additive only:
- New crate: llmspell-templates
- New CLI command: template
- New Lua global: Template (16th global)
- No changes to existing APIs

---

## Team Assignments

**Templates Team Lead**: Overall coordination, template architecture, registry
**Research Template Lead**: Research Assistant implementation (most complex)
**Chat Template Lead**: Interactive Chat implementation (session integration)
**Templates Team**: Additional 4 templates (Data Analysis, Code Generator, Document Processor, Workflow Orchestrator)
**CLI Team Lead**: CLI command structure and exec command
**CLI Team**: List, info, search, schema commands
**Bridge Team Lead**: Template global implementation and registration
**Bridge Team**: Lua type conversions and examples
**QA Team**: Testing, benchmarks, integration validation
**Documentation Team**: User guides, API docs, architecture document
**Examples Team**: Lua and CLI examples for all templates
**Performance Team**: Benchmarking and performance validation
**Release Manager**: Release notes, version updates, handoff package

---

## Daily Standup Topics

**Day 1**: Crate setup, Template trait, ExecutionContext, Registry foundation
**Day 2**: Template trait completion, registry discovery, CLI command structure
**Day 3**: CLI list/info commands, parameter validation, output formatting
**Day 4**: CLI exec/search/schema commands, end-to-end CLI testing
**Day 5**: Research Assistant Phase 1-2 (gather + ingest), web search + RAG integration
**Day 6**: Research Assistant Phase 3-4 (synthesize + validate), testing, examples
**Day 7**: Interactive Chat template, Data Analysis template, testing
**Day 8**: Code Generator, Document Processor, Workflow Orchestrator templates
**Day 9**: Lua Template global, bridge integration, Lua examples
**Day 10**: Final testing, quality gates, documentation, release prep

---

**END OF PHASE 12 TODO DOCUMENT**
