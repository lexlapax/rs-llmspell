# Documentation Consolidation Plan

**Date**: 2025-11-08
**Scope**: Reorganize docs/user-guide/ for holistic user experience
**Current State**: 53 files, 43,335 lines - TOO FRAGMENTED
**Target State**: 12-15 core user guides, clear hierarchy, logical flow

## Executive Summary

**Problem**: Documentation is scattered across 53+ files with unclear boundaries between user-facing, developer-facing, and technical content. Users face decision paralysis and can't find information efficiently.

**Solution**: Consolidate into 3 clear documentation tiers:
1. **docs/user-guide/** (10-12 files): USER-FACING - how to USE llmspell for AI experimentation
2. **docs/developer-guide/** (5-8 files): DEVELOPER-FACING - how to EXTEND llmspell with Rust
3. **docs/technical/** (8-10 files): ARCHITECTURE/TECHNICAL - deep technical details, schemas, benchmarks

**Impact**:
- 53 → 15 user guide files (71% reduction)
- 21 Rust API docs → moved to developer-guide/reference/
- Clear separation of concerns
- Single linear path for new users
- Advanced topics organized by use case

---

## Current State Analysis

### File Count by Directory

```
docs/user-guide/
├── Main guides: 15 files (getting-started, concepts, configuration, etc.)
├── storage/: 6 files (postgresql-setup, schema-reference, performance-tuning, etc.)
├── templates/: 11 files (10 template guides + README)
├── api/rust/: 21 files (one per crate) ← DEVELOPER CONTENT
├── api/lua/: 1 file (3,729 lines) ← USER CONTENT but HUGE
└── Total: 53+ files, 43,335 lines
```

### Problems Identified

1. **Mixed Audiences**: User content (Lua API) mixed with developer content (Rust API crates)
2. **Overlapping Files**: performance-tuning.md (Phase 10) vs storage/performance-tuning.md (PostgreSQL)
3. **Duplicate Troubleshooting**: troubleshooting.md vs troubleshooting-phase10.md
4. **Schema Details in User Guide**: storage/schema-reference.md (1,359 lines) is TECHNICAL, not user-facing
5. **Fragmented Storage Docs**: 6 storage files when most users just need setup + config
6. **Massive Lua API File**: 3,729 lines in single file - hard to navigate
7. **No Clear Learning Path**: 53 files with no obvious sequence

---

## Target Structure

### Tier 1: docs/user-guide/ (USER-FACING)

**Purpose**: How to USE llmspell for AI experimentation via Lua scripting

**Target**: 10-12 consolidated files organized by user journey

```
docs/user-guide/
├── README.md                           # Hub with clear user journey map
├── 01-getting-started.md               # Quick start (15 min to first script)
├── 02-core-concepts.md                 # Agents, Tools, RAG, Memory, Context
├── 03-configuration.md                 # config.toml, providers, feature flags
├── 04-lua-scripting.md                 # Lua API essentials (from api/lua/)
├── 05-cli-reference.md                 # All CLI commands
├── 06-templates-and-workflows.md       # 10 experimental templates
├── 07-storage-setup.md                 # Storage backends (Memory/Sled/PostgreSQL quick start)
├── 08-deployment.md                    # Service deployment, Docker, systemd
├── 09-security.md                      # Sandboxing, permissions, multi-tenancy
├── 10-troubleshooting.md               # Common problems + solutions
└── appendix/
    ├── lua-api-reference.md            # Complete Lua API (split from huge file)
    └── template-catalog.md             # Detailed template documentation
```

**Consolidations**:
- Merge troubleshooting.md + troubleshooting-phase10.md → 10-troubleshooting.md
- Merge getting-started.md + quick setup sections → 01-getting-started.md
- Merge concepts.md + memory-configuration.md + context concepts → 02-core-concepts.md
- Merge ide-integration.md + service-deployment.md → 08-deployment.md
- Merge local-llm.md + provider-best-practices.md sections → 03-configuration.md
- Consolidate storage/* into 07-storage-setup.md (user-facing parts only)
- Split api/lua/README.md (3,729 lines) → 04-lua-scripting.md (essentials) + appendix/lua-api-reference.md (complete)
- Consolidate templates/* → 06-templates-and-workflows.md + appendix/template-catalog.md

---

### Tier 2: docs/developer-guide/ (DEVELOPER-FACING)

**Purpose**: How to EXTEND llmspell with Rust (build custom tools, agents, backends)

**Target**: 6-8 developer-focused guides

```
docs/developer-guide/
├── README.md                           # Developer hub
├── extending-llmspell.md               # How to build custom components (EXISTING + expanded)
├── bridge-pattern-guide.md             # Lua-Rust bridge patterns (EXISTING)
├── testing-guide.md                    # Testing patterns with llmspell-testing
├── performance-optimization.md         # Benchmarking, profiling (MOVED from user-guide/performance-tuning.md)
└── reference/
    ├── README.md                       # API reference hub
    ├── core-traits.md                  # BaseAgent, Tool, ExecutionContext
    ├── storage-backends.md             # StorageBackend trait implementation
    ├── rag-pipeline.md                 # RAG pipeline extension
    ├── memory-backends.md              # Memory backend implementation
    ├── security-integration.md         # Security sandbox integration
    └── crate-index.md                  # Quick reference to all 21 crates
```

**Moves**:
- **ALL api/rust/*.md files** (21 files) → developer-guide/reference/ (consolidated into ~6 thematic guides)
- performance-tuning.md (Phase 10 kernel) → developer-guide/performance-optimization.md
- Expand extending-llmspell.md with patterns from various crate docs

**Consolidation Strategy for Rust API**:
Instead of 21 separate crate files, group by purpose:
- **core-traits.md**: llmspell-core, llmspell-utils, llmspell-testing
- **storage-backends.md**: llmspell-storage implementation guide
- **rag-pipeline.md**: llmspell-rag, llmspell-context, llmspell-graph
- **memory-backends.md**: llmspell-memory backends
- **security-integration.md**: llmspell-security, llmspell-tenancy
- Reference individual crates via cargo doc links

---

### Tier 3: docs/technical/ (ARCHITECTURE/TECHNICAL)

**Purpose**: Deep technical details, architecture decisions, schemas, benchmarks

**Target**: 8-10 technical documents

```
docs/technical/
├── README.md                           # Technical documentation hub
├── architecture.md                     # System architecture (EXISTING: current-architecture.md)
├── storage-architecture.md             # Storage layer design (EXISTING)
├── kernel-execution-paths.md           # Kernel initialization (EXISTING)
├── platform-support.md                 # Cross-platform details (EXISTING)
├── postgresql-schema.md                # MOVED from user-guide/storage/schema-reference.md
├── postgresql-performance.md           # MOVED from user-guide/storage/performance-tuning.md
├── benchmarks.md                       # Performance baselines, stress tests
├── migration-internals.md              # MOVED from user-guide/storage/migration-guide.md
└── design-decisions.md                 # ADRs and design rationale
```

**Moves**:
- storage/schema-reference.md → technical/postgresql-schema.md
- storage/performance-tuning.md → technical/postgresql-performance.md
- storage/migration-guide.md → technical/migration-internals.md
- Keep postgresql-setup.md in user-guide as 07-storage-setup.md (user-facing quick start)

---

## Consolidation Details

### 1. Main User Guide Consolidation

#### Before (15 files):
```
getting-started.md
concepts.md
configuration.md
cli.md
provider-best-practices.md
memory-configuration.md
local-llm.md
ide-integration.md
service-deployment.md
security-and-permissions.md
performance-tuning.md
troubleshooting.md
troubleshooting-phase10.md
README.md
```

#### After (10 files + 2 appendix):
```
README.md                    # Clear user journey
01-getting-started.md        # MERGE: getting-started.md + quick setups
02-core-concepts.md          # MERGE: concepts.md + memory-configuration.md memory sections
03-configuration.md          # MERGE: configuration.md + provider-best-practices.md + local-llm.md config sections
04-lua-scripting.md          # EXTRACT: Essentials from api/lua/README.md (first 500 lines)
05-cli-reference.md          # KEEP: cli.md (already good)
06-templates-and-workflows.md # CONSOLIDATE: templates/README.md + template overview
07-storage-setup.md          # MERGE: storage/postgresql-setup.md + storage/backup-restore.md (user-facing parts)
08-deployment.md             # MERGE: service-deployment.md + ide-integration.md
09-security.md               # KEEP: security-and-permissions.md
10-troubleshooting.md        # MERGE: troubleshooting.md + troubleshooting-phase10.md
appendix/
  lua-api-reference.md       # MOVE: api/lua/README.md (complete reference)
  template-catalog.md        # CONSOLIDATE: templates/*.md (10 detailed template guides)
```

**Deletions**: performance-tuning.md (→ developer-guide)

---

### 2. Storage Documentation Consolidation

#### Before (6 files in storage/):
```
README.md
postgresql-setup.md          # 1,010 lines - USER FACING
schema-reference.md          # 1,359 lines - TECHNICAL
performance-tuning.md        # 961 lines - TECHNICAL
backup-restore.md            # 824 lines - USER FACING
migration-guide.md           # 1,434 lines - TECHNICAL
```

#### After:
**User Guide** (1 file):
```
07-storage-setup.md          # Consolidate: postgresql-setup.md + backup-restore.md (quick start sections)
                             # 600 lines: Quick start, Docker Compose, Connection pooling, Basic backup
```

**Technical Docs** (3 files):
```
technical/postgresql-schema.md       # MOVE: storage/schema-reference.md
technical/postgresql-performance.md  # MOVE: storage/performance-tuning.md
technical/migration-internals.md     # MOVE: storage/migration-guide.md
```

**Deleted**:
- storage/README.md (content merged into 07-storage-setup.md intro)

**Rationale**:
- Users need: "How do I set up PostgreSQL?" (quick start, Docker, backup basics)
- Developers need: Schema details, performance tuning, migration internals
- Separation prevents overwhelming users with 1,359-line schema docs

---

### 3. Templates Documentation Consolidation

#### Before (11 files):
```
README.md                    # 855 lines - Overview
code-generator.md            # 667 lines
code-review.md               # 719 lines
content-generation.md        # 929 lines
data-analysis.md             # 640 lines
document-processor.md        # 676 lines
file-classification.md       # 947 lines
interactive-chat.md          # 628 lines
knowledge-management.md      # 774 lines
research-assistant.md        # 928 lines
workflow-orchestrator.md     # 1,269 lines
```

#### After (2 files):
**User Guide**:
```
06-templates-and-workflows.md   # 600 lines
  - Template system overview
  - How to use templates (CLI commands)
  - Quick start for each of 10 templates (50-60 lines each)
  - When to use which template
  - Customization basics
```

**Appendix**:
```
appendix/template-catalog.md     # 6,000+ lines
  - Detailed docs for all 10 templates
  - Complete configuration reference
  - Advanced customization
  - Integration patterns
```

**Rationale**:
- Most users need: "What templates exist? How do I use them?"
- Power users need: Complete docs for customization
- Reduces decision paralysis (10 files → 1 + appendix)

---

### 4. API Documentation Consolidation

#### Before:
```
api/
├── README.md                # Hub
├── lua/
│   └── README.md            # 3,729 lines - MASSIVE single file
└── rust/
    ├── README.md            # Hub
    ├── llmspell-core.md     # 829 lines
    ├── llmspell-storage.md  # 1,354 lines
    ├── llmspell-bridge.md   # 659 lines
    ├── llmspell-rag.md      # 788 lines
    └── ... 17 more files    # ~10,000 lines total
```

#### After:
**User Guide**:
```
04-lua-scripting.md           # 800 lines
  - Lua scripting essentials
  - 18 global objects overview
  - Common patterns (Agent, Tool, RAG, Memory, Template)
  - Simple examples
  - Links to complete reference
```

**User Guide Appendix**:
```
appendix/lua-api-reference.md  # 3,500 lines
  - Complete Lua API (split from api/lua/README.md)
  - All 18 globals with 200+ methods
  - Organized by category
  - Searchable structure
```

**Developer Guide** (Rust API moved entirely):
```
developer-guide/reference/
├── README.md                 # API reference hub
├── core-traits.md            # BaseAgent, Tool, ExecutionContext
├── storage-backends.md       # Storage abstraction + backends
├── rag-pipeline.md           # RAG, Context, Graph crates
├── memory-backends.md        # Memory system extension
├── security-integration.md   # Security, Tenancy crates
└── crate-index.md            # Quick index to all 21 crates (with cargo doc links)
```

**Rationale**:
- Lua API is USER-FACING (how to write scripts) → stays in user-guide (but split for navigation)
- Rust API is DEVELOPER-FACING (how to extend llmspell) → moves to developer-guide
- 21 crate files → 6 thematic guides (easier to navigate)
- Link to cargo doc for complete API details

---

## Implementation Plan

### Phase 1: Move Developer Content (Low Risk)
**Goal**: Clear separation of user vs developer docs

1. **Create developer-guide/reference/ structure**
   ```bash
   mkdir -p docs/developer-guide/reference
   ```

2. **Move ALL Rust API docs**
   ```bash
   git mv docs/user-guide/api/rust/*.md docs/developer-guide/reference/
   ```

3. **Consolidate 21 crate files → 6 thematic guides**
   - Create core-traits.md, storage-backends.md, rag-pipeline.md, memory-backends.md, security-integration.md, crate-index.md
   - Extract relevant content from individual crate files
   - Delete individual crate files

4. **Move performance-tuning.md**
   ```bash
   git mv docs/user-guide/performance-tuning.md docs/developer-guide/performance-optimization.md
   ```

5. **Update developer-guide/README.md**
   - Add reference/ section
   - Create clear navigation

**Validation**: Developer-facing content separated, user-guide/ is cleaner

---

### Phase 2: Move Technical Content (Medium Risk)
**Goal**: Separate deep technical details from user guides

1. **Move storage technical docs**
   ```bash
   git mv docs/user-guide/storage/schema-reference.md docs/technical/postgresql-schema.md
   git mv docs/user-guide/storage/performance-tuning.md docs/technical/postgresql-performance.md
   git mv docs/user-guide/storage/migration-guide.md docs/technical/migration-internals.md
   ```

2. **Consolidate user-facing storage docs**
   - Create docs/user-guide/07-storage-setup.md
   - Extract quick start sections from postgresql-setup.md
   - Extract basic backup procedures from backup-restore.md
   - Delete storage/README.md, storage/postgresql-setup.md, storage/backup-restore.md

3. **Update technical/README.md**
   - Add PostgreSQL docs section
   - Cross-reference from user guide

**Validation**: Technical details separated, users see only what they need

---

### Phase 3: Consolidate User Guides (High Risk - Most Changes)
**Goal**: Reduce 15 main files → 10 numbered files

1. **Create numbered structure**
   ```bash
   # Rename/consolidate in order
   cp getting-started.md 01-getting-started.md
   cp concepts.md 02-core-concepts.md
   # ... etc
   ```

2. **Merge overlapping files**
   - 10-troubleshooting.md ← troubleshooting.md + troubleshooting-phase10.md
   - 03-configuration.md ← configuration.md + provider-best-practices.md + local-llm.md sections
   - 08-deployment.md ← service-deployment.md + ide-integration.md

3. **Extract Lua API essentials**
   - Create 04-lua-scripting.md (first 500-800 lines of api/lua/README.md)
   - Move complete api/lua/README.md → appendix/lua-api-reference.md

4. **Delete old files**
   ```bash
   git rm troubleshooting-phase10.md provider-best-practices.md memory-configuration.md local-llm.md ide-integration.md
   ```

5. **Update README.md**
   - Clear user journey (01 → 10)
   - Quick links to appendix

**Validation**: User journey is clear, no information loss

---

### Phase 4: Consolidate Templates (Low Risk)
**Goal**: 11 files → 2 files

1. **Create consolidated template guide**
   - docs/user-guide/06-templates-and-workflows.md
   - Extract overview + quick start from each template

2. **Create template catalog**
   - docs/user-guide/appendix/template-catalog.md
   - Consolidate all 10 template detail docs

3. **Delete individual template files**
   ```bash
   git rm docs/user-guide/templates/*.md
   rmdir docs/user-guide/templates
   ```

**Validation**: Templates are easier to discover, details available for power users

---

### Phase 5: Update Navigation & Cross-References (Critical)
**Goal**: Fix all broken links, update navigation

1. **Update all README.md files**
   - docs/README.md (top-level hub)
   - docs/user-guide/README.md
   - docs/developer-guide/README.md
   - docs/technical/README.md

2. **Fix cross-references**
   - Search for all links to moved files
   - Update relative paths
   - Verify cargo doc links work

3. **Add navigation headers**
   - Every file gets: **🔗 Navigation**: [← Previous](link) | [Next →](link)
   - User guides follow 01 → 10 sequence

4. **Create quick reference**
   - Single-page cheat sheet in user-guide/appendix/quick-reference.md
   - Most common tasks, CLI commands, Lua patterns

**Validation**: No broken links, clear navigation

---

## Migration Safety

### Validation Steps

For each phase:
1. **Before**: Git tag current state (`git tag pre-consolidation-phaseN`)
2. **During**: Make changes in feature branch
3. **After**:
   - Run link checker: `find docs -name "*.md" -exec markdown-link-check {} \;`
   - Verify no broken internal links
   - Check file size totals (should be roughly same)
   - Review with: `git diff --stat pre-consolidation-phaseN..HEAD`
4. **Rollback**: If issues: `git reset --hard pre-consolidation-phaseN`

### Content Preservation

- **No content deletion**: All content preserved, just reorganized
- **Git history**: Use `git mv` to preserve file history
- **Deprecation notices**: Old file locations get 3-month deprecation notice
- **Redirects**: Could add redirects in docs/README.md ("Looking for X? It moved to Y")

---

## Expected Outcomes

### Before vs After

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| **User Guide Files** | 53 | 15 | **-71%** |
| **User Guide Main Files** | 15 | 10 | **-33%** |
| **API Docs in User Guide** | 22 | 2 | **-91%** |
| **Storage Files** | 6 | 1 | **-83%** |
| **Template Files** | 11 | 2 | **-82%** |
| **Developer Guide Files** | 2 | 8 | +300% (proper home) |
| **Technical Docs** | 6 | 9 | +50% (proper home) |
| **Total Lines** | 43,335 | ~43,335 | **0% (preserved)** |

### User Experience Improvements

1. **Clear Entry Point**: README → 01 → 02 → 03 ... → 10 (linear path)
2. **Reduced Overwhelm**: 53 → 15 files in user-guide/
3. **Audience Clarity**: User vs Developer vs Technical clearly separated
4. **Easy Discovery**: Numbered guides, appendix for deep dives
5. **Better Search**: Consolidated content easier to search
6. **Faster Onboarding**: New users follow 01-03, skip rest until needed

### Developer Experience Improvements

1. **Rust API Findable**: All in developer-guide/reference/
2. **Thematic Organization**: 6 guides vs 21 files
3. **Extension Patterns**: Clear extending-llmspell.md guide
4. **Performance Docs**: Benchmarking and optimization in one place

### Maintenance Improvements

1. **Less Duplication**: Merged overlapping files
2. **Clear Ownership**: User/Developer/Technical boundaries clear
3. **Easier Updates**: Know where to add new content
4. **Consistent Structure**: Numbered user guides, thematic dev guides

---

## Risks & Mitigation

### Risk 1: Broken Links
**Mitigation**:
- Phase 5 dedicated to link fixing
- Link checker validation
- Deprecation notices for 3 months

### Risk 2: Content Loss During Consolidation
**Mitigation**:
- Git tags before each phase
- Content audit (diff check)
- Appendix preserves all detailed docs

### Risk 3: User Confusion During Transition
**Mitigation**:
- Clear deprecation notices in old files
- "Moved to..." redirects in README
- Phased rollout (developer content first, less disruptive)

### Risk 4: Search Engines Have Old Links
**Mitigation**:
- Keep old file locations with redirect notice for 6 months
- Add canonical URLs
- Update external links (if any)

---

## Success Criteria

### Quantitative
- [ ] User guide files reduced from 53 to ≤15
- [ ] Zero broken internal links
- [ ] All content preserved (line count within 5%)
- [ ] 100% of Rust API docs moved to developer-guide/

### Qualitative
- [ ] New user can follow clear path: 01 → 02 → 03 → first script
- [ ] Developer knows where to find extension docs (developer-guide/)
- [ ] Technical details separated from user guides
- [ ] No duplicate/overlapping content
- [ ] README.md provides clear navigation for all audiences

---

## Timeline

**Phase 1**: Move Developer Content - **2 hours**
**Phase 2**: Move Technical Content - **2 hours**
**Phase 3**: Consolidate User Guides - **4 hours**
**Phase 4**: Consolidate Templates - **2 hours**
**Phase 5**: Update Navigation - **3 hours**

**Total**: ~13 hours

**Recommended Approach**: Execute phases sequentially with validation after each

---

## Appendix: File Mapping

### Complete Before → After Mapping

```
docs/user-guide/getting-started.md → docs/user-guide/01-getting-started.md
docs/user-guide/concepts.md → docs/user-guide/02-core-concepts.md (+ memory-configuration.md sections)
docs/user-guide/configuration.md → docs/user-guide/03-configuration.md (+ provider-best-practices.md + local-llm.md)
docs/user-guide/api/lua/README.md → docs/user-guide/04-lua-scripting.md (essentials) + appendix/lua-api-reference.md (complete)
docs/user-guide/cli.md → docs/user-guide/05-cli-reference.md
docs/user-guide/templates/*.md → docs/user-guide/06-templates-and-workflows.md + appendix/template-catalog.md
docs/user-guide/storage/* → docs/user-guide/07-storage-setup.md + docs/technical/postgresql-*.md
docs/user-guide/service-deployment.md + ide-integration.md → docs/user-guide/08-deployment.md
docs/user-guide/security-and-permissions.md → docs/user-guide/09-security.md
docs/user-guide/troubleshooting.md + troubleshooting-phase10.md → docs/user-guide/10-troubleshooting.md
docs/user-guide/performance-tuning.md → docs/developer-guide/performance-optimization.md
docs/user-guide/api/rust/*.md → docs/developer-guide/reference/*.md (consolidated into 6 thematic guides)
```

### Deleted Files (content merged)
```
docs/user-guide/memory-configuration.md (→ 02-core-concepts.md)
docs/user-guide/provider-best-practices.md (→ 03-configuration.md)
docs/user-guide/local-llm.md (→ 03-configuration.md)
docs/user-guide/ide-integration.md (→ 08-deployment.md)
docs/user-guide/troubleshooting-phase10.md (→ 10-troubleshooting.md)
docs/user-guide/storage/README.md (→ 07-storage-setup.md)
docs/user-guide/storage/postgresql-setup.md (→ 07-storage-setup.md)
docs/user-guide/storage/backup-restore.md (→ 07-storage-setup.md)
docs/user-guide/templates/README.md (→ 06-templates-and-workflows.md)
docs/user-guide/templates/*.md (10 files → appendix/template-catalog.md)
docs/user-guide/api/rust/*.md (21 files → developer-guide/reference/*.md)
```

---

**End of Consolidation Plan**
