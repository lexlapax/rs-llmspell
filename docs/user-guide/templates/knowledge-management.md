# Knowledge Management Template

RAG-powered knowledge management with ingest-query-synthesize pipeline for personal knowledge bases.

## Overview

The Knowledge Management template provides CRUD operations for building and querying knowledge bases using a simplified RAG (Retrieval-Augmented Generation) pattern. It supports multi-collection storage, document chunking, semantic search (mock implementation via word overlap), and citation tracking.

## Template Metadata

- **ID**: `knowledge-management`
- **Category**: Research
- **Version**: 0.1.0
- **Tags**: `rag`, `knowledge-base`, `semantic-search`, `research`, `learning`

## Capabilities

### 5 Core Operations

1. **Ingest**: Add documents to knowledge base with automatic chunking
2. **Query**: Semantic search with citation tracking
3. **Update**: Modify existing documents
4. **Delete**: Remove documents from collection
5. **List**: View all documents in a collection

### Key Features

- **Multi-Collection Support**: Organize knowledge into separate collections
- **Document Chunking**: Automatic splitting with configurable size and overlap
- **Source Type Handling**: `text`, `markdown`, `file` (path-based)
- **Citation Tracking**: Metadata preservation for source attribution
- **Tag Extraction**: Automatic keyword-based tagging
- **Semantic Search**: Simple word-overlap scoring (mock RAG for testing)
- **State Persistence**: Uses StateManager for cross-session storage

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `operation` | String | Yes | - | Operation: `ingest`, `query`, `update`, `delete`, `list` |
| `collection` | String | Yes | - | Collection name (1-100 chars) |
| `content` | String | Conditional | - | Content to ingest/update or file path |
| `query` | String | Conditional | - | Search query for retrieval |
| `document_id` | String | Conditional | - | Document identifier for update/delete |
| `source_type` | String | No | `"text"` | Source type: `text`, `markdown`, `file` |
| `max_results` | Integer | No | `5` | Maximum query results (1-50) |
| `include_citations` | Boolean | No | `true` | Include source citations in results |
| `chunk_size` | Integer | No | `200` | Words per chunk (50-1000) |
| `chunk_overlap` | Integer | No | `50` | Word overlap between chunks |
| `output_format` | String | No | `"text"` | Output format: `text`, `json` |

### Memory Parameters

All templates support optional memory integration for context-aware execution:

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `session_id` | String | `null` | Any string | Session identifier for conversation memory filtering |
| `memory_enabled` | Boolean | `true` | `true`, `false` | Enable memory-enhanced execution (uses episodic + semantic memory) |
| `context_budget` | Integer | `2000` | `100-8000` | Token budget for context assembly (higher = more context) |

**Memory Integration**: When `session_id` is provided and `memory_enabled` is `true`, the template will:
- Retrieve relevant episodic memory from conversation history
- Query semantic memory for related concepts
- Assemble context within the `context_budget` token limit
- Provide memory-enhanced context to LLM for better results

### Provider Parameters

Templates support dual-path provider resolution:

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `provider_name` | String | `null` | `"ollama"`, `"openai"`, etc. | Provider name (mutually exclusive with `model`) |

**Provider Resolution**:
- Use `provider_name` to select a provider with its default model (e.g., `provider_name: "ollama"`)
- Use `model` for explicit model selection (e.g., `model: "gpt-4"`)
- If both provided, `model` takes precedence
- `provider_name` and `model` are mutually exclusive

### Parameter Constraints

**operation**: Must be one of:
- `ingest`: Add new document
- `query`: Search knowledge base
- `update`: Modify existing document (requires `document_id` + `content`)
- `delete`: Remove document (requires `document_id`)
- `list`: Show all documents

**Conditional Requirements**:
- `ingest`/`update`: Requires `content`
- `query`: Requires `query`
- `update`/`delete`: Requires `document_id`

## Usage Examples

### 1. Ingest Text Document

