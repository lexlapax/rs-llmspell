# Document Intelligence System

A comprehensive knowledge extraction and Q&A system that processes documents, builds semantic knowledge structures, and provides intelligent query interfaces using llmspell's advanced workflow capabilities.

## Overview

The Document Intelligence System demonstrates:
- **4-Phase Architecture**: Ingestion → Processing → Knowledge Building → Q&A Interface
- **Loop Processing**: Iterates through multiple documents for comprehensive analysis
- **Parallel Ingestion**: Simultaneous document loading, text extraction, and metadata parsing
- **Conditional Q&A Interface**: Routes between question answering and document analysis
- **8 Specialized Agents**: Most complex agent system in the application suite
- **Blueprint v2.0 Compliant**: Production-grade knowledge management patterns

## Prerequisites

### Required
- llmspell built and available (`cargo build --release`)
- At least one of:
  - OpenAI API key: `export OPENAI_API_KEY="sk-..."`
  - Anthropic API key: `export ANTHROPIC_API_KEY="sk-ant-..."`

### Optional
- Both API keys for multi-provider functionality
- Vector database for production embeddings (future enhancement)
- Graph database for knowledge graph storage (future enhancement)

## Quick Start

### 1. Basic Execution (No API Keys)
```bash
# Runs with simulated agents and basic document processing
./target/debug/llmspell run examples/script-users/applications/document-intelligence/main.lua
```

### 2. With Configuration File
```bash
# Uses the provided config.toml for provider settings
LLMSPELL_CONFIG=examples/script-users/applications/document-intelligence/config.toml \
  ./target/debug/llmspell run examples/script-users/applications/document-intelligence/main.lua
```

### 3. Full Production Mode
```bash
# Set API keys for real intelligence extraction
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Run with full capabilities
./target/debug/llmspell run examples/script-users/applications/document-intelligence/main.lua
```

## Architecture

### Workflow Structure

```
Main Intelligence Workflow (Sequential)
├── Phase 1: Document Ingestion (Parallel)
│   ├── Load Documents (Tool)
│   ├── Extract Text (Tool) - includes PDF processing
│   └── Parse Metadata (Tool)
│
├── Phase 2: Processing Pipeline (Loop - iterates 3 documents)
│   └── For each document:
│       └── Document Processing (Sequential Sub-workflow)
│           ├── Chunk Document (Tool)
│           ├── Extract Entities (Agent)
│           ├── Identify Topics (Agent)
│           └── Generate Summary (Agent)
│
├── Phase 3: Knowledge Building (Sequential)
│   ├── Create Embeddings (Agent)
│   ├── Build Knowledge Graph (Tool)
│   └── Index for Search (Tool)
│
└── Phase 4: Q&A Interface (Conditional)
    ├── Classify Query (Agent)
    ├── If Question:
    │   └── Q&A Workflow (Sequential)
    │       ├── Search Knowledge (Tool)
    │       ├── Generate Answer (Agent)
    │       └── Format Citations (Tool)
    └── If Analysis:
        └── Analysis Workflow (Parallel)
            ├── Compare Documents (Agent)
            ├── Find Patterns (Agent)
            └── Generate Insights (Agent)
```

### Agents (8 Total - Most Complex System)

| Agent | Model | Purpose | Temperature |
|-------|-------|---------|-------------|
| **Entity Extractor** | GPT-4o-mini | Named entity recognition (people, orgs, locations) | 0.2 |
| **Topic Analyzer** | Claude-3-haiku | Topic modeling and theme identification | 0.3 |
| **Summarizer** | Claude-3-haiku | Comprehensive document summarization | 0.4 |
| **Embedding Generator** | text-embedding-ada-002* | Vector embeddings for semantic search | 0.1 |
| **Q&A Responder** | GPT-4o-mini | Accurate question answering with citations | 0.3 |
| **Document Comparer** | Claude-3-haiku | Multi-document comparison and contrast | 0.3 |
| **Pattern Analyzer** | GPT-4o-mini | Pattern and trend discovery | 0.4 |
| **Insight Generator** | Claude-3-haiku | Actionable insight generation | 0.5 |

*Note: Embedding generation simulated with GPT in demo mode

### Tools

