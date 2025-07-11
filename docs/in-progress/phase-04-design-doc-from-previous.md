# Phase 2.5: External Integration Tools - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: Planning  
**Phase**: 4 
**Timeline**: Weeks 9-10 (2 weeks)  
**Priority**: HIGH (MVP Enhancement)

> **📋 Detailed Implementation Guide**: This document contains extracted designs for external dependency tools moved from Phase 2 to Phase 2.5.

---

## Overview
---

## 1. Search Tools (Extracted from Phase 2)


### 1.2 CodeSearchTool (this goes in phase 4)

```rust
// llmspell-tools/src/search/code_search.rs
pub struct CodeSearchTool {
    index_path: PathBuf,
    language_parsers: HashMap<String, Box<dyn LanguageParser>>,
}
```

**Implementation Details:**
- Tree-sitter integration for syntax-aware parsing (this goes in Phase 4)
- Support for Rust, Python, JavaScript/TypeScript (this goes in Phase 4)
- Symbol extraction (functions, classes, variables)
- Full-text search with ranking
- Git integration for repository search
- Incremental indexing support

### 1.3 SemanticSearchTool (*this goes in phase 4**)

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