```bash
llmspell template exec knowledge-management \
  --param operation=ingest \
  --param collection=rust-docs \
  --param content="Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety." \
  --param source_type=text \
  --param chunk_size=100
```

**Output**:
```
✅ Document ingested successfully

Document ID: doc-a1b2c3d4e5f6
Chunks: 2
Tags: rust
Collection: rust-docs
```

### CLI - With Memory and Provider

Enable memory-enhanced execution with custom provider:

```bash
llmspell template exec knowledge-management \
  --param operation=ingest \
  --param collection=rust-docs \
  --param content="Rust is a systems programming language..." \
  --param source_type=text \
  --param session-id="user-session-123" \
  --param memory-enabled=true \
  --param context-budget=3000 \
  --param provider-name="ollama"
```

### 2. Ingest Markdown File

```bash
llmspell template exec knowledge-management \
  --param operation=ingest \
  --param collection=project-notes \
  --param content="path/to/notes.md" \
  --param source_type=file \
  --param chunk_size=300 \
  --param chunk_overlap=75
```

### 3. Query Knowledge Base

```bash
llmspell template exec knowledge-management \
  --param operation=query \
  --param collection=rust-docs \
  --param query="memory safety in Rust" \
  --param max_results=3 \
  --param include_citations=true
```

**Output**:
```
=== KNOWLEDGE QUERY RESULTS ===

Query: "memory safety in Rust"
Results: 2 matches found

Result 1: (relevance: 0.75)
  Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.

  Citations:
    Document ID: doc-a1b2c3d4e5f6-chunk-0
    Source Type: text
    Tags: rust
    Timestamp: 2025-10-21T20:30:45Z

Result 2: (relevance: 0.50)
  [Additional relevant chunk...]
```

### 4. Update Document

```bash
llmspell template exec knowledge-management \
  --param operation=update \
  --param collection=rust-docs \
  --param document_id=doc-a1b2c3d4e5f6 \
  --param content="Rust is a memory-safe systems programming language with zero-cost abstractions." \
  --param source_type=text
```

### 5. Delete Document

```bash
llmspell template exec knowledge-management \
  --param operation=delete \
  --param collection=rust-docs \
  --param document_id=doc-a1b2c3d4e5f6
```

### 6. List All Documents

```bash
llmspell template exec knowledge-management \
  --param operation=list \
  --param collection=rust-docs \
  --param output_format=text
```

### Lua - With Memory and Provider

Enable memory-enhanced execution:

```lua
local result = Template.execute("knowledge-management", {
    operation = "query",
    collection = "rust-docs",
    query = "memory safety",
    max_results = 3,
    session_id = "user-session-123",
    memory_enabled = true,
    context_budget = 3000,
    provider_name = "ollama"
})
```

**Output**:
```
=== KNOWLEDGE BASE: rust-docs ===

Total Documents: 3

Document 1:
  ID: doc-a1b2c3d4e5f6
  Chunks: 2
  Tags: rust
  Source: text
  Timestamp: 2025-10-21T20:30:45Z
  Preview: Rust is a memory-safe systems programming language...

[Additional documents...]
```

## Lua Script Examples

### Example 1: Research Workflow

