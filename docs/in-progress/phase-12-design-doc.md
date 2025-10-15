# Phase 12: Production-Ready AI Agent Templates - Comprehensive Design

**Version**: 1.0 (Holistic Design Document)
**Date**: October 2025
**Status**: 🔄 READY FOR IMPLEMENTATION
**Phase**: 12 (Production-Ready AI Agent Templates)
**Timeline**: 10 working days (2 weeks)
**Dependencies**: Phase 11b Local LLM Cleanup ✅

> **📋 Document Purpose**: Authoritative design document for Phase 12, covering architecture, implementation, integration, testing, and operations in unified comprehensive view. This document defines the template system that solves the "0-day retention problem" by providing turn-key AI workflow templates matching industry baseline (LangChain 50+, AutoGen ~10, CrewAI ~15).

---

## Executive Summary

Phase 12 delivers production-ready AI agent templates system enabling immediate user value post-installation. Solves critical adoption gap: download → "what do I do?" → abandonment. Templates are NOT internal infrastructure (that's `llmspell-agents/templates`) but end-user facing workflow patterns combining agents, tools, RAG, and LocalLLM into executable solutions.

### Key Achievements (Target)

**6 Production Templates Delivered**:
1. ✅ Research Assistant (4-phase: gather → ingest → synthesize → validate)
2. ✅ Interactive Chat (session-based conversation with optional memory)
3. ✅ Data Analysis (stats + visualization agents)
4. ✅ Code Generator (spec → impl → test agent chain)
5. ✅ Document Processor (PDF/OCR + transformation)
6. ✅ Workflow Orchestrator (custom parallel/sequential patterns)

**Architectural Integration**:
- NEW `llmspell-templates` crate for workflow templates
- CLI integration: `llmspell template {list|info|exec|search|schema}`
- Lua bridge: `Template` global (16th global, joining Agent/Workflow/Tool/RAG)
- Zero new dependencies (100% existing infrastructure)
- 2-week timeline (10 days implementation)

**Competitive Advantages**:
- **10-100x faster** than Python frameworks (Rust performance)
- **Compile-time validation** vs runtime template errors
- **True local-first** with Candle offline execution
- **Multi-language** templates (Lua/JS/Python via bridge)
- **CLI-direct** execution without scripting
- **Zero external dependencies** self-contained

---

## Table of Contents

1. [Strategic Context](#strategic-context)
2. [Architecture Overview](#architecture-overview)
3. [Component 1: Template Trait System](#component-1-template-trait-system)
4. [Component 2: Template Registry](#component-2-template-registry)
5. [Component 3: Built-in Templates](#component-3-built-in-templates)
6. [Component 4: CLI Integration](#component-4-cli-integration)
7. [Component 5: Lua Bridge](#component-5-lua-bridge)
8. [Component 6: Parameter Validation](#component-6-parameter-validation)
9. [Component 7: Artifact Management](#component-7-artifact-management)
10. [Integration Architecture](#integration-architecture)
11. [Performance Targets](#performance-targets)
12. [Testing Strategy](#testing-strategy)
13. [Operations Guide](#operations-guide)
14. [Phase 13 Memory Synergy](#phase-13-memory-synergy)
15. [Competitive Analysis](#competitive-analysis)
16. [Implementation Timeline](#implementation-timeline)

---

## Strategic Context

### The Adoption Crisis (0-Day Retention Problem)

**Problem Statement**: Users encounter immediate usability barrier post-installation.

**User Journey Without Templates** (Current State):
```
Download LLMSpell v0.11.2 → Read docs → Face blank canvas →
Must architect workflows from scratch → Steep learning curve →
Abandon (0-day retention failure)
```

**User Journey With Templates** (Phase 12 Target):
```
Download LLMSpell v0.12.0 → Run `llmspell template list` →
Execute `llmspell template exec research-assistant --param topic="..."` →
Immediate value (working pipeline) → Inspect code → Modify → Build custom
```

### Industry Baseline Requirement

Templates are NOT optional feature, they're **adoption baseline**:

| Framework | Templates | Distribution | Market Share |
|-----------|-----------|--------------|--------------|
| **LangChain** | 50+ (LangGraph) | 40% Research, 30% Chat | Dominant |
| **AutoGen** | ~10 patterns | Group chat orchestration | Microsoft-backed |
| **CrewAI** | ~15 role-based | Researcher/Writer/Critic | Growing |
| **Semantic Kernel** | Plugin-based | Skills + Planners | Enterprise |
| **LLMSpell** | 0 (Phase 11) → 6 (Phase 12) | Research/Chat/Code/Data | **Competitive parity** |

**Market Expectation**: Templates ship WITH framework, not as separate package.

### Why Now? (Post-Phase 11b Timing)

**Infrastructure Readiness**:
- ✅ **Phase 0-10**: Foundation (agents, workflows, tools, RAG, sessions, hooks, kernel, IDE)
- ✅ **Phase 11**: Local LLM (Ollama + Candle) - templates can run 100% offline
- ✅ **Phase 11a**: Bridge consolidation (87% compile speedup, API standardization)
- ✅ **Phase 11b**: Config unification, model discovery, LocalLLM global (15/15 globals working)
- **Result**: 100% existing infrastructure, zero new dependencies, 2-week realistic

**Competitive Positioning**:
- Users compare LLMSpell to LangChain/AutoGen/CrewAI NOW
- Competitors ALL ship templates
- Without templates: LLMSpell perceived as "incomplete" despite superior infrastructure
- Urgency: Market share depends on adoption, adoption depends on templates

**Phase 13 Memory Synergy**:
- **Templates Now** (Phase 12): Fully functional without memory
- **Memory Later** (Phase 13): Same templates enhanced with A-TKG (zero breaking changes)
- **Marketing Story**: "Phase 12 makes it usable, Phase 13 makes it intelligent"

### Alternatives Rejected

**Alternative 1**: User-Contributed Templates Only
- **Rejected**: Chicken-egg problem (need templates to learn, need framework knowledge to create)

**Alternative 2**: Documentation Examples Instead
- **Rejected**: Docs show fragments, templates provide working end-to-end solutions

**Alternative 3**: Defer Until Post-1.0
- **Rejected**: Adoption suffers until 1.0, negative momentum vs competitors

**Alternative 4**: External Template Marketplace
- **Rejected**: Fragmentation, quality issues, discovery problems (framework MUST ship baseline)

---

## Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        llmspell CLI                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │   template   │ │   template   │ │   template   │            │
│  │   list       │ │   info       │ │   exec       │            │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘            │
└─────────┼─────────────────┼─────────────────┼────────────────────┘
          │                 │                 │
          ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│               llmspell-templates (NEW CRATE)                     │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Template Trait System                                       │ │
│  │  • Template trait (metadata, schema, execute)              │ │
│  │  • TemplateMetadata (id, name, category, version, tags)   │ │
│  │  • ConfigSchema (typed parameters with validation)         │ │
│  │  • TemplateParams (key-value parameter store)              │ │
│  │  • TemplateOutput (result, artifacts, metrics)             │ │
│  │  • ExecutionContext (state, RAG, LLM, tools, sessions)     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Template Registry                                           │ │
│  │  • TemplateRegistry (register, get, discover, search)      │ │
│  │  • TEMPLATE_REGISTRY (global lazy_static singleton)        │ │
│  │  • register_builtin_templates() (6 templates)              │ │
│  │  • Category-based discovery                                │ │
│  │  • Keyword search across name/description/tags             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Built-in Templates (6 Production Templates)                 │ │
│  │  1. ResearchAssistantTemplate                              │ │
│  │     • 4-phase: gather (web) → ingest (RAG) → synthesize →  │ │
│  │       validate (citations)                                  │ │
│  │     • Tools: web-search, rag-search                        │ │
│  │     • Agents: synthesizer, validator                        │ │
│  │     • Configurable: topic, max_sources, model, format      │ │
│  │                                                              │ │
│  │  2. InteractiveChatTemplate                                │ │
│  │     • Session-based conversation with history              │ │
│  │     • Tool integration (user-configurable)                 │ │
│  │     • Interactive (stdin) + programmatic modes             │ │
│  │     • Memory placeholder (Phase 13 enhancement)            │ │
│  │                                                              │ │
│  │  3. DataAnalysisTemplate                                   │ │
│  │     • Stats agent + visualization agent                    │ │
│  │     • Sequential workflow pattern                           │ │
│  │                                                              │ │
│  │  4. CodeGeneratorTemplate                                  │ │
│  │     • 3-agent chain: spec → impl → test                    │ │
│  │     • Code tool integration                                 │ │
│  │                                                              │ │
│  │  5. DocumentProcessorTemplate                              │ │
│  │     • PDF extraction + transformation                       │ │
│  │     • Parallel workflow for multi-doc processing           │ │
│  │                                                              │ │
│  │  6. WorkflowOrchestratorTemplate                           │ │
│  │     • Custom parallel/sequential patterns                   │ │
│  │     • User-configurable agent/tool composition             │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
          │                 │                 │
          ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│              Existing Infrastructure (Phases 0-11b)              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐ │
│  │  Agents    │ │ Workflows  │ │   Tools    │ │     RAG      │ │
│  │ (Phase 3)  │ │ (Phase 3)  │ │ (Phase 2)  │ │  (Phase 8)   │ │
│  └────────────┘ └────────────┘ └────────────┘ └──────────────┘ │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐ │
│  │  Sessions  │ │   Hooks    │ │   Kernel   │ │  LocalLLM    │ │
│  │ (Phase 6)  │ │ (Phase 4)  │ │ (Phase 10) │ │  (Phase 11)  │ │
│  └────────────┘ └────────────┘ └────────────┘ └──────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Core Principles

**1. Templates ≠ Agent Templates**
- Existing: `llmspell-agents/src/templates/` = internal agent patterns (ToolAgentTemplate, OrchestratorAgentTemplate)
- NEW: `llmspell-templates/` = end-user workflow templates (ResearchAssistantTemplate, ChatTemplate)
- Distinction: Infrastructure vs User-facing

**2. Templates = Pre-Configured Workflows**
- Combine agents + tools + RAG + sessions into turn-key solutions
- Users execute directly via CLI or customize via Lua
- Example: Research Assistant = 4-phase workflow with 2 agents, 2 tools, RAG integration

**3. Zero New Dependencies**
- 100% existing infrastructure (Phases 0-11b)
- No external crates added
- Leverages: llmspell-agents, llmspell-workflows, llmspell-tools, llmspell-rag, llmspell-kernel, llmspell-bridge

**4. CLI-First Design**
- Direct execution: `llmspell template exec research-assistant --param topic="..."`
- Discovery: `llmspell template list`, `llmspell template search "research"`
- Inspection: `llmspell template info research-assistant`, `llmspell template schema research-assistant`

**5. Bridge Integration (16th Global)**
- Lua: `Template.list()`, `Template.info(id)`, `Template.execute(id, params)`, `Template.search(query)`
- JavaScript (Phase 15): Same API via JS bridge
- Python (future): Same API via Python bridge

### Module Structure

```
llmspell-templates/src/       (NEW CRATE - 1,200 LOC estimated)
├── core.rs                     250 LOC  - Template trait, metadata, schema
├── registry.rs                 180 LOC  - Registry with discovery/search
├── context.rs                  150 LOC  - ExecutionContext builder
├── params.rs                   120 LOC  - Parameter validation
├── output.rs                   100 LOC  - Output handling, artifacts
├── error.rs                     80 LOC  - Template-specific errors
├── builtin/
│   ├── mod.rs                   50 LOC  - Builtin registration
│   ├── research_assistant.rs   280 LOC  - Research Assistant (4-phase)
│   ├── interactive_chat.rs     220 LOC  - Interactive Chat
│   ├── data_analysis.rs        180 LOC  - Data Analysis
│   ├── code_generator.rs       200 LOC  - Code Generator
│   ├── document_processor.rs   180 LOC  - Document Processor
│   └── workflow_orchestrator.rs 150 LOC - Workflow Orchestrator
└── lib.rs                       60 LOC  - Crate exports

llmspell-cli/src/commands/
└── template.rs                 450 LOC  - CLI command handlers

llmspell-bridge/src/globals/
└── template_global.rs          380 LOC  - Lua bridge (Template global)

docs/user-guide/templates/
├── README.md                   400 LOC  - Template system overview
├── research-assistant.md       250 LOC  - Research Assistant guide
├── interactive-chat.md         220 LOC  - Interactive Chat guide
├── data-analysis.md            180 LOC  - Data Analysis guide
├── code-generator.md           200 LOC  - Code Generator guide
├── document-processor.md       180 LOC  - Document Processor guide
└── workflow-orchestrator.md    180 LOC  - Workflow Orchestrator guide

examples/templates/
├── research/                           - Research Assistant examples
│   ├── basic.lua                       - Basic usage
│   ├── cli.sh                          - CLI usage
│   └── customized.lua                  - Custom configuration
├── chat/                               - Interactive Chat examples
├── analysis/                           - Data Analysis examples
├── codegen/                            - Code Generator examples
├── documents/                          - Document Processor examples
└── orchestration/                      - Workflow Orchestrator examples
```

**Total New Code**: ~2,500 LOC (1,200 core + 450 CLI + 380 bridge + 470 docs/examples)
**Test Code**: ~1,000 LOC (unit + integration tests)
**Documentation**: ~1,610 LOC (guides + examples)

---

## Component 1: Template Trait System

### Overview

Core trait defining template contract with metadata, schema, validation, and execution. Similar to `BaseAgent` trait but specialized for pre-configured workflow patterns.

**File**: `llmspell-templates/src/core.rs`
**LOC**: 250 lines
**Tests**: 15 unit tests
**Status**: 🔄 TO IMPLEMENT

### Template Trait

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core template trait - all templates implement this
#[async_trait]
pub trait Template: Send + Sync {
    /// Template metadata (id, name, description, category, version, tags)
    fn metadata(&self) -> &TemplateMetadata;

    /// Configuration schema with parameter types and defaults
    fn config_schema(&self) -> ConfigSchema;

    /// Execute template with parameters and context
    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput, TemplateError>;

    /// Optional: Validate parameters before execution
    fn validate(&self, params: &TemplateParams) -> Result<(), ValidationError> {
        // Default: check against config_schema
        self.config_schema().validate(params)
    }

    /// Optional: Estimate execution cost (tokens, time)
    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        CostEstimate::unknown()
    }
}
```

### Template Metadata

```rust
/// Template metadata (similar to tool metadata but for workflows)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub id: String,               // "research-assistant"
    pub name: String,             // "Research Assistant"
    pub description: String,      // "Multi-source research with citations"
    pub category: TemplateCategory,
    pub version: String,          // "0.1.0"
    pub author: Option<String>,
    pub requires: Vec<String>,    // ["rag", "local-llm", "web-search"]
    pub tags: Vec<String>,        // ["research", "citations", "multi-source"]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    Research,      // Research Assistant
    Chat,          // Interactive Chat
    Analysis,      // Data Analysis
    CodeGen,       // Code Generator
    Document,      // Document Processor
    Workflow,      // Workflow Orchestrator
    Custom(String),
}
```

### Configuration Schema

```rust
/// Configuration schema with typed parameters (similar to tool schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub parameters: Vec<ConfigParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigParameter {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub validation: Option<ParameterValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<ParameterType>),
    Object(HashMap<String, ParameterType>),
    Enum(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValidation {
    MinLength(usize),
    MaxLength(usize),
    Range { min: f64, max: f64 },
    Pattern(String),  // Regex pattern
    Custom(String),   // Custom validation rule
}
```

### Template Parameters

```rust
/// Template execution parameters (type-erased key-value store)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParams {
    pub values: HashMap<String, serde_json::Value>,
}

impl TemplateParams {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.values.insert(key, value);
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, TemplateError> {
        let value = self.values.get(key)
            .ok_or_else(|| TemplateError::MissingParameter(key.to_string()))?;
        serde_json::from_value(value.clone())
            .map_err(|e| TemplateError::InvalidParameter {
                name: key.to_string(),
                expected: std::any::type_name::<T>(),
                error: e.to_string(),
            })
    }

    pub fn get_or<T: serde::de::DeserializeOwned>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }
}
```

### Execution Context

```rust
/// Execution context with state, RAG, agents (built from existing infrastructure)
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

impl ExecutionContext {
    /// Create builder for ExecutionContext
    pub fn builder() -> ExecutionContextBuilder {
        ExecutionContextBuilder::new()
    }

    // Accessor methods
    pub fn state(&self) -> &Arc<dyn StateProvider> { &self.state }
    pub fn rag_store(&self) -> Option<&Arc<dyn RAGStore>> { self.rag_store.as_ref() }
    pub fn llm_registry(&self) -> &Arc<LLMRegistry> { &self.llm_registry }
    pub fn tool_registry(&self) -> &Arc<ToolRegistry> { &self.tool_registry }
    pub fn agent_registry(&self) -> &Arc<AgentRegistry> { &self.agent_registry }
    pub fn workflow_factory(&self) -> &Arc<dyn WorkflowFactory> { &self.workflow_factory }
    pub fn session_id(&self) -> Option<&str> { self.session_id.as_deref() }
    pub fn output_dir(&self) -> Option<&Path> { self.output_dir.as_deref() }
}
```

### Template Output

```rust
/// Template execution output (result + artifacts + metrics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOutput {
    pub result: TemplateResult,
    pub artifacts: Vec<Artifact>,
    pub metadata: OutputMetadata,
    pub metrics: ExecutionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateResult {
    Text(String),
    Structured(serde_json::Value),
    File(PathBuf),
    Multiple(Vec<TemplateResult>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub content_type: String,
    pub path: PathBuf,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetadata {
    pub template_id: String,
    pub template_version: String,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub parameters: TemplateParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub duration_ms: u64,
    pub tokens_used: Option<u64>,
    pub cost_usd: Option<f64>,
    pub agents_invoked: usize,
    pub tools_called: usize,
    pub rag_queries: usize,
}
```

### Performance

| Operation | Target | Status |
|-----------|--------|--------|
| Metadata Access | <1µs | Inline |
| Schema Validation | <5ms | Compile-time checks |
| Parameter Parsing | <10ms | serde_json |
| Template Execution | Varies | Depends on template |

---

## Component 2: Template Registry

### Overview

Global registry for template discovery, search, and retrieval. Similar to `ToolRegistry` but specialized for templates with category-based discovery and keyword search.

**File**: `llmspell-templates/src/registry.rs`
**LOC**: 180 lines
**Tests**: 12 unit tests
**Status**: 🔄 TO IMPLEMENT

### Registry Implementation

```rust
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Global template registry (similar to ToolRegistry pattern)
pub struct TemplateRegistry {
    templates: RwLock<HashMap<String, Arc<dyn Template>>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self { templates: RwLock::new(HashMap::new()) }
    }

    /// Register a template (fails if ID already exists)
    pub fn register(&self, template: Arc<dyn Template>) -> Result<(), RegistryError> {
        let id = template.metadata().id.clone();
        let mut templates = self.templates.write();

        if templates.contains_key(&id) {
            return Err(RegistryError::DuplicateId(id));
        }

        templates.insert(id, template);
        Ok(())
    }

    /// Get template by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Template>> {
        self.templates.read().get(id).cloned()
    }

    /// Discover templates by category (returns metadata only)
    pub fn discover(&self, category: Option<TemplateCategory>) -> Vec<TemplateMetadata> {
        let templates = self.templates.read();
        templates.values()
            .filter(|t| {
                category.as_ref()
                    .map(|c| &t.metadata().category == c)
                    .unwrap_or(true)
            })
            .map(|t| t.metadata().clone())
            .collect()
    }

    /// Search templates by query (searches name, description, tags)
    pub fn search(&self, query: &str) -> Vec<TemplateMetadata> {
        let query_lower = query.to_lowercase();
        let templates = self.templates.read();

        templates.values()
            .filter(|t| {
                let meta = t.metadata();
                meta.name.to_lowercase().contains(&query_lower)
                    || meta.description.to_lowercase().contains(&query_lower)
                    || meta.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .map(|t| t.metadata().clone())
            .collect()
    }

    /// List all template IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.templates.read().keys().cloned().collect()
    }

    /// Count registered templates
    pub fn count(&self) -> usize {
        self.templates.read().len()
    }
}
```

### Global Registry Singleton

```rust
/// Global registry instance (similar to COMPONENT_REGISTRY pattern)
lazy_static! {
    pub static ref TEMPLATE_REGISTRY: TemplateRegistry = {
        let registry = TemplateRegistry::new();
        register_builtin_templates(&registry);
        registry
    };
}

/// Register all built-in templates (called during initialization)
fn register_builtin_templates(registry: &TemplateRegistry) {
    use crate::builtin::*;

    // Research template
    registry.register(Arc::new(ResearchAssistantTemplate::new()))
        .expect("Failed to register research-assistant template");

    // Chat template
    registry.register(Arc::new(InteractiveChatTemplate::new()))
        .expect("Failed to register interactive-chat template");

    // Data analysis template
    registry.register(Arc::new(DataAnalysisTemplate::new()))
        .expect("Failed to register data-analysis template");

    // Code generator template
    registry.register(Arc::new(CodeGeneratorTemplate::new()))
        .expect("Failed to register code-generator template");

    // Document processor template
    registry.register(Arc::new(DocumentProcessorTemplate::new()))
        .expect("Failed to register document-processor template");

    // Workflow orchestrator template
    registry.register(Arc::new(WorkflowOrchestratorTemplate::new()))
        .expect("Failed to register workflow-orchestrator template");
}
```

### Registry Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Template with ID '{0}' already registered")]
    DuplicateId(String),

    #[error("Template '{0}' not found")]
    NotFound(String),
}
```

### Test Coverage

12 comprehensive tests:
- Template registration (success, duplicate)
- Get by ID (existing, missing)
- Discovery by category (Research, Chat, All)
- Search by keyword (name, description, tags)
- List IDs
- Count
- Global registry initialization
- Builtin registration validation

---

## Component 3: Built-in Templates

### Overview

Six production-ready templates covering 90% of common use cases based on industry distribution (40% Research, 30% Chat, 15% CodeGen, 10% Data, 5% Workflow).

**Files**: `llmspell-templates/src/builtin/*.rs`
**LOC**: 1,210 lines total (6 templates)
**Tests**: 24 integration tests (4 per template)
**Status**: 🔄 TO IMPLEMENT

### Template 1: Research Assistant

**File**: `research_assistant.rs` (280 LOC)
**Category**: Research
**Complexity**: High (4-phase workflow, 2 agents, RAG integration)

#### Architecture

**4-Phase Execution**:
1. **Gather** (Parallel): Web search for sources
2. **Ingest** (Sequential): RAG indexing
3. **Synthesize** (Agent): LLM with RAG retrieval
4. **Validate** (Agent): Citation validation

```rust
pub struct ResearchAssistantTemplate {
    metadata: TemplateMetadata,
}

impl ResearchAssistantTemplate {
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "research-assistant".to_string(),
                name: "Research Assistant".to_string(),
                description: "Multi-source research with synthesis and citations".to_string(),
                category: TemplateCategory::Research,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec![
                    "web-search".to_string(),
                    "rag".to_string(),
                    "local-llm".to_string(),
                ],
                tags: vec![
                    "research".to_string(),
                    "citations".to_string(),
                    "multi-source".to_string(),
                    "synthesis".to_string(),
                ],
            },
        }
    }
}

#[async_trait]
impl Template for ResearchAssistantTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema {
            parameters: vec![
                ConfigParameter {
                    name: "topic".to_string(),
                    param_type: ParameterType::String,
                    description: "Research topic or question".to_string(),
                    required: true,
                    default: None,
                    validation: Some(ParameterValidation::MinLength(3)),
                },
                ConfigParameter {
                    name: "max_sources".to_string(),
                    param_type: ParameterType::Integer,
                    description: "Maximum number of sources to gather".to_string(),
                    required: false,
                    default: Some(json!(10)),
                    validation: Some(ParameterValidation::Range { min: 1.0, max: 50.0 }),
                },
                ConfigParameter {
                    name: "model".to_string(),
                    param_type: ParameterType::String,
                    description: "LLM model for synthesis".to_string(),
                    required: false,
                    default: Some(json!("ollama/llama3.2:3b")),
                    validation: None,
                },
                ConfigParameter {
                    name: "output_format".to_string(),
                    param_type: ParameterType::Enum(vec![
                        "markdown".to_string(),
                        "json".to_string(),
                        "html".to_string(),
                    ]),
                    description: "Output format".to_string(),
                    required: false,
                    default: Some(json!("markdown")),
                    validation: None,
                },
                ConfigParameter {
                    name: "include_citations".to_string(),
                    param_type: ParameterType::Boolean,
                    description: "Include source citations".to_string(),
                    required: false,
                    default: Some(json!(true)),
                    validation: None,
                },
            ],
        }
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput, TemplateError> {
        let start = std::time::Instant::now();

        // Extract parameters
        let topic: String = params.get("topic")?;
        let max_sources: usize = params.get_or("max_sources", 10);
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());
        let output_format: String = params.get_or("output_format", "markdown".to_string());
        let include_citations: bool = params.get_or("include_citations", true);

        tracing::info!("Phase 1: Gathering sources for topic: {}", topic);

        // PHASE 1: Parallel web search (gather sources)
        let workflow = context.workflow_factory()
            .create_parallel_workflow("research-gather", vec![
                WorkflowStep::tool("web-search", json!({
                    "query": format!("{} research papers", topic),
                    "max_results": max_sources / 2
                })),
                WorkflowStep::tool("web-search", json!({
                    "query": format!("{} technical documentation", topic),
                    "max_results": max_sources / 2
                })),
            ])?;

        let gather_result = workflow.execute(WorkflowInput::empty()).await?;
        let documents = extract_documents(&gather_result)?;

        tracing::info!("Phase 1 complete: {} sources gathered", documents.len());

        // PHASE 2: RAG ingestion (index documents)
        tracing::info!("Phase 2: Indexing sources into RAG");

        let rag_store = context.rag_store()
            .ok_or_else(|| TemplateError::MissingDependency("rag".to_string()))?;

        let session_tag = format!("research:{}", uuid::Uuid::new_v4());

        for doc in &documents {
            rag_store.ingest(doc.clone(), Some(&session_tag)).await?;
        }

        tracing::info!("Phase 2 complete: {} documents indexed", documents.len());

        // PHASE 3: Synthesis (agent with RAG retrieval)
        tracing::info!("Phase 3: Synthesizing research findings");

        let agent = context.agent_registry()
            .create_agent(AgentConfig {
                name: "research-synthesizer".to_string(),
                model: model.clone(),
                system_prompt: format!(
                    "You are a research synthesizer. Analyze the following topic and provide \
                     a comprehensive synthesis based on retrieved sources:\n\n\
                     Topic: {}\n\n\
                     Requirements:\n\
                     - Synthesize key findings from multiple sources\n\
                     - Identify consensus and contradictions\n\
                     - Provide critical analysis\n\
                     {}",
                    topic,
                    if include_citations {
                        "- Include inline citations [Source N]"
                    } else {
                        ""
                    }
                ),
                tools: vec!["rag-search".to_string()],
                ..Default::default()
            })
            .await?;

        let synthesis = agent.execute(AgentInput {
            text: format!("Synthesize research on: {}", topic),
            rag_context: Some(session_tag.clone()),
            ..Default::default()
        }).await?;

        tracing::info!("Phase 3 complete: synthesis generated");

        // PHASE 4: Validation (citation validator agent)
        tracing::info!("Phase 4: Validating citations and claims");

        let validator = context.agent_registry()
            .create_agent(AgentConfig {
                name: "citation-validator".to_string(),
                model,
                system_prompt: "You are a citation validator. Review the research synthesis and verify:\n\
                     - All claims are supported by citations\n\
                     - Citations reference actual sources\n\
                     - No hallucinated information\n\
                     Provide validation report.".to_string(),
                ..Default::default()
            })
            .await?;

        let validation = validator.execute(AgentInput {
            text: format!("Validate synthesis:\n\n{}", synthesis.text),
            ..Default::default()
        }).await?;

        tracing::info!("Phase 4 complete: validation report generated");

        // Format output
        let result = match output_format.as_str() {
            "json" => TemplateResult::Structured(json!({
                "topic": topic,
                "synthesis": synthesis.text,
                "validation": validation.text,
                "sources": documents.iter().map(|d| d.metadata()).collect::<Vec<_>>(),
            })),
            "html" => TemplateResult::Text(format_html(&synthesis.text, &documents)),
            _ => TemplateResult::Text(format_markdown(&synthesis.text, &documents)),
        };

        // Save artifacts
        let mut artifacts = vec![];
        if let Some(output_dir) = context.output_dir() {
            let synthesis_path = output_dir.join("synthesis.md");
            std::fs::write(&synthesis_path, &synthesis.text)?;
            artifacts.push(Artifact {
                name: "synthesis".to_string(),
                content_type: "text/markdown".to_string(),
                path: synthesis_path,
                metadata: HashMap::new(),
            });

            let validation_path = output_dir.join("validation.md");
            std::fs::write(&validation_path, &validation.text)?;
            artifacts.push(Artifact {
                name: "validation".to_string(),
                content_type: "text/markdown".to_string(),
                path: validation_path,
                metadata: HashMap::new(),
            });
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(TemplateOutput {
            result,
            artifacts,
            metadata: OutputMetadata {
                template_id: self.metadata.id.clone(),
                template_version: self.metadata.version.clone(),
                executed_at: chrono::Utc::now(),
                parameters: params.clone(),
            },
            metrics: ExecutionMetrics {
                duration_ms,
                tokens_used: None, // TODO: aggregate from agents
                cost_usd: None,
                agents_invoked: 2,
                tools_called: 1 + documents.len(), // web-search + RAG queries
                rag_queries: documents.len(),
            },
        })
    }
}
```

#### Usage Examples

**CLI**:
```bash
# Basic usage
llmspell template exec research-assistant --param topic="Rust async runtime design"

# Custom configuration
llmspell template exec research-assistant \
    --param topic="Temporal Knowledge Graphs" \
    --param max_sources=20 \
    --param model="ollama/llama3.1:8b" \
    --param output_format="json"

# With output directory for artifacts
llmspell template exec research-assistant \
    --param topic="Vector databases" \
    --output-dir ./research-output
```

**Lua**:
```lua
-- Basic usage
local result = Template.execute("research-assistant", {
    topic = "Rust async runtime design"
})
print(result.text)

-- Custom configuration
local result = Template.execute("research-assistant", {
    topic = "Temporal Knowledge Graphs",
    max_sources = 20,
    model = "ollama/llama3.1:8b",
    output_format = "json",
    include_citations = true
})

-- Inspect artifacts
for _, artifact in ipairs(result.artifacts) do
    print("Artifact: " .. artifact.name .. " at " .. artifact.path)
end

-- Check metrics
print("Duration: " .. result.metrics.duration_ms .. "ms")
print("Agents invoked: " .. result.metrics.agents_invoked)
print("Tools called: " .. result.metrics.tools_called)
```

### Template 2: Interactive Chat

**File**: `interactive_chat.rs` (220 LOC)
**Category**: Chat
**Complexity**: Medium (session-based, tool integration)

#### Architecture

**Session-Based Conversation**:
- Persistent conversation history
- Optional tool integration
- Interactive (stdin) + programmatic modes
- Memory placeholder (Phase 13 enhancement ready)

```rust
pub struct InteractiveChatTemplate {
    metadata: TemplateMetadata,
}

impl InteractiveChatTemplate {
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "interactive-chat".to_string(),
                name: "Interactive Chat".to_string(),
                description: "Session-based conversation with memory and context".to_string(),
                category: TemplateCategory::Chat,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["local-llm".to_string()],
                tags: vec![
                    "chat".to_string(),
                    "conversation".to_string(),
                    "session".to_string(),
                    "memory".to_string(),
                ],
            },
        }
    }
}

#[async_trait]
impl Template for InteractiveChatTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema {
            parameters: vec![
                ConfigParameter {
                    name: "model".to_string(),
                    param_type: ParameterType::String,
                    description: "LLM model for chat".to_string(),
                    required: false,
                    default: Some(json!("ollama/llama3.2:3b")),
                    validation: None,
                },
                ConfigParameter {
                    name: "system_prompt".to_string(),
                    param_type: ParameterType::String,
                    description: "System prompt for agent".to_string(),
                    required: false,
                    default: Some(json!("You are a helpful AI assistant.")),
                    validation: None,
                },
                ConfigParameter {
                    name: "max_turns".to_string(),
                    param_type: ParameterType::Integer,
                    description: "Maximum conversation turns".to_string(),
                    required: false,
                    default: Some(json!(100)),
                    validation: Some(ParameterValidation::Range { min: 1.0, max: 1000.0 }),
                },
                ConfigParameter {
                    name: "tools".to_string(),
                    param_type: ParameterType::Array(Box::new(ParameterType::String)),
                    description: "Tools available to agent".to_string(),
                    required: false,
                    default: Some(json!([])),
                    validation: None,
                },
                ConfigParameter {
                    name: "enable_memory".to_string(),
                    param_type: ParameterType::Boolean,
                    description: "Enable conversation memory (Phase 13)".to_string(),
                    required: false,
                    default: Some(json!(false)),
                    validation: None,
                },
            ],
        }
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput, TemplateError> {
        let start = std::time::Instant::now();

        // Extract parameters
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());
        let system_prompt: String = params.get_or("system_prompt", "You are a helpful AI assistant.".to_string());
        let max_turns: usize = params.get_or("max_turns", 100);
        let tool_names: Vec<String> = params.get_or("tools", Vec::new());
        let enable_memory: bool = params.get_or("enable_memory", false);

        // Create session
        let session_id = context.session_id()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("chat-{}", uuid::Uuid::new_v4()));

        tracing::info!("Starting chat session: {}", session_id);

        // Load tools
        let tools: Vec<Arc<dyn Tool>> = tool_names.iter()
            .filter_map(|name| context.tool_registry().get(name).ok())
            .collect();

        // Create chat agent
        let mut agent = context.agent_registry()
            .create_agent(AgentConfig {
                name: format!("chat-agent-{}", session_id),
                model,
                system_prompt,
                tools: tool_names,
                ..Default::default()
            })
            .await?;

        // Optional: Enable memory (Phase 13 placeholder)
        if enable_memory {
            // agent.enable_memory(&session_id)?;
            tracing::warn!("Memory not yet implemented (Phase 13)");
        }

        // Conversation loop
        let mut conversation_history = vec![];
        let mut turn_count = 0;

        tracing::info!("Chat session ready. Type 'exit' or 'quit' to end.");

        loop {
            if turn_count >= max_turns {
                tracing::info!("Maximum turns reached");
                break;
            }

            // Get user input (in CLI mode, this would be stdin)
            let user_message = if let Some(msg) = params.get::<String>("message").ok() {
                msg
            } else {
                // Interactive mode: read from stdin
                print!("> ");
                std::io::Write::flush(&mut std::io::stdout())?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            };

            if user_message.is_empty() {
                continue;
            }

            if user_message.eq_ignore_ascii_case("exit")
                || user_message.eq_ignore_ascii_case("quit") {
                break;
            }

            conversation_history.push(("user".to_string(), user_message.clone()));

            // Execute agent
            let response = agent.execute(AgentInput {
                text: user_message,
                session_id: Some(session_id.clone()),
                ..Default::default()
            }).await?;

            conversation_history.push(("assistant".to_string(), response.text.clone()));

            println!("{}", response.text);

            turn_count += 1;

            // Single-shot mode (programmatic)
            if params.values.contains_key("message") {
                break;
            }
        }

        tracing::info!("Chat session ended: {} turns", turn_count);

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(TemplateOutput {
            result: TemplateResult::Structured(json!({
                "session_id": session_id,
                "turns": turn_count,
                "conversation": conversation_history,
            })),
            artifacts: vec![],
            metadata: OutputMetadata {
                template_id: self.metadata.id.clone(),
                template_version: self.metadata.version.clone(),
                executed_at: chrono::Utc::now(),
                parameters: params.clone(),
            },
            metrics: ExecutionMetrics {
                duration_ms,
                tokens_used: None,
                cost_usd: None,
                agents_invoked: 1,
                tools_called: tools.len(),
                rag_queries: 0,
            },
        })
    }
}
```

### Template 3-6: Summary Table

| Template | Category | Agents | Tools | Workflows | RAG | Est. Time |
|----------|----------|--------|-------|-----------|-----|-----------|
| Research Assistant | Research | 2 (synthesizer, validator) | web-search, rag-search | Parallel gather | Yes | 30-60s |
| Interactive Chat | Chat | 1 (conversational) | User-configured | None | Optional | <1s/turn |
| Data Analysis | Analysis | 2 (analyzer, visualizer) | data-loader, stats | Sequential | No | 10-30s |
| Code Generator | CodeGen | 3 (spec, impl, test) | code-tools, lint | Sequential | No | 20-60s |
| Document Processor | Document | 2 (extractor, transformer) | pdf-reader, ocr | Parallel | Yes | 15-45s |
| Workflow Orchestrator | Workflow | User-configured | User-configured | Custom | Optional | Variable |

**Note**: Templates 3-6 follow same structural pattern as templates 1-2 (metadata, schema, execute) but with different agent/tool/workflow compositions. See `llmspell-templates/src/builtin/*.rs` for complete implementations.

---

## Component 4: CLI Integration

### Overview

Unified CLI interface with `llmspell template` subcommands matching existing command patterns (kernel, tool, state). Direct execution without scripting.

**File**: `llmspell-cli/src/commands/template.rs`
**LOC**: 450 lines
**Tests**: 18 integration tests
**Status**: 🔄 TO IMPLEMENT

### CLI Commands

```bash
llmspell template list [--category <cat>]
# Lists 6 built-in templates with descriptions

llmspell template info <name>
# Detailed template documentation (parameters, examples)

llmspell template exec <name> --param key=value [--output-dir <dir>]
# Direct template execution, returns structured result

llmspell template search <query>
# Keyword search across template names/descriptions/tags

llmspell template schema <name>
# JSON schema output for programmatic validation
```

### Command Structure (Clap Integration)

Add to `llmspell-cli/src/cli.rs`:

```rust
/// Template management and execution (NEW)
#[command(
    long_about = "Manage and execute production-ready AI workflow templates.

Templates are pre-configured workflows combining agents, tools, RAG, and sessions
into turn-key solutions for common use cases.

EXAMPLES:
    llmspell template list                         # List all templates
    llmspell template list --category Research     # Filter by category
    llmspell template info research-assistant      # Show template details
    llmspell template exec research-assistant --param topic=\"Rust async\" # Execute
    llmspell template search \"research\"             # Search by keyword"
)]
Template {
    #[command(subcommand)]
    command: TemplateCommands,
},

/// Template subcommands
#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// List available templates with filtering
    #[command(
        long_about = "List all registered templates in TEMPLATE_REGISTRY.

EXAMPLES:
    llmspell template list                         # List all
    llmspell template list --category Research     # Filter by category
    llmspell template list --format json           # JSON output"
    )]
    List {
        /// Filter by template category
        #[arg(long)]
        category: Option<String>, // Will be parsed to TemplateCategory

        /// Output format (overrides global format)
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Show detailed template information
    #[command(
        long_about = "Display detailed information about a specific template including schema.

EXAMPLES:
    llmspell template info research-assistant      # Show details
    llmspell template info interactive-chat --show-schema  # Include parameter schema"
    )]
    Info {
        /// Template ID to show information for
        name: String,

        /// Show detailed parameter schema
        #[arg(long)]
        show_schema: bool,
    },

    /// Execute template with parameters
    #[command(
        long_about = "Execute a template directly with specified parameters.

EXAMPLES:
    llmspell template exec research-assistant --param topic=\"Rust async\"
    llmspell template exec interactive-chat --param model=\"ollama/llama3.1:8b\"
    llmspell template exec data-analysis --param file=\"data.csv\" --output-dir ./results"
    )]
    Exec {
        /// Template ID to execute
        name: String,

        /// Parameters (key=value format, repeatable)
        #[arg(long = "param", value_parser = parse_key_val::<String, String>)]
        params: Vec<(String, String)>,

        /// Output directory for artifacts
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Search templates by capability/keywords
    #[command(
        long_about = "Search for templates by keywords, capabilities, or descriptions.

EXAMPLES:
    llmspell template search \"research\"           # Search for research templates
    llmspell template search \"chat\" \"memory\"      # Search with multiple keywords
    llmspell template search \"code\" --category CodeGen  # Search with category filter"
    )]
    Search {
        /// Search keywords (can specify multiple)
        query: Vec<String>,

        /// Filter by template category
        #[arg(long)]
        category: Option<String>,
    },

    /// Show template parameter schema
    #[command(long_about = "Display JSON schema for template parameters.

EXAMPLES:
    llmspell template schema research-assistant    # Show parameter schema
    llmspell template schema interactive-chat --format json  # JSON output")]
    Schema {
        /// Template ID to show schema for
        name: String,
    },
}
```

### Implementation

```rust
// llmspell-cli/src/commands/template.rs

use llmspell_templates::{TEMPLATE_REGISTRY, TemplateCategory, TemplateParams, ExecutionContext};
use crate::cli::{TemplateCommands, OutputFormat};
use anyhow::Result;

pub async fn handle_template_command(
    cmd: TemplateCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match cmd {
        TemplateCommands::List { category, format } => {
            let category_enum = category
                .map(|c| parse_category(&c))
                .transpose()?;

            let templates = TEMPLATE_REGISTRY.discover(category_enum);

            let output_format = format.unwrap_or(output_format);

            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&templates)?);
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("Available Templates ({}):\n", templates.len());
                    for template in templates {
                        println!("  {} ({})", template.name, template.id);
                        println!("    Category: {:?}", template.category);
                        println!("    {}", template.description);
                        println!();
                    }
                }
            }
        }

        TemplateCommands::Info { name, show_schema } => {
            let template = TEMPLATE_REGISTRY.get(&name)
                .ok_or_else(|| anyhow!("Template not found: {}", name))?;

            let meta = template.metadata();

            match output_format {
                OutputFormat::Json => {
                    let info = json!({
                        "metadata": meta,
                        "schema": if show_schema { Some(template.config_schema()) } else { None },
                    });
                    println!("{}", serde_json::to_string_pretty(&info)?);
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("Template: {} ({})", meta.name, meta.id);
                    println!("Category: {:?}", meta.category);
                    println!("Version: {}", meta.version);
                    println!("Description: {}", meta.description);
                    println!("Requires: {}", meta.requires.join(", "));
                    println!("Tags: {}", meta.tags.join(", "));

                    if show_schema {
                        let schema = template.config_schema();
                        println!("\nParameters:");
                        for param in schema.parameters {
                            println!("  - {} ({:?}): {}",
                                param.name,
                                param.param_type,
                                param.description
                            );
                            if let Some(default) = param.default {
                                println!("    Default: {}", default);
                            }
                            if param.required {
                                println!("    Required");
                            }
                        }
                    }
                }
            }
        }

        TemplateCommands::Exec { name, params, output } => {
            let template = TEMPLATE_REGISTRY.get(&name)
                .ok_or_else(|| anyhow!("Template not found: {}", name))?;

            // Build parameters
            let mut template_params = TemplateParams::new();
            for (key, value) in params {
                // Try parsing as JSON first, fallback to string
                let json_value = serde_json::from_str(&value)
                    .unwrap_or_else(|_| json!(value));
                template_params.insert(key, json_value);
            }

            // Build execution context from runtime_config
            let context = ExecutionContext::from_config(&runtime_config, output)?;

            println!("Executing template: {}", name);

            let result = template.execute(template_params, context).await?;

            println!("\n✓ Template execution complete");
            println!("  Duration: {}ms", result.metrics.duration_ms);
            println!("  Agents invoked: {}", result.metrics.agents_invoked);
            println!("  Tools called: {}", result.metrics.tools_called);

            if !result.artifacts.is_empty() {
                println!("\n  Artifacts:");
                for artifact in result.artifacts {
                    println!("    - {}: {}", artifact.name, artifact.path.display());
                }
            }

            match result.result {
                TemplateResult::Text(text) => println!("\n{}", text),
                TemplateResult::Structured(json) => {
                    println!("\n{}", serde_json::to_string_pretty(&json)?);
                }
                TemplateResult::File(path) => {
                    println!("\nOutput written to: {}", path.display());
                }
                TemplateResult::Multiple(results) => {
                    println!("\nMultiple results generated ({} total)", results.len());
                }
            }
        }

        TemplateCommands::Search { query, category } => {
            let query_str = query.join(" ");
            let templates = TEMPLATE_REGISTRY.search(&query_str);

            // Additional category filter if specified
            let templates: Vec<_> = if let Some(cat) = category {
                let category_enum = parse_category(&cat)?;
                templates.into_iter()
                    .filter(|t| &t.category == &category_enum)
                    .collect()
            } else {
                templates
            };

            match output_format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&templates)?);
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    println!("Search results for '{}' ({} found):\n", query_str, templates.len());
                    for template in templates {
                        println!("  {} ({})", template.name, template.id);
                        println!("    {}", template.description);
                        println!();
                    }
                }
            }
        }

        TemplateCommands::Schema { name } => {
            let template = TEMPLATE_REGISTRY.get(&name)
                .ok_or_else(|| anyhow!("Template not found: {}", name))?;

            let schema = template.config_schema();
            let json = serde_json::to_string_pretty(&schema)?;
            println!("{}", json);
        }
    }

    Ok(())
}

fn parse_category(s: &str) -> Result<TemplateCategory> {
    match s.to_lowercase().as_str() {
        "research" => Ok(TemplateCategory::Research),
        "chat" => Ok(TemplateCategory::Chat),
        "analysis" => Ok(TemplateCategory::Analysis),
        "codegen" => Ok(TemplateCategory::CodeGen),
        "document" => Ok(TemplateCategory::Document),
        "workflow" => Ok(TemplateCategory::Workflow),
        _ => Err(anyhow!("Unknown category: {}", s)),
    }
}
```

### Performance

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Template List | <10ms | Registry read + serialize |
| Template Info | <5ms | Metadata access |
| Template Exec | Varies | Depends on template complexity |
| Template Search | <20ms | Linear search (6 templates) |
| Template Schema | <5ms | Schema access + serialize |

---

## Component 5: Lua Bridge

### Overview

`Template` global (16th global) providing script access to template system. Matches existing global patterns (Agent, Workflow, Tool, RAG, LocalLLM).

**File**: `llmspell-bridge/src/globals/template_global.rs`
**LOC**: 380 lines
**Tests**: 12 Lua integration tests
**Status**: 🔄 TO IMPLEMENT

### Lua API

```lua
-- Template.list([category]) -> table
local templates = Template.list()
for _, t in ipairs(templates) do
    print(t.name .. " (" .. t.id .. "): " .. t.description)
end

-- Template.info(id) -> table
local info = Template.info("research-assistant")
print("Template: " .. info.name)
print("Parameters:")
for _, param in ipairs(info.parameters) do
    print("  " .. param.name .. " (" .. param.type .. "): " .. param.description)
end

-- Template.execute(id, params) -> table (async)
local result = Template.execute("research-assistant", {
    topic = "Rust async runtime design",
    max_sources = 15,
    model = "ollama/llama3.2:3b",
    output_format = "markdown"
})

print("Research complete in " .. result.metrics.duration_ms .. "ms")
print(result.text)

-- Check artifacts
for _, artifact in ipairs(result.artifacts) do
    print("Artifact: " .. artifact.name .. " at " .. artifact.path)
end

-- Template.search(query) -> table
local found = Template.search("research")
for _, t in ipairs(found) do
    print("Found: " .. t.name)
end
```

### Implementation

```rust
// llmspell-bridge/src/globals/template_global.rs

use mlua::prelude::*;
use std::sync::Arc;
use llmspell_templates::{TEMPLATE_REGISTRY, TemplateParams, ExecutionContext};

/// Inject Template global into Lua (16th global)
pub fn inject_template_global(lua: &Lua, context: Arc<GlobalContext>) -> LuaResult<()> {
    let template = lua.create_table()?;

    // Template.list([category]) -> table
    template.set("list", lua.create_function(|lua, category: Option<String>| {
        let category_enum = category
            .map(|c| parse_category(&c))
            .transpose()
            .map_err(|e| LuaError::RuntimeError(format!("Invalid category: {}", e)))?;

        let templates = TEMPLATE_REGISTRY.discover(category_enum);

        let result = lua.create_table()?;
        for (i, meta) in templates.iter().enumerate() {
            let template_table = lua.create_table()?;
            template_table.set("id", meta.id.clone())?;
            template_table.set("name", meta.name.clone())?;
            template_table.set("description", meta.description.clone())?;
            template_table.set("category", format!("{:?}", meta.category))?;
            template_table.set("version", meta.version.clone())?;

            let tags = lua.create_table()?;
            for (j, tag) in meta.tags.iter().enumerate() {
                tags.set(j + 1, tag.clone())?;
            }
            template_table.set("tags", tags)?;

            result.set(i + 1, template_table)?;
        }

        Ok(result)
    })?)?;

    // Template.info(id) -> table
    template.set("info", lua.create_function(|lua, id: String| {
        let template = TEMPLATE_REGISTRY.get(&id)
            .ok_or_else(|| LuaError::RuntimeError(format!("Template not found: {}", id)))?;

        let meta = template.metadata();
        let schema = template.config_schema();

        let info = lua.create_table()?;
        info.set("id", meta.id.clone())?;
        info.set("name", meta.name.clone())?;
        info.set("description", meta.description.clone())?;
        info.set("category", format!("{:?}", meta.category))?;
        info.set("version", meta.version.clone())?;

        // Schema parameters
        let params = lua.create_table()?;
        for (i, param) in schema.parameters.iter().enumerate() {
            let param_table = lua.create_table()?;
            param_table.set("name", param.name.clone())?;
            param_table.set("type", format!("{:?}", param.param_type))?;
            param_table.set("description", param.description.clone())?;
            param_table.set("required", param.required)?;

            if let Some(default) = &param.default {
                param_table.set("default", default.to_string())?;
            }

            params.set(i + 1, param_table)?;
        }
        info.set("parameters", params)?;

        Ok(info)
    })?)?;

    // Template.execute(id, params) -> result (async)
    let context_clone = context.clone();
    template.set("execute", lua.create_async_function(move |lua, (id, params): (String, LuaTable)| {
        let context = context_clone.clone();
        async move {
            let template = TEMPLATE_REGISTRY.get(&id)
                .ok_or_else(|| LuaError::RuntimeError(format!("Template not found: {}", id)))?;

            // Convert Lua table to TemplateParams
            let mut template_params = TemplateParams::new();
            for pair in params.pairs::<String, LuaValue>() {
                let (key, value) = pair?;
                let json_value = lua_value_to_json(value)?;
                template_params.insert(key, json_value);
            }

            // Build execution context from GlobalContext
            let exec_context = build_execution_context(&context)
                .map_err(|e| LuaError::RuntimeError(format!("Context error: {}", e)))?;

            // Execute template
            let output = template.execute(template_params, exec_context)
                .await
                .map_err(|e| LuaError::RuntimeError(format!("Execution error: {}", e)))?;

            // Convert output to Lua
            template_output_to_lua(lua, output)
        }
    })?)?;

    // Template.search(query) -> table
    template.set("search", lua.create_function(|lua, query: String| {
        let templates = TEMPLATE_REGISTRY.search(&query);

        let result = lua.create_table()?;
        for (i, meta) in templates.iter().enumerate() {
            let template_table = lua.create_table()?;
            template_table.set("id", meta.id.clone())?;
            template_table.set("name", meta.name.clone())?;
            template_table.set("description", meta.description.clone())?;
            result.set(i + 1, template_table)?;
        }

        Ok(result)
    })?)?;

    lua.globals().set("Template", template)?;
    Ok(())
}

fn template_output_to_lua(lua: &Lua, output: TemplateOutput) -> LuaResult<LuaTable> {
    let result_table = lua.create_table()?;

    // Result
    match output.result {
        TemplateResult::Text(text) => result_table.set("text", text)?,
        TemplateResult::Structured(json) => {
            let json_str = serde_json::to_string(&json)
                .map_err(|e| LuaError::RuntimeError(format!("JSON error: {}", e)))?;
            result_table.set("json", json_str)?;
        }
        TemplateResult::File(path) => result_table.set("file", path.to_string_lossy().to_string())?,
        TemplateResult::Multiple(_) => result_table.set("multiple", true)?,
    }

    // Metrics
    let metrics = lua.create_table()?;
    metrics.set("duration_ms", output.metrics.duration_ms)?;
    metrics.set("agents_invoked", output.metrics.agents_invoked)?;
    metrics.set("tools_called", output.metrics.tools_called)?;
    result_table.set("metrics", metrics)?;

    // Artifacts
    let artifacts = lua.create_table()?;
    for (i, artifact) in output.artifacts.iter().enumerate() {
        let artifact_table = lua.create_table()?;
        artifact_table.set("name", artifact.name.clone())?;
        artifact_table.set("path", artifact.path.to_string_lossy().to_string())?;
        artifacts.set(i + 1, artifact_table)?;
    }
    result_table.set("artifacts", artifacts)?;

    Ok(result_table)
}

// Helper: Convert Lua value to JSON
fn lua_value_to_json(value: LuaValue) -> LuaResult<serde_json::Value> {
    match value {
        LuaValue::Nil => Ok(serde_json::Value::Null),
        LuaValue::Boolean(b) => Ok(json!(b)),
        LuaValue::Integer(i) => Ok(json!(i)),
        LuaValue::Number(n) => Ok(json!(n)),
        LuaValue::String(s) => Ok(json!(s.to_str()?)),
        LuaValue::Table(t) => {
            // Try as array first, fallback to object
            if is_lua_array(&t) {
                let mut arr = vec![];
                for pair in t.pairs::<usize, LuaValue>() {
                    let (_, val) = pair?;
                    arr.push(lua_value_to_json(val)?);
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for pair in t.pairs::<String, LuaValue>() {
                    let (key, val) = pair?;
                    map.insert(key, lua_value_to_json(val)?);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Err(LuaError::RuntimeError(format!("Unsupported Lua type")))
    }
}

fn is_lua_array(table: &LuaTable) -> bool {
    // Simple heuristic: if all keys are consecutive integers starting from 1
    if let Ok(len) = table.raw_len() {
        len > 0
    } else {
        false
    }
}

// Helper: Build ExecutionContext from GlobalContext
fn build_execution_context(context: &Arc<GlobalContext>) -> Result<ExecutionContext> {
    ExecutionContext::builder()
        .with_state(/* from context */)
        .with_rag(/* from context */)
        .with_llm_registry(context.providers.clone())
        .with_tool_registry(context.registry.clone())
        .with_agent_registry(/* from context */)
        .with_workflow_factory(/* from context */)
        .build()
}
```

### Registration

Update `llmspell-bridge/src/globals/mod.rs`:

```rust
/// Register template global if available
fn register_template_global(
    builder: &mut GlobalRegistryBuilder,
    context: &Arc<GlobalContext>,
) {
    builder.register(Arc::new(template_global::TemplateGlobal::new(context.clone())));
}

// In create_standard_registry():
pub async fn create_standard_registry(context: Arc<GlobalContext>) -> Result<GlobalRegistry> {
    let mut builder = GlobalRegistryBuilder::new();

    // ... existing globals (15 globals)

    // Register template global (16th global)
    register_template_global(&mut builder, &context);

    builder.build()
}
```

---

## Component 6: Parameter Validation

### Overview

Type-safe parameter validation with JSON Schema-inspired rules. Provides clear error messages for user-facing template execution.

**File**: `llmspell-templates/src/params.rs`
**LOC**: 120 lines
**Tests**: 10 validation tests
**Status**: 🔄 TO IMPLEMENT

### Validation Implementation

```rust
impl ConfigSchema {
    /// Validate template parameters against schema
    pub fn validate(&self, params: &TemplateParams) -> Result<(), ValidationError> {
        let mut errors = Vec::new();

        for param_def in &self.parameters {
            // Check required parameters
            if param_def.required && !params.values.contains_key(&param_def.name) {
                errors.push(ValidationError::MissingParameter {
                    name: param_def.name.clone(),
                    description: param_def.description.clone(),
                });
                continue;
            }

            // Validate parameter if present
            if let Some(value) = params.values.get(&param_def.name) {
                if let Err(e) = self.validate_parameter(param_def, value) {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::Multiple(errors))
        }
    }

    fn validate_parameter(
        &self,
        param_def: &ConfigParameter,
        value: &serde_json::Value,
    ) -> Result<(), ValidationError> {
        // Type validation
        self.validate_type(&param_def.param_type, value)?;

        // Constraint validation
        if let Some(validation) = &param_def.validation {
            self.validate_constraint(param_def, validation, value)?;
        }

        Ok(())
    }

    fn validate_type(
        &self,
        param_type: &ParameterType,
        value: &serde_json::Value,
    ) -> Result<(), ValidationError> {
        match (param_type, value) {
            (ParameterType::String, serde_json::Value::String(_)) => Ok(()),
            (ParameterType::Integer, serde_json::Value::Number(n)) if n.is_i64() => Ok(()),
            (ParameterType::Float, serde_json::Value::Number(_)) => Ok(()),
            (ParameterType::Boolean, serde_json::Value::Bool(_)) => Ok(()),
            (ParameterType::Array(_), serde_json::Value::Array(_)) => Ok(()), // TODO: validate items
            (ParameterType::Object(_), serde_json::Value::Object(_)) => Ok(()), // TODO: validate fields
            (ParameterType::Enum(variants), serde_json::Value::String(s)) => {
                if variants.contains(s) {
                    Ok(())
                } else {
                    Err(ValidationError::InvalidEnum {
                        value: s.clone(),
                        variants: variants.clone(),
                    })
                }
            }
            _ => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", param_type),
                got: format!("{:?}", value),
            }),
        }
    }

    fn validate_constraint(
        &self,
        param_def: &ConfigParameter,
        validation: &ParameterValidation,
        value: &serde_json::Value,
    ) -> Result<(), ValidationError> {
        match validation {
            ParameterValidation::MinLength(min) => {
                if let serde_json::Value::String(s) = value {
                    if s.len() < *min {
                        return Err(ValidationError::MinLength {
                            parameter: param_def.name.clone(),
                            min: *min,
                            actual: s.len(),
                        });
                    }
                }
            }
            ParameterValidation::MaxLength(max) => {
                if let serde_json::Value::String(s) = value {
                    if s.len() > *max {
                        return Err(ValidationError::MaxLength {
                            parameter: param_def.name.clone(),
                            max: *max,
                            actual: s.len(),
                        });
                    }
                }
            }
            ParameterValidation::Range { min, max } => {
                if let serde_json::Value::Number(n) = value {
                    let val = n.as_f64().unwrap();
                    if val < *min || val > *max {
                        return Err(ValidationError::OutOfRange {
                            parameter: param_def.name.clone(),
                            min: *min,
                            max: *max,
                            actual: val,
                        });
                    }
                }
            }
            ParameterValidation::Pattern(regex) => {
                if let serde_json::Value::String(s) = value {
                    let re = regex::Regex::new(regex).map_err(|e| {
                        ValidationError::InvalidPattern {
                            pattern: regex.clone(),
                            error: e.to_string(),
                        }
                    })?;
                    if !re.is_match(s) {
                        return Err(ValidationError::PatternMismatch {
                            parameter: param_def.name.clone(),
                            pattern: regex.clone(),
                            value: s.clone(),
                        });
                    }
                }
            }
            ParameterValidation::Custom(_) => {
                // Custom validation rules (future extensibility)
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required parameter: {name} ({description})")]
    MissingParameter { name: String, description: String },

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Invalid enum value '{value}'. Valid values: {}", variants.join(", "))]
    InvalidEnum { value: String, variants: Vec<String> },

    #[error("String too short: {parameter} must be at least {min} characters (got {actual})")]
    MinLength { parameter: String, min: usize, actual: usize },

    #[error("String too long: {parameter} must be at most {max} characters (got {actual})")]
    MaxLength { parameter: String, max: usize, actual: usize },

    #[error("Value out of range: {parameter} must be between {min} and {max} (got {actual})")]
    OutOfRange { parameter: String, min: f64, max: f64, actual: f64 },

    #[error("Pattern mismatch: {parameter} must match pattern '{pattern}' (got '{value}')")]
    PatternMismatch { parameter: String, pattern: String, value: String },

    #[error("Invalid pattern: {pattern} ({error})")]
    InvalidPattern { pattern: String, error: String },

    #[error("Multiple validation errors: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Multiple(Vec<ValidationError>),
}
```

### Example Error Messages

```bash
# Missing required parameter
Error: Missing required parameter: topic (Research topic or question)

# Type mismatch
Error: Type mismatch: expected Integer, got String

# Out of range
Error: Value out of range: max_sources must be between 1 and 50 (got 100)

# Invalid enum
Error: Invalid enum value 'yaml'. Valid values: markdown, json, html

# Pattern mismatch
Error: Pattern mismatch: model must match pattern '^(ollama|candle)/' (got 'invalid/model')
```

---

## Component 7: Artifact Management

### Overview

Structured artifact generation and management for template outputs (files, reports, data). Integrates with existing `llmspell-kernel/sessions/artifact/` infrastructure.

**File**: `llmspell-templates/src/output.rs`
**LOC**: 100 lines
**Tests**: 6 artifact tests
**Status**: 🔄 TO IMPLEMENT

### Artifact Handling

```rust
impl TemplateOutput {
    /// Save all artifacts to output directory
    pub fn save_artifacts(&self, output_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        std::fs::create_dir_all(output_dir)?;

        let mut saved_paths = Vec::new();

        for artifact in &self.artifacts {
            // Copy artifact to output directory
            let dest_path = output_dir.join(artifact.path.file_name().unwrap());
            std::fs::copy(&artifact.path, &dest_path)?;
            saved_paths.push(dest_path);
        }

        // Also save metadata
        let metadata_path = output_dir.join("template_metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        std::fs::write(&metadata_path, metadata_json)?;
        saved_paths.push(metadata_path);

        // Save metrics
        let metrics_path = output_dir.join("template_metrics.json");
        let metrics_json = serde_json::to_string_pretty(&self.metrics)?;
        std::fs::write(&metrics_path, metrics_json)?;
        saved_paths.push(metrics_path);

        Ok(saved_paths)
    }

    /// Get artifact by name
    pub fn get_artifact(&self, name: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.name == name)
    }

    /// List artifact names
    pub fn artifact_names(&self) -> Vec<String> {
        self.artifacts.iter().map(|a| a.name.clone()).collect()
    }
}

impl Artifact {
    /// Create new artifact
    pub fn new(name: String, content_type: String, path: PathBuf) -> Self {
        Self {
            name,
            content_type,
            path,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to artifact
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Read artifact content as string
    pub fn read_text(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.path)
    }

    /// Read artifact content as bytes
    pub fn read_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(&self.path)
    }
}
```

### Artifact Examples

```rust
// Research Assistant artifacts
let artifacts = vec![
    Artifact::new(
        "synthesis".to_string(),
        "text/markdown".to_string(),
        PathBuf::from("/tmp/research/synthesis.md"),
    ).with_metadata("word_count".to_string(), json!(1250)),

    Artifact::new(
        "validation".to_string(),
        "text/markdown".to_string(),
        PathBuf::from("/tmp/research/validation.md"),
    ),

    Artifact::new(
        "sources".to_string(),
        "application/json".to_string(),
        PathBuf::from("/tmp/research/sources.json"),
    ).with_metadata("source_count".to_string(), json!(15)),
];

// Save all artifacts
let saved = template_output.save_artifacts(Path::new("./research-output"))?;
println!("Saved {} artifacts", saved.len());
```

---

## Integration Architecture

### Template System Integration with Existing Infrastructure

```
Template Execution Flow:
    ↓
TemplateRegistry.get(id) → Template
    ↓
Template.execute(params, context)
    ↓
ExecutionContext:
  • StateManager (Phase 5)
  • RAGStore (Phase 8)
  • LLMRegistry (Phase 11)
  • ToolRegistry (Phase 2)
  • AgentRegistry (Phase 3)
  • WorkflowFactory (Phase 3)
    ↓
Template Internal Logic:
  1. Create agents via AgentRegistry
  2. Create workflows via WorkflowFactory
  3. Execute tools via ToolRegistry
  4. Query RAG via RAGStore
  5. Manage state via StateManager
    ↓
TemplateOutput:
  • Result (text/structured/file)
  • Artifacts (files)
  • Metadata (execution info)
  • Metrics (performance)
```

### Dependency Map

```
llmspell-templates (NEW)
├── llmspell-core (traits)
├── llmspell-agents (agent creation)
├── llmspell-workflows (workflow patterns)
├── llmspell-tools (tool execution)
├── llmspell-rag (RAG operations)
├── llmspell-kernel (sessions, state)
├── llmspell-config (configuration)
└── llmspell-providers (LLM access)

llmspell-cli (commands/template.rs)
└── llmspell-templates (template execution)

llmspell-bridge (globals/template_global.rs)
└── llmspell-templates (Lua bridge)
```

**Zero New External Dependencies**: All functionality built on Phases 0-11b infrastructure.

---

## Performance Targets

### Template System Performance

| Operation | Target | Rationale | Status |
|-----------|--------|-----------|--------|
| **Registry Operations** | | | |
| Template registration | <1ms | Initialization only | Inline |
| Template lookup (get) | <100µs | HashMap access | Inline |
| Template discovery (list) | <10ms | 6 templates, filter | TBD |
| Template search (keyword) | <20ms | Linear search, 6 items | TBD |
| **Execution Overhead** | | | |
| Parameter validation | <5ms | Schema check | TBD |
| Context building | <50ms | Infrastructure setup | TBD |
| Template execution | Varies | Template-dependent | N/A |
| **Per-Template Targets** | | | |
| Research Assistant | 30-60s | 4-phase, 2 agents, RAG | TBD |
| Interactive Chat | <1s/turn | Single agent | TBD |
| Data Analysis | 10-30s | 2 agents, sequential | TBD |
| Code Generator | 20-60s | 3 agents, sequential | TBD |
| Document Processor | 15-45s | Parallel processing | TBD |
| Workflow Orchestrator | Variable | User-configured | N/A |

### Comparison vs Python Frameworks

| Metric | LLMSpell (Rust) | LangChain (Python) | Speedup |
|--------|-----------------|--------------------|---------|
| Template execution overhead | <100ms | 500-1000ms | **5-10x** |
| Memory usage | <50MB | 200-300MB | **4-6x** |
| Cold start | <200ms | 2-3s | **10-15x** |
| Hot path latency | <10ms | 50-100ms | **5-10x** |

---

## Testing Strategy

### Test Distribution

**Total: 91 automated tests**

- Core tests: 15 (template trait, metadata, schema)
- Registry tests: 12 (registration, discovery, search)
- Built-in templates: 24 (4 per template × 6 templates)
- CLI tests: 18 (list, info, exec, search, schema)
- Lua bridge tests: 12 (list, info, execute, search)
- Parameter validation: 10 (required, type, range, enum, pattern)

### Test Categories

**Unit Tests** (52 tests):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Template trait tests
    #[test]
    fn test_template_metadata() { /* ... */ }

    #[test]
    fn test_config_schema_validation() { /* ... */ }

    // Registry tests
    #[test]
    fn test_template_registration() { /* ... */ }

    #[test]
    fn test_duplicate_registration_fails() { /* ... */ }

    #[test]
    fn test_template_discovery_by_category() { /* ... */ }

    #[test]
    fn test_template_search_by_keyword() { /* ... */ }

    // Parameter validation tests
    #[test]
    fn test_required_parameter_validation() { /* ... */ }

    #[test]
    fn test_type_validation() { /* ... */ }

    #[test]
    fn test_range_validation() { /* ... */ }

    #[test]
    fn test_enum_validation() { /* ... */ }
}
```

**Integration Tests** (27 tests):
```rust
// tests/template_integration.rs

#[tokio::test]
async fn test_research_assistant_full_execution() {
    // Test complete Research Assistant workflow
    let template = TEMPLATE_REGISTRY.get("research-assistant").unwrap();

    let params = TemplateParams::from(json!({
        "topic": "Rust async",
        "max_sources": 5,
    }));

    let context = ExecutionContext::builder()
        .with_mock_rag()
        .with_mock_llm()
        .build()
        .unwrap();

    let result = template.execute(params, context).await.unwrap();

    assert!(result.metrics.duration_ms > 0);
    assert_eq!(result.metrics.agents_invoked, 2);
    assert!(!result.artifacts.is_empty());
}

#[tokio::test]
async fn test_interactive_chat_session() {
    // Test chat template with session management
    // ...
}

#[tokio::test]
async fn test_cli_template_exec() {
    // Test CLI execution
    // ...
}

#[tokio::test]
async fn test_lua_bridge_template_execute() {
    // Test Lua bridge
    // ...
}
```

**Lua Integration Tests** (12 tests):
```lua
-- tests/lua/template_test.lua

-- Test 1: Template.list()
local templates = Template.list()
assert(#templates == 6, "Should have 6 built-in templates")

-- Test 2: Template.info()
local info = Template.info("research-assistant")
assert(info.name == "Research Assistant", "Name should match")
assert(#info.parameters > 0, "Should have parameters")

-- Test 3: Template.execute()
local result = Template.execute("interactive-chat", {
    message = "Hello",
    model = "ollama/llama3.2:3b"
})
assert(result.metrics.agents_invoked == 1, "Should invoke 1 agent")

-- Test 4: Template.search()
local found = Template.search("research")
assert(#found >= 1, "Should find at least 1 research template")
```

### Test Commands

```bash
# Run all tests
cargo test --workspace --all-features

# Run template tests only
cargo test -p llmspell-templates

# Run CLI integration tests
cargo test -p llmspell-cli template

# Run Lua bridge tests
cargo test -p llmspell-bridge template_global

# Run with coverage
cargo tarpaulin --workspace --all-features --exclude-files "tests/*"
```

### Quality Gates

| Gate | Target | Status |
|------|--------|--------|
| Unit test coverage | >90% | TBD |
| Integration test coverage | >85% | TBD |
| Lua bridge tests | 100% API coverage | TBD |
| CLI tests | All commands tested | TBD |
| Documentation coverage | >95% | TBD |
| Zero clippy warnings | Required | TBD |
| Compile time | <30s (incremental) | TBD |
| Test execution time | <60s (full suite) | TBD |

---

## Operations Guide

### Production Deployment

**systemd Service** (with templates):
```ini
[Unit]
Description=LLMSpell Kernel Service with Templates
After=network.target

[Service]
Type=forking
PIDFile=/var/run/llmspell/kernel.pid
ExecStart=/usr/local/bin/llmspell kernel start --daemon --all
ExecStop=/bin/kill -TERM $MAINPID
Restart=on-failure
User=llmspell
Group=llmspell
Environment="LLMSPELL_CONFIG=/etc/llmspell/config.toml"

[Install]
WantedBy=multi-user.target
```

**Docker Deployment**:
```yaml
# docker-compose.yml
version: '3.8'
services:
  llmspell-templates:
    build: .
    command: kernel start --daemon --port 59000
    volumes:
      - ./config:/etc/llmspell
      - ./templates:/var/llmspell/templates  # Custom templates
      - logs:/var/log/llmspell
    ports:
      - "59000-59004:59000-59004"
    environment:
      - LLMSPELL_CONFIG=/etc/llmspell/config.toml
      - RUST_LOG=info
    restart: unless-stopped
```

### Monitoring

```bash
# List available templates
llmspell template list

# Check template execution
llmspell template exec research-assistant --param topic="test" --output-dir /tmp/test

# Monitor template performance
watch -n 5 'llmspell template exec interactive-chat --param message="status"'

# Template metrics
llmspell template exec research-assistant --param topic="..." | jq '.metrics'
```

### Troubleshooting

**Template Not Found**:
```bash
# Check registration
llmspell template list
# Expected: 6 templates (research-assistant, interactive-chat, ...)

# Check registry initialization
RUST_LOG=debug llmspell template list
```

**Parameter Validation Errors**:
```bash
# Get schema
llmspell template schema research-assistant

# Example valid parameters
llmspell template exec research-assistant \
    --param topic="Rust" \
    --param max_sources=10 \
    --param model="ollama/llama3.2:3b"
```

**Execution Failures**:
```bash
# Check dependencies
llmspell template info research-assistant
# Requires: web-search, rag, local-llm

# Verify infrastructure
llmspell tool list | grep web-search
llmspell model status
```

---

## Phase 13 Memory Synergy

Templates designed for seamless Phase 13 memory enhancement with A-TKG (Adaptive Temporal Knowledge Graph).

### Memory Integration Strategy

**Phase 12** (Current): Templates fully functional without memory
```rust
let chat_template = TEMPLATE_REGISTRY.get("interactive-chat").unwrap();
let result = chat_template.execute(params, context).await?;
// Works perfectly: session-based conversation with history
```

**Phase 13** (Memory Enhancement): Same templates enhanced with A-TKG
```rust
let chat_template = TEMPLATE_REGISTRY.get("interactive-chat").unwrap();
let mut params = params.clone();
params.insert("enable_memory".to_string(), json!(true)); // Just enable flag
let result = chat_template.execute(params, context).await?;
// Now: temporal context, preference learning, cross-session synthesis
```

**Zero Breaking Changes**: Adding memory is opt-in via parameter flag.

### Per-Template Memory Benefits

**1. Research Assistant + A-TKG**:
- **Temporal Context**: Track research topic evolution over time
- **Source Deduplication**: Avoid re-ingesting previously analyzed sources
- **Cross-Research Synthesis**: Reference findings from previous research sessions
- **Citation Memory**: Remember which sources were most valuable

**2. Interactive Chat + A-TKG**:
- **Conversation Continuity**: Remember context across sessions
- **Preference Learning**: Adapt to user communication style
- **Topic Threading**: Connect related conversations over time
- **Semantic Search**: Find relevant past exchanges

**3. Code Generator + A-TKG**:
- **Pattern Recognition**: Learn from previous code generations
- **Error Avoidance**: Remember and avoid past mistakes
- **Style Consistency**: Maintain coding style across generations
- **Dependency Tracking**: Remember which libraries work well together

**4. Data Analysis + A-TKG**:
- **Historical Trends**: Compare current analysis to past results
- **Anomaly Detection**: Flag deviations from historical patterns
- **Insight Persistence**: Remember important findings

**5. Document Processor + A-TKG**:
- **Document Relationships**: Link related documents across time
- **Content Evolution**: Track document changes over versions
- **Entity Tracking**: Remember entities across documents

**6. Workflow Orchestrator + A-TKG**:
- **Workflow History**: Track successful workflow patterns
- **Performance Optimization**: Adjust based on past execution times
- **Error Recovery**: Learn from past failures

### Implementation Pattern

```rust
// Phase 12 (current): No memory
impl Template for InteractiveChatTemplate {
    async fn execute(&self, params: TemplateParams, context: ExecutionContext) -> Result<TemplateOutput> {
        let enable_memory = params.get_or("enable_memory", false);

        let agent = context.agent_registry().create_agent(config).await?;

        // Phase 13 enhancement point (currently placeholder)
        if enable_memory {
            // agent.enable_memory(&session_id)?;
            tracing::warn!("Memory not yet implemented (Phase 13)");
        }

        // Rest of template logic unchanged
        // ...
    }
}
```

**Phase 13 Integration**: When A-TKG is ready, uncomment and implement `enable_memory()` - no template rewrite needed.

---

## Competitive Analysis

### LLMSpell vs. Competitors

| Feature | LLMSpell (Phase 12) | LangChain | AutoGen | CrewAI |
|---------|---------------------|-----------|---------|--------|
| **Language** | Rust | Python | Python | Python |
| **Performance** | 10-100x faster | Baseline | ~Baseline | ~Baseline |
| **Type Safety** | Compile-time | Runtime | Runtime | Runtime |
| **Local-First** | Yes (Candle) | Limited | No | Limited |
| **Multi-Language Bridges** | Lua/JS/(Python future) | Python only | Python only | Python only |
| **Template Count** | 6 (v0.12) | 50+ | ~10 | ~15 |
| **CLI Execution** | Yes | No | No | No |
| **Offline Support** | Full | Partial | No | Partial |
| **Memory System** | Phase 13 (A-TKG) | Built-in | Built-in | Built-in |
| **RAG Integration** | Native | LangChain | External | External |
| **Workflow Patterns** | Sequential/Parallel/Custom | LangGraph | Group Chat | Hierarchical |
| **Zero Dependencies** | Yes | No | No | No |
| **Compile-Time Validation** | Yes | No | No | No |

### Unique Selling Points

1. **Rust Performance**: 10-100x faster template execution than Python frameworks
2. **Compile-Time Safety**: Template validation at compile time, not runtime
3. **True Local-First**: Works 100% offline with Candle provider (no cloud dependency)
4. **Multi-Language**: Same templates accessible from Lua, JavaScript (Phase 15), Python (future)
5. **CLI-First**: Direct execution without scripting (`llmspell template exec research-assistant`)
6. **Zero Dependencies**: Self-contained, no external services required
7. **Production-Ready**: Built on v0.11.2 battle-tested infrastructure (40+ tools, RAG, LocalLLM)

### Market Positioning

**Target Users**:
- **Rust developers** building AI applications
- **Enterprise teams** requiring offline/airgapped deployments
- **Research institutions** needing reproducible workflows
- **Multi-language projects** (microservices with different languages)
- **Performance-critical applications** (real-time processing)

**Use Cases**:
- Automated research and analysis pipelines
- Local-first AI assistants
- Document processing and extraction
- Code generation and review automation
- Enterprise workflow automation

---

## Implementation Timeline

### Week 1: Core Infrastructure (Days 1-5)

**Day 1-2: Template Trait & Registry** (16 hours)
- [ ] Create `llmspell-templates` crate
- [ ] Implement `Template` trait (core.rs)
- [ ] Implement `TemplateRegistry` (registry.rs)
- [ ] Add error types (`TemplateError`, `ValidationError`)
- [ ] Unit tests for registry (register, get, discover, search)
- [ ] Integration with `llmspell-core` types

**Day 3-4: CLI Integration** (16 hours)
- [ ] Add `template` command to `llmspell-cli`
- [ ] Implement subcommands (list, info, exec, search, schema)
- [ ] Parameter parsing (key=value format)
- [ ] Output formatting (text, JSON, structured)
- [ ] Error handling with user-friendly messages
- [ ] CLI integration tests

**Day 5: Documentation & Context** (8 hours)
- [ ] Update `master-architecture-vision.md` with template system
- [ ] Create `docs/user-guide/templates/README.md`
- [ ] Document CLI usage with examples
- [ ] Update `implementation-phases.md` (Phase 12 complete)
- [ ] ExecutionContext implementation (context.rs)

### Week 2: Built-in Templates & Bridge (Days 6-10)

**Day 6-7: Research Assistant Template** (16 hours)
- [ ] Implement `ResearchAssistantTemplate`
- [ ] 4-phase execution (gather, ingest, synthesize, validate)
- [ ] Web search integration
- [ ] RAG integration
- [ ] Citation formatting (markdown/JSON/HTML)
- [ ] Integration tests

**Day 8: Interactive Chat Template** (8 hours)
- [ ] Implement `InteractiveChatTemplate`
- [ ] Session management
- [ ] Conversation history
- [ ] Tool integration
- [ ] Interactive mode (stdin/stdout) + programmatic mode
- [ ] Memory hook placeholder (Phase 13 ready)
- [ ] Integration tests

**Day 9: Additional Templates** (8 hours)
- [ ] Implement `DataAnalysisTemplate` (stats + visualization agents)
- [ ] Implement `CodeGeneratorTemplate` (spec → impl → test agent chain)
- [ ] Implement `DocumentProcessorTemplate` (PDF/OCR + transformation)
- [ ] Implement `WorkflowOrchestratorTemplate` (custom patterns)
- [ ] Unit tests for all templates

**Day 10: Lua Bridge & Finalization** (8 hours)
- [ ] Implement `inject_template_global()` in llmspell-bridge
- [ ] Template.list(), Template.info(), Template.execute(), Template.search()
- [ ] Lua ↔ Rust type conversions
- [ ] Async execution support
- [ ] Lua integration tests
- [ ] Example Lua scripts (6 templates)

**Day 11-12: Quality & Release** (Overlaps with Day 10, buffer time)
- [ ] Run `./scripts/quality/quality-check.sh` (format, clippy, tests, docs)
- [ ] Performance benchmarks (template execution overhead <100ms)
- [ ] Documentation review (API docs, user guides, examples)
- [ ] Update `RELEASE_NOTES.md` (Phase 12 / v0.12.0)
- [ ] Git commit + tag v0.12.0
- [ ] Announcement and documentation publication

### Timeline Summary

| Phase | Days | LOC | Deliverables |
|-------|------|-----|--------------|
| Core Infrastructure | 1-5 | 1,000 | Trait, Registry, CLI, ExecutionContext |
| Built-in Templates | 6-9 | 1,210 | 6 production templates |
| Lua Bridge | 10 | 380 | Template global (16th) |
| Quality & Release | 10-12 | - | Tests, docs, release |
| **Total** | **10 days** | **~2,590** | **Phase 12 complete** |

**Risk Buffer**: 2 days (20% contingency) built into Day 11-12 overlap.

---

## Success Criteria

### Functional Requirements
- ✅ 6 built-in templates implemented and tested
- ✅ CLI commands functional: list, info, exec, search, schema
- ✅ Lua bridge complete: Template.list/info/execute/search
- ✅ Template discovery works (by category, by query)
- ✅ Parameter validation with clear error messages
- ✅ Artifact generation (files, reports, outputs)

### Non-Functional Requirements
- ✅ Template execution overhead <100ms
- ✅ >90% test coverage
- ✅ >95% API documentation coverage
- ✅ Zero clippy warnings
- ✅ `./scripts/quality/quality-check.sh` passes
- ✅ Examples for all templates (CLI + Lua)

### Documentation Requirements
- ✅ User guide with examples
- ✅ Template catalog (6 templates)
- ✅ CLI usage documentation
- ✅ Lua API documentation
- ✅ Custom template creation guide (for future extension)
- ✅ Phase 12 completion in implementation-phases.md

### Adoption Metrics (Post-Release)
- **Time-to-First-Value**: <5 minutes (download → template execution → result)
- **Learning Curve**: Users modify template parameters before writing custom code
- **Retention**: Template users proceed to custom agent development
- **Community Growth**: User-contributed templates emerge

---

## Appendix

### Related Documentation
- `docs/technical/master-architecture-vision.md` - Overall architecture
- `docs/in-progress/implementation-phases.md` - Phase roadmap (Phase 12 insertion)
- `docs/in-progress/template-system-proposal.md` - Original proposal (transformed into this design doc)
- `docs/in-progress/phase-10-design-doc.md` - Phase 10 design (style reference)
- `docs/in-progress/phase-11a-design-doc.md` - Phase 11a design (consolidation phase)
- `RELEASE_NOTES_v0.11.1.md` - v0.11.1 changelog (Phase 11a complete)

### External References
- LangChain Templates: https://python.langchain.com/docs/templates
- AutoGen Patterns: https://microsoft.github.io/autogen/docs/topics/groupchat/
- CrewAI Framework: https://docs.crewai.com/
- Semantic Kernel: https://learn.microsoft.com/en-us/semantic-kernel/

### Stakeholder Sign-Off
- **Architecture Review**: ✅ APPROVED
- **User Experience Review**: ✅ APPROVED
- **Technical Feasibility**: ✅ APPROVED (100% existing infrastructure)
- **Schedule Approval**: ✅ APPROVED (Phase 12 insertion, 2 weeks)
- **Budget Approval**: ✅ APPROVED (zero new dependencies)

---

**Document Version**: 1.0 (Comprehensive Holistic Design)
**Last Updated**: October 2025
**Status**: 🔄 READY FOR IMPLEMENTATION (Phase 12 greenlit)
**Next Phase**: Phase 13 - Adaptive Memory System (5 weeks, builds on Phase 12 templates)
