# llmspell-cli

Command-line interface for Rs-LLMSpell - scriptable LLM interactions via Lua and JavaScript with production RAG capabilities.

## Features

### Core Capabilities
- Interactive REPL for Lua/JavaScript scripting with LLMs
- Execute script files with agent and workflow support
- Built-in library access with 40+ tools and templates
- **RAG Integration (Phase 8)**: Complete document ingestion, vector search, and multi-tenant isolation
- **Embedded Applications**: 7 production-ready applications compiled into the binary

### RAG-Enabled Applications (Phase 8)
- **Document Intelligence**: Advanced document analysis with semantic search
- **Research Assistant**: Multi-source research with citation tracking
- **Code Review Assistant**: Enhanced with codebase context and best practices retrieval
- **WebApp Creator**: AI-driven development with pattern library and framework knowledge

## Usage

### Basic Commands
```bash
llmspell repl --language lua        # Start Lua REPL
llmspell run script.lua             # Execute script file
llmspell list-tools                 # Show available tools
```

### Embedded Applications (Production-Ready)
All applications are compiled directly into the `llmspell` binary for single-file distribution:

```bash
# List all embedded applications with RAG capabilities
llmspell apps list                  

# Universal Complexity (2-3 agents)
llmspell apps file-organizer        # Smart file organization with content analysis
llmspell apps content-creator       # Multi-format content generation

# Professional Complexity (4-5 agents)  
llmspell apps research-collector    # Research automation with document ingestion
llmspell apps communication-manager # Business communication with template RAG

# Expert Complexity (8-20 agents)
llmspell apps process-orchestrator  # Advanced workflow orchestration
llmspell apps code-review-assistant # Code review with codebase RAG knowledge
llmspell apps webapp-creator        # Full-stack development with 20 agents
```

### RAG-Enhanced Application Features (Phase 8)
- **Document Intelligence**: Semantic document analysis, classification, and extraction
- **Knowledge Base Integration**: Persistent knowledge with vector search across sessions
- **Multi-Tenant Isolation**: Complete data separation for enterprise deployments
- **Context-Aware Processing**: Conversation memory and cross-session context retention
- **Codebase Understanding**: Deep code analysis with pattern matching and best practices

### Application Architecture
These applications demonstrate:
- **Compiled Distribution**: Using `include_str!` from `resources/applications/`
- **Zero-Config Deployment**: Extract to temp directory at runtime with no setup
- **Progressive Complexity**: From Universal (2-3 agents) to Expert (20+ agents)
- **Production Patterns**: Real-world multi-agent orchestration examples
- **RAG Integration**: Document processing, vector storage, and intelligent retrieval

For development, the source applications remain in `examples/script-users/applications/` where they can be edited and tested using traditional path-based execution:

```bash
llmspell run examples/script-users/applications/file-organizer/main.lua
```

### RAG CLI Examples (Phase 8)

```bash
# Run RAG-enabled research assistant
llmspell apps research-collector --config configs/rag-production.toml

# WebApp creator with codebase RAG knowledge  
llmspell apps webapp-creator --input user-requirements.lua --output ./generated-app

# Code review with repository context
llmspell apps code-review-assistant --repo-path ./my-project --output review-report.md

# Interactive RAG REPL
llmspell repl --language lua --config configs/rag-basic.toml
-- RAG operations available in REPL:
-- RAG.ingest_documents("tenant-123", documents)
-- RAG.retrieve("tenant-123", {query = "search terms"})
```

### Multi-Tenant CLI Usage

```bash
# Set up multi-tenant environment
export LLMSPELL_CONFIG=configs/rag-multi-tenant.toml
export TENANT_ID=company-123

# Run application with tenant isolation
llmspell apps webapp-creator --tenant $TENANT_ID --input requirements.lua

# Tenant-specific data management
llmspell exec 'RAG.cleanup_tenant("old-tenant-456")'
llmspell exec 'print("Tenant stats:", RAG.get_stats("'$TENANT_ID'"))'
```

## Dependencies
- `llmspell-core` - Core traits and types
- `llmspell-bridge` - Script engine integration with RAG globals
- `llmspell-agents` - Agent implementations
- `llmspell-workflows` - Workflow orchestration
- `llmspell-rag` - RAG pipeline functionality (Phase 8)
- `llmspell-storage` - Vector storage and HNSW (Phase 8)
- `llmspell-tenancy` - Multi-tenant isolation (Phase 8)
- `llmspell-security` - Access control and sandboxing
- `llmspell-config` - Configuration management