```lua
#!/usr/bin/env llmspell

-- Knowledge base research workflow
-- Ingest → Query → Synthesize pattern

local Template = require("template")

-- Step 1: Ingest research papers
print("Ingesting research documents...")

local papers = {
    {path = "papers/rust-memory-model.md", type = "file"},
    {path = "papers/ownership-system.md", type = "file"},
    {content = "Key insight: Rust's ownership rules are checked at compile time.", type = "text"}
}

for i, paper in ipairs(papers) do
    local result = Template.execute("knowledge-management", {
        operation = "ingest",
        collection = "rust-research",
        content = paper.path or paper.content,
        source_type = paper.type,
        chunk_size = 250,
        chunk_overlap = 50
    })

    if result.success then
        print(string.format("✅ Ingested paper %d: %s", i, result.metrics.document_id))
    else
        print(string.format("❌ Failed to ingest paper %d: %s", i, result.error))
    end
end

-- Step 2: Query knowledge base
print("\nQuerying knowledge base...")

local queries = {
    "How does Rust's ownership system prevent memory leaks?",
    "What are the trade-offs of compile-time memory safety?",
    "Explain borrowing rules in Rust"
}

for _, query_text in ipairs(queries) do
    print(string.format("\nQuery: %s", query_text))

    local query_result = Template.execute("knowledge-management", {
        operation = "query",
        collection = "rust-research",
        query = query_text,
        max_results = 3,
        include_citations = true,
        output_format = "text"
    })

    if query_result.success then
        print(query_result.result)
    else
        print("Query failed: " .. query_result.error)
    end
end

-- Step 3: List all documents
print("\nKnowledge base summary:")
local list_result = Template.execute("knowledge-management", {
    operation = "list",
    collection = "rust-research"
})

print(list_result.result)
```

### Example 2: Document Update Workflow

```lua
#!/usr/bin/env llmspell

local Template = require("template")

-- Ingest initial version
local ingest = Template.execute("knowledge-management", {
    operation = "ingest",
    collection = "project-notes",
    content = "Project Alpha: Initial design phase",
    source_type = "text"
})

local doc_id = ingest.metrics.document_id
print("Created document: " .. doc_id)

-- Query to verify
local query1 = Template.execute("knowledge-management", {
    operation = "query",
    collection = "project-notes",
    query = "design phase"
})

print("\nBefore update:")
print(query1.result)

-- Update document
Template.execute("knowledge-management", {
    operation = "update",
    collection = "project-notes",
    document_id = doc_id,
    content = "Project Alpha: Implementation phase - milestone 1 complete",
    source_type = "text"
})

-- Query again to see changes
local query2 = Template.execute("knowledge-management", {
    operation = "query",
    collection = "project-notes",
    query = "implementation phase"
})

print("\nAfter update:")
print(query2.result)
```

### Example 3: Multi-Collection Management

```lua
#!/usr/bin/env llmspell

local Template = require("template")

-- Create multiple knowledge bases
local collections = {
    {name = "rust-docs", topic = "Rust programming language"},
    {name = "python-docs", topic = "Python programming language"},
    {name = "architecture", topic = "Software architecture patterns"}
}

-- Populate each collection
for _, coll in ipairs(collections) do
    print(string.format("Setting up collection: %s", coll.name))

    Template.execute("knowledge-management", {
        operation = "ingest",
        collection = coll.name,
        content = string.format("Collection for %s", coll.topic),
        source_type = "text"
    })
end

-- Query across collections
local search_term = "programming"

for _, coll in ipairs(collections) do
    print(string.format("\n=== Searching in %s ===", coll.name))

    local result = Template.execute("knowledge-management", {
        operation = "query",
        collection = coll.name,
        query = search_term,
        max_results = 2
    })

    print(result.result)
end

-- Clean up: delete collections
for _, coll in ipairs(collections) do
    local list = Template.execute("knowledge-management", {
        operation = "list",
        collection = coll.name,
        output_format = "json"
    })

    -- Parse JSON and delete all documents
    for _, doc in ipairs(list.result.documents) do
        Template.execute("knowledge-management", {
            operation = "delete",
            collection = coll.name,
            document_id = doc.id
        })
    end
end
```

## Implementation Details

### Document Structure

```rust
pub struct KnowledgeDocument {
    pub id: String,              // Generated hash from content + collection
    pub content: String,          // Original document content
    pub metadata: DocumentMetadata,
    pub chunks: Vec<String>,      // Chunked content for retrieval
}

pub struct DocumentMetadata {
    pub title: Option<String>,    // Extracted from content or None
    pub source_type: String,      // "text", "markdown", "file"
    pub source_path: Option<String>, // File path if source_type="file"
    pub category: Option<String>, // User-defined category
    pub tags: Vec<String>,        // Auto-extracted keywords
    pub timestamp: String,        // RFC3339 timestamp
}
```

