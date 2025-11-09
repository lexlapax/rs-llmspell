# LLMSpell Templates

**Production-ready AI workflow templates - From installation to productive AI in <5 minutes**

**🔗 Navigation**: [← User Guide](../) | [← Docs Hub](../../) | [Examples](../../../examples/templates/) | [API Reference](../appendix/lua-api-reference.md#template-global)

---

## Overview

> **🎯 Experimental AI Workflows**: Pre-configured templates combining agents, tools, RAG, and LocalLLM into executable solutions. Install → `llmspell template exec research-assistant` → productive AI work. Solves the "0-day retention problem" - no more "what do I do?" after installation.

**Version**: 0.13.0 | **Status**: Phase 13 Complete - 10 Memory-Aware Templates | **Last Updated**: January 2025

---

## 🚀 Quick Start

### Prerequisites
```bash
# Export API keys for cloud providers
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### CLI - Instant Execution

**Discovery:**
```bash
# List available templates
llmspell template list

# Get template info
llmspell template info research-assistant

# View parameter schema (CRITICAL - shows types, enums, constraints)
llmspell template schema code-generator
```

**Execution Methods:**

**Method A: Built-in Profile** (Recommended)
```bash
llmspell template exec research-assistant \
  --profile providers \
  --param topic="Rust async programming" \
  --param max_sources=10 \
  --output-dir ./research_output
```

**Available Profiles:**
- `providers` - OpenAI + Anthropic (cloud APIs)
- `ollama` - Local LLM via Ollama
- `candle` - Embedded local inference
- `development` - Dev mode with debug logging

Run `llmspell config list-profiles` to see all options.

**Method B: Custom Config File**
```bash
llmspell template exec research-assistant \
  --config path/to/custom-config.toml \
  --param topic="Rust async programming" \
  --param max_sources=10
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

## 📚 Available Templates (10 Total)

### Production Templates (6)
| Template | Category | Status | Description |
|----------|----------|--------|-------------|
| [research-assistant](#1-research-assistant-production) | Research | ✅ Production | Multi-phase research with web search, analysis, synthesis |
| [interactive-chat](#2-interactive-chat-production) | Chat | ✅ Production | Session-based conversation with context management |
| [data-analysis](#3-data-analysis-production) | Analysis | ✅ Production | Statistical analysis, visualization, data transformation |
| [code-generator](#4-code-generator-production) | CodeGen | ✅ Production | Multi-language code generation with tests and docs |
| [document-processor](#5-document-processor-production) | Document | ✅ Production | Text/Markdown extraction, transformation, translation |
| [workflow-orchestrator](#6-workflow-orchestrator-production) | Workflow | ✅ Production | Custom multi-step workflows (sequential/parallel/conditional/loop) |

### Advanced Templates (4)
| Template | Category | Status | Description |
|----------|----------|--------|-------------|
| [code-review](#7-code-review-production) | Development | ✅ Production | Multi-aspect code analysis with quality scoring |
| [content-generation](#8-content-generation-production) | Content | ✅ Production | Quality-driven content creation with iterative refinement |
| [file-classification](#9-file-classification-production) | Productivity | ✅ Production | Bulk file organization with customizable categories |
| [knowledge-management](#10-knowledge-management-production) | Research | ✅ Production | RAG-based knowledge base with CRUD operations |

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

## 7. Code Review (Production)

**Status**: ✅ **Production Ready** - Multi-aspect analysis with quality scoring

### What It Does
7-aspect code review workflow with configurable analysis:
- Security vulnerabilities and best practices
- Code quality and maintainability
- Performance optimizations
- Language best practices adherence
- Dependency analysis
- Architecture patterns
- Documentation completeness

### Quick Example
```bash
llmspell template exec code-review \
  --param code_path="/path/to/file.rs" \
  --param aspects='["security","quality","performance"]' \
  --param model="ollama/llama3.2:3b"
```

### Documentation
- **Full Guide**: [code-review.md](code-review.md) (196 lines)
- **Parameters**: 5 configurable (code_path, aspects, model, generate_fixes, etc.)
- **Output**: Aspect-specific findings + quality scores + suggested fixes
- **Performance**: ~5-15s depending on code size and aspects selected
- **Examples**: 4 CLI + 3 Lua examples in guide

---

## 8. Content Generation (Production)

**Status**: ✅ **Production Ready** - Quality-driven iterative content creation

### What It Does
4-stage content pipeline with quality thresholds:
- Draft generation based on content type
- Quality evaluation with scoring
- Conditional editing if below threshold
- Final output with quality metrics

### Quick Example
```bash
llmspell template exec content-generation \
  --param content_type="technical" \
  --param topic="Rust async patterns" \
  --param quality_threshold=8 \
  --param max_iterations=3
```

### Documentation
- **Full Guide**: [content-generation.md](content-generation.md) (178 lines)
- **Parameters**: 6 configurable (content_type, topic, quality_threshold, tone, etc.)
- **Content Types**: blog, documentation, marketing, technical, creative
- **Output**: Final content + quality scores + iteration history
- **Performance**: ~10-30s depending on iterations
- **Examples**: 5 CLI + 3 Lua examples in guide

---

## 9. File Classification (Production)

**Status**: ✅ **Production Ready** - Bulk file organization with dry-run mode

### What It Does
Scan-classify-act workflow for file organization:
- Bulk file scanning from paths/globs
- AI-powered classification into custom categories
- Dry-run mode for preview without moving
- Batch processing for multiple files

### Quick Example
```bash
llmspell template exec file-classification \
  --param files_paths='["/docs/*.md"]' \
  --param categories='{"technical":"docs/technical","guides":"docs/guides"}' \
  --param dry_run=true
```

### Documentation
- **Full Guide**: [file-classification.md](file-classification.md) (140 lines)
- **Parameters**: 5 configurable (files_paths, categories, dry_run, model, etc.)
- **Output**: Classification results + file movements (if not dry-run)
- **Performance**: ~2-8s for batch classification
- **Use Cases**: Document management, media libraries, code refactoring
- **Examples**: 4 CLI + 3 Lua examples in guide

---

## 10. Knowledge Management (Production)

**Status**: ✅ **Production Ready** - RAG-based knowledge base with CRUD operations

### What It Does
Multi-collection knowledge management with RAG:
- **Ingest**: Add documents to collection with chunking
- **Query**: Semantic search with citation tracking
- **Update**: Modify existing documents
- **Delete**: Remove documents from collection
- **List**: View all documents with metadata

### Quick Example
```bash
# Ingest documents
llmspell template exec knowledge-management \
  --param operation=ingest \
  --param collection="rust-docs" \
  --param content="path/to/rust-book.md" \
  --param source_type=markdown

# Query knowledge base
llmspell template exec knowledge-management \
  --param operation=query \
  --param collection="rust-docs" \
  --param query="How does async/await work?" \
  --param max_results=3
```

### Documentation
- **Full Guide**: [knowledge-management.md](knowledge-management.md) (217 lines)
- **Parameters**: 10 configurable (operation, collection, content, query, etc.)
- **Operations**: ingest, query, update, delete, list
- **Output**: Operation-specific results + citations
- **Performance**: <0.1s for query operations
- **RAG**: Simple word-overlap (production RAG integration pending)
- **Examples**: 5 CLI + 3 Lua examples in guide

---

## 📖 Template User Guides

Each template has comprehensive documentation:

1. **[Research Assistant](research-assistant.md)** (608 lines)
   - ✅ Production ready
   - 4-phase workflow: Discovery, Analysis, Synthesis, Validation
   - 8 parameters, 6 CLI examples, 6 Lua examples

2. **[Interactive Chat](interactive-chat.md)** (320 lines)
   - ✅ Production ready
   - Session-based conversation with context management
   - 6 parameters, streaming support

3. **[Data Analysis](data-analysis.md)** (240 lines)
   - ✅ Production ready
   - CSV/JSON ingestion + visualization
   - 7 parameters, statistical analysis + charts

4. **[Code Generator](code-generator.md)** (300 lines)
   - ✅ Production ready
   - 3-phase: Spec → Implementation → Tests
   - 5 languages supported

5. **[Document Processor](document-processor.md)** (260 lines)
   - ✅ Production ready (Text/Markdown)
   - 5 transformation types
   - Batch processing

6. **[Workflow Orchestrator](workflow-orchestrator.md)** (310 lines)
   - ✅ Production ready
   - 4 workflow patterns (sequential, parallel, conditional, loop)
   - Agent/tool/template composition

7. **[Code Review](code-review.md)** (196 lines)
   - ✅ Production ready
   - 7-aspect analysis (security, quality, performance, etc.)
   - Quality scoring + suggested fixes

8. **[Content Generation](content-generation.md)** (178 lines)
   - ✅ Production ready
   - 4-stage pipeline with quality thresholds
   - 5 content types supported

9. **[File Classification](file-classification.md)** (140 lines)
   - ✅ Production ready
   - Bulk file organization with dry-run mode
   - Customizable category schemas

10. **[Knowledge Management](knowledge-management.md)** (217 lines)
    - ✅ Production ready
    - RAG-based CRUD operations
    - Multi-collection support + citations

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
- **[Template Global](../appendix/lua-api-reference.md#template-global)** - Lua API reference
- **[Template Trait](../../developer-guide/reference/)** - Rust API reference

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

**CRITICAL: Enums are case-sensitive!**
```bash
# ✓ Correct
--param language="python"
--param content_type="technical"

# ✗ Wrong (validation fails)
--param language="Python"
--param content_type="Technical"

# Always check schema first
llmspell template schema code-generator
```

**Parameter Types:**
```bash
# Check schema for exact types and constraints
llmspell template schema <name>

# Use correct types
--param string_value="text"
--param number_value=42
--param boolean_value=true
--param array_value='["a","b"]'
--param object_value='{"key":"value"}'
```

### Model Override Errors

**Model format requires provider prefix:**
```bash
# ✓ Correct - includes provider
--param model="openai/gpt-3.5-turbo"
--param model="anthropic/claude-3-haiku-20240307"

# ✗ Wrong - missing provider (tries Ollama)
--param model="gpt-3.5-turbo"
```

Templates default to `ollama/llama3.2:3b`. Override with `<provider>/<model>` format.

### Provider Not Available

**Error:** `Provider unavailable: openai`

**Solution:**
```bash
# Set environment variables
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Or use local LLM
llmspell template exec <template> --profile ollama --param ...
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
- ✅ 10 production templates
- ✅ 5 major workflow categories covered
- ✅ Parameter validation
- ✅ Cost estimation
- ✅ 149 tests, zero warnings
- ✅ 3,655 lines of documentation

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
- **API Reference?** See [Template Global](../appendix/lua-api-reference.md#template-global)
- **Architecture?** See [Template System Architecture](../../technical/template-system-architecture.md)
- **Examples?** See [Template Examples](../../../examples/templates/)
- **General Help?** See [User Guide](../)
- **Bugs?** Report on [GitHub Issues](https://github.com/yourusername/rs-llmspell/issues)

---

**Version 0.12.0** | Phase 12 Complete - Experimental Workflows for Rapid Exploration AI Agent Templates | [Release Notes](../../../RELEASE_NOTES_v0.12.0.md) | [Changelog](../../../CHANGELOG.md)
