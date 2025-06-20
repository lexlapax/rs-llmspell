# CLAUDE.md

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🚧 **Phase 5B ACTIVE**: Research Existing Crate Ecosystem
- **Completed**: Phase 5 - All integration analysis including async patterns
- **Current**: Task 5B.1 - **CRITICAL** LLM Provider Layer Crates Research (rust-genai evaluation)
- **Next**: Task 5B.2 - Scripting Engine Crates Evaluation

**Architecture Progress**: 10/14 phases completed in research/design cycle
- ✅ Phase 1: Research Foundation (go-llms, ADK, state management, Rust patterns)
- ✅ Phase 2: Analyze Current State (architecture mapping, scripting implications)  
- ✅ Phase 3: Synthesize Core Architecture (hierarchy design, hooks/events)
- ✅ Phase 4: Research Implementation Patterns (built-in components, orchestration)
- ✅ Phase 5: Analyze Integration Points (bridge analysis, async patterns completed)
- 🚧 Phase 5B: Research Existing Crate Ecosystem (LLM providers, script engines, infrastructure)

## Architecture

**Core Principle**: Bridge-first scriptable interface for LLM interactions
**New Critical Focus**: BaseAgent/Agent/Tool/Workflow hierarchy with state-based agent handoff and async patterns for single-threaded scripting engines

### Component Hierarchy
- **BaseAgent**: Tool-handling foundation with state/hook management
- **Agent**: LLM wrapper with specialized prompts, uses multiple tools  
- **Tool**: LLM-callable functions, can wrap agents
- **Workflow**: Deterministic agents (sequential, parallel, conditional, loop)
- **Built-in Components**: 40+ tools, agent templates, custom workflows

### Async Patterns (NEW CRITICAL AREA)
- **Lua**: Coroutines, cooperative scheduling, yield-based programming
- **JavaScript**: Promise/Future abstractions, event loop integration with Tokio
- **Cross-Engine**: Unified async interface, non-blocking execution patterns

## Implementation Workflow

1. **Be thorough** - No shortcuts or deferrals. Ask questions when needed
2. **TDD mandatory** - Write tests first, use testutils  
3. **Bridge-first** - Wrap underlying crates, never reimplement
4. **Research rig or llm wrapper libraries** - Use existing Rust LLM libs, don't reinvent
5. **Async-aware design** - Account for single-threaded script engine limitations
6. **Update TODO.md** - Mark tasks with timestamps, track async pattern progress
7. **Run tests and linting** - Ensure code quality

## Commands

```bash
cargo test     # Run test suite
cargo clippy   # Lint and check
cargo fmt      # Format code  
cargo build    # Build project
```

## Key Reminders

- **Complete tasks fully** - No lazy implementations or deferrals
- **Async patterns critical** - Single-threaded script engines need cooperative scheduling
- **Bridge everything** - If it's in underlying library, bridge it - don't reimplement
- **State-first architecture** - Agent handoff via shared state, not just messages
- **Tool-wrapped agents** - Agents can be wrapped as tools for composition
- **No backward compatibility** requirements until we reach version 1.0.0
- Do what's asked; nothing more, nothing less
- Prefer editing existing files over creating new ones
- Update TODO.md with task completion timestamps as tasks get completed
- **Created documentation should be put in `/docs/technical/`**

## Current Research Documents

- `/docs/technical/architecture_mapping.md` - Current vs target architecture gaps
- `/docs/technical/state_management_research.md` - Agent handoff patterns
- `/docs/technical/core_hierarchy_design.md` - BaseAgent/Agent/Tool/Workflow design  
- `/docs/technical/rust_patterns_research.md` - Event hooks and async patterns
- `/docs/technical/scripting_interface_analysis.md` - Bridge layer implications
- `/docs/technical/builtin_components_research.md` - 40+ built-in tools
- `/docs/technical/composition_orchestration_research.md` - Multi-agent workflows
- `/docs/technical/bridge_integration_analysis.md` - Bridge redesign for new hierarchy