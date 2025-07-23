# Documentation Archives

**Purpose**: Preserve historical documents, research notes, and deprecated documentation for reference.

**🔗 Navigation**: [← Documentation Hub](../README.md) | [Project Home](../../README.md)

---

## What This Directory Contains

The archives directory serves as a **historical repository** for documentation that is no longer current but may be valuable for understanding the evolution of rs-llmspell. This includes:

### Document Types

- **Historical Design Decisions**: Original design documents that shaped the project
- **Deprecated Guides**: User or developer guides that have been superseded
- **Research Documentation**: Explorations and experiments that informed development
- **Migration Notes**: Documentation for transitions between major versions
- **Meeting Notes**: Important project discussions and decisions
- **Legacy Examples**: Code examples from earlier versions

### Why We Keep Archives

1. **Historical Context**: Understanding why certain decisions were made
2. **Migration Support**: Helping users upgrade from older versions
3. **Research Reference**: Preserving explorations that might be revisited
4. **Project Evolution**: Documenting how the project has grown and changed
5. **Learning Resource**: Understanding past mistakes and successes

## Using the Archives

### ⚠️ Important Notes

- **NOT CURRENT**: Documents here reflect past states of the project
- **DO NOT USE** for current implementation guidance
- **FOR REFERENCE ONLY**: Always check current documentation first
- **MAY CONTAIN**: Outdated APIs, deprecated patterns, or abandoned features

### When to Use Archives

✅ **Good reasons to browse archives:**
- Understanding historical design decisions
- Researching project evolution
- Migrating from very old versions
- Academic or historical interest

❌ **Do NOT use archives for:**
- Current API reference
- Implementation guidance
- Best practices
- Production code examples

## Organization

Documents are organized by:
- **Phase**: Related to specific development phases (e.g., `phase-2-original-design.md`)
- **Date**: When the document was archived (e.g., `2024-12-deprecated-tool-api.md`)
- **Category**: Type of document (design, guide, research, etc.)

## Contributing to Archives

When moving documents to archives:
1. Add a header noting when and why it was archived
2. Include reference to the replacement documentation
3. Preserve the original content without modification
4. Update any indexes or cross-references

Example archive header:
```markdown
# [ARCHIVED 2025-01-30] Original Tool API Design

**Status**: DEPRECATED - Replaced by standardized Tool trait in Phase 3
**Replacement**: See [Tool Development Guide](../developer-guide/tool-development-guide.md)
**Reason**: API redesigned for consistency and security requirements

---
[Original content follows...]
```

---

**For current documentation**: 
- **[User Guide](../user-guide/)** - How to use rs-llmspell
- **[Developer Guide](../developer-guide/)** - How to contribute
- **[Technical Docs](../technical/)** - Architecture and design