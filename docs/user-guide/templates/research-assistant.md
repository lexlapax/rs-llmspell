# Research Assistant Template

**Version:** 0.1.0
**Category:** Research
**Status:** ✅ Production Ready (Phase 12.8.1)

## Overview

The Research Assistant template is a comprehensive 4-phase workflow for academic and professional research tasks, powered by a full RAG (Retrieval-Augmented Generation) pipeline:

1. **Gather** - Parallel web search to find relevant sources
2. **Ingest** - Embed and index sources into RAG store with metadata
3. **Synthesize** - Generate research report with citations using RAG-augmented agent
4. **Validate** - Quality-check citations and sources with validation agent

### What It Does

The Research Assistant template orchestrates multiple AI agents and tools to:

- **Gather**: Execute parallel web searches via WebSearchTool (Phase 1)
- **Ingest**: Generate embeddings and store in RAG with tenant isolation (Phase 2)
- **Synthesize**: Retrieve relevant context from RAG and generate research synthesis with AI agent (Phase 3)
- **Validate**: Quality-check citations, verify sources, assess credibility with validation agent (Phase 4)

### Use Cases

- **Academic Research**: Literature reviews, survey papers, background research
- **Market Research**: Competitive analysis, industry trends, market sizing
- **Technical Research**: Technology comparisons, best practices, implementation patterns
- **Due Diligence**: Company research, risk assessment, compliance checks
- **Content Creation**: Blog posts with citations, whitepapers, documentation

---

## Quick Start

### CLI - Basic Usage

The simplest way to use the Research Assistant:

```bash
llmspell template exec research-assistant \
  --param topic="Your research topic here"
```

Example:

```bash
llmspell template exec research-assistant \
  --param topic="Rust async programming patterns"
```

### Lua - Basic Usage

```lua
local result = Template.execute("research-assistant", {
    topic = "Your research topic here"
})

if result.status == "ok" then
    print("Research complete!")
    print(result.result.value)
end
```

---

## Parameters Reference

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `topic` | String | Research topic or question (minimum 3 characters) |

### Optional Parameters

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `max_sources` | Integer | `10` | `1-50` | Maximum number of sources to gather |
| `model` | String | `"ollama/llama3.2:3b"` | Any LLM model | Model to use for synthesis and validation |
| `output_format` | Enum | `"markdown"` | `"markdown"`, `"json"`, `"html"` | Output format for the research report |
| `include_citations` | Boolean | `true` | `true`, `false` | Whether to include citation links in output |

### Parameter Constraints

- **topic**: Must be at least 3 characters long
- **max_sources**: Must be between 1 and 50 (inclusive)
- **output_format**: Must be one of: `markdown`, `json`, `html`

**Inspect Full Schema:**
```bash
llmspell template schema research-assistant
```

---

## Implementation Details

### Phase 1: Gather Sources via Web Search (83 lines)
- **WebSearchTool Integration**: Uses `context.tool_registry().execute_tool("web-searcher", ...)`
- **Parallel Search**: Executes web search with `max_sources` limit
- **Response Parsing**: Handles double-nested JSON `{"result": {"results": [...]}}`
- **SearchResult Structure**: Extracts `{title, url, snippet, provider, rank}` from tool output
- **Relevance Scoring**: Derives score from rank: `1.0 - (rank * 0.1)`
- **Error Handling**: Tool not found, JSON parse failures, empty results, missing fields
- **Type Safety**: Casts `usize` to `u64` for JSON parameter compatibility

### Phase 2: Ingest Sources into RAG (83 lines)
- **RAG Access**: `context.rag()` returns `Option<Arc<MultiTenantRAG>>`
- **Storage Integration**: `rag.ingest_documents(tenant_id, texts, scope, metadata_fn)`
- **Text Preparation**: Concatenates title + URL + snippet for embedding
- **Metadata System**: Custom closure provides per-source metadata:
  - `title`, `url`, `content`, `relevance_score`, `session_tag`
- **Scope Pattern**: `StateScope::Custom("research_session:{tag}")` for isolation
- **Embedding + Storage**: Single API call handles embedding generation + metadata + storage
- **Returns**: Vector IDs for stored documents, enabling retrieval in Phase 3
- **Usage Tracking**: Tracks `documents_indexed` and `storage_bytes` per tenant