- **file_operations**: Document loading and output saving
- **text_manipulator**: Text chunking, extraction, and formatting
- **json_processor**: Metadata parsing, indexing, and search
- **pdf_processor**: PDF text extraction (simulated)
- **graph_builder**: Knowledge graph construction (simulated)
- **vector_search**: Similarity search in embeddings (simulated)
- **citation_formatter**: Reference formatting (simulated)

## Sample Documents

The system processes three diverse document types to demonstrate capabilities:

1. **Technical Paper**: Neural Architecture Search research with AutoML-X
2. **Business Report**: Q1 2024 AI market trends and analysis
3. **Research Proposal**: Quantum-classical hybrid computing for drug discovery

### Knowledge Extracted

**Entities Identified**:
- People: Dr. Sarah Chen, Prof. Michael Roberts, Dr. Emily Watson, Dr. James Liu, Dr. Maria Garcia
- Organizations: AI Research Institute, TechInsights Analytics, Microsoft, Google, Amazon, NVIDIA, NSF
- Locations: Implicit global references

**Topics Discovered**:
- Neural Architecture Search (NAS)
- AutoML and automation
- Edge AI and edge computing
- MLOps and infrastructure
- Quantum computing
- Drug discovery
- AI governance
- Market trends

**Key Insights**:
- AutoML-X achieves 60% faster NAS with 10x less compute
- 73% of Fortune 500 companies actively deploying ML
- AI market reached $387 billion (42% YoY growth)
- Quantum-classical hybrid could reduce drug development by 50%

## Configuration

### config.toml Structure

```toml
default_engine = "lua"

[providers.providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini"

[providers.providers.anthropic]
provider_type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-haiku-20240307"
```

### Customization Options

Edit `main.lua` to customize:

```lua
local config = {
    processing_settings = {
        max_documents = 10,      -- Process more documents
        chunk_size = 1000,       -- Larger chunks for context
        embedding_dimensions = 1536,
        similarity_threshold = 0.75  -- Adjust search sensitivity
    }
}
```

## Output Files

| File | Description |
|------|-------------|
| `/tmp/documents/*.txt` | Input documents for processing |
| `/tmp/document-index.json` | Searchable document index |
| `/tmp/knowledge-graph.json` | Entity relationships and connections |
| `/tmp/embeddings.json` | Vector embeddings for semantic search |
| `/tmp/qa-responses.txt` | Generated Q&A responses |
| `/tmp/analysis-report.md` | Document analysis insights |
| `/tmp/intelligence-summary.txt` | Execution summary and metrics |

## Performance Metrics

Typical execution times:

- **Document Ingestion** (Parallel): ~100ms
- **Processing Pipeline** (Loop × 3): ~200ms
  - Each document processing: ~65ms
- **Knowledge Building** (Sequential): ~75ms
- **Q&A Interface** (Conditional): ~75ms
- **Total Intelligence Time**: ~450ms

## Loop Workflow Implementation

The processing pipeline demonstrates loop workflows:

```lua
local processing_pipeline = Workflow.builder()
    :name("processing_pipeline")
    :loop_workflow()
    :max_iterations(3)  -- Process 3 documents
    :add_step({
        name = "process_document",
        type = "workflow",
        workflow = document_processing  -- Sequential sub-workflow
    })
    :build()
```

Each iteration processes a document through entity extraction, topic analysis, and summarization.

## Conditional Q&A Interface

The system demonstrates conditional routing:

```lua
-- Classify query type
:condition(function(ctx)
    local result = ctx.classify_query or ""
    return string.match(result:lower(), "question") ~= nil
end)

-- Route to appropriate workflow
:add_then_step({
    name = "answer_question",
    type = "workflow",
    workflow = qa_workflow
})
:add_else_step({
    name = "perform_analysis",
    type = "workflow",
    workflow = analysis_workflow
})
```

## Sample Queries and Responses

### Question Mode
**Q**: "How does AutoML-X improve upon existing NAS methods?"

**A**: "AutoML-X improves NAS through: 1) Hybrid evolutionary-RL search strategy, 2) 60% faster search time, 3) 10x less computational resources required, 4) Hardware-aware optimization. It achieves 97.3% accuracy on CIFAR-10 and 84.2% on ImageNet."

