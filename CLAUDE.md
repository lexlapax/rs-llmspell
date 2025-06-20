# CLAUDE.md

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

✅ **Phase 12 COMPLETE**: Architecture Documentation Finalized
- **Completed**: All research and documentation phases (1-12.2)
- **Delivered**: `/docs/rs-llmspell-complete-architecture.md` - 15,034+ line standalone guide
- **Next**: Phase 13 - Implementation roadmap

## Architecture

**Core Principle**: Bridge-first scriptable interface for LLM interactions
**Component Hierarchy**: BaseAgent/Agent/Tool/Workflow with state-based handoff

### Key Design Elements
- **BaseAgent**: Tool-handling foundation with state/hook management
- **Agent**: LLM wrapper with specialized prompts  
- **Tool**: LLM-callable functions, can wrap agents
- **Workflow**: Deterministic orchestration patterns
- **Built-in Library**: 40+ tools, agent templates, workflows
- **Async Patterns**: Coroutines (Lua), Promises (JS), unified interface

### Technology Stack
- **LLM Providers**: rig + candle for local models
- **Scripting**: mlua (Lua 5.4), boa/quickjs (JavaScript)
- **Storage**: sled (dev) / rocksdb (prod)
- **Events**: tokio-stream + crossbeam hybrid
- **Testing**: mockall + proptest + criterion
- **Observability**: tracing + metrics-rs

## Implementation Workflow

1. **Bridge-first** - Wrap existing crates (rig, mlua, sled)
2. **TDD mandatory** - Tests before implementation  
3. **Async-aware** - Cooperative scheduling for script engines
4. **Update TODO.md** - Track progress with timestamps
5. **Test and lint** - `cargo test && cargo clippy && cargo fmt`

## Key Reminders

- **Complete tasks fully** - No deferrals or lazy implementations
- **State-first architecture** - Agent handoff via shared state
- **Tool-wrapped agents** - Agents as composable tools
- **No backward compatibility** until v1.0.0
- Update TODO.md with completion timestamps

## Primary Documentation

**🎯 Complete Architecture**: `/docs/rs-llmspell-complete-architecture.md`
- Standalone 15,034+ line comprehensive guide
- Synthesizes all 30+ technical documents
- Production-ready specifications with examples
- No external references required

**Master Document**: `/docs/technical/architecture.md`
- Original architecture specification
- Updated with Phase 1-8 refinements

**Research Documents**: 30+ documents in `/docs/technical/`
- Organized by phase (1-11)
- Covers all architectural decisions
- Implementation ready