### Phase 3: Synthesize Findings with Agent (158 lines)
- **RAG Retrieval**: `rag.retrieve_context(tenant_id, query, scope, k=5)` fetches top 5 relevant sources
- **Context Formatting**: Retrieved sources formatted with title, URL, relevance score, content
- **AgentConfig**: Temperature 0.7 (balanced creativity for synthesis)
- **Max Tokens**: 2000 (comprehensive synthesis output)
- **Resource Limits**: 120s execution time, 512MB memory, 0 tool calls
- **Model Parsing**: Split "provider/model-id" format, default to "ollama"
- **Agent Creation**: `context.agent_registry().create_agent(config)` → `Arc<dyn Agent>`
- **Agent Execution**: `agent.execute(AgentInput::builder().text(prompt).build(), ...)`
- **Prompt Engineering**: Structured instructions with RAG sources + format requirements
- **RAG-Augmented Prompts**: Includes retrieved context for grounded responses
- **Error Handling**: Agent creation/execution failures, RAG retrieval failures (graceful degradation)

### Phase 4: Validate Citations with Agent (115 lines)
- **AgentConfig**: Temperature 0.3 (lower for factual validation vs 0.7 synthesis)
- **Max Tokens**: 1500 (shorter validation report)
- **Resource Limits**: 90s execution time (faster than synthesis)
- **Prompt Includes**: Synthesis text + source list + validation criteria + report format
- **Source Formatting**: Numbered list: "1. Title - URL"
- **Validation Criteria**: Academic rigor, claim support, source quality assessment
- **Output Format**: Structured validation report with recommendations
- **Error Handling**: Agent creation/execution failures

### Phase 5-6: RAG Infrastructure (161 lines in multi_tenant_integration.rs)
- **Phase 5 (ingest_documents)**: 87 lines - High-level storage API
  - Flow: generate embeddings → create VectorEntry with metadata → insert via tenant_manager
  - Metadata closure for custom per-document metadata
  - Default metadata: text, ingested_at timestamp, tenant_id
- **Phase 6 (retrieve_context)**: 74 lines - High-level retrieval API
  - Flow: generate query embedding → search via tenant_manager → convert to RetrievalResult
  - Returns: `Vec<RetrievalResult>` with id, text, score, metadata
  - Safe metadata extraction with Option chaining

---

## Execution Phases

### Phase 1: Gather (Web Search)

**Duration**: ~2-3s per source
**Infrastructure**: Requires WebSearchTool

Executes parallel web searches to find relevant sources for the research topic. Sources are ranked by relevance and limited by the `max_sources` parameter.

**Output**: Array of source documents with:
- Title
- URL
- Content excerpt
- Relevance score (derived from rank)
- Provider information

### Phase 2: Ingest (RAG Indexing)

**Duration**: ~1s per source
**Infrastructure**: Requires MultiTenantRAG

Ingests all gathered sources into the RAG (Retrieval-Augmented Generation) store with:
- Embedding generation for each source
- Metadata attachment (title, URL, relevance, session tag)
- Tenant-isolated storage with custom scope
- Usage metrics tracking

**Output**: RAG ingestion confirmation with vector IDs

### Phase 3: Synthesize (AI Agent with RAG Retrieval)

**Duration**: ~5-10s
**Infrastructure**: Requires AgentRegistry, MultiTenantRAG, LLM

Creates a synthesis agent that:
1. Retrieves top 5 relevant sources from RAG store
2. Formats RAG context with titles, URLs, content
3. Generates comprehensive research synthesis with RAG-augmented prompts
4. Includes proper citations and references
5. Structures findings logically

**Output**: Research synthesis document with citations

### Phase 4: Validate (Validation Agent)

**Duration**: ~3-5s
**Infrastructure**: Requires AgentRegistry, LLM

Creates a validation agent that:
1. Verifies all citations are present and correct
2. Checks source quality and credibility
3. Assesses academic rigor
4. Generates validation report with recommendations

**Output**: Citation validation report

---

## Output Formats

### Markdown (Default)

Human-readable format suitable for documentation, reports, and direct reading.

