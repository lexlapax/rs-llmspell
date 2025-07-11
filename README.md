# rs-llmspell

**Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems 🧙‍♂️✨

> 🎉 **v0.2.0 RELEASED**: Phase 2 complete with 25 self-contained tools implemented!
> Breaking changes coming in v0.3.0 as we standardize all tool interfaces.
> See [Current Status](#current-status) and [Release Notes](CHANGELOG.md) for details.

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
- **25 Tools Available Now** (v0.2.0): File ops, data processing, system integration
- **41+ Tools Coming** (v0.3.0): Adding web scraping, email, databases, API testing
- **Agent Templates**: Research, analysis, coding patterns (coming soon)
- **Workflow Patterns**: Sequential, parallel, conditional, loop (coming v0.3.0)
- **Protocol Integration**: MCP, A2A (future phases)

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

### **🚀 Version 0.2.0 - Phase 2 Complete**

**Release Date**: July 11, 2025  
**Status**: Pre-1.0 Development - 25 Tools Implemented

#### **✅ What's Working**
- **25 Self-Contained Tools** across 6 categories:
  - **Data Processing** (4): JsonProcessor, CsvAnalyzer, HttpRequest, GraphQLQuery
  - **File System** (8): FileOperations, ArchiveHandler, FileWatcher, FileConverter, FileSearch
  - **System Integration** (4): EnvironmentReader, ProcessExecutor, ServiceChecker, SystemMonitor
  - **Media Processing** (3): AudioProcessor, VideoProcessor, ImageProcessor
  - **Utilities** (8): Calculator, TextManipulator, DateTimeHandler, UuidGenerator, HashCalculator, Base64Encoder, DiffCalculator
  - **Search** (1): WebSearch (basic implementation)
- **Lua script execution** with full tool access
- **Provider enhancements**: ModelSpecifier, base URL overrides
- **JSON API** for seamless script-tool communication
- **CLI** with streaming output and progress indicators
- **90%+ test coverage** with zero warnings

#### **🔄 Coming in v0.3.0 (Breaking Changes)**
- **Tool Standardization**: All 25 tools will use consistent parameter names
- **ResponseBuilder Pattern**: Unified response format across all tools
- **16 New External Tools**: WebSearchTool enhancement, email, database connectors
- **Security Hardening**: DoS protection, path traversal prevention
- **Workflow Orchestration**: Sequential, conditional, loop patterns

#### **📋 Phase Completion Status**
- ✅ Phase 0: Foundation Infrastructure (Complete)
- ✅ Phase 1: Core Execution Runtime (Complete)
- ✅ Phase 2: Self-Contained Tools Library (Complete - v0.2.0)
- 🚀 Phase 3: Tool Enhancement & Workflow Orchestration (Starting - 8 weeks)
- 🔜 Phase 4: Vector Storage and Search (Future)

### **🎯 Ready For**
- Building tool-based automation scripts
- File system operations and data processing
- System integration and monitoring
- Basic media file processing
- Testing the tool ecosystem

### **⚠️ Still In Development**
- Agent execution with actual LLM calls
- Workflow orchestration patterns
- JavaScript/Python support
- External API integrations (coming in v0.3.0)
- Production deployment features

## 🛠️ Project Timeline

### **🎉 Phase 2 Complete - v0.2.0 Released**

**Release Date**: 2025-07-11
**Achievement**: 25 self-contained tools implemented and tested

#### **✅ Completed Phases**
- ✅ **Phase 0**: Foundation Infrastructure (13-crate workspace, CI/CD)
- ✅ **Phase 1**: Core Execution Runtime (ScriptEngineBridge, Lua integration)
- ✅ **Phase 2**: Self-Contained Tools Library (25 tools, 90%+ coverage)

#### **🚀 Phase 3 Starting - Tool Enhancement & Workflow**
- **Timeline**: 8 weeks (Weeks 9-16)
- **Sub-phases**:
  - **3.0**: Critical Tool Fixes - Standardization & DRY (Weeks 9-10)
  - **3.1**: External Integration Tools - 16 new tools (Weeks 11-12)
  - **3.2**: Security & Performance - Hardening all 41 tools (Weeks 13-14)
  - **3.3**: Workflow Orchestration - Patterns & engine (Weeks 15-16)
- **Breaking Changes**: Clean break approach (no migration tools)
- **Target**: 41+ production-ready tools with workflow support

### **🎯 Implementation Roadmap**

**16-Phase Journey**: From MVP Foundation to Production Platform

#### **MVP Foundation (Phases 0-3)**
- ✅ **Phase 0**: Foundation Infrastructure - Complete
- ✅ **Phase 1**: Core Execution Runtime - Complete  
- 🔄 **Phase 2**: Built-in Tools Library - In Progress
- 🔜 **Phase 3**: Workflow Orchestration - Next

#### **Production Features (Phases 4-7)**
- **Phase 4**: Hook and Event System
- **Phase 5**: JavaScript Engine Support
- **Phase 6**: REPL Interactive Mode
- **Phase 7**: Persistent State Management

#### **Advanced Integration (Phases 8-12)**
- **Phase 8**: Daemon and Service Mode
- **Phase 9**: MCP Tool Integration
- **Phase 10**: MCP Server Mode
- **Phase 11**: A2A Client Support
- **Phase 12**: A2A Server Support

#### **Platform Support (Phases 13-15)**
- **Phase 13**: Library Mode Support
- **Phase 14**: Cross-Platform Support
- **Phase 15**: Production Optimization

**Timeline**: MVP (Phases 0-3) - 8 weeks total, Production Ready - 16 weeks

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

## 📦 Available Tools (v0.2.0)

### Data Processing
- **JsonProcessor**: Query and transform JSON with JQ syntax
- **CsvAnalyzer**: Parse, analyze, and transform CSV data
- **HttpRequest**: Make HTTP requests with full control
- **GraphQLQuery**: Execute GraphQL queries and mutations

### File System  
- **FileOperations**: Read, write, copy, move files
- **ArchiveHandler**: Create and extract archives (zip, tar)
- **FileWatcher**: Monitor file system changes
- **FileConverter**: Convert between file formats
- **FileSearch**: Search files with patterns

### System Integration
- **EnvironmentReader**: Access environment variables
- **ProcessExecutor**: Run system commands safely
- **ServiceChecker**: Check service availability
- **SystemMonitor**: Get system resource info

### Utilities
- **Calculator**: Evaluate mathematical expressions
- **TextManipulator**: Transform and analyze text
- **DateTimeHandler**: Parse and format dates/times
- **UuidGenerator**: Generate various UUID formats
- **HashCalculator**: Compute cryptographic hashes
- **Base64Encoder**: Encode/decode base64
- **DiffCalculator**: Compute text differences

## 🚀 Getting Started

### Installation

```bash
# From source (recommended for now)
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Quick Example

```lua
-- file_processor.lua
local file_tool = Tool.load("file_operations")
local json_tool = Tool.load("json_processor")

-- Read a JSON file
local content = file_tool:execute({
    operation = "read",
    path = "data.json"
})

-- Process the JSON
local result = json_tool:execute({
    operation = "query",
    input = content.output,
    query = ".users[] | select(.active)"
})

print("Active users:", result.output)
```

### Run Your Script

```bash
# Execute with the CLI
llmspell run file_processor.lua

# With streaming output
llmspell exec -s "print(Tool.list())"
```

## 🏗️ Development

### **Contributing**

```bash
# Clone and build
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell

# Run quality checks
./scripts/quality-check-minimal.sh  # Quick checks
./scripts/quality-check-fast.sh     # With unit tests
./scripts/quality-check.sh          # Full validation

# Current development focus
cat docs/in-progress/PHASE03-TODO.md  # Phase 3 tasks
```

### **Phase 3 Development (v0.3.0)**
- **Breaking Changes**: Tool parameter standardization
- **16 New Tools**: External integrations (web, email, databases)
- **Security Hardening**: DoS protection, resource limits
- **Workflow Engine**: Orchestration patterns
- **Clean Break**: No migration tools (pre-1.0 freedom)

### **Architecture Documentation**
- **Complete Architecture**: [`docs/technical/rs-llmspell-final-architecture.md`](docs/technical/rs-llmspell-final-architecture.md) (15,034+ lines)
- **Implementation Phases**: [`docs/in-progress/implementation-phases.md`](docs/in-progress/implementation-phases.md) (16-phase roadmap)
- **Phase 3 Design**: [`docs/in-progress/phase-03-design-doc.md`](docs/in-progress/phase-03-design-doc.md) (Current focus)
- **Phase 3 Tasks**: [`docs/in-progress/PHASE03-TODO.md`](docs/in-progress/PHASE03-TODO.md) (40 tasks over 8 weeks)
- **Breaking Changes**: Clean break approach for v0.3.0

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
- **Phase 0-1**: Foundation and core runtime ✅ Complete
- **Phase 2**: Built-in tools library 🔄 Current focus
- **Phase 3**: Workflow orchestration (Next up)
- **Phase 4-7**: Production features (hooks, JavaScript, REPL, state)
- **Phase 8-12**: Advanced integrations (MCP, A2A protocols)
- **Phase 13-15**: Platform support and optimization

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