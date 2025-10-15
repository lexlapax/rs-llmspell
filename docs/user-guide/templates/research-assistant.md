# Research Assistant Template

**Version:** 0.1.0
**Category:** Research
**Status:** Production Ready (Placeholder Implementation)

## Overview

The Research Assistant template is a comprehensive 4-phase workflow for academic and professional research tasks. It automates the process of gathering sources, ingesting them into a knowledge base, synthesizing findings with citations, and validating the quality of the research output.

### What It Does

The Research Assistant template orchestrates multiple AI agents and tools to:

1. **Gather** - Parallel web search to find relevant sources
2. **Ingest** - Index sources into RAG (Retrieval-Augmented Generation) store
3. **Synthesize** - Generate research report with citations using AI agent
4. **Validate** - Quality-check citations and sources with validation agent

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

if result.success then
    print("Research complete!")
    print(result.result)
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

---

## Execution Phases

### Phase 1: Gather (Web Search)

**Duration**: ~2-3s per source
**Infrastructure**: Requires web-search tool

Executes parallel web searches to find relevant sources for the research topic. Sources are ranked by relevance and limited by the `max_sources` parameter.

**Output**: Array of source documents with:
- Title
- URL
- Content excerpt
- Relevance score

### Phase 2: Ingest (RAG Indexing)

**Duration**: ~1s per source
**Infrastructure**: Requires RAG store

Ingests all gathered sources into the RAG (Retrieval-Augmented Generation) store with a unique session tag. This enables context-aware synthesis in the next phase.

**Output**: RAG ingestion confirmation with document count

### Phase 3: Synthesize (AI Agent)

**Duration**: ~5-10s
**Infrastructure**: Requires agent creation, RAG retrieval, LLM

Creates a synthesis agent that:
1. Retrieves relevant context from RAG store
2. Generates comprehensive research synthesis
3. Includes proper citations and references
4. Structures findings logically

**Output**: Research synthesis document with citations

### Phase 4: Validate (Validation Agent)

**Duration**: ~3-5s
**Infrastructure**: Requires agent creation, LLM

Creates a validation agent that:
1. Verifies all citations are present and correct
2. Checks for broken links
3. Assesses source quality
4. Generates validation report

**Output**: Citation validation report

---

## Output Formats

### Markdown (Default)

Human-readable format suitable for documentation, reports, and direct reading.

**Structure**:
```markdown
# Research Report: {topic}

---

{synthesis content}

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

if result.success then
    print("Research Duration: " .. result.metrics.duration_ms .. "ms")
    print(result.result)
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
if result.success then
    print("✓ Research complete!")
    print("  Duration: " .. result.metrics.duration_ms .. "ms")
    print("  Sources: " .. result.metrics.tools_invoked)
    print("  Agents: " .. result.metrics.agents_invoked)

    -- Save JSON output
    local file = io.open("research.json", "w")
    file:write(JSON.encode(result.result))
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

    if result.success then
        -- Save to topic-specific file
        local filename = topic:gsub("%s+", "_"):lower() .. ".md"
        local file = io.open(filename, "w")
        file:write(result.result)
        file:close()
        print("  ✓ Saved to: " .. filename)
    else
        print("  ✗ Failed: " .. result.error)
    end
end
```

---

## Cost Estimation

The template provides cost estimates before execution:

```bash
llmspell template info research-assistant --show-schema
```

### Estimated Costs (per execution)

| Sources | Tokens | Cost (USD) | Duration |
|---------|--------|------------|----------|
| 5 | ~5,500 | $0.00055 | ~18s |
| 10 | ~8,000 | $0.00080 | ~33s |
| 20 | ~13,000 | $0.00130 | ~63s |
| 50 | ~28,000 | $0.00280 | ~153s |

**Assumptions**:
- Local LLM pricing: $0.10 per 1M tokens
- ~500 tokens per source for RAG ingestion
- ~2000 tokens for synthesis
- ~1000 tokens for validation
- ~2s per source for gathering
- ~1s per source for ingestion
- ~5s for synthesis
- ~3s for validation

**Note**: Actual costs and duration depend on model, source complexity, and infrastructure performance.

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

**Cause**: Web search tool is not available in the current environment.

**Solution**: Ensure web-search tool is enabled in your LLMSpell configuration.

Check available infrastructure:
```bash
llmspell tool list
llmspell template info research-assistant
```

#### Warning: "Using placeholder sources"

**Cause**: Web search integration is not yet fully implemented.

**Status**: This is expected behavior in Phase 12.3. Full integration will be completed in later phases.

**Workaround**: The template will generate placeholder sources for testing purposes.

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

## Advanced Usage

### Integration with Other Templates

Combine Research Assistant with other templates for complex workflows:

```lua
-- Lua example: Research + Code Generation
local research = Template.execute("research-assistant", {
    topic = "Async error handling patterns in Rust"
})

if research.success then
    -- Use research synthesis as input for code generation
    local code = Template.execute("code-generator", {
        description = research.result,
        language = "rust",
        include_tests = true
    })

    if code.success then
        print("Generated code based on research!")
    end
end
```

### Custom Model Configuration

Use different models for synthesis vs validation:

**Note**: Current implementation uses same model for both phases. This feature will be added in a future release.

---

## Requirements

### Infrastructure Dependencies

- **web-search**: Web search tool for source gathering
- **rag**: RAG store for document indexing and retrieval
- **local-llm**: Local LLM provider for agent execution

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

## Roadmap

### Current Status (Phase 12.3)

✅ Template core implementation
✅ Parameter validation
✅ Output formatting (markdown, JSON, HTML)
✅ Cost estimation
⏳ Web search integration (placeholder)
⏳ RAG integration (placeholder)
⏳ Agent synthesis (placeholder)
⏳ Citation validation (placeholder)

### Future Enhancements

**Phase 13** - Adaptive Memory Integration:
- Remember previous research sessions
- Build knowledge graph across topics
- Suggest related research based on history

**Phase 14+** - Additional Features:
- Custom source filtering (by domain, date, type)
- Multi-language support
- Export to academic citation formats (BibTeX, APA, MLA)
- Collaborative research (multi-user sessions)

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [CLI Reference](../../cli/template-commands.md)
- [Lua Template API](../../api/lua/template-global.md)
- [RAG Integration Guide](../../rag-integration.md)
- [Local LLM Configuration](../local-llm.md)

---

## Support

Having issues? Check:

1. [Troubleshooting Guide](../troubleshooting.md)
2. [GitHub Issues](https://github.com/lexlapax/rs-llmspell/issues)
3. [Community Forum](https://github.com/lexlapax/rs-llmspell/discussions)

---

**Last Updated**: Phase 12.3 (Research Assistant Template Implementation)
**Next Review**: Phase 13 (Adaptive Memory Integration)