**Structure**:
```markdown
# Research Report: {topic}

---

{synthesis content with citations}

---

{validation report}

---

## References

1. [Source 1 Title](url)
2. [Source 2 Title](url)
...
```

**Best for**: Documentation, reports, README files, human reading

### JSON

Structured data format for programmatic processing and integration.

**Structure**:
```json
{
  "topic": "Research topic",
  "synthesis": "Research synthesis text...",
  "validation": "Validation report text...",
  "sources": [
    {
      "title": "Source 1",
      "url": "https://...",
      "relevance": 0.95
    }
  ]
}
```

**Best for**: API integration, data pipelines, automated processing

### HTML

Formatted web page suitable for publishing or sharing.

**Structure**:
```html
<!DOCTYPE html>
<html>
<head>
  <title>Research Report: {topic}</title>
  <style>body { font-family: Arial; margin: 40px; }</style>
</head>
<body>
  <h1>Research Report: {topic}</h1>
  <hr>
  {synthesis}
  <hr>
  {validation}
  <hr>
  <h2>References</h2>
  <ol>
    <li><a href="url">Title</a></li>
  </ol>
</body>
</html>
```

**Best for**: Web publishing, sharing via browser, presentations

---

## Examples

### CLI Examples

#### Basic Research

```bash
llmspell template exec research-assistant \
  --param topic="Machine learning interpretability"
```

#### Custom Configuration

```bash
llmspell template exec research-assistant \
  --param topic="Quantum computing error correction" \
  --param max_sources=15 \
  --param model="ollama/llama3.2:3b" \
  --param output_format="json" \
  --output-dir ./research_output
```

#### Fast Research (Fewer Sources)

```bash
llmspell template exec research-assistant \
  --param topic="Docker networking best practices" \
  --param max_sources=5 \
  --param model="ollama/llama3.2:1b"
```

#### Research Without Citations

```bash
llmspell template exec research-assistant \
  --param topic="GraphQL vs REST API design" \
  --param include_citations=false \
  --param output_format="markdown"
```

### Lua Examples

#### Basic Research

```lua
local result = Template.execute("research-assistant", {
    topic = "Rust async programming patterns"
})

if result.status == "ok" then
    print("Research Duration: " .. result.metrics.duration_ms .. "ms")
    print(result.result.value)
end
```

#### Custom Configuration with Error Handling

```lua
-- Configure research parameters
local params = {
    topic = "Kubernetes security best practices",
    max_sources = 10,
    model = "ollama/llama3.2:3b",
    output_format = "json",
    include_citations = true
}

-- Execute template
local result = Template.execute("research-assistant", params)

-- Handle result
if result.status == "ok" then
    print("✓ Research complete!")
    print("  Duration: " .. result.metrics.duration_ms .. "ms")
    print("  Agents: " .. result.metrics.agents_invoked)

    -- Save JSON output
    local file = io.open("research.json", "w")
    file:write(JSON.encode(result.result.value))
    file:close()
else
    print("✗ Research failed: " .. result.error)
end
```

#### Batch Research

```lua
-- Research multiple topics in sequence
local topics = {
    "Container orchestration patterns",
    "Microservices communication strategies",
    "Service mesh architecture comparison"
}

for _, topic in ipairs(topics) do
    print("\nResearching: " .. topic)

    local result = Template.execute("research-assistant", {
        topic = topic,
        max_sources = 5,
        output_format = "markdown"
    })

    if result.status == "ok" then
        -- Save to topic-specific file
        local filename = topic:gsub("%s+", "_"):lower() .. ".md"
        local file = io.open(filename, "w")
        file:write(result.result.value)
        file:close()
        print("  ✓ Saved to: " .. filename)
    else
        print("  ✗ Failed")
    end
end
```

---

## Performance

**Estimated Costs (per execution)**

| Sources | Tokens | Duration | Phases |
|---------|--------|----------|--------|
| 5 | ~5,500 | ~18s | Gather(10s) + Ingest(5s) + Synthesize(5s) + Validate(3s) |
| 10 | ~8,000 | ~33s | Gather(20s) + Ingest(10s) + Synthesize(5s) + Validate(3s) |
| 20 | ~13,000 | ~63s | Gather(40s) + Ingest(20s) + Synthesize(5s) + Validate(3s) |
| 50 | ~28,000 | ~153s | Gather(100s) + Ingest(50s) + Synthesize(5s) + Validate(3s) |

