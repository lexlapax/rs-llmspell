# rs-llmspell

**Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems 🧙‍♂️✨

> ⚠️ **ALPHA SOFTWARE**: This is version 0.1.0-alpha.1, a development preview release.
> Core features like agent execution are not yet implemented. This release is for
> testing the architecture and providing feedback only. See [Current Status](#current-status) below.

Rs-LLMSpell is a **scriptable LLM interaction framework** that revolutionizes AI application development through multi-language orchestration, comprehensive built-in libraries, and made with a production-readiness mindset.

```lua
-- Example spell: Multi-agent research workflow
local research_spell = Spell.create({
    name = "market_research_analysis",
    
    workflow = Workflow.sequential({
        -- Parallel research gathering
        {
            name = "research_phase",
            type = "parallel",
            agents = {
                { agent = "AcademicResearcher", tools = {"scholarly_search", "pdf_analysis"} },
                { agent = "MarketAnalyst", tools = {"market_data", "trend_analysis"} },
                { agent = "NewsAnalyst", tools = {"news_search", "sentiment_analysis"} }
            }
        },
        -- Synthesis and validation
        {
            name = "synthesis_phase", 
            agent = "SynthesisExpert",
            tools = {"statistical_analysis", "report_generator"}
        }
    }),
    
    error_strategy = ErrorStrategy.cascade({
        retry_count = 3,
        circuit_breaker = { failure_threshold = 5 }
    })
})

-- Execute the spell
local result = research_spell:cast({
    topic = "AI regulation impact on startups",
    output_format = "executive_summary"
})
```

## 🚀 Features

### **🎯 Multi-Language Scripting**
- **Lua**: High-performance scripting with cooperative async
- **JavaScript**: Familiar syntax with Promise-based workflows  
- **Python**: Planned support for data science integration
- **Identical APIs**: Same capabilities across all languages

### **🏗️ Production-Minded Infrastructure**
- **Built-in Hooks & Events**: 20+ hook points for logging, metrics, security
- **State Management**: Persistent agent state with transaction support
- **Circuit Breakers**: Automatic failure recovery and resource protection
- **Observability**: Comprehensive logging, metrics, and distributed tracing

### **📦 Comprehensive Built-in Library**
- **40+ Tools**: File system, web APIs, data processing, AI capabilities
- **Agent Templates**: Research, analysis, coding, customer service patterns
- **Workflow Patterns**: Sequential, parallel, conditional, loop, fan-out, map-reduce
- **Protocol Integration**: MCP (Model Control Protocol), Agent-to-Agent (A2A)

### **⚡ Bridge-First Architecture**
- **LLM Providers**: Unified access via `rig` crate (OpenAI, Anthropic, local models)
- **Storage**: `sled` for development, `rocksdb` for production
- **Script Engines**: `mlua` (Lua), `boa` (JavaScript), `pyo3` (Python)
- **Standing on Giants**: Leverages best-in-class Rust crates

## 🏛️ Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    Script Layer                     │  ← Your Spells (Lua/JS/Python)
├─────────────────────────────────────────────────────┤
│  Bridge Layer: Multi-Language API Unification      │  ← Unified scripting interface
├─────────────────────────────────────────────────────┤
│  Component Layer: Agents │ Tools │ Workflows       │  ← BaseAgent trait hierarchy
├─────────────────────────────────────────────────────┤
│  Infrastructure: Hooks │ Events │ State │ Security  │  ← Production-ready foundation
├─────────────────────────────────────────────────────┤
│  Provider Layer: LLMs │ Storage │ Protocols        │  ← External integrations
└─────────────────────────────────────────────────────┘
```

### **Component Hierarchy**
```
BaseAgent ← Agent ← SpecializedAgent (Research, Analysis, etc.)
    ↑
  Tool ← ToolWrappedAgent (Agents as Tools)
    ↑  
Workflow ← Sequential, Parallel, Conditional, Loop
```

**Key Principle**: Every component implements the same foundational interfaces, enabling seamless composition and orchestration.

## 🛠️ Current Status

### **⚠️ Alpha Release 0.1.0-alpha.1**

**Release Date**: June 27, 2025  
**Status**: Development Preview - Architecture Testing Only

#### **✅ What's Working**
- Lua script execution with ScriptEngineBridge abstraction
- Provider listing (no actual LLM calls)
- CLI with basic commands (`run`, `exec`, `info`, `providers`)
- Streaming infrastructure (stub implementation)
- Multimodal types (structure only)
- Configuration system
- Security sandboxing

#### **❌ What's NOT Working Yet**
- **Agent execution** - `Agent.create()` returns mock data only
- **Tool execution** - Tools cannot be called
- **Workflow orchestration** - Not implemented
- **Actual LLM calls** - Provider integration is listing only
- **JavaScript/Python** - Only Lua is available

#### **📋 Phase Completion Status**
- ✅ Phase 0: Foundation Infrastructure (Complete)
- ✅ Phase 1: Core Execution Runtime (Complete)
- 🔜 Phase 2: Agent Implementation (Next)
- 🔜 Phase 3: Tool System (Future)
- 🔜 Phase 4: Workflow Orchestration (Future)

### **🎯 Safe to Use For**
- Testing the ScriptEngineBridge architecture
- Evaluating Lua script execution performance
- Reviewing API design and providing feedback
- Understanding the project structure
- Contributing to core infrastructure

### **⚠️ NOT Ready For**
- Production use
- Building actual LLM applications
- Agent-based workflows
- Tool integration
- Real LLM API calls

## 🛠️ Project Status

### **🎉 Phase 1 Complete - Architecture Validated**

**Current Status**: Core runtime implemented, ready for agent development
**Latest Update**: 2025-06-27

#### **✅ Phase 0 Achievements**
- 12-crate workspace with zero warnings
- 165 comprehensive tests passing
- Complete CI/CD pipeline operational
- >95% documentation coverage
- Performance benchmarking framework ready

#### **🚀 Phase 1 Starting Now**
- **Goal**: Core Execution Runtime with Lua scripting
- **Timeline**: 10 working days (Weeks 3-4)
- **New Features**: Streaming support, multimodal content, utils crate
- **Deliverables**: Working Lua scripts calling LLM agents

### **🎯 Implementation Roadmap**

**Architecture**: Complete research and design (Phases 1-13) ✅  
**Phase 0**: Foundation Infrastructure ✅ **[COMPLETE]**
**Phase 1**: Core Execution Runtime 🚀 **[IN PROGRESS]**  

#### **Completed Deliverables**
- ✅ **15,034+ line architecture document** with streaming/multimodal updates
- ✅ **12-crate workspace** operational with CI/CD pipeline
- ✅ **Complete trait hierarchy** implemented and tested
- ✅ **165 tests** with >90% coverage
- ✅ **Phase 1 design** and task breakdown ready

#### **Phase 1 Focus (Current)**
- 🔄 `llmspell-utils` crate for shared utilities
- 🔄 Streaming execution support
- 🔄 Multimodal content types
- 🔄 Basic Lua script runtime
- 🔄 CLI with streaming output

**Timeline**: Phase 1 implementation - 10 working days

## 🔮 What Makes Rs-LLMSpell Different?

### **The AI Development Crisis We Solve**
- **Development Velocity Barrier**: Compilation cycles kill AI experimentation
- **Orchestration Complexity**: Multi-agent workflows need sophisticated coordination
- **Language Lock-in**: Teams forced into single-language ecosystems
- **Production Readiness Gap**: Research frameworks lack production infrastructure
- **Integration Fragmentation**: Each provider requires custom integration code

### **Our Solution**
- **🚀 10x Faster Development**: No compilation cycles for AI workflow changes
- **🔧 Production Ready**: Built-in hooks, events, monitoring, and security
- **🌐 Language Agnostic**: Same capabilities across Lua, JavaScript, Python
- **⚡ High Performance**: Rust core with zero-cost abstractions
- **🛡️ Thought-through Security**: Comprehensive threat model and mitigations
- **🔌 Flexible Integration**: Standalone framework or native library

## 💡 Usage Patterns

### **Standalone Framework**
```bash
# Execute spells directly
llmspell run research_analysis.lua --input "climate change impacts"
llmspell run market_analysis.js --config production.toml
```

### **Native Library Integration**
```lua
-- Enhance existing Lua applications
local llmspell = require('llmspell')
llmspell.init_library({
    providers = {"openai", "anthropic"},
    tools = {"web_search", "data_analysis"}
})

-- Now use agents in your existing app
local agent = llmspell.Agent.new("DataAnalyst")
local result = agent:execute(your_existing_data)
```

## 🚀 Quick Start (Post-Implementation)

```bash
# Install rs-llmspell
cargo install llmspell-cli

# Initialize new project
llmspell init my-ai-project --language lua

# Run your first spell
cd my-ai-project
llmspell run examples/hello_world.lua
```

## 🏗️ Development

### **Current Focus: Phase 0 Implementation**
- **Foundation Infrastructure**: Core traits, workspace setup, CI/CD
- **Target**: 2 weeks, 37 specific tasks with acceptance criteria
- **Goal**: Compilable foundation ready for Phase 1 (Agent implementations)

### **Contributing to Phase 0**
```bash
# Get involved in foundation development
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell

# See current Phase 0 tasks
cat TODO.md | grep -A5 "Phase 0.1"

# Development workflow
cargo check --workspace
cargo test --workspace  
cargo clippy -- -D warnings
cargo fmt
```

### **Architecture Documentation**
- **Complete Architecture**: [`docs/technical/rs-llmspell-final-architecture.md`](docs/technical/rs-llmspell-final-architecture.md) (15,034+ lines)
- **Implementation Phases**: [`docs/in-progress/implementation-phases.md`](docs/in-progress/implementation-phases.md)
- **Phase 0 Design**: [`docs/in-progress/phase-00-design-doc.md`](docs/in-progress/phase-00-design-doc.md)

## 📦 Core Technology Stack

### **Performance & Safety**
- **Rust Core**: Memory safety, zero-cost abstractions, fearless concurrency
- **Tokio**: Async runtime with cooperative scheduling for script engines

### **Multi-Language Support**  
- **mlua**: High-performance Lua 5.4 integration with async support
- **boa/quickjs**: JavaScript engines with controlled execution environments
- **pyo3 (future)**: Python integration for data science workflows

### **LLM & Storage**
- **rig**: Multi-provider LLM integration (OpenAI, Anthropic, local models)
- **sled/rocksdb**: Development and production storage backends
- **candle**: Local model inference capabilities

### **Production Infrastructure**
- **tracing**: Structured logging and distributed tracing
- **metrics-rs**: Comprehensive metrics collection
- **serde**: Serialization across language boundaries

## 🤝 Community & Support

### **Getting Help**
- **Architecture Questions**: Review the complete architecture document
- **Implementation**: Track Phase 1 progress in TODO.md
- **Discussion**: GitHub Discussions for design decisions

### **Contributing**
- **Phase 0**: Foundation infrastructure (current focus)
- **Phase 1-3**: MVP implementation (agents, tools, workflows)
- **Phase 4+**: Advanced features and optimization

**Development Philosophy**: Bridge-first design, comprehensive testing, production-ready from day one.

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in rs-llmspell by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

**🧙‍♂️ Ready to cast your first spell?** Rs-LLMSpell transforms AI development from compilation-heavy coding to expressive, multi-language orchestration. Architecture complete - implementation starting now.