### Analysis Mode
**Query**: "Analyze trends across all documents"

**Results**:
- **Patterns**: Convergence on hybrid approaches (quantum-classical, evolution-RL)
- **Trends**: Explosive growth in edge AI (156% increase), MLOps (89% growth)
- **Insights**: Organizations need AI governance frameworks and clear ROI metrics

## Cost Considerations

**Warning**: Real API usage incurs costs:

- **Entity/Topic Extraction**: ~$0.002 per document
- **Summarization**: ~$0.003 per document
- **Q&A Generation**: ~$0.002 per query
- **Embeddings**: ~$0.0001 per 1000 tokens
- **Typical run cost**: $0.02 - $0.03 per full pipeline

To minimize costs:
1. Limit `max_documents` in processing
2. Use smaller chunk sizes
3. Cache embeddings for unchanged documents
4. Use cheaper models for non-critical analysis

## Knowledge Graph Structure

The system builds a knowledge graph with:

```json
{
  "nodes": [
    {"id": "1", "type": "person", "name": "Dr. Sarah Chen"},
    {"id": "2", "type": "concept", "name": "AutoML-X"},
    {"id": "3", "type": "organization", "name": "AI Research Institute"}
  ],
  "edges": [
    {"from": "1", "to": "2", "relationship": "created"},
    {"from": "1", "to": "3", "relationship": "affiliated_with"}
  ]
}
```

## Troubleshooting

### "Agent needs API key" Messages
- System continues with simulated processing
- Set environment variables for full AI capabilities

### Loop Workflow Issues
- Check `max_documents` doesn't exceed available documents
- Verify sufficient memory for large document sets

### Embedding Generation
- Demo uses simulated embeddings
- Production would use OpenAI's text-embedding-ada-002

### Conditional Routing Problems
- Ensure query classification returns expected format
- Check condition functions for proper string matching

## Blueprint Compliance

✅ 4-Phase Sequential Architecture
✅ Parallel document ingestion
✅ Loop workflow for document processing
✅ Conditional Q&A interface routing
✅ 8 specialized agents (most complex)
✅ Knowledge graph and embeddings
✅ Production error handling

## Example Use Cases

1. **Research Literature Review**: Process academic papers, extract citations, find gaps
2. **Business Intelligence**: Analyze reports, identify trends, generate insights
3. **Legal Document Analysis**: Extract entities, find precedents, compare contracts
4. **Knowledge Base Creation**: Build searchable knowledge from documentation
5. **Competitive Analysis**: Compare competitor documents, find patterns

## Extending the System

1. **Real Vector Database**: Integrate Pinecone, Weaviate, or Qdrant
2. **Graph Database**: Use Neo4j for complex relationship queries
3. **OCR Integration**: Add optical character recognition for scanned documents
4. **Multi-language Support**: Process documents in various languages
5. **Streaming Processing**: Handle real-time document streams

## Integration Points

### Vector Database Integration
```lua
-- Future: Real vector database
local embeddings = vector_db:create_embeddings(document_chunks)
local results = vector_db:similarity_search(query, k=5)
```

### Knowledge Graph Queries
```lua
-- Future: Graph database queries
local related = graph_db:query("MATCH (n)-[r]->(m) WHERE n.name = $name", {name = entity})
```

### Document Sources
- Local files (implemented)
- Cloud storage (S3, GCS, Azure)
- Document management systems
- Web scraping
- API integrations

## Performance Optimization

1. **Batch Processing**: Process documents in optimized batches
2. **Caching**: Cache embeddings and frequently accessed data
3. **Parallel Chunking**: Parallelize document chunking operations
4. **Lazy Loading**: Load documents on-demand for large collections
5. **Incremental Updates**: Only process new or changed documents

## Related Examples

- **Code Review Assistant**: Loop workflow patterns
- **Data Pipeline**: Parallel processing techniques
- **Content Generation Platform**: Multi-agent collaboration
- **Customer Support System**: Conditional routing
- **Workflow Examples**: Basic patterns in `examples/lua/workflows/`

## Support

For issues or questions:
- Review the main llmspell documentation
- Check blueprint.md for architectural patterns
- See examples/script-users/getting-started/ for basics