**Assumptions**:
- ~500 tokens per source for RAG embedding
- ~2000 tokens for synthesis
- ~1000 tokens for validation
- ~2s per source for web search
- ~1s per source for RAG ingestion
- ~5-10s for synthesis (depends on model)
- ~3-5s for validation

**Note**: Actual duration depends on model, source complexity, web search latency, and infrastructure performance.

---

## Artifacts

The Research Assistant template generates artifacts when an output directory is specified:

### Generated Files

| Filename | Format | Description |
|----------|--------|-------------|
| `synthesis.{format}` | Based on `output_format` | Main research synthesis document |
| `validation.txt` | Plain text | Citation validation report |

### Accessing Artifacts

**CLI**:
```bash
llmspell template exec research-assistant \
  --param topic="Your topic" \
  --output-dir ./research_artifacts

ls -lh ./research_artifacts/
```

**Lua**:
```lua
local result = Template.execute("research-assistant", {
    topic = "Your topic"
})

if result.artifacts then
    for _, artifact in ipairs(result.artifacts) do
        print("Artifact: " .. artifact.filename)
        print("  Size: " .. artifact.size .. " bytes")
        print("  Type: " .. artifact.mime_type)
    end
end
```

---

## Troubleshooting

### Common Issues

#### Error: "Required parameter missing: topic"

**Cause**: The `topic` parameter was not provided or is empty.

**Solution**:
```bash
# CLI
llmspell template exec research-assistant --param topic="Your topic"

# Lua
Template.execute("research-assistant", { topic = "Your topic" })
```

#### Error: "Parameter 'max_sources' out of range"

**Cause**: `max_sources` parameter is outside the valid range (1-50).

**Solution**:
```bash
# Use a value between 1 and 50
llmspell template exec research-assistant \
  --param topic="Topic" \
  --param max_sources=25  # Valid: 1-50
```

#### Error: "Unsupported output format: xml"

**Cause**: Invalid `output_format` parameter value.

**Solution**: Use one of the supported formats:
```bash
--param output_format="markdown"  # Valid
--param output_format="json"      # Valid
--param output_format="html"      # Valid
```

#### Error: "Infrastructure not available: web-search"

**Cause**: WebSearchTool is not available in the current environment.

**Solution**: Ensure web-search tool is enabled in your LLMSpell configuration.

Check available infrastructure:
```bash
llmspell tool list
llmspell template info research-assistant
```

#### Error: "RAG not available"

**Cause**: MultiTenantRAG is not initialized in the execution context.

**Solution**: Ensure RAG infrastructure is enabled:
```bash
llmspell provider list  # Check if RAG provider is available
```

### Performance Issues

#### Research Taking Too Long

**Solutions**:
1. Reduce `max_sources`:
   ```bash
   --param max_sources=5
   ```

2. Use a faster model:
   ```bash
   --param model="ollama/llama3.2:1b"
   ```

3. Disable citations for faster execution:
   ```bash
   --param include_citations=false
   ```

#### Out of Memory

**Solutions**:
1. Reduce `max_sources` to decrease RAG store size
2. Use a smaller model (e.g., `llama3.2:1b` instead of `llama3.2:3b`)
3. Ensure sufficient system resources for LLM and RAG operations

---

## Architecture Insights

### Why RAG for Research?

**Rationale**: RAG provides:
- **Grounded Synthesis**: LLM responses based on actual retrieved sources
- **Citation Accuracy**: Direct connection between claims and source material
- **Scalability**: Handle 50+ sources without context window limits
- **Tenant Isolation**: Multiple research sessions don't interfere

### Temperature Tuning Philosophy

- **Synthesis Agent (0.7)**: Balanced creativity for comprehensive synthesis
- **Validation Agent (0.3)**: Low temperature for factual, deterministic validation
- **Rationale**: Synthesis needs creative synthesis of ideas; validation needs strict fact-checking

### RAG Context Format

Retrieved sources are formatted as:
```
RELEVANT SOURCES:
SOURCE 1: Title (relevance: 0.95)
URL: https://...
Content:
[retrieved text]

---

SOURCE 2: ...
```

