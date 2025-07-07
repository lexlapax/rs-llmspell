# Phase 2.5: External Integration Tools - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: Planning  
**Phase**: 2.5 (External Integration Tools)  
**Timeline**: Weeks 9-10 (2 weeks)  
**Priority**: HIGH (MVP Enhancement)

> **📋 Detailed Implementation Guide**: This document contains extracted designs for external dependency tools moved from Phase 2 to Phase 2.5.

---

## Overview

Phase 2.5 implements tools that require external dependencies, APIs, or complex infrastructure. These were moved from Phase 2 to maintain clean separation between self-contained and external tools.

### Tool Categories
- **Web & Network Tools**: Including WebSearchTool and other web interaction tools
- **Advanced Search Tools**: CodeSearchTool with tree-sitter, SemanticSearchTool with vectors
- **Communication & API Integration**: Email, Slack, GitHub, database connectors
- **Complex Data Processing**: Tools requiring external services or heavy dependencies

---

## 1. Search Tools (Extracted from Phase 2)

### 1.1 WebSearchTool

```rust
// llmspell-tools/src/search/web_search.rs
pub struct WebSearchTool {
    client: reqwest::Client,
    api_key: Option<String>,
    provider: SearchProvider,
}

impl Tool for WebSearchTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "search_type": {
                        "type": "string",
                        "enum": ["web", "news", "images", "videos"],
                        "default": "web"
                    }
                },
                "required": ["query"]
            }),
        }
    }
}
```

**Implementation Details:**
- DuckDuckGo provider (no API key required)
- Google Custom Search provider (API key required)
- Bing Search provider (API key required)
- Rate limiting and retry logic
- Response parsing and normalization

### 1.2 CodeSearchTool

```rust
// llmspell-tools/src/search/code_search.rs
pub struct CodeSearchTool {
    index_path: PathBuf,
    language_parsers: HashMap<String, Box<dyn LanguageParser>>,
}
```

**Implementation Details:**
- Tree-sitter integration for syntax-aware parsing
- Support for Rust, Python, JavaScript/TypeScript
- Symbol extraction (functions, classes, variables)
- Full-text search with ranking
- Git integration for repository search
- Incremental indexing support

### 1.3 SemanticSearchTool

```rust
// llmspell-tools/src/search/semantic_search.rs
pub struct SemanticSearchTool {
    embedding_model: Box<dyn EmbeddingModel>,
    vector_store: Box<dyn VectorStore>,
}
```

**Implementation Details:**
- Embedding model abstraction (local or API-based)
- Vector store trait with multiple backends
- In-memory vector store implementation
- Similarity search algorithms (cosine, k-NN)
- Metadata filtering support
- Optional integration with external vector databases

---

## 2. Additional External Integration Tools

### 2.1 Web & Network Tools
- **WebScraperTool**: JavaScript rendering, HTML parsing
- **UrlAnalyzerTool**: URL validation, metadata extraction
- **ApiTesterTool**: REST API testing, response validation
- **WebhookCallerTool**: Webhook invocation, retry logic
- **WebpageMonitorTool**: Change detection, notifications
- **SitemapCrawlerTool**: Sitemap parsing, URL discovery

### 2.2 Communication & API Tools
- **EmailSenderTool**: SMTP, SendGrid, SES integration
- **SlackIntegrationTool**: Slack API, webhooks
- **GitHubIntegrationTool**: GitHub API, issue/PR management
- **DatabaseConnectorTool**: PostgreSQL, MySQL, SQLite

### 2.3 Data Processing Tools
- **XmlProcessorTool**: XML parsing, XSLT transformation
- **YamlProcessorTool**: YAML parsing, validation
- **DataTransformerTool**: Format conversion, mapping
- **StatisticalAnalyzerTool**: Statistical computations
- **TextAnalyzerTool**: NLP operations, sentiment analysis
- **DataVisualizerTool**: Chart generation, data visualization

---

## 3. Dependencies and Infrastructure

### External Dependencies
- `reqwest`: HTTP client
- `tree-sitter`: Code parsing
- `tantivy`: Full-text search
- `git2`: Git integration
- `lettre`: Email sending
- `slack-api`: Slack integration
- `octocrab`: GitHub API
- `sqlx`: Database connections

### Infrastructure Requirements
- API key management system
- Rate limiting implementation
- Authentication handling
- Connection pooling
- Retry mechanisms
- Circuit breakers

---

## 4. Testing Strategy

### Integration Testing
- Mock external services for unit tests
- Real API integration tests (rate limited)
- Error scenario testing
- Performance benchmarking

### Security Testing
- API key exposure prevention
- Input sanitization
- Rate limit enforcement
- Network isolation in tests

---

## 5. Migration from Phase 2

Tools extracted from Phase 2 design maintain their specifications but gain:
- Enhanced error handling for external failures
- Comprehensive retry logic
- Circuit breaker patterns
- Better authentication management
- Connection pooling where applicable

---

## 6. Success Criteria

- [ ] 16+ external integration tools functional
- [ ] All API integrations properly authenticated
- [ ] Rate limiting respected for all providers
- [ ] Error handling graceful for external failures
- [ ] Performance acceptable despite external dependencies
- [ ] Security boundaries maintained
- [ ] Documentation includes configuration examples