### Document Chunking Algorithm

1. Split content into words (whitespace-delimited)
2. Create overlapping windows of `chunk_size` words
3. Advance by `(chunk_size - chunk_overlap)` words
4. Store chunks with document metadata

**Example** (chunk_size=5, overlap=2):
```
Content: "Rust is a systems programming language that runs fast"

Chunk 1: "Rust is a systems programming"
Chunk 2: "systems programming language that runs"  // overlaps "systems programming"
Chunk 3: "that runs fast"
```

### Search Algorithm (Mock RAG)

Current implementation uses simple word-overlap scoring:

1. Convert query and chunks to lowercase
2. Count matching words between query and each chunk
3. Score = (matches / total_query_words)
4. Sort by relevance score
5. Return top-k results

**Note**: This is a simplified mock implementation for testing. Production RAG would use:
- Vector embeddings (OpenAI, sentence-transformers)
- Similarity metrics (cosine similarity)
- Vector databases (Pinecone, Weaviate, Qdrant)

### State Storage

Documents are stored in StateManager with keys:
```
knowledge:collections:<collection_name>
```

Value format:
```json
[
  {
    "id": "doc-abc123",
    "content": "...",
    "metadata": {...},
    "chunks": ["...", "..."]
  }
]
```

## Output Formats

### Text Format (Default)

```
=== KNOWLEDGE QUERY RESULTS ===

Query: "search term"
Results: 3 matches found

Result 1: (relevance: 0.85)
  [Chunk content...]

  Citations:
    Document ID: doc-abc123-chunk-0
    Title: Document Title
    Source Type: markdown
    Tags: rust, programming
    Timestamp: 2025-10-21T20:30:45Z

[Additional results...]
```

### JSON Format

```json
{
  "query": "search term",
  "collection": "rust-docs",
  "results": [
    {
      "document_id": "doc-abc123-chunk-0",
      "chunk": "Chunk content...",
      "relevance_score": 0.85,
      "metadata": {
        "title": "Document Title",
        "source_type": "markdown",
        "tags": ["rust", "programming"],
        "timestamp": "2025-10-21T20:30:45Z"
      }
    }
  ],
  "total_documents": 5,
  "max_results": 3
}
```

## Performance Characteristics

- **Ingest Operation**: ~100-500ms (depends on content size, chunking)
- **Query Operation**: ~50-300ms (depends on collection size, simplified search)
- **Update Operation**: ~150-400ms (retrieve + modify + store)
- **Delete Operation**: ~50-100ms (retrieve + filter + store)
- **List Operation**: ~50-150ms (state retrieval + formatting)

**Memory Usage**:
- In-memory document storage via StateManager
- Chunk overhead: ~20-30% of original content size

## Limitations

### Current Implementation (v0.1.0)

1. **No True RAG**: Uses word-overlap instead of vector embeddings
2. **No Embedding Provider**: Requires external RAG implementation for production
3. **Limited Source Types**: PDF and URL ingestion not implemented
4. **Simple Tagging**: Keyword-based, not semantic
5. **No Relevance Tuning**: Fixed scoring algorithm
6. **Memory-Only Storage**: StateManager persistence depends on backend configuration

### Planned Enhancements

- Integration with vector embedding providers (OpenAI, Cohere)
- PDF extraction via `pdf-extract` or `lopdf`
- URL fetching and content extraction
- Semantic tagging using LLM
- Configurable relevance thresholds
- Multi-modal support (images, code)

## Troubleshooting

### Collection Empty or Not Found

**Error**: `Collection 'xyz' is empty or does not exist`

**Cause**: Querying/updating/deleting from non-existent or empty collection

**Solution**:
1. Verify collection name spelling
2. Use `list` operation to check if collection exists
3. Ingest at least one document before querying

### Document Not Found

**Error**: `Document 'doc-xyz' not found in collection`

**Cause**: Invalid `document_id` for update/delete operation

