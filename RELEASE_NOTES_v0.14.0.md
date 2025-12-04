# Release Notes: v0.14.0 - Usability & Cohesion Refinement

**Phase 13c Complete** - Production-ready developer experience through consolidation and quality enhancement.

## Highlights

### Profile System Standardization
- **21 Preset Profiles**: Complete 4-layer architecture for any deployment scenario
- **Layer Architecture**: bases (3) → backends (7) → features (4) → envs (1) → presets (21)
- **Decision Matrix**: Clear profile selection guide for different use cases
- **Zero-Config Deployment**: Pick a preset and start scripting

### Example Consolidation
- **56 Lua Examples**: All standardized with consistent headers
- **Getting-Started Path**: 6 progressive examples for <30 minute onboarding
- **examples-validation.sh**: Automated testing ensures all examples work
- **Profile Recommendations**: Every example specifies its optimal profile

### Quality Infrastructure
- **5540 Tests**: 100% passing across all crates
- **Zero Clippy Warnings**: Clean codebase with strict linting
- **Comprehensive Validation**: quality-check-minimal.sh and quality-check-fast.sh

## Metrics

| Category | Metric | Value |
|----------|--------|-------|
| **Profiles** | Preset profiles | 21 |
| **Examples** | Lua scripts | 56 |
| **Examples** | Rust examples | 3 |
| **Examples** | Getting-started | 6 |
| **Tests** | Total tests | 5540 |
| **Binary** | Release size | 44MB |
| **Build** | Release time | 11m 34s |

## Phase 13c Tasks Completed

- **13c.1**: Dependencies (completed earlier)
- **13c.2**: Storage Architecture (completed earlier)
- **13c.3**: Legacy Cleanup (completed earlier)
- **13c.4**: Profile Standardization ✅
- **13c.5**: Example Consolidation ✅
- **13c.6**: Validation Scripts ✅
- **13c.7**: Documentation Overhaul ✅
- **13c.8**: Integration Testing & Release ✅

## Getting Started

```bash
# Quick start with minimal profile
./target/debug/llmspell -p minimal run examples/script-users/getting-started/00-hello-world.lua

# Memory features
./target/debug/llmspell -p memory run examples/script-users/getting-started/05-memory-rag-advanced.lua

# Full getting-started path
ls examples/script-users/getting-started/
```

## What's Next

**Phase 14**: MCP Tool Integration
**Phase 15+**: Advanced integrations

See [implementation-phases.md](docs/in-progress/implementation-phases.md) for roadmap details.

## Upgrade Notes

- No breaking changes from v0.13.1
- All existing scripts and configurations work unchanged
- New preset profiles available for simpler configuration
