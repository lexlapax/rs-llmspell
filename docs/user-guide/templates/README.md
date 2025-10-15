# LLMSpell Templates

**Production-ready AI workflow templates - From installation to productive AI in <5 minutes**

**🔗 Navigation**: [← User Guide](../) | [← Docs Hub](../../) | [Examples](../../../examples/templates/) | [API Reference](../api/lua/README.md#template-global)

---

## Overview

> **🎯 Turn-key AI Workflows**: Pre-configured templates combining agents, tools, RAG, and LocalLLM into executable solutions. Install → `llmspell template exec research-assistant` → productive AI work. Solves the "0-day retention problem" - no more "what do I do?" after installation.

**Version**: 0.12.0 | **Status**: Phase 12 Complete - 6 Templates (1 production, 5 structure complete) | **Last Updated**: October 2025

---

## 🚀 Quick Start

### CLI - Instant Execution
```bash
# List available templates
llmspell template list

# Get template info
llmspell template info research-assistant

# Execute template
llmspell template exec research-assistant \
  --param topic="Rust async programming" \
  --param max_sources=10 \
  --output-dir ./research_output
```

### Lua - Programmatic Control
```lua
-- List templates
local templates = Template.list()
for _, t in ipairs(templates) do
    print(t.name .. ": " .. t.description)
end

-- Execute template
local result = Template.execute("research-assistant", {
    topic = "Rust async programming",
    max_sources = 10
})

-- Access output
print(result.result)  -- Main result
for _, artifact in ipairs(result.artifacts) do
    print("Generated: " .. artifact.filename)
end
```

---

## 📚 Available Templates (6 Total)

### Production Templates (1)
| Template | Category | Status | Description |
|----------|----------|--------|-------------|
| [research-assistant](#1-research-assistant-production) | Research | ✅ Production | Multi-phase research with web search, analysis, synthesis |

### Structure Complete Templates (5)
| Template | Category | Status | Description |
|----------|----------|--------|-------------|
| [interactive-chat](#2-interactive-chat-structure) | Chat | 🔨 Structure | Session-based conversation with context and memory |
| [data-analysis](#3-data-analysis-structure) | Analysis | 🔨 Structure | Statistical analysis, visualization, data transformation |
| [code-generator](#4-code-generator-structure) | CodeGen | 🔨 Structure | Multi-language code generation with tests and docs |
| [document-processor](#5-document-processor-structure) | Document | 🔨 Structure | PDF/OCR extraction, transformation, translation |
| [workflow-orchestrator](#6-workflow-orchestrator-structure) | Workflow | 🔨 Structure | Custom multi-step workflows with agents/tools/templates |

---

## 1. Research Assistant (Production)

**Status**: ✅ **Production Ready** - Fully implemented, tested, and documented

### What It Does
4-phase research workflow: Discovery → Analysis → Synthesis → Validation
- Web search across multiple sources
- Content extraction and quality filtering
- Intelligent synthesis with citations
- Fact-checking and validation

### Quick Example
```bash
llmspell template exec research-assistant \
  --param topic="Climate change impacts on agriculture" \
  --param max_sources=15 \
  --param enable_validation=true
```

### Documentation
- **Full Guide**: [research-assistant.md](research-assistant.md) (608 lines)
- **Parameters**: 8 configurable (topic, max_sources, min_quality, depth, etc.)
- **Output**: Markdown research report + JSON source list
- **Performance**: ~45s for 10 sources, ~2,500 tokens
- **Examples**: 6 CLI + 6 Lua examples in guide

---

## 2. Interactive Chat (Structure)

**Status**: 🔨 **Structure Complete** - Schema and validation ready, placeholder execution

### What It Does
Session-based conversational AI with:
- Multi-turn conversation with context
- Session management and persistence
- Conversation memory and retrieval
- Streaming responses

### Quick Example
```bash
llmspell template exec interactive-chat \
  --param session_name="my-chat" \
  --param initial_prompt="Help me learn Rust"
```

### Documentation
- **Full Guide**: [interactive-chat.md](interactive-chat.md) (320 lines)
- **Parameters**: 6 configurable (session_name, system_prompt, model, etc.)
- **Output**: Conversational response + session state
- **Roadmap**: Full implementation in Phase 14

---

## 3. Data Analysis (Structure)

**Status**: 🔨 **Structure Complete** - Schema and validation ready, placeholder execution

### What It Does
Automated data analysis pipeline:
- CSV/Excel/JSON data ingestion
- Statistical analysis and correlation
- Visualization generation (charts/graphs)
- Insight extraction with LLM

### Quick Example
```bash
llmspell template exec data-analysis \
  --param data_source="sales_data.csv" \
  --param analysis_type="exploratory" \
  --param generate_visualizations=true
```

### Documentation
- **Full Guide**: [data-analysis.md](data-analysis.md) (240 lines)
- **Parameters**: 7 configurable (data_source, analysis_type, columns, etc.)
- **Output**: Analysis report + visualizations + statistics JSON
- **Roadmap**: Full implementation in Phase 14

---

## 4. Code Generator (Structure)

**Status**: 🔨 **Production Structure** - Complete workflow, placeholder LLM generation

### What It Does
3-phase code generation:
- Specification generation from requirements
- Multi-language code implementation
- Test suite and documentation generation

### Quick Example
```bash
llmspell template exec code-generator \
  --param description="Binary search tree with generic types" \
  --param language="rust" \
  --param include_tests=true
```

### Documentation
- **Full Guide**: [code-generator.md](code-generator.md) (300 lines)
- **Parameters**: 5 configurable (description, language, style_guide, etc.)
- **Languages**: rust, python, javascript, typescript, go
- **Output**: Source code + tests + documentation + specification
- **Roadmap**: LLM integration in Phase 14

---

## 5. Document Processor (Structure)

**Status**: 🔨 **Structure Complete** - Schema and validation ready, placeholder execution

### What It Does
Document extraction and transformation:
- PDF/DOCX/Image (OCR) extraction
- Format conversion and transformation
- Translation and summarization
- Batch processing

### Quick Example
```bash
llmspell template exec document-processor \
  --param documents='["invoice.pdf","receipt.pdf"]' \
  --param transformation="extract" \
  --param output_format="json"
```

### Documentation
- **Full Guide**: [document-processor.md](document-processor.md) (260 lines)
- **Parameters**: 5 configurable (documents, transformation, language, etc.)
- **Transformations**: extract, summarize, translate, convert
- **Output**: Transformed documents + structured data
- **Roadmap**: Full implementation in Phase 14

---

## 6. Workflow Orchestrator (Structure)

**Status**: 🔨 **Structure Complete** - Schema and validation ready, placeholder execution

### What It Does
Custom workflow orchestration:
- Multi-step workflow definition
- Agent/tool/template composition
- Conditional branching and loops
- Error handling and retry logic

### Quick Example
```bash
llmspell template exec workflow-orchestrator \
  --param workflow_definition='{
    "steps": [
      {"type": "template", "template": "research-assistant"},
      {"type": "template", "template": "code-generator"}
    ]
  }' \
  --param input_data='{"topic": "REST API design"}'
```

### Documentation
- **Full Guide**: [workflow-orchestrator.md](workflow-orchestrator.md) (310 lines)
- **Parameters**: 4 configurable (workflow_definition, input_data, max_steps, etc.)
- **Patterns**: Sequential, parallel, conditional, loop
- **Output**: Workflow result with all step outputs
- **Roadmap**: Full implementation in Phase 15

---

## 📖 Template User Guides

Each template has comprehensive documentation:

1. **[Research Assistant](research-assistant.md)** (608 lines)
   - ✅ Production ready
   - 4-phase workflow: Discovery, Analysis, Synthesis, Validation
   - 8 parameters, 6 CLI examples, 6 Lua examples
   - Cost estimation, troubleshooting, performance metrics

2. **[Interactive Chat](interactive-chat.md)** (320 lines)
   - 🔨 Structure complete
   - Session-based conversation with memory
   - 6 parameters, streaming support
   - Full implementation: Phase 14

3. **[Data Analysis](data-analysis.md)** (240 lines)
   - 🔨 Structure complete
   - CSV/Excel/JSON ingestion + visualization
   - 7 parameters, multiple analysis types
   - Full implementation: Phase 14

4. **[Code Generator](code-generator.md)** (300 lines)
   - 🔨 Production structure
   - 3-phase: Spec → Implementation → Tests
   - 5 languages supported, style guide enforcement
   - LLM integration: Phase 14

5. **[Document Processor](document-processor.md)** (260 lines)
   - 🔨 Structure complete
   - PDF/OCR/format conversion
   - 4 transformation types, batch processing
   - Full implementation: Phase 14

6. **[Workflow Orchestrator](workflow-orchestrator.md)** (310 lines)
   - 🔨 Structure complete
   - Agent/tool/template composition
   - 4 workflow patterns (sequential, parallel, conditional, loop)
   - Full implementation: Phase 15

---

## 🎯 Common Use Cases

### Research and Learning
```bash
# Deep research on technical topic
llmspell template exec research-assistant \
  --param topic="Rust ownership system" \
  --param max_sources=20 \
  --param depth="comprehensive"
```

### Interactive Assistance
```bash
# Start conversational session
llmspell template exec interactive-chat \
  --param session_name="learning-rust" \
  --param system_prompt="You are a Rust expert teacher"
```

### Data Insights
```bash
# Analyze sales data
llmspell template exec data-analysis \
  --param data_source="sales.csv" \
  --param analysis_type="exploratory" \
  --param generate_visualizations=true
```

### Code Generation
```bash
# Generate complete module
llmspell template exec code-generator \
  --param description="HTTP client with retry logic" \
  --param language="rust" \
  --param include_tests=true
```

### Document Processing
```bash
# Extract invoice data
llmspell template exec document-processor \
  --param documents='["invoices/*.pdf"]' \
  --param transformation="extract" \
  --param output_format="json"
```

### Custom Workflows
```bash
# Research → Code → Review workflow
llmspell template exec workflow-orchestrator \
  --param workflow_definition='{
    "steps": [
      {"type": "template", "template": "research-assistant"},
      {"type": "template", "template": "code-generator"},
      {"type": "agent", "agent": "code-reviewer"}
    ]
  }'
```

---

## 🔧 Template Discovery

### List Templates
```bash
# All templates
llmspell template list

# By category
llmspell template list --category research
llmspell template list --category chat

# With details
llmspell template list --verbose
```

### Search Templates
```bash
# By keyword
llmspell template search "data"
llmspell template search "code"

# By tags
llmspell template search --tag "research"
llmspell template search --tag "automation"
```

### Template Information
```bash
# Basic info
llmspell template info research-assistant

# With schema
llmspell template schema research-assistant
```

---

## 📊 Performance Characteristics

| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Template list | <10ms | ~0.5ms | ✅ 20x faster |
| Template info | <5ms | ~0.3ms | ✅ 16x faster |
| Template discovery | <10ms | ~1ms | ✅ 10x faster |
| Parameter validation | <5ms | ~0.1ms | ✅ 50x faster |
| ExecutionContext creation | <100ms | ~2ms | ✅ 50x faster |
| Registry search | <20ms | ~1ms | ✅ 20x faster |

### Research Assistant Performance (Production)
- **10 sources**: ~45s, ~2,500 tokens, $0.00025
- **20 sources**: ~85s, ~4,800 tokens, $0.00048
- **Parallel search**: 2-3x faster for 5+ sources
- **Quality filtering**: <100ms per source

---

## 🏗️ Architecture

### Template System Components
```
Template Trait
├── Metadata (name, description, version, category)
├── Config Schema (parameter validation)
├── Cost Estimation (token/time prediction)
├── Execute (async workflow)
└── Output (result + artifacts + metrics)

TemplateRegistry (DashMap)
├── Template storage (Arc-shared)
├── Discovery (by category, tags, query)
├── Search (keyword matching)
└── Global singleton (lazy initialization)

ExecutionContext (Builder)
├── Infrastructure (Tools, Agents, Workflows, RAG)
├── LLM Provider (ProviderManager)
├── Memory Manager (Phase 13 - placeholder)
└── Session Management

CLI Integration (5 commands)
├── template list [--category] [--verbose]
├── template info <name> [--show-schema]
├── template exec <name> --param key=value [--output dir]
├── template search <query> [--category]
└── template schema <name>

Lua Bridge (Template Global - 17th global)
├── Template.list([category]) -> TemplateMetadata[]
├── Template.info(name, [with_schema]) -> TemplateMetadata
├── Template.execute(name, params) -> TemplateOutput
├── Template.search(query, [category]) -> TemplateMetadata[]
└── Template.schema(name) -> ConfigSchema
```

### 4-Layer Bridge Pattern
```
Layer 0: Core Template (Rust trait)
  ↓
Layer 1: TemplateBridge (wrapper with Arc<TemplateRegistry>)
  ↓
Layer 2: TemplateGlobal (Lua table injection)
  ↓
Layer 3: User Script (Template.execute(...))
```

---

## 🔬 Extending Templates

### Create Custom Template

**Step 1: Implement Template Trait**
```rust
use llmspell_templates::prelude::*;

pub struct CustomTemplate;

#[async_trait]
impl Template for CustomTemplate {
    fn metadata(&self) -> TemplateMetadata {
        TemplateMetadata {
            name: "custom-template".to_string(),
            description: "My custom workflow".to_string(),
            version: "0.1.0".to_string(),
            category: TemplateCategory::Custom,
            tags: vec!["custom".to_string()],
        }
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new()
            .add_parameter(
                "input",
                ParameterSchema::new(ParameterType::String)
                    .required(true)
                    .description("Input data")
            )
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        // Your implementation
        Ok(TemplateOutput::text("Result"))
    }
}
```

**Step 2: Register Template**
```rust
let registry = TemplateRegistry::new();
registry.register(Arc::new(CustomTemplate))?;
```

**Step 3: Use Template**
```bash
llmspell template exec custom-template --param input="data"
```

### Template Development Guide

See [Template System Architecture](../../technical/template-system-architecture.md) for:
- Complete trait reference
- Parameter validation patterns
- Cost estimation strategies
- Testing with mocks
- Performance optimization
- Phase 13 memory integration

---

## 🎓 Learning Resources

### Examples
- **[Template Examples](../../../examples/templates/)** - 12 complete examples
- **CLI Examples**: In each template guide
- **Lua Examples**: In each template guide

### API Documentation
- **[Template Global](../api/lua/README.md#template-global)** - Lua API reference
- **[Template Trait](../api/rust/README.md#llmspell-templates)** - Rust API reference

### Technical Documentation
- **[Template System Architecture](../../technical/template-system-architecture.md)** (700+ lines)
  - System design and components
  - 4-layer bridge pattern
  - Performance benchmarks
  - Extension guide
  - Phase 13 integration points

---

## 🔍 Troubleshooting

### Template Not Found
```bash
# List all templates
llmspell template list

# Check exact name
llmspell template info <name>
```

### Parameter Validation Errors
```bash
# Check schema
llmspell template schema <name>

# Use correct types
--param string_value="text"
--param number_value=42
--param boolean_value=true
--param array_value='["a","b"]'
--param object_value='{"key":"value"}'
```

### Execution Errors
```bash
# Enable debug logging
RUST_LOG=llmspell_templates=debug llmspell template exec ...

# Check infrastructure requirements
llmspell tool list           # Required tools available?
llmspell provider list       # LLM provider configured?
```

### Placeholder Implementation
- **Current Status**: 5 templates have structure but placeholder execution
- **Workaround**: Use production template (research-assistant) or wait for Phase 14
- **Timeline**: Full implementations in Phase 14 (Data/Document/Code) and Phase 15 (Workflow)

---

## 📈 Roadmap

### Phase 12 (Current - Complete) ✅
- ✅ Template trait system
- ✅ Registry with discovery
- ✅ CLI integration (5 commands)
- ✅ Lua bridge (Template global)
- ✅ 1 production template (Research Assistant)
- ✅ 5 templates with structure
- ✅ Parameter validation
- ✅ Cost estimation
- ✅ 126 tests, zero warnings

### Phase 13 (Next - Memory Integration)
- Memory-enhanced templates
- Conversation history in Interactive Chat
- Research context in Research Assistant
- Cross-session learning
- Template memory configuration

### Phase 14 (Advanced Templates)
- Full Data Analysis implementation
- Full Document Processor implementation
- Full Code Generator LLM integration
- Advanced Interactive Chat features
- Template marketplace preparation

### Phase 15 (Workflow Orchestration)
- Full Workflow Orchestrator implementation
- Conditional branching and loops
- Parallel step execution
- Error recovery and retry
- Workflow debugging tools
- Visual workflow builder

---

## 🆘 Need Help?

- **Template Issues?** Check individual template guides above
- **API Reference?** See [Template Global](../api/lua/README.md#template-global)
- **Architecture?** See [Template System Architecture](../../technical/template-system-architecture.md)
- **Examples?** See [Template Examples](../../../examples/templates/)
- **General Help?** See [User Guide](../)
- **Bugs?** Report on [GitHub Issues](https://github.com/yourusername/rs-llmspell/issues)

---

**Version 0.12.0** | Phase 12 Complete - Production-Ready AI Agent Templates | [Release Notes](../../../RELEASE_NOTES_v0.12.0.md) | [Changelog](../../../CHANGELOG.md)