**Solution**:
1. Use `list` operation to get valid document IDs
2. Ensure document wasn't already deleted
3. Check if using correct collection name

### Missing Required Parameters

**Error**: `Required parameter missing: content`

**Cause**: Operation requires parameter that wasn't provided

**Solution**:
- `ingest`/`update`: Must provide `content`
- `query`: Must provide `query`
- `update`/`delete`: Must provide `document_id`

### StateManager Not Available

**Error**: `StateManager required for knowledge management`

**Cause**: ExecutionContext doesn't have StateManager configured

**Solution**:
```rust
let state_manager = Arc::new(StateManager::new().await?);
let context = ExecutionContext::builder()
    .with_state_manager(state_manager)
    .build()?;
```

### Large Document Performance

**Issue**: Slow ingest for documents >10,000 words

**Solutions**:
1. Reduce `chunk_size` for faster processing
2. Increase `chunk_overlap` cautiously (increases chunk count)
3. Pre-process documents to extract relevant sections
4. Split very large documents into multiple ingestions

## Best Practices

### Collection Organization

```lua
-- Good: Organized by topic/project
collections = {
    "rust-language-docs",
    "project-alpha-notes",
    "research-papers-ml"
}

-- Avoid: Too generic
collections = {
    "docs",           -- What kind of docs?
    "notes",          -- Which project?
    "research"        -- What field?
}
```

### Chunking Strategy

```lua
-- For technical docs: smaller chunks, low overlap
{
    chunk_size = 150,
    chunk_overlap = 30
}

-- For narratives: larger chunks, higher overlap
{
    chunk_size = 400,
    chunk_overlap = 100
}

-- For code: very small chunks, preserve context
{
    chunk_size = 50,
    chunk_overlap = 10
}
```

### Query Formulation

```lua
-- Good: Specific, keyword-rich
query = "Rust ownership rules prevent memory leaks"

-- Avoid: Too vague
query = "memory"

-- Good: Natural language questions
query = "How does async/await work in Rust?"

-- Good: Multi-concept queries
query = "lifetime annotations borrowing checker"
```

### Citation Tracking

Always enable citations for research workflows:

```lua
Template.execute("knowledge-management", {
    operation = "query",
    collection = "research",
    query = "...",
    include_citations = true,  -- Track sources
    output_format = "json"     -- Parse citations programmatically
})
```

## Integration Patterns

### Pattern 1: Research Pipeline

```
1. Ingest multiple sources
2. Query with research question
3. Extract top-k citations
4. Pass to LLM for synthesis
5. Store synthesized answer back to knowledge base
```

### Pattern 2: Iterative Learning

```
1. Ingest initial knowledge
2. Query for understanding gaps
3. Fetch external resources (web, papers)
4. Ingest new knowledge
5. Re-query with refined questions
6. Repeat until satisfactory coverage
```

### Pattern 3: Document Versioning

```
1. Ingest document v1
2. Store document_id for tracking
3. When updates needed:
   - Update operation with same document_id
   - Timestamps track version history
4. Query operations always use latest version
```

## Metrics

Template execution includes performance metrics:

```json
{
  "metrics": {
    "duration_ms": 145,
    "operation": "query",
    "collection": "rust-docs",
    "results_found": 3,
    "documents_searched": 12,
    "document_id": "doc-abc123",      // For ingest/update/delete
    "chunks_created": 5,               // For ingest
    "chunks_updated": 4,               // For update
    "tags_extracted": 3,               // For ingest
    "remaining_documents": 11          // For delete
  }
}
```

## Related Templates

- **Research Assistant**: Multi-source research with synthesis
- **Document Processor**: PDF/OCR extraction and transformation
- **Interactive Chat**: Session-based Q&A with context

## See Also

- [Template System Overview](../README.md)
- [Research Assistant Template](./research-assistant.md)
- [Workflow Orchestrator Template](./workflow-orchestrator.md)
- [StateManager Documentation](../../developer-guide/state-management.md)