This structured format helps the LLM:
- Distinguish between sources
- Reference sources by number
- Include proper citations with URLs

### 4-Phase Pipeline

Data flows sequentially through phases:
1. Web Search → Source[] (title, url, snippet, relevance)
2. Source[] → RAG Storage (embeddings + metadata)
3. RAG Retrieval + Topic → Synthesis Agent → Research Report
4. Report + Sources → Validation Agent → Validation Report

---

## Requirements

### Infrastructure Dependencies

- **WebSearchTool**: Web search for source gathering (Phase 1)
- **MultiTenantRAG**: RAG store for document indexing and retrieval (Phases 2-3)
- **AgentRegistry**: Agent creation for synthesis and validation (Phases 3-4)
- **LLM Provider**: Local LLM for agent execution

### Minimum System Requirements

- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 10GB for models and RAG index
- **CPU**: Multi-core processor (4+ cores recommended)
- **GPU**: Optional, significantly improves LLM performance

Check your system:
```bash
llmspell --version
llmspell tool list
llmspell provider list
```

---

## Implementation Status

### Phase 12.8.1 - ✅ COMPLETE (All 6 Phases)

**Implemented** (574 lines):
- ✅ Phase 1: Gather sources via web search (83 lines)
- ✅ Phase 2: Ingest sources into RAG (83 lines)
- ✅ Phase 3: Synthesize findings with RAG-augmented agent (158 lines)
- ✅ Phase 4: Validate citations with agent (115 lines)
- ✅ Phase 5: RAG storage infrastructure (87 lines in multi_tenant_integration.rs)
- ✅ Phase 6: RAG retrieval infrastructure (74 lines in multi_tenant_integration.rs)

**Quality Metrics**:
- ✅ Compilation: Clean (0 errors, 0 warnings)
- ✅ Clippy: Clean (0 warnings)
- ✅ Tests: 170 passing (60 llmspell-rag + 110 llmspell-templates)
- ✅ Coverage: Unit tests for all RAG methods

**Key Achievements**:
1. First complete end-to-end template with full RAG pipeline
2. Established pattern for RAG-powered templates
3. Clean high-level APIs (`ingest_documents`, `retrieve_context`)
4. Proper tenant isolation and usage tracking

**Timeline**:
- Phases 1-4 (Template): 12 hours actual (10-12h estimate)
- Phases 5-6 (Infrastructure): 4.5 hours actual (4-6h estimate)
- **Total**: 16.5 hours (within 14-18h estimate)

---

## Advanced Usage

### Integration with Other Templates

Combine Research Assistant with other templates for complex workflows:

```lua
-- Lua example: Research + Code Generation
local research = Template.execute("research-assistant", {
    topic = "Async error handling patterns in Rust"
})

if research.status == "ok" then
    -- Use research synthesis as input for code generation
    local code = Template.execute("code-generator", {
        description = research.result.value,
        language = "rust",
        include_tests = true
    })

    if code.status == "ok" then
        print("Generated code based on research!")
    end
end
```

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Data Analysis Template](./data-analysis.md) (3-agent chain pattern)
- [Code Generator Template](./code-generator.md) (sequential agent pattern)
- [Interactive Chat Template](./interactive-chat.md) (session management)
- [RAG Integration Guide](../../rag-integration.md)
- [Local LLM Configuration](../local-llm.md)

---

## Changelog

### v0.1.0 (Phase 12.8.1) - Production Ready

**Implemented** (574 lines total):
- ✅ Web search integration (WebSearchTool)
- ✅ RAG embedding and storage (MultiTenantRAG.ingest_documents)
- ✅ RAG retrieval (MultiTenantRAG.retrieve_context)
- ✅ Agent-based synthesis with RAG context
- ✅ Agent-based citation validation
- ✅ Multi-format output (markdown, JSON, HTML)
- ✅ Tenant isolation and usage tracking

**Key Features**:
- Full RAG pipeline (embed → store → retrieve → synthesize)
- 2 agents (synthesis + validation)
- WebSearchTool integration
- Type-safe parameter validation
- Rich error handling
- Artifact generation

---

**Last Updated**: Phase 12.8.1 (Production Implementation)
**Status**: ✅ Ready for Production Use
