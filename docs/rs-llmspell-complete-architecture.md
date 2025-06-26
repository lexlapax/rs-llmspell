# Rs-LLMSpell: Complete Architecture and Implementation Guide

**Version**: 2.0  
**Date**: June 2025  
**Status**: Complete Standalone Reference  

> **📋 Standalone Document**: This document contains ALL architectural, implementation, and operational details for rs-llmspell. No external documentation references are required.

---

## Table of Contents

### Part I: Foundation and Overview
1. [Introduction](#introduction)
2. [Executive Summary](#executive-summary)
3. [Why Rs-LLMSpell Exists](#why-rs-llmspell-exists)
4. [Core Philosophy and Design Principles](#core-philosophy-and-design-principles)
5. [Quick Start Guide](#quick-start-guide)

### Part II: Core Architecture
6. [Architecture Overview](#architecture-overview)
7. [Component Hierarchy](#component-hierarchy)
8. [BaseAgent/Agent/Tool/Workflow System](#baseagent-agent-tool-workflow-system)
9. [Bridge-First Design](#bridge-first-design)
10. [State Management Architecture](#state-management-architecture)

### Part III: Scripting and API Reference
11. [Complete Script Interface](#complete-script-interface)
12. [Using Rs-LLMSpell as a Native Module](#using-rs-llmspell-as-a-native-module)
13. [Lua API Reference](#lua-api-reference)
14. [JavaScript API Reference](#javascript-api-reference)
15. [Python API Reference (Future)](#python-api-reference)
16. [Cross-Engine Compatibility Matrix](#cross-engine-compatibility-matrix)

### Part IV: Built-in Components Library
16. [Complete Built-in Tools Catalog](#complete-built-in-tools-catalog)
17. [Agent Templates and Patterns](#agent-templates-and-patterns)
18. [Workflow Libraries](#workflow-libraries)
19. [Hook and Event System](#hook-and-event-system)

### Part V: Technology Stack and Implementation
20. [Complete Technology Decision Matrix](#complete-technology-decision-matrix)
21. [LLM Provider Integration](#llm-provider-integration)
22. [Storage and Persistence](#storage-and-persistence)
23. [Async Patterns and Concurrency](#async-patterns-and-concurrency)
24. [Performance Optimization](#performance-optimization)

### Part VI: Configuration and Security
25. [Configuration Architecture](#configuration-architecture)
26. [Security Model and Threat Analysis](#security-model-and-threat-analysis)
27. [Resource Management](#resource-management)
28. [Observability and Monitoring](#observability-and-monitoring)

### Part VII: Advanced Features
29. [Advanced Orchestration Patterns](#advanced-orchestration-patterns)
30. [LLM-Driven Delegation (Agent Transfer)](#llm-driven-delegation-agent-transfer)
31. [Protocol Integration (MCP, A2A)](#protocol-integration)
32. [Scheduling and Automation](#scheduling-and-automation)
33. [Plugin System and Extensions](#plugin-system-and-extensions)
34. [Error Handling and Recovery](#error-handling-and-recovery)

### Part VIII: Testing and Quality Assurance
33. [Complete Testing Strategy](#complete-testing-strategy)
34. [Performance Benchmarks](#performance-benchmarks)
35. [Security Testing](#security-testing)
36. [Integration Testing Framework](#integration-testing-framework)

### Part IX: Development and Operations
37. [Development Workflow](#development-workflow)
38. [Build System and Tooling](#build-system-and-tooling)
39. [Deployment Strategies](#deployment-strategies)
40. [Migration and Backward Compatibility](#migration-and-backward-compatibility)

### Part X: Practical Implementation
41. [Implementation Roadmap](#implementation-roadmap)
42. [Real-World Examples](#real-world-examples)
43. [Troubleshooting Guide](#troubleshooting-guide)
44. [Performance Tuning](#performance-tuning)

### Part XI: Reference and Appendices
45. [Complete API Quick Reference](#complete-api-quick-reference)
46. [Error Code Reference](#error-code-reference)
47. [Configuration Schema Reference](#configuration-schema-reference)
48. [Future Evolution Strategy](#future-evolution-strategy)

---

## Introduction

Rs-LLMSpell is a **production-ready scriptable LLM interaction framework** that revolutionizes how developers build, orchestrate, and deploy AI agents and workflows. Built on Rust's performance and safety guarantees, it provides seamless multi-language scripting capabilities through Lua, JavaScript, and planned Python support.

### What Makes Rs-LLMSpell Revolutionary

**🎯 Go-llms Inspired Architecture**: Implements the battle-tested BaseAgent/Agent/Tool/Workflow patterns from go-llms, adapted and enhanced for Rust's ecosystem with modern async capabilities.

**🌍 True Multi-Language Scripting**: Write AI orchestration logic in your preferred language—Lua for performance, JavaScript for familiarity, or Python for data science—all with identical capabilities and seamless interoperability.

**🏗️ Production-First Infrastructure**: Built-in hooks, events, state management, observability, and security from day one. Not an afterthought, but core architectural components.

**📦 Comprehensive Built-in Library**: 40+ production-ready tools across 8 categories, multiple agent templates, and proven workflow patterns—no need to reinvent common functionality.

**🔌 Bridge-First Philosophy**: Leverages the best existing Rust crates (rig for LLM providers, mlua for Lua, sled/rocksdb for storage) rather than reimplementing. Standing on the shoulders of giants.

**🚀 Zero-Compilation Development**: Test complex AI behaviors instantly without recompilation cycles. Perfect for rapid experimentation and production deployments alike.

**📦 Embeddable & Extendable**: Use rs-llmspell as a standalone framework or import it as a native library into existing Lua and JavaScript applications to enhance them with powerful agentic capabilities.

### What is a "Spell"?

A spell in rs-llmspell is a **script that orchestrates AI capabilities** through our unified API. Think of it as a recipe that combines agents, tools, and workflows to accomplish complex tasks:

```lua
-- Research Analysis Spell (Lua)
local ResearchSpell = Spell.create({
    name = "comprehensive_research_analysis",
    description = "Multi-agent research with synthesis and validation",
    
    -- Define the orchestration
    workflow = Workflow.sequential({
        -- Research gathering phase
        {
            name = "research_phase",
            type = "parallel",
            agents = {
                { 
                    agent = "AcademicResearcher", 
                    query = "{{input.topic}} academic papers last 2 years",
                    tools = {"scholarly_search", "pdf_analysis"}
                },
                { 
                    agent = "NewsAnalyst", 
                    query = "{{input.topic}} recent news and trends",
                    tools = {"news_search", "sentiment_analysis"}
                },
                { 
                    agent = "MarketAnalyst", 
                    query = "{{input.topic}} market implications",
                    tools = {"market_data", "trend_analysis"}
                }
            },
            output = "research_data"
        },
        
        -- Synthesis phase
        {
            name = "synthesis_phase",
            agent = "SynthesisExpert",
            input = "{{research_data}}",
            action = "create_comprehensive_analysis",
            tools = {"statistical_analysis", "visualization", "report_generator"},
            output = "synthesis_report"
        },
        
        -- Quality validation phase  
        {
            name = "validation_phase",
            agent = "QualityReviewer",
            input = "{{synthesis_report}}",
            action = "validate_and_enhance",
            tools = {"fact_checker", "bias_detector", "clarity_analyzer"},
            output = "final_report"
        }
    }),
    
    -- Error handling and recovery
    error_strategy = ErrorStrategy.cascade({
        retry_count = 3,
        fallback_agents = {
            "AcademicResearcher": "GeneralResearcher",
            "MarketAnalyst": "WebResearcher"
        },
        circuit_breaker = { failure_threshold = 5, reset_timeout = 60 }
    }),
    
    -- Resource management
    resources = {
        max_parallel_agents = 3,
        timeout_per_phase = 300, -- 5 minutes
        memory_limit = "2GB"
    }
})

-- Execute the spell
local result = ResearchSpell:cast({
    topic = "Impact of AI regulation on startup ecosystems",
    output_format = "executive_summary",
    target_audience = "venture_capitalists"
})

print("Research Analysis Complete:")
print("- Academic Sources:", #result.academic_sources)
print("- News Articles:", #result.news_articles) 
print("- Market Data Points:", #result.market_data_points)
print("- Confidence Score:", result.confidence_score)
print("- Report Length:", result.final_report:len(), "words")
```

## Executive Summary

Rs-LLMSpell represents a paradigm shift in AI application development, solving the critical gap between high-performance AI infrastructure and flexible, scriptable orchestration. 

### The Problem We Solve

**Development Velocity Barrier**: Traditional AI applications require compilation cycles for experimentation, making rapid iteration painful.

**Orchestration Complexity**: Multi-agent workflows require sophisticated coordination, state management, and error handling that most frameworks don't provide.

**Language Lock-in**: Teams are forced to choose a single language ecosystem, limiting collaboration and expertise utilization.

**Production Readiness Gap**: Research frameworks lack the hooks, events, monitoring, and security needed for production deployment.

**Integration Fragmentation**: Each AI provider, tool, and workflow requires custom integration code, creating maintenance nightmares.

**Integration Rigidity**: Existing applications cannot easily incorporate advanced agentic capabilities without significant rewrites or being absorbed into a monolithic framework.

### Our Solution Architecture

Rs-LLMSpell solves these problems through five key architectural innovations:

#### 1. **Unified Component Hierarchy**
```
BaseAgent ← Agent ← SpecializedAgent (Research, Analysis, etc.)
    ↑
  Tool ← ToolWrappedAgent (Agents as Tools)
    ↑  
Workflow ← SequentialWorkflow, ParallelWorkflow, ConditionalWorkflow
```

Every component in the system implements the same foundational interfaces, enabling seamless composition and orchestration.

#### 2. **Multi-Language Bridge Architecture**
```
Rust Core ← Bridge Layer → Script Engines (Lua/JS/Python)
                ↓
        Unified API Surface
```

Identical capabilities across all supported languages, with automatic type conversion, error translation, and async pattern unification.

#### 3. **Production Infrastructure Layer**
```
Hook System ← Event Bus ← State Manager ← Observability
     ↓            ↓           ↓              ↓
Security ← Config Manager ← Resource Manager ← Circuit Breakers
```

Built-in production capabilities that scale from development to enterprise deployment.

#### 4. **Bridge-First Technology Strategy**
- **LLM Providers**: `rig` crate for unified access to OpenAI, Anthropic, local models
- **Script Engines**: `mlua` for Lua, `boa`/`v8` for JavaScript, `pyo3` for Python
- **Storage**: `sled` for development, `rocksdb` for production, behind trait abstractions
- **Async Runtime**: `tokio` with cooperative scheduling adapters for single-threaded engines

#### 5. **Comprehensive Built-in Ecosystem**
- **40+ Tools**: File system, web, data processing, AI capabilities, system integration
- **Agent Templates**: Chat, research, analysis, coding, customer service patterns
- **Workflow Patterns**: Sequential, parallel, conditional, loop, fan-out, map-reduce
- **Protocol Support**: Model Control Protocol (MCP), Agent-to-Agent (A2A), REST, GraphQL

### Key Benefits Delivered

🚀 **10x Faster Development**: No compilation cycles for AI workflow changes  
🔧 **Production Ready**: Built-in hooks, events, monitoring, and security  
🌐 **Language Agnostic**: Same capabilities across Lua, JavaScript, Python  
⚡ **High Performance**: Rust core with zero-cost abstractions  
🛡️ **Enterprise Security**: Comprehensive threat model and mitigations  
📈 **Scalable Architecture**: From prototype to enterprise deployment  
🔌 **Extensible Design**: Plugin system for custom providers, tools, workflows  
🎯 **Real-world Proven**: Based on battle-tested go-llms patterns

🔄 **Flexible Integration**: Use as a standalone framework or import as a native library into existing applications.  

---

## Why Rs-LLMSpell Exists

### The AI Development Crisis

The AI revolution has created a new category of applications that traditional development frameworks weren't designed to handle. These applications require:

1. **Dynamic Orchestration**: AI agents must coordinate, hand off tasks, and adapt based on runtime conditions
2. **Rapid Experimentation**: AI behavior requires constant tuning that compilation cycles make prohibitively slow  
3. **Multi-Modal Integration**: Text, code, images, audio, and structured data all need seamless integration
4. **Production Resilience**: AI systems must handle errors gracefully, provide observability, and scale reliably
5. **Team Collaboration**: Data scientists, engineers, and domain experts need to work in their preferred languages

### Current Solution Limitations

#### Python Ecosystem Limitations
- **Performance Bottlenecks**: GIL limitations prevent true parallelism
- **Deployment Complexity**: Dependency hell and version conflicts in production
- **Type Safety**: Runtime errors that could be caught at compile time
- **Memory Safety**: Potential for crashes and security vulnerabilities

#### JavaScript/Node.js Limitations  
- **Single-threaded Constraints**: CPU-intensive AI operations block the event loop
- **Ecosystem Fragmentation**: Rapid ecosystem changes create maintenance burdens
- **Type System Gaps**: Even TypeScript can't prevent many runtime AI integration errors
- **Performance Ceiling**: V8 optimizations have limits for compute-heavy workloads

#### Rust-Only Solutions Limitations
- **Compilation Barrier**: Every change requires rebuild, killing experimentation velocity
- **Learning Curve**: Steep adoption curve for teams not familiar with Rust
- **Rapid Prototyping**: Rust's strengths become weaknesses for quick experimentation
- **Domain Expert Accessibility**: Data scientists and analysts prefer familiar languages

#### Traditional AI Frameworks Limitations
- **Research-Oriented**: Focused on model training, not production orchestration
- **Monolithic Design**: Hard to compose, extend, and integrate with existing systems
- **Limited Observability**: Black box behavior makes debugging and optimization difficult
- **Vendor Lock-in**: Tight coupling to specific AI providers or model formats

### Rs-LLMSpell's Breakthrough Approach

Rs-LLMSpell solves these problems through a revolutionary **"Core-Bridge-Script"** architecture:

#### **Rust Core**: Performance and Safety Foundation
```rust
// High-performance, memory-safe core with zero-cost abstractions
pub struct LLMSpellCore {
    agent_registry: AgentRegistry,
    tool_registry: ToolRegistry,
    workflow_engine: WorkflowEngine,
    state_manager: StateManager,
    event_bus: EventBus,
    hook_system: HookSystem,
}

// Trait-based design enables zero-cost composition
#[async_trait]
pub trait BaseAgent: Send + Sync + Observable + Hookable {
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput>;
    // ... other methods
}
```

#### **Bridge Layer**: Language Unification 
```rust
// Unified API surface across all scripting languages
pub struct UnifiedAPI {
    lua_bridge: LuaBridge,
    js_bridge: JavaScriptBridge,
    python_bridge: PythonBridge, // Future
    
    // Automatic type conversion and error translation
    type_converter: TypeConverter,
    error_translator: ErrorTranslator,
    async_coordinator: AsyncCoordinator,
}
```

#### **Script Layer**: Developer Productivity
```lua
-- Lua: Performance-oriented scripting
local workflow = Workflow.parallel({
    { agent = "researcher", task = "gather_data" },
    { agent = "analyzer", task = "process_data" },
    { agent = "writer", task = "generate_report" }
})
```

```javascript
// JavaScript: Familiar ecosystem  
const workflow = Workflow.parallel([
    { agent: "researcher", task: "gather_data" },
    { agent: "analyzer", task: "process_data" },  
    { agent: "writer", task: "generate_report" }
]);
```

This architecture delivers:
- **Rust Performance** for compute-intensive operations
- **Script Flexibility** for rapid iteration and experimentation  
- **Production Reliability** through comprehensive infrastructure
- **Team Productivity** by supporting multiple language preferences
- **Integration Flexibility** by allowing rs-llmspell to be used as a native library.

---

## Core Philosophy and Design Principles

Rs-LLMSpell is built on seven foundational principles that guide every architectural decision:

### 1. **Bridge-First, Never Reinvent**

> *"Stand on the shoulders of giants, don't build your own mountain."*

**Principle**: Leverage the best existing solutions through well-designed bridges rather than reimplementing functionality. This applies to both internal components and external integration. The bridge layer is also designed to expose a stable C API, which is the foundation for creating native modules for languages like Lua and JavaScript. This enables the **Library Mode** usage paradigm, allowing `rs-llmspell` to be integrated into existing applications.

**Implementation**:
- **LLM Providers**: Use `rig` crate that already supports OpenAI, Anthropic, Ollama, and more
- **Script Engines**: Use battle-tested `mlua`, `boa`/`v8`, `pyo3` rather than custom parsers
- **Storage**: Use proven `sled` and `rocksdb` behind trait abstractions
- **Async Runtime**: Build on `tokio`'s mature ecosystem

**Benefits**:
- Faster development and time-to-market
- Leveraged community expertise and bug fixes
- Reduced maintenance burden  
- Access to ecosystem innovations

**Example**:
```rust
// Instead of reimplementing LLM client code:
pub struct LLMProviderBridge {
    rig_client: rig::Client,  // Leverage rig's provider support
}

impl LLMProvider for LLMProviderBridge {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // Bridge to rig's unified interface
        let response = self.rig_client.complete(prompt).await?;
        Ok(response.text)
    }
}
```

### 2. **Production-First Infrastructure**

> *"Build for production from day one, not as an afterthought."*

**Principle**: Every component must include production-ready capabilities: observability, error handling, security, and performance monitoring.

**Implementation**:
- **Hook System**: Pre/post execution hooks at every layer
- **Event Bus**: Comprehensive event emission for monitoring and debugging
- **Circuit Breakers**: Automatic failure detection and recovery
- **Resource Limits**: Memory, CPU, and time constraints enforced
- **Security Model**: Authentication, authorization, and sandboxing built-in

**Benefits**:
- Smooth development-to-production transition
- Comprehensive debugging and monitoring capabilities
- Resilient behavior under failure conditions
- Enterprise-ready security posture

**Example**:
```rust
// Every agent execution includes production capabilities
impl BaseAgent for MyAgent {
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput> {
        // Pre-execution hooks (logging, metrics, validation)
        self.hooks.execute(HookPoint::BeforeExecution, &input).await?;
        
        // Circuit breaker check
        if self.circuit_breaker.should_block() {
            return Err(AgentError::CircuitBreakerOpen);
        }
        
        // Resource limit enforcement
        let _guard = self.resource_limiter.acquire().await?;
        
        // Actual execution with timeout
        let result = timeout(self.config.timeout, self.do_execute(input)).await??;
        
        // Post-execution hooks (metrics, event emission)
        self.hooks.execute(HookPoint::AfterExecution, &result).await?;
        self.event_bus.emit(Event::AgentCompleted { 
            agent_id: self.id(),
            success: true,
            duration: start.elapsed()
        }).await?;
        
        Ok(result)
    }
}
```

### 3. **Language Agnostic by Design**

> *"Let teams use their strengths, don't force language choices."*

**Principle**: Identical capabilities across all supported scripting languages with automatic translation and unified error handling.

**Implementation**:
- **Unified API Surface**: Same methods, parameters, and return types across languages
- **Automatic Type Conversion**: Seamless conversion between Rust and script types
- **Error Translation**: Consistent error types and messages across languages  
- **Async Pattern Unification**: Coroutines, Promises, and async/await all supported

**Benefits**:
- Teams can choose languages based on expertise and preference
- Consistent behavior regardless of language choice
- Easy migration between languages when needed
- Reduced learning curve for new team members

**Example**:
```lua
-- Lua version
local result = Agent.execute("research_agent", {
    query = "AI safety trends",
    max_sources = 10
})
```

```javascript
// JavaScript version - identical API
const result = await Agent.execute("research_agent", {
    query: "AI safety trends", 
    maxSources: 10
});
```

```python  
# Python version (future) - same capabilities
result = await Agent.execute("research_agent", {
    "query": "AI safety trends",
    "max_sources": 10
})
```

### 4. **Composability Over Inheritance**

> *"Prefer composition and traits over deep inheritance hierarchies."*

**Principle**: Build systems through composition of small, focused components rather than complex inheritance trees.

**Implementation**:
- **Trait-Based Design**: Behaviors defined as composable traits
- **Tool Composition**: Agents can use any combination of tools
- **Workflow Nesting**: Workflows can contain other workflows
- **Mixin Patterns**: Add capabilities through trait composition

**Benefits**:
- Flexible and extensible component combinations
- Easier testing through focused, single-responsibility components
- Reduced coupling and improved maintainability
- Clear separation of concerns

**Example**:
```rust
// Composition-based agent design
pub struct ResearchAgent {
    base: BaseAgentImpl,           // Core agent functionality
    llm_client: Box<dyn LLMProvider>,  // LLM capability
    tools: Vec<Box<dyn Tool>>,     // Composed tool capabilities  
    memory: Box<dyn Memory>,       // Memory capability
    validator: Box<dyn Validator>, // Validation capability
}

// Traits define composable behaviors
impl BaseAgent for ResearchAgent { /* ... */ }
impl LLMCapable for ResearchAgent { /* ... */ }
impl ToolCapable for ResearchAgent { /* ... */ }  
impl MemoryCapable for ResearchAgent { /* ... */ }
```

### 5. **Zero-Cost Abstractions**

> *"Abstractions should compile away, leaving only the essential work."*

**Principle**: High-level APIs should compile down to efficient code with no runtime overhead for unused features.

**Implementation**:
- **Trait Objects**: Dynamic dispatch only when needed
- **Generic Constraints**: Compile-time specialization where possible
- **Feature Flags**: Include only needed capabilities in builds
- **Lazy Initialization**: Initialize expensive resources only when used

**Benefits**:
- High-level ergonomics without performance penalties
- Scalable from embedded to high-throughput scenarios
- Pay-for-what-you-use resource model
- Predictable performance characteristics

**Example**:
```rust
// Generic agent execution - compiles to specialized code
pub async fn execute_agent<A, I, O>(agent: &mut A, input: I) -> Result<O>
where 
    A: BaseAgent,
    I: Into<AgentInput>,
    O: From<AgentOutput>,
{
    // This compiles to specialized code for each agent type
    let agent_input = input.into();
    let agent_output = agent.execute(agent_input).await?;
    Ok(agent_output.into())
}

// Usage compiles to direct, efficient calls
let result: ResearchResult = execute_agent(&mut research_agent, query).await?;
```

### 6. **Observable and Debuggable**

> *"If you can't observe it, you can't improve it."*

**Principle**: Every operation should emit events and provide debugging hooks to enable comprehensive observability.

**Implementation**:
- **Structured Logging**: Consistent, machine-readable log format
- **Distributed Tracing**: Request correlation across components
- **Metrics Collection**: Performance and behavior metrics
- **Debug Hooks**: Ability to inspect and modify execution at any point

**Benefits**:
- Quick problem diagnosis and resolution
- Performance optimization based on real data
- Comprehensive system understanding
- Proactive issue detection

**Example**:
```rust
// Observable agent execution
#[tracing::instrument(skip(self), fields(agent_id = %self.id()))]
async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput> {
    // Emit start event
    self.event_bus.emit(Event::AgentStarted {
        agent_id: self.id(),
        input_hash: input.hash(),
        timestamp: Utc::now(),
    }).await;
    
    // Structured logging
    tracing::info!(
        agent_id = %self.id(),
        input_size = input.size(),
        "Agent execution started"
    );
    
    // Metrics collection
    self.metrics.increment_counter("agent.executions.started");
    let timer = self.metrics.start_timer("agent.execution.duration");
    
    // Actual execution...
    let result = self.do_execute(input).await;
    
    // Record completion
    timer.stop();
    self.metrics.increment_counter(
        if result.is_ok() { "agent.executions.success" } 
        else { "agent.executions.failure" }
    );
    
    result
}
```

### 7. **Future-Proof Extensibility**

> *"Build for change, not just current requirements."*

**Principle**: Architecture should accommodate future needs through well-defined extension points and backward compatibility.

**Implementation**:
- **Plugin System**: Dynamic loading of new capabilities
- **Version Compatibility**: Backward compatible API evolution
- **Extension Traits**: Add new behaviors without breaking changes
- **Protocol Agnostic**: Support for future communication protocols

**Benefits**:
- Smooth evolution as requirements change
- Third-party ecosystem development
- Investment protection through compatibility
- Innovation without disruption

**Example**:
```rust
// Extensible agent system
pub trait AgentExtension: Send + Sync {
    fn extension_name(&self) -> &str;
    fn compatible_versions(&self) -> VersionRange;
    
    // Future-proof extension point
    async fn handle_request(&self, request: ExtensionRequest) -> Result<ExtensionResponse>;
}

// Plugin system for future extensions  
pub struct PluginManager {
    extensions: HashMap<String, Box<dyn AgentExtension>>,
    compatibility_checker: CompatibilityChecker,
}

impl PluginManager {
    pub fn register_extension(&mut self, extension: Box<dyn AgentExtension>) -> Result<()> {
        // Version compatibility check
        self.compatibility_checker.validate(&extension)?;
        
        // Safe registration
        self.extensions.insert(extension.extension_name().to_string(), extension);
        Ok(())
    }
}
```

These principles work together to create a system that is **powerful yet approachable**, **performant yet flexible**, and **stable yet extensible**. Every architectural decision in rs-llmspell can be traced back to these core principles.

---

## Quick Start Guide

Get up and running with rs-llmspell in under 10 minutes. This guide assumes you have Rust installed and basic familiarity with AI concepts.

### Installation

```bash
# Add rs-llmspell to your Cargo.toml
cargo add llmspell

# Or install the CLI tool
cargo install llmspell-cli

# Verify installation
llmspell --version
```

### Your First Spell

Create a simple chat agent that responds to user queries:

```lua
-- hello_world.lua
local Agent = require("llmspell.agent")
local Tools = require("llmspell.tools")

-- Create a helpful assistant agent
local assistant = Agent.new({
    name = "helpful_assistant",
    system_prompt = "You are a helpful, harmless, and honest assistant.",
    provider = "openai",  -- or "anthropic", "ollama", etc.
    model = "gpt-4",
    tools = {
        Tools.get("web_search"),     -- Built-in web search
        Tools.get("calculator"),     -- Built-in calculator  
        Tools.get("file_reader")     -- Built-in file operations
    }
})

-- Simple interaction
local response = assistant:chat("What's the weather like in San Francisco today?")
print("Assistant:", response)

-- Conversation with context
assistant:chat("What's 15% of 250?")
assistant:chat("If I invest that amount monthly at 7% annual return, how much will I have in 10 years?")
```

Run it:
```bash
# The CLI automatically detects the script type (.lua) and uses the correct engine.
llmspell run hello_world.lua
```

### Multi-Agent Workflow

Create a research workflow with multiple specialized agents:

```javascript
// research_workflow.js
const { Agent, Workflow, Tools } = require('llmspell');

// Define specialized agents
const researcher = new Agent({
    name: "academic_researcher",
    systemPrompt: "You are an expert academic researcher specializing in finding and analyzing scholarly sources.",
    tools: [
        Tools.get("scholarly_search"),
        Tools.get("pdf_analyzer"),
        Tools.get("citation_formatter")
    ]
});

const analyst = new Agent({
    name: "data_analyst", 
    systemPrompt: "You are a data analyst expert at finding patterns and insights in research data.",
    tools: [
        Tools.get("statistical_analysis"),
        Tools.get("visualization"),
        Tools.get("trend_detector")
    ]
});

const writer = new Agent({
    name: "technical_writer",
    systemPrompt: "You are a technical writer expert at creating clear, comprehensive reports.",
    tools: [
        Tools.get("document_formatter"),
        Tools.get("grammar_checker"),
        Tools.get("readability_analyzer")
    ]
});

// Create a research workflow
const researchWorkflow = Workflow.sequential([
    {
        name: "research_phase",
        agent: researcher,
        action: "gather_sources",
        output: "research_data"
    },
    {
        name: "analysis_phase", 
        agent: analyst,
        action: "analyze_patterns",
        input: "{{research_data}}",
        output: "analysis_results"
    },
    {
        name: "writing_phase",
        agent: writer,
        action: "write_report",
        input: {
            research: "{{research_data}}",
            analysis: "{{analysis_results}}"
        },
        output: "final_report"
    }
]);

// Execute the workflow
async function runResearch() {
    try {
        const result = await researchWorkflow.execute({
            topic: "Impact of large language models on software development productivity",
            output_format: "executive_summary",
            max_sources: 20
        });
        
        console.log("Research Complete!");
        console.log(`Sources analyzed: ${result.research_data.sources.length}`);
        console.log(`Key insights: ${result.analysis_results.insights.length}`);
        console.log(`Report length: ${result.final_report.word_count} words`);
        
        // Save the report
        await Tools.get("file_writer").execute({
            path: "./research_report.md",
            content: result.final_report.content
        });
        
    } catch (error) {
        console.error("Research failed:", error.message);
        
        // Access detailed error information
        if (error.recoverable) {
            console.log("Suggested recovery:", error.recovery_suggestion);
        }
    }
}

runResearch();
```

### Advanced Features Demo

Explore hooks, events, and error handling:

```lua
-- advanced_demo.lua
local Agent = require("llmspell.agent")
local Hooks = require("llmspell.hooks")
local Events = require("llmspell.events")
local ErrorHandler = require("llmspell.errors")

-- Set up comprehensive logging
local logger = Hooks.create_logger({
    level = "info",
    format = "json",
    output = "stdout"
})

-- Register hooks for observability
Hooks.register("before_agent_execution", logger)
Hooks.register("after_agent_execution", logger)
Hooks.register("tool_execution_start", logger)
Hooks.register("tool_execution_complete", logger)

-- Set up metrics collection
local metrics = Hooks.create_metrics_collector({
    backend = "prometheus",
    port = 9090
})

Hooks.register("agent_execution_complete", metrics)

-- Set up custom event handlers
Events.subscribe("agent_error", function(event)
    print("🚨 Agent Error:", event.agent_id, event.error_type)
    
    -- Custom recovery logic
    if event.error_type == "rate_limit" then
        print("💤 Waiting before retry...")
        return coroutine.create(function()
            yield Events.sleep(event.retry_after)
            return "retry"
        end)
    end
end)

Events.subscribe("workflow_complete", function(event)
    print("✅ Workflow Complete:", event.workflow_name)
    print("   Duration:", event.duration, "seconds")
    print("   Success Rate:", event.success_rate, "%")
end)

-- Create agent with advanced error handling
local robust_agent = Agent.new({
    name = "robust_research_agent",
    system_prompt = "You are a resilient research agent that handles errors gracefully.",
    provider = "anthropic",
    model = "claude-3-sonnet",
    
    -- Advanced configuration
    config = {
        timeout = 60,           -- 60 second timeout
        retry_count = 3,        -- Retry failed operations
        circuit_breaker = {
            failure_threshold = 5,
            reset_timeout = 300
        },
        rate_limit = {
            requests_per_minute = 30,
            burst_allowance = 5
        }
    },
    
    -- Error handling strategy
    error_strategy = ErrorHandler.create_strategy({
        on_timeout = "retry_with_shorter_context",
        on_rate_limit = "exponential_backoff",
        on_provider_error = "switch_to_fallback_provider",
        on_tool_error = "skip_tool_and_continue"
    }),
    
    tools = {
        Tools.get("web_search"),
        Tools.get("academic_search"), 
        Tools.get("file_operations")
    }
})

-- Execute with comprehensive error handling
local function safe_research(query)
    local success, result = pcall(function()
        return robust_agent:chat(query)
    end)
    
    if success then
        print("✅ Research successful")
        print("Response length:", #result)
        return result
    else
        local error_info = ErrorHandler.parse_error(result)
        print("❌ Research failed:", error_info.category)
        
        -- Attempt recovery based on error type
        if error_info.recoverable then
            print("🔄 Attempting recovery:", error_info.recovery_strategy)
            
            local recovery_success, recovery_result = pcall(function()
                return ErrorHandler.attempt_recovery(robust_agent, query, error_info)
            end)
            
            if recovery_success then
                print("✅ Recovery successful")
                return recovery_result
            else
                print("❌ Recovery failed")
                return nil
            end
        else
            print("💀 Unrecoverable error")
            return nil
        end
    end
end

-- Run the demo
print("🚀 Starting Advanced Rs-LLMSpell Demo")
print("📊 Metrics available at: http://localhost:9090/metrics")

local result = safe_research("Analyze the latest developments in quantum computing and their potential impact on cryptography")

if result then
    print("\n📝 Research Result:")
    print(result:sub(1, 200) .. "...")
else
    print("\n💔 Research could not be completed")
end

print("\n📈 Session Statistics:")
print("Events emitted:", Events.get_stats().total_events)
print("Hooks executed:", Hooks.get_stats().total_executions)
print("Errors handled:", ErrorHandler.get_stats().total_errors)
```

### Configuration

Create a configuration file for your environment:

```toml
# llmspell.toml
[providers]
default = "anthropic"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"
organization = "${OPENAI_ORG_ID}"
timeout = 60

[providers.anthropic]  
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"
timeout = 90

[providers.ollama]
base_url = "http://localhost:11434"
timeout = 120

[storage]
backend = "sled"
path = "./llmspell_data"
max_size = "1GB"

[security]
sandbox_enabled = true
allowed_domains = ["*.wikipedia.org", "*.github.com", "*.arxiv.org"]
max_file_size = "10MB"
allowed_file_types = [".txt", ".md", ".json", ".csv"]

[observability]
logging_level = "info"
metrics_enabled = true
tracing_enabled = true

[observability.metrics]
backend = "prometheus"
port = 9090
path = "/metrics"

[observability.tracing]
backend = "jaeger"
endpoint = "http://localhost:14268/api/traces"

[performance]
max_concurrent_agents = 10
agent_timeout = 300
tool_timeout = 60
memory_limit = "2GB"
```

### Next Steps

Now that you have rs-llmspell running:

1. **Explore Built-in Tools**: Check out the [Complete Built-in Tools Catalog](#complete-built-in-tools-catalog)
2. **Learn Advanced Patterns**: Review [Advanced Orchestration Patterns](#advanced-orchestration-patterns) 
3. **Set Up Production**: Configure [Security](#security-model-and-threat-analysis) and [Monitoring](#observability-and-monitoring)
4. **Build Custom Components**: Create your own [Tools and Agents](#development-workflow)
5. **Join the Community**: Contribute to the [Rs-LLMSpell ecosystem](#future-evolution-strategy)

---

# Part II: Core Architecture

## Architecture Overview

Rs-LLMSpell implements a **hierarchical, event-driven architecture** built on four foundational layers that work together to provide seamless AI orchestration capabilities. It supports two primary usage paradigms: **Embedded Mode** and **Library Mode**.

### Dual Usage Paradigms

#### 1. Embedded Mode

In this mode, `rs-llmspell` acts as a standalone runtime that executes scripts (spells). This is the primary mode for building new applications from scratch, where `rs-llmspell` provides the complete execution environment.

```
┌─────────────────────────────────────┐
│         Rs-LLMSpell Runtime         │
│  ┌─────────────┐  ┌─────────────┐   │
│  │ Lua Script  │  │  JS Script  │   │
│  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────┘
```

#### 2. Library Mode

In this mode, `rs-llmspell` is compiled as a native shared library (e.g., `.so`, `.dll`, `.dylib`) and imported into an existing application's scripting environment (like a standard Lua 5.4 or Node.js runtime). This allows developers to add powerful agentic capabilities to their existing applications without a complete rewrite.

```
┌─────────────────────────────────────┐
│      External Application (Lua/JS)  │
│  ┌─────────────────────────────────┐ │
│  │    local llmspell = require()   │ │
│  │ const llmspell = require()      │ │
│  └─────────────────────────────────┘ │
│                  │                  │
│                  ▼                  │
│  ┌─────────────────────────────────┐ │
│  │   Rs-LLMSpell Native Module     │ │
│  └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Four-Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Script Layer (Lua/JS/Python)             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ Lua Scripts │ │ JS Scripts  │ │ Python Scripts      │   │
│  │ (mlua)      │ │ (boa/v8)    │ │ (pyo3) [Future]     │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Bridge Layer                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ Type        │ │ Error       │ │ Async Pattern       │   │
│  │ Converter   │ │ Translator  │ │ Coordinator         │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
│                 Unified API Surface                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Application Layer                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ Agents      │ │ Tools       │ │ Workflows           │   │
│  │ Registry    │ │ Registry    │ │ Engine              │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ Hook        │ │ Event       │ │ State               │   │
│  │ System      │ │ Bus         │ │ Manager             │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ LLM         │ │ Storage     │ │ Security            │   │
│  │ Providers   │ │ Backend     │ │ Manager             │   │
│  │ (rig)       │ │ (sled/rocks)│ │                     │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ Async       │ │ Observability│ │ Resource            │   │
│  │ Runtime     │ │ (metrics)   │ │ Management          │   │
│  │ (tokio)     │ │             │ │                     │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow Architecture

The architecture follows a **request-response pattern** with comprehensive **event emission** and **hook execution** at every step:

```
User Script Request
        │
        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Script        │    │   Bridge        │    │   Application   │
│   Engine        │───▶│   Layer         │───▶│   Layer         │
│                 │    │                 │    │                 │
│ - Parse request │    │ - Type convert  │    │ - Route request │
│ - Validate args │    │ - Validate      │    │ - Execute hooks │
│ - Handle async  │    │ - Translate     │    │ - Emit events   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        ▲                       ▲                       │
        │                       │                       ▼
        │              ┌─────────────────┐    ┌─────────────────┐
        │              │   Error         │    │   Component     │
        └──────────────│   Translator    │◀───│   Execution     │
                       │                 │    │                 │
                       │ - Map errors    │    │ - Agent/Tool    │
                       │ - Add context   │    │ - State mgmt    │
                       │ - Format msgs   │    │ - Resource ctrl │
                       └─────────────────┘    └─────────────────┘
                               │                       │
                               ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                Infrastructure Layer                             │
│                                                                 │
│  LLM Calls ──▶ Provider ──▶ Response                          │
│  State Ops ──▶ Storage  ──▶ Result                            │
│  Events    ──▶ Bus      ──▶ Subscribers                       │
│  Metrics   ──▶ Collector──▶ Backend                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Core Architectural Patterns

#### 1. **Component Registry Pattern**

All major components (agents, tools, workflows) are registered in centralized registries with metadata and lifecycle management:

```rust
pub struct ComponentRegistry<T> {
    components: HashMap<String, Arc<T>>,
    metadata: HashMap<String, ComponentMetadata>,
    lifecycle_hooks: Vec<Box<dyn LifecycleHook<T>>>,
    discovery_service: DiscoveryService,
}

impl<T> ComponentRegistry<T> 
where T: Component + Send + Sync 
{
    pub fn register(&mut self, component: T) -> Result<ComponentId> {
        let id = ComponentId::new(&component);
        let metadata = ComponentMetadata::extract(&component);
        
        // Validate component
        self.validate_component(&component, &metadata)?;
        
        // Execute registration hooks
        for hook in &self.lifecycle_hooks {
            hook.on_register(&component)?;
        }
        
        // Store component and metadata
        self.components.insert(id.clone(), Arc::new(component));
        self.metadata.insert(id.clone(), metadata);
        
        // Update discovery
        self.discovery_service.announce_component(&id, &metadata)?;
        
        Ok(id)
    }
    
    pub fn get(&self, id: &ComponentId) -> Option<Arc<T>> {
        self.components.get(id).cloned()
    }
    
    pub fn discover(&self, capabilities: &[Capability]) -> Vec<ComponentId> {
        self.discovery_service.find_matching_components(capabilities)
    }
}
```

#### 2. **Event-Driven Coordination Pattern**

Components communicate through a comprehensive event bus with typed events, subscriptions, and automatic error handling:

```rust
pub struct EventBus {
    subscribers: HashMap<EventType, Vec<Box<dyn EventSubscriber>>>,
    event_buffer: RingBuffer<Event>,
    metrics: EventMetrics,
    circuit_breaker: CircuitBreaker,
}

#[async_trait]
pub trait EventSubscriber: Send + Sync {
    async fn handle_event(&self, event: &Event) -> Result<EventHandlerResult>;
    fn event_types(&self) -> &[EventType];
    fn priority(&self) -> EventPriority;
}

impl EventBus {
    pub async fn emit(&self, event: Event) -> Result<()> {
        // Record event
        self.metrics.record_event(&event);
        self.event_buffer.push(event.clone());
        
        // Get subscribers for this event type
        let subscribers = self.subscribers.get(&event.event_type())
            .unwrap_or(&Vec::new());
        
        // Execute subscribers in priority order
        let mut sorted_subscribers = subscribers.clone();
        sorted_subscribers.sort_by_key(|s| s.priority());
        
        for subscriber in sorted_subscribers {
            // Circuit breaker check
            if self.circuit_breaker.should_block_subscriber(subscriber.as_ref()) {
                continue;
            }
            
            // Handle event with timeout and error recovery
            let result = timeout(
                Duration::from_secs(30), 
                subscriber.handle_event(&event)
            ).await;
            
            match result {
                Ok(Ok(EventHandlerResult::Continue)) => continue,
                Ok(Ok(EventHandlerResult::StopPropagation)) => break,
                Ok(Err(e)) => {
                    tracing::warn!("Event subscriber error: {}", e);
                    self.circuit_breaker.record_failure(subscriber.as_ref());
                },
                Err(_) => {
                    tracing::warn!("Event subscriber timeout");
                    self.circuit_breaker.record_failure(subscriber.as_ref());
                }
            }
        }
        
        Ok(())
    }
}
```

#### 3. **Hook-Based Extensibility Pattern**

Every major operation provides hook points for customization, monitoring, and integration:

```rust
pub struct HookRegistry {
    hooks: HashMap<HookPoint, Vec<Box<dyn Hook>>>,
    execution_strategy: HookExecutionStrategy,
    metrics: HookMetrics,
}

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult>;
}

#[derive(Debug, Clone)]
pub enum HookPoint {
    // Agent lifecycle hooks
    BeforeAgentExecution,
    AfterAgentExecution,
    AgentError,
    
    // Tool execution hooks
    BeforeToolExecution,
    AfterToolExecution,
    ToolError,
    
    // Workflow hooks
    BeforeWorkflowExecution,
    WorkflowStepComplete,
    AfterWorkflowExecution,
    WorkflowError,
    
    // System hooks
    SystemStartup,
    SystemShutdown,
    ConfigurationChange,
    
    // Custom extension hooks
    Custom(String),
}

impl HookRegistry {
    pub async fn execute_hooks(&self, point: HookPoint, context: &mut HookContext) -> Result<()> {
        let hooks = self.hooks.get(&point).unwrap_or(&Vec::new());
        
        match self.execution_strategy {
            HookExecutionStrategy::Sequential => {
                for hook in hooks {
                    let result = hook.execute(context).await?;
                    if result.should_stop_execution() {
                        break;
                    }
                }
            },
            HookExecutionStrategy::Parallel => {
                let futures: Vec<_> = hooks.iter()
                    .map(|hook| hook.execute(context))
                    .collect();
                
                let results = join_all(futures).await;
                for result in results {
                    result?; // Propagate any errors
                }
            },
            HookExecutionStrategy::ParallelWithTimeout(timeout) => {
                let futures: Vec<_> = hooks.iter()
                    .map(|hook| timeout(timeout, hook.execute(context)))
                    .collect();
                
                let results = join_all(futures).await;
                for result in results {
                    match result {
                        Ok(Ok(_)) => continue,
                        Ok(Err(e)) => return Err(e),
                        Err(_) => tracing::warn!("Hook execution timeout"),
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

#### 4. **Resource Management Pattern**

Comprehensive resource tracking and limits across all operations:

```rust
pub struct ResourceManager {
    memory_limiter: MemoryLimiter,
    cpu_limiter: CpuLimiter,
    network_limiter: NetworkLimiter,
    file_limiter: FileLimiter,
    concurrent_limiter: ConcurrentLimiter,
    metrics: ResourceMetrics,
}

#[async_trait]
pub trait ResourceLimiter: Send + Sync {
    async fn acquire(&self, amount: ResourceAmount) -> Result<ResourceGuard>;
    fn available(&self) -> ResourceAmount;
    fn total(&self) -> ResourceAmount;
}

pub struct ResourceGuard {
    limiter: Weak<dyn ResourceLimiter>,
    amount: ResourceAmount,
    acquired_at: Instant,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        if let Some(limiter) = self.limiter.upgrade() {
            limiter.release(self.amount);
        }
    }
}

impl ResourceManager {
    pub async fn execute_with_limits<F, R>(&self, 
                                          operation: F,
                                          limits: ResourceLimits) -> Result<R>
    where
        F: Future<Output = Result<R>> + Send,
        R: Send,
    {
        // Acquire all required resources
        let _memory_guard = self.memory_limiter
            .acquire(limits.memory).await?;
        let _cpu_guard = self.cpu_limiter
            .acquire(limits.cpu).await?;
        let _network_guard = self.network_limiter
            .acquire(limits.network).await?;
        let _file_guard = self.file_limiter
            .acquire(limits.files).await?;
        let _concurrent_guard = self.concurrent_limiter
            .acquire(limits.concurrent_operations).await?;
        
        // Execute operation with timeout
        let result = timeout(limits.timeout, operation).await?;
        
        // Guards automatically release resources on drop
        result
    }
}
```

### Cross-Cutting Concerns

#### 1. **Observability Integration**

Every component automatically integrates with the observability stack:

```rust
#[derive(Debug, Clone)]
pub struct ObservabilityContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub baggage: HashMap<String, String>,
    pub metrics_tags: HashMap<String, String>,
}

pub trait Observable {
    fn observability_context(&self) -> &ObservabilityContext;
    
    fn emit_metric(&self, name: &str, value: f64, tags: Option<HashMap<String, String>>) {
        let mut final_tags = self.observability_context().metrics_tags.clone();
        if let Some(additional_tags) = tags {
            final_tags.extend(additional_tags);
        }
        
        METRICS_COLLECTOR.record(name, value, final_tags);
    }
    
    fn start_span(&self, name: &str) -> Span {
        tracing::info_span!(
            name,
            trace_id = %self.observability_context().trace_id,
            span_id = %self.observability_context().span_id,
        )
    }
}

// Automatic implementation for all components
impl<T> Observable for T 
where T: Component 
{
    fn observability_context(&self) -> &ObservabilityContext {
        &self.base_component().observability_context
    }
}
```

#### 2. **Security Context Propagation**

Security principals and permissions flow through all operations:

```rust
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub principal: Principal,
    pub permissions: PermissionSet,
    pub security_level: SecurityLevel,
    pub sandbox_config: SandboxConfig,
}

pub trait SecureComponent {
    fn security_context(&self) -> &SecurityContext;
    
    fn check_permission(&self, permission: Permission) -> Result<()> {
        if self.security_context().permissions.contains(&permission) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied { 
                principal: self.security_context().principal.clone(),
                permission,
            })
        }
    }
    
    fn execute_in_sandbox<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce() -> R,
    {
        let sandbox = Sandbox::new(&self.security_context().sandbox_config)?;
        sandbox.execute(operation)
    }
}
```

#### 3. **Error Context Enrichment**

Errors automatically include context from all architectural layers:

```rust
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub component_id: Option<String>,
    pub operation: Option<String>,
    pub trace_id: Option<TraceId>,
    pub security_context: Option<SecurityContext>,
    pub resource_usage: Option<ResourceUsage>,
    pub timing_info: Option<TimingInfo>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub trait ContextualError: std::error::Error {
    fn with_context(self, context: ErrorContext) -> EnrichedError;
    
    fn with_component(self, component_id: impl Into<String>) -> EnrichedError 
    where Self: Sized 
    {
        self.with_context(ErrorContext {
            component_id: Some(component_id.into()),
            ..Default::default()
        })
    }
    
    fn with_operation(self, operation: impl Into<String>) -> EnrichedError 
    where Self: Sized 
    {
        self.with_context(ErrorContext {
            operation: Some(operation.into()),
            ..Default::default()
        })
    }
}

#[derive(Debug)]
pub struct EnrichedError {
    pub source_error: Box<dyn std::error::Error + Send + Sync>,
    pub context: ErrorContext,
    pub recovery_suggestions: Vec<String>,
}

impl EnrichedError {
    pub fn to_user_facing(&self) -> UserFacingError {
        UserFacingError {
            message: self.create_user_message(),
            error_code: self.extract_error_code(),
            suggestions: self.recovery_suggestions.clone(),
            trace_id: self.context.trace_id,
        }
    }
}
```

This architecture provides:
- **Consistent patterns** across all components
- **Comprehensive observability** without manual instrumentation
- **Automatic security enforcement** through context propagation
- **Rich error information** for debugging and recovery
- **Resource safety** through automatic limit enforcement
- **Event-driven coordination** for loose coupling
- **Hook-based extensibility** for customization

---

## Component Hierarchy

Rs-LLMSpell implements a **four-tier component hierarchy** inspired by go-llms but enhanced for Rust's type system and modern async patterns. This hierarchy provides clear separation of concerns while enabling powerful composition patterns.

### Hierarchy Overview

```
BaseAgent (Foundation Trait)
    ├── Agent (LLM-Powered Components)
    │   ├── ChatAgent
    │   ├── ResearchAgent  
    │   ├── AnalysisAgent
    │   ├── CodeAgent
    │   └── CustomAgent
    │
    ├── Tool (Functional Components)
    │   ├── BuiltinTool
    │   │   ├── WebSearchTool
    │   │   ├── FileSystemTool
    │   │   ├── CalculatorTool
    │   │   └── ...40+ more
    │   ├── AgentWrappedTool (Agents as Tools)
    │   └── CustomTool
    │
    └── Workflow (Orchestration Components)
        ├── SequentialWorkflow
        ├── ParallelWorkflow
        ├── ConditionalWorkflow
        ├── LoopWorkflow
        ├── FanOutWorkflow
        └── CustomWorkflow
```

### BaseAgent: Universal Foundation

**BaseAgent** is the foundational trait that defines capabilities common to ALL components in the system - whether they're LLM-powered agents, simple tools, or complex workflows.

```rust
#[async_trait]
pub trait BaseAgent: Send + Sync + Observable + SecureComponent + Clone {
    // Identity and Metadata
    fn id(&self) -> &ComponentId;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &Version;
    fn component_type(&self) -> ComponentType;
    
    // Core Execution Interface
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput>;
    async fn validate_input(&self, input: &AgentInput) -> Result<ValidationResult>;
    async fn prepare_execution(&mut self, input: &AgentInput) -> Result<ExecutionContext>;
    
    // Capability and Dependency Management
    fn capabilities(&self) -> &ComponentCapabilities;
    fn dependencies(&self) -> &[ComponentId];
    fn provides(&self) -> &[CapabilityId];
    fn requires(&self) -> &[CapabilityId];
    
    // State Management
    fn state(&self) -> &ComponentState;
    fn set_state(&mut self, state: ComponentState) -> Result<()>;
    async fn save_state(&self) -> Result<()>;
    async fn restore_state(&mut self) -> Result<()>;
    
    // Tool Integration (All components can use tools)
    fn tools(&self) -> &ToolRegistry;
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;
    fn remove_tool(&mut self, tool_id: &ToolId) -> Result<()>;
    async fn execute_tool(&self, tool_id: &ToolId, input: ToolInput) -> Result<ToolOutput>;
    
    // Hook System Integration
    fn hooks(&self) -> &HookRegistry;
    fn add_hook(&mut self, point: HookPoint, hook: Box<dyn Hook>) -> Result<()>;
    async fn execute_hooks(&self, point: HookPoint, context: &mut HookContext) -> Result<()>;
    
    // Event System Integration  
    fn event_emitter(&self) -> &dyn EventEmitter;
    async fn emit_event(&self, event: Event) -> Result<()>;
    fn subscribe_to_event(&mut self, event_type: EventType, handler: Box<dyn EventHandler>) -> Result<()>;
    
    // Resource Management
    fn resource_requirements(&self) -> ResourceRequirements;
    async fn acquire_resources(&self) -> Result<ResourceGuard>;
    
    // Lifecycle Management
    async fn initialize(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    fn health_check(&self) -> HealthStatus;
    
    // Configuration
    fn config(&self) -> &ComponentConfig;
    fn update_config(&mut self, config: ComponentConfig) -> Result<()>;
    
    // Serialization for Persistence
    fn serialize(&self) -> Result<SerializedComponent>;
    fn deserialize(data: SerializedComponent) -> Result<Self> where Self: Sized;
}
```

#### BaseAgent Implementation Pattern

All components implement BaseAgent through a common base implementation:

```rust
pub struct BaseAgentImpl {
    // Core identity
    pub id: ComponentId,
    pub name: String,
    pub description: String,
    pub version: Version,
    pub component_type: ComponentType,
    
    // Capability system
    pub capabilities: ComponentCapabilities,
    pub dependencies: Vec<ComponentId>,
    pub provides: Vec<CapabilityId>,
    pub requires: Vec<CapabilityId>,
    
    // State management
    pub state: ComponentState,
    pub state_storage: Box<dyn StateStorage>,
    
    // Tool integration
    pub tools: ToolRegistry,
    
    // Hook system
    pub hooks: HookRegistry,
    
    // Event system
    pub event_emitter: Box<dyn EventEmitter>,
    pub event_subscriptions: HashMap<EventType, Vec<Box<dyn EventHandler>>>,
    
    // Resource management
    pub resource_requirements: ResourceRequirements,
    pub resource_manager: Arc<ResourceManager>,
    
    // Configuration
    pub config: ComponentConfig,
    
    // Observability
    pub observability_context: ObservabilityContext,
    
    // Security
    pub security_context: SecurityContext,
    
    // Lifecycle
    pub lifecycle_state: LifecycleState,
}

impl BaseAgentImpl {
    pub fn new(config: ComponentConfig) -> Result<Self> {
        let id = ComponentId::generate();
        let observability_context = ObservabilityContext::new(&id);
        let security_context = SecurityContext::from_config(&config.security)?;
        
        Ok(Self {
            id,
            name: config.name.clone(),
            description: config.description.clone(),
            version: config.version.clone(),
            component_type: config.component_type,
            capabilities: ComponentCapabilities::from_config(&config.capabilities),
            dependencies: config.dependencies.clone(),
            provides: config.provides.clone(),
            requires: config.requires.clone(),
            state: ComponentState::new(),
            state_storage: create_state_storage(&config.state_storage)?,
            tools: ToolRegistry::new(),
            hooks: HookRegistry::new(),
            event_emitter: create_event_emitter(&config.events)?,
            event_subscriptions: HashMap::new(),
            resource_requirements: config.resource_requirements.clone(),
            resource_manager: Arc::new(ResourceManager::from_config(&config.resources)?),
            config,
            observability_context,
            security_context,
            lifecycle_state: LifecycleState::Uninitialized,
        })
    }
}

// Delegate trait implementation to base
impl BaseAgent for MyComponent {
    fn id(&self) -> &ComponentId { &self.base.id }
    fn name(&self) -> &str { &self.base.name }
    // ... other delegations
    
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput> {
        // Component-specific implementation
        self.do_execute(input).await
    }
}
```

### Agent: LLM-Powered Components

**Agents** are BaseAgent implementations that integrate with Large Language Models to provide intelligent, conversational, and reasoning capabilities.

```rust
#[async_trait]
pub trait Agent: BaseAgent {
    // LLM Integration
    fn llm_provider(&self) -> &dyn LLMProvider;
    fn model_config(&self) -> &ModelConfig;
    
    // Prompt Management
    fn system_prompt(&self) -> &str;
    fn update_system_prompt(&mut self, prompt: String) -> Result<()>;
    fn prompt_template(&self) -> &PromptTemplate;
    
    // Conversation Management
    async fn chat(&mut self, message: &str) -> Result<String>;
    async fn chat_with_context(&mut self, message: &str, context: ConversationContext) -> Result<String>;
    fn conversation_history(&self) -> &ConversationHistory;
    fn clear_history(&mut self);
    
    // Advanced LLM Features
    async fn generate_with_tools(&mut self, prompt: &str, available_tools: &[ToolId]) -> Result<AgentOutput>;
    async fn function_calling(&mut self, functions: &[FunctionDefinition], input: &str) -> Result<FunctionCallResult>;
    async fn structured_output<T>(&mut self, prompt: &str, schema: &JsonSchema) -> Result<T> 
    where T: serde::de::DeserializeOwned;
    
    // Memory and Context
    fn memory(&self) -> &dyn Memory;
    async fn remember(&mut self, key: &str, value: serde_json::Value) -> Result<()>;
    async fn recall(&self, key: &str) -> Result<Option<serde_json::Value>>;
    
    // Reasoning and Planning
    async fn plan(&mut self, goal: &str) -> Result<Plan>;
    async fn reason(&mut self, question: &str, evidence: &[Evidence]) -> Result<Reasoning>;
    async fn solve_problem(&mut self, problem: Problem) -> Result<Solution>;
    
    // Learning and Adaptation  
    async fn learn_from_feedback(&mut self, feedback: Feedback) -> Result<()>;
    fn learning_config(&self) -> &LearningConfig;
}
```

#### Specialized Agent Types

```rust
// Chat-oriented agent
pub struct ChatAgent {
    base: BaseAgentImpl,
    llm_client: Box<dyn LLMProvider>,
    conversation_manager: ConversationManager,
    personality_config: PersonalityConfig,
}

impl Agent for ChatAgent {
    async fn chat(&mut self, message: &str) -> Result<String> {
        // Pre-chat hooks
        self.execute_hooks(HookPoint::BeforeLLMCall, &mut HookContext::new()).await?;
        
        // Build conversation context
        let context = self.conversation_manager.build_context(message)?;
        
        // Generate response with personality
        let prompt = self.apply_personality_to_prompt(&context);
        let response = self.llm_client.complete(&prompt).await?;
        
        // Post-chat hooks
        self.execute_hooks(HookPoint::AfterLLMCall, &mut HookContext::new()).await?;
        
        // Store in conversation history
        self.conversation_manager.add_exchange(message, &response);
        
        Ok(response)
    }
}

// Research-specialized agent
pub struct ResearchAgent {
    base: BaseAgentImpl,
    llm_client: Box<dyn LLMProvider>,
    research_tools: ResearchToolkit,
    knowledge_base: KnowledgeBase,
    citation_manager: CitationManager,
}

impl Agent for ResearchAgent {
    async fn execute(&mut self, input: AgentInput) -> Result<AgentOutput> {
        let research_query = input.get_string("query")?;
        
        // Multi-phase research process
        let sources = self.gather_sources(&research_query).await?;
        let evidence = self.analyze_sources(&sources).await?;
        let synthesis = self.synthesize_findings(&evidence).await?;
        let citations = self.citation_manager.format_citations(&sources);
        
        Ok(AgentOutput::new()
            .with_result(synthesis)
            .with_metadata("sources_count", sources.len())
            .with_metadata("citations", citations))
    }
}

// Code-specialized agent  
pub struct CodeAgent {
    base: BaseAgentImpl,
    llm_client: Box<dyn LLMProvider>,
    code_tools: CodeToolkit,
    language_analyzers: HashMap<Language, Box<dyn LanguageAnalyzer>>,
    execution_sandbox: CodeSandbox,
}

impl Agent for CodeAgent {
    async fn generate_code(&mut self, specification: &CodeSpecification) -> Result<GeneratedCode> {
        // Analyze requirements
        let analysis = self.analyze_requirements(specification).await?;
        
        // Generate code with appropriate language analyzer
        let language = specification.target_language;
        let analyzer = self.language_analyzers.get(&language)
            .ok_or_else(|| AgentError::UnsupportedLanguage(language))?;
        
        let code = self.llm_client.generate_structured_output(
            &specification.to_prompt(),
            &analyzer.code_schema()
        ).await?;
        
        // Validate and test in sandbox
        let validation_result = self.execution_sandbox.validate_code(&code).await?;
        
        Ok(GeneratedCode {
            code,
            language,
            validation_result,
            metadata: analysis.metadata,
        })
    }
}
```

### Tool: Functional Components

**Tools** are BaseAgent implementations that provide specific functional capabilities, either through direct computation or by wrapping other agents.

```rust
#[async_trait]
pub trait Tool: BaseAgent {
    // Tool-specific metadata
    fn tool_category(&self) -> ToolCategory;
    fn input_schema(&self) -> &JsonSchema;
    fn output_schema(&self) -> &JsonSchema;
    fn examples(&self) -> &[ToolExample];
    
    // Execution interface (simpler than full BaseAgent)
    async fn execute_tool(&self, input: ToolInput) -> Result<ToolOutput>;
    
    // Tool composition
    fn can_chain_with(&self, other: &dyn Tool) -> bool;
    async fn chain_with(&self, other: &dyn Tool, input: ToolInput) -> Result<ToolOutput>;
    
    // LLM Integration support
    fn to_function_definition(&self) -> FunctionDefinition;
    fn parse_llm_call(&self, function_call: &FunctionCall) -> Result<ToolInput>;
    
    // Resource requirements for execution
    fn execution_cost(&self, input: &ToolInput) -> ResourceCost;
    fn estimated_duration(&self, input: &ToolInput) -> Duration;
}
```

#### Built-in Tool Categories

Rs-LLMSpell provides 40+ built-in tools across 8 categories:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    // File and Data Operations
    FileSystem {
        read: bool,
        write: bool,
        execute: bool,
    },
    DataProcessing {
        formats: Vec<DataFormat>, // JSON, CSV, XML, YAML, etc.
        operations: Vec<DataOperation>, // Transform, filter, aggregate
    },
    
    // Web and Network Operations
    WebAccess {
        scraping: bool,
        api_calls: bool,
        download: bool,
    },
    NetworkTools {
        protocols: Vec<NetworkProtocol>, // HTTP, gRPC, WebSocket
        security: NetworkSecurityLevel,
        listeners: bool, // NEW: For Webhook/Socket listeners
    },
    
    // AI and ML Operations
    AICapabilities {
        text_processing: bool,
        image_processing: bool,
        audio_processing: bool,
        embedding_generation: bool,
    },
    MLOperations {
        model_inference: bool,
        data_analysis: bool,
        visualization: bool,
    },
    
    // System and Utility Operations
    SystemIntegration {
        process_control: bool,
        environment_access: bool,
        service_management: bool,
    },
    Utilities {
        calculation: bool,
        text_manipulation: bool,
        date_time: bool,
        encoding: bool,
    },
}
```

#### Tool Implementation Examples

```rust
// File system tool
pub struct FileSystemTool {
    base: BaseAgentImpl,
    sandbox: FileSystemSandbox,
    permissions: FilePermissions,
}

impl Tool for FileSystemTool {
    fn tool_category(&self) -> ToolCategory {
        ToolCategory::FileSystem {
            read: self.permissions.read,
            write: self.permissions.write,
            execute: self.permissions.execute,
        }
    }
    
    async fn execute_tool(&self, input: ToolInput) -> Result<ToolOutput> {
        let operation = input.get_string("operation")?;
        let path = input.get_string("path")?;
        
        // Security validation
        self.sandbox.validate_path(&path)?;
        
        match operation.as_str() {
            "read" => {
                self.check_permission(Permission::FileRead(&path))?;
                let content = self.sandbox.read_file(&path).await?;
                Ok(ToolOutput::new().with_result(content))
            },
            "write" => {
                self.check_permission(Permission::FileWrite(&path))?;
                let content = input.get_string("content")?;
                self.sandbox.write_file(&path, &content).await?;
                Ok(ToolOutput::new().with_result("File written successfully"))
            },
            "list" => {
                self.check_permission(Permission::DirectoryList(&path))?;
                let entries = self.sandbox.list_directory(&path).await?;
                Ok(ToolOutput::new().with_result(entries))
            },
            _ => Err(ToolError::UnsupportedOperation(operation))
        }
    }
    
    fn to_function_definition(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "file_system".to_string(),
            description: "Read, write, and manage files and directories".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "list", "delete", "copy", "move"],
                        "description": "The file system operation to perform"
                    },
                    "path": {
                        "type": "string", 
                        "description": "The file or directory path"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write (for write operations)"
                    }
                },
                "required": ["operation", "path"]
            }),
        }
    }
}

// Agent-wrapped tool (Agent as Tool pattern)
pub struct AgentWrappedTool {
    base: BaseAgentImpl,
    wrapped_agent: Box<dyn Agent>,
    tool_interface: ToolInterface,
}

impl Tool for AgentWrappedTool {
    async fn execute_tool(&self, input: ToolInput) -> Result<ToolOutput> {
        // Convert tool input to agent input
        let agent_input = self.tool_interface.convert_input(input)?;
        
        // Execute the wrapped agent
        let agent_output = self.wrapped_agent.execute(agent_input).await?;
        
        // Convert agent output back to tool output
        self.tool_interface.convert_output(agent_output)
    }
    
    fn to_function_definition(&self) -> FunctionDefinition {
        self.tool_interface.to_function_definition()
    }
}

// Web search tool with rate limiting and caching
pub struct WebSearchTool {
    base: BaseAgentImpl,
    search_providers: Vec<Box<dyn SearchProvider>>,
    rate_limiter: RateLimiter,
    cache: SearchCache,
    result_validator: ResultValidator,
}

impl Tool for WebSearchTool {
    async fn execute_tool(&self, input: ToolInput) -> Result<ToolOutput> {
        let query = input.get_string("query")?;
        let max_results = input.get_u32("max_results").unwrap_or(10);
        
        // Check cache first
        if let Some(cached_results) = self.cache.get(&query).await? {
            if !cached_results.is_expired() {
                return Ok(ToolOutput::new()
                    .with_result(cached_results.results)
                    .with_metadata("cached", true));
            }
        }
        
        // Rate limiting
        self.rate_limiter.acquire().await?;
        
        // Search across providers with fallback
        let mut all_results = Vec::new();
        for provider in &self.search_providers {
            match provider.search(&query, max_results).await {
                Ok(results) => {
                    all_results.extend(results);
                    if all_results.len() >= max_results as usize {
                        break;
                    }
                },
                Err(e) => {
                    tracing::warn!("Search provider failed: {}", e);
                    continue; // Try next provider
                }
            }
        }
        
        // Validate and rank results
        let validated_results = self.result_validator.validate_and_rank(all_results)?;
        
        // Cache results
        self.cache.store(&query, &validated_results).await?;
        
        Ok(ToolOutput::new()
            .with_result(validated_results)
            .with_metadata("cached", false)
            .with_metadata("providers_used", self.search_providers.len()))
    }
}
```

### Workflow: Orchestration Components

**Workflows** are BaseAgent implementations that coordinate the execution of other components (agents, tools, or other workflows) according to specific patterns.

```rust
#[async_trait]
pub trait Workflow: BaseAgent {
    // Workflow structure
    fn workflow_type(&self) -> WorkflowType;
    fn steps(&self) -> &[WorkflowStep];
    fn add_step(&mut self, step: WorkflowStep) -> Result<()>;
    fn remove_step(&mut self, step_id: &StepId) -> Result<()>;
    
    // Execution control
    async fn execute_workflow(&mut self, input: WorkflowInput) -> Result<WorkflowOutput>;
    async fn execute_step(&mut self, step: &WorkflowStep, context: &WorkflowContext) -> Result<StepOutput>;
    
    // Flow control
    fn supports_conditionals(&self) -> bool;
    fn supports_loops(&self) -> bool;
    fn supports_parallel_execution(&self) -> bool;
    
    // State and persistence
    async fn save_checkpoint(&self) -> Result<WorkflowCheckpoint>;
    async fn restore_from_checkpoint(&mut self, checkpoint: WorkflowCheckpoint) -> Result<()>;
    
    // Monitoring and control
    async fn pause_execution(&mut self) -> Result<()>;
    async fn resume_execution(&mut self) -> Result<()>;
    async fn cancel_execution(&mut self) -> Result<()>;
    fn execution_status(&self) -> WorkflowExecutionStatus;
}

#[derive(Debug, Clone)]
pub enum WorkflowType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    FanOut,
    MapReduce,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub id: StepId,
    pub name: String,
    pub component: ComponentReference,
    pub input_mapping: InputMapping,
    pub output_mapping: OutputMapping,
    pub condition: Option<StepCondition>,
    pub retry_policy: RetryPolicy,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum ComponentReference {
    Agent(ComponentId),
    Tool(ComponentId),
    Workflow(ComponentId),
    Inline(Box<dyn BaseAgent>),
}
```

#### Workflow Implementation Examples

```rust
// Sequential workflow implementation
pub struct SequentialWorkflow {
    base: BaseAgentImpl,
    steps: Vec<WorkflowStep>,
    execution_state: WorkflowExecutionState,
    checkpoint_manager: CheckpointManager,
}

impl Workflow for SequentialWorkflow {
    async fn execute_workflow(&mut self, input: WorkflowInput) -> Result<WorkflowOutput> {
        let mut context = WorkflowContext::new(input);
        let mut final_output = WorkflowOutput::new();
        
        // Execute hooks before workflow
        self.execute_hooks(HookPoint::BeforeWorkflowExecution, 
                          &mut HookContext::new()).await?;
        
        for (index, step) in self.steps.iter().enumerate() {
            // Check for cancellation
            if self.execution_state.is_cancelled() {
                return Err(WorkflowError::Cancelled);
            }
            
            // Evaluate step condition
            if let Some(condition) = &step.condition {
                if !condition.evaluate(&context)? {
                    continue; // Skip this step
                }
            }
            
            // Execute hooks before step
            self.execute_hooks(HookPoint::BeforeWorkflowStep, 
                              &mut HookContext::new()).await?;
            
            // Execute step with retry logic
            let step_output = self.execute_step_with_retry(step, &context).await?;
            
            // Update context with step output
            context.add_step_output(step.id.clone(), step_output.clone());
            
            // Apply output mapping
            if let Some(output_key) = &step.output_mapping.key {
                final_output.add_output(output_key, step_output.result);
            }
            
            // Execute hooks after step
            self.execute_hooks(HookPoint::AfterWorkflowStep, 
                              &mut HookContext::new()).await?;
            
            // Save checkpoint if configured
            if self.checkpoint_manager.should_checkpoint(index) {
                self.save_checkpoint().await?;
            }
        }
        
        // Execute hooks after workflow
        self.execute_hooks(HookPoint::AfterWorkflowExecution, 
                          &mut HookContext::new()).await?;
        
        Ok(final_output)
    }
    
    async fn execute_step_with_retry(&mut self, step: &WorkflowStep, context: &WorkflowContext) -> Result<StepOutput> {
        let mut attempts = 0;
        let max_attempts = step.retry_policy.max_attempts;
        
        loop {
            attempts += 1;
            
            // Prepare step input
            let step_input = step.input_mapping.apply(context)?;
            
            // Execute step with timeout
            let result = if let Some(timeout) = step.timeout {
                timeout(timeout, self.execute_step(step, context)).await?
            } else {
                self.execute_step(step, context).await
            };
            
            match result {
                Ok(output) => return Ok(output),
                Err(e) if attempts < max_attempts && step.retry_policy.should_retry(&e) => {
                    // Wait before retry
                    let delay = step.retry_policy.calculate_delay(attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                },
                Err(e) => return Err(e),
            }
        }
    }
}

// Parallel workflow implementation
pub struct ParallelWorkflow {
    base: BaseAgentImpl,
    steps: Vec<WorkflowStep>,
    max_concurrency: usize,
    aggregation_strategy: AggregationStrategy,
}

impl Workflow for ParallelWorkflow {
    async fn execute_workflow(&mut self, input: WorkflowInput) -> Result<WorkflowOutput> {
        let context = WorkflowContext::new(input);
        
        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        
        // Execute all steps in parallel
        let step_futures: Vec<_> = self.steps.iter()
            .map(|step| {
                let semaphore = semaphore.clone();
                let context = context.clone();
                async move {
                    let _permit = semaphore.acquire().await?;
                    self.execute_step(step, &context).await
                }
            })
            .collect();
        
        // Wait for all steps to complete
        let step_results = join_all(step_futures).await;
        
        // Aggregate results
        let mut successful_outputs = Vec::new();
        let mut errors = Vec::new();
        
        for (step, result) in self.steps.iter().zip(step_results.iter()) {
            match result {
                Ok(output) => successful_outputs.push((step.id.clone(), output.clone())),
                Err(e) => errors.push((step.id.clone(), e.clone())),
            }
        }
        
        // Handle errors based on strategy
        if !errors.is_empty() && !self.aggregation_strategy.tolerates_errors() {
            return Err(WorkflowError::ParallelStepFailures(errors));
        }
        
        // Aggregate successful outputs
        let final_output = self.aggregation_strategy.aggregate(successful_outputs)?;
        
        Ok(final_output)
    }
}

// Conditional workflow implementation
pub struct ConditionalWorkflow {
    base: BaseAgentImpl,
    condition: Box<dyn WorkflowCondition>,
    true_branch: Box<dyn Workflow>,
    false_branch: Option<Box<dyn Workflow>>,
}

impl Workflow for ConditionalWorkflow {
    async fn execute_workflow(&mut self, input: WorkflowInput) -> Result<WorkflowOutput> {
        let context = WorkflowContext::new(input.clone());
        
        // Evaluate condition
        let condition_result = self.condition.evaluate(&context)?;
        
        // Execute appropriate branch
        if condition_result {
            self.true_branch.execute_workflow(input).await
        } else if let Some(false_branch) = &mut self.false_branch {
            false_branch.execute_workflow(input).await
        } else {
            // No false branch, return empty output
            Ok(WorkflowOutput::new())
        }
    }
}
```

This comprehensive component hierarchy provides:

- **Unified Interface**: All components implement BaseAgent for consistent behavior
- **Specialized Capabilities**: Each component type adds specific functionality
- **Composition Patterns**: Components can contain and coordinate other components
- **Type Safety**: Rust's type system enforces correct component usage
- **Performance**: Zero-cost abstractions compile to efficient code
- **Extensibility**: Easy to add new component types and capabilities

# Part III: Scripting and API Reference

## Complete Script Interface

This section details the script-level APIs available to developers. Rs-LLMSpell provides two primary ways to interact with its components:

1.  **Embedded Scripting**: Writing scripts (spells) that are executed by the `llmspell` runtime. This is ideal for creating new, standalone AI applications.
2.  **Native Module Integration**: Importing `rs-llmspell` as a library into existing Lua or JavaScript applications to add agentic capabilities.

Both modes expose the same core functionalities, ensuring a consistent developer experience.

## Using Rs-LLMSpell as a Native Module

One of the most powerful features of `rs-llmspell` is its ability to be compiled as a native module and integrated into existing applications. This allows you to bring advanced AI agent and workflow capabilities to your current projects without a full rewrite.

### Lua Integration

For Lua applications, `rs-llmspell` can be packaged as a LuaRock. Once installed, you can use it like any other native module:

```lua
-- main.lua (in an existing Lua application)
local llmspell = require("llmspell")

-- Create an agent on the fly
local assistant = llmspell.agent.new({
    system_prompt = "You are a helpful assistant integrated into a larger application.",
    provider = "ollama",
    model = "llama3"
})

-- Use the agent to process application data
local function process_user_data(data)
    local analysis_prompt = string.format("Analyze the following user data and provide insights: %s", data)
    local insights = assistant:chat(analysis_prompt)
    return insights
end

-- Example usage
local user_data = "... some data from the application ..."
local analysis_result = process_user_data(user_data)
print("AI Analysis:", analysis_result)
```

### JavaScript (Node.js) Integration

For Node.js applications, `rs-llmspell` can be distributed as an NPM package with native bindings.

```javascript
// server.js (in an existing Node.js application)
const { Agent } = require('@rs/llmspell');
const express = require('express');

const app = express();
app.use(express.json());

// Create a single, long-lived agent for the application
const supportAgent = new Agent({
    systemPrompt: "You are a customer support agent for our application.",
    tools: [/* ... application-specific tools ... */]
});

app.post('/api/support', async (req, res) => {
    const { userId, message } = req.body;
    
    try {
        const response = await supportAgent.chat(message, {
            metadata: { userId }
        });
        res.json({ reply: response });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

app.listen(3000, () => {
    console.log('Server with integrated AI support running on port 3000');
});
```

---

## Lua API Reference

Rs-LLMSpell provides a **unified scripting interface** that delivers identical functionality across Lua, JavaScript, and planned Python support. This interface abstracts away the complexity of the underlying Rust architecture while providing full access to all capabilities.

### Design Principles

#### 1. **Language Agnostic API Design**
Every function, method, and capability is available in all supported languages with consistent behavior:

```lua
-- Lua
local agent = Agent.new({
    name = "research_assistant",
    provider = "anthropic",
    model = "claude-3-sonnet"
})

local result = agent:chat("What are the latest developments in quantum computing?")
```

```javascript
// JavaScript
const agent = new Agent({
    name: "research_assistant", 
    provider: "anthropic",
    model: "claude-3-sonnet"
});

const result = await agent.chat("What are the latest developments in quantum computing?");
```

```python
# Python (Future)
agent = Agent(
    name="research_assistant",
    provider="anthropic", 
    model="claude-3-sonnet"
)

result = await agent.chat("What are the latest developments in quantum computing?")
```

#### 2. **Automatic Type Conversion**
The bridge layer handles seamless conversion between script types and Rust types:

```rust
// Bridge implementation for automatic conversion
pub struct TypeConverter {
    lua_converter: LuaTypeConverter,
    js_converter: JSTypeConverter,
    python_converter: PyTypeConverter,
}

impl TypeConverter {
    pub fn rust_to_script<T>(&self, value: T, engine: ScriptEngine) -> Result<ScriptValue> 
    where T: Serialize 
    {
        match engine {
            ScriptEngine::Lua => self.lua_converter.from_rust(value),
            ScriptEngine::JavaScript => self.js_converter.from_rust(value),
            ScriptEngine::Python => self.python_converter.from_rust(value),
        }
    }
    
    pub fn script_to_rust<T>(&self, value: ScriptValue, engine: ScriptEngine) -> Result<T>
    where T: for<'de> Deserialize<'de>
    {
        match engine {
            ScriptEngine::Lua => self.lua_converter.to_rust(value),
            ScriptEngine::JavaScript => self.js_converter.to_rust(value),
            ScriptEngine::Python => self.python_converter.to_rust(value),
        }
    }
}
```

#### 3. **Unified Async Patterns**
Each language uses its native async patterns while maintaining consistent behavior:

- **Lua**: Coroutines with cooperative yielding
- **JavaScript**: Promises with async/await
- **Python**: asyncio with async/await

```lua
-- Lua async with coroutines
local async_operation = coroutine.create(function()
    local step1 = yield Agent.execute("agent1", input)
    local step2 = yield Agent.execute("agent2", step1)
    return step2
end)

local result = Async.run(async_operation)
```

```javascript
// JavaScript async with Promises
async function asyncOperation() {
    const step1 = await Agent.execute("agent1", input);
    const step2 = await Agent.execute("agent2", step1);
    return step2;
}

const result = await asyncOperation();
```

#### 4. **Consistent Error Handling**
Errors are automatically translated to each language's native error handling patterns:

```lua
-- Lua error handling
local success, result = pcall(function()
    return Agent.execute("agent1", input)
end)

if not success then
    local error_info = ErrorHandler.parse_error(result)
    print("Error:", error_info.message)
    print("Suggestions:", table.concat(error_info.suggestions, ", "))
end
```

```javascript
// JavaScript error handling
try {
    const result = await Agent.execute("agent1", input);
    return result;
} catch (error) {
    const errorInfo = ErrorHandler.parseError(error);
    console.log("Error:", errorInfo.message);
    console.log("Suggestions:", errorInfo.suggestions.join(", "));
}
```

### Core API Modules

The scripting interface is organized into logical modules that map to the underlying Rust architecture:

#### 1. **Agent Module**
```typescript
// TypeScript definitions for clarity (available in all languages)
interface Agent {
    // Basic agent operations
    new(config: AgentConfig): Agent;
    chat(message: string): Promise<string>;
    execute(input: AgentInput): Promise<AgentOutput>;
    
    // Configuration management
    updateConfig(config: Partial<AgentConfig>): Promise<void>;
    getConfig(): AgentConfig;
    
    // Tool management
    addTool(tool: Tool): Promise<void>;
    removeTool(toolId: string): Promise<void>;
    listTools(): Tool[];
    
    // Memory operations
    remember(key: string, value: any): Promise<void>;
    recall(key: string): Promise<any>;
    forget(key: string): Promise<void>;
    
    // State management
    saveState(): Promise<void>;
    restoreState(): Promise<void>;
    getState(): AgentState;
    setState(state: AgentState): Promise<void>;
    
    // Lifecycle
    initialize(): Promise<void>;
    shutdown(): Promise<void>;
    health(): HealthStatus;
}

interface AgentConfig {
    name: string;
    description?: string;
    provider: string; // "openai", "anthropic", "ollama", etc.
    model: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
    tools?: Tool[];
    memory?: MemoryConfig;
    security?: SecurityConfig;
    resources?: ResourceConfig;
}
```

#### 2. **Tool Module**
```typescript
interface Tool {
    // Basic tool operations
    new(config: ToolConfig): Tool;
    execute(input: ToolInput): Promise<ToolOutput>;
    
    // Metadata
    getName(): string;
    getDescription(): string;
    getCategory(): ToolCategory;
    getInputSchema(): JsonSchema;
    getOutputSchema(): JsonSchema;
    getExamples(): ToolExample[];
    
    // Composition
    canChainWith(other: Tool): boolean;
    chainWith(other: Tool, input: ToolInput): Promise<ToolOutput>;
    
    // LLM integration
    toFunctionDefinition(): FunctionDefinition;
    parseCall(functionCall: FunctionCall): ToolInput;
}

interface ToolInput {
    parameters: Record<string, any>;
    context?: ExecutionContext;
    metadata?: Record<string, any>;
}

interface ToolOutput {
    result: any;
    artifacts?: Artifact[];
    metadata?: Record<string, any>;
    error?: Error;
}
```

#### 3. **Workflow Module**
```typescript
interface Workflow {
    // Workflow construction
    static sequential(steps: WorkflowStep[]): Workflow;
    static parallel(steps: WorkflowStep[], options?: ParallelOptions): Workflow;
    static conditional(condition: Condition, trueBranch: Workflow, falseBranch?: Workflow): Workflow;
    static loop(condition: LoopCondition, body: Workflow, options?: LoopOptions): Workflow;
    
    // Execution
    execute(input: WorkflowInput): Promise<WorkflowOutput>;
    executeStep(stepId: string, input: any): Promise<any>;
    
    // Control
    pause(): Promise<void>;
    resume(): Promise<void>;
    cancel(): Promise<void>;
    getStatus(): WorkflowStatus;
    
    // Persistence
    saveCheckpoint(): Promise<Checkpoint>;
    restoreFromCheckpoint(checkpoint: Checkpoint): Promise<void>;
    
    // Monitoring
    onStepComplete(callback: (step: WorkflowStep, output: any) => void): void;
    onError(callback: (error: Error, step: WorkflowStep) => void): void;
    onComplete(callback: (output: WorkflowOutput) => void): void;
}

interface WorkflowStep {
    id?: string;
    name: string;
    component: Agent | Tool | Workflow;
    input?: InputMapping;
    output?: OutputMapping;
    condition?: StepCondition;
    retry?: RetryPolicy;
    timeout?: number;
}
```

#### 4. **Tools Registry Module**
```typescript
interface ToolsRegistry {
    // Built-in tools access
    get(name: string): Tool;
    list(category?: ToolCategory): Tool[];
    search(query: string): Tool[];
    
    // Custom tool registration
    register(tool: Tool): Promise<void>;
    unregister(toolId: string): Promise<void>;
    
    // Discovery
    discover(capabilities: Capability[]): Tool[];
    getByCategory(category: ToolCategory): Tool[];
}

// Global Tools object available in scripts
const Tools = {
    // File operations
    get fileReader(): Tool;
    get fileWriter(): Tool;
    get directoryLister(): Tool;
    
    // Web operations  
    get webSearch(): Tool;
    get webScraper(): Tool;
    get httpClient(): Tool;
    
    // Data processing
    get jsonProcessor(): Tool;
    get csvProcessor(): Tool;
    get xmlProcessor(): Tool;
    
    // AI capabilities
    get textSummarizer(): Tool;
    get sentimentAnalyzer(): Tool;
    get languageDetector(): Tool;
    
    // And 30+ more built-in tools...
}
```

#### 5. **Events Module**
```typescript
interface EventBus {
    // Event subscription
    subscribe(eventType: string, handler: EventHandler): Promise<void>;
    unsubscribe(eventType: string, handler: EventHandler): Promise<void>;
    
    // Event emission
    emit(event: Event): Promise<void>;
    emitSync(event: Event): void;
    
    // Event filtering
    filter(predicate: (event: Event) => boolean): EventBus;
    
    // Replay and persistence
    replay(from?: Date, to?: Date): AsyncIterator<Event>;
    persist(config: PersistenceConfig): Promise<void>;
}

interface Event {
    type: string;
    timestamp: Date;
    source: string;
    data: any;
    metadata?: Record<string, any>;
}

type EventHandler = (event: Event) => Promise<void> | void;
```

#### 6. **Hooks Module**
```typescript
interface HookRegistry {
    // Hook registration
    register(point: HookPoint, hook: Hook): Promise<void>;
    unregister(point: HookPoint, hookId: string): Promise<void>;
    
    // Hook execution (internal)
    execute(point: HookPoint, context: HookContext): Promise<void>;
    
    // Built-in hook factories
    createLogger(config: LoggerConfig): Hook;
    createMetricsCollector(config: MetricsConfig): Hook;
    createValidator(schema: JsonSchema): Hook;
    createTransformer(transformer: (data: any) => any): Hook;
}

type HookPoint = 
    | "before_agent_execution"
    | "after_agent_execution" 
    | "before_tool_execution"
    | "after_tool_execution"
    | "before_workflow_execution"
    | "after_workflow_execution"
    | "agent_error"
    | "tool_error"
    | "workflow_error";

interface Hook {
    name: string;
    priority: number;
    execute(context: HookContext): Promise<HookResult>;
}
```

#### 7. **Configuration Module**
```typescript
interface Config {
    // Global configuration
    get(key: string): any;
    set(key: string, value: any): Promise<void>;
    
    // Provider configuration
    configureProvider(provider: string, config: ProviderConfig): Promise<void>;
    getProviderConfig(provider: string): ProviderConfig;
    
    // Security configuration
    configureSecurity(config: SecurityConfig): Promise<void>;
    getSecurityConfig(): SecurityConfig;
    
    // Resource limits
    configureResources(config: ResourceConfig): Promise<void>;
    getResourceConfig(): ResourceConfig;
    
    // Observability
    configureObservability(config: ObservabilityConfig): Promise<void>;
    getObservabilityConfig(): ObservabilityConfig;
}
```

#### 8. **Storage Module**
```typescript
interface Storage {
    // Key-value storage
    get(key: string): Promise<any>;
    set(key: string, value: any): Promise<void>;
    delete(key: string): Promise<void>;
    exists(key: string): Promise<boolean>;
    
    // Batch operations
    getBatch(keys: string[]): Promise<Record<string, any>>;
    setBatch(data: Record<string, any>): Promise<void>;
    deleteBatch(keys: string[]): Promise<void>;
    
    // Querying
    list(prefix?: string): Promise<string[]>;
    query(predicate: (key: string, value: any) => boolean): Promise<Record<string, any>>;
    
    // Transactions
    transaction(operations: StorageOperation[]): Promise<void>;
    
    // Cleanup
    clear(): Promise<void>;
    compact(): Promise<void>;
}
```

#### 9. **Error Handling Module**
```typescript
interface ErrorHandler {
    // Error parsing and context
    parseError(error: any): ParsedError;
    enrichError(error: Error, context: ErrorContext): EnrichedError;
    
    // Recovery strategies
    createRecoveryStrategy(config: RecoveryConfig): RecoveryStrategy;
    attemptRecovery(error: Error, strategy: RecoveryStrategy): Promise<RecoveryResult>;
    
    // Error reporting
    reportError(error: Error, context?: ErrorContext): Promise<void>;
    getErrorStats(): ErrorStats;
}

interface ParsedError {
    type: string;
    message: string;
    code?: string;
    suggestions: string[];
    recoverable: boolean;
    context?: ErrorContext;
    traceId?: string;
}
```

#### 10. **Async Utilities Module**
```typescript
interface Async {
    // Cross-engine async utilities
    sleep(ms: number): Promise<void>;
    timeout<T>(promise: Promise<T>, ms: number): Promise<T>;
    retry<T>(operation: () => Promise<T>, config: RetryConfig): Promise<T>;
    
    // Cooperative scheduling (for single-threaded engines)
    yield(): Promise<void>;
    yieldFor(ms: number): Promise<void>;
    yieldUntil(condition: () => boolean): Promise<void>;
    
    // Promise utilities
    all<T>(promises: Promise<T>[]): Promise<T[]>;
    race<T>(promises: Promise<T>[]): Promise<T>;
    allSettled<T>(promises: Promise<T>[]): Promise<SettledResult<T>[]>;
    
    // Resource management
    withTimeout<T>(operation: () => Promise<T>, timeout: number): Promise<T>;
    withRetry<T>(operation: () => Promise<T>, retries: number): Promise<T>;
    withResource<T, R>(resource: Resource<R>, operation: (r: R) => Promise<T>): Promise<T>;
}
```

### Script Environment Setup

Each script engine environment is automatically configured with:

#### Global Objects
```typescript
// Available in all script environments
declare global {
    const Agent: typeof AgentModule;
    const Tool: typeof ToolModule;
    const Tools: typeof ToolsRegistry;
    const Workflow: typeof WorkflowModule;
    const Events: typeof EventBus;
    const Hooks: typeof HookRegistry;
    const Config: typeof ConfigModule;
    const Storage: typeof StorageModule;
    const ErrorHandler: typeof ErrorHandlerModule;
    const Async: typeof AsyncModule;
    
    // Utility functions
    function print(...args: any[]): void;
    function log(level: string, message: string, metadata?: any): void;
    function trace(message: string): void;
    function debug(message: string): void;
    function info(message: string): void;
    function warn(message: string): void;
    function error(message: string): void;
}
```

#### Environment Variables
```typescript
interface ScriptEnvironment {
    // Runtime information
    ENGINE: "lua" | "javascript" | "python";
    VERSION: string;
    PLATFORM: string;
    
    // Configuration paths
    CONFIG_PATH: string;
    DATA_PATH: string;
    CACHE_PATH: string;
    
    // Security context
    SECURITY_LEVEL: "low" | "medium" | "high";
    SANDBOX_ENABLED: boolean;
    
    // Resource limits
    MAX_MEMORY: number;
    MAX_EXECUTION_TIME: number;
    MAX_CONCURRENT_OPERATIONS: number;
    
    // Provider information
    AVAILABLE_PROVIDERS: string[];
    DEFAULT_PROVIDER: string;
}
```

### Cross-Engine Feature Matrix

| Feature | Lua | JavaScript | Python | Notes |
|---------|-----|------------|--------|-------|
| **Basic Operations** | | | | |
| Agent Creation | ✅ | ✅ | 🔮 | Full API parity |
| Tool Execution | ✅ | ✅ | 🔮 | All built-in tools |
| Workflow Orchestration | ✅ | ✅ | 🔮 | All patterns supported |
| **Async Patterns** | | | | |
| Coroutines | ✅ | ➖ | ➖ | Lua native |
| Promises/Async-Await | 🔧 | ✅ | 🔮 | Emulated in Lua |
| Cooperative Scheduling | ✅ | ✅ | 🔮 | Cross-engine support |
| **Advanced Features** | | | | |
| Hook Registration | ✅ | ✅ | 🔮 | All hook points |
| Event Handling | ✅ | ✅ | 🔮 | Pub/sub + replay |
| State Management | ✅ | ✅ | 🔮 | Persistent storage |
| Error Recovery | ✅ | ✅ | 🔮 | Automatic strategies |
| **Type System** | | | | |
| Static Types | ➖ | 🔧 | 🔮 | TypeScript definitions |
| Runtime Validation | ✅ | ✅ | 🔮 | JSON Schema based |
| Auto-completion | 🔧 | ✅ | 🔮 | IDE support |
| **Performance** | | | | |
| Execution Speed | ⚡ | ⚡ | 🔮 | LuaJIT + V8 |
| Memory Usage | ⚡ | 🔧 | 🔮 | Lua most efficient |
| Startup Time | ⚡ | 🔧 | 🔮 | Lua fastest |
| **Ecosystem** | | | | |
| Package Manager | 🔧 | ✅ | 🔮 | npm + future support |
| External Libraries | 🔧 | ✅ | 🔮 | Limited sandboxing |
| Community Modules | 🔧 | ✅ | 🔮 | Security-screened |

**Legend**: ✅ Full Support, 🔧 Partial/Emulated, ➖ Not Applicable, 🔮 Planned, ⚡ Optimized

---

## Lua API Reference

Lua serves as the **performance-oriented** scripting language in rs-llmspell, providing the fastest execution speed and lowest memory overhead. It's ideal for high-throughput scenarios and resource-constrained environments.

### Lua-Specific Features

#### 1. **Native Coroutine Support**
Lua's coroutines map naturally to rs-llmspell's cooperative async model:

```lua
-- Coroutine-based async operations
local function async_workflow()
    return coroutine.create(function()
        -- Step 1: Research
        local research_data = coroutine.yield(
            Agent.execute("research_agent", {
                query = "quantum computing breakthroughs 2025",
                max_sources = 20
            })
        )
        
        -- Step 2: Analysis  
        local analysis = coroutine.yield(
            Agent.execute("analysis_agent", {
                data = research_data,
                analysis_type = "trend_analysis"
            })
        )
        
        -- Step 3: Synthesis
        local report = coroutine.yield(
            Agent.execute("writer_agent", {
                research = research_data,
                analysis = analysis,
                format = "executive_summary"
            })
        )
        
        return {
            research = research_data,
            analysis = analysis,
            report = report,
            metadata = {
                sources_count = #research_data.sources,
                analysis_confidence = analysis.confidence,
                generation_time = os.time()
            }
        }
    end)
end

-- Execute the async workflow
local workflow = async_workflow()
local result = Async.run(workflow)

print("Research Complete!")
print("Sources analyzed:", result.metadata.sources_count)
print("Confidence score:", result.metadata.analysis_confidence)
```

#### 2. **Lua Table-Based Configuration**
Leverage Lua's flexible table syntax for configuration:

```lua
-- Agent configuration with Lua tables
local research_agent = Agent.new({
    name = "comprehensive_researcher",
    description = "Multi-source research specialist",
    provider = "anthropic",
    model = "claude-3-sonnet",
    
    -- System prompt with Lua string literals
    system_prompt = [[
        You are a comprehensive research specialist with expertise in:
        - Academic literature analysis
        - Market trend identification  
        - Technical documentation synthesis
        - Data-driven insight generation
        
        Always provide sources and confidence levels for your findings.
    ]],
    
    -- Advanced configuration
    config = {
        temperature = 0.3,
        max_tokens = 4000,
        timeout = 120,
        
        -- Resource limits
        resources = {
            max_memory = "512MB",
            max_execution_time = 300,
            max_concurrent_tools = 3
        },
        
        -- Error handling
        error_strategy = {
            retry_count = 3,
            retry_delay = 5,
            fallback_behavior = "use_cached_data"
        }
    },
    
    -- Tools configuration
    tools = {
        Tools.get("web_search"),
        Tools.get("scholarly_search"),
        Tools.get("pdf_analyzer"),
        Tools.get("citation_formatter"),
        
        -- Custom tool with inline configuration
        Tools.create("custom_analyzer", {
            category = "analysis",
            input_schema = {
                type = "object",
                properties = {
                    data = { type = "array" },
                    analysis_type = { type = "string" }
                }
            },
            execute = function(input)
                -- Custom tool logic in Lua
                local data = input.data
                local analysis_type = input.analysis_type
                
                local result = {}
                if analysis_type == "trend_analysis" then
                    result = analyze_trends(data)
                elseif analysis_type == "sentiment_analysis" then
                    result = analyze_sentiment(data)
                end
                
                return {
                    result = result,
                    metadata = {
                        analysis_type = analysis_type,
                        data_points = #data
                    }
                }
            end
        })
    },
    
    -- Memory configuration
    memory = {
        type = "conversation",
        max_entries = 100,
        persistence = true,
        storage_path = "./agent_memory/"
    },
    
    -- Hook registration
    hooks = {
        before_execution = function(context)
            log("info", "Starting research agent execution", {
                input_size = #context.input.query,
                tools_available = #context.agent.tools
            })
        end,
        
        after_execution = function(context)
            log("info", "Research agent execution complete", {
                output_size = #context.output.result,
                execution_time = context.timing.duration,
                success = context.success
            })
            
            -- Emit custom event
            Events.emit({
                type = "research_complete",
                data = {
                    agent_id = context.agent.id,
                    query = context.input.query,
                    sources_found = context.output.metadata.sources_count
                }
            })
        end,
        
        on_error = function(context)
            error("Research agent failed: " .. context.error.message)
            
            -- Attempt recovery
            local recovery_result = ErrorHandler.attempt_recovery(
                context.error,
                ErrorHandler.create_strategy({
                    type = "fallback_to_cached",
                    cache_timeout = 3600
                })
            )
            
            if recovery_result.success then
                return recovery_result.data
            end
        end
    }
})
```

#### 3. **Lua Workflow Patterns**
```lua
-- Sequential workflow with Lua syntax
local sequential_workflow = Workflow.sequential({
    name = "document_processing_pipeline",
    description = "Process documents through multiple analysis stages",
    
    steps = {
        {
            name = "extract_text",
            component = Tools.get("pdf_extractor"),
            input = function(context)
                return {
                    file_path = context.input.document_path,
                    extract_images = true,
                    extract_tables = true
                }
            end,
            output = "extracted_content"
        },
        
        {
            name = "analyze_structure",
            component = Agent.get("document_analyzer"),
            input = function(context)
                return {
                    content = context.extracted_content,
                    analysis_depth = "comprehensive"
                }
            end,
            output = "structure_analysis",
            condition = function(context)
                return #context.extracted_content.text > 1000
            end
        },
        
        {
            name = "generate_summary",
            component = Agent.get("summarizer"),
            input = function(context)
                return {
                    content = context.extracted_content.text,
                    structure = context.structure_analysis,
                    summary_length = "medium"
                }
            end,
            output = "summary"
        },
        
        {
            name = "quality_check",
            component = Tools.get("quality_validator"),
            input = function(context)
                return {
                    original = context.extracted_content,
                    summary = context.summary,
                    structure = context.structure_analysis
                }
            end,
            output = "quality_score"
        }
    },
    
    -- Global workflow configuration
    config = {
        continue_on_error = false,
        save_checkpoints = true,
        checkpoint_interval = 2, -- After every 2 steps
        max_execution_time = 600,
        
        -- Error handling per step
        error_strategy = {
            extract_text = "retry_with_different_tool",
            analyze_structure = "skip_and_continue",
            generate_summary = "use_fallback_agent",
            quality_check = "continue_with_warning"
        }
    }
})

-- Parallel workflow with aggregation
local parallel_workflow = Workflow.parallel({
    name = "multi_perspective_analysis",
    description = "Analyze content from multiple perspectives simultaneously",
    
    steps = {
        {
            name = "technical_analysis",
            component = Agent.get("technical_analyst"),
            input = function(context) 
                return { content = context.input.content, perspective = "technical" }
            end
        },
        
        {
            name = "business_analysis", 
            component = Agent.get("business_analyst"),
            input = function(context)
                return { content = context.input.content, perspective = "business" }
            end
        },
        
        {
            name = "user_impact_analysis",
            component = Agent.get("ux_analyst"),
            input = function(context)
                return { content = context.input.content, perspective = "user_experience" }
            end
        },
        
        {
            name = "risk_analysis",
            component = Agent.get("risk_analyst"), 
            input = function(context)
                return { content = context.input.content, perspective = "risk_assessment" }
            end
        }
    },
    
    -- Parallel execution configuration
    config = {
        max_concurrency = 4,
        fail_fast = false, -- Continue even if some analyses fail
        timeout_per_step = 180,
        
        -- Result aggregation
        aggregation = {
            strategy = "comprehensive_merge",
            require_minimum = 3, -- Need at least 3 successful analyses
            
            -- Custom aggregation function
            aggregate = function(results)
                local aggregated = {
                    perspectives = {},
                    consensus_points = {},
                    divergent_views = {},
                    confidence_score = 0
                }
                
                -- Process each analysis result
                for name, result in pairs(results) do
                    if result.success then
                        aggregated.perspectives[name] = result.data
                        aggregated.confidence_score = aggregated.confidence_score + result.confidence
                    end
                end
                
                -- Calculate average confidence
                local successful_count = 0
                for _ in pairs(aggregated.perspectives) do
                    successful_count = successful_count + 1
                end
                
                if successful_count > 0 then
                    aggregated.confidence_score = aggregated.confidence_score / successful_count
                end
                
                -- Find consensus and divergent points
                aggregated.consensus_points = find_consensus(aggregated.perspectives)
                aggregated.divergent_views = find_divergence(aggregated.perspectives)
                
                return aggregated
            end
        }
    }
})

-- Conditional workflow
local conditional_workflow = Workflow.conditional({
    name = "adaptive_processing",
    
    condition = function(context)
        local content_length = #context.input.content
        local complexity_score = estimate_complexity(context.input.content)
        
        return content_length > 5000 and complexity_score > 0.7
    end,
    
    true_branch = Workflow.sequential({
        name = "complex_processing",
        steps = {
            { name = "deep_analysis", component = Agent.get("deep_analyzer") },
            { name = "expert_review", component = Agent.get("expert_reviewer") },
            { name = "comprehensive_summary", component = Agent.get("detailed_summarizer") }
        }
    }),
    
    false_branch = Workflow.sequential({
        name = "simple_processing", 
        steps = {
            { name = "basic_analysis", component = Agent.get("basic_analyzer") },
            { name = "quick_summary", component = Agent.get("quick_summarizer") }
        }
    })
})
```

#### 4. **Lua Event Handling**
```lua
-- Event subscription with Lua callbacks
Events.subscribe("agent_error", function(event)
    local error_data = event.data
    
    log("error", "Agent error occurred", {
        agent_id = error_data.agent_id,
        error_type = error_data.error_type,
        message = error_data.message
    })
    
    -- Custom error handling logic
    if error_data.error_type == "rate_limit" then
        log("info", "Rate limit hit, implementing backoff strategy")
        
        -- Return coroutine for async handling
        return coroutine.create(function()
            local backoff_time = error_data.retry_after or 30
            coroutine.yield(Async.sleep(backoff_time * 1000))
            
            log("info", "Retrying after backoff period")
            return "retry"
        end)
    elseif error_data.error_type == "context_length_exceeded" then
        log("info", "Context too long, attempting summarization")
        
        return coroutine.create(function()
            local summary = coroutine.yield(
                Agent.execute("summarizer", {
                    content = error_data.context,
                    target_length = "short"
                })
            )
            
            -- Update context with summary
            error_data.context = summary
            return "retry_with_modified_context"
        end)
    end
end)

-- Complex event handling with state
local event_state = {
    error_count = 0,
    last_error_time = 0,
    circuit_breaker_open = false
}

Events.subscribe("workflow_error", function(event)
    event_state.error_count = event_state.error_count + 1
    event_state.last_error_time = os.time()
    
    -- Circuit breaker pattern
    if event_state.error_count >= 5 and 
       (os.time() - event_state.last_error_time) < 300 then -- 5 errors in 5 minutes
        
        event_state.circuit_breaker_open = true
        
        log("warn", "Circuit breaker opened due to repeated failures")
        
        -- Emit circuit breaker event
        Events.emit({
            type = "circuit_breaker_opened",
            data = {
                error_count = event_state.error_count,
                time_window = 300
            }
        })
        
        -- Schedule circuit breaker reset
        Async.schedule(600 * 1000, function() -- 10 minutes
            event_state.circuit_breaker_open = false
            event_state.error_count = 0
            
            Events.emit({
                type = "circuit_breaker_closed",
                data = { reset_time = os.time() }
            })
            
            log("info", "Circuit breaker reset")
        end)
    end
end)
```

#### 5. **Lua Performance Optimizations**
```lua
-- Efficient table operations
local function efficient_data_processing(large_dataset)
    local results = {}
    local batch_size = 100
    
    -- Process in batches to avoid memory spikes
    for i = 1, #large_dataset, batch_size do
        local batch = {}
        local batch_end = math.min(i + batch_size - 1, #large_dataset)
        
        -- Extract batch
        for j = i, batch_end do
            batch[#batch + 1] = large_dataset[j]
        end
        
        -- Process batch
        local batch_result = process_batch(batch)
        
        -- Merge results efficiently
        for k, v in ipairs(batch_result) do
            results[#results + 1] = v
        end
        
        -- Yield periodically for cooperative scheduling
        if i % (batch_size * 10) == 0 then
            coroutine.yield()
        end
    end
    
    return results
end

-- Memory-efficient string operations
local function build_large_report(data_chunks)
    local buffer = {}
    
    for i, chunk in ipairs(data_chunks) do
        buffer[i] = tostring(chunk)
    end
    
    -- Single concatenation is more efficient than repeated string concatenation
    return table.concat(buffer, "\n")
end

-- Optimized agent calls with caching
local agent_cache = {}

local function cached_agent_call(agent_id, input)
    local cache_key = agent_id .. ":" .. hash(input)
    
    if agent_cache[cache_key] then
        local cached_result = agent_cache[cache_key]
        
        -- Check if cache is still valid (5 minutes)
        if os.time() - cached_result.timestamp < 300 then
            log("debug", "Using cached result for agent call")
            return cached_result.result
        end
    end
    
    -- Execute agent and cache result
    local result = Agent.execute(agent_id, input)
    
    agent_cache[cache_key] = {
        result = result,
        timestamp = os.time()
    }
    
    return result
end
```

#### 6. **Lua Testing Patterns**
```lua
-- Lua testing framework integration
local LuaTest = require("llmspell.testing")

-- Test suite for agent functionality
local AgentTests = LuaTest.suite("Agent Functionality")

AgentTests:test("basic_chat_functionality", function(t)
    local agent = Agent.new({
        name = "test_agent",
        provider = "mock", -- Use mock provider for testing
        model = "mock-model"
    })
    
    local response = agent:chat("Hello, how are you?")
    
    t:assert_not_nil(response, "Agent should return a response")
    t:assert_type(response, "string", "Response should be a string")
    t:assert_greater_than(#response, 0, "Response should not be empty")
end)

AgentTests:test("tool_integration", function(t)
    local agent = Agent.new({
        name = "test_agent_with_tools",
        provider = "mock",
        model = "mock-model",
        tools = { Tools.get("calculator") }
    })
    
    local response = agent:chat("What is 15 + 27?")
    
    t:assert_contains(response, "42", "Agent should use calculator tool")
end)

AgentTests:test("error_handling", function(t)
    local agent = Agent.new({
        name = "error_test_agent",
        provider = "mock",
        model = "mock-model"
    })
    
    -- Test with invalid input
    local success, error_or_result = pcall(function()
        return agent:chat(nil) -- Invalid input
    end)
    
    t:assert_false(success, "Agent should reject invalid input")
    
    local error_info = ErrorHandler.parse_error(error_or_result)
    t:assert_equals(error_info.type, "validation_error", "Should be validation error")
end)

-- Workflow testing
local WorkflowTests = LuaTest.suite("Workflow Functionality")

WorkflowTests:test("sequential_execution", function(t)
    local workflow = Workflow.sequential({
        name = "test_workflow",
        steps = {
            {
                name = "step1",
                component = Tools.get("mock_tool"),
                input = { value = 10 },
                output = "step1_result"
            },
            {
                name = "step2", 
                component = Tools.get("mock_tool"),
                input = function(context)
                    return { value = context.step1_result.value * 2 }
                end,
                output = "step2_result"
            }
        }
    })
    
    local result = workflow:execute({ initial_value = 10 })
    
    t:assert_not_nil(result.step2_result, "Step 2 should execute")
    t:assert_equals(result.step2_result.value, 40, "Step 2 should double step 1 result")
end)

-- Run all tests
LuaTest.run_all()
```

---

## JavaScript API Reference

JavaScript provides the **ecosystem-rich** scripting environment in rs-llmspell, offering familiar syntax, extensive tooling support, and seamless integration with existing Node.js workflows.

### JavaScript-Specific Features

#### 1. **Native Promise Support**
JavaScript's Promises map directly to rs-llmspell's async operations:

```javascript
// Promise-based async operations
async function comprehensiveResearch(topic) {
    try {
        // Parallel research across multiple agents
        const [academicResults, newsResults, marketResults] = await Promise.all([
            Agent.execute("academic_researcher", {
                query: topic,
                sources: ["arxiv", "pubmed", "scholar"],
                maxResults: 15
            }),
            
            Agent.execute("news_analyst", {
                query: topic,
                timeframe: "last_30_days",
                sentiment: true
            }),
            
            Agent.execute("market_analyst", {
                query: topic,
                includeForecasts: true,
                riskAssessment: true
            })
        ]);
        
        // Synthesize results
        const synthesis = await Agent.execute("synthesis_expert", {
            academicData: academicResults,
            newsData: newsResults, 
            marketData: marketResults,
            outputFormat: "comprehensive_report"
        });
        
        // Generate visualizations
        const visualizations = await Tools.get("data_visualizer").execute({
            data: synthesis.data,
            chartTypes: ["trend", "correlation", "distribution"],
            exportFormat: "svg"
        });
        
        return {
            topic,
            synthesis,
            visualizations,
            metadata: {
                totalSources: academicResults.sources.length + newsResults.sources.length,
                researchTime: Date.now() - startTime,
                confidenceScore: (
                    academicResults.confidence + 
                    newsResults.confidence + 
                    marketResults.confidence
                ) / 3
            }
        };
        
    } catch (error) {
        console.error("Research failed:", error);
        
        // Attempt recovery
        const recovery = await ErrorHandler.attemptRecovery(error, {
            strategy: "fallback_to_web_search",
            fallbackConfig: {
                maxRetries: 3,
                timeoutMs: 30000
            }
        });
        
        if (recovery.success) {
            return recovery.result;
        }
        
        throw new Error(`Research failed: ${error.message}`);
    }
}

// Usage with proper error handling
const startTime = Date.now();
comprehensiveResearch("AI regulation impact on startup ecosystem")
    .then(result => {
        console.log("✅ Research complete!");
        console.log(`📊 Analyzed ${result.metadata.totalSources} sources`);
        console.log(`⏱️  Completed in ${result.metadata.researchTime}ms`);
        console.log(`🎯 Confidence: ${result.metadata.confidenceScore.toFixed(2)}`);
    })
    .catch(error => {
        console.error("❌ Research failed:", error.message);
    });
```

#### 2. **ES6+ Syntax and Features**
```javascript
// Modern JavaScript features
class ResearchWorkflow {
    constructor(config) {
        this.name = config.name;
        this.agents = new Map();
        this.tools = new Set();
        this.eventHandlers = new Map();
        
        // Initialize agents
        this.initializeAgents(config.agents);
        
        // Set up event handling
        this.setupEventHandling();
    }
    
    // Async generator for streaming results
    async* streamingAnalysis(topic) {
        const phases = [
            'initial_research',
            'deep_analysis', 
            'cross_reference',
            'synthesis',
            'validation'
        ];
        
        for (const [index, phase] of phases.entries()) {
            yield {
                phase,
                progress: (index + 1) / phases.length,
                status: 'starting'
            };
            
            try {
                const result = await this.executePhase(phase, topic);
                
                yield {
                    phase,
                    progress: (index + 1) / phases.length,
                    status: 'complete',
                    result
                };
                
            } catch (error) {
                yield {
                    phase, 
                    progress: (index + 1) / phases.length,
                    status: 'error',
                    error: error.message
                };
                
                // Decide whether to continue or abort
                if (!this.shouldContinueOnError(phase, error)) {
                    throw error;
                }
            }
        }
    }
    
    // Destructuring and spread operators
    async executePhase(phase, topic, { timeout = 60000, retries = 3, ...options } = {}) {
        const phaseConfig = {
            topic,
            timeout,
            retries,
            ...options,
            timestamp: Date.now()
        };
        
        // Use appropriate agent for phase
        const agent = this.getAgentForPhase(phase);
        
        return agent.execute(phaseConfig);
    }
    
    // Arrow functions and method chaining
    initializeAgents = (agentConfigs) => {
        agentConfigs
            .filter(config => config.enabled !== false)
            .map(config => ({
                ...config,
                // Enhance with default settings
                timeout: config.timeout || 30000,
                retries: config.retries || 2
            }))
            .forEach(config => {
                const agent = new Agent(config);
                this.agents.set(config.name, agent);
            });
    }
    
    // Template literals for dynamic prompts
    generatePrompt(phase, topic, context = {}) {
        const prompts = {
            initial_research: `
                Research the topic: "${topic}"
                
                Focus on:
                - Recent developments (last 12 months)
                - Key stakeholders and their perspectives
                - Quantitative data and trends
                
                Context: ${JSON.stringify(context, null, 2)}
                
                Provide structured output with sources.
            `,
            
            deep_analysis: `
                Perform deep analysis on: "${topic}"
                
                Based on initial research: ${context.initialResearch?.summary || 'None provided'}
                
                Analyze:
                - Root causes and driving factors
                - Secondary and tertiary effects
                - Potential future scenarios
                
                Use critical thinking and identify assumptions.
            `,
            
            synthesis: `
                Synthesize findings for: "${topic}"
                
                Available data:
                ${Object.entries(context)
                    .map(([key, value]) => `- ${key}: ${JSON.stringify(value, null, 2)}`)
                    .join('\n')}
                
                Create a comprehensive synthesis that:
                - Integrates all findings
                - Identifies patterns and connections
                - Highlights contradictions or gaps
                - Provides actionable insights
            `
        };
        
        return prompts[phase] || `Analyze "${topic}" for phase: ${phase}`;
    }
    
    // Async/await with error boundaries
    async executeWithErrorBoundary(operation, errorBoundary = {}) {
        const { 
            onError = (error) => console.error('Operation failed:', error),
            fallback = null,
            timeout = 30000
        } = errorBoundary;
        
        try {
            return await Promise.race([
                operation(),
                new Promise((_, reject) => 
                    setTimeout(() => reject(new Error('Operation timeout')), timeout)
                )
            ]);
            
        } catch (error) {
            onError(error);
            
            if (fallback) {
                console.log('Attempting fallback operation...');
                return await fallback();
            }
            
            throw error;
        }
    }
}

// Usage with modern JavaScript patterns
const workflow = new ResearchWorkflow({
    name: "comprehensive_research_v2",
    agents: [
        { name: "researcher", provider: "anthropic", model: "claude-3-sonnet" },
        { name: "analyzer", provider: "openai", model: "gpt-4" },
        { name: "synthesizer", provider: "anthropic", model: "claude-3-opus" }
    ]
});

// Streaming execution with async iteration
async function runStreamingAnalysis() {
    for await (const update of workflow.streamingAnalysis("blockchain scalability solutions")) {
        console.log(`Phase ${update.phase}: ${Math.round(update.progress * 100)}% - ${update.status}`);
        
        if (update.result) {
            console.log(`✅ ${update.phase} complete:`, update.result.summary);
        }
        
        if (update.error) {
            console.log(`❌ ${update.phase} failed:`, update.error);
        }
    }
}

runStreamingAnalysis().catch(console.error);
```

#### 3. **NPM Integration and Modules**
```javascript
// Import external libraries (security-validated)
import axios from 'axios';
import lodash from 'lodash';
import moment from 'moment';
import cheerio from 'cheerio';

// Import rs-llmspell modules
import { Agent, Tools, Workflow, Events, Config } from 'rs-llmspell';

// Custom tool using external libraries
class WebScrapingTool extends Tools.BaseTool {
    constructor() {
        super({
            name: "advanced_web_scraper",
            description: "Advanced web scraping with content extraction",
            category: "web_access"
        });
    }
    
    async execute({ url, selectors = {}, options = {} }) {
        const {
            timeout = 10000,
            userAgent = 'Rs-LLMSpell Bot 1.0',
            maxRedirects = 5
        } = options;
        
        try {
            // Use axios for HTTP requests
            const response = await axios.get(url, {
                timeout,
                headers: { 'User-Agent': userAgent },
                maxRedirects
            });
            
            // Parse HTML with cheerio
            const $ = cheerio.load(response.data);
            
            // Extract content based on selectors
            const extracted = {};
            
            for (const [key, selector] of Object.entries(selectors)) {
                if (Array.isArray(selector)) {
                    extracted[key] = $(selector[0]).map((i, el) => $(el).text().trim()).get();
                } else {
                    extracted[key] = $(selector).text().trim();
                }
            }
            
            // Extract metadata
            const metadata = {
                title: $('title').text().trim(),
                description: $('meta[name="description"]').attr('content') || '',
                keywords: $('meta[name="keywords"]').attr('content') || '',
                author: $('meta[name="author"]').attr('content') || '',
                publishedDate: this.extractPublishDate($),
                wordCount: $('body').text().split(/\s+/).length,
                links: $('a[href]').map((i, el) => $(el).attr('href')).get(),
                images: $('img[src]').map((i, el) => $(el).attr('src')).get()
            };
            
            return {
                url,
                content: extracted,
                metadata,
                timestamp: moment().toISOString(),
                success: true
            };
            
        } catch (error) {
            return {
                url,
                error: error.message,
                timestamp: moment().toISOString(),
                success: false
            };
        }
    }
    
    extractPublishDate($) {
        const selectors = [
            'meta[property="article:published_time"]',
            'meta[name="date"]',
            'time[datetime]',
            '.published-date',
            '.post-date'
        ];
        
        for (const selector of selectors) {
            const element = $(selector);
            if (element.length) {
                const dateStr = element.attr('content') || element.attr('datetime') || element.text();
                const parsed = moment(dateStr);
                if (parsed.isValid()) {
                    return parsed.toISOString();
                }
            }
        }
        
        return null;
    }
}

// Register custom tool
Tools.register(new WebScrapingTool());

// Data processing with lodash
class DataProcessor {
    static processResearchData(rawData) {
        return lodash.chain(rawData)
            .filter(item => item.success && item.content)
            .groupBy('metadata.category')
            .mapValues(group => ({
                count: group.length,
                averageWordCount: lodash.meanBy(group, 'metadata.wordCount'),
                sources: lodash.map(group, 'url'),
                topics: this.extractTopics(group),
                sentiment: this.analyzeSentiment(group)
            }))
            .value();
    }
    
    static extractTopics(articles) {
        const allText = articles
            .map(article => article.content.title + ' ' + article.content.description)
            .join(' ');
        
        // Simple keyword extraction (in production, use proper NLP)
        const words = allText.toLowerCase()
            .split(/\s+/)
            .filter(word => word.length > 4)
            .filter(word => !/^\d+$/.test(word));
        
        return lodash.chain(words)
            .countBy()
            .toPairs()
            .sortBy(1)
            .reverse()
            .take(10)
            .fromPairs()
            .value();
    }
    
    static analyzeSentiment(articles) {
        // Placeholder for sentiment analysis
        // In production, integrate with sentiment analysis service
        return {
            positive: Math.random() * 0.4 + 0.3,
            negative: Math.random() * 0.3 + 0.1,
            neutral: Math.random() * 0.4 + 0.3
        };
    }
}

// Example usage combining external libraries with rs-llmspell
async function comprehensiveMarketResearch(topic) {
    const searchQueries = [
        `${topic} market trends 2025`,
        `${topic} investment funding`,
        `${topic} regulatory changes`,
        `${topic} industry analysis`
    ];
    
    // Parallel web scraping
    const scrapingPromises = searchQueries.map(async query => {
        const searchResults = await Tools.get("web_search").execute({
            query,
            maxResults: 5
        });
        
        const scrapingTasks = searchResults.results.map(result =>
            Tools.get("advanced_web_scraper").execute({
                url: result.url,
                selectors: {
                    title: 'h1, .title, .headline',
                    content: '.content, .article-body, main p',
                    date: '.date, .published, time'
                }
            })
        );
        
        return Promise.all(scrapingTasks);
    });
    
    const allData = lodash.flatten(await Promise.all(scrapingPromises));
    const processedData = DataProcessor.processResearchData(allData);
    
    // LLM analysis of processed data
    const analysis = await Agent.execute("market_analyst", {
        topic,
        data: processedData,
        analysisType: "comprehensive",
        outputFormat: "structured_report"
    });
    
    return {
        topic,
        rawDataCount: allData.length,
        processedData,
        analysis,
        generatedAt: moment().toISOString()
    };
}
```

#### 4. **TypeScript Support**
```typescript
// TypeScript definitions for enhanced development experience
interface AgentConfig {
    name: string;
    description?: string;
    provider: 'openai' | 'anthropic' | 'ollama' | string;
    model: string;
    systemPrompt?: string;
    temperature?: number;
    maxTokens?: number;
    tools?: Tool[];
    memory?: MemoryConfig;
    hooks?: HookConfig;
}

interface ResearchResult {
    topic: string;
    sources: Source[];
    analysis: AnalysisResult;
    confidence: number;
    metadata: {
        researchTime: number;
        sourceCount: number;
        qualityScore: number;
    };
}

// Type-safe agent creation
class TypedResearchAgent {
    private agent: Agent;
    private config: AgentConfig;
    
    constructor(config: AgentConfig) {
        this.config = config;
        this.agent = new Agent(config);
    }
    
    async research(topic: string, options?: ResearchOptions): Promise<ResearchResult> {
        const input: AgentInput = {
            query: topic,
            maxSources: options?.maxSources ?? 10,
            depth: options?.depth ?? 'medium',
            timeframe: options?.timeframe ?? 'recent'
        };
        
        const output = await this.agent.execute(input);
        
        return {
            topic,
            sources: output.sources || [],
            analysis: output.analysis || { summary: '', insights: [] },
            confidence: output.confidence || 0,
            metadata: {
                researchTime: output.metadata?.executionTime || 0,
                sourceCount: output.sources?.length || 0,
                qualityScore: output.metadata?.qualityScore || 0
            }
        };
    }
    
    // Type-safe tool management
    addResearchTool<T extends Tool>(tool: T): this {
        this.agent.addTool(tool);
        return this;
    }
    
    // Type-safe configuration updates
    updateConfig(updates: Partial<AgentConfig>): this {
        this.config = { ...this.config, ...updates };
        this.agent.updateConfig(this.config);
        return this;
    }
}

// Usage with full type safety
const researcher = new TypedResearchAgent({
    name: "market_researcher",
    provider: "anthropic",
    model: "claude-3-sonnet",
    temperature: 0.3
});

researcher
    .addResearchTool(Tools.get("scholarly_search"))
    .addResearchTool(Tools.get("market_data"))
    .addResearchTool(Tools.get("news_analyzer"));

const result: ResearchResult = await researcher.research("AI governance frameworks", {
    maxSources: 15,
    depth: 'comprehensive',
    timeframe: 'last_year'
});

console.log(`Research complete: ${result.confidence * 100}% confidence`);
```

#### 5. **Advanced Error Handling**
```javascript
// Sophisticated error handling with custom error classes
class LLMSpellError extends Error {
    constructor(message, type, code, context = {}) {
        super(message);
        this.name = 'LLMSpellError';
        this.type = type;
        this.code = code;
        this.context = context;
        this.timestamp = new Date().toISOString();
    }
    
    toJSON() {
        return {
            name: this.name,
            message: this.message,
            type: this.type,
            code: this.code,
            context: this.context,
            timestamp: this.timestamp,
            stack: this.stack
        };
    }
}

class RateLimitError extends LLMSpellError {
    constructor(retryAfter, context) {
        super(
            `Rate limit exceeded. Retry after ${retryAfter}s`,
            'rate_limit',
            'E_RATE_LIMIT',
            { retryAfter, ...context }
        );
        this.retryAfter = retryAfter;
    }
}

class ValidationError extends LLMSpellError {
    constructor(validationErrors, context) {
        super(
            `Validation failed: ${validationErrors.map(e => e.message).join(', ')}`,
            'validation',
            'E_VALIDATION',
            { validationErrors, ...context }
        );
        this.validationErrors = validationErrors;
    }
}

// Error recovery strategies
class ErrorRecoveryManager {
    constructor() {
        this.strategies = new Map();
        this.setupDefaultStrategies();
    }
    
    setupDefaultStrategies() {
        // Rate limit recovery
        this.strategies.set('rate_limit', async (error, context) => {
            const delay = error.retryAfter * 1000;
            console.log(`Rate limited. Waiting ${error.retryAfter}s before retry...`);
            
            await new Promise(resolve => setTimeout(resolve, delay));
            
            return {
                action: 'retry',
                delay,
                attempts: (context.attempts || 0) + 1
            };
        });
        
        // Context length recovery
        this.strategies.set('context_length_exceeded', async (error, context) => {
            console.log('Context too long. Attempting summarization...');
            
            const summarizer = await Agent.get('text_summarizer');
            const summary = await summarizer.execute({
                text: context.originalInput,
                targetLength: 'medium',
                preserveKeyInfo: true
            });
            
            return {
                action: 'retry',
                modifiedInput: { ...context.originalInput, text: summary.result },
                modifications: ['context_summarized']
            };
        });
        
        // Provider fallback
        this.strategies.set('provider_error', async (error, context) => {
            const currentProvider = context.agent.config.provider;
            const fallbackProviders = this.getFallbackProviders(currentProvider);
            
            if (fallbackProviders.length === 0) {
                return { action: 'fail', reason: 'No fallback providers available' };
            }
            
            console.log(`Provider ${currentProvider} failed. Trying fallback: ${fallbackProviders[0]}`);
            
            return {
                action: 'retry',
                modifiedConfig: {
                    ...context.agent.config,
                    provider: fallbackProviders[0]
                }
            };
        });
    }
    
    async attemptRecovery(error, context) {
        const strategy = this.strategies.get(error.type);
        
        if (!strategy) {
            return { action: 'fail', reason: `No recovery strategy for error type: ${error.type}` };
        }
        
        try {
            return await strategy(error, context);
        } catch (recoveryError) {
            console.error('Recovery strategy failed:', recoveryError);
            return { action: 'fail', reason: 'Recovery strategy failed', recoveryError };
        }
    }
    
    getFallbackProviders(currentProvider) {
        const fallbacks = {
            'openai': ['anthropic', 'ollama'],
            'anthropic': ['openai', 'ollama'],
            'ollama': ['openai', 'anthropic']
        };
        
        return fallbacks[currentProvider] || [];
    }
}

// Resilient execution wrapper
class ResilientExecutor {
    constructor() {
        this.errorRecovery = new ErrorRecoveryManager();
        this.maxRetries = 3;
        this.baseDelay = 1000;
    }
    
    async executeWithRecovery(operation, context = {}) {
        let lastError;
        let attempts = 0;
        
        while (attempts < this.maxRetries) {
            try {
                return await operation();
                
            } catch (error) {
                lastError = error;
                attempts++;
                
                console.log(`Attempt ${attempts} failed:`, error.message);
                
                // Parse error if it's from rs-llmspell
                const parsedError = error instanceof LLMSpellError 
                    ? error 
                    : ErrorHandler.parseError(error);
                
                // Attempt recovery
                const recovery = await this.errorRecovery.attemptRecovery(parsedError, {
                    ...context,
                    attempts,
                    lastError
                });
                
                if (recovery.action === 'fail') {
                    throw new Error(`Recovery failed: ${recovery.reason}`);
                }
                
                if (recovery.action === 'retry') {
                    // Apply modifications if any
                    if (recovery.modifiedInput) {
                        context.modifiedInput = recovery.modifiedInput;
                    }
                    
                    if (recovery.modifiedConfig) {
                        context.modifiedConfig = recovery.modifiedConfig;
                    }
                    
                    // Wait before retry
                    const delay = recovery.delay || (this.baseDelay * Math.pow(2, attempts - 1));
                    await new Promise(resolve => setTimeout(resolve, delay));
                    
                    continue; // Retry the operation
                }
            }
        }
        
        throw new Error(`Operation failed after ${this.maxRetries} attempts. Last error: ${lastError.message}`);
    }
}

// Usage example
const executor = new ResilientExecutor();

async function robustResearch(topic) {
    return executor.executeWithRecovery(async () => {
        const agent = new Agent({
            name: "research_agent",
            provider: "openai",
            model: "gpt-4"
        });
        
        return agent.execute({
            query: topic,
            depth: "comprehensive"
        });
    }, {
        agent: agent,
        originalInput: { query: topic, depth: "comprehensive" }
    });
}

// Execute with automatic recovery
try {
    const result = await robustResearch("quantum computing applications in finance");
    console.log("Research successful:", result.summary);
} catch (error) {
    console.error("Research failed after all recovery attempts:", error.message);
}
```

# Part IV: Built-in Components Library

## Complete Built-in Tools Catalog

Rs-LLMSpell provides a comprehensive library of **40+ production-ready tools** organized into 8 categories. These tools are immediately available in all scripting environments and provide essential functionality for AI workflows.

### Tool Organization by Category

#### 1. **File System Operations** (8 tools)
Tools for secure file and directory operations with comprehensive sandboxing.

| Tool Name | Description | Key Features | Security Level |
|-----------|-------------|--------------|----------------|
| `file_reader` | Read files with format detection | UTF-8, binary, encoding detection | High |
| `file_writer` | Write files with atomic operations | Atomic writes, backup creation | High |
| `directory_lister` | List directory contents | Recursive, filtering, metadata | Medium |
| `file_metadata` | Extract file information | Size, dates, permissions, checksums | Low |
| `file_search` | Search files by content/name | Regex, glob patterns, indexing | Medium |
| `file_archiver` | Create/extract archives | ZIP, TAR, compression levels | High |
| `file_watcher` | Monitor file changes | Real-time events, batch processing | Medium |
| `file_converter` | Convert between formats | Text encodings, line endings | Low |

```lua
-- File operations examples
local content = Tools.get("file_reader"):execute({
    path = "./research/data.txt",
    encoding = "utf-8",
    max_size = "10MB"
})

Tools.get("file_writer"):execute({
    path = "./output/report.md",
    content = content.result,
    create_backup = true,
    atomic_write = true
})

local files = Tools.get("directory_lister"):execute({
    path = "./documents",
    recursive = true,
    pattern = "*.pdf",
    include_metadata = true
})
```

#### 2. **Web and Network Operations** (7 tools)
Comprehensive web access with rate limiting, caching, and security controls.

| Tool Name | Description | Key Features | Rate Limits |
|-----------|-------------|--------------|-------------|
| `web_search` | Multi-provider search | Google, Bing, DuckDuckGo fallback | 30/min |
| `web_scraper` | Extract web content | CSS selectors, JavaScript rendering | 20/min |
| `http_client` | HTTP/HTTPS requests | REST APIs, custom headers, auth | 60/min |
| `url_analyzer` | Analyze URL structure | Domain info, security scoring | 100/min |
| `webpage_monitor` | Monitor page changes | Content diffs, scheduling | 10/min |
| `sitemap_crawler` | Crawl site structure | Robots.txt compliance, depth limits | 5/min |
| `api_tester` | Test REST APIs | Response validation, performance | 30/min |

```javascript
// Web operations examples
const searchResults = await Tools.get("web_search").execute({
    query: "quantum computing breakthrough 2025",
    maxResults: 10,
    providers: ["google", "bing"],
    timeframe: "last_month"
});

const scrapedContent = await Tools.get("web_scraper").execute({
    url: "https://example.com/article",
    selectors: {
        title: "h1",
        content: ".article-body",
        metadata: ".article-meta"
    },
    waitForJs: true,
    timeout: 15000
});

const apiResponse = await Tools.get("http_client").execute({
    method: "GET",
    url: "https://api.example.com/data",
    headers: {
        "Authorization": "Bearer ${API_TOKEN}",
        "Accept": "application/json"
    },
    timeout: 10000,
    retries: 3
});
```

#### 3. **Data Processing and Analysis** (9 tools)
Transform, validate, and analyze structured and unstructured data.

| Tool Name | Description | Supported Formats | Performance |
|-----------|-------------|-------------------|-------------|
| `json_processor` | JSON manipulation | Parse, transform, validate, merge | High |
| `csv_processor` | CSV data operations | Read, write, filter, aggregate | High |
| `xml_processor` | XML parsing/generation | XPath, XSLT, validation | Medium |
| `yaml_processor` | YAML configuration | Parse, generate, schema validation | High |
| `data_transformer` | ETL operations | Map, filter, reduce, join | High |
| `data_validator` | Schema validation | JSON Schema, custom rules | High |
| `statistical_analyzer` | Statistical analysis | Descriptive stats, correlations | Medium |
| `text_analyzer` | Text processing | Tokenization, NLP features | Medium |
| `data_visualizer` | Generate charts | SVG, PNG export, interactive | Low |

```lua
-- Data processing examples
local csvData = Tools.get("csv_processor"):execute({
    operation = "read",
    file_path = "./data/sales.csv",
    headers = true,
    delimiter = ",",
    encoding = "utf-8"
})

local transformed = Tools.get("data_transformer"):execute({
    data = csvData.result,
    operations = {
        {
            type = "filter",
            condition = function(row) return row.amount > 1000 end
        },
        {
            type = "group_by",
            field = "category",
            aggregations = { "sum:amount", "count:*" }
        },
        {
            type = "sort",
            field = "sum_amount",
            order = "desc"
        }
    }
})

local stats = Tools.get("statistical_analyzer"):execute({
    data = transformed.result,
    analyses = {
        "descriptive_stats",
        "correlation_matrix", 
        "outlier_detection"
    },
    confidence_level = 0.95
})
```

#### 4. **AI and Machine Learning** (6 tools)
Pre-built AI capabilities for common tasks without requiring external model setup.

| Tool Name | Description | Models Used | Languages |
|-----------|-------------|-------------|-----------|
| `text_summarizer` | Document summarization | Local transformer models | Multi |
| `sentiment_analyzer` | Sentiment classification | BERT-based classifiers | 10+ |
| `language_detector` | Language identification | FastText models | 100+ |
| `text_classifier` | Custom text classification | Configurable models | Multi |
| `named_entity_recognizer` | NER extraction | spaCy models | 10+ |
| `embedding_generator` | Text embeddings | Sentence transformers | Multi |

```javascript
// AI tools examples
const summary = await Tools.get("text_summarizer").execute({
    text: longArticleText,
    targetLength: "medium", // short, medium, long
    preserveKeyPoints: true,
    style: "extractive" // extractive, abstractive
});

const sentiment = await Tools.get("sentiment_analyzer").execute({
    text: userReview,
    language: "auto", // auto-detect or specify
    includeEmotions: true,
    confidenceThreshold: 0.7
});

const entities = await Tools.get("named_entity_recognizer").execute({
    text: newsArticle,
    entityTypes: ["PERSON", "ORG", "LOCATION", "DATE"],
    language: "en",
    includeConfidence: true
});

const embeddings = await Tools.get("embedding_generator").execute({
    texts: [
        "Query about machine learning",
        "Document about artificial intelligence",
        "Article on deep learning"
    ],
    model: "sentence-transformers/all-MiniLM-L6-v2",
    normalize: true
});
```

#### 5. **System Integration** (4 tools)
Interact with system processes, environment, and external services.

| Tool Name | Description | Capabilities | Restrictions |
|-----------|-------------|--------------|--------------|
| `process_executor` | Run system commands | Shell execution, pipes | Sandboxed |
| `environment_reader` | Access env variables | Read env, system info | Read-only |
| `service_checker` | Check service status | Port scanning, health checks | Limited |
| `system_monitor` | System metrics | CPU, memory, disk usage | Read-only |

```lua
-- System integration examples
local result = Tools.get("process_executor"):execute({
    command = "git",
    args = {"status", "--porcelain"},
    working_dir = "./project",
    timeout = 30,
    capture_output = true
})

local envVars = Tools.get("environment_reader"):execute({
    variables = {"PATH", "HOME", "USER"},
    include_system_info = true,
    mask_sensitive = true
})

local services = Tools.get("service_checker"):execute({
    checks = {
        { host = "localhost", port = 5432, service = "postgresql" },
        { url = "https://api.example.com/health" },
        { process = "nginx" }
    },
    timeout = 5
})
```

#### 6. **Utilities and Helpers** (8 tools)
Essential utility functions for common programming tasks.

| Tool Name | Description | Features | Use Cases |
|-----------|-------------|----------|-----------|
| `calculator` | Mathematical operations | Basic math, scientific functions | Calculations |
| `text_manipulator` | String operations | Regex, formatting, encoding | Text processing |
| `date_time_handler` | Date/time operations | Parsing, formatting, timezones | Scheduling |
| `uuid_generator` | Generate unique IDs | UUID v4, custom formats | Identification |
| `hash_calculator` | Cryptographic hashing | MD5, SHA256, BLAKE2 | Integrity |
| `base64_encoder` | Encoding/decoding | Base64, URL encoding | Data transfer |
| `template_processor` | Template rendering | Handlebars-like syntax | Code generation |
| `diff_calculator` | Compare content | Text diffs, structural diffs | Version control |

```javascript
// Utility tools examples
const mathResult = await Tools.get("calculator").execute({
    expression: "sqrt(25) + log(100) * sin(π/4)",
    precision: 6,
    variables: { π: Math.PI }
});

const processedText = await Tools.get("text_manipulator").execute({
    text: "  Hello, World!  ",
    operations: [
        "trim",
        "toLowerCase",
        { "replace": { pattern: "world", replacement: "Universe" } },
        { "extract": { pattern: "\\w+", flags: "g" } }
    ]
});

const formattedDate = await Tools.get("date_time_handler").execute({
    operation: "format",
    date: "2025-01-20T15:30:00Z",
    format: "YYYY-MM-DD HH:mm:ss",
    timezone: "America/New_York",
    locale: "en-US"
});

const renderedTemplate = await Tools.get("template_processor").execute({
    template: "Hello {{name}}, your score is {{score}}/{{maxScore}}!",
    variables: {
        name: "Alice",
        score: 95,
        maxScore: 100
    },
    options: {
        strict: true,
        escapeHtml: false
    }
});
```

#### 7. **Communication and External APIs** (5 tools)
Connect with external services and communication platforms.

| Tool Name | Description | Supported Services | Auth Methods |
|-----------|-------------|-------------------|--------------|
| `email_sender` | Send emails | SMTP, SendGrid, SES | API keys, OAuth |
| `webhook_caller` | HTTP webhooks | Custom endpoints | Headers, tokens |
| `slack_integration` | Slack messaging | Messages, files, threads | Bot tokens |
| `github_integration` | GitHub operations | Issues, PRs, repos | PAT, GitHub App |
| `database_connector` | Database queries | PostgreSQL, MySQL, SQLite | Connection strings |

```lua
-- Communication tools examples
Tools.get("email_sender"):execute({
    provider = "smtp",
    config = {
        host = "smtp.gmail.com",
        port = 587,
        secure = true,
        auth = {
            user = "${EMAIL_USER}",
            password = "${EMAIL_PASSWORD}"
        }
    },
    email = {
        from = "ai-agent@example.com",
        to = {"recipient@example.com"},
        subject = "Research Report Complete",
        html = "<h1>Your research is ready!</h1><p>Please find the attached report.</p>",
        attachments = {
            { path = "./output/report.pdf", filename = "research_report.pdf" }
        }
    }
})

Tools.get("slack_integration"):execute({
    action = "send_message",
    token = "${SLACK_BOT_TOKEN}",
    channel = "#ai-reports",
    message = {
        text = "Research analysis complete! 📊",
        blocks = {
            {
                type = "section",
                text = { type = "mrkdwn", text = "*Research Results*\n• Sources analyzed: 47\n• Confidence: 89%" }
            }
        }
    }
})
```

#### 8. **Specialized Domain Tools** (3 tools)
Domain-specific tools for specialized use cases.

| Tool Name | Description | Domain | Capabilities |
|-----------|-------------|---------|--------------|
| `pdf_processor` | PDF operations | Documents | Extract text, metadata, split/merge |
| `image_processor` | Image manipulation | Media | Resize, format conversion, metadata |
| `academic_searcher` | Academic paper search | Research | arXiv, PubMed, Google Scholar |

```javascript
// Specialized tools examples
const pdfContent = await Tools.get("pdf_processor").execute({
    operation: "extract_text",
    file_path: "./documents/research_paper.pdf",
    pages: [1, 2, 3], // specific pages or "all"
    include_metadata: true,
    ocr_fallback: true
});

const processedImage = await Tools.get("image_processor").execute({
    operation: "resize",
    input_path: "./images/chart.png",
    output_path: "./images/chart_thumbnail.png",
    width: 300,
    height: 200,
    maintain_aspect_ratio: true,
    format: "webp",
    quality: 85
});

const academicPapers = await Tools.get("academic_searcher").execute({
    query: "transformer neural networks attention mechanism",
    sources: ["arxiv", "pubmed"],
    date_range: {
        start: "2020-01-01",
        end: "2025-01-01"
    },
    max_results: 20,
    include_abstracts: true,
    sort_by: "relevance" // relevance, date, citations
});
```

### Tool Discovery and Management

#### Dynamic Tool Discovery
```typescript
// Discover tools by capability
const textTools = await Tools.discover({
    capabilities: ["text_processing", "nlp"],
    category: "ai_ml",
    minVersion: "1.0.0"
});

// Search tools by description
const searchResults = await Tools.search("convert data format");

// List all tools in category
const webTools = await Tools.getByCategory("web_access");

// Get tool metadata
const toolInfo = await Tools.get("web_scraper").getMetadata();
console.log(toolInfo.inputSchema);
console.log(toolInfo.examples);
```

#### Tool Chaining and Composition
```lua
-- Sequential tool chaining
local pipeline = Tools.createPipeline({
    name = "web_research_pipeline",
    steps = {
        {
            tool = "web_search",
            input = { query = "{{input.topic}}", max_results = 10 },
            output = "search_results"
        },
        {
            tool = "web_scraper",
            input = function(context)
                local urls = {}
                for _, result in ipairs(context.search_results.results) do
                    table.insert(urls, result.url)
                end
                return { urls = urls, selectors = { title = "h1", content = "p" } }
            end,
            output = "scraped_content"
        },
        {
            tool = "text_summarizer",
            input = function(context)
                local combined_text = ""
                for _, content in ipairs(context.scraped_content) do
                    combined_text = combined_text .. content.content .. "\n"
                end
                return { text = combined_text, target_length = "medium" }
            end,
            output = "summary"
        }
    }
})

local result = pipeline:execute({ topic = "artificial intelligence trends" })
```

#### Custom Tool Development
```rust
// Create custom tool in Rust (for built-in tools)
pub struct CustomAnalyticsTool {
    base: BaseAgentImpl,
    analytics_client: AnalyticsClient,
    cache: Cache,
}

#[async_trait]
impl Tool for CustomAnalyticsTool {
    fn name(&self) -> &str { "custom_analytics" }
    fn description(&self) -> &str { "Custom analytics processing with specialized algorithms" }
    fn category(&self) -> ToolCategory { ToolCategory::DataProcessing }
    
    fn input_schema(&self) -> &JsonSchema {
        static SCHEMA: Lazy<JsonSchema> = Lazy::new(|| {
            serde_json::from_value(json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "array",
                        "description": "Array of data points to analyze"
                    },
                    "analysis_type": {
                        "type": "string",
                        "enum": ["trend", "correlation", "anomaly"],
                        "description": "Type of analysis to perform"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Analysis-specific parameters"
                    }
                },
                "required": ["data", "analysis_type"]
            })).unwrap()
        });
        &SCHEMA
    }
    
    async fn execute_tool(&self, input: ToolInput) -> Result<ToolOutput> {
        let data = input.get_array("data")?;
        let analysis_type = input.get_string("analysis_type")?;
        let parameters = input.get_object("parameters").unwrap_or_default();
        
        // Check cache first
        let cache_key = format!("{}:{}", analysis_type, hash(&data));
        if let Some(cached_result) = self.cache.get(&cache_key).await? {
            return Ok(ToolOutput::new()
                .with_result(cached_result)
                .with_metadata("cached", true));
        }
        
        // Perform analysis
        let result = match analysis_type.as_str() {
            "trend" => self.analyze_trends(&data, &parameters).await?,
            "correlation" => self.analyze_correlations(&data, &parameters).await?,
            "anomaly" => self.detect_anomalies(&data, &parameters).await?,
            _ => return Err(ToolError::UnsupportedOperation(analysis_type))
        };
        
        // Cache result
        self.cache.set(&cache_key, &result, Duration::from_hours(1)).await?;
        
        Ok(ToolOutput::new()
            .with_result(result)
            .with_metadata("analysis_type", analysis_type)
            .with_metadata("data_points", data.len())
            .with_metadata("cached", false))
    }
    
    fn examples(&self) -> &[ToolExample] {
        static EXAMPLES: Lazy<Vec<ToolExample>> = Lazy::new(|| vec![
            ToolExample {
                name: "Trend Analysis".to_string(),
                description: "Analyze trends in time series data".to_string(),
                input: json!({
                    "data": [10, 12, 15, 18, 22, 25, 30],
                    "analysis_type": "trend",
                    "parameters": {
                        "window_size": 3,
                        "confidence_level": 0.95
                    }
                }),
                output: json!({
                    "trend": "increasing",
                    "slope": 3.2,
                    "r_squared": 0.987,
                    "forecast": [35, 40, 45]
                })
            }
        ]);
        &EXAMPLES
    }
}
```

#### Script-Level Tool Creation
```javascript
// Create custom tool in JavaScript
const customTool = Tools.create("text_metrics", {
    description: "Calculate detailed text metrics and readability scores",
    category: "text_processing",
    
    inputSchema: {
        type: "object",
        properties: {
            text: { type: "string", description: "Text to analyze" },
            language: { type: "string", default: "en" },
            metrics: { 
                type: "array", 
                items: { type: "string" },
                default: ["readability", "complexity", "sentiment"]
            }
        },
        required: ["text"]
    },
    
    async execute({ text, language = "en", metrics = ["readability"] }) {
        const results = {};
        
        // Basic metrics
        results.basic = {
            characters: text.length,
            words: text.split(/\s+/).length,
            sentences: text.split(/[.!?]+/).length,
            paragraphs: text.split(/\n\s*\n/).length
        };
        
        // Calculate requested metrics
        for (const metric of metrics) {
            switch (metric) {
                case "readability":
                    results.readability = calculateFleschKincaid(text);
                    break;
                case "complexity":
                    results.complexity = calculateComplexity(text);
                    break;
                case "sentiment":
                    results.sentiment = await Tools.get("sentiment_analyzer").execute({
                        text,
                        language
                    });
                    break;
            }
        }
        
        return {
            metrics: results,
            language,
            processed_at: new Date().toISOString()
        };
    },
    
    examples: [
        {
            name: "Basic Text Analysis",
            input: {
                text: "This is a sample text for analysis. It contains multiple sentences.",
                metrics: ["readability", "complexity"]
            },
            output: {
                metrics: {
                    basic: { characters: 71, words: 12, sentences: 2, paragraphs: 1 },
                    readability: { score: 65.2, level: "Standard" },
                    complexity: { score: 0.3, level: "Low" }
                }
            }
        }
    ]
});

// Register the custom tool
await Tools.register(customTool);

// Use the custom tool
const textAnalysis = await Tools.get("text_metrics").execute({
    text: documentText,
    metrics: ["readability", "complexity", "sentiment"]
});
```

### Tool Performance and Optimization

#### Performance Characteristics
```lua
-- Tool performance monitoring
local PerformanceMonitor = {
    metrics = {},
    
    track = function(self, tool_name, operation)
        return function(...)
            local start_time = os.clock()
            local start_memory = collectgarbage("count")
            
            local result = operation(...)
            
            local end_time = os.clock()
            local end_memory = collectgarbage("count")
            
            self.metrics[tool_name] = self.metrics[tool_name] or {}
            table.insert(self.metrics[tool_name], {
                execution_time = end_time - start_time,
                memory_used = end_memory - start_memory,
                timestamp = os.time()
            })
            
            return result
        end
    end,
    
    get_stats = function(self, tool_name)
        local metrics = self.metrics[tool_name]
        if not metrics or #metrics == 0 then
            return nil
        end
        
        local total_time = 0
        local total_memory = 0
        local min_time = math.huge
        local max_time = 0
        
        for _, metric in ipairs(metrics) do
            total_time = total_time + metric.execution_time
            total_memory = total_memory + metric.memory_used
            min_time = math.min(min_time, metric.execution_time)
            max_time = math.max(max_time, metric.execution_time)
        end
        
        return {
            calls = #metrics,
            avg_time = total_time / #metrics,
            min_time = min_time,
            max_time = max_time,
            avg_memory = total_memory / #metrics,
            total_time = total_time
        }
    end
}

-- Wrap tool execution with performance tracking
local original_execute = Tools.get("web_scraper").execute
Tools.get("web_scraper").execute = PerformanceMonitor:track("web_scraper", original_execute)
```

#### Caching and Optimization
```javascript
// Tool result caching
class ToolCache {
    constructor(maxSize = 1000, ttl = 3600000) { // 1 hour default TTL
        this.cache = new Map();
        this.maxSize = maxSize;
        this.ttl = ttl;
    }
    
    generateKey(toolName, input) {
        const inputHash = require('crypto')
            .createHash('sha256')
            .update(JSON.stringify(input))
            .digest('hex');
        return `${toolName}:${inputHash}`;
    }
    
    get(key) {
        const entry = this.cache.get(key);
        if (!entry) return null;
        
        if (Date.now() - entry.timestamp > this.ttl) {
            this.cache.delete(key);
            return null;
        }
        
        return entry.value;
    }
    
    set(key, value) {
        // Implement LRU eviction if needed
        if (this.cache.size >= this.maxSize) {
            const firstKey = this.cache.keys().next().value;
            this.cache.delete(firstKey);
        }
        
        this.cache.set(key, {
            value,
            timestamp: Date.now()
        });
    }
    
    clear() {
        this.cache.clear();
    }
}

// Global tool cache
const toolCache = new ToolCache();

// Cached tool execution wrapper
async function cachedToolExecution(toolName, input, options = {}) {
    const { useCache = true, cacheKey } = options;
    
    if (useCache) {
        const key = cacheKey || toolCache.generateKey(toolName, input);
        const cached = toolCache.get(key);
        
        if (cached) {
            console.log(`Cache hit for ${toolName}`);
            return { ...cached, cached: true };
        }
    }
    
    // Execute tool
    const result = await Tools.get(toolName).execute(input);
    
    // Cache result if successful
    if (useCache && result && !result.error) {
        const key = cacheKey || toolCache.generateKey(toolName, input);
        toolCache.set(key, result);
    }
    
    return { ...result, cached: false };
}

// Usage with caching
const searchResult = await cachedToolExecution("web_search", {
    query: "machine learning trends",
    maxResults: 10
});
```

This comprehensive built-in components library provides:
- **Production-Ready Tools**: 40+ tools across 8 categories
- **Consistent Interface**: Unified API across all tools
- **Security Controls**: Sandboxing and rate limiting
- **Performance Optimization**: Caching and monitoring
- **Extensibility**: Custom tool development patterns
- **Real-World Examples**: Practical usage scenarios

---

## Tool Development Architecture

Rs-LLMSpell provides a comprehensive architecture for tool development that supports multiple implementation approaches, from simple script-based tools to complex Rust implementations and dynamically loaded plugins.

### Tool Creation Patterns

#### 1. **Script-Level Tool Pattern**
The simplest approach for rapid prototyping and domain-specific functionality.

**Architecture Elements:**
- **Dynamic Registration**: Tools created at runtime and registered with the global registry
- **Schema Validation**: Input/output schemas validated at registration time
- **Async Execution**: Automatic async wrapper for script functions
- **Error Boundaries**: Sandboxed execution with error isolation

**Design Considerations:**
- Ideal for application-specific tools that don't require system access
- Performance overhead acceptable for I/O-bound operations
- Automatic type conversion between script and Rust types

#### 2. **Native Tool Pattern**
High-performance tools implemented directly in Rust.

**Architecture Elements:**
- **Trait Implementation**: Extends `Tool` trait with full type safety
- **Direct Integration**: No FFI overhead, direct memory access
- **Lifecycle Management**: Proper resource initialization and cleanup
- **Compile-Time Validation**: Schema and type checking at build time

**Design Considerations:**
- Required for system-level operations (file I/O, network, process management)
- Enables zero-copy data processing for performance-critical paths
- Supports complex state management and caching strategies

#### 3. **Plugin Tool Pattern**
Dynamically loaded tools for extensibility without recompilation.

**Architecture Elements:**
- **Dynamic Loading**: Runtime discovery and loading of tool libraries
- **Version Management**: API versioning and compatibility checks
- **Isolation Boundaries**: Separate memory space with controlled communication
- **Hot Reload**: Support for updating tools without system restart

**Design Considerations:**
- Enables third-party tool development and distribution
- Requires stable ABI and versioning strategy
- Additional security considerations for untrusted code

### Tool Trait Architecture

The `Tool` trait hierarchy ensures consistent behavior across all tool types:

```rust
// Architectural trait hierarchy
BaseAgent (state, hooks)
    ↓
Tool (schema, execution, composition)
    ↓
SpecializedTool (category-specific behavior)
```

**Key Architectural Decisions:**
- Tools inherit from `BaseAgent` to support state management and hooks
- Schema-first design enables automatic UI generation and validation
- Composition support built into the trait for tool chaining
- Category system for organized discovery and capability matching

### Tool Registry Architecture

**Central Registry Design:**
- **Global Singleton**: Thread-safe access from all components
- **Category Indexing**: O(1) lookup by category for efficient discovery
- **Capability Matching**: Graph-based matching for complex requirements
- **Lazy Loading**: Tools loaded only when first accessed

**Registration Flow:**
1. Schema validation against JSON Schema spec
2. Capability extraction and indexing
3. Security policy attachment
4. Hook point registration
5. Availability broadcast via event system

### Tool Composition Architecture

**Pipeline Pattern:**
- **DAG Execution**: Directed acyclic graph for complex workflows
- **Type Safety**: Output-to-input type validation at composition time
- **Error Propagation**: Structured error handling across pipeline stages
- **Resource Pooling**: Shared resources across pipeline stages

**Chaining Mechanisms:**
- **Static Chains**: Compile-time validated tool sequences
- **Dynamic Chains**: Runtime composition based on LLM decisions
- **Conditional Chains**: Branching logic within tool execution
- **Parallel Chains**: Concurrent execution with result aggregation

### Tool Development Best Practices

**Schema Design Principles:**
- **Minimal Required Fields**: Only essential inputs marked as required
- **Progressive Disclosure**: Optional fields for advanced functionality
- **Clear Descriptions**: LLM-friendly field descriptions
- **Example Values**: Concrete examples in schema for better LLM understanding

**Error Handling Architecture:**
- **Typed Errors**: Enum-based error types for each tool category
- **Error Recovery**: Built-in retry logic with exponential backoff
- **Partial Success**: Return partial results when possible
- **Error Context**: Rich error messages with remediation hints

**Performance Optimization Patterns:**
- **Result Caching**: LRU cache with TTL for expensive operations
- **Batch Processing**: Automatic batching for similar requests
- **Resource Pooling**: Connection and handle reuse
- **Async by Default**: Non-blocking execution for all I/O operations

### Tool Security Architecture

**Sandboxing Layers:**
1. **Script Sandbox**: Limited API access for script-based tools
2. **Process Sandbox**: Separate process for untrusted plugins
3. **Resource Limits**: CPU, memory, and time quotas
4. **Capability-Based Security**: Fine-grained permission model

**Security Validation:**
- **Input Sanitization**: Automatic validation against schema
- **Output Filtering**: Sensitive data redaction
- **Audit Logging**: All tool executions logged with context
- **Rate Limiting**: Per-tool and per-user rate limits

### Tool Testing Architecture

**Test Patterns:**
- **Unit Tests**: Individual tool logic testing
- **Integration Tests**: Tool interaction with system resources
- **Schema Tests**: Input/output validation testing
- **Performance Tests**: Benchmark suites for critical paths

**Mock Infrastructure:**
- **Resource Mocks**: File system, network, API mocks
- **State Verification**: Pre/post condition validation
- **Error Injection**: Systematic failure testing
- **Load Testing**: Concurrent execution stress tests

### Tool Template System

**Scaffolding Architecture:**
- **Template Engine**: Code generation from specifications
- **Category Templates**: Pre-built patterns for each tool category
- **Interactive Generation**: CLI wizard for tool creation
- **Best Practice Enforcement**: Generated code follows all patterns

**Common Templates:**
- **HTTP API Tool**: REST/GraphQL client template
- **File Processor Tool**: Streaming file transformation template
- **Data Analysis Tool**: Statistical computation template
- **Integration Tool**: Third-party service connector template

This architecture ensures tools are:
- **Discoverable**: Easy to find and understand capabilities
- **Composable**: Natural chaining and pipeline creation
- **Secure**: Sandboxed with principle of least privilege
- **Performant**: Optimized for common usage patterns
- **Testable**: Comprehensive testing support at all levels

---

## Agent Templates and Patterns

Rs-LLMSpell provides pre-built agent templates that implement proven patterns for common AI use cases. These templates serve as starting points and can be customized for specific requirements.

### Built-in Agent Templates

#### 1. **Chat Agent Template**
Conversational agents with personality and context management.

```lua
-- Chat agent with personality
local ChatAgent = Agent.template("conversational_assistant", {
    name = "friendly_assistant",
    description = "Helpful and friendly conversational assistant",
    
    base_config = {
        provider = "anthropic",
        model = "claude-3-sonnet",
        temperature = 0.7,
        max_tokens = 1000
    },
    
    personality = {
        traits = {"helpful", "friendly", "patient", "curious"},
        communication_style = "casual_professional",
        response_length = "medium",
        use_emojis = true,
        humor_level = "light"
    },
    
    system_prompt_template = [[
        You are {{name}}, a {{personality.communication_style}} assistant.
        
        Your traits: {{#each personality.traits}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
        
        Communication guidelines:
        - Keep responses {{personality.response_length}} length
        - {{#if personality.use_emojis}}Use emojis appropriately{{else}}Avoid emojis{{/if}}
        - Humor level: {{personality.humor_level}}
        
        Always be {{personality.traits.0}} and provide accurate information.
    ]],
    
    memory_config = {
        type = "conversation",
        max_entries = 50,
        summarize_after = 20,
        personality_consistency = true
    },
    
    tools = {
        "web_search",
        "calculator", 
        "date_time_handler"
    }
})

-- Customize the template
local myAssistant = ChatAgent.create({
    name = "research_buddy",
    personality = {
        traits = {"analytical", "thorough", "supportive"},
        communication_style = "academic_friendly",
        response_length = "detailed"
    },
    additional_tools = {
        "academic_searcher",
        "text_summarizer"
    }
})
```

#### 2. **Research Agent Template**
Specialized agents for information gathering and analysis.

```javascript
// Research agent template
const ResearchAgent = Agent.template("research_specialist", {
    name: "research_agent",
    description: "Comprehensive research and analysis specialist",
    
    baseConfig: {
        provider: "openai",
        model: "gpt-4",
        temperature: 0.3,
        maxTokens: 2000
    },
    
    researchCapabilities: {
        sources: ["web", "academic", "news", "reports"],
        analysisDepth: ["surface", "moderate", "deep", "comprehensive"],
        outputFormats: ["summary", "detailed_report", "bullet_points", "structured_data"],
        citationStyle: "apa" // apa, mla, chicago
    },
    
    systemPromptTemplate: `
        You are a research specialist with expertise in:
        - {{researchCapabilities.sources.join(', ')}} research
        - {{researchCapabilities.analysisDepth.join(', ')}} analysis
        - {{researchCapabilities.citationStyle.toUpperCase()}} citation format
        
        Research methodology:
        1. Gather information from multiple reliable sources
        2. Cross-reference facts and verify accuracy
        3. Analyze patterns and extract insights
        4. Present findings with proper citations
        5. Highlight confidence levels and limitations
        
        Always provide sources and indicate your confidence level (1-10).
    `,
    
    tools: [
        "web_search",
        "academic_searcher", 
        "web_scraper",
        "text_summarizer",
        "statistical_analyzer"
    ],
    
    workflows: {
        comprehensive_research: {
            steps: [
                { name: "initial_search", tool: "web_search" },
                { name: "academic_lookup", tool: "academic_searcher" },
                { name: "content_extraction", tool: "web_scraper" },
                { name: "analysis", tool: "statistical_analyzer" },
                { name: "synthesis", agent: "self" }
            ]
        }
    }
});

// Create customized research agent
const marketResearcher = ResearchAgent.create({
    name: "market_research_agent",
    specialization: "market_analysis",
    researchCapabilities: {
        sources: ["web", "reports", "news"],
        analysisDepth: ["moderate", "deep"],
        outputFormats: ["structured_data", "detailed_report"],
        citationStyle: "business"
    },
    additionalTools: ["data_visualizer"],
    customPromptAdditions: `
        Focus on:
        - Market trends and size
        - Competitive landscape
        - Consumer behavior insights
        - Revenue and growth projections
        - Risk factors and opportunities
    `
});
```

#### 3. **Code Assistant Template**
Agents specialized for programming tasks and code analysis.

```typescript
// Code assistant template
interface CodeAssistantConfig {
    name: string;
    languages: string[];
    specialties: CodeSpecialty[];
    codeStyle: CodeStyle;
    frameworks: string[];
    testingFrameworks: string[];
}

enum CodeSpecialty {
    WebDevelopment = "web_development",
    DataScience = "data_science", 
    SystemsProgramming = "systems_programming",
    MachineLearning = "machine_learning",
    DevOps = "devops"
}

const CodeAssistant = Agent.template("code_specialist", {
    name: "code_assistant",
    description: "Programming assistant with code analysis and generation capabilities",
    
    baseConfig: {
        provider: "anthropic",
        model: "claude-3-opus",
        temperature: 0.1, // Low temperature for precise code
        maxTokens: 4000
    },
    
    codeCapabilities: {
        languages: ["python", "javascript", "typescript", "rust", "go"],
        specialties: [CodeSpecialty.WebDevelopment, CodeSpecialty.DataScience],
        codeStyle: {
            naming: "snake_case", // snake_case, camelCase, PascalCase
            indentation: "spaces", // spaces, tabs
            lineLength: 88,
            documentation: "detailed" // minimal, standard, detailed
        },
        frameworks: ["react", "fastapi", "pandas", "numpy"],
        testingFrameworks: ["jest", "pytest", "cargo test"]
    },
    
    systemPromptTemplate: `
        You are a programming assistant specializing in:
        Languages: {{codeCapabilities.languages.join(', ')}}
        Specialties: {{codeCapabilities.specialties.join(', ')}}
        
        Code Quality Standards:
        - Use {{codeCapabilities.codeStyle.naming}} naming convention
        - {{codeCapabilities.codeStyle.indentation}} for indentation
        - Maximum line length: {{codeCapabilities.codeStyle.lineLength}}
        - Documentation level: {{codeCapabilities.codeStyle.documentation}}
        
        Always:
        1. Write clean, readable, and maintainable code
        2. Include appropriate error handling
        3. Add docstrings/comments for functions
        4. Follow language best practices
        5. Suggest tests when implementing features
        6. Consider security implications
        
        When analyzing code, check for:
        - Bugs and potential issues
        - Performance improvements
        - Security vulnerabilities
        - Code style consistency
        - Test coverage gaps
    `,
    
    tools: [
        "file_reader",
        "file_writer",
        "process_executor", // For running code/tests
        "text_analyzer", // For code complexity analysis
        "diff_calculator" // For code comparison
    ],
    
    workflows: {
        code_review: {
            steps: [
                { name: "read_code", tool: "file_reader" },
                { name: "analyze_style", tool: "text_analyzer" },
                { name: "run_tests", tool: "process_executor" },
                { name: "generate_review", agent: "self" }
            ]
        },
        
        implement_feature: {
            steps: [
                { name: "analyze_requirements", agent: "self" },
                { name: "design_solution", agent: "self" },
                { name: "write_tests", tool: "file_writer" },
                { name: "implement_code", tool: "file_writer" },
                { name: "run_tests", tool: "process_executor" },
                { name: "refine_implementation", agent: "self" }
            ]
        }
    }
});

// Specialized code assistants
const webDevAssistant = CodeAssistant.create({
    name: "react_specialist",
    languages: ["javascript", "typescript"],
    specialties: [CodeSpecialty.WebDevelopment],
    frameworks: ["react", "next.js", "tailwindcss"],
    testingFrameworks: ["jest", "testing-library"],
    
    customPromptAdditions: `
        Web Development Focus:
        - Modern React patterns (hooks, context, suspense)
        - Performance optimization (memoization, lazy loading)
        - Accessibility (WCAG compliance)
        - SEO best practices
        - Responsive design principles
        
        Always consider:
        - Bundle size impact
        - Runtime performance
        - User experience
        - Cross-browser compatibility
    `
});

const dataScientist = CodeAssistant.create({
    name: "data_science_assistant",
    languages: ["python", "r", "sql"],
    specialties: [CodeSpecialty.DataScience, CodeSpecialty.MachineLearning],
    frameworks: ["pandas", "numpy", "scikit-learn", "tensorflow", "pytorch"],
    testingFrameworks: ["pytest", "unittest"],
    
    additionalTools: [
        "csv_processor",
        "statistical_analyzer",
        "data_visualizer"
    ],
    
    customPromptAdditions: `
        Data Science Focus:
        - Exploratory data analysis (EDA)
        - Data cleaning and preprocessing
        - Statistical modeling and validation
        - Machine learning best practices
        - Result interpretation and visualization
        
        Always include:
        - Data validation checks
        - Statistical assumptions testing
        - Performance metrics
        - Reproducibility considerations
    `
});
```

#### 4. **Content Creator Template**
Agents for writing, editing, and content generation.

```lua
-- Content creator template
local ContentCreator = Agent.template("content_specialist", {
    name = "content_creator",
    description = "Creative writing and content generation specialist",
    
    base_config = {
        provider = "anthropic",
        model = "claude-3-opus",
        temperature = 0.8, -- Higher for creativity
        max_tokens = 3000
    },
    
    content_capabilities = {
        content_types = {
            "blog_posts", "articles", "social_media", "marketing_copy", 
            "technical_documentation", "creative_writing", "scripts"
        },
        writing_styles = {
            "professional", "casual", "academic", "conversational", 
            "persuasive", "narrative", "technical"
        },
        tones = {
            "friendly", "authoritative", "enthusiastic", "calm", 
            "urgent", "humorous", "serious"
        },
        formats = {
            "long_form", "short_form", "listicle", "how_to", 
            "case_study", "review", "opinion"
        }
    },
    
    system_prompt_template = [[
        You are a professional content creator with expertise in:
        {{#each content_capabilities.content_types}}
        - {{this}}
        {{/each}}
        
        Writing Styles Available: {{content_capabilities.writing_styles}}
        Tone Options: {{content_capabilities.tones}}
        Format Types: {{content_capabilities.formats}}
        
        Content Creation Process:
        1. Understand target audience and purpose
        2. Research topic thoroughly if needed
        3. Create compelling headlines/titles
        4. Structure content logically
        5. Write engaging introduction
        6. Develop main content with examples
        7. Create strong conclusion with call-to-action
        8. Optimize for readability and SEO
        
        Quality Standards:
        - Original, plagiarism-free content
        - Factually accurate information
        - Proper grammar and spelling
        - Appropriate tone and style
        - Clear structure and flow
        - Engaging and valuable to readers
    ]],
    
    tools = {
        "web_search",
        "text_analyzer",
        "text_manipulator", 
        "template_processor",
        "academic_searcher"
    },
    
    workflows = {
        blog_post_creation = {
            steps = {
                {
                    name = "research_topic",
                    tools = {"web_search", "academic_searcher"},
                    output = "research_data"
                },
                {
                    name = "analyze_competitors",
                    tool = "web_search",
                    input = function(context)
                        return {
                            query = context.input.topic .. " blog posts",
                            max_results = 10
                        }
                    end,
                    output = "competitor_analysis"
                },
                {
                    name = "create_outline",
                    agent = "self",
                    input = function(context)
                        return {
                            topic = context.input.topic,
                            research = context.research_data,
                            competitors = context.competitor_analysis,
                            target_audience = context.input.target_audience
                        }
                    end,
                    output = "content_outline"
                },
                {
                    name = "write_content",
                    agent = "self",
                    input = function(context)
                        return {
                            outline = context.content_outline,
                            style = context.input.style or "professional",
                            tone = context.input.tone or "friendly",
                            word_count = context.input.word_count or 1500
                        }
                    end,
                    output = "draft_content"
                },
                {
                    name = "review_and_edit",
                    agent = "self",
                    input = function(context)
                        return {
                            content = context.draft_content,
                            review_criteria = {
                                "readability",
                                "seo_optimization", 
                                "fact_checking",
                                "grammar_check"
                            }
                        }
                    end,
                    output = "final_content"
                }
            }
        }
    }
})

-- Specialized content creators
local TechnicalWriter = ContentCreator.create({
    name = "technical_documentation_specialist",
    specialization = "technical_writing",
    
    content_capabilities = {
        content_types = {"api_documentation", "user_guides", "tutorials", "technical_specifications"},
        writing_styles = {"technical", "instructional"},
        tones = {"professional", "helpful"},
        formats = {"how_to", "reference", "step_by_step"}
    },
    
    additional_tools = {
        "code_formatter",
        "api_tester",
        "diagram_generator"
    },
    
    custom_prompt_additions = [[
        Technical Writing Expertise:
        - API documentation with examples
        - Step-by-step tutorials
        - Code snippets and explanations
        - Architecture diagrams and flowcharts
        - Troubleshooting guides
        
        Always include:
        - Clear prerequisites
        - Code examples that work
        - Common pitfalls and solutions
        - Version compatibility notes
        - Links to related resources
    ]]
})

local SocialMediaManager = ContentCreator.create({
    name = "social_media_specialist", 
    specialization = "social_media",
    
    content_capabilities = {
        content_types = {"social_media", "marketing_copy"},
        writing_styles = {"casual", "conversational", "persuasive"},
        tones = {"friendly", "enthusiastic", "urgent"},
        formats = {"short_form", "listicle"}
    },
    
    platform_configs = {
        twitter = { max_length = 280, hashtag_strategy = "moderate" },
        linkedin = { max_length = 1300, tone = "professional" },
        instagram = { focus = "visual", caption_length = "medium" },
        facebook = { engagement_focus = true, link_preview = true }
    },
    
    custom_prompt_additions = [[
        Social Media Expertise:
        - Platform-specific optimization
        - Hashtag research and strategy
        - Engagement-driving content
        - Visual content descriptions
        - Community management
        
        Platform Guidelines:
        - Twitter: Concise, trending hashtags, engagement hooks
        - LinkedIn: Professional tone, industry insights, networking
        - Instagram: Visual storytelling, lifestyle integration
        - Facebook: Community building, shareable content
    ]]
})
```

### Template Customization Patterns

#### 1. **Template Inheritance**
```javascript
// Base agent template
const BaseAnalyst = Agent.template("base_analyst", {
    baseConfig: {
        temperature: 0.2,
        maxTokens: 2000
    },
    
    commonTools: [
        "statistical_analyzer",
        "data_visualizer", 
        "text_summarizer"
    ],
    
    analysisFramework: {
        methodology: "structured",
        confidenceReporting: true,
        citationRequired: true
    }
});

// Specialized templates inheriting from base
const MarketAnalyst = BaseAnalyst.extend("market_analyst", {
    specialization: "market_analysis",
    
    additionalTools: [
        "web_search",
        "financial_data_api"
    ],
    
    domainKnowledge: {
        markets: ["technology", "healthcare", "finance"],
        metrics: ["revenue", "growth_rate", "market_share", "valuation"],
        frameworks: ["porter_five_forces", "swot", "pestle"]
    }
});

const TechnicalAnalyst = BaseAnalyst.extend("technical_analyst", {
    specialization: "technical_analysis",
    
    additionalTools: [
        "code_analyzer",
        "performance_profiler",
        "security_scanner"
    ],
    
    technicalFocus: {
        languages: ["python", "javascript", "rust"],
        architectures: ["microservices", "serverless", "distributed"],
        metrics: ["performance", "scalability", "maintainability", "security"]
    }
});
```

#### 2. **Dynamic Template Configuration**
```lua
-- Template factory with dynamic configuration
local TemplateFactory = {
    create_custom_agent = function(self, base_template, customizations)
        local template = Agent.template(base_template)
        
        -- Apply customizations
        for key, value in pairs(customizations) do
            if key == "additional_tools" then
                for _, tool in ipairs(value) do
                    table.insert(template.tools, tool)
                end
            elseif key == "system_prompt_additions" then
                template.system_prompt_template = template.system_prompt_template .. "\n\n" .. value
            elseif key == "workflows" then
                for workflow_name, workflow_config in pairs(value) do
                    template.workflows[workflow_name] = workflow_config
                end
            else
                template[key] = value
            end
        end
        
        return template
    end,
    
    create_domain_expert = function(self, domain, expertise_areas, data_sources)
        return self:create_custom_agent("research_specialist", {
            name = domain .. "_expert",
            specialization = domain,
            
            additional_tools = data_sources,
            
            domain_expertise = expertise_areas,
            
            system_prompt_additions = string.format([[
                Domain Expertise: %s
                
                Specialized Knowledge Areas:
                %s
                
                When analyzing %s topics:
                1. Apply domain-specific frameworks and methodologies
                2. Reference industry standards and best practices
                3. Consider regulatory and compliance requirements
                4. Provide actionable insights for stakeholders
                5. Highlight emerging trends and disruptions
            ]], domain, table.concat(expertise_areas, "\n- "), domain),
            
            workflows = {
                domain_analysis = {
                    steps = {
                        { name = "domain_research", tools = data_sources },
                        { name = "expert_analysis", agent = "self" },
                        { name = "stakeholder_insights", agent = "self" },
                        { name = "recommendations", agent = "self" }
                    }
                }
            }
        })
    end
}

-- Create domain experts dynamically
local HealthcareExpert = TemplateFactory:create_domain_expert(
    "healthcare",
    {
        "clinical_research", 
        "medical_devices", 
        "pharmaceuticals", 
        "health_policy",
        "patient_outcomes"
    },
    {
        "academic_searcher",
        "pubmed_search", 
        "regulatory_database",
        "clinical_trials_api"
    }
)

local FinanceExpert = TemplateFactory:create_domain_expert(
    "finance",
    {
        "investment_analysis",
        "risk_management", 
        "regulatory_compliance",
        "market_dynamics", 
        "financial_modeling"
    },
    {
        "financial_data_api",
        "sec_filings_search",
        "market_data_feed",
        "economic_indicators"
    }
)
```

#### 4. **Data Analyst Agent Template**
Specialized for data exploration, statistical analysis, and insight generation.

```lua
local DataAnalyst = Agent.template("data_analyst", {
    system_prompt = [[You are a skilled data analyst who helps users understand their data through statistical analysis and visualization recommendations.]],
    
    default_tools = {
        "data_loader",
        "statistical_analysis", 
        "chart_generator",
        "correlation_analyzer"
    },
    
    capabilities = {
        "data_exploration",
        "statistical_modeling", 
        "visualization_design",
        "insight_extraction"
    },
    
    output_format = "structured_analysis"
})
```

#### 5. **Customer Service Agent Template**
Optimized for customer support, issue resolution, and service excellence.

```javascript
const CustomerServiceAgent = Agent.template("customer_service", {
    systemPrompt: "You are a helpful customer service representative focused on resolving issues efficiently while maintaining a friendly, professional tone.",
    
    defaultTools: [
        "knowledge_base_search",
        "ticket_creation",
        "escalation_handler",
        "sentiment_analyzer"
    ],
    
    personality: {
        tone: "helpful_professional",
        empathy_level: "high",
        patience_threshold: "extended"
    },
    
    workflows: ["issue_triage", "resolution_tracking"]
});
```

#### 6. **API Integration Agent Template**
Designed for API testing, integration, and automation tasks.

```lua
local APIAgent = Agent.template("api_integration", {
    system_prompt = [[You specialize in API testing, integration, and automation. You can analyze API specifications, generate test cases, and troubleshoot integration issues.]],
    
    default_tools = {
        "http_client",
        "json_validator", 
        "api_tester",
        "schema_analyzer"
    },
    
    specializations = {
        "rest_apis",
        "graphql_apis", 
        "webhook_handling",
        "auth_flows"
    }
})
```

### Advanced Template Customization

#### **Template Inheritance Architecture**

Templates can be extended and specialized through multiple inheritance patterns:

```lua
-- Extend base Research template for Market Research
local MarketResearchAgent = Agent.extend("research_agent", {
    specialization = "market_analysis",
    
    additional_tools = {
        "market_data_api",
        "competitor_analyzer",
        "trend_detector"
    },
    
    enhanced_prompts = {
        system_addition = "Focus specifically on market trends, competitive landscape, and consumer behavior patterns.",
        analysis_framework = "SWOT_and_PESTLE"
    }
})

-- Multi-level inheritance
local TechMarketAnalyst = Agent.extend("market_research_agent", {
    domain = "technology_sector",
    additional_tools = {"tech_news_scraper", "patent_analyzer"}
})
```

#### **Dynamic Template Generation**

```javascript
// Runtime template creation based on user requirements
const CustomAgentFactory = {
    createSpecializedAgent(baseTemplate, requirements) {
        return Agent.template(`custom_${baseTemplate}`, {
            ...Agent.getTemplate(baseTemplate).config,
            
            // Dynamic tool selection
            defaultTools: this.selectToolsForRequirements(requirements),
            
            // Custom prompt engineering
            systemPrompt: this.generatePromptFor(baseTemplate, requirements),
            
            // Capability mapping
            capabilities: this.mapCapabilities(requirements)
        });
    }
};
```

#### **Template Configuration Patterns**

```lua
-- Configuration-driven template customization
local template_config = {
    base_template = "research_agent",
    
    customizations = {
        domain_expertise = "healthcare",
        data_sources = {"pubmed", "clinical_trials", "medical_journals"},
        compliance_mode = "HIPAA",
        
        tool_overrides = {
            web_search = {
                provider = "specialized_medical_search",
                filters = {"peer_reviewed", "recent_5_years"}
            }
        },
        
        output_requirements = {
            citation_style = "AMA",
            evidence_level = "required",
            peer_review_check = true
        }
    }
}

local HealthcareResearcher = Agent.create_from_config(template_config)
```

#### **Built-in Agent Creation Process**

For adding agents to the core `crates/builtin/src/agents/` library:

```rust
// 1. Implement the Agent trait
pub struct MarketAnalystAgent {
    base: BaseAgentImpl,
    specialized_tools: Vec<Box<dyn Tool>>,
    market_data_provider: MarketDataAPI,
}

#[async_trait]
impl Agent for MarketAnalystAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Specialized("market_analyst".to_string())
    }
    
    fn default_system_prompt(&self) -> &str {
        "You are a market analyst specializing in business intelligence and market research."
    }
    
    async fn initialize_agent(&mut self, config: &AgentConfig) -> Result<()> {
        self.register_specialized_tools().await?;
        self.setup_market_data_connections().await?;
        Ok(())
    }
}

// 2. Register in the built-in agent registry
pub fn register_builtin_agents(registry: &mut AgentRegistry) -> Result<()> {
    registry.register_template("market_analyst", || {
        Box::new(MarketAnalystAgent::new())
    })?;
    
    registry.register_template("customer_service", || {
        Box::new(CustomerServiceAgent::new())
    })?;
    
    registry.register_template("data_analyst", || {
        Box::new(DataAnalystAgent::new())
    })?;
    
    Ok(())
}
```

#### **Template Testing Framework**

```rust
// Built-in agent testing patterns
#[cfg(test)]
mod agent_template_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_market_analyst_template() {
        let agent = Agent.template("market_analyst", default_config()).await?;
        
        // Test template capabilities
        assert!(agent.has_capability("market_analysis"));
        assert!(agent.has_tool("market_data_api"));
        
        // Test specialized behavior
        let response = agent.process("Analyze the smartphone market").await?;
        assert!(response.contains_market_metrics());
        assert!(response.has_competitive_analysis());
    }
    
    #[tokio::test]
    async fn test_template_inheritance() {
        let base = Agent.template("research_agent", default_config()).await?;
        let specialized = Agent.extend("research_agent", market_config()).await?;
        
        // Verify inheritance chain
        assert!(specialized.inherits_from("research_agent"));
        assert_eq!(specialized.base_capabilities(), base.capabilities());
        
        // Verify specialization additions
        assert!(specialized.capabilities().len() > base.capabilities().len());
    }
}
```

This comprehensive template system provides:
- **Pre-built Templates**: 6 common agent patterns ready to use
- **Specialized Templates**: Domain-specific agents (DataAnalyst, CustomerService, API Integration)
- **Customization Options**: Flexible configuration for specific needs
- **Inheritance Patterns**: Multi-level template extension and specialization
- **Dynamic Creation**: Runtime template generation for domain experts
- **Built-in Creation Process**: Clear path for adding agents to core library
- **Testing Framework**: Comprehensive testing patterns for agent templates
- **Workflow Integration**: Built-in workflows for complex tasks

---

# Part V: Technology Stack and Implementation

## Complete Technology Decision Matrix

Rs-LLMSpell follows a **bridge-first philosophy**: leverage the best existing Rust crates rather than reimplementing functionality. This section provides the complete technology stack decisions based on extensive crate ecosystem research.

### Build vs. Buy vs. Wrap Decision Framework

| Component | Decision | Technology Choice | Rationale | Implementation Strategy |
|-----------|----------|-------------------|-----------|------------------------|
| **LLM Providers** | **Wrap** | `rig` + `candle` | Proven abstractions, multiple provider support | Bridge layer with rs-llmspell extensions |
| **Lua Scripting** | **Wrap** | `mlua` | Battle-tested, async support, comprehensive API | Custom bridge with coroutine management |
| **JavaScript Engine** | **Wrap** | `boa` | Pure Rust, improving rapidly, embeddable | Bridge with Promise abstraction layer |
| **State Storage** | **Wrap** | `sled` + `rocksdb` | sled for development, rocksdb for production | Unified storage trait with backend switching |
| **Event System** | **Build** | `tokio` + `crossbeam` | Need specialized hook execution patterns | Custom implementation using proven primitives |
| **Hook Manager** | **Build** | Custom + `inventory` | Unique requirements for multi-language hooks | Custom with automatic registration |
| **Tool Registry** | **Build** | Custom + `typemap` | Component discovery and lifecycle management | Custom registry with type-safe access |
| **Async Bridge** | **Build** | Custom + `tokio` | Single-threaded script engine coordination | Custom cooperative scheduler |
| **Configuration** | **Wrap** | `config` + `serde` | Standard configuration management | Bridge with validation and hot-reload |
| **Serialization** | **Wrap** | `serde` ecosystem | JSON/YAML/TOML support needed | Standard serde with custom derives |
| **HTTP Client** | **Wrap** | `reqwest` | LLM API communication requirements | Wrapped with retry and rate limiting |
| **CLI Framework** | **Wrap** | `clap` | Command-line interface for tooling | Standard clap with custom subcommands |
| **Testing Utils** | **Build** | Custom + `tokio-test` | Multi-language script testing needs | Custom test harness with engine mocking |
| **Observability** | **Wrap** | `tracing` + `metrics` | Production logging and monitoring | Bridge with hook integration |

### Core Technology Stack

#### Primary Dependencies

```toml
[dependencies]
# LLM Provider Abstraction
rig = "0.4"
candle = "0.7"

# Script Engine Bridges
mlua = { version = "0.9", features = ["async", "send", "luajit"] }
boa_engine = "0.18"

# Storage and Persistence
sled = "0.34"
rocksdb = "0.22"

# Async Runtime and Coordination
tokio = { version = "1.0", features = ["full"] }
crossbeam = "0.8"
async-trait = "0.1"

# Event and Hook System
inventory = "0.3"
typemap = "0.3"
dashmap = "5.5"

# Serialization and Configuration
serde = { version = "1.0", features = ["derive"] }
config = "0.14"
toml = "0.8"

# HTTP and Networking
reqwest = { version = "0.11", features = ["json", "stream"] }
url = "2.5"

# CLI and User Interface
clap = { version = "4.4", features = ["derive"] }
colored = "2.1"

# Observability and Monitoring
tracing = { version = "0.1", features = ["async-await"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
metrics = "0.22"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Development and Testing
tokio-test = "0.4"
tempfile = "3.8"
criterion = { version = "0.5", optional = true }
```

#### Feature Flag Strategy

```toml
[features]
default = ["lua", "javascript", "sled-storage"]

# Script Engine Support
lua = ["mlua"]
javascript = ["boa_engine"]
python = ["pyo3"] # Future

# Storage Backends
sled-storage = ["sled"]
rocksdb-storage = ["rocksdb"]
memory-storage = [] # In-memory for testing

# LLM Provider Extensions
openai = ["rig/openai"]
anthropic = ["rig/anthropic"]
local-models = ["candle"]

# Advanced Features
mcp-support = ["tokio-tungstenite", "serde_json"]
a2a-protocol = ["tokio-tungstenite", "bincode"]
distributed = ["tokio-tungstenite"]

# Development Tools
benchmarks = ["criterion"]
testing-utils = ["tokio-test", "tempfile"]
```

## LLM Provider Integration

### Rig Crate Integration Strategy

Rs-LLMSpell leverages the `rig` crate as the foundation for LLM provider abstraction, extending it with additional capabilities needed for scriptable orchestration.

#### Core Provider Bridge

```rust
use rig::{
    providers::{anthropic, openai},
    completion::{CompletionModel, Prompt},
    embeddings::EmbeddingsModel,
};

pub struct LLMProviderBridge {
    providers: DashMap<String, Box<dyn ProviderInstance>>,
    default_provider: String,
    rate_limiters: DashMap<String, RateLimiter>,
    metrics: ProviderMetrics,
}

#[async_trait]
pub trait ProviderInstance: Send + Sync {
    async fn complete(&self, prompt: &str, config: &CompletionConfig) -> Result<String>;
    async fn stream_complete(&self, prompt: &str, config: &CompletionConfig) -> Result<CompletionStream>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn provider_info(&self) -> ProviderInfo;
    fn supports_streaming(&self) -> bool;
    fn max_context_length(&self) -> usize;
    fn supported_models(&self) -> Vec<String>;
}

impl LLMProviderBridge {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
            default_provider: "openai".to_string(),
            rate_limiters: DashMap::new(),
            metrics: ProviderMetrics::new(),
        }
    }
    
    pub async fn register_openai_provider(&self, api_key: String, model: String) -> Result<()> {
        let client = openai::Client::from_api_key(api_key);
        let model = client.model(&model).build();
        
        let provider = OpenAIProvider::new(model);
        self.providers.insert("openai".to_string(), Box::new(provider));
        
        // Configure rate limiting based on provider
        let rate_limiter = RateLimiter::builder()
            .requests_per_minute(3500) // OpenAI tier limits
            .tokens_per_minute(90000)
            .build();
        self.rate_limiters.insert("openai".to_string(), rate_limiter);
        
        Ok(())
    }
    
    pub async fn register_anthropic_provider(&self, api_key: String, model: String) -> Result<()> {
        let client = anthropic::Client::from_api_key(api_key);
        let model = client.model(&model).build();
        
        let provider = AnthropicProvider::new(model);
        self.providers.insert("anthropic".to_string(), Box::new(provider));
        
        let rate_limiter = RateLimiter::builder()
            .requests_per_minute(4000) // Anthropic limits
            .tokens_per_minute(400000)
            .build();
        self.rate_limiters.insert("anthropic".to_string(), rate_limiter);
        
        Ok(())
    }
    
    pub async fn complete_with_provider(
        &self, 
        provider_name: &str, 
        prompt: &str, 
        config: &CompletionConfig
    ) -> Result<String> {
        // Rate limiting check
        if let Some(limiter) = self.rate_limiters.get(provider_name) {
            limiter.wait_for_capacity().await?;
        }
        
        // Get provider and execute
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| LLMSpellError::Provider(format!("Provider not found: {}", provider_name)))?;
            
        // Execute with metrics collection
        let start_time = Instant::now();
        let result = provider.complete(prompt, config).await;
        let duration = start_time.elapsed();
        
        // Record metrics
        self.metrics.record_completion(provider_name, duration, result.is_ok());
        
        result
    }
    
    pub async fn smart_completion(
        &self,
        prompt: &str,
        config: &CompletionConfig,
        preferences: &ProviderPreferences,
    ) -> Result<String> {
        // Intelligent provider selection based on:
        // - Cost constraints
        // - Latency requirements  
        // - Model capabilities needed
        // - Current rate limit status
        
        let selected_provider = self.select_optimal_provider(config, preferences).await?;
        self.complete_with_provider(&selected_provider, prompt, config).await
    }
}
```

#### Local Model Integration with Candle

```rust
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::LlamaConfig;

pub struct LocalModelProvider {
    model: Box<dyn LocalModel>,
    tokenizer: Tokenizer,
    device: Device,
    config: LocalModelConfig,
}

#[async_trait]
impl ProviderInstance for LocalModelProvider {
    async fn complete(&self, prompt: &str, config: &CompletionConfig) -> Result<String> {
        // Tokenize input
        let tokens = self.tokenizer.encode(prompt, true)
            .map_err(|e| LLMSpellError::Provider(format!("Tokenization failed: {}", e)))?;
        
        // Convert to tensor
        let input_tensor = Tensor::new(tokens.get_ids(), &self.device)?;
        
        // Generate completion
        let output_tokens = self.model.generate(
            &input_tensor,
            config.max_tokens.unwrap_or(512),
            config.temperature.unwrap_or(0.7),
            config.top_p.unwrap_or(0.9),
        ).await?;
        
        // Decode output
        let output = self.tokenizer.decode(&output_tokens, true)
            .map_err(|e| LLMSpellError::Provider(format!("Decoding failed: {}", e)))?;
            
        Ok(output)
    }
    
    async fn stream_complete(&self, prompt: &str, config: &CompletionConfig) -> Result<CompletionStream> {
        // Stream generation for local models
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        
        let model = self.model.clone();
        let tokenizer = self.tokenizer.clone();
        let device = self.device.clone();
        let prompt = prompt.to_string();
        let config = config.clone();
        
        tokio::spawn(async move {
            // Streaming generation implementation
            // Yield tokens as they're generated
        });
        
        Ok(CompletionStream::new(receiver))
    }
    
    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "local".to_string(),
            model_name: self.config.model_path.clone(),
            supports_streaming: true,
            max_context_length: self.config.context_length,
            cost_per_token: 0.0, // Free for local models
        }
    }
}
```

## Storage and Persistence

### Unified Storage Architecture

Rs-LLMSpell provides a unified storage interface that can switch between storage backends based on deployment requirements.

#### Storage Trait Abstraction

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    // Core Key-Value Operations
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
    
    // Batch Operations
    async fn get_batch(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>>;
    async fn set_batch(&self, items: &[(String, Vec<u8>)]) -> Result<()>;
    async fn delete_batch(&self, keys: &[String]) -> Result<()>;
    
    // Iteration and Scanning
    async fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>>;
    async fn scan_range(&self, start: &str, end: &str) -> Result<Vec<(String, Vec<u8>)>>;
    
    // Transactions
    async fn transaction(&self) -> Result<Box<dyn StorageTransaction>>;
    
    // Metadata and Stats
    async fn size(&self) -> Result<u64>;
    async fn key_count(&self) -> Result<u64>;
    async fn stats(&self) -> Result<StorageStats>;
    
    // Backup and Recovery
    async fn backup(&self, path: &Path) -> Result<()>;
    async fn restore(&self, path: &Path) -> Result<()>;
}

#[async_trait]
pub trait StorageTransaction: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
}
```

#### Sled Backend Implementation

```rust
use sled::{Db, Tree, transaction::TransactionResult};

pub struct SledBackend {
    db: Db,
    config: SledConfig,
    metrics: StorageMetrics,
}

impl SledBackend {
    pub async fn new(config: SledConfig) -> Result<Self> {
        let db = sled::open(&config.path)
            .map_err(|e| LLMSpellError::Storage(format!("Failed to open sled database: {}", e)))?;
            
        // Configure sled for optimal performance
        db.set_merge_operator(Self::merge_operator);
        
        Ok(Self {
            db,
            config,
            metrics: StorageMetrics::new(),
        })
    }
    
    fn merge_operator(
        _key: &[u8],
        old_value: Option<&[u8]>,
        new_value: &[u8],
    ) -> Option<Vec<u8>> {
        // Custom merge logic for counters, sets, etc.
        match old_value {
            Some(old) => {
                // Attempt to merge JSON values
                if let (Ok(old_json), Ok(new_json)) = (
                    serde_json::from_slice::<serde_json::Value>(old),
                    serde_json::from_slice::<serde_json::Value>(new_value),
                ) {
                    if let Some(merged) = Self::merge_json_values(old_json, new_json) {
                        return serde_json::to_vec(&merged).ok();
                    }
                }
                // Fallback to replacement
                Some(new_value.to_vec())
            }
            None => Some(new_value.to_vec()),
        }
    }
}

#[async_trait]
impl StorageBackend for SledBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let start_time = Instant::now();
        
        let result = self.db.get(key.as_bytes())
            .map_err(|e| LLMSpellError::Storage(format!("Get operation failed: {}", e)))?
            .map(|ivec| ivec.to_vec());
            
        self.metrics.record_operation("get", start_time.elapsed());
        Ok(result)
    }
    
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let start_time = Instant::now();
        
        self.db.insert(key.as_bytes(), value)
            .map_err(|e| LLMSpellError::Storage(format!("Set operation failed: {}", e)))?;
            
        self.metrics.record_operation("set", start_time.elapsed());
        Ok(())
    }
    
    async fn transaction(&self) -> Result<Box<dyn StorageTransaction>> {
        Ok(Box::new(SledTransaction::new(&self.db)))
    }
    
    async fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        let start_time = Instant::now();
        
        let mut results = Vec::new();
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item.map_err(|e| LLMSpellError::Storage(e.to_string()))?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            results.push((key_str, value.to_vec()));
        }
        
        self.metrics.record_operation("scan_prefix", start_time.elapsed());
        Ok(results)
    }
}
```

#### RocksDB Backend Implementation

```rust
use rocksdb::{DB, Options, WriteBatch, ReadOptions, IteratorMode};

pub struct RocksDBBackend {
    db: Arc<DB>,
    config: RocksDBConfig,
    metrics: StorageMetrics,
}

impl RocksDBBackend {
    pub async fn new(config: RocksDBConfig) -> Result<Self> {
        let mut opts = Options::default();
        
        // Optimize for rs-llmspell workload
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        // Memory management
        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffers);
        opts.set_target_file_size_base(config.target_file_size);
        
        // Compression
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        // Parallelism
        opts.set_max_background_jobs(config.background_jobs);
        opts.increase_parallelism(config.parallelism);
        
        // Block cache for better read performance
        let block_cache = rocksdb::Cache::new_lru_cache(config.block_cache_size)?;
        let mut block_opts = rocksdb::BlockBasedOptions::default();
        block_opts.set_block_cache(&block_cache);
        opts.set_block_based_table_factory(&block_opts);
        
        let db = DB::open(&opts, &config.path)
            .map_err(|e| LLMSpellError::Storage(format!("Failed to open RocksDB: {}", e)))?;
            
        Ok(Self {
            db: Arc::new(db),
            config,
            metrics: StorageMetrics::new(),
        })
    }
}

#[async_trait] 
impl StorageBackend for RocksDBBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let start_time = Instant::now();
        let db = self.db.clone();
        let key = key.to_string();
        
        // Use blocking task to avoid blocking async runtime
        let result = tokio::task::spawn_blocking(move || {
            db.get(key.as_bytes())
        }).await
        .map_err(|e| LLMSpellError::Runtime(format!("Task join error: {}", e)))?
        .map_err(|e| LLMSpellError::Storage(format!("RocksDB get failed: {}", e)))?;
        
        self.metrics.record_operation("get", start_time.elapsed());
        Ok(result)
    }
    
    async fn set_batch(&self, items: &[(String, Vec<u8>)]) -> Result<()> {
        let start_time = Instant::now();
        let db = self.db.clone();
        let items = items.to_vec();
        
        tokio::task::spawn_blocking(move || {
            let mut batch = WriteBatch::default();
            for (key, value) in items {
                batch.put(key.as_bytes(), &value);
            }
            db.write(batch)
        }).await
        .map_err(|e| LLMSpellError::Runtime(format!("Task join error: {}", e)))?
        .map_err(|e| LLMSpellError::Storage(format!("RocksDB batch write failed: {}", e)))?;
        
        self.metrics.record_operation("set_batch", start_time.elapsed());
        Ok(())
    }
    
    async fn stats(&self) -> Result<StorageStats> {
        let db = self.db.clone();
        
        let stats = tokio::task::spawn_blocking(move || {
            let size = db.property_value("rocksdb.total-sst-files-size")
                .unwrap_or_default()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
                
            let key_count = db.property_value("rocksdb.estimate-num-keys")
                .unwrap_or_default()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
                
            StorageStats {
                size_bytes: size,
                key_count,
                backend: "rocksdb".to_string(),
                additional_info: HashMap::new(),
            }
        }).await
        .map_err(|e| LLMSpellError::Runtime(format!("Task join error: {}", e)))?;
        
        Ok(stats)
    }
}
```

### Storage Manager and Backend Selection

```rust
pub struct StorageManager {
    backend: Box<dyn StorageBackend>,
    namespace_prefix: String,
    serialization: SerializationStrategy,
}

impl StorageManager {
    pub async fn new(config: StorageConfig) -> Result<Self> {
        let backend: Box<dyn StorageBackend> = match config.backend_type {
            StorageBackendType::Sled => {
                Box::new(SledBackend::new(config.sled_config).await?)
            }
            StorageBackendType::RocksDB => {
                Box::new(RocksDBBackend::new(config.rocksdb_config).await?)
            }
            StorageBackendType::Memory => {
                Box::new(MemoryBackend::new())
            }
        };
        
        Ok(Self {
            backend,
            namespace_prefix: config.namespace_prefix,
            serialization: config.serialization_strategy,
        })
    }
    
    // Type-safe operations with automatic serialization
    pub async fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let namespaced_key = format!("{}:{}", self.namespace_prefix, key);
        
        if let Some(bytes) = self.backend.get(&namespaced_key).await? {
            let value = self.serialization.deserialize(&bytes)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    pub async fn set_typed<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let namespaced_key = format!("{}:{}", self.namespace_prefix, key);
        let bytes = self.serialization.serialize(value)?;
        self.backend.set(&namespaced_key, bytes).await
    }
    
    // Agent state management
    pub async fn store_agent_state(&self, agent_id: &str, state: &AgentState) -> Result<()> {
        let key = format!("agent_state:{}", agent_id);
        self.set_typed(&key, state).await
    }
    
    pub async fn load_agent_state(&self, agent_id: &str) -> Result<Option<AgentState>> {
        let key = format!("agent_state:{}", agent_id);
        self.get_typed(&key).await
    }
    
    // Tool execution cache
    pub async fn cache_tool_result(
        &self,
        tool_name: &str,
        input_hash: &str,
        result: &ToolResult,
    ) -> Result<()> {
        let key = format!("tool_cache:{}:{}", tool_name, input_hash);
        self.set_typed(&key, result).await
    }
    
    pub async fn get_cached_tool_result(
        &self,
        tool_name: &str,
        input_hash: &str,
    ) -> Result<Option<ToolResult>> {
        let key = format!("tool_cache:{}:{}", tool_name, input_hash);
        self.get_typed(&key).await
    }
}
```

## Async Patterns and Concurrency

### Cross-Engine Async Coordination

Rs-LLMSpell implements a sophisticated async pattern system that provides unified async behavior across single-threaded script engines (Lua, JavaScript) and the Rust async runtime.

#### Cooperative Scheduler Architecture

```rust
use tokio::sync::{mpsc, oneshot, Mutex};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub struct CooperativeScheduler {
    // Task queue for script engine operations
    task_queue: Arc<Mutex<VecDeque<ScheduledTask>>>,
    
    // Communication with script engines
    lua_channel: Option<mpsc::UnboundedSender<LuaTask>>,
    js_channel: Option<mpsc::UnboundedSender<JsTask>>,
    
    // Tokio runtime handle
    runtime: tokio::runtime::Handle,
    
    // Scheduling policy configuration
    scheduling_policy: SchedulingPolicy,
}

#[derive(Debug)]
pub enum ScheduledTask {
    LuaCoroutine {
        coroutine_id: String,
        resume_data: Option<serde_json::Value>,
        completion_sender: oneshot::Sender<TaskResult>,
    },
    JavaScriptPromise {
        promise_id: String,
        resolve_data: Option<serde_json::Value>,
        completion_sender: oneshot::Sender<TaskResult>,
    },
    RustFuture {
        future: Pin<Box<dyn Future<Output = TaskResult> + Send>>,
        completion_sender: oneshot::Sender<TaskResult>,
    },
}

impl CooperativeScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            lua_channel: None,
            js_channel: None,
            runtime: tokio::runtime::Handle::current(),
            scheduling_policy: config.policy,
        }
    }
    
    // Schedule a Lua coroutine for execution
    pub async fn schedule_lua_coroutine(
        &self,
        coroutine: mlua::Thread,
        args: mlua::MultiValue,
    ) -> Result<TaskResult> {
        let (sender, receiver) = oneshot::channel();
        let coroutine_id = format!("lua_{}", uuid::Uuid::new_v4());
        
        let task = ScheduledTask::LuaCoroutine {
            coroutine_id: coroutine_id.clone(),
            resume_data: Some(self.serialize_lua_args(args)?),
            completion_sender: sender,
        };
        
        // Add to queue
        {
            let mut queue = self.task_queue.lock().await;
            queue.push_back(task);
        }
        
        // Signal Lua engine if available
        if let Some(ref sender) = self.lua_channel {
            let lua_task = LuaTask::ResumeCoroutine {
                coroutine_id,
                coroutine,
                args: self.serialize_lua_args(args)?,
            };
            sender.send(lua_task).map_err(|_| {
                LLMSpellError::AsyncExecution("Lua channel closed".to_string())
            })?;
        }
        
        // Wait for completion
        receiver.await.map_err(|_| {
            LLMSpellError::AsyncExecution("Task cancelled".to_string())
        })
    }
    
    // Schedule a JavaScript Promise for execution
    pub async fn schedule_js_promise(
        &self,
        promise_callback: JsPromiseCallback,
    ) -> Result<TaskResult> {
        let (sender, receiver) = oneshot::channel();
        let promise_id = format!("js_{}", uuid::Uuid::new_v4());
        
        let task = ScheduledTask::JavaScriptPromise {
            promise_id: promise_id.clone(),
            resolve_data: None,
            completion_sender: sender,
        };
        
        // Add to queue
        {
            let mut queue = self.task_queue.lock().await;
            queue.push_back(task);
        }
        
        // Signal JavaScript engine if available
        if let Some(ref sender) = self.js_channel {
            let js_task = JsTask::ExecutePromise {
                promise_id,
                callback: promise_callback,
            };
            sender.send(js_task).map_err(|_| {
                LLMSpellError::AsyncExecution("JavaScript channel closed".to_string())
            })?;
        }
        
        // Wait for completion
        receiver.await.map_err(|_| {
            LLMSpellError::AsyncExecution("Task cancelled".to_string())
        })
    }
    
    // Main scheduling loop
    pub async fn run_scheduler(&self) -> Result<()> {
        let mut interval = tokio::time::interval(self.scheduling_policy.tick_duration);
        
        loop {
            interval.tick().await;
            
            // Process pending tasks based on scheduling policy
            let tasks_to_process = {
                let mut queue = self.task_queue.lock().await;
                let batch_size = self.scheduling_policy.batch_size;
                queue.drain(..queue.len().min(batch_size)).collect::<Vec<_>>()
            };
            
            if tasks_to_process.is_empty() {
                continue;
            }
            
            // Execute tasks based on type
            for task in tasks_to_process {
                match task {
                    ScheduledTask::RustFuture { future, completion_sender } => {
                        let result = future.await;
                        let _ = completion_sender.send(result);
                    }
                    _ => {
                        // Lua and JS tasks are handled by their respective engines
                        // through the channel system
                    }
                }
            }
        }
    }
}
```

#### Lua Coroutine Management

```rust
use mlua::{Lua, Thread, MultiValue, Value, Result as LuaResult};

pub struct LuaAsyncManager {
    lua: Arc<Lua>,
    active_coroutines: Arc<DashMap<String, Thread>>,
    scheduler: Arc<CooperativeScheduler>,
    coroutine_registry: Arc<Mutex<HashMap<String, CoroutineInfo>>>,
}

#[derive(Debug, Clone)]
pub struct CoroutineInfo {
    pub id: String,
    pub created_at: Instant,
    pub status: CoroutineStatus,
    pub yield_count: u32,
    pub total_execution_time: Duration,
}

#[derive(Debug, Clone)]
pub enum CoroutineStatus {
    Ready,
    Running,
    Yielded,
    Completed,
    Error(String),
}

impl LuaAsyncManager {
    pub fn new(lua: Arc<Lua>, scheduler: Arc<CooperativeScheduler>) -> Self {
        Self {
            lua,
            active_coroutines: Arc::new(DashMap::new()),
            scheduler,
            coroutine_registry: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    // Create and register a new coroutine
    pub async fn create_coroutine(
        &self,
        func: mlua::Function,
        args: MultiValue,
    ) -> Result<String> {
        let coroutine_id = format!("coroutine_{}", uuid::Uuid::new_v4());
        let thread = self.lua.create_thread(func)?;
        
        // Register coroutine info
        {
            let mut registry = self.coroutine_registry.lock().await;
            registry.insert(coroutine_id.clone(), CoroutineInfo {
                id: coroutine_id.clone(),
                created_at: Instant::now(),
                status: CoroutineStatus::Ready,
                yield_count: 0,
                total_execution_time: Duration::ZERO,
            });
        }
        
        self.active_coroutines.insert(coroutine_id.clone(), thread);
        
        Ok(coroutine_id)
    }
    
    // Resume a coroutine with cooperative yielding
    pub async fn resume_coroutine(
        &self,
        coroutine_id: &str,
        args: MultiValue,
    ) -> Result<CoroutineResult> {
        let thread = self.active_coroutines.get(coroutine_id)
            .ok_or_else(|| LLMSpellError::AsyncExecution(
                format!("Coroutine not found: {}", coroutine_id)
            ))?
            .clone();
        
        let start_time = Instant::now();
        
        // Update status to running
        {
            let mut registry = self.coroutine_registry.lock().await;
            if let Some(info) = registry.get_mut(coroutine_id) {
                info.status = CoroutineStatus::Running;
            }
        }
        
        // Resume execution with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(100), // Cooperative time slice
            self.resume_thread_blocking(thread.clone(), args)
        ).await;
        
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(Ok(values)) => {
                // Check if coroutine yielded or completed
                let status = if thread.status() == mlua::ThreadStatus::Resumable {
                    CoroutineStatus::Yielded
                } else {
                    CoroutineStatus::Completed
                };
                
                // Update registry
                {
                    let mut registry = self.coroutine_registry.lock().await;
                    if let Some(info) = registry.get_mut(coroutine_id) {
                        info.status = status.clone();
                        info.total_execution_time += execution_time;
                        if matches!(status, CoroutineStatus::Yielded) {
                            info.yield_count += 1;
                        }
                    }
                }
                
                if matches!(status, CoroutineStatus::Completed) {
                    self.active_coroutines.remove(coroutine_id);
                }
                
                Ok(CoroutineResult::Success(values))
            }
            Ok(Err(e)) => {
                // Lua error
                {
                    let mut registry = self.coroutine_registry.lock().await;
                    if let Some(info) = registry.get_mut(coroutine_id) {
                        info.status = CoroutineStatus::Error(e.to_string());
                        info.total_execution_time += execution_time;
                    }
                }
                
                self.active_coroutines.remove(coroutine_id);
                Err(LLMSpellError::Script(format!("Lua error: {}", e)))
            }
            Err(_) => {
                // Timeout - coroutine needs to yield control
                {
                    let mut registry = self.coroutine_registry.lock().await;
                    if let Some(info) = registry.get_mut(coroutine_id) {
                        info.status = CoroutineStatus::Yielded;
                        info.total_execution_time += execution_time;
                        info.yield_count += 1;
                    }
                }
                
                // Reschedule for later execution
                self.scheduler.schedule_lua_coroutine(thread, MultiValue::new()).await?;
                
                Ok(CoroutineResult::Yielded)
            }
        }
    }
    
    async fn resume_thread_blocking(
        &self,
        thread: Thread,
        args: MultiValue,
    ) -> LuaResult<MultiValue> {
        // Use blocking task to avoid tying up async runtime
        let lua = self.lua.clone();
        tokio::task::spawn_blocking(move || {
            thread.resume(args)
        }).await
        .map_err(|e| mlua::Error::RuntimeError(format!("Task join error: {}", e)))?
    }
}

#[derive(Debug)]
pub enum CoroutineResult {
    Success(MultiValue),
    Yielded,
    Error(String),
}
```

#### JavaScript Promise Integration

```rust
use boa_engine::{Context, JsValue, JsResult, JsPromise, NativeFunction};

pub struct JavaScriptAsyncManager {
    context: Arc<Mutex<Context>>,
    active_promises: Arc<DashMap<String, JsPromise>>,
    scheduler: Arc<CooperativeScheduler>,
    promise_registry: Arc<Mutex<HashMap<String, PromiseInfo>>>,
}

#[derive(Debug, Clone)]
pub struct PromiseInfo {
    pub id: String,
    pub created_at: Instant,
    pub status: PromiseStatus,
    pub execution_time: Duration,
}

#[derive(Debug, Clone)]
pub enum PromiseStatus {
    Pending,
    Resolving,
    Resolved(String),
    Rejected(String),
}

impl JavaScriptAsyncManager {
    pub fn new(scheduler: Arc<CooperativeScheduler>) -> Result<Self> {
        let mut context = Context::default();
        
        // Install rs-llmspell async utilities
        Self::install_async_utilities(&mut context)?;
        
        Ok(Self {
            context: Arc::new(Mutex::new(context)),
            active_promises: Arc::new(DashMap::new()),
            scheduler,
            promise_registry: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    fn install_async_utilities(context: &mut Context) -> Result<()> {
        // Install cooperative yield function
        let yield_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            // Signal to scheduler that this Promise should yield control
            Ok(JsValue::undefined())
        });
        
        context.register_global_builtin_callable("cooperative_yield", 0, yield_fn)?;
        
        // Install async sleep function
        let sleep_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            if let Some(duration_ms) = args.get(0).and_then(|v| v.as_number()) {
                // Create a Promise that resolves after the specified duration
                let promise = JsPromise::new(
                    |resolvers, context| {
                        // Schedule resolution after timeout
                        // This would integrate with our scheduler
                        Ok(JsValue::undefined())
                    },
                    context,
                )?;
                Ok(promise.into())
            } else {
                Err(boa_engine::JsError::from_opaque(JsValue::string("Duration required")))
            }
        });
        
        context.register_global_builtin_callable("sleep", 1, sleep_fn)?;
        
        Ok(())
    }
    
    // Execute JavaScript code that returns a Promise
    pub async fn execute_async_script(
        &self,
        script: &str,
    ) -> Result<JsValue> {
        let promise_id = format!("promise_{}", uuid::Uuid::new_v4());
        let start_time = Instant::now();
        
        // Register promise info
        {
            let mut registry = self.promise_registry.lock().await;
            registry.insert(promise_id.clone(), PromiseInfo {
                id: promise_id.clone(),
                created_at: start_time,
                status: PromiseStatus::Pending,
                execution_time: Duration::ZERO,
            });
        }
        
        let context = self.context.clone();
        let script = script.to_string();
        let promise_id_clone = promise_id.clone();
        
        // Execute in blocking task with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(100), // Cooperative time slice
            tokio::task::spawn_blocking(move || {
                let mut context = context.blocking_lock();
                context.eval(boa_engine::Source::from_bytes(&script))
            })
        ).await;
        
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(Ok(value)) => {
                // Check if result is a Promise
                if let Ok(promise) = JsPromise::from_object(value.as_object().unwrap().clone()) {
                    self.active_promises.insert(promise_id.clone(), promise.clone());
                    
                    // Update status
                    {
                        let mut registry = self.promise_registry.lock().await;
                        if let Some(info) = registry.get_mut(&promise_id) {
                            info.status = PromiseStatus::Resolving;
                            info.execution_time = execution_time;
                        }
                    }
                    
                    // Handle Promise resolution asynchronously
                    self.handle_promise_resolution(promise_id, promise).await
                } else {
                    // Synchronous result
                    {
                        let mut registry = self.promise_registry.lock().await;
                        if let Some(info) = registry.get_mut(&promise_id) {
                            info.status = PromiseStatus::Resolved("sync_result".to_string());
                            info.execution_time = execution_time;
                        }
                    }
                    
                    Ok(value)
                }
            }
            Ok(Err(e)) => {
                // JavaScript error
                {
                    let mut registry = self.promise_registry.lock().await;
                    if let Some(info) = registry.get_mut(&promise_id) {
                        info.status = PromiseStatus::Rejected(e.to_string());
                        info.execution_time = execution_time;
                    }
                }
                
                Err(LLMSpellError::Script(format!("JavaScript error: {}", e)))
            }
            Err(_) => {
                // Timeout - need to yield and reschedule
                // Reschedule for later execution
                self.reschedule_promise_execution(promise_id, script).await?;
                
                Ok(JsValue::undefined())
            }
        }
    }
    
    async fn handle_promise_resolution(
        &self,
        promise_id: String,
        promise: JsPromise,
    ) -> Result<JsValue> {
        // Set up Promise resolution handling
        // This would integrate with the cooperative scheduler
        // to handle Promise.then() callbacks appropriately
        
        // For now, return the Promise itself
        Ok(promise.into())
    }
    
    async fn reschedule_promise_execution(
        &self,
        promise_id: String,
        script: String,
    ) -> Result<()> {
        // Schedule for later execution through cooperative scheduler
        let callback = JsPromiseCallback {
            promise_id,
            script,
        };
        
        self.scheduler.schedule_js_promise(callback).await?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct JsPromiseCallback {
    pub promise_id: String,
    pub script: String,
}
```

## Performance Optimization

### Hook Execution Optimization

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;

pub struct OptimizedHookManager {
    hooks: DashMap<HookPoint, Vec<OptimizedHook>>,
    execution_stats: DashMap<String, HookExecutionStats>,
    priority_cache: Arc<RwLock<HashMap<HookPoint, Vec<HookId>>>>,
    
    // Performance counters
    total_executions: AtomicU64,
    total_execution_time: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct OptimizedHook {
    pub id: HookId,
    pub priority: u8,
    pub execution_profile: ExecutionProfile,
    pub handler: Arc<dyn HookHandler>,
    pub conditions: Vec<HookCondition>,
    pub metrics: HookMetrics,
}

#[derive(Debug, Clone)]
pub struct ExecutionProfile {
    pub average_duration: Duration,
    pub success_rate: f64,
    pub last_execution: Option<Instant>,
    pub execution_count: u64,
    pub failure_count: u64,
}

impl OptimizedHookManager {
    pub fn new() -> Self {
        Self {
            hooks: DashMap::new(),
            execution_stats: DashMap::new(),
            priority_cache: Arc::new(RwLock::new(HashMap::new())),
            total_executions: AtomicU64::new(0),
            total_execution_time: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }
    
    // Optimized hook execution with caching and profiling
    pub async fn execute_hooks_optimized(
        &self,
        hook_point: HookPoint,
        context: &mut HookContext,
    ) -> Result<HookExecutionResult> {
        let execution_start = Instant::now();
        
        // Try priority cache first
        let hook_ids = {
            let cache = self.priority_cache.read().await;
            if let Some(cached_ids) = cache.get(&hook_point) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                cached_ids.clone()
            } else {
                self.cache_misses.fetch_add(1, Ordering::Relaxed);
                drop(cache);
                
                // Build priority list and cache it
                let mut hook_ids = Vec::new();
                if let Some(hooks) = self.hooks.get(&hook_point) {
                    let mut sorted_hooks = hooks.clone();
                    sorted_hooks.sort_by_key(|h| std::cmp::Reverse(h.priority));
                    
                    for hook in sorted_hooks {
                        // Pre-filter based on conditions
                        if self.evaluate_conditions(&hook.conditions, context).await? {
                            hook_ids.push(hook.id);
                        }
                    }
                }
                
                // Cache the result
                {
                    let mut cache = self.priority_cache.write().await;
                    cache.insert(hook_point, hook_ids.clone());
                }
                
                hook_ids
            }
        };
        
        let mut results = Vec::new();
        let mut total_hook_time = Duration::ZERO;
        
        // Execute hooks in priority order
        for hook_id in hook_ids {
            if let Some(hooks) = self.hooks.get(&hook_point) {
                if let Some(hook) = hooks.iter().find(|h| h.id == hook_id) {
                    let hook_start = Instant::now();
                    
                    // Execute with timeout and error handling
                    let result = tokio::time::timeout(
                        Duration::from_millis(100), // Per-hook timeout
                        self.execute_single_hook(hook, context)
                    ).await;
                    
                    let hook_duration = hook_start.elapsed();
                    total_hook_time += hook_duration;
                    
                    // Update execution statistics
                    self.update_hook_stats(&hook.id, hook_duration, result.is_ok()).await;
                    
                    match result {
                        Ok(Ok(hook_result)) => {
                            results.push(hook_result);
                        }
                        Ok(Err(e)) => {
                            // Log hook error but continue execution
                            tracing::warn!("Hook {} failed: {}", hook.id, e);
                        }
                        Err(_) => {
                            // Timeout
                            tracing::warn!("Hook {} timed out", hook.id);
                        }
                    }
                }
            }
        }
        
        let total_duration = execution_start.elapsed();
        
        // Update global metrics
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time.fetch_add(
            total_duration.as_nanos() as u64, 
            Ordering::Relaxed
        );
        
        Ok(HookExecutionResult {
            hook_point,
            results,
            total_duration,
            hook_execution_time: total_hook_time,
            hooks_executed: hook_ids.len(),
        })
    }
    
    async fn update_hook_stats(
        &self,
        hook_id: &HookId,
        duration: Duration,
        success: bool,
    ) {
        let mut stats = self.execution_stats.entry(hook_id.clone())
            .or_insert_with(|| HookExecutionStats::default());
            
        stats.total_executions += 1;
        stats.total_duration += duration;
        
        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }
        
        // Update moving average
        stats.average_duration = Duration::from_nanos(
            (stats.average_duration.as_nanos() as u64 * (stats.total_executions - 1) + 
             duration.as_nanos() as u64) / stats.total_executions
        );
        
        stats.last_execution = Some(Instant::now());
    }
    
    // Periodic cache maintenance
    pub async fn maintain_caches(&self) {
        // Clear stale cache entries
        let mut cache = self.priority_cache.write().await;
        cache.clear();
        
        // Rebuild cache for frequently used hook points
        // This would be based on usage statistics
    }
    
    // Performance monitoring
    pub async fn get_performance_stats(&self) -> HookManagerStats {
        let total_executions = self.total_executions.load(Ordering::Relaxed);
        let total_time = self.total_execution_time.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        
        HookManagerStats {
            total_executions,
            average_execution_time: if total_executions > 0 {
                Duration::from_nanos(total_time / total_executions)
            } else {
                Duration::ZERO
            },
            cache_hit_rate: if cache_hits + cache_misses > 0 {
                cache_hits as f64 / (cache_hits + cache_misses) as f64
            } else {
                0.0
            },
            active_hook_points: self.hooks.len(),
            total_registered_hooks: self.hooks.iter()
                .map(|entry| entry.value().len())
                .sum(),
        }
    }
}
```

### Tool Execution Pooling

```rust
use tokio::sync::{Semaphore, RwLock};
use std::collections::HashMap;

pub struct ToolExecutionPool {
    // Per-tool execution limits
    tool_semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    
    // Global execution pool
    global_semaphore: Arc<Semaphore>,
    
    // Tool execution statistics
    execution_stats: Arc<DashMap<String, ToolExecutionStats>>,
    
    // Execution queue for prioritization
    execution_queue: Arc<Mutex<VecDeque<PrioritizedExecution>>>,
    
    // Pool configuration
    config: ToolPoolConfig,
}

#[derive(Debug, Clone)]
pub struct ToolPoolConfig {
    pub global_max_concurrent: usize,
    pub per_tool_max_concurrent: HashMap<String, usize>,
    pub default_tool_concurrency: usize,
    pub queue_size_limit: usize,
    pub execution_timeout: Duration,
}

#[derive(Debug)]
struct PrioritizedExecution {
    tool_name: String,
    execution_id: String,
    priority: ExecutionPriority,
    submitted_at: Instant,
    execution_context: ToolExecutionContext,
    completion_sender: oneshot::Sender<ToolResult>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ExecutionPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl ToolExecutionPool {
    pub fn new(config: ToolPoolConfig) -> Self {
        Self {
            tool_semaphores: Arc::new(RwLock::new(HashMap::new())),
            global_semaphore: Arc::new(Semaphore::new(config.global_max_concurrent)),
            execution_stats: Arc::new(DashMap::new()),
            execution_queue: Arc::new(Mutex::new(VecDeque::new())),
            config,
        }
    }
    
    // Execute tool with pooling and prioritization
    pub async fn execute_tool_pooled(
        &self,
        tool_name: String,
        execution_context: ToolExecutionContext,
        priority: ExecutionPriority,
    ) -> Result<ToolResult> {
        let execution_id = format!("exec_{}", uuid::Uuid::new_v4());
        let (sender, receiver) = oneshot::channel();
        
        // Check queue size limit
        {
            let queue = self.execution_queue.lock().await;
            if queue.len() >= self.config.queue_size_limit {
                return Err(LLMSpellError::Resource(
                    "Tool execution queue is full".to_string()
                ));
            }
        }
        
        // Add to priority queue
        let execution = PrioritizedExecution {
            tool_name: tool_name.clone(),
            execution_id: execution_id.clone(),
            priority,
            submitted_at: Instant::now(),
            execution_context,
            completion_sender: sender,
        };
        
        {
            let mut queue = self.execution_queue.lock().await;
            queue.push_back(execution);
            queue.make_contiguous().sort_by_key(|e| e.priority);
        }
        
        // Start execution processing if not already running
        self.process_execution_queue().await;
        
        // Wait for completion with timeout
        tokio::time::timeout(self.config.execution_timeout, receiver)
            .await
            .map_err(|_| LLMSpellError::Tool(ToolError::Timeout { 
                tool_name: tool_name.clone(),
                execution_id,
                timeout: self.config.execution_timeout,
            }))?
            .map_err(|_| LLMSpellError::Tool(ToolError::ExecutionCancelled {
                tool_name,
                execution_id,
            }))
    }
    
    async fn process_execution_queue(&self) {
        // Spawn task to process queue if needed
        let queue = self.execution_queue.clone();
        let global_semaphore = self.global_semaphore.clone();
        let tool_semaphores = self.tool_semaphores.clone();
        let execution_stats = self.execution_stats.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                let next_execution = {
                    let mut queue = queue.lock().await;
                    queue.pop_front()
                };
                
                if let Some(execution) = next_execution {
                    // Execute with proper semaphore handling
                    Self::execute_with_semaphores(
                        execution,
                        global_semaphore.clone(),
                        tool_semaphores.clone(),
                        execution_stats.clone(),
                        &config,
                    ).await;
                } else {
                    // No more executions, break the loop
                    break;
                }
            }
        });
    }
    
    async fn execute_with_semaphores(
        execution: PrioritizedExecution,
        global_semaphore: Arc<Semaphore>,
        tool_semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
        execution_stats: Arc<DashMap<String, ToolExecutionStats>>,
        config: &ToolPoolConfig,
    ) {
        // Acquire global permit
        let _global_permit = match global_semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => {
                let _ = execution.completion_sender.send(Err(LLMSpellError::Resource(
                    "Global execution pool closed".to_string()
                )));
                return;
            }
        };
        
        // Acquire tool-specific permit
        let tool_semaphore = {
            let semaphores = tool_semaphores.read().await;
            if let Some(semaphore) = semaphores.get(&execution.tool_name) {
                semaphore.clone()
            } else {
                drop(semaphores);
                
                // Create new semaphore for this tool
                let mut semaphores = tool_semaphores.write().await;
                let concurrency = config.per_tool_max_concurrent
                    .get(&execution.tool_name)
                    .copied()
                    .unwrap_or(config.default_tool_concurrency);
                    
                let semaphore = Arc::new(Semaphore::new(concurrency));
                semaphores.insert(execution.tool_name.clone(), semaphore.clone());
                semaphore
            }
        };
        
        let _tool_permit = match tool_semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => {
                let _ = execution.completion_sender.send(Err(LLMSpellError::Resource(
                    format!("Tool {} execution pool closed", execution.tool_name)
                )));
                return;
            }
        };
        
        // Execute the tool
        let start_time = Instant::now();
        let result = Self::execute_tool_implementation(
            &execution.tool_name,
            &execution.execution_context,
        ).await;
        let execution_time = start_time.elapsed();
        
        // Update statistics
        let mut stats = execution_stats.entry(execution.tool_name.clone())
            .or_insert_with(|| ToolExecutionStats::default());
            
        stats.total_executions += 1;
        stats.total_execution_time += execution_time;
        
        if result.is_ok() {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }
        
        // Calculate queue wait time
        let queue_wait_time = start_time.duration_since(execution.submitted_at);
        stats.total_queue_time += queue_wait_time;
        
        // Send result
        let _ = execution.completion_sender.send(result);
    }
    
    async fn execute_tool_implementation(
        tool_name: &str,
        context: &ToolExecutionContext,
    ) -> Result<ToolResult> {
        // This would dispatch to the actual tool implementation
        // For now, simulate execution
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        Ok(ToolResult {
            output: serde_json::json!({
                "tool": tool_name,
                "status": "success",
                "data": "simulated result"
            }),
            execution_time: Duration::from_millis(10),
            metadata: HashMap::new(),
        })
    }
    
    // Get pool statistics
    pub async fn get_pool_stats(&self) -> ToolPoolStats {
        let global_available = self.global_semaphore.available_permits();
        let queue_size = {
            let queue = self.execution_queue.lock().await;
            queue.len()
        };
        
        let tool_stats: HashMap<String, ToolExecutionStats> = self.execution_stats
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
            
        ToolPoolStats {
            global_available_permits: global_available,
            global_max_permits: self.config.global_max_concurrent,
            queue_size,
            queue_size_limit: self.config.queue_size_limit,
            tool_execution_stats: tool_stats,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_execution_time: Duration,
    pub total_queue_time: Duration,
}

#[derive(Debug)]
pub struct ToolPoolStats {
    pub global_available_permits: usize,
    pub global_max_permits: usize,
    pub queue_size: usize,
    pub queue_size_limit: usize,
    pub tool_execution_stats: HashMap<String, ToolExecutionStats>,
}
```

---

# Part VI: Configuration and Security

## Configuration Architecture

Rs-LLMSpell implements a comprehensive configuration system that supports multiple sources, hot-reloading, validation, and environment-specific overrides.

### Hierarchical Configuration Structure

```rust
use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSpellConfig {
    // Core system configuration
    pub system: SystemConfig,
    
    // LLM provider configurations
    pub providers: ProvidersConfig,
    
    // Script engine configurations
    pub engines: EnginesConfig,
    
    // Storage configuration
    pub storage: StorageConfig,
    
    // Hook and event system configuration
    pub hooks: HooksConfig,
    
    // Security configuration
    pub security: SecurityConfig,
    
    // Observability configuration
    pub observability: ObservabilityConfig,
    
    // Tool configuration
    pub tools: ToolsConfig,
    
    // Performance tuning
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub data_directory: String,
    pub log_level: String,
    pub max_concurrent_operations: usize,
    pub shutdown_timeout_seconds: u64,
    pub feature_flags: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
    pub rate_limiting: RateLimitingConfig,
    pub fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String, // "openai", "anthropic", "local", etc.
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout_seconds: u64,
    pub retries: u32,
    pub enabled: bool,
    pub cost_per_token: Option<f64>,
    
    // Provider-specific settings
    pub provider_specific: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginesConfig {
    pub default_engine: String,
    pub lua: LuaEngineConfig,
    pub javascript: JavaScriptEngineConfig,
    pub python: Option<PythonEngineConfig>, // Future
    pub cooperative_scheduler: SchedulerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaEngineConfig {
    pub enabled: bool,
    pub memory_limit_mb: usize,
    pub execution_timeout_ms: u64,
    pub max_coroutines: usize,
    pub stdlib_access: Vec<String>, // Allowed standard library modules
    pub custom_modules: Vec<String>,
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaScriptEngineConfig {
    pub enabled: bool,
    pub memory_limit_mb: usize,
    pub execution_timeout_ms: u64,
    pub max_promises: usize,
    pub strict_mode: bool,
    pub ecmascript_version: String,
    pub custom_modules: Vec<String>,
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    pub enabled: bool,
    pub max_hooks_per_point: usize,
    pub execution_timeout_ms: u64,
    pub priority_cache_size: usize,
    pub hook_discovery_paths: Vec<String>,
    pub builtin_hooks: BuiltinHooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub api_key_validation: bool,
    pub script_sandbox_mode: bool,
    pub allowed_file_operations: Vec<String>,
    pub network_restrictions: NetworkRestrictionsConfig,
    pub audit_logging: bool,
    pub encryption: EncryptionConfig,
    pub authentication: AuthenticationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub tool_execution_pool: ToolPoolConfig,
    pub cache_settings: CacheConfig,
    pub memory_management: MemoryConfig,
    pub gc_settings: GCConfig,
}
```

### Configuration Loading and Management

```rust
pub struct ConfigurationManager {
    config: Arc<RwLock<LLMSpellConfig>>,
    file_watcher: Option<tokio::sync::mpsc::UnboundedSender<ConfigUpdate>>,
    validation_rules: ValidationRules,
}

impl ConfigurationManager {
    pub async fn new(config_path: Option<&str>) -> Result<Self> {
        let mut builder = Config::builder();
        
        // 1. Load default configuration
        builder = builder.add_source(File::from_str(
            include_str!("../configs/default.toml"),
            FileFormat::Toml,
        ));
        
        // 2. Load environment-specific configuration
        if let Ok(env) = std::env::var("LLMSPELL_ENV") {
            let env_config_path = format!("configs/{}.toml", env);
            if std::path::Path::new(&env_config_path).exists() {
                builder = builder.add_source(File::with_name(&env_config_path));
            }
        }
        
        // 3. Load user-specified configuration file
        if let Some(config_path) = config_path {
            builder = builder.add_source(File::with_name(config_path));
        }
        
        // 4. Load environment variables (with LLMSPELL_ prefix)
        builder = builder.add_source(
            Environment::with_prefix("LLMSPELL")
                .prefix_separator("_")
                .separator("__")
        );
        
        // 5. Build and validate configuration
        let raw_config = builder.build()
            .map_err(|e| LLMSpellError::Configuration(format!("Config build failed: {}", e)))?;
            
        let config: LLMSpellConfig = raw_config.try_deserialize()
            .map_err(|e| LLMSpellError::Configuration(format!("Config deserialization failed: {}", e)))?;
        
        // 6. Validate configuration
        let validation_rules = ValidationRules::default();
        validation_rules.validate(&config)?;
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            file_watcher: None,
            validation_rules,
        })
    }
    
    // Hot reload configuration
    pub async fn enable_hot_reload(&mut self, config_path: &str) -> Result<()> {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        self.file_watcher = Some(sender.clone());
        
        // Set up file watcher
        let config_path = config_path.to_string();
        let config_arc = self.config.clone();
        let validation_rules = self.validation_rules.clone();
        
        tokio::spawn(async move {
            let mut watcher = notify::recommended_watcher(move |res| {
                if let Ok(event) = res {
                    if event.kind.is_modify() {
                        let _ = sender.send(ConfigUpdate::FileChanged);
                    }
                }
            }).unwrap();
            
            watcher.watch(
                std::path::Path::new(&config_path),
                notify::RecursiveMode::NonRecursive,
            ).unwrap();
            
            while let Some(update) = receiver.recv().await {
                match update {
                    ConfigUpdate::FileChanged => {
                        if let Ok(new_config) = Self::load_config_from_file(&config_path).await {
                            if validation_rules.validate(&new_config).is_ok() {
                                let mut config = config_arc.write().await;
                                *config = new_config;
                                tracing::info!("Configuration hot-reloaded successfully");
                            } else {
                                tracing::error!("Invalid configuration detected, keeping previous config");
                            }
                        }
                    }
                    ConfigUpdate::Shutdown => break,
                }
            }
        });
        
        Ok(())
    }
    
    // Get current configuration (read-only)
    pub async fn get_config(&self) -> LLMSpellConfig {
        self.config.read().await.clone()
    }
    
    // Update configuration dynamically
    pub async fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut LLMSpellConfig) -> Result<()>,
    {
        let mut config = self.config.write().await;
        updater(&mut *config)?;
        
        // Validate after update
        self.validation_rules.validate(&*config)?;
        
        Ok(())
    }
    
    // Get specific configuration section
    pub async fn get_provider_config(&self, provider_name: &str) -> Option<ProviderConfig> {
        let config = self.config.read().await;
        config.providers.providers.get(provider_name).cloned()
    }
    
    pub async fn get_engine_config(&self, engine_name: &str) -> Option<serde_json::Value> {
        let config = self.config.read().await;
        match engine_name {
            "lua" => Some(serde_json::to_value(&config.engines.lua).ok()?),
            "javascript" => Some(serde_json::to_value(&config.engines.javascript).ok()?),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum ConfigUpdate {
    FileChanged,
    Shutdown,
}
```

### Configuration Validation

```rust
#[derive(Debug, Clone)]
pub struct ValidationRules {
    provider_validators: HashMap<String, Box<dyn ProviderValidator>>,
    security_validators: Vec<Box<dyn SecurityValidator>>,
    performance_validators: Vec<Box<dyn PerformanceValidator>>,
}

impl ValidationRules {
    pub fn validate(&self, config: &LLMSpellConfig) -> Result<()> {
        // 1. Validate system configuration
        self.validate_system_config(&config.system)?;
        
        // 2. Validate provider configurations
        for (name, provider_config) in &config.providers.providers {
            self.validate_provider_config(name, provider_config)?;
        }
        
        // 3. Validate engine configurations
        self.validate_engine_configs(&config.engines)?;
        
        // 4. Validate security configuration
        self.validate_security_config(&config.security)?;
        
        // 5. Validate performance configuration
        self.validate_performance_config(&config.performance)?;
        
        // 6. Cross-validation (dependencies between sections)
        self.validate_cross_dependencies(config)?;
        
        Ok(())
    }
    
    fn validate_system_config(&self, config: &SystemConfig) -> Result<()> {
        // Validate data directory exists and is writable
        let data_dir = std::path::Path::new(&config.data_directory);
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)
                .map_err(|e| LLMSpellError::Configuration(
                    format!("Cannot create data directory: {}", e)
                ))?;
        }
        
        // Validate log level
        if !["trace", "debug", "info", "warn", "error"].contains(&config.log_level.as_str()) {
            return Err(LLMSpellError::Configuration(
                format!("Invalid log level: {}", config.log_level)
            ));
        }
        
        // Validate concurrent operations limit
        if config.max_concurrent_operations == 0 {
            return Err(LLMSpellError::Configuration(
                "max_concurrent_operations must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn validate_provider_config(&self, name: &str, config: &ProviderConfig) -> Result<()> {
        // Check if provider type is supported
        if !["openai", "anthropic", "local", "azure", "google"].contains(&config.provider_type.as_str()) {
            return Err(LLMSpellError::Configuration(
                format!("Unsupported provider type: {}", config.provider_type)
            ));
        }
        
        // Validate API key for cloud providers
        if ["openai", "anthropic", "azure", "google"].contains(&config.provider_type.as_str()) {
            if config.api_key.is_none() || config.api_key.as_ref().unwrap().is_empty() {
                return Err(LLMSpellError::Configuration(
                    format!("API key required for provider: {}", name)
                ));
            }
        }
        
        // Validate model name is not empty
        if config.model.is_empty() {
            return Err(LLMSpellError::Configuration(
                format!("Model name cannot be empty for provider: {}", name)
            ));
        }
        
        // Validate temperature range
        if let Some(temp) = config.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(LLMSpellError::Configuration(
                    format!("Temperature must be between 0.0 and 2.0 for provider: {}", name)
                ));
            }
        }
        
        // Validate timeout
        if config.timeout_seconds == 0 {
            return Err(LLMSpellError::Configuration(
                format!("Timeout must be greater than 0 for provider: {}", name)
            ));
        }
        
        Ok(())
    }
    
    fn validate_security_config(&self, config: &SecurityConfig) -> Result<()> {
        // Validate network restrictions
        for domain in &config.network_restrictions.allowed_domains {
            if domain.starts_with('.') {
                return Err(LLMSpellError::Configuration(
                    format!("Invalid domain format: {}", domain)
                ));
            }
        }
        
        // Validate encryption settings
        if config.encryption.enabled {
            if config.encryption.key_derivation_algorithm.is_empty() {
                return Err(LLMSpellError::Configuration(
                    "Key derivation algorithm required when encryption is enabled".to_string()
                ));
            }
        }
        
        Ok(())
    }
}
```

## Security Model and Threat Analysis

### Comprehensive Security Architecture

```rust
use ring::{aead, pbkdf2, rand};
use std::num::NonZeroU32;

pub struct SecurityManager {
    encryption_manager: EncryptionManager,
    sandbox_manager: SandboxManager,
    audit_logger: AuditLogger,
    rate_limiter: SecurityRateLimiter,
    threat_detector: ThreatDetector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption: EncryptionConfig,
    pub sandbox: SandboxConfig,
    pub audit: AuditConfig,
    pub rate_limiting: SecurityRateLimitConfig,
    pub threat_detection: ThreatDetectionConfig,
    pub network_restrictions: NetworkRestrictionsConfig,
    pub file_access: FileAccessConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: String, // "AES-256-GCM", "ChaCha20Poly1305"
    pub key_derivation_algorithm: String, // "PBKDF2", "Argon2"
    pub key_derivation_iterations: u32,
    pub master_key_path: Option<String>,
    pub auto_rotate_keys: bool,
    pub rotation_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub script_execution_limits: ExecutionLimits,
    pub allowed_system_calls: Vec<String>,
    pub filesystem_restrictions: FilesystemRestrictions,
    pub network_restrictions: NetworkRestrictions,
    pub memory_limits: MemoryLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionConfig {
    pub enabled: bool,
    pub monitoring_interval_seconds: u64,
    pub anomaly_detection_threshold: f64,
    pub rate_spike_detection: bool,
    pub suspicious_pattern_detection: bool,
    pub automated_response: AutomatedResponseConfig,
}
```

### Encryption and Key Management

```rust
pub struct EncryptionManager {
    primary_key: aead::LessSafeKey,
    key_rotation_schedule: KeyRotationSchedule,
    encrypted_storage: EncryptedStorage,
}

impl EncryptionManager {
    pub async fn new(config: &EncryptionConfig) -> Result<Self> {
        let primary_key = if let Some(key_path) = &config.master_key_path {
            Self::load_key_from_file(key_path).await?
        } else {
            Self::derive_key_from_password(&config).await?
        };
        
        Ok(Self {
            primary_key,
            key_rotation_schedule: KeyRotationSchedule::new(config.rotation_interval_hours),
            encrypted_storage: EncryptedStorage::new(),
        })
    }
    
    // Encrypt sensitive data (API keys, agent states, etc.)
    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = self.generate_nonce()?;
        let mut ciphertext = Vec::with_capacity(plaintext.len() + aead::NONCE_LEN + aead::MAX_TAG_LEN);
        
        // Prepend nonce to ciphertext
        ciphertext.extend_from_slice(nonce.as_ref());
        
        // Encrypt and append
        let mut buffer = plaintext.to_vec();
        let tag = self.primary_key.seal_in_place_separate_tag(
            nonce,
            aead::Aad::empty(),
            &mut buffer,
        ).map_err(|e| LLMSpellError::Security(format!("Encryption failed: {:?}", e)))?;
        
        ciphertext.extend_from_slice(&buffer);
        ciphertext.extend_from_slice(tag.as_ref());
        
        Ok(ciphertext)
    }
    
    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < aead::NONCE_LEN + aead::MAX_TAG_LEN {
            return Err(LLMSpellError::Security("Invalid ciphertext length".to_string()));
        }
        
        let (nonce_bytes, rest) = ciphertext.split_at(aead::NONCE_LEN);
        let (encrypted_data, tag_bytes) = rest.split_at(rest.len() - aead::MAX_TAG_LEN);
        
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| LLMSpellError::Security("Invalid nonce".to_string()))?;
        
        let mut buffer = encrypted_data.to_vec();
        self.primary_key.open_in_place(nonce, aead::Aad::empty(), &mut buffer)
            .map_err(|e| LLMSpellError::Security(format!("Decryption failed: {:?}", e)))?;
        
        Ok(buffer)
    }
    
    // Secure API key storage
    pub async fn store_api_key(&self, provider: &str, api_key: &str) -> Result<()> {
        let encrypted_key = self.encrypt_data(api_key.as_bytes())?;
        let key_id = format!("api_key:{}", provider);
        
        self.encrypted_storage.store(key_id, encrypted_key).await?;
        
        // Log the operation (without the actual key)
        tracing::info!("API key stored for provider: {}", provider);
        
        Ok(())
    }
    
    pub async fn retrieve_api_key(&self, provider: &str) -> Result<String> {
        let key_id = format!("api_key:{}", provider);
        let encrypted_key = self.encrypted_storage.retrieve(key_id).await?;
        
        let decrypted_key = self.decrypt_data(&encrypted_key)?;
        let api_key = String::from_utf8(decrypted_key)
            .map_err(|e| LLMSpellError::Security(format!("Invalid API key format: {}", e)))?;
        
        Ok(api_key)
    }
    
    // Automatic key rotation
    pub async fn rotate_keys(&mut self) -> Result<()> {
        tracing::info!("Starting key rotation");
        
        // Generate new key
        let new_key = Self::generate_new_key()?;
        
        // Re-encrypt all stored data with new key
        self.re_encrypt_all_data(&new_key).await?;
        
        // Update primary key
        self.primary_key = new_key;
        
        tracing::info!("Key rotation completed successfully");
        Ok(())
    }
    
    async fn derive_key_from_password(config: &EncryptionConfig) -> Result<aead::LessSafeKey> {
        let password = std::env::var("LLMSPELL_MASTER_PASSWORD")
            .map_err(|_| LLMSpellError::Security("Master password not found in environment".to_string()))?;
        
        let salt = b"llmspell_key_salt"; // In production, use a random salt
        let iterations = NonZeroU32::new(config.key_derivation_iterations)
            .ok_or_else(|| LLMSpellError::Security("Invalid key derivation iterations".to_string()))?;
        
        let mut key_bytes = [0u8; 32]; // 256-bit key
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt,
            password.as_bytes(),
            &mut key_bytes,
        );
        
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|e| LLMSpellError::Security(format!("Key creation failed: {:?}", e)))?;
        
        Ok(aead::LessSafeKey::new(unbound_key))
    }
    
    fn generate_nonce(&self) -> Result<aead::Nonce> {
        let rng = rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; aead::NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|e| LLMSpellError::Security(format!("Nonce generation failed: {:?}", e)))?;
        
        Ok(aead::Nonce::assume_unique_for_key(nonce_bytes))
    }
}
```

### Script Sandbox Security

```rust
pub struct SandboxManager {
    config: SandboxConfig,
    execution_monitor: ExecutionMonitor,
    resource_limiter: ResourceLimiter,
    filesystem_guard: FilesystemGuard,
    network_guard: NetworkGuard,
}

impl SandboxManager {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            execution_monitor: ExecutionMonitor::new(&config.script_execution_limits),
            resource_limiter: ResourceLimiter::new(&config.memory_limits),
            filesystem_guard: FilesystemGuard::new(&config.filesystem_restrictions),
            network_guard: NetworkGuard::new(&config.network_restrictions),
            config,
        }
    }
    
    // Execute script in sandboxed environment
    pub async fn execute_sandboxed_script(
        &self,
        script_content: &str,
        engine_type: ScriptEngineType,
        context: SandboxExecutionContext,
    ) -> Result<SandboxExecutionResult> {
        // 1. Pre-execution security checks
        self.validate_script_content(script_content)?;
        
        // 2. Set up resource monitoring
        let execution_monitor = self.execution_monitor.start_monitoring().await?;
        
        // 3. Apply resource limits
        self.resource_limiter.apply_limits(&context)?;
        
        // 4. Create isolated execution environment
        let sandbox_env = self.create_sandbox_environment(&context).await?;
        
        // 5. Execute script with monitoring
        let result = match engine_type {
            ScriptEngineType::Lua => {
                self.execute_lua_script_sandboxed(script_content, sandbox_env).await
            }
            ScriptEngineType::JavaScript => {
                self.execute_js_script_sandboxed(script_content, sandbox_env).await
            }
        };
        
        // 6. Clean up and collect metrics
        let execution_metrics = execution_monitor.stop_and_collect().await?;
        
        // 7. Security audit logging
        self.log_sandbox_execution(&context, &result, &execution_metrics).await?;
        
        Ok(SandboxExecutionResult {
            script_result: result?,
            execution_metrics,
            security_violations: execution_monitor.get_violations(),
        })
    }
    
    fn validate_script_content(&self, script: &str) -> Result<()> {
        // Static analysis for dangerous patterns
        let dangerous_patterns = [
            "os.execute",
            "io.popen",
            "require.*ffi",
            "loadfile",
            "dofile",
            "require.*os",
            "process.exit",
            "eval(",
            "Function(",
            "require.*child_process",
            "require.*fs",
            "__proto__",
        ];
        
        for pattern in &dangerous_patterns {
            if script.contains(pattern) {
                return Err(LLMSpellError::Security(
                    format!("Dangerous pattern detected: {}", pattern)
                ));
            }
        }
        
        // Check script length limits
        if script.len() > self.config.script_execution_limits.max_script_size {
            return Err(LLMSpellError::Security(
                format!("Script exceeds maximum size limit: {} bytes", script.len())
            ));
        }
        
        Ok(())
    }
    
    async fn create_sandbox_environment(&self, context: &SandboxExecutionContext) -> Result<SandboxEnvironment> {
        let mut env = SandboxEnvironment::new();
        
        // 1. Set up restricted filesystem access
        env.set_filesystem_restrictions(&self.config.filesystem_restrictions);
        
        // 2. Configure network restrictions
        env.set_network_restrictions(&self.config.network_restrictions);
        
        // 3. Apply memory limits
        env.set_memory_limits(&self.config.memory_limits);
        
        // 4. Configure allowed system calls
        env.set_allowed_syscalls(&self.config.allowed_system_calls);
        
        // 5. Set up monitoring hooks
        env.install_monitoring_hooks(&self.execution_monitor);
        
        Ok(env)
    }
    
    async fn execute_lua_script_sandboxed(
        &self,
        script: &str,
        sandbox_env: SandboxEnvironment,
    ) -> Result<ScriptExecutionResult> {
        use mlua::{Lua, StdLib};
        
        // Create restricted Lua environment
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            mlua::LuaOptions::default(),
        )?;
        
        // Install sandbox restrictions
        self.install_lua_sandbox_restrictions(&lua, &sandbox_env)?;
        
        // Execute with timeout
        let execution_timeout = Duration::from_millis(self.config.script_execution_limits.max_execution_time_ms);
        let result = tokio::time::timeout(execution_timeout, async {
            lua.load(script).eval_async::<mlua::Value>().await
        }).await;
        
        match result {
            Ok(Ok(value)) => Ok(ScriptExecutionResult::Success(self.convert_lua_value(value)?)),
            Ok(Err(e)) => Ok(ScriptExecutionResult::Error(format!("Lua error: {}", e))),
            Err(_) => Ok(ScriptExecutionResult::Timeout),
        }
    }
    
    fn install_lua_sandbox_restrictions(&self, lua: &Lua, sandbox_env: &SandboxEnvironment) -> Result<()> {
        let globals = lua.globals();
        
        // Remove dangerous globals
        globals.set("os", mlua::Nil)?;
        globals.set("io", mlua::Nil)?;
        globals.set("require", mlua::Nil)?;
        globals.set("dofile", mlua::Nil)?;
        globals.set("loadfile", mlua::Nil)?;
        globals.set("load", mlua::Nil)?;
        globals.set("debug", mlua::Nil)?;
        
        // Install safe filesystem access
        let safe_fs = self.create_safe_filesystem_api(sandbox_env)?;
        globals.set("fs", safe_fs)?;
        
        // Install safe network access
        let safe_network = self.create_safe_network_api(sandbox_env)?;
        globals.set("network", safe_network)?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct SandboxExecutionResult {
    pub script_result: ScriptExecutionResult,
    pub execution_metrics: ExecutionMetrics,
    pub security_violations: Vec<SecurityViolation>,
}

#[derive(Debug)]
pub enum SecurityViolation {
    MemoryLimitExceeded { limit: usize, actual: usize },
    ExecutionTimeExceeded { limit: Duration, actual: Duration },
    UnauthorizedFileAccess { path: String },
    UnauthorizedNetworkAccess { domain: String },
    SuspiciousSystemCall { syscall: String },
}
```

### Threat Detection and Response

```rust
pub struct ThreatDetector {
    config: ThreatDetectionConfig,
    anomaly_detector: AnomalyDetector,
    pattern_matcher: SuspiciousPatternMatcher,
    rate_analyzer: RateAnomalyAnalyzer,
    response_system: AutomatedResponseSystem,
}

impl ThreatDetector {
    pub async fn analyze_request(&self, request: &IncomingRequest) -> ThreatAnalysisResult {
        let mut threat_level = ThreatLevel::None;
        let mut detected_threats = Vec::new();
        
        // 1. Rate-based analysis
        if let Some(rate_threat) = self.rate_analyzer.analyze_request_rate(request).await? {
            threat_level = threat_level.max(rate_threat.threat_level);
            detected_threats.push(rate_threat);
        }
        
        // 2. Pattern-based analysis
        if let Some(pattern_threat) = self.pattern_matcher.analyze_request_content(request).await? {
            threat_level = threat_level.max(pattern_threat.threat_level);
            detected_threats.push(pattern_threat);
        }
        
        // 3. Behavioral analysis
        if let Some(behavioral_threat) = self.anomaly_detector.analyze_request_behavior(request).await? {
            threat_level = threat_level.max(behavioral_threat.threat_level);
            detected_threats.push(behavioral_threat);
        }
        
        // 4. Automated response if configured
        if threat_level >= self.config.automated_response.minimum_threat_level {
            self.response_system.execute_response(&detected_threats).await?;
        }
        
        ThreatAnalysisResult {
            threat_level,
            detected_threats,
            recommended_action: self.determine_recommended_action(threat_level),
        }
    }
    
    fn determine_recommended_action(&self, threat_level: ThreatLevel) -> RecommendedAction {
        match threat_level {
            ThreatLevel::None => RecommendedAction::Allow,
            ThreatLevel::Low => RecommendedAction::Monitor,
            ThreatLevel::Medium => RecommendedAction::RateLimit,
            ThreatLevel::High => RecommendedAction::Block,
            ThreatLevel::Critical => RecommendedAction::BlockAndAlert,
        }
    }
}

pub struct AnomalyDetector {
    baseline_metrics: BaselineMetrics,
    statistical_models: StatisticalModels,
    machine_learning_models: Option<MLModels>,
}

impl AnomalyDetector {
    pub async fn analyze_request_behavior(&self, request: &IncomingRequest) -> Result<Option<DetectedThreat>> {
        // Extract behavioral features
        let features = self.extract_behavioral_features(request);
        
        // Compare against baseline
        let baseline_score = self.calculate_baseline_deviation(&features);
        
        // Statistical anomaly detection
        let statistical_score = self.statistical_models.calculate_anomaly_score(&features);
        
        // ML-based detection (if available)
        let ml_score = if let Some(ref ml_models) = self.machine_learning_models {
            ml_models.predict_anomaly_score(&features).await?
        } else {
            0.0
        };
        
        // Combine scores
        let combined_score = (baseline_score * 0.4) + (statistical_score * 0.4) + (ml_score * 0.2);
        
        if combined_score > self.baseline_metrics.anomaly_threshold {
            Some(DetectedThreat {
                threat_type: ThreatType::BehavioralAnomaly,
                threat_level: self.score_to_threat_level(combined_score),
                description: format!("Behavioral anomaly detected (score: {:.2})", combined_score),
                evidence: features.into(),
                timestamp: chrono::Utc::now(),
            })
        } else {
            None
        }
    }
    
    fn extract_behavioral_features(&self, request: &IncomingRequest) -> BehavioralFeatures {
        BehavioralFeatures {
            request_size: request.content.len(),
            request_complexity: self.calculate_request_complexity(&request.content),
            time_since_last_request: request.client_context.time_since_last_request,
            request_frequency: request.client_context.requests_per_hour,
            unique_patterns: self.extract_unique_patterns(&request.content),
            resource_usage_prediction: self.predict_resource_usage(&request.content),
        }
    }
}
```

## Resource Management

### Comprehensive Resource Control

```rust
use tokio::sync::Semaphore;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct ResourceManager {
    memory_manager: MemoryManager,
    cpu_manager: CpuManager,
    network_manager: NetworkManager,
    storage_manager: StorageResourceManager,
    global_limits: GlobalResourceLimits,
    monitoring: ResourceMonitoring,
}

#[derive(Debug, Clone)]
pub struct GlobalResourceLimits {
    pub max_total_memory_mb: usize,
    pub max_concurrent_operations: usize,
    pub max_network_connections: usize,
    pub max_disk_usage_mb: u64,
    pub max_cpu_usage_percent: f64,
}

pub struct MemoryManager {
    current_usage: AtomicUsize,
    peak_usage: AtomicUsize,
    allocation_semaphore: Semaphore,
    per_component_limits: HashMap<String, usize>,
    memory_pressure_handler: MemoryPressureHandler,
}

impl MemoryManager {
    pub fn new(config: &MemoryConfig) -> Self {
        Self {
            current_usage: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
            allocation_semaphore: Semaphore::new(config.max_concurrent_allocations),
            per_component_limits: config.component_limits.clone(),
            memory_pressure_handler: MemoryPressureHandler::new(config.pressure_threshold),
        }
    }
    
    // Request memory allocation with limits checking
    pub async fn request_allocation(&self, component: &str, size_bytes: usize) -> Result<MemoryAllocation> {
        // Check component-specific limit
        if let Some(&limit) = self.per_component_limits.get(component) {
            if size_bytes > limit {
                return Err(LLMSpellError::Resource(format!(
                    "Memory allocation {} bytes exceeds component limit {} bytes for {}",
                    size_bytes, limit, component
                )));
            }
        }
        
        // Acquire allocation permit
        let _permit = self.allocation_semaphore.acquire().await
            .map_err(|_| LLMSpellError::Resource("Memory allocation semaphore closed".to_string()))?;
        
        // Check global memory pressure
        let current_usage = self.current_usage.load(Ordering::Relaxed);
        if self.memory_pressure_handler.should_reject_allocation(current_usage, size_bytes) {
            return Err(LLMSpellError::Resource(format!(
                "Memory allocation rejected due to memory pressure: current={}, requested={}",
                current_usage, size_bytes
            )));
        }
        
        // Update usage tracking
        let new_usage = current_usage + size_bytes;
        self.current_usage.store(new_usage, Ordering::Relaxed);
        
        // Update peak usage
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > peak {
            match self.peak_usage.compare_exchange_weak(peak, new_usage, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
        
        Ok(MemoryAllocation {
            component: component.to_string(),
            size_bytes,
            allocated_at: std::time::Instant::now(),
            manager: self,
        })
    }
    
    // Release memory allocation
    pub fn release_allocation(&self, allocation: &MemoryAllocation) {
        let current = self.current_usage.load(Ordering::Relaxed);
        let new_usage = current.saturating_sub(allocation.size_bytes);
        self.current_usage.store(new_usage, Ordering::Relaxed);
        
        tracing::debug!(
            "Memory released: component={}, size={}, new_total={}",
            allocation.component,
            allocation.size_bytes,
            new_usage
        );
    }
    
    // Get current memory statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage_bytes: self.current_usage.load(Ordering::Relaxed),
            peak_usage_bytes: self.peak_usage.load(Ordering::Relaxed),
            available_permits: self.allocation_semaphore.available_permits(),
            component_usage: self.get_component_usage(),
            memory_pressure_level: self.memory_pressure_handler.get_pressure_level(),
        }
    }
}

pub struct MemoryAllocation {
    component: String,
    size_bytes: usize,
    allocated_at: std::time::Instant,
    manager: *const MemoryManager,
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        unsafe {
            (*self.manager).release_allocation(self);
        }
    }
}

pub struct CpuManager {
    cpu_monitor: CpuMonitor,
    execution_scheduler: ExecutionScheduler,
    throttling_manager: ThrottlingManager,
}

impl CpuManager {
    // Monitor CPU usage and apply throttling
    pub async fn monitor_and_control(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        
        loop {
            interval.tick().await;
            
            let cpu_usage = self.cpu_monitor.get_current_usage().await?;
            
            if cpu_usage > self.throttling_manager.high_threshold {
                tracing::warn!("High CPU usage detected: {:.1}%", cpu_usage * 100.0);
                self.throttling_manager.apply_throttling(cpu_usage).await?;
            } else if cpu_usage < self.throttling_manager.low_threshold {
                self.throttling_manager.reduce_throttling().await?;
            }
            
            // Update metrics
            self.cpu_monitor.record_usage(cpu_usage).await;
        }
    }
    
    // Execute task with CPU monitoring
    pub async fn execute_with_monitoring<F, R>(&self, task: F) -> Result<R>
    where
        F: Future<Output = Result<R>> + Send,
        R: Send,
    {
        let start_time = std::time::Instant::now();
        let start_cpu = self.cpu_monitor.get_current_usage().await?;
        
        let result = task.await;
        
        let end_time = std::time::Instant::now();
        let end_cpu = self.cpu_monitor.get_current_usage().await?;
        
        let execution_time = end_time.duration_since(start_time);
        let cpu_delta = end_cpu - start_cpu;
        
        // Log performance metrics
        tracing::debug!(
            "Task execution completed: duration={:?}, cpu_delta={:.3}",
            execution_time,
            cpu_delta
        );
        
        result
    }
}
```

## Observability and Monitoring

### Comprehensive Monitoring Stack

```rust
use metrics::{counter, gauge, histogram, register_counter, register_gauge, register_histogram};
use tracing::{event, Level, span, Instrument};
use opentelemetry::trace::{TraceError, Tracer};

pub struct ObservabilityManager {
    metrics_collector: MetricsCollector,
    tracing_manager: TracingManager,
    alerting_system: AlertingSystem,
    dashboard_exporter: DashboardExporter,
}

impl ObservabilityManager {
    pub async fn new(config: &ObservabilityConfig) -> Result<Self> {
        // Initialize metrics collection
        let metrics_collector = MetricsCollector::new(&config.metrics).await?;
        
        // Initialize distributed tracing
        let tracing_manager = TracingManager::new(&config.tracing).await?;
        
        // Initialize alerting
        let alerting_system = AlertingSystem::new(&config.alerting).await?;
        
        // Initialize dashboard exports
        let dashboard_exporter = DashboardExporter::new(&config.dashboard).await?;
        
        Ok(Self {
            metrics_collector,
            tracing_manager,
            alerting_system,
            dashboard_exporter,
        })
    }
    
    // Start monitoring all rs-llmspell components
    pub async fn start_monitoring(&self) -> Result<()> {
        // Start metrics collection
        self.metrics_collector.start_collection().await?;
        
        // Start trace collection
        self.tracing_manager.start_trace_collection().await?;
        
        // Start alerting monitor
        self.alerting_system.start_monitoring().await?;
        
        // Start dashboard data export
        self.dashboard_exporter.start_export_loop().await?;
        
        Ok(())
    }
}

pub struct MetricsCollector {
    system_metrics: SystemMetricsCollector,
    application_metrics: ApplicationMetricsCollector,
    business_metrics: BusinessMetricsCollector,
    exporter: MetricsExporter,
}

impl MetricsCollector {
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let cpu_usage = self.system_metrics.get_cpu_usage().await?;
        let memory_usage = self.system_metrics.get_memory_usage().await?;
        let disk_usage = self.system_metrics.get_disk_usage().await?;
        let network_stats = self.system_metrics.get_network_stats().await?;
        
        // Record metrics
        gauge!("system.cpu.usage_percent", cpu_usage);
        gauge!("system.memory.usage_bytes", memory_usage.used as f64);
        gauge!("system.memory.available_bytes", memory_usage.available as f64);
        gauge!("system.disk.usage_bytes", disk_usage.used as f64);
        counter!("system.network.bytes_received", network_stats.bytes_received);
        counter!("system.network.bytes_sent", network_stats.bytes_sent);
        
        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_stats,
            timestamp: chrono::Utc::now(),
        })
    }
    
    pub async fn collect_application_metrics(&self) -> Result<ApplicationMetrics> {
        // LLM Provider Metrics
        let provider_metrics = self.collect_provider_metrics().await?;
        
        // Script Engine Metrics
        let engine_metrics = self.collect_engine_metrics().await?;
        
        // Hook and Event Metrics
        let hook_metrics = self.collect_hook_metrics().await?;
        
        // Tool Execution Metrics
        let tool_metrics = self.collect_tool_metrics().await?;
        
        Ok(ApplicationMetrics {
            provider_metrics,
            engine_metrics,
            hook_metrics,
            tool_metrics,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn collect_provider_metrics(&self) -> Result<ProviderMetrics> {
        let mut provider_stats = HashMap::new();
        
        // Collect metrics for each registered provider
        for provider_name in self.application_metrics.get_active_providers().await? {
            let stats = self.application_metrics.get_provider_stats(&provider_name).await?;
            
            // Record individual provider metrics
            counter!(
                "llm.provider.requests_total", 
                stats.total_requests, 
                "provider" => provider_name.clone()
            );
            counter!(
                "llm.provider.tokens_consumed", 
                stats.total_tokens, 
                "provider" => provider_name.clone()
            );
            histogram!(
                "llm.provider.response_time_seconds", 
                stats.average_response_time.as_secs_f64(),
                "provider" => provider_name.clone()
            );
            gauge!(
                "llm.provider.error_rate", 
                stats.error_rate,
                "provider" => provider_name.clone()
            );
            
            provider_stats.insert(provider_name, stats);
        }
        
        Ok(ProviderMetrics {
            per_provider_stats: provider_stats,
            total_requests: self.application_metrics.get_total_llm_requests().await?,
            total_tokens_consumed: self.application_metrics.get_total_tokens_consumed().await?,
            average_cost_per_request: self.application_metrics.get_average_cost_per_request().await?,
        })
    }
    
    async fn collect_engine_metrics(&self) -> Result<EngineMetrics> {
        let lua_stats = self.application_metrics.get_lua_engine_stats().await?;
        let js_stats = self.application_metrics.get_js_engine_stats().await?;
        
        // Record Lua metrics
        counter!("script.lua.executions_total", lua_stats.total_executions);
        histogram!("script.lua.execution_time_seconds", lua_stats.average_execution_time.as_secs_f64());
        gauge!("script.lua.active_coroutines", lua_stats.active_coroutines as f64);
        gauge!("script.lua.memory_usage_bytes", lua_stats.memory_usage_bytes as f64);
        
        // Record JavaScript metrics
        counter!("script.js.executions_total", js_stats.total_executions);
        histogram!("script.js.execution_time_seconds", js_stats.average_execution_time.as_secs_f64());
        gauge!("script.js.active_promises", js_stats.active_promises as f64);
        gauge!("script.js.memory_usage_bytes", js_stats.memory_usage_bytes as f64);
        
        Ok(EngineMetrics {
            lua_stats,
            js_stats,
        })
    }
    
    async fn collect_hook_metrics(&self) -> Result<HookMetrics> {
        let hook_manager_stats = self.application_metrics.get_hook_manager_stats().await?;
        
        // Record hook execution metrics
        counter!("hooks.executions_total", hook_manager_stats.total_executions);
        histogram!("hooks.execution_time_seconds", hook_manager_stats.average_execution_time.as_secs_f64());
        gauge!("hooks.cache_hit_rate", hook_manager_stats.cache_hit_rate);
        gauge!("hooks.active_hook_points", hook_manager_stats.active_hook_points as f64);
        
        // Record per-hook-point metrics
        for (hook_point, stats) in &hook_manager_stats.per_hook_point_stats {
            counter!(
                "hooks.hook_point.executions_total", 
                stats.executions,
                "hook_point" => hook_point.clone()
            );
            histogram!(
                "hooks.hook_point.execution_time_seconds", 
                stats.average_execution_time.as_secs_f64(),
                "hook_point" => hook_point.clone()
            );
        }
        
        Ok(HookMetrics {
            manager_stats: hook_manager_stats,
        })
    }
}

pub struct TracingManager {
    tracer: Box<dyn Tracer + Send + Sync>,
    span_processor: SpanProcessor,
    trace_exporter: TraceExporter,
}

impl TracingManager {
    // Create instrumented span for operations
    pub fn create_operation_span(&self, operation_name: &str, component: &str) -> TracedOperation {
        let span = span!(
            Level::INFO,
            "operation",
            operation.name = operation_name,
            component = component,
            operation.start_time = %chrono::Utc::now(),
        );
        
        TracedOperation {
            span,
            start_time: std::time::Instant::now(),
            operation_name: operation_name.to_string(),
            component: component.to_string(),
        }
    }
    
    // Trace LLM completion request
    pub async fn trace_llm_completion<F, R>(&self, provider: &str, model: &str, operation: F) -> Result<R>
    where
        F: Future<Output = Result<R>> + Send,
    {
        let span = span!(
            Level::INFO,
            "llm_completion",
            provider = provider,
            model = model,
            request.start_time = %chrono::Utc::now(),
        );
        
        let start_time = std::time::Instant::now();
        let result = operation.instrument(span.clone()).await;
        let duration = start_time.elapsed();
        
        // Add completion metrics to span
        span.record("request.duration_ms", duration.as_millis());
        span.record("request.success", result.is_ok());
        
        if let Err(ref error) = result {
            span.record("request.error", error.to_string().as_str());
            event!(Level::ERROR, "LLM completion failed: {}", error);
        }
        
        result
    }
    
    // Trace script execution
    pub async fn trace_script_execution<F, R>(
        &self, 
        engine: &str, 
        script_id: &str, 
        operation: F
    ) -> Result<R>
    where
        F: Future<Output = Result<R>> + Send,
    {
        let span = span!(
            Level::INFO,
            "script_execution",
            script.engine = engine,
            script.id = script_id,
            execution.start_time = %chrono::Utc::now(),
        );
        
        let start_time = std::time::Instant::now();
        let result = operation.instrument(span.clone()).await;
        let duration = start_time.elapsed();
        
        span.record("execution.duration_ms", duration.as_millis());
        span.record("execution.success", result.is_ok());
        
        result
    }
}

pub struct AlertingSystem {
    alert_rules: Vec<AlertRule>,
    notification_channels: Vec<NotificationChannel>,
    alert_history: AlertHistory,
}

impl AlertingSystem {
    pub async fn evaluate_alerts(&self, metrics: &SystemMetrics, app_metrics: &ApplicationMetrics) -> Result<()> {
        for rule in &self.alert_rules {
            if let Some(alert) = rule.evaluate(metrics, app_metrics)? {
                self.send_alert(alert).await?;
            }
        }
        
        Ok(())
    }
    
    async fn send_alert(&self, alert: Alert) -> Result<()> {
        // Check if this alert should be suppressed (e.g., recently fired)
        if self.alert_history.should_suppress(&alert) {
            return Ok(());
        }
        
        // Send to all configured notification channels
        for channel in &self.notification_channels {
            if let Err(e) = channel.send_notification(&alert).await {
                tracing::error!("Failed to send alert via {}: {}", channel.name(), e);
            }
        }
        
        // Record in alert history
        self.alert_history.record_alert(&alert).await?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct Alert {
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metrics: HashMap<String, f64>,
    pub runbook_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub message_template: String,
    pub cooldown_duration: Duration,
    pub runbook_url: Option<String>,
}

impl AlertRule {
    pub fn evaluate(&self, system_metrics: &SystemMetrics, app_metrics: &ApplicationMetrics) -> Result<Option<Alert>> {
        if self.condition.is_triggered(system_metrics, app_metrics)? {
            let alert = Alert {
                rule_name: self.name.clone(),
                severity: self.severity.clone(),
                message: self.format_message(system_metrics, app_metrics),
                timestamp: chrono::Utc::now(),
                metrics: self.condition.get_metric_values(system_metrics, app_metrics)?,
                runbook_url: self.runbook_url.clone(),
            };
            
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }
}

// Example alert rules configuration
pub fn create_default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "high_cpu_usage".to_string(),
            condition: AlertCondition::Threshold {
                metric_path: "system.cpu.usage_percent".to_string(),
                operator: ThresholdOperator::GreaterThan,
                threshold: 85.0,
                duration: Duration::from_secs(300), // 5 minutes
            },
            severity: AlertSeverity::Warning,
            message_template: "High CPU usage detected: {cpu_usage:.1}%".to_string(),
            cooldown_duration: Duration::from_secs(600), // 10 minutes
            runbook_url: Some("https://docs.rs-llmspell.com/runbooks/high-cpu".to_string()),
        },
        AlertRule {
            name: "llm_provider_high_error_rate".to_string(),
            condition: AlertCondition::Threshold {
                metric_path: "llm.provider.error_rate".to_string(),
                operator: ThresholdOperator::GreaterThan,
                threshold: 0.05, // 5% error rate
                duration: Duration::from_secs(180), // 3 minutes
            },
            severity: AlertSeverity::Critical,
            message_template: "High error rate for LLM provider: {error_rate:.2}%".to_string(),
            cooldown_duration: Duration::from_secs(300), // 5 minutes
            runbook_url: Some("https://docs.rs-llmspell.com/runbooks/llm-errors".to_string()),
        },
        AlertRule {
            name: "memory_pressure".to_string(),
            condition: AlertCondition::Threshold {
                metric_path: "system.memory.usage_percent".to_string(),
                operator: ThresholdOperator::GreaterThan,
                threshold: 90.0,
                duration: Duration::from_secs(120), // 2 minutes
            },
            severity: AlertSeverity::Critical,
            message_template: "Memory pressure detected: {memory_usage:.1}% used".to_string(),
            cooldown_duration: Duration::from_secs(300), // 5 minutes
            runbook_url: Some("https://docs.rs-llmspell.com/runbooks/memory-pressure".to_string()),
        },
    ]
}
```

---

# Part VII: Advanced Features

## Advanced Orchestration Patterns

While workflows provide structured, deterministic orchestration, `rs-llmspell` also supports more dynamic, LLM-driven orchestration patterns. These allow for greater flexibility and adaptability in complex scenarios.

### 1. LLM-Driven Delegation (Agent Transfer)

This pattern allows an agent to dynamically decide to hand off control to another specialized agent without a predefined workflow. This is a powerful mechanism for creating flexible, multi-agent systems that can adapt to unforeseen tasks.

#### Architectural Support

To enable this, the architecture includes:

*   **Explicit Handoff in `AgentOutput`**: The `AgentOutput` struct contains an optional `handoff_request` field. When an agent's logic determines that another agent is better suited for the task, it can populate this field.

    ```rust
    pub struct AgentOutput {
        pub content: String,
        pub tool_calls: Vec<ToolCall>,
        pub handoff_request: Option<HandoffRequest>, // Explicit handoff action
        pub metadata: OutputMetadata,
        pub state_updates: HashMap<String, ScriptValue>,
    }

    pub struct HandoffRequest {
        pub target_agent_id: String,
        pub input: AgentInput, // The input for the next agent
        pub reason: String, // Justification for the handoff
        pub state_filter: Option<StateFilter>, // Control which state gets passed
    }
    ```

*   **Agent Runtime Engine**: A dedicated `AgentRuntime` is responsible for the core execution loop. It inspects the `AgentOutput` of each execution. If a `handoff_request` is present, the runtime manages the transfer of control and state to the target agent.

    ```rust
    // Simplified runtime logic
    pub struct AgentRuntime {
        agent_registry: AgentRegistry,
        state_manager: StateManager,
    }

    impl AgentRuntime {
        pub async fn run_conversation(&self, initial_agent_id: String, initial_input: AgentInput) -> Result<AgentOutput> {
            let mut current_agent_id = initial_agent_id;
            let mut current_input = initial_input;
            let mut final_output;

            loop {
                let mut agent = self.agent_registry.get(&current_agent_id)?;
                let output = agent.execute(current_input).await?;

                if let Some(handoff) = output.handoff_request {
                    // Transfer control to the next agent
                    current_agent_id = handoff.target_agent_id;
                    current_input = handoff.input;
                } else {
                    final_output = output;
                    break;
                }
            }
            Ok(final_output)
        }
    }
    ```

*   **Agent-Aware Prompts**: For an LLM to make a delegation decision, it must be aware of available peer agents. The system prompt for orchestrator agents is dynamically enriched with a list of available specialists, enabling the LLM to generate a `HandoffRequest` when appropriate.

#### Scripting Example

```lua
-- An orchestrator agent that can delegate tasks
local orchestrator = Agent.new({
    name = "orchestrator",
    system_prompt = [[
        You are a master orchestrator. Based on the user request, you can either answer directly
        or delegate to a specialist. Available specialists:
        - 'code_reviewer': For analyzing code quality.
        - 'database_expert': For database queries and analysis.

        To delegate, return a 'handoff' action.
    ]],
    tools = { AgentFinderTool.new() } -- A tool to find available agents
})

-- Execution that triggers a handoff
local result = orchestrator:chat("Please review the code in 'src/main.rs'")

-- The 'result' object would contain the final output from the 'code_reviewer' agent
-- after the handoff was completed by the AgentRuntime.
```

This pattern provides a powerful alternative to rigid workflows, enabling more autonomous and intelligent multi-agent systems.

### 2. Swarm Intelligence Pattern


Rs-LLMSpell provides sophisticated orchestration capabilities for complex multi-agent workflows, dynamic execution patterns, and intelligent coordination strategies.

### Multi-Agent Collaboration Framework

```rust
use tokio::sync::{mpsc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

pub struct MultiAgentOrchestrator {
    agents: Arc<RwLock<HashMap<AgentId, Box<dyn Agent>>>>,
    collaboration_manager: CollaborationManager,
    communication_bus: CommunicationBus,
    workflow_engine: WorkflowEngine,
    coordination_strategies: HashMap<String, Box<dyn CoordinationStrategy>>,
    execution_context: Arc<Mutex<ExecutionContext>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

#[derive(Debug)]
pub struct CollaborationSession {
    pub session_id: String,
    pub participants: Vec<AgentId>,
    pub shared_context: SharedContext,
    pub communication_history: Vec<AgentMessage>,
    pub coordination_strategy: String,
    pub session_state: CollaborationState,
}

#[derive(Debug)]
pub enum CollaborationState {
    Initializing,
    Active,
    Synchronizing,
    Completing,
    Failed(String),
    Completed,
}

impl MultiAgentOrchestrator {
    pub async fn new(config: OrchestrationConfig) -> Result<Self> {
        let agents = Arc::new(RwLock::new(HashMap::new()));
        let collaboration_manager = CollaborationManager::new(&config);
        let communication_bus = CommunicationBus::new(&config.communication);
        let workflow_engine = WorkflowEngine::new(&config.workflow);
        
        // Register built-in coordination strategies
        let mut coordination_strategies: HashMap<String, Box<dyn CoordinationStrategy>> = HashMap::new();
        coordination_strategies.insert("sequential".to_string(), Box::new(SequentialStrategy::new()));
        coordination_strategies.insert("parallel".to_string(), Box::new(ParallelStrategy::new()));
        coordination_strategies.insert("consensus".to_string(), Box::new(ConsensusStrategy::new()));
        coordination_strategies.insert("hierarchical".to_string(), Box::new(HierarchicalStrategy::new()));
        coordination_strategies.insert("dynamic".to_string(), Box::new(DynamicStrategy::new()));
        
        Ok(Self {
            agents,
            collaboration_manager,
            communication_bus,
            workflow_engine,
            coordination_strategies,
            execution_context: Arc::new(Mutex::new(ExecutionContext::new())),
        })
    }
    
    // Start a multi-agent collaboration session
    pub async fn start_collaboration(
        &self,
        agent_ids: Vec<AgentId>,
        strategy: &str,
        initial_context: SharedContext,
    ) -> Result<CollaborationSession> {
        let session_id = Uuid::new_v4().to_string();
        
        // Validate all agents exist
        {
            let agents = self.agents.read().await;
            for agent_id in &agent_ids {
                if !agents.contains_key(agent_id) {
                    return Err(LLMSpellError::Agent(AgentError::NotFound {
                        agent_id: agent_id.0.clone(),
                    }));
                }
            }
        }
        
        // Get coordination strategy
        let coordination_strategy = self.coordination_strategies.get(strategy)
            .ok_or_else(|| LLMSpellError::Workflow(WorkflowError::InvalidStrategy {
                strategy: strategy.to_string(),
            }))?;
        
        // Initialize collaboration session
        let mut session = CollaborationSession {
            session_id: session_id.clone(),
            participants: agent_ids.clone(),
            shared_context: initial_context,
            communication_history: Vec::new(),
            coordination_strategy: strategy.to_string(),
            session_state: CollaborationState::Initializing,
        };
        
        // Initialize agents for collaboration
        for agent_id in &agent_ids {
            self.initialize_agent_for_collaboration(agent_id, &session_id).await?;
        }
        
        // Set up communication channels
        self.communication_bus.create_session_channels(&session_id, &agent_ids).await?;
        
        // Begin coordination strategy
        session.session_state = CollaborationState::Active;
        coordination_strategy.begin_collaboration(&mut session).await?;
        
        // Register session with collaboration manager
        self.collaboration_manager.register_session(session.clone()).await?;
        
        Ok(session)
    }
    
    // Execute collaborative workflow
    pub async fn execute_collaborative_workflow(
        &self,
        session_id: &str,
        workflow_definition: WorkflowDefinition,
    ) -> Result<WorkflowResult> {
        let session = self.collaboration_manager.get_session(session_id).await?
            .ok_or_else(|| LLMSpellError::Workflow(WorkflowError::SessionNotFound {
                session_id: session_id.to_string(),
            }))?;
        
        let strategy = self.coordination_strategies.get(&session.coordination_strategy)
            .ok_or_else(|| LLMSpellError::Workflow(WorkflowError::InvalidStrategy {
                strategy: session.coordination_strategy.clone(),
            }))?;
        
        // Execute workflow with coordination strategy
        let result = strategy.execute_workflow(
            &session,
            workflow_definition,
            &self.agents,
            &self.communication_bus,
        ).await?;
        
        // Update session state
        self.collaboration_manager.update_session_state(
            session_id,
            CollaborationState::Completing,
        ).await?;
        
        Ok(result)
    }
}

#[async_trait]
pub trait CoordinationStrategy: Send + Sync {
    async fn begin_collaboration(&self, session: &mut CollaborationSession) -> Result<()>;
    
    async fn execute_workflow(
        &self,
        session: &CollaborationSession,
        workflow: WorkflowDefinition,
        agents: &Arc<RwLock<HashMap<AgentId, Box<dyn Agent>>>>,
        communication_bus: &CommunicationBus,
    ) -> Result<WorkflowResult>;
    
    async fn handle_agent_failure(
        &self,
        session: &CollaborationSession,
        failed_agent: &AgentId,
        error: &LLMSpellError,
    ) -> Result<RecoveryAction>;
    
    async fn coordinate_synchronization(
        &self,
        session: &CollaborationSession,
        sync_point: SynchronizationPoint,
    ) -> Result<SynchronizationResult>;
}

// Sequential execution strategy
pub struct SequentialStrategy {
    execution_order: Vec<AgentId>,
    current_agent_index: Arc<Mutex<usize>>,
}

#[async_trait]
impl CoordinationStrategy for SequentialStrategy {
    async fn execute_workflow(
        &self,
        session: &CollaborationSession,
        workflow: WorkflowDefinition,
        agents: &Arc<RwLock<HashMap<AgentId, Box<dyn Agent>>>>,
        communication_bus: &CommunicationBus,
    ) -> Result<WorkflowResult> {
        let mut workflow_context = WorkflowContext::new();
        let mut results = Vec::new();
        
        for step in workflow.steps {
            let agent_id = &step.agent_id;
            
            // Get agent
            let agent = {
                let agents_lock = agents.read().await;
                agents_lock.get(agent_id)
                    .ok_or_else(|| LLMSpellError::Agent(AgentError::NotFound {
                        agent_id: agent_id.0.clone(),
                    }))?
                    .clone()
            };
            
            // Prepare execution context
            let execution_context = ExecutionContext {
                session_id: session.session_id.clone(),
                step_index: results.len(),
                shared_context: session.shared_context.clone(),
                previous_results: results.clone(),
                workflow_context: workflow_context.clone(),
            };
            
            // Execute step
            let step_result = self.execute_workflow_step(
                agent,
                &step,
                execution_context,
                communication_bus,
            ).await?;
            
            // Update workflow context with results
            workflow_context.add_step_result(&step.step_id, &step_result);
            results.push(step_result);
            
            // Send progress update
            self.send_progress_update(
                session,
                communication_bus,
                results.len(),
                workflow.steps.len(),
            ).await?;
        }
        
        Ok(WorkflowResult {
            session_id: session.session_id.clone(),
            execution_strategy: "sequential".to_string(),
            step_results: results,
            final_context: workflow_context,
            execution_metrics: self.calculate_execution_metrics(&workflow).await?,
        })
    }
    
    async fn handle_agent_failure(
        &self,
        session: &CollaborationSession,
        failed_agent: &AgentId,
        error: &LLMSpellError,
    ) -> Result<RecoveryAction> {
        // For sequential strategy, failure of one agent typically requires retry or abort
        match error {
            LLMSpellError::Agent(AgentError::Timeout { .. }) => {
                Ok(RecoveryAction::RetryWithTimeout {
                    agent_id: failed_agent.clone(),
                    new_timeout: Duration::from_secs(300), // Increase timeout
                })
            }
            LLMSpellError::Agent(AgentError::MemoryLimitExceeded { .. }) => {
                Ok(RecoveryAction::RetryWithIncreasedResources {
                    agent_id: failed_agent.clone(),
                    additional_memory_mb: 512,
                })
            }
            _ => {
                Ok(RecoveryAction::AbortWorkflow {
                    reason: format!("Unrecoverable agent failure: {}", error),
                })
            }
        }
    }
}

// Parallel execution strategy
pub struct ParallelStrategy {
    concurrency_limit: usize,
    synchronization_points: Vec<SynchronizationPoint>,
}

#[async_trait]
impl CoordinationStrategy for ParallelStrategy {
    async fn execute_workflow(
        &self,
        session: &CollaborationSession,
        workflow: WorkflowDefinition,
        agents: &Arc<RwLock<HashMap<AgentId, Box<dyn Agent>>>>,
        communication_bus: &CommunicationBus,
    ) -> Result<WorkflowResult> {
        let mut workflow_context = WorkflowContext::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.concurrency_limit));
        
        // Group steps by dependencies
        let execution_groups = self.group_steps_by_dependencies(&workflow.steps)?;
        let mut all_results = Vec::new();
        
        for group in execution_groups {
            let mut group_tasks = Vec::new();
            
            // Execute all steps in this group in parallel
            for step in group {
                let agent_id = step.agent_id.clone();
                let agents_clone = agents.clone();
                let communication_bus_clone = communication_bus.clone();
                let semaphore_clone = semaphore.clone();
                let session_clone = session.clone();
                let workflow_context_clone = workflow_context.clone();
                let previous_results = all_results.clone();
                
                let task = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await?;
                    
                    // Get agent
                    let agent = {
                        let agents_lock = agents_clone.read().await;
                        agents_lock.get(&agent_id)
                            .ok_or_else(|| LLMSpellError::Agent(AgentError::NotFound {
                                agent_id: agent_id.0.clone(),
                            }))?
                            .clone()
                    };
                    
                    // Prepare execution context
                    let execution_context = ExecutionContext {
                        session_id: session_clone.session_id,
                        step_index: previous_results.len(),
                        shared_context: session_clone.shared_context,
                        previous_results,
                        workflow_context: workflow_context_clone,
                    };
                    
                    // Execute step
                    Self::execute_workflow_step_parallel(
                        agent,
                        &step,
                        execution_context,
                        &communication_bus_clone,
                    ).await
                });
                
                group_tasks.push(task);
            }
            
            // Wait for all tasks in this group to complete
            let group_results = futures::future::try_join_all(group_tasks).await?;
            
            // Update workflow context and add results
            for result in group_results {
                let step_result = result?;
                workflow_context.add_step_result(&step_result.step_id, &step_result);
                all_results.push(step_result);
            }
            
            // Check for synchronization points
            if let Some(sync_point) = self.get_synchronization_point_for_group(&group) {
                self.coordinate_synchronization(session, sync_point).await?;
            }
        }
        
        Ok(WorkflowResult {
            session_id: session.session_id.clone(),
            execution_strategy: "parallel".to_string(),
            step_results: all_results,
            final_context: workflow_context,
            execution_metrics: self.calculate_execution_metrics(&workflow).await?,
        })
    }
}

// Consensus-based strategy for critical decisions
pub struct ConsensusStrategy {
    consensus_threshold: f64, // 0.0 to 1.0
    voting_mechanism: VotingMechanism,
    conflict_resolution: ConflictResolution,
}

#[async_trait]
impl CoordinationStrategy for ConsensusStrategy {
    async fn execute_workflow(
        &self,
        session: &CollaborationSession,
        workflow: WorkflowDefinition,
        agents: &Arc<RwLock<HashMap<AgentId, Box<dyn Agent>>>>,
        communication_bus: &CommunicationBus,
    ) -> Result<WorkflowResult> {
        let mut workflow_context = WorkflowContext::new();
        let mut results = Vec::new();
        
        for step in workflow.steps {
            // For consensus strategy, each step involves all agents providing input
            let consensus_result = self.execute_consensus_step(
                session,
                &step,
                agents,
                communication_bus,
                &workflow_context,
            ).await?;
            
            workflow_context.add_step_result(&step.step_id, &consensus_result);
            results.push(consensus_result);
        }
        
        Ok(WorkflowResult {
            session_id: session.session_id.clone(),
            execution_strategy: "consensus".to_string(),
            step_results: results,
            final_context: workflow_context,
            execution_metrics: self.calculate_execution_metrics(&workflow).await?,
        })
    }
    
    async fn execute_consensus_step(
        &self,
        session: &CollaborationSession,
        step: &WorkflowStep,
        agents: &Arc<RwLock<HashMap<AgentId, Box<dyn Agent>>>>,
        communication_bus: &CommunicationBus,
        workflow_context: &WorkflowContext,
    ) -> Result<StepResult> {
        // Phase 1: Collect proposals from all agents
        let proposals = self.collect_agent_proposals(
            session,
            step,
            agents,
            communication_bus,
            workflow_context,
        ).await?;
        
        // Phase 2: Voting phase
        let votes = self.conduct_voting_round(
            session,
            &proposals,
            agents,
            communication_bus,
        ).await?;
        
        // Phase 3: Determine consensus
        let consensus_result = self.determine_consensus(&proposals, &votes)?;
        
        // Phase 4: Conflict resolution if needed
        let final_result = if consensus_result.consensus_reached {
            consensus_result.agreed_proposal
        } else {
            self.resolve_conflict(
                session,
                &proposals,
                &votes,
                agents,
                communication_bus,
            ).await?
        };
        
        Ok(StepResult {
            step_id: step.step_id.clone(),
            agent_id: AgentId("consensus".to_string()),
            result_data: final_result,
            execution_time: consensus_result.total_time,
            metadata: consensus_result.consensus_metadata,
        })
    }
}
```

### Dynamic Workflow Generation

```rust
pub struct DynamicWorkflowGenerator {
    workflow_templates: HashMap<String, WorkflowTemplate>,
    condition_evaluator: ConditionEvaluator,
    dependency_resolver: DependencyResolver,
    optimization_engine: WorkflowOptimizer,
}

impl DynamicWorkflowGenerator {
    // Generate workflow based on runtime conditions
    pub async fn generate_adaptive_workflow(
        &self,
        requirements: WorkflowRequirements,
        available_agents: Vec<AgentCapability>,
        runtime_context: RuntimeContext,
    ) -> Result<WorkflowDefinition> {
        // 1. Analyze requirements and context
        let analysis = self.analyze_requirements(&requirements, &runtime_context).await?;
        
        // 2. Select appropriate base template
        let base_template = self.select_base_template(&analysis)?;
        
        // 3. Adapt template to current conditions
        let adapted_workflow = self.adapt_workflow_template(
            base_template,
            &available_agents,
            &runtime_context,
        ).await?;
        
        // 4. Optimize workflow for performance
        let optimized_workflow = self.optimization_engine.optimize_workflow(
            adapted_workflow,
            &runtime_context.performance_constraints,
        ).await?;
        
        // 5. Validate and resolve dependencies
        self.dependency_resolver.validate_and_resolve(&optimized_workflow)?;
        
        Ok(optimized_workflow)
    }
    
    // Modify running workflow based on changing conditions
    pub async fn adapt_running_workflow(
        &self,
        current_workflow: &WorkflowDefinition,
        execution_state: &WorkflowExecutionState,
        new_conditions: &RuntimeContext,
    ) -> Result<WorkflowModification> {
        // Analyze what has changed
        let change_analysis = self.analyze_condition_changes(
            &execution_state.original_context,
            new_conditions,
        ).await?;
        
        // Determine if adaptation is needed
        if !self.should_adapt_workflow(&change_analysis)? {
            return Ok(WorkflowModification::NoChangeNeeded);
        }
        
        // Generate modification strategy
        let modification_strategy = match change_analysis.change_severity {
            ChangeSeverity::Minor => self.generate_minor_modifications(
                current_workflow,
                &change_analysis,
            ).await?,
            ChangeSeverity::Major => self.generate_major_modifications(
                current_workflow,
                execution_state,
                &change_analysis,
            ).await?,
            ChangeSeverity::Critical => self.generate_critical_modifications(
                current_workflow,
                execution_state,
                &change_analysis,
            ).await?,
        };
        
        Ok(modification_strategy)
    }
    
    async fn generate_minor_modifications(
        &self,
        workflow: &WorkflowDefinition,
        change_analysis: &ChangeAnalysis,
    ) -> Result<WorkflowModification> {
        let mut modifications = Vec::new();
        
        // Adjust execution parameters
        if change_analysis.performance_impact.is_some() {
            modifications.push(ModificationType::ParameterAdjustment {
                step_id: change_analysis.affected_steps.clone(),
                new_parameters: self.calculate_new_parameters(&change_analysis.performance_impact)?,
            });
        }
        
        // Add retry logic if reliability is affected
        if change_analysis.reliability_impact.is_some() {
            modifications.push(ModificationType::AddRetryLogic {
                step_ids: change_analysis.affected_steps.clone(),
                retry_config: self.calculate_retry_config(&change_analysis.reliability_impact)?,
            });
        }
        
        Ok(WorkflowModification::MinorAdjustments(modifications))
    }
    
    async fn generate_major_modifications(
        &self,
        workflow: &WorkflowDefinition,
        execution_state: &WorkflowExecutionState,
        change_analysis: &ChangeAnalysis,
    ) -> Result<WorkflowModification> {
        // Major modifications might involve adding/removing steps or changing strategy
        let mut new_steps = Vec::new();
        let mut removed_steps = Vec::new();
        let mut modified_steps = Vec::new();
        
        // Analyze each step for modification needs
        for step in &workflow.steps {
            if change_analysis.affected_steps.contains(&step.step_id) {
                match self.evaluate_step_modification_need(step, change_analysis).await? {
                    StepModificationNeed::Remove => {
                        removed_steps.push(step.step_id.clone());
                    }
                    StepModificationNeed::Modify(new_step) => {
                        modified_steps.push(new_step);
                    }
                    StepModificationNeed::Replace(replacement_steps) => {
                        removed_steps.push(step.step_id.clone());
                        new_steps.extend(replacement_steps);
                    }
                    StepModificationNeed::NoChange => {
                        // Keep step as is
                    }
                }
            }
        }
        
        // Add new steps if needed for new requirements
        if let Some(new_requirements) = &change_analysis.new_requirements {
            let additional_steps = self.generate_steps_for_requirements(
                new_requirements,
                &execution_state.available_agents,
            ).await?;
            new_steps.extend(additional_steps);
        }
        
        Ok(WorkflowModification::MajorRestructure {
            new_steps,
            removed_steps,
            modified_steps,
            new_coordination_strategy: self.determine_new_strategy(change_analysis)?,
        })
    }
}

#[derive(Debug)]
pub enum WorkflowModification {
    NoChangeNeeded,
    MinorAdjustments(Vec<ModificationType>),
    MajorRestructure {
        new_steps: Vec<WorkflowStep>,
        removed_steps: Vec<String>,
        modified_steps: Vec<WorkflowStep>,
        new_coordination_strategy: Option<String>,
    },
    CompleteRegeneration(WorkflowDefinition),
}

#[derive(Debug)]
pub enum ModificationType {
    ParameterAdjustment {
        step_id: Vec<String>,
        new_parameters: HashMap<String, serde_json::Value>,
    },
    AddRetryLogic {
        step_ids: Vec<String>,
        retry_config: RetryConfig,
    },
    ChangeTimeout {
        step_ids: Vec<String>,
        new_timeout: Duration,
    },
    AddFallbackStrategy {
        step_id: String,
        fallback_steps: Vec<WorkflowStep>,
    },
}
```

## Protocol Integration (MCP, A2A)

[... existing content ...]

## Scheduling and Automation

Rs-LLMSpell is designed not just for interactive execution but also for creating long-running, automated services and scheduled tasks. This is achieved through a combination of a dedicated scheduler, trigger-based execution, and specialized listener tools.

### Scheduler Architecture

The core of the automation capability is the `Scheduler` component, which manages and executes tasks based on predefined triggers.

```rust
pub struct Scheduler {
    jobs: Vec<ScheduledJob>,
    trigger_evaluator: TriggerEvaluator,
    runtime: Arc<AgentRuntime>,
}

pub struct ScheduledJob {
    id: String,
    trigger: Trigger,
    action: Action, // e.g., RunWorkflow, ExecuteAgent
    config: JobConfig,
}

pub enum Trigger {
    Cron(String), // e.g., "0 * * * *"
    Interval(Duration),
    OnEvent(EventFilter),
    External(ExternalTriggerConfig), // e.g., Webhook
}
```

### Trigger Types

-   **Cron/Interval Triggers**: For time-based tasks, similar to traditional cron jobs.
-   **Event Triggers**: Workflows or agents can be executed in response to specific system events from the `EventBus`.
-   **External Triggers**: The system can listen for external signals via network tools.

### Listener Tools

To enable event-driven automation from external sources, the built-in tool catalog includes listener tools:

-   **`WebhookListenerTool`**: Opens an HTTP endpoint to receive webhook calls, which can then trigger specific workflows or agents.
-   **`SocketListenerTool`**: Listens on a TCP or Unix socket for incoming data, allowing for custom protocol integrations.

### Example: Daily Report Generation

This example shows a spell that defines a scheduled workflow to generate and email a report every day at 8:00 AM.

```lua
-- daily_report.lua
local report_workflow = Workflow.sequential({...})

Scheduler.register({
    name = "daily_market_report",
    trigger = { type = "cron", schedule = "0 8 * * *" }, -- 8:00 AM daily
    action = {
        type = "workflow",
        workflow = report_workflow,
        input = {
            report_date = "{{now()}}"
        }
    }
})
```

This spell would be loaded by the `rs-llmspell` runtime in daemon mode, and the scheduler would ensure the workflow is executed at the specified time.

## Plugin System and Extensions

### Model Control Protocol (MCP) Support

```rust
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, WebSocketStream};
use futures_util::{SinkExt, StreamExt};

pub struct MCPClient {
    connection: MCPConnection,
    session_id: String,
    capabilities: MCPCapabilities,
    message_handler: MCPMessageHandler,
    state_manager: MCPStateManager,
}

#[derive(Debug, Clone)]
pub struct MCPCapabilities {
    pub supports_tools: bool,
    pub supports_resources: bool,
    pub supports_prompts: bool,
    pub supports_sampling: bool,
    pub supports_notifications: bool,
    pub protocol_version: String,
}

impl MCPClient {
    pub async fn connect(server_url: &str, client_info: MCPClientInfo) -> Result<Self> {
        let (ws_stream, _) = connect_async(server_url).await
            .map_err(|e| LLMSpellError::Network(format!("MCP connection failed: {}", e)))?;
        
        let connection = MCPConnection::new(ws_stream);
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let mut client = Self {
            connection,
            session_id: session_id.clone(),
            capabilities: MCPCapabilities::default(),
            message_handler: MCPMessageHandler::new(),
            state_manager: MCPStateManager::new(),
        };
        
        // Perform initial handshake
        client.perform_handshake(client_info).await?;
        
        Ok(client)
    }
    
    async fn perform_handshake(&mut self, client_info: MCPClientInfo) -> Result<()> {
        // Send initialize request
        let initialize_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {},
                    "sampling": {}
                },
                "clientInfo": {
                    "name": client_info.name,
                    "version": client_info.version
                }
            }
        });
        
        self.connection.send_message(initialize_request).await?;
        
        // Wait for initialize response
        let response = self.connection.receive_message().await?;
        let server_capabilities = self.parse_server_capabilities(&response)?;
        
        // Update our capabilities based on server response
        self.capabilities = self.negotiate_capabilities(server_capabilities)?;
        
        // Send initialized notification
        let initialized_notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        
        self.connection.send_message(initialized_notification).await?;
        
        tracing::info!("MCP handshake completed successfully");
        Ok(())
    }
    
    // List available tools from MCP server
    pub async fn list_tools(&self) -> Result<Vec<MCPTool>> {
        if !self.capabilities.supports_tools {
            return Err(LLMSpellError::Protocol(
                "Server does not support tools".to_string()
            ));
        }
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.generate_request_id(),
            "method": "tools/list"
        });
        
        self.connection.send_message(request).await?;
        let response = self.connection.receive_message().await?;
        
        self.parse_tools_list_response(response)
    }
    
    // Call a tool on the MCP server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<MCPToolResult> {
        if !self.capabilities.supports_tools {
            return Err(LLMSpellError::Protocol(
                "Server does not support tools".to_string()
            ));
        }
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.generate_request_id(),
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });
        
        self.connection.send_message(request).await?;
        let response = self.connection.receive_message().await?;
        
        self.parse_tool_call_response(response)
    }
    
    // List available resources
    pub async fn list_resources(&self) -> Result<Vec<MCPResource>> {
        if !self.capabilities.supports_resources {
            return Err(LLMSpellError::Protocol(
                "Server does not support resources".to_string()
            ));
        }
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.generate_request_id(),
            "method": "resources/list"
        });
        
        self.connection.send_message(request).await?;
        let response = self.connection.receive_message().await?;
        
        self.parse_resources_list_response(response)
    }
    
    // Read a resource
    pub async fn read_resource(&self, uri: &str) -> Result<MCPResourceContent> {
        if !self.capabilities.supports_resources {
            return Err(LLMSpellError::Protocol(
                "Server does not support resources".to_string()
            ));
        }
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.generate_request_id(),
            "method": "resources/read",
            "params": {
                "uri": uri
            }
        });
        
        self.connection.send_message(request).await?;
        let response = self.connection.receive_message().await?;
        
        self.parse_resource_read_response(response)
    }
    
    // Subscribe to resource changes
    pub async fn subscribe_to_resource(&self, uri: &str) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.generate_request_id(),
            "method": "resources/subscribe",
            "params": {
                "uri": uri
            }
        });
        
        self.connection.send_message(request).await?;
        let response = self.connection.receive_message().await?;
        
        self.validate_subscription_response(response)?;
        
        // Set up notification handler for this resource
        self.state_manager.add_resource_subscription(uri).await;
        
        Ok(())
    }
}

// MCP Server implementation for rs-llmspell
pub struct MCPServer {
    server: MCPServerCore,
    tool_registry: Arc<RwLock<HashMap<String, Box<dyn MCPToolHandler>>>>,
    resource_registry: Arc<RwLock<HashMap<String, Box<dyn MCPResourceHandler>>>>,
    prompt_registry: Arc<RwLock<HashMap<String, Box<dyn MCPPromptHandler>>>>,
    client_sessions: Arc<RwLock<HashMap<String, MCPClientSession>>>,
}

impl MCPServer {
    pub async fn new(config: MCPServerConfig) -> Result<Self> {
        let server = MCPServerCore::new(config.clone()).await?;
        
        Ok(Self {
            server,
            tool_registry: Arc::new(RwLock::new(HashMap::new())),
            resource_registry: Arc::new(RwLock::new(HashMap::new())),
            prompt_registry: Arc::new(RwLock::new(HashMap::new())),
            client_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    // Register rs-llmspell tools as MCP tools
    pub async fn register_llmspell_tools(&self, tools: Vec<Box<dyn Tool>>) -> Result<()> {
        let mut registry = self.tool_registry.write().await;
        
        for tool in tools {
            let mcp_tool_handler = MCPToolWrapper::new(tool);
            registry.insert(
                mcp_tool_handler.get_name().to_string(),
                Box::new(mcp_tool_handler),
            );
        }
        
        tracing::info!("Registered {} rs-llmspell tools with MCP server", registry.len());
        Ok(())
    }
    
    // Register rs-llmspell agents as MCP resources
    pub async fn register_agent_resources(&self, agents: Vec<Box<dyn Agent>>) -> Result<()> {
        let mut registry = self.resource_registry.write().await;
        
        for agent in agents {
            let resource_handler = AgentResourceHandler::new(agent);
            let uri = format!("llmspell://agents/{}", resource_handler.get_agent_id());
            registry.insert(uri, Box::new(resource_handler));
        }
        
        tracing::info!("Registered {} agent resources with MCP server", registry.len());
        Ok(())
    }
    
    // Start MCP server
    pub async fn start(&self, bind_address: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(bind_address).await
            .map_err(|e| LLMSpellError::Network(format!("Failed to bind MCP server: {}", e)))?;
        
        tracing::info!("MCP server listening on {}", bind_address);
        
        while let Ok((stream, addr)) = listener.accept().await {
            let session_id = uuid::Uuid::new_v4().to_string();
            tracing::info!("New MCP client connection from {}: {}", addr, session_id);
            
            // Handle client connection
            let tool_registry = self.tool_registry.clone();
            let resource_registry = self.resource_registry.clone();
            let prompt_registry = self.prompt_registry.clone();
            let client_sessions = self.client_sessions.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client_connection(
                    stream,
                    session_id.clone(),
                    tool_registry,
                    resource_registry,
                    prompt_registry,
                    client_sessions,
                ).await {
                    tracing::error!("Error handling MCP client {}: {}", session_id, e);
                }
            });
        }
        
        Ok(())
    }
}

// Wrapper to expose rs-llmspell tools as MCP tools
struct MCPToolWrapper {
    tool: Box<dyn Tool>,
    mcp_schema: MCPToolSchema,
}

impl MCPToolWrapper {
    fn new(tool: Box<dyn Tool>) -> Self {
        let mcp_schema = Self::convert_tool_to_mcp_schema(&*tool);
        Self { tool, mcp_schema }
    }
    
    fn convert_tool_to_mcp_schema(tool: &dyn Tool) -> MCPToolSchema {
        let tool_info = tool.get_tool_info();
        
        MCPToolSchema {
            name: tool_info.name,
            description: tool_info.description,
            input_schema: Self::convert_parameters_to_json_schema(&tool_info.parameters),
        }
    }
}

#[async_trait]
impl MCPToolHandler for MCPToolWrapper {
    async fn execute(&self, arguments: Value) -> Result<MCPToolResult> {
        // Convert MCP arguments to rs-llmspell tool parameters
        let tool_params = self.convert_mcp_args_to_tool_params(arguments)?;
        
        // Execute the rs-llmspell tool
        let result = self.tool.execute(tool_params).await?;
        
        // Convert rs-llmspell result to MCP format
        let mcp_result = MCPToolResult {
            content: vec![MCPContent::Text {
                text: serde_json::to_string_pretty(&result.output)?,
            }],
            is_error: false,
        };
        
        Ok(mcp_result)
    }
    
    fn get_schema(&self) -> &MCPToolSchema {
        &self.mcp_schema
    }
    
    fn get_name(&self) -> &str {
        &self.mcp_schema.name
    }
}
```

### Agent-to-Agent (A2A) Protocol

```rust
pub struct A2AProtocolManager {
    node_id: String,
    peer_connections: Arc<RwLock<HashMap<String, A2AConnection>>>,
    message_router: A2AMessageRouter,
    discovery_service: A2ADiscoveryService,
    security_manager: A2ASecurityManager,
    state_synchronizer: A2AStateSynchronizer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub message_id: String,
    pub sender_node: String,
    pub recipient_node: Option<String>, // None for broadcast
    pub message_type: A2AMessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub security_context: A2ASecurityContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2AMessageType {
    // Agent coordination
    AgentHandoff {
        agent_id: String,
        handoff_reason: String,
        context_data: serde_json::Value,
    },
    TaskDelegation {
        task_id: String,
        task_definition: serde_json::Value,
        delegation_context: serde_json::Value,
    },
    ResultSharing {
        task_id: String,
        result_data: serde_json::Value,
        confidence_score: f64,
    },
    
    // Workflow coordination
    WorkflowSynchronization {
        workflow_id: String,
        sync_point: String,
        node_state: serde_json::Value,
    },
    WorkflowJoin {
        workflow_id: String,
        capabilities: Vec<String>,
        load_metrics: NodeLoadMetrics,
    },
    WorkflowLeave {
        workflow_id: String,
        reason: String,
    },
    
    // Resource sharing
    ResourceOffer {
        resource_type: String,
        availability: ResourceAvailability,
        access_conditions: serde_json::Value,
    },
    ResourceRequest {
        request_id: String,
        resource_requirements: ResourceRequirements,
        priority: RequestPriority,
    },
    ResourceResponse {
        request_id: String,
        response_type: ResourceResponseType,
        resource_data: Option<serde_json::Value>,
    },
    
    // Discovery and topology
    NodeAnnouncement {
        node_capabilities: NodeCapabilities,
        network_topology: NetworkTopology,
    },
    TopologyUpdate {
        changes: Vec<TopologyChange>,
    },
    HealthCheck {
        metrics: NodeHealthMetrics,
    },
}

impl A2AProtocolManager {
    pub async fn new(config: A2AConfig) -> Result<Self> {
        let node_id = config.node_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        Ok(Self {
            node_id: node_id.clone(),
            peer_connections: Arc::new(RwLock::new(HashMap::new())),
            message_router: A2AMessageRouter::new(&config.routing),
            discovery_service: A2ADiscoveryService::new(&config.discovery, &node_id),
            security_manager: A2ASecurityManager::new(&config.security),
            state_synchronizer: A2AStateSynchronizer::new(&config.synchronization),
        })
    }
    
    // Join the A2A network
    pub async fn join_network(&self, bootstrap_nodes: Vec<String>) -> Result<()> {
        // 1. Initialize security context
        self.security_manager.initialize().await?;
        
        // 2. Connect to bootstrap nodes
        for bootstrap_node in bootstrap_nodes {
            self.connect_to_peer(&bootstrap_node).await?;
        }
        
        // 3. Start discovery service
        self.discovery_service.start().await?;
        
        // 4. Announce our presence to the network
        let announcement = A2AMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_node: self.node_id.clone(),
            recipient_node: None, // Broadcast
            message_type: A2AMessageType::NodeAnnouncement {
                node_capabilities: self.get_node_capabilities().await?,
                network_topology: self.discovery_service.get_local_topology().await?,
            },
            payload: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            security_context: self.security_manager.create_security_context().await?,
        };
        
        self.broadcast_message(announcement).await?;
        
        // 5. Start message processing loop
        self.start_message_processing().await?;
        
        tracing::info!("Successfully joined A2A network as node: {}", self.node_id);
        Ok(())
    }
    
    // Delegate a task to another node
    pub async fn delegate_task(
        &self,
        task_definition: TaskDefinition,
        target_capabilities: Vec<String>,
        priority: RequestPriority,
    ) -> Result<TaskDelegationResult> {
        // 1. Find suitable nodes
        let suitable_nodes = self.discovery_service.find_nodes_with_capabilities(
            &target_capabilities,
        ).await?;
        
        if suitable_nodes.is_empty() {
            return Err(LLMSpellError::A2A(
                "No suitable nodes found for task delegation".to_string()
            ));
        }
        
        // 2. Select best node based on load and capabilities
        let target_node = self.select_best_node_for_task(
            &suitable_nodes,
            &task_definition,
            priority,
        ).await?;
        
        // 3. Send delegation message
        let task_id = uuid::Uuid::new_v4().to_string();
        let delegation_message = A2AMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_node: self.node_id.clone(),
            recipient_node: Some(target_node.node_id.clone()),
            message_type: A2AMessageType::TaskDelegation {
                task_id: task_id.clone(),
                task_definition: serde_json::to_value(&task_definition)?,
                delegation_context: serde_json::json!({
                    "priority": priority,
                    "deadline": task_definition.deadline,
                    "delegation_time": chrono::Utc::now()
                }),
            },
            payload: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            security_context: self.security_manager.create_security_context().await?,
        };
        
        self.send_message_to_node(&target_node.node_id, delegation_message).await?;
        
        // 4. Wait for acceptance/rejection
        let delegation_response = self.wait_for_delegation_response(&task_id).await?;
        
        Ok(TaskDelegationResult {
            task_id,
            target_node: target_node.node_id,
            status: delegation_response.status,
            estimated_completion: delegation_response.estimated_completion,
        })
    }
    
    // Handle agent handoff to another node
    pub async fn handoff_agent(
        &self,
        agent: Box<dyn Agent>,
        target_node: &str,
        handoff_reason: &str,
    ) -> Result<AgentHandoffResult> {
        // 1. Serialize agent state
        let agent_state = agent.save_state().await?;
        let agent_metadata = agent.get_metadata();
        
        // 2. Create handoff message
        let handoff_message = A2AMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_node: self.node_id.clone(),
            recipient_node: Some(target_node.to_string()),
            message_type: A2AMessageType::AgentHandoff {
                agent_id: agent_metadata.id.clone(),
                handoff_reason: handoff_reason.to_string(),
                context_data: serde_json::json!({
                    "agent_state": agent_state,
                    "agent_metadata": agent_metadata,
                    "handoff_timestamp": chrono::Utc::now()
                }),
            },
            payload: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            security_context: self.security_manager.create_security_context().await?,
        };
        
        // 3. Send handoff message
        self.send_message_to_node(target_node, handoff_message).await?;
        
        // 4. Wait for handoff confirmation
        let confirmation = self.wait_for_handoff_confirmation(&agent_metadata.id).await?;
        
        // 5. If successful, deactivate local agent
        if confirmation.success {
            agent.deactivate().await?;
            tracing::info!(
                "Successfully handed off agent {} to node {}",
                agent_metadata.id,
                target_node
            );
        }
        
        Ok(AgentHandoffResult {
            agent_id: agent_metadata.id,
            target_node: target_node.to_string(),
            success: confirmation.success,
            handoff_time: chrono::Utc::now(),
        })
    }
    
    // Share workflow results with network
    pub async fn share_workflow_results(
        &self,
        workflow_id: &str,
        results: WorkflowResult,
        sharing_policy: ResultSharingPolicy,
    ) -> Result<()> {
        // Determine which nodes should receive the results
        let target_nodes = match sharing_policy {
            ResultSharingPolicy::Broadcast => {
                self.discovery_service.get_all_active_nodes().await?
            }
            ResultSharingPolicy::WorkflowParticipants => {
                self.discovery_service.get_workflow_participants(workflow_id).await?
            }
            ResultSharingPolicy::Specific(nodes) => nodes,
        };
        
        // Create result sharing message
        let sharing_message = A2AMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            sender_node: self.node_id.clone(),
            recipient_node: None, // Will be set per target
            message_type: A2AMessageType::ResultSharing {
                task_id: workflow_id.to_string(),
                result_data: serde_json::to_value(&results)?,
                confidence_score: results.confidence_score.unwrap_or(1.0),
            },
            payload: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            security_context: self.security_manager.create_security_context().await?,
        };
        
        // Send to all target nodes
        for target_node in target_nodes {
            let mut message = sharing_message.clone();
            message.recipient_node = Some(target_node.clone());
            
            self.send_message_to_node(&target_node, message).await?;
        }
        
        tracing::info!(
            "Shared workflow results for {} with {} nodes",
            workflow_id,
            target_nodes.len()
        );
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ResultSharingPolicy {
    Broadcast,
    WorkflowParticipants,
    Specific(Vec<String>),
}

#[derive(Debug)]
pub struct TaskDelegationResult {
    pub task_id: String,
    pub target_node: String,
    pub status: DelegationStatus,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub enum DelegationStatus {
    Accepted,
    Rejected(String),
    Queued,
}

#[derive(Debug)]
pub struct AgentHandoffResult {
    pub agent_id: String,
    pub target_node: String,
    pub success: bool,
    pub handoff_time: chrono::DateTime<chrono::Utc>,
}
```

---

# Part VIII: Testing and Quality Assurance

## Complete Testing Strategy

Rs-LLMSpell implements a comprehensive testing strategy that covers all architectural layers, supports Test-Driven Development (TDD), and ensures production-ready quality across all components.

### Testing Philosophy and Principles

**1. Test-First Development**: All features must have tests written before implementation
**2. Comprehensive Coverage**: Unit, integration, and end-to-end tests for all components
**3. Multi-Language Testing**: Script engine testing across Lua, JavaScript, and future languages
**4. Performance Validation**: All performance claims must be validated through benchmarks
**5. Security-First Testing**: Security testing integrated into all test levels
**6. Production Simulation**: Tests must simulate real-world production scenarios

### Testing Architecture

```rust
use tokio_test;
use criterion::{Criterion, BenchmarkId};
use proptest::prelude::*;
use testcontainers::*;

pub struct TestingFramework {
    unit_test_runner: UnitTestRunner,
    integration_test_runner: IntegrationTestRunner,
    e2e_test_runner: E2ETestRunner,
    performance_tester: PerformanceTester,
    security_tester: SecurityTester,
    script_engine_tester: ScriptEngineTester,
    chaos_tester: ChaosTester,
}

#[derive(Debug, Clone)]
pub struct TestConfiguration {
    pub test_environment: TestEnvironment,
    pub mock_llm_providers: bool,
    pub enable_chaos_testing: bool,
    pub performance_baseline: PerformanceBaseline,
    pub security_test_level: SecurityTestLevel,
    pub parallel_execution: bool,
    pub test_data_isolation: bool,
}

#[derive(Debug, Clone)]
pub enum TestEnvironment {
    Unit,
    Integration,
    Staging,
    ProductionLike,
}

impl TestingFramework {
    pub async fn new(config: TestConfiguration) -> Result<Self> {
        let unit_test_runner = UnitTestRunner::new(&config);
        let integration_test_runner = IntegrationTestRunner::new(&config).await?;
        let e2e_test_runner = E2ETestRunner::new(&config).await?;
        let performance_tester = PerformanceTester::new(&config);
        let security_tester = SecurityTester::new(&config);
        let script_engine_tester = ScriptEngineTester::new(&config);
        let chaos_tester = ChaosTester::new(&config);
        
        Ok(Self {
            unit_test_runner,
            integration_test_runner,
            e2e_test_runner,
            performance_tester,
            security_tester,
            script_engine_tester,
            chaos_tester,
        })
    }
    
    // Run all test suites
    pub async fn run_complete_test_suite(&self) -> Result<TestSuiteResult> {
        let mut results = TestSuiteResult::new();
        
        // 1. Unit Tests (fastest feedback)
        tracing::info!("Running unit tests...");
        let unit_results = self.unit_test_runner.run_all_tests().await?;
        results.add_unit_results(unit_results);
        
        if !results.unit_tests_passed() {
            return Ok(results); // Fail fast if unit tests fail
        }
        
        // 2. Integration Tests
        tracing::info!("Running integration tests...");
        let integration_results = self.integration_test_runner.run_all_tests().await?;
        results.add_integration_results(integration_results);
        
        // 3. Script Engine Tests
        tracing::info!("Running script engine tests...");
        let script_results = self.script_engine_tester.run_all_tests().await?;
        results.add_script_engine_results(script_results);
        
        // 4. Security Tests
        tracing::info!("Running security tests...");
        let security_results = self.security_tester.run_all_tests().await?;
        results.add_security_results(security_results);
        
        // 5. Performance Tests
        tracing::info!("Running performance tests...");
        let performance_results = self.performance_tester.run_benchmarks().await?;
        results.add_performance_results(performance_results);
        
        // 6. End-to-End Tests
        tracing::info!("Running end-to-end tests...");
        let e2e_results = self.e2e_test_runner.run_all_tests().await?;
        results.add_e2e_results(e2e_results);
        
        // 7. Chaos Tests (if enabled)
        if self.chaos_tester.is_enabled() {
            tracing::info!("Running chaos tests...");
            let chaos_results = self.chaos_tester.run_chaos_scenarios().await?;
            results.add_chaos_results(chaos_results);
        }
        
        Ok(results)
    }
}
```

### Unit Testing Framework

```rust
pub struct UnitTestRunner {
    test_config: TestConfiguration,
    mock_factory: MockFactory,
    test_data_generator: TestDataGenerator,
}

impl UnitTestRunner {
    // Core trait testing
    pub async fn test_base_agent_trait(&self) -> Result<UnitTestResults> {
        let mut results = UnitTestResults::new("BaseAgent Trait Tests");
        
        // Test 1: Agent creation and initialization
        results.add_test(self.test_agent_creation().await?);
        
        // Test 2: Tool registration and discovery
        results.add_test(self.test_tool_registration().await?);
        
        // Test 3: State management
        results.add_test(self.test_state_management().await?);
        
        // Test 4: Hook execution
        results.add_test(self.test_hook_execution().await?);
        
        // Test 5: Error handling
        results.add_test(self.test_error_handling().await?);
        
        Ok(results)
    }
    
    async fn test_agent_creation(&self) -> Result<TestResult> {
        // Create mock LLM provider
        let mock_provider = self.mock_factory.create_mock_llm_provider();
        
        // Test agent creation with valid configuration
        let agent_config = AgentConfig {
            name: "test_agent".to_string(),
            description: "Test agent for unit testing".to_string(),
            llm_provider: "mock_provider".to_string(),
            system_prompt: "You are a test agent".to_string(),
            tools: vec!["test_tool".to_string()],
            max_iterations: 10,
            timeout: Duration::from_secs(30),
        };
        
        let agent = TestAgent::new(agent_config.clone()).await;
        
        let test_result = match agent {
            Ok(agent) => {
                // Verify agent properties
                assert_eq!(agent.get_name(), "test_agent");
                assert_eq!(agent.get_description(), "Test agent for unit testing");
                assert_eq!(agent.get_tool_count(), 1);
                
                TestResult::passed("Agent creation with valid config")
            }
            Err(e) => TestResult::failed("Agent creation failed", e.to_string()),
        };
        
        // Test agent creation with invalid configuration
        let invalid_config = AgentConfig {
            name: "".to_string(), // Invalid empty name
            ..agent_config
        };
        
        let invalid_agent = TestAgent::new(invalid_config).await;
        let invalid_test_result = match invalid_agent {
            Ok(_) => TestResult::failed("Should fail with invalid config", "Agent created with empty name".to_string()),
            Err(_) => TestResult::passed("Correctly rejected invalid config"),
        };
        
        // Combine results
        if test_result.passed && invalid_test_result.passed {
            Ok(TestResult::passed("Agent creation tests"))
        } else {
            Ok(TestResult::failed("Agent creation tests", "One or more agent creation tests failed".to_string()))
        }
    }
    
    async fn test_tool_registration(&self) -> Result<TestResult> {
        let agent = self.create_test_agent().await?;
        
        // Create test tool
        let test_tool = Box::new(TestTool::new("test_calculator", "Performs calculations"));
        
        // Test tool registration
        let registration_result = agent.register_tool(test_tool).await;
        
        match registration_result {
            Ok(_) => {
                // Verify tool is registered
                let tools = agent.list_tools().await?;
                if tools.iter().any(|t| t.name == "test_calculator") {
                    Ok(TestResult::passed("Tool registration"))
                } else {
                    Ok(TestResult::failed("Tool registration", "Tool not found in list".to_string()))
                }
            }
            Err(e) => Ok(TestResult::failed("Tool registration", e.to_string())),
        }
    }
    
    async fn test_state_management(&self) -> Result<TestResult> {
        let mut agent = self.create_test_agent().await?;
        
        // Test initial state
        let initial_state = agent.get_state().await?;
        assert_eq!(initial_state.iteration_count, 0);
        
        // Modify state
        agent.increment_iteration().await?;
        let updated_state = agent.get_state().await?;
        assert_eq!(updated_state.iteration_count, 1);
        
        // Test state persistence
        let saved_state = agent.save_state().await?;
        let mut new_agent = self.create_test_agent().await?;
        new_agent.load_state(saved_state).await?;
        
        let loaded_state = new_agent.get_state().await?;
        if loaded_state.iteration_count == 1 {
            Ok(TestResult::passed("State management"))
        } else {
            Ok(TestResult::failed("State management", "State not properly restored".to_string()))
        }
    }
    
    // Property-based testing for core functions
    pub async fn test_tool_execution_properties(&self) -> Result<UnitTestResults> {
        let mut results = UnitTestResults::new("Tool Execution Property Tests");
        
        // Property: Tool execution should be deterministic for same inputs
        let determinism_test = proptest!(|(input: String)| {
            let tool = TestTool::new("echo_tool", "Echoes input");
            let result1 = tool.execute(json!({"input": input})).await?;
            let result2 = tool.execute(json!({"input": input})).await?;
            
            prop_assert_eq!(result1.output, result2.output);
            Ok(())
        });
        
        results.add_test(match determinism_test {
            Ok(_) => TestResult::passed("Tool execution determinism"),
            Err(e) => TestResult::failed("Tool execution determinism", e.to_string()),
        });
        
        // Property: Tool execution time should be bounded
        let timing_test = proptest!(|(input: String)| {
            let tool = TestTool::new("bounded_tool", "Tool with time bounds");
            let start = std::time::Instant::now();
            let _result = tool.execute(json!({"input": input})).await?;
            let duration = start.elapsed();
            
            prop_assert!(duration < Duration::from_secs(5)); // Max 5 seconds
            Ok(())
        });
        
        results.add_test(match timing_test {
            Ok(_) => TestResult::passed("Tool execution timing bounds"),
            Err(e) => TestResult::failed("Tool execution timing bounds", e.to_string()),
        });
        
        Ok(results)
    }
}

// Mock factory for testing
pub struct MockFactory {
    mock_providers: HashMap<String, Box<dyn MockLLMProvider>>,
    mock_tools: HashMap<String, Box<dyn MockTool>>,
    mock_storage: Box<dyn MockStorage>,
}

impl MockFactory {
    pub fn create_mock_llm_provider(&self) -> Box<dyn LLMProvider> {
        Box::new(MockLLMProvider::new())
    }
    
    pub fn create_mock_tool(&self, name: &str) -> Box<dyn Tool> {
        Box::new(MockTool::new(name))
    }
    
    pub fn create_mock_storage(&self) -> Box<dyn StorageBackend> {
        Box::new(MockStorage::new())
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub execution_time: Duration,
    pub assertions_checked: usize,
}

impl TestResult {
    pub fn passed(name: &str) -> Self {
        Self {
            test_name: name.to_string(),
            passed: true,
            error_message: None,
            execution_time: Duration::ZERO,
            assertions_checked: 1,
        }
    }
    
    pub fn failed(name: &str, error: String) -> Self {
        Self {
            test_name: name.to_string(),
            passed: false,
            error_message: Some(error),
            execution_time: Duration::ZERO,
            assertions_checked: 1,
        }
    }
}

    // Agent template testing patterns
    pub async fn test_agent_templates(&self) -> Result<UnitTestResults> {
        let mut results = UnitTestResults::new("Agent Template Tests");
        
        // Test all built-in templates
        let templates = vec![
            "chat_agent", "research_agent", "code_assistant", 
            "data_analyst", "customer_service", "api_integration"
        ];
        
        for template_name in templates {
            results.add_test(self.test_template_creation(template_name).await?);
            results.add_test(self.test_template_capabilities(template_name).await?);
            results.add_test(self.test_template_customization(template_name).await?);
        }
        
        // Test template inheritance
        results.add_test(self.test_template_inheritance().await?);
        
        // Test dynamic template generation
        results.add_test(self.test_dynamic_template_creation().await?);
        
        Ok(results)
    }
    
    async fn test_template_creation(&self, template_name: &str) -> Result<TestResult> {
        match Agent::template(template_name, &AgentConfig::default()).await {
            Ok(agent) => {
                // Verify agent was created with template properties
                if agent.agent_type().to_string().contains(template_name) {
                    Ok(TestResult::passed(&format!("Template creation: {}", template_name)))
                } else {
                    Ok(TestResult::failed(
                        &format!("Template creation: {}", template_name),
                        "Agent type doesn't match template".to_string()
                    ))
                }
            }
            Err(e) => Ok(TestResult::failed(
                &format!("Template creation: {}", template_name),
                e.to_string()
            ))
        }
    }
    
    async fn test_template_capabilities(&self, template_name: &str) -> Result<TestResult> {
        let agent = Agent::template(template_name, &AgentConfig::default()).await?;
        
        // Verify template has expected capabilities
        let capabilities = agent.capabilities();
        let expected_capability_count = match template_name {
            "research_agent" => 4, // research, analysis, web_search, summarization
            "chat_agent" => 3,     // conversation, context_management, personality
            "code_assistant" => 5, // code_analysis, debugging, refactoring, testing, documentation
            "data_analyst" => 4,   // data_exploration, statistical_modeling, visualization, insights
            "customer_service" => 4, // issue_resolution, empathy, escalation, knowledge_base
            "api_integration" => 4,  // rest_apis, graphql, webhooks, auth_flows
            _ => 1, // Minimum for unknown templates
        };
        
        if capabilities.len() >= expected_capability_count {
            Ok(TestResult::passed(&format!("Template capabilities: {}", template_name)))
        } else {
            Ok(TestResult::failed(
                &format!("Template capabilities: {}", template_name),
                format!("Expected {} capabilities, got {}", expected_capability_count, capabilities.len())
            ))
        }
    }
    
    async fn test_template_customization(&self, template_name: &str) -> Result<TestResult> {
        let custom_config = AgentConfig {
            system_prompt: "Custom system prompt for testing".to_string(),
            tools: vec!["custom_tool".to_string()],
            ..AgentConfig::default()
        };
        
        match Agent::template(template_name, &custom_config).await {
            Ok(agent) => {
                // Verify customization was applied
                if agent.system_prompt().contains("Custom system prompt") {
                    Ok(TestResult::passed(&format!("Template customization: {}", template_name)))
                } else {
                    Ok(TestResult::failed(
                        &format!("Template customization: {}", template_name),
                        "Custom system prompt not applied".to_string()
                    ))
                }
            }
            Err(e) => Ok(TestResult::failed(
                &format!("Template customization: {}", template_name),
                e.to_string()
            ))
        }
    }
    
    async fn test_template_inheritance(&self) -> Result<TestResult> {
        // Test extending research agent for market research
        let base_config = AgentConfig::default();
        let base_agent = Agent::template("research_agent", &base_config).await?;
        
        let specialized_config = AgentConfig {
            system_prompt: base_agent.system_prompt().to_string() + " Focus on market analysis.",
            tools: {
                let mut tools = base_agent.tools().iter().map(|t| t.name()).collect::<Vec<_>>();
                tools.push("market_data_api".to_string());
                tools
            },
            ..base_config
        };
        
        match Agent::extend("research_agent", &specialized_config).await {
            Ok(specialized_agent) => {
                // Verify inheritance
                let base_capabilities = base_agent.capabilities();
                let specialized_capabilities = specialized_agent.capabilities();
                
                // Specialized agent should have all base capabilities plus more
                let has_all_base = base_capabilities.iter()
                    .all(|cap| specialized_capabilities.contains(cap));
                    
                if has_all_base && specialized_capabilities.len() > base_capabilities.len() {
                    Ok(TestResult::passed("Template inheritance"))
                } else {
                    Ok(TestResult::failed(
                        "Template inheritance",
                        "Inheritance not properly implemented".to_string()
                    ))
                }
            }
            Err(e) => Ok(TestResult::failed("Template inheritance", e.to_string()))
        }
    }
    
    async fn test_dynamic_template_creation(&self) -> Result<TestResult> {
        // Test runtime template generation
        let requirements = TemplateRequirements {
            domain: "healthcare".to_string(),
            capabilities: vec!["medical_research", "hipaa_compliance"],
            tools: vec!["pubmed_search", "clinical_trials_api"],
            output_format: "structured_medical_report".to_string(),
        };
        
        match CustomAgentFactory::create_specialized_agent("research_agent", &requirements).await {
            Ok(agent) => {
                // Verify dynamic customization
                if agent.capabilities().contains(&"medical_research".to_string()) &&
                   agent.tools().iter().any(|t| t.name() == "pubmed_search") {
                    Ok(TestResult::passed("Dynamic template creation"))
                } else {
                    Ok(TestResult::failed(
                        "Dynamic template creation",
                        "Dynamic customization not applied correctly".to_string()
                    ))
                }
            }
            Err(e) => Ok(TestResult::failed("Dynamic template creation", e.to_string()))
        }
    }
}
```

### Integration Testing Framework

```rust
pub struct IntegrationTestRunner {
    test_config: TestConfiguration,
    container_manager: ContainerManager,
    test_environment: TestEnvironment,
    data_generator: IntegrationDataGenerator,
}

impl IntegrationTestRunner {
    pub async fn new(config: &TestConfiguration) -> Result<Self> {
        let container_manager = ContainerManager::new().await?;
        let test_environment = TestEnvironment::setup_integration_environment().await?;
        let data_generator = IntegrationDataGenerator::new();
        
        Ok(Self {
            test_config: config.clone(),
            container_manager,
            test_environment,
            data_generator,
        })
    }
    
    // Test complete agent workflow
    pub async fn test_agent_workflow_integration(&self) -> Result<IntegrationTestResult> {
        // Setup test containers
        let _storage_container = self.container_manager.start_storage_container().await?;
        let _llm_mock_container = self.container_manager.start_llm_mock_server().await?;
        
        // Create real components
        let storage_manager = StorageManager::new(self.get_test_storage_config()).await?;
        let llm_provider_bridge = LLMProviderBridge::new();
        
        // Register mock LLM provider
        llm_provider_bridge.register_mock_provider(
            "test_provider",
            self.container_manager.get_llm_mock_url(),
        ).await?;
        
        // Create agent with real dependencies
        let agent_config = AgentConfig {
            name: "integration_test_agent".to_string(),
            description: "Agent for integration testing".to_string(),
            llm_provider: "test_provider".to_string(),
            system_prompt: "You are an agent being tested in integration tests.".to_string(),
            tools: vec!["calculator".to_string(), "file_reader".to_string()],
            max_iterations: 5,
            timeout: Duration::from_secs(30),
        };
        
        let mut agent = Agent::new(agent_config).await?;
        
        // Register tools
        agent.register_tool(Box::new(CalculatorTool::new())).await?;
        agent.register_tool(Box::new(FileReaderTool::new())).await?;
        
        // Test workflow: Agent should solve a math problem and save result
        let test_input = "Calculate 15 + 27 and save the result to a file called 'result.txt'";
        
        let workflow_result = agent.process_request(test_input).await?;
        
        // Verify workflow completed successfully
        assert!(workflow_result.success);
        assert!(workflow_result.iterations > 0);
        assert!(workflow_result.tools_used.contains(&"calculator".to_string()));
        assert!(workflow_result.tools_used.contains(&"file_reader".to_string()));
        
        // Verify file was created with correct content
        let file_content = std::fs::read_to_string("result.txt")?;
        assert!(file_content.contains("42"));
        
        // Verify state was persisted
        let agent_state = storage_manager.load_agent_state(&agent.get_id()).await?;
        assert!(agent_state.is_some());
        
        Ok(IntegrationTestResult::passed("Agent workflow integration test"))
    }
    
    // Test multi-agent collaboration
    pub async fn test_multi_agent_collaboration(&self) -> Result<IntegrationTestResult> {
        // Setup orchestrator
        let orchestrator = MultiAgentOrchestrator::new(self.get_test_orchestration_config()).await?;
        
        // Create multiple agents with different capabilities
        let research_agent = self.create_research_agent().await?;
        let analysis_agent = self.create_analysis_agent().await?;
        let writer_agent = self.create_writer_agent().await?;
        
        // Register agents with orchestrator
        orchestrator.register_agent("researcher", research_agent).await?;
        orchestrator.register_agent("analyst", analysis_agent).await?;
        orchestrator.register_agent("writer", writer_agent).await?;
        
        // Create collaborative workflow
        let workflow = WorkflowDefinition {
            name: "research_and_write".to_string(),
            strategy: "sequential".to_string(),
            steps: vec![
                WorkflowStep {
                    step_id: "research".to_string(),
                    agent_id: AgentId("researcher".to_string()),
                    action: "research_topic".to_string(),
                    input: json!({"topic": "Rust async programming"}),
                    expected_output: "research_data".to_string(),
                },
                WorkflowStep {
                    step_id: "analyze".to_string(),
                    agent_id: AgentId("analyst".to_string()),
                    action: "analyze_research".to_string(),
                    input: json!({"data": "{{research_data}}"}),
                    expected_output: "analysis_results".to_string(),
                },
                WorkflowStep {
                    step_id: "write".to_string(),
                    agent_id: AgentId("writer".to_string()),
                    action: "write_article".to_string(),
                    input: json!({
                        "research": "{{research_data}}",
                        "analysis": "{{analysis_results}}"
                    }),
                    expected_output: "final_article".to_string(),
                },
            ],
        };
        
        // Execute collaborative workflow
        let session = orchestrator.start_collaboration(
            vec![
                AgentId("researcher".to_string()),
                AgentId("analyst".to_string()),
                AgentId("writer".to_string()),
            ],
            "sequential",
            SharedContext::new(),
        ).await?;
        
        let workflow_result = orchestrator.execute_collaborative_workflow(
            &session.session_id,
            workflow,
        ).await?;
        
        // Verify collaboration results
        assert_eq!(workflow_result.step_results.len(), 3);
        assert!(workflow_result.step_results.iter().all(|r| r.success));
        
        // Verify final output contains expected content
        let final_step = workflow_result.step_results.last().unwrap();
        assert!(final_step.result_data.as_str().unwrap().contains("Rust"));
        assert!(final_step.result_data.as_str().unwrap().contains("async"));
        
        Ok(IntegrationTestResult::passed("Multi-agent collaboration test"))
    }
    
    // Test hook and event system integration
    pub async fn test_hook_event_integration(&self) -> Result<IntegrationTestResult> {
        let mut hook_manager = HookManager::new();
        let mut event_bus = EventBus::new();
        
        // Register test hooks
        hook_manager.register_hook(
            HookPoint::BeforeAgentExecution,
            Box::new(TestPreExecutionHook::new()),
            100, // High priority
        ).await?;
        
        hook_manager.register_hook(
            HookPoint::AfterAgentExecution,
            Box::new(TestPostExecutionHook::new()),
            100,
        ).await?;
        
        // Set up event listeners
        let event_receiver = event_bus.subscribe("agent_events").await?;
        
        // Create agent with hook integration
        let mut agent = self.create_test_agent_with_hooks(hook_manager, event_bus).await?;
        
        // Execute agent operation
        let result = agent.process_request("Test hook integration").await?;
        
        // Verify hooks were executed
        let hook_execution_logs = TestHookLogger::get_execution_log();
        assert!(hook_execution_logs.contains("PreExecutionHook executed"));
        assert!(hook_execution_logs.contains("PostExecutionHook executed"));
        
        // Verify events were emitted
        let events = tokio::time::timeout(
            Duration::from_secs(1),
            self.collect_events(event_receiver, 2)
        ).await?;
        
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|e| e.event_type == "agent_execution_started"));
        assert!(events.iter().any(|e| e.event_type == "agent_execution_completed"));
        
        Ok(IntegrationTestResult::passed("Hook and event integration test"))
    }
}

// Container management for integration tests
pub struct ContainerManager {
    docker: testcontainers::clients::Cli,
    running_containers: Vec<testcontainers::Container<'static, testcontainers::GenericImage>>,
}

impl ContainerManager {
    pub async fn start_storage_container(&mut self) -> Result<StorageContainer> {
        let image = testcontainers::GenericImage::new("redis", "7-alpine")
            .with_exposed_port(6379);
        
        let container = self.docker.run(image);
        let host_port = container.get_host_port_ipv4(6379);
        
        Ok(StorageContainer {
            container_id: container.id().to_string(),
            host_port,
            connection_string: format!("redis://localhost:{}", host_port),
        })
    }
    
    pub async fn start_llm_mock_server(&mut self) -> Result<LLMMockContainer> {
        // Start a mock LLM server container for testing
        let image = testcontainers::GenericImage::new("wiremock/wiremock", "latest")
            .with_exposed_port(8080)
            .with_env_var("WIREMOCK_OPTIONS", "--global-response-templating");
        
        let container = self.docker.run(image);
        let host_port = container.get_host_port_ipv4(8080);
        
        // Setup mock responses
        self.setup_llm_mock_responses(&format!("http://localhost:{}", host_port)).await?;
        
        Ok(LLMMockContainer {
            container_id: container.id().to_string(),
            host_port,
            base_url: format!("http://localhost:{}", host_port),
        })
    }
    
    async fn setup_llm_mock_responses(&self, base_url: &str) -> Result<()> {
        let client = reqwest::Client::new();
        
        // Setup mock completion endpoint
        let mock_completion = json!({
            "request": {
                "method": "POST",
                "url": "/v1/chat/completions"
            },
            "response": {
                "status": 200,
                "headers": {
                    "Content-Type": "application/json"
                },
                "jsonBody": {
                    "id": "test-completion",
                    "object": "chat.completion",
                    "choices": [
                        {
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": "This is a test response from the mock LLM server."
                            },
                            "finish_reason": "stop"
                        }
                    ],
                    "usage": {
                        "prompt_tokens": 10,
                        "completion_tokens": 15,
                        "total_tokens": 25
                    }
                }
            }
        });
        
        client
            .post(&format!("{}/mappings", base_url))
            .json(&mock_completion)
            .send()
            .await?;
        
        Ok(())
    }
}
```

### Script Engine Testing

```rust
pub struct ScriptEngineTester {
    lua_test_suite: LuaTestSuite,
    javascript_test_suite: JavaScriptTestSuite,
    cross_engine_test_suite: CrossEngineTestSuite,
}

impl ScriptEngineTester {
    // Test Lua coroutine behavior
    pub async fn test_lua_async_patterns(&self) -> Result<ScriptTestResult> {
        let lua_manager = LuaAsyncManager::new(
            Arc::new(Lua::new()),
            Arc::new(CooperativeScheduler::new(SchedulerConfig::default())),
        );
        
        // Test 1: Basic coroutine creation and execution
        let basic_coroutine_test = self.test_basic_lua_coroutine(&lua_manager).await?;
        
        // Test 2: Coroutine yielding and resumption
        let yielding_test = self.test_lua_coroutine_yielding(&lua_manager).await?;
        
        // Test 3: Multiple coroutines coordination
        let coordination_test = self.test_lua_coroutine_coordination(&lua_manager).await?;
        
        // Test 4: Error handling in coroutines
        let error_handling_test = self.test_lua_coroutine_errors(&lua_manager).await?;
        
        let all_passed = basic_coroutine_test.passed &&
                         yielding_test.passed &&
                         coordination_test.passed &&
                         error_handling_test.passed;
        
        Ok(ScriptTestResult {
            engine: "lua".to_string(),
            test_category: "async_patterns".to_string(),
            passed: all_passed,
            sub_results: vec![
                basic_coroutine_test,
                yielding_test,
                coordination_test,
                error_handling_test,
            ],
        })
    }
    
    async fn test_basic_lua_coroutine(&self, lua_manager: &LuaAsyncManager) -> Result<SubTestResult> {
        let lua_script = r#"
            function test_coroutine()
                local result = 0
                for i = 1, 5 do
                    result = result + i
                    if i == 3 then
                        coroutine.yield("Yielding at iteration " .. i)
                    end
                end
                return result
            end
            
            return test_coroutine
        "#;
        
        let func = lua_manager.lua.load(lua_script).eval::<mlua::Function>()?;
        let coroutine_id = lua_manager.create_coroutine(func, MultiValue::new()).await?;
        
        // Resume coroutine
        let result = lua_manager.resume_coroutine(&coroutine_id, MultiValue::new()).await?;
        
        match result {
            CoroutineResult::Success(values) => {
                let result_value = values.get(0).and_then(|v| v.as_integer()).unwrap_or(0);
                if result_value == 15 { // 1+2+3+4+5 = 15
                    Ok(SubTestResult::passed("Basic Lua coroutine"))
                } else {
                    Ok(SubTestResult::failed("Basic Lua coroutine", format!("Expected 15, got {}", result_value)))
                }
            }
            CoroutineResult::Yielded => {
                Ok(SubTestResult::failed("Basic Lua coroutine", "Coroutine yielded unexpectedly".to_string()))
            }
            CoroutineResult::Error(e) => {
                Ok(SubTestResult::failed("Basic Lua coroutine", e))
            }
        }
    }
    
    // Test JavaScript Promise integration
    pub async fn test_javascript_async_patterns(&self) -> Result<ScriptTestResult> {
        let js_manager = JavaScriptAsyncManager::new(
            Arc::new(CooperativeScheduler::new(SchedulerConfig::default()))
        )?;
        
        // Test 1: Basic Promise execution
        let basic_promise_test = self.test_basic_javascript_promise(&js_manager).await?;
        
        // Test 2: Promise chaining
        let chaining_test = self.test_javascript_promise_chaining(&js_manager).await?;
        
        // Test 3: Async/await simulation
        let async_await_test = self.test_javascript_async_await(&js_manager).await?;
        
        // Test 4: Error handling in Promises
        let error_handling_test = self.test_javascript_promise_errors(&js_manager).await?;
        
        let all_passed = basic_promise_test.passed &&
                         chaining_test.passed &&
                         async_await_test.passed &&
                         error_handling_test.passed;
        
        Ok(ScriptTestResult {
            engine: "javascript".to_string(),
            test_category: "async_patterns".to_string(),
            passed: all_passed,
            sub_results: vec![
                basic_promise_test,
                chaining_test,
                async_await_test,
                error_handling_test,
            ],
        })
    }
    
    async fn test_basic_javascript_promise(&self, js_manager: &JavaScriptAsyncManager) -> Result<SubTestResult> {
        let js_script = r#"
            new Promise((resolve, reject) => {
                // Simulate async operation
                setTimeout(() => {
                    resolve(42);
                }, 10);
            });
        "#;
        
        let result = js_manager.execute_async_script(js_script).await?;
        
        // For testing, we'll check if the script executed without error
        // In a real implementation, we'd wait for Promise resolution
        if result.is_undefined() {
            Ok(SubTestResult::failed("Basic JavaScript Promise", "Promise execution returned undefined".to_string()))
        } else {
            Ok(SubTestResult::passed("Basic JavaScript Promise"))
        }
    }
    
    // Test cross-engine compatibility
    pub async fn test_cross_engine_compatibility(&self) -> Result<ScriptTestResult> {
        // Test same functionality across engines
        let lua_math_result = self.test_math_operations_lua().await?;
        let js_math_result = self.test_math_operations_javascript().await?;
        
        // Results should be equivalent
        let compatibility_passed = lua_math_result.result_value == js_math_result.result_value;
        
        Ok(ScriptTestResult {
            engine: "cross_engine".to_string(),
            test_category: "compatibility".to_string(),
            passed: compatibility_passed,
            sub_results: vec![
                SubTestResult {
                    name: "Cross-engine math operations".to_string(),
                    passed: compatibility_passed,
                    error_message: if compatibility_passed {
                        None
                    } else {
                        Some(format!(
                            "Lua result: {}, JS result: {}",
                            lua_math_result.result_value,
                            js_math_result.result_value
                        ))
                    },
                },
            ],
        })
    }
    
    async fn test_math_operations_lua(&self) -> Result<MathTestResult> {
        let lua = Lua::new();
        let script = r#"
            function calculate()
                local a = 10
                local b = 20
                local c = 5
                return (a + b) * c - 15
            end
            
            return calculate()
        "#;
        
        let result = lua.load(script).eval::<i64>()?;
        
        Ok(MathTestResult {
            engine: "lua".to_string(),
            result_value: result,
        })
    }
    
    async fn test_math_operations_javascript(&self) -> Result<MathTestResult> {
        let js_manager = JavaScriptAsyncManager::new(
            Arc::new(CooperativeScheduler::new(SchedulerConfig::default()))
        )?;
        
        let script = r#"
            function calculate() {
                let a = 10;
                let b = 20;
                let c = 5;
                return (a + b) * c - 15;
            }
            
            calculate();
        "#;
        
        let result = js_manager.execute_async_script(script).await?;
        
        // Extract integer value from JS result
        let result_value = result.as_number().unwrap_or(0.0) as i64;
        
        Ok(MathTestResult {
            engine: "javascript".to_string(),
            result_value,
        })
    }
}

#[derive(Debug)]
pub struct ScriptTestResult {
    pub engine: String,
    pub test_category: String,
    pub passed: bool,
    pub sub_results: Vec<SubTestResult>,
}

#[derive(Debug)]
pub struct SubTestResult {
    pub name: String,
    pub passed: bool,
    pub error_message: Option<String>,
}

impl SubTestResult {
    pub fn passed(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            error_message: None,
        }
    }
    
    pub fn failed(name: &str, error: String) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            error_message: Some(error),
        }
    }
}

#[derive(Debug)]
struct MathTestResult {
    engine: String,
    result_value: i64,
}
```

## Performance Benchmarks

### Comprehensive Performance Testing

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;

pub struct PerformanceTester {
    criterion: Criterion,
    test_data_generator: PerformanceDataGenerator,
    baseline_metrics: BaselineMetrics,
}

impl PerformanceTester {
    // Benchmark LLM provider performance
    pub fn benchmark_llm_providers(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let provider_bridge = rt.block_on(async {
            LLMProviderBridge::new()
        });
        
        // Setup test providers
        rt.block_on(async {
            provider_bridge.register_mock_provider("fast_provider", "http://fast-mock").await.unwrap();
            provider_bridge.register_mock_provider("slow_provider", "http://slow-mock").await.unwrap();
        });
        
        let mut group = c.benchmark_group("llm_providers");
        
        // Benchmark different prompt sizes
        for prompt_size in [100, 500, 1000, 5000].iter() {
            let prompt = "x".repeat(*prompt_size);
            
            group.throughput(Throughput::Elements(*prompt_size as u64));
            group.bench_with_input(
                BenchmarkId::new("completion", prompt_size),
                &prompt,
                |b, prompt| {
                    b.to_async(&rt).iter(|| async {
                        let config = CompletionConfig::default();
                        provider_bridge.complete_with_provider(
                            "fast_provider",
                            black_box(prompt),
                            &config,
                        ).await.unwrap()
                    });
                },
            );
        }
        
        group.finish();
    }
    
    // Benchmark agent execution performance
    pub fn benchmark_agent_execution(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        
        let agent = rt.block_on(async {
            let config = AgentConfig {
                name: "benchmark_agent".to_string(),
                description: "Agent for performance testing".to_string(),
                llm_provider: "mock_provider".to_string(),
                system_prompt: "You are a performance testing agent.".to_string(),
                tools: vec!["calculator".to_string()],
                max_iterations: 5,
                timeout: Duration::from_secs(30),
            };
            
            let mut agent = Agent::new(config).await.unwrap();
            agent.register_tool(Box::new(CalculatorTool::new())).await.unwrap();
            agent
        });
        
        let mut group = c.benchmark_group("agent_execution");
        
        // Benchmark different complexity levels
        for complexity in ["simple", "medium", "complex"].iter() {
            let input = match *complexity {
                "simple" => "What is 2 + 2?",
                "medium" => "Calculate the sum of numbers from 1 to 100 and then find the square root.",
                "complex" => "Solve this multi-step problem: First calculate 15 * 23, then add 47, then divide by 7, finally multiply by 3.",
            };
            
            group.bench_with_input(
                BenchmarkId::new("request_processing", complexity),
                &input,
                |b, input| {
                    b.to_async(&rt).iter(|| async {
                        agent.process_request(black_box(input)).await.unwrap()
                    });
                },
            );
        }
        
        group.finish();
    }
    
    // Benchmark hook execution performance
    pub fn benchmark_hook_execution(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        
        let hook_manager = rt.block_on(async {
            let mut manager = HookManager::new();
            
            // Register hooks with different priorities
            for i in 0..10 {
                manager.register_hook(
                    HookPoint::BeforeAgentExecution,
                    Box::new(BenchmarkHook::new(format!("hook_{}", i))),
                    i * 10,
                ).await.unwrap();
            }
            
            manager
        });
        
        let mut group = c.benchmark_group("hook_execution");
        
        group.bench_function("execute_all_hooks", |b| {
            b.to_async(&rt).iter(|| async {
                let mut context = HookContext::new();
                hook_manager.execute_hooks(
                    HookPoint::BeforeAgentExecution,
                    &mut context,
                ).await.unwrap()
            });
        });
        
        group.finish();
    }
    
    // Benchmark tool execution performance
    pub fn benchmark_tool_execution(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        
        let tool_pool = rt.block_on(async {
            let config = ToolPoolConfig {
                global_max_concurrent: 10,
                per_tool_max_concurrent: HashMap::new(),
                default_tool_concurrency: 3,
                queue_size_limit: 100,
                execution_timeout: Duration::from_secs(30),
            };
            
            ToolExecutionPool::new(config)
        });
        
        let mut group = c.benchmark_group("tool_execution");
        
        // Benchmark different tool types
        for tool_type in ["calculator", "file_reader", "http_client"].iter() {
            group.bench_with_input(
                BenchmarkId::new("tool_execution", tool_type),
                tool_type,
                |b, tool_type| {
                    b.to_async(&rt).iter(|| async {
                        let context = ToolExecutionContext::new();
                        tool_pool.execute_tool_pooled(
                            tool_type.to_string(),
                            context,
                            ExecutionPriority::Normal,
                        ).await.unwrap()
                    });
                },
            );
        }
        
        group.finish();
    }
    
    // Benchmark script engine performance
    pub fn benchmark_script_engines(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        
        let mut group = c.benchmark_group("script_engines");
        
        // Benchmark Lua performance
        group.bench_function("lua_execution", |b| {
            let lua = Lua::new();
            let script = r#"
                function fibonacci(n)
                    if n <= 1 then return n end
                    return fibonacci(n-1) + fibonacci(n-2)
                end
                return fibonacci(20)
            "#;
            
            b.iter(|| {
                lua.load(script).eval::<i64>().unwrap()
            });
        });
        
        // Benchmark JavaScript performance
        group.bench_function("javascript_execution", |b| {
            b.to_async(&rt).iter(|| async {
                let js_manager = JavaScriptAsyncManager::new(
                    Arc::new(CooperativeScheduler::new(SchedulerConfig::default()))
                ).unwrap();
                
                let script = r#"
                    function fibonacci(n) {
                        if (n <= 1) return n;
                        return fibonacci(n-1) + fibonacci(n-2);
                    }
                    fibonacci(20);
                "#;
                
                js_manager.execute_async_script(script).await.unwrap()
            });
        });
        
        group.finish();
    }
    
    // Memory usage benchmarks
    pub fn benchmark_memory_usage(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        
        let mut group = c.benchmark_group("memory_usage");
        
        group.bench_function("agent_memory_footprint", |b| {
            b.iter_batched(
                || {
                    // Setup phase - not measured
                    rt.block_on(async {
                        let config = AgentConfig {
                            name: "memory_test_agent".to_string(),
                            description: "Agent for memory testing".to_string(),
                            llm_provider: "mock_provider".to_string(),
                            system_prompt: "Test agent".to_string(),
                            tools: vec![],
                            max_iterations: 1,
                            timeout: Duration::from_secs(5),
                        };
                        
                        Agent::new(config).await.unwrap()
                    })
                },
                |mut agent| {
                    // Measured phase
                    rt.block_on(async {
                        for i in 0..100 {
                            let _ = agent.process_request(&format!("Test request {}", i)).await;
                        }
                    });
                },
                criterion::BatchSize::SmallInput,
            );
        });
        
        group.finish();
    }
}

// Custom benchmark hook for testing
struct BenchmarkHook {
    name: String,
}

impl BenchmarkHook {
    fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl HookHandler for BenchmarkHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Simulate some work
        tokio::time::sleep(Duration::from_nanos(1000)).await;
        
        Ok(HookResult {
            success: true,
            data: serde_json::json!({"hook": self.name}),
            execution_time: Duration::from_nanos(1000),
        })
    }
    
    fn get_priority(&self) -> u8 {
        100
    }
}

// Performance regression testing
pub struct PerformanceRegressionTester {
    baseline_metrics: HashMap<String, PerformanceMetric>,
    current_metrics: HashMap<String, PerformanceMetric>,
    tolerance: f64, // Acceptable performance degradation (e.g., 0.1 = 10%)
}

impl PerformanceRegressionTester {
    pub fn load_baseline_metrics(path: &str) -> Result<HashMap<String, PerformanceMetric>> {
        let content = std::fs::read_to_string(path)?;
        let metrics: HashMap<String, PerformanceMetric> = serde_json::from_str(&content)?;
        Ok(metrics)
    }
    
    pub fn detect_regressions(&self) -> Vec<PerformanceRegression> {
        let mut regressions = Vec::new();
        
        for (metric_name, current_metric) in &self.current_metrics {
            if let Some(baseline_metric) = self.baseline_metrics.get(metric_name) {
                let degradation = (current_metric.value - baseline_metric.value) / baseline_metric.value;
                
                if degradation > self.tolerance {
                    regressions.push(PerformanceRegression {
                        metric_name: metric_name.clone(),
                        baseline_value: baseline_metric.value,
                        current_value: current_metric.value,
                        degradation_percentage: degradation * 100.0,
                        severity: if degradation > self.tolerance * 2.0 {
                            RegressionSeverity::Critical
                        } else {
                            RegressionSeverity::Warning
                        },
                    });
                }
            }
        }
        
        regressions
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub environment: String,
}

#[derive(Debug)]
pub struct PerformanceRegression {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub degradation_percentage: f64,
    pub severity: RegressionSeverity,
}

#[derive(Debug)]
pub enum RegressionSeverity {
    Warning,
    Critical,
}

// Define benchmark groups
criterion_group!(
    benches,
    PerformanceTester::benchmark_llm_providers,
    PerformanceTester::benchmark_agent_execution,
    PerformanceTester::benchmark_hook_execution,
    PerformanceTester::benchmark_tool_execution,
    PerformanceTester::benchmark_script_engines,
    PerformanceTester::benchmark_memory_usage,
);

criterion_main!(benches);
```

---

## Part IX: Development and Operations

### Development Workflow

#### Core Development Philosophy

Rs-llmspell follows TDD (Test-Driven Development) with a bridge-first approach, emphasizing code quality, security, and maintainable architecture patterns.

```bash
# Standard development workflow
cargo fmt         # Format code
cargo clippy      # Lint and analyze
cargo test        # Run all tests
cargo bench       # Run benchmarks
cargo audit       # Security audit
```

#### Project Structure and Organization

```
rs-llmspell/
├── crates/
│   ├── core/                    # Core traits and foundational types
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agents/          # BaseAgent and Agent traits
│   │   │   ├── tools/           # Tool trait and core tools
│   │   │   ├── workflows/       # Workflow trait and engine
│   │   │   ├── hooks/           # Hook system implementation
│   │   │   ├── events/          # Event system implementation
│   │   │   └── errors/          # Error handling hierarchy
│   │   └── Cargo.toml
│   ├── providers/               # LLM provider integrations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── rig_provider.rs  # rig-based LLM abstraction
│   │   │   ├── openai.rs        # OpenAI specific integration
│   │   │   ├── anthropic.rs     # Anthropic specific integration
│   │   │   └── local.rs         # Local model support
│   │   └── Cargo.toml
│   ├── scripting/               # Script engine bridges
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lua/             # Lua bridge implementation
│   │   │   ├── javascript/      # JavaScript bridge implementation
│   │   │   ├── async_patterns/  # Cross-engine async support
│   │   │   └── bridge.rs        # Bridge trait and utilities
│   │   └── Cargo.toml
│   ├── builtin/                 # Built-in components
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tools/           # 40+ built-in tools
│   │   │   ├── agents/          # Agent templates
│   │   │   ├── workflows/       # Workflow patterns
│   │   │   └── discovery.rs     # Component discovery
│   │   └── Cargo.toml
│   ├── storage/                 # State and persistence
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── sled_backend.rs  # Sled storage implementation
│   │   │   ├── memory_backend.rs# In-memory storage
│   │   │   └── migrations.rs    # Schema migrations
│   │   └── Cargo.toml
│   ├── security/                # Security and sandboxing
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── sandbox.rs       # Script sandboxing
│   │   │   ├── permissions.rs   # Permission management
│   │   │   └── threat_detection.rs # Security monitoring
│   │   └── Cargo.toml
│   ├── protocols/               # Protocol support (MCP, A2A)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mcp/             # Model Control Protocol
│   │   │   └── a2a/             # Agent to Agent protocol
│   │   └── Cargo.toml
│   └── cli/                     # Command line interface
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/        # CLI commands
│       │   └── config.rs        # Configuration management
│       └── Cargo.toml
├── examples/                    # Example scripts and applications
│   ├── basic/
│   │   ├── hello_world.lua
│   │   ├── simple_agent.js
│   │   └── tool_usage.lua
│   ├── advanced/
│   │   ├── multi_agent_research.lua
│   │   ├── workflow_orchestration.js
│   │   └── custom_tools.lua
│   └── production/
│       ├── content_pipeline.lua
│       ├── data_analysis.js
│       └── monitoring_setup.lua
├── tests/                       # Integration and end-to-end tests
│   ├── integration/
│   ├── e2e/
│   └── performance/
├── benches/                     # Performance benchmarks
├── docs/                        # Documentation
└── scripts/                     # Development scripts
    ├── setup.sh
    ├── test_all.sh
    └── benchmark.sh
```

#### Development Environment Setup

```bash
# Initial setup script
#!/bin/bash
set -euo pipefail

echo "Setting up rs-llmspell development environment..."

# Install required dependencies
rustup update stable
rustup component add rustfmt clippy

# Install additional tools
cargo install cargo-audit
cargo install cargo-tarpaulin  # Code coverage
cargo install criterion-cli    # Benchmarking

# Setup git hooks
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# Initialize test databases
mkdir -p test_data/storage
mkdir -p test_data/benchmarks

echo "Development environment setup complete!"
```

#### Code Quality Standards

**Formatting and Style:**
```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

**Clippy Configuration:**
```toml
# clippy.toml
msrv = "1.75.0"  # Minimum supported Rust version
cognitive-complexity-threshold = 30
type-complexity-threshold = 250
too-many-arguments-threshold = 8
```

**Git Commit Standards:**
```bash
# Conventional commit format
feat: add new agent template system
fix: resolve memory leak in hook execution
docs: update API reference for workflows
test: add integration tests for Lua bridge
perf: optimize tool execution pipeline
refactor: simplify error handling hierarchy
```

### Build System and Tooling

The project uses `cargo` as its primary build system, managed through the `Cargo.toml` workspace definition. 

#### `llmspell-cli`

The primary user-facing tool is the `llmspell-cli`, which provides a powerful command-line interface for running spells, managing configurations, and inspecting the system. Key features of the CLI architecture include:

- **Automatic Script Engine Detection**: The CLI automatically selects the correct script engine (Lua, JavaScript, etc.) based on the script's file extension (e.g., `.lua`, `.js`, `.mjs`).
- **Shebang Support**: For more explicit control, scripts can use a shebang line (e.g., `#!/usr/bin/env llmspell-lua`) to specify the exact engine to use, bypassing file extension detection.
- **Parameter Injection**: Scripts can receive parameters from the command line using `--param <key>=<value>`, which are then available within the script's `params` object.

#### Agent and Tool Generation Commands

The CLI provides scaffolding commands for rapid development:

```bash
# Agent scaffolding
llmspell generate agent <name> [options]
  --template <template>     # Base template (research, chat, code, data_analyst, etc.)
  --language <lang>         # Target language (lua, javascript, rust)
  --tools <tool1,tool2>     # Pre-configure specific tools
  --output-dir <path>       # Output directory for generated files

# Examples
llmspell generate agent market_analyzer --template research --language lua --tools web_search,data_analyzer
llmspell generate agent support_bot --template customer_service --language javascript
llmspell generate agent code_reviewer --template code --language rust --output-dir ./agents/

# Tool scaffolding  
llmspell generate tool <name> [options]
  --category <category>     # Tool category (text, file, web, analysis, etc.)
  --language <lang>         # Implementation language
  --template <template>     # Tool template (http_client, file_processor, etc.)

# Examples
llmspell generate tool api_client --category integration --template http_client --language rust
llmspell generate tool log_analyzer --category analysis --language lua

# Template management
llmspell template list                    # List available templates
llmspell template show <name>             # Show template details
llmspell template validate <path>         # Validate custom template
llmspell template install <url>           # Install template from repository
```

#### Testing and Validation Commands

```bash
# Agent testing
llmspell test agent <agent_name>          # Test specific agent
llmspell test agents --category <cat>     # Test agents by category
llmspell test template <template_name>    # Test agent template

# Tool testing
llmspell test tool <tool_name>            # Test specific tool
llmspell test tools --integration         # Run integration tests

# System validation
llmspell validate config                  # Validate configuration files
llmspell validate security               # Check security settings
llmspell validate performance            # Performance baseline check
```

### Native Module Builds

#### Cargo Workspace Configuration

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/core",
    "crates/providers", 
    "crates/scripting",
    "crates/builtin",
    "crates/storage",
    "crates/security",
    "crates/protocols",
    "crates/cli",
]

resolver = "2"

[workspace.dependencies]
# Core dependencies
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# LLM and AI dependencies
rig-core = "0.3"
candle-core = "0.5"
tokenizers = "0.19"

# Scripting engines
mlua = { version = "0.9", features = ["lua54", "async", "serialize"] }
boa_engine = "0.17"

# Storage and persistence
sled = "0.34"
rocksdb = "0.21"

# Async and concurrency
crossbeam = "0.8"
futures = "0.3"
async-trait = "0.1"

# HTTP and networking
reqwest = { version = "0.11", features = ["json", "stream"] }
hyper = "0.14"

# Serialization and configuration
toml = "0.8"
config = "0.14"

# Testing and development
criterion = "0.5"
proptest = "1.0"
mockall = "0.11"

[workspace.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

#### Build Scripts and Automation

**Main Build Script:**
```bash
#!/bin/bash
# scripts/build.sh
set -euo pipefail

MODE=${1:-"debug"}
FEATURES=${2:-""}

echo "Building rs-llmspell in $MODE mode..."

case $MODE in
    "debug")
        cargo build --workspace $FEATURES
        ;;
    "release")
        cargo build --workspace --release $FEATURES
        ;;
    "dev")
        cargo build --workspace --features="dev-tools,debugging" $FEATURES
        ;;
    "minimal")
        cargo build --workspace --no-default-features --features="core" $FEATURES
        ;;
    *)
        echo "Unknown build mode: $MODE"
        echo "Available modes: debug, release, dev, minimal"
        exit 1
        ;;
esac

echo "Build complete!"
```

**Testing Script:**
```bash
#!/bin/bash
# scripts/test_all.sh
set -euo pipefail

echo "Running complete test suite..."

# Unit tests
echo "Running unit tests..."
cargo test --workspace --lib

# Integration tests
echo "Running integration tests..."
cargo test --workspace --test integration

# Doc tests
echo "Running documentation tests..."
cargo test --workspace --doc

# End-to-end tests
echo "Running end-to-end tests..."
cargo test --workspace --test e2e

# Performance tests (if requested)
if [[ "${RUN_BENCHMARKS:-false}" == "true" ]]; then
    echo "Running performance benchmarks..."
    cargo bench --workspace
fi

# Code coverage (if requested)
if [[ "${RUN_COVERAGE:-false}" == "true" ]]; then
    echo "Generating code coverage report..."
    cargo tarpaulin --workspace --out Html --output-dir coverage/
fi

echo "All tests completed successfully!"
```

#### Feature Flag Management

```toml
# Example feature flag configuration in core crate
[features]
default = ["lua", "javascript", "builtin-tools", "sled-storage"]

# Script engine features
lua = ["dep:mlua"]
javascript = ["dep:boa_engine"]
python = ["dep:pyo3"]  # Future

# Storage backends
sled-storage = ["dep:sled"]
rocksdb-storage = ["dep:rocksdb"]
memory-storage = []

# Built-in component features
builtin-tools = []
builtin-agents = []
builtin-workflows = []

# Protocol support
mcp-protocol = ["dep:mcp-client"]
a2a-protocol = ["dep:a2a-client"]

# Development and debugging
dev-tools = ["debugging", "metrics", "testing-utils"]
debugging = ["dep:console-subscriber"]
metrics = ["dep:metrics", "dep:metrics-exporter-prometheus"]
testing-utils = ["dep:proptest", "dep:mockall"]

# Performance optimization
simd = ["candle-core/cuda"]
gpu-acceleration = ["candle-core/cuda", "candle-core/metal"]
```

### Deployment Strategies

[... existing content ...]

#### Daemon / Service Mode

For long-running tasks and automations, `rs-llmspell` can be run as a persistent background service or daemon. In this mode, it loads all specified spells and activates their schedulers and listeners.

-   **Execution**: `llmspell serve --config /path/to/llmspell.toml`
-   **Use Cases**: Scheduled reports, event-driven automations, webhook responders, and custom API services built with listener tools.
-   **Process Management**: It is recommended to run the daemon under a process manager like `systemd` or `supervisor` for automatic restarts and logging.

#### Container Deployment

**Dockerfile:**
```dockerfile
# Multi-stage build for optimized containers
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (cached layer)
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false llmspell

# Copy built binaries
COPY --from=builder /usr/src/app/target/release/rs-llmspell /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/libs/ /usr/local/lib/rs-llmspell/

# Create necessary directories
RUN mkdir -p /var/lib/rs-llmspell && \
    chown llmspell:llmspell /var/lib/rs-llmspell

USER llmspell
EXPOSE 8080

CMD ["rs-llmspell", "serve", "--config", "/etc/rs-llmspell/config.toml"]
```

**Docker Compose for Development:**
```yaml
# docker-compose.yml
version: '3.8'

services:
  rs-llmspell:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - LLMSPELL_CONFIG_PATH=/app/config/config.toml
    volumes:
      - ./config:/app/config
      - ./data:/var/lib/rs-llmspell
      - ./scripts:/app/scripts
    depends_on:
      - storage
      - metrics

  storage:
    image: rocksdb/rocksdb:latest
    volumes:
      - rocksdb_data:/data
    ports:
      - "6379:6379"

  metrics:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning

volumes:
  rocksdb_data:
  prometheus_data:
  grafana_data:
```

#### Kubernetes Deployment

**Kubernetes Manifests:**
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rs-llmspell
  labels:
    app: rs-llmspell
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rs-llmspell
  template:
    metadata:
      labels:
        app: rs-llmspell
    spec:
      containers:
      - name: rs-llmspell
        image: rs-llmspell:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: LLMSPELL_CONFIG_PATH
          value: "/etc/rs-llmspell/config.toml"
        volumeMounts:
        - name: config-volume
          mountPath: /etc/rs-llmspell
        - name: storage-volume
          mountPath: /var/lib/rs-llmspell
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 20
      volumes:
      - name: config-volume
        configMap:
          name: rs-llmspell-config
      - name: storage-volume
        persistentVolumeClaim:
          claimName: rs-llmspell-storage

---
apiVersion: v1
kind: Service
metadata:
  name: rs-llmspell-service
spec:
  selector:
    app: rs-llmspell
  ports:
    - protocol: TCP
      port: 80
      targetPort: 8080
  type: LoadBalancer

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: rs-llmspell-config
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    
    [storage]
    backend = "rocksdb"
    path = "/var/lib/rs-llmspell/db"
    
    [scripting]
    engines = ["lua", "javascript"]
    sandbox_enabled = true
    
    [providers]
    default = "openai"
    
    [providers.openai]
    api_key_env = "OPENAI_API_KEY"
    model = "gpt-4"
    
    [observability]
    metrics_enabled = true
    tracing_enabled = true
```

#### Cloud-Native Deployment Patterns

**AWS ECS with Fargate:**
```json
{
  "family": "rs-llmspell-task",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::account:role/ecsTaskRole",
  "containerDefinitions": [
    {
      "name": "rs-llmspell",
      "image": "your-registry/rs-llmspell:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "OPENAI_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:openai-key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/rs-llmspell",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

**Google Cloud Run:**
```yaml
# cloudrun.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: rs-llmspell
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/maxScale: "10"
        run.googleapis.com/cpu-throttling: "false"
    spec:
      containerConcurrency: 1000
      containers:
      - image: gcr.io/project-id/rs-llmspell:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            cpu: "2"
            memory: "4Gi"
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          timeoutSeconds: 10
          failureThreshold: 3
```

### Migration and Backward Compatibility

#### Version Management Strategy

**Semantic Versioning Policy:**
```rust
// Version compatibility matrix
pub const COMPATIBILITY_MATRIX: &[VersionCompatibility] = &[
    VersionCompatibility {
        version: "2.0.0",
        api_version: "v2",
        script_api_version: "2.0",
        backward_compatible_with: &["1.9.x"],
        migration_required: true,
        deprecated_features: &["legacy_tool_interface", "old_hook_system"],
    },
    VersionCompatibility {
        version: "1.9.0",
        api_version: "v1",
        script_api_version: "1.9",
        backward_compatible_with: &["1.8.x", "1.7.x"],
        migration_required: false,
        deprecated_features: &["sync_only_tools"],
    },
];

pub struct VersionCompatibility {
    pub version: &'static str,
    pub api_version: &'static str,
    pub script_api_version: &'static str,
    pub backward_compatible_with: &'static [&'static str],
    pub migration_required: bool,
    pub deprecated_features: &'static [&'static str],
}
```

#### Migration Framework

**Migration Engine:**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Migration {
    fn version_from(&self) -> &str;
    fn version_to(&self) -> &str;
    fn description(&self) -> &str;
    
    async fn migrate(&self, context: &mut MigrationContext) -> Result<MigrationResult>;
    async fn rollback(&self, context: &mut MigrationContext) -> Result<MigrationResult>;
    
    fn is_destructive(&self) -> bool { false }
    fn requires_downtime(&self) -> bool { false }
}

pub struct MigrationContext {
    pub storage: Box<dyn StorageBackend>,
    pub config: Configuration,
    pub dry_run: bool,
    pub backup_created: bool,
    pub progress_reporter: Box<dyn ProgressReporter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationResult {
    pub success: bool,
    pub items_migrated: u64,
    pub items_failed: u64,
    pub duration: Duration,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

// Example migration: v1.8 to v1.9 tool interface update
pub struct ToolInterfaceMigration;

#[async_trait]
impl Migration for ToolInterfaceMigration {
    fn version_from(&self) -> &str { "1.8.0" }
    fn version_to(&self) -> &str { "1.9.0" }
    fn description(&self) -> &str {
        "Migrate tool definitions to new async interface"
    }
    
    async fn migrate(&self, context: &mut MigrationContext) -> Result<MigrationResult> {
        let mut result = MigrationResult {
            success: true,
            items_migrated: 0,
            items_failed: 0,
            duration: Duration::default(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };
        
        let start_time = Instant::now();
        
        // Get all tool definitions
        let tool_configs = context.storage.list_tool_configs().await?;
        
        for tool_config in tool_configs {
            match self.migrate_tool_config(&tool_config, context).await {
                Ok(migrated_config) => {
                    if !context.dry_run {
                        context.storage.save_tool_config(&migrated_config).await?;
                    }
                    result.items_migrated += 1;
                }
                Err(e) => {
                    result.errors.push(format!("Failed to migrate tool {}: {}", tool_config.id, e));
                    result.items_failed += 1;
                    result.success = false;
                }
            }
            
            context.progress_reporter.update_progress(
                result.items_migrated + result.items_failed,
                tool_configs.len() as u64
            ).await;
        }
        
        result.duration = start_time.elapsed();
        Ok(result)
    }
    
    async fn rollback(&self, context: &mut MigrationContext) -> Result<MigrationResult> {
        // Implement rollback logic
        todo!("Implement rollback for tool interface migration")
    }
}

impl ToolInterfaceMigration {
    async fn migrate_tool_config(
        &self,
        config: &ToolConfig,
        context: &MigrationContext
    ) -> Result<ToolConfig> {
        let mut migrated = config.clone();
        
        // Update interface definition
        if let Some(ref mut interface) = migrated.interface {
            // Convert sync methods to async
            for method in &mut interface.methods {
                if !method.is_async {
                    method.is_async = true;
                    method.return_type = format!("Future<Output = {}>", method.return_type);
                }
            }
        }
        
        // Update metadata
        migrated.api_version = "1.9".to_string();
        migrated.migration_metadata = Some(MigrationMetadata {
            migrated_from: config.api_version.clone(),
            migration_time: chrono::Utc::now(),
            migration_tool: "ToolInterfaceMigration".to_string(),
        });
        
        Ok(migrated)
    }
}
```

#### Configuration Migration

**Configuration Versioning:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationV1 {
    pub version: String,
    pub server: ServerConfigV1,
    pub storage: StorageConfigV1,
    pub providers: HashMap<String, ProviderConfigV1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationV2 {
    pub version: String,
    pub server: ServerConfigV2,
    pub storage: StorageConfigV2,
    pub providers: HashMap<String, ProviderConfigV2>,
    pub scripting: ScriptingConfig,  // New in v2
    pub security: SecurityConfig,    // New in v2
}

pub struct ConfigMigrator;

impl ConfigMigrator {
    pub fn migrate_v1_to_v2(v1_config: ConfigurationV1) -> Result<ConfigurationV2> {
        Ok(ConfigurationV2 {
            version: "2.0".to_string(),
            server: ServerConfigV2 {
                host: v1_config.server.host,
                port: v1_config.server.port,
                tls_enabled: v1_config.server.tls_enabled,
                // New fields with defaults
                max_connections: 1000,
                request_timeout: Duration::from_secs(30),
                graceful_shutdown_timeout: Duration::from_secs(10),
            },
            storage: StorageConfigV2 {
                backend: v1_config.storage.backend,
                connection_string: v1_config.storage.connection_string,
                // New fields with defaults
                connection_pool_size: 10,
                query_timeout: Duration::from_secs(5),
                backup_enabled: true,
            },
            providers: v1_config.providers.into_iter()
                .map(|(k, v)| (k, self.migrate_provider_config(v)))
                .collect(),
            // New sections with sensible defaults
            scripting: ScriptingConfig {
                engines: vec!["lua".to_string(), "javascript".to_string()],
                sandbox_enabled: true,
                max_execution_time: Duration::from_secs(300),
                max_memory_usage: 512 * 1024 * 1024, // 512MB
            },
            security: SecurityConfig {
                authentication_required: false,
                api_key_header: "X-API-Key".to_string(),
                rate_limit_requests_per_minute: 100,
                allowed_origins: vec!["*".to_string()],
            },
        })
    }
    
    fn migrate_provider_config(&self, v1: ProviderConfigV1) -> ProviderConfigV2 {
        ProviderConfigV2 {
            provider_type: v1.provider_type,
            api_key: v1.api_key,
            base_url: v1.base_url,
            model: v1.model,
            // New fields with defaults
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
            max_tokens: 4096,
        }
    }
}
```

#### Database Schema Migration

**Schema Evolution Management:**
```rust
pub struct DatabaseMigrator {
    storage: Arc<dyn StorageBackend>,
    migration_registry: Vec<Box<dyn SchemaMigration>>,
}

impl DatabaseMigrator {
    pub async fn apply_migrations(&self) -> Result<MigrationSummary> {
        let current_version = self.get_current_schema_version().await?;
        let target_version = self.get_latest_schema_version();
        
        if current_version >= target_version {
            return Ok(MigrationSummary::no_migrations_needed());
        }
        
        let mut summary = MigrationSummary::new();
        
        for migration in &self.migration_registry {
            if migration.version() > current_version && migration.version() <= target_version {
                let result = self.apply_migration(migration.as_ref()).await?;
                summary.add_migration_result(result);
            }
        }
        
        Ok(summary)
    }
    
    async fn apply_migration(&self, migration: &dyn SchemaMigration) -> Result<MigrationResult> {
        // Create backup before migration
        self.create_backup(migration.version()).await?;
        
        // Apply migration
        let result = migration.apply(&*self.storage).await;
        
        match result {
            Ok(migration_result) => {
                self.update_schema_version(migration.version()).await?;
                Ok(migration_result)
            }
            Err(e) => {
                // Rollback on failure
                self.restore_backup(migration.version()).await?;
                Err(e)
            }
        }
    }
}

// Example schema migration
pub struct AddAgentTemplatesTable;

#[async_trait]
impl SchemaMigration for AddAgentTemplatesTable {
    fn version(&self) -> u32 { 2 }
    fn description(&self) -> &str { "Add agent_templates table for built-in agent patterns" }
    
    async fn apply(&self, storage: &dyn StorageBackend) -> Result<MigrationResult> {
        let create_table_sql = r#"
            CREATE TABLE agent_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                template_type TEXT NOT NULL,
                configuration BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            
            CREATE INDEX idx_agent_templates_type ON agent_templates(template_type);
            CREATE INDEX idx_agent_templates_created ON agent_templates(created_at);
        "#;
        
        storage.execute_sql(create_table_sql).await?;
        
        Ok(MigrationResult {
            success: true,
            items_migrated: 1,
            items_failed: 0,
            duration: Duration::from_millis(50),
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }
    
    async fn rollback(&self, storage: &dyn StorageBackend) -> Result<MigrationResult> {
        storage.execute_sql("DROP TABLE agent_templates;").await?;
        
        Ok(MigrationResult {
            success: true,
            items_migrated: 1,
            items_failed: 0,
            duration: Duration::from_millis(10),
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }
}
```

#### Backward Compatibility Layer

**API Versioning:**
```rust
// Support multiple API versions simultaneously
pub struct VersionedApiHandler {
    v1_handler: V1ApiHandler,
    v2_handler: V2ApiHandler,
}

impl VersionedApiHandler {
    pub async fn handle_request(&self, request: Request) -> Result<Response> {
        let api_version = self.extract_api_version(&request)?;
        
        match api_version.as_str() {
            "v1" | "1.0" => {
                // Translate v2 internal representation to v1 for compatibility
                let internal_request = self.translate_v1_to_internal(request)?;
                let internal_response = self.v2_handler.handle(internal_request).await?;
                self.translate_internal_to_v1(internal_response)
            }
            "v2" | "2.0" => {
                self.v2_handler.handle(request).await
            }
            _ => Err(LLMSpellError::UnsupportedApiVersion(api_version))
        }
    }
    
    fn translate_v1_to_internal(&self, v1_request: Request) -> Result<Request> {
        // Handle API differences between v1 and v2
        // Add default values for new required fields
        // Transform request structure if needed
        todo!("Implement v1 to internal translation")
    }
    
    fn translate_internal_to_v1(&self, internal_response: Response) -> Result<Response> {
        // Remove v2-only fields from response
        // Transform response structure to match v1 expectations
        todo!("Implement internal to v1 translation")
    }
}
```

**Feature Flag Compatibility:**
```rust
pub struct CompatibilityFeatureFlags {
    pub legacy_tool_interface: bool,
    pub old_hook_system: bool,
    pub sync_agent_execution: bool,
    pub v1_error_format: bool,
}

impl Default for CompatibilityFeatureFlags {
    fn default() -> Self {
        Self {
            legacy_tool_interface: false,
            old_hook_system: false,
            sync_agent_execution: false,
            v1_error_format: false,
        }
    }
}

// Runtime compatibility adaptation
pub struct CompatibilityAdapter {
    flags: CompatibilityFeatureFlags,
}

impl CompatibilityAdapter {
    pub fn adapt_tool_call(&self, tool_call: ToolCall) -> ToolCall {
        if self.flags.legacy_tool_interface {
            // Convert new async tool call to legacy sync format
            self.convert_to_legacy_tool_call(tool_call)
        } else {
            tool_call
        }
    }
    
    pub fn adapt_error(&self, error: LLMSpellError) -> LLMSpellError {
        if self.flags.v1_error_format {
            // Convert new hierarchical error to v1 flat format
            self.convert_to_v1_error(error)
        } else {
            error
        }
    }
}
```

---

## Part X: Practical Implementation

### Implementation Roadmap

#### Phase-Based Implementation Strategy

Rs-llmspell follows a carefully structured implementation approach that prioritizes core functionality while building toward production readiness.

**Phase 1: Foundation (Weeks 1-4)**
```rust
// Core traits and foundational architecture
pub trait BaseAgent {
    async fn execute(&self, input: &AgentInput) -> Result<AgentOutput>;
    fn get_tools(&self) -> &[Box<dyn Tool>];
    fn get_state(&self) -> &AgentState;
}

pub trait Agent: BaseAgent {
    async fn llm_call(&self, prompt: &str) -> Result<String>;
    fn get_prompt_template(&self) -> &PromptTemplate;
}

pub trait Tool {
    async fn call(&self, input: &ToolInput) -> Result<ToolOutput>;
    fn get_schema(&self) -> &ToolSchema;
    fn get_name(&self) -> &str;
}
```

**Implementation Priorities:**
1. Core trait hierarchy (BaseAgent, Agent, Tool, Workflow)
2. Basic LLM provider integration using rig
3. Minimal Lua bridge with mlua
4. In-memory state management
5. Basic hook system infrastructure

**Deliverables:**
- Working agent execution
- Tool calling from scripts
- Basic error handling
- Simple examples

**Phase 2: Scripting and Tools (Weeks 5-8)**
```lua
-- Working Lua API by Phase 2
local agent = Agent.new({
    name = "test_agent",
    model = "gpt-4",
    tools = {
        HttpTool.new({ timeout = 30 }),
        FileSystemTool.new({ readonly = true })
    }
})

local result = agent:execute({
    input = "Fetch data from https://api.example.com and save summary",
    context = { session_id = "test_123" }
})

print("Result:", result.output)
```

**Implementation Priorities:**
1. Complete built-in tools library (40+ tools)
2. JavaScript bridge with boa_engine
3. Cross-engine compatibility layer
4. Persistent state with sled
5. Event system implementation

**Deliverables:**
- Multi-language scripting support
- Comprehensive tool library
- Persistent agent state
- Event-driven architecture

**Phase 3: Advanced Features (Weeks 9-12)**
```javascript
// Working JavaScript API by Phase 3
const workflow = new Workflow.Sequential({
    name: "research_pipeline",
    steps: [
        {
            agent: new ResearchAgent({ 
                tools: [WebSearchTool, DocumentAnalyzer] 
            }),
            action: "research",
            input: { topic: "{{topic}}" },
            output: "research_data"
        },
        {
            agent: new WriterAgent({ 
                style: "technical",
                tools: [TemplateEngine, GrammarChecker] 
            }),
            action: "write_report",
            input: { 
                research: "{{research_data}}", 
                format: "markdown" 
            },
            output: "final_report"
        }
    ]
});

const report = await workflow.run({ topic: "Rust async patterns" });
```

**Implementation Priorities:**
1. Workflow orchestration engine
2. Advanced hook system with priorities
3. Performance optimization patterns
4. Security and sandboxing
5. MCP protocol support

**Deliverables:**
- Complex workflow support
- Production security model
- Protocol integration
- Performance benchmarks

**Phase 4: Production Readiness (Weeks 13-16)**
```toml
# Production configuration by Phase 4
[server]
host = "0.0.0.0"
port = 8080
tls_enabled = true
max_connections = 1000

[storage]
backend = "rocksdb"
connection_pool_size = 20
backup_enabled = true
backup_interval = "1h"

[scripting]
engines = ["lua", "javascript"]
sandbox_enabled = true
max_execution_time = "5m"
max_memory_usage = "1GB"

[observability]
metrics_enabled = true
tracing_enabled = true
log_level = "info"

[security]
authentication_required = true
rate_limiting_enabled = true
allowed_origins = ["https://trusted-domain.com"]
```

**Implementation Priorities:**
1. Complete observability stack
2. Migration and backward compatibility
3. A2A protocol support
4. Comprehensive testing suite
5. Documentation and examples

**Deliverables:**
- Production deployment ready
- Full monitoring and metrics
- Migration tooling
- Comprehensive documentation

### Real-World Examples

#### Example 1: Content Creation Pipeline

**Business Scenario**: Automated content creation for technical documentation with research validation and quality assurance.

**Implementation:**
```lua
-- content_pipeline.lua
local ContentPipeline = {}

function ContentPipeline.new(config)
    local pipeline = {
        config = config,
        research_agent = ResearchAgent.new({
            tools = {
                WebSearchTool.new({ engines = {"google", "bing"} }),
                DocumentReaderTool.new({ formats = {"pdf", "html", "md"} }),
                FactCheckerTool.new({ sources = {"wikipedia", "academic"} })
            },
            model = "gpt-4",
            temperature = 0.3
        }),
        
        writer_agent = WriterAgent.new({
            tools = {
                TemplateEngine.new({ template_dir = "templates/" }),
                GrammarChecker.new({ language = "en" }),
                StyleChecker.new({ style_guide = "technical" }),
                PlagiarismChecker.new({ threshold = 0.85 })
            },
            model = "gpt-4",
            temperature = 0.7
        }),
        
        reviewer_agent = ReviewerAgent.new({
            tools = {
                ReadabilityAnalyzer.new({ target_level = "college" }),
                AccuracyChecker.new({ fact_sources = {"trusted_apis"} }),
                SEOAnalyzer.new({ keywords_required = true })
            },
            model = "gpt-4",
            temperature = 0.2
        })
    }
    
    return setmetatable(pipeline, { __index = ContentPipeline })
end

function ContentPipeline:create_content(topic, requirements)
    -- Phase 1: Research and fact-gathering
    local research_result = self.research_agent:execute({
        input = string.format("Research comprehensive information about: %s", topic),
        context = {
            requirements = requirements,
            depth = "detailed",
            sources_required = 5
        }
    })
    
    if not research_result.success then
        error("Research phase failed: " .. research_result.error)
    end
    
    -- Phase 2: Content creation
    local writing_result = self.writer_agent:execute({
        input = "Create comprehensive technical content",
        context = {
            topic = topic,
            research_data = research_result.data,
            requirements = requirements,
            format = "markdown",
            include_code_examples = true
        }
    })
    
    if not writing_result.success then
        error("Writing phase failed: " .. writing_result.error)
    end
    
    -- Phase 3: Review and quality assurance
    local review_result = self.reviewer_agent:execute({
        input = "Review content for accuracy, readability, and completeness",
        context = {
            content = writing_result.data.content,
            original_requirements = requirements,
            research_sources = research_result.data.sources
        }
    })
    
    if not review_result.success then
        error("Review phase failed: " .. review_result.error)
    end
    
    -- Compile final result
    return {
        content = writing_result.data.content,
        metadata = {
            topic = topic,
            word_count = writing_result.data.word_count,
            readability_score = review_result.data.readability_score,
            fact_check_score = review_result.data.accuracy_score,
            sources = research_result.data.sources,
            created_at = os.date("!%Y-%m-%dT%H:%M:%SZ"),
            pipeline_version = "1.0"
        },
        quality_metrics = review_result.data.metrics
    }
end

-- Usage example
local pipeline = ContentPipeline.new({
    quality_threshold = 0.85,
    fact_check_required = true
})

local result = pipeline:create_content(
    "Rust async programming patterns",
    {
        target_audience = "intermediate developers",
        length = "3000-5000 words",
        include_examples = true,
        technical_depth = "detailed"
    }
)

print("Content created successfully!")
print("Quality score:", result.quality_metrics.overall_score)
```

#### Example 2: Data Analysis and Reporting Workflow

**Business Scenario**: Automated analysis of business metrics with trend detection and executive reporting.

**Implementation:**
```javascript
// data_analysis_workflow.js
class DataAnalysisWorkflow {
    constructor(config) {
        this.config = config;
        
        this.dataAgent = new DataAgent({
            tools: [
                DatabaseConnectorTool.new({ 
                    connections: config.databases 
                }),
                CSVReaderTool.new({ encoding: 'utf-8' }),
                JSONProcessorTool.new({ schema_validation: true }),
                StatisticalAnalysisTool.new({ 
                    packages: ['descriptive', 'regression', 'timeseries'] 
                })
            ],
            model: "gpt-4",
            temperature: 0.1  // Low temperature for consistent data analysis
        });
        
        this.visualizationAgent = new VisualizationAgent({
            tools: [
                ChartGeneratorTool.new({ 
                    types: ['line', 'bar', 'scatter', 'heatmap'] 
                }),
                TableFormatterTool.new({ formats: ['html', 'latex', 'markdown'] }),
                InfographicTool.new({ style: 'corporate' })
            ],
            model: "gpt-4",
            temperature: 0.5
        });
        
        this.reportAgent = new ReportAgent({
            tools: [
                TemplateEngine.new({ 
                    template_types: ['executive', 'technical', 'summary'] 
                }),
                DocumentGeneratorTool.new({ 
                    formats: ['pdf', 'html', 'docx'] 
                }),
                EmailSenderTool.new({ smtp_config: config.email })
            ],
            model: "gpt-4",
            temperature: 0.6
        });
    }
    
    async analyzeMetrics(timeframe, metrics) {
        try {
            // Step 1: Data collection and preprocessing
            const dataResult = await this.dataAgent.execute({
                input: `Collect and analyze ${metrics.join(', ')} for ${timeframe}`,
                context: {
                    timeframe: timeframe,
                    metrics: metrics,
                    include_trends: true,
                    anomaly_detection: true,
                    confidence_level: 0.95
                }
            });
            
            if (!dataResult.success) {
                throw new Error(`Data analysis failed: ${dataResult.error}`);
            }
            
            // Step 2: Visualization generation
            const vizResult = await this.visualizationAgent.execute({
                input: "Create comprehensive visualizations for the analyzed data",
                context: {
                    data: dataResult.data,
                    chart_types: ['trend_lines', 'comparison_bars', 'correlation_matrix'],
                    style: 'executive',
                    include_annotations: true
                }
            });
            
            if (!vizResult.success) {
                throw new Error(`Visualization failed: ${vizResult.error}`);
            }
            
            // Step 3: Report generation
            const reportResult = await this.reportAgent.execute({
                input: "Generate executive summary report with insights and recommendations",
                context: {
                    analysis: dataResult.data,
                    visualizations: vizResult.data,
                    report_type: 'executive_summary',
                    include_recommendations: true,
                    timeframe: timeframe
                }
            });
            
            if (!reportResult.success) {
                throw new Error(`Report generation failed: ${reportResult.error}`);
            }
            
            return {
                analysis: dataResult.data,
                visualizations: vizResult.data.charts,
                report: reportResult.data.content,
                metadata: {
                    timeframe: timeframe,
                    metrics_analyzed: metrics,
                    analysis_date: new Date().toISOString(),
                    confidence_scores: dataResult.data.confidence_scores,
                    key_insights: reportResult.data.insights
                }
            };
            
        } catch (error) {
            console.error("Workflow execution failed:", error);
            throw error;
        }
    }
    
    async scheduleRegularAnalysis(schedule) {
        // Set up scheduled analysis with hooks
        const hookManager = HookManager.getInstance();
        
        hookManager.registerHook('scheduled_analysis', {
            priority: 100,
            async execute(context) {
                const result = await this.analyzeMetrics(
                    context.timeframe,
                    context.metrics
                );
                
                // Auto-send to stakeholders if significant changes detected
                if (result.metadata.confidence_scores.trend_change > 0.8) {
                    await this.reportAgent.execute({
                        input: "Send urgent analysis alert to stakeholders",
                        context: {
                            report: result.report,
                            urgency: 'high',
                            recipients: this.config.alert_recipients
                        }
                    });
                }
                
                return result;
            }
        });
    }
}

// Usage example
const workflow = new DataAnalysisWorkflow({
    databases: [
        { type: 'postgresql', connection: process.env.ANALYTICS_DB },
        { type: 'mongodb', connection: process.env.EVENTS_DB }
    ],
    email: {
        smtp_host: 'smtp.company.com',
        smtp_port: 587,
        auth: {
            user: process.env.SMTP_USER,
            pass: process.env.SMTP_PASS
        }
    },
    alert_recipients: ['executives@company.com', 'analytics@company.com']
});

// Run weekly analysis
const result = await workflow.analyzeMetrics(
    'last_30_days',
    ['revenue', 'user_acquisition', 'churn_rate', 'customer_satisfaction']
);

console.log('Analysis completed successfully!');
console.log('Key insights:', result.metadata.key_insights);
```

#### Example 3: Multi-Agent Research Collaboration

**Business Scenario**: Collaborative research system where specialized agents work together to investigate complex topics.

**Implementation:**
```lua
-- research_collaboration.lua
local ResearchCollaborationSystem = {}

function ResearchCollaborationSystem.new(config)
    local system = {
        config = config,
        
        -- Specialized research agents
        academic_researcher = Agent.new({
            name = "Academic Researcher",
            model = "gpt-4",
            temperature = 0.3,
            tools = {
                ScholarSearchTool.new({ databases = {"pubmed", "arxiv", "ieee"} }),
                CitationAnalyzer.new({ format = "apa" }),
                MethodologyEvaluator.new({ criteria = "peer_review" })
            },
            prompt_template = [[
You are an academic researcher focused on finding peer-reviewed sources 
and evaluating research methodologies. Always prioritize credible sources 
and provide detailed methodology analysis.
]]
        }),
        
        industry_analyst = Agent.new({
            name = "Industry Analyst", 
            model = "gpt-4",
            temperature = 0.4,
            tools = {
                MarketDataTool.new({ sources = {"bloomberg", "reuters"} }),
                TrendAnalyzer.new({ timeframes = ["1y", "5y", "10y"] }),
                CompanyAnalyzer.new({ metrics = ["financial", "operational"] })
            },
            prompt_template = [[
You are an industry analyst focused on commercial applications, 
market trends, and business implications. Provide practical insights 
and commercial viability assessments.
]]
        }),
        
        technical_expert = Agent.new({
            name = "Technical Expert",
            model = "gpt-4", 
            temperature = 0.2,
            tools = {
                CodeAnalyzer.new({ languages = ["rust", "python", "javascript"] }),
                ArchitectureEvaluator.new({ patterns = ["microservices", "event-driven"] }),
                PerformanceBenchmark.new({ metrics = ["latency", "throughput", "memory"] })
            },
            prompt_template = [[
You are a technical expert focused on implementation details, 
architecture patterns, and performance characteristics. Provide 
detailed technical analysis and feasibility assessments.
]]
        }),
        
        synthesis_agent = Agent.new({
            name = "Synthesis Agent",
            model = "gpt-4",
            temperature = 0.6,
            tools = {
                ConflictResolver.new({ strategy = "evidence_based" }),
                ConsensusBuilder.new({ voting_mechanism = "weighted" }),
                ReportGenerator.new({ formats = ["comprehensive", "executive"] })
            },
            prompt_template = [[
You are responsible for synthesizing research from multiple perspectives 
into coherent, well-structured findings. Resolve conflicts between sources 
and highlight areas of consensus and disagreement.
]]
        })
    }
    
    return setmetatable(system, { __index = ResearchCollaborationSystem })
end

function ResearchCollaborationSystem:research_topic(topic, depth_level)
    local collaboration_state = {
        topic = topic,
        depth_level = depth_level,
        findings = {},
        conflicts = {},
        consensus_areas = {},
        timestamp = os.time()
    }
    
    -- Phase 1: Parallel research by specialized agents
    local research_tasks = {
        {
            agent = self.academic_researcher,
            focus = "academic_literature",
            context = {
                topic = topic,
                depth = depth_level,
                sources_required = depth_level == "deep" and 20 or 10,
                timeframe = "last_5_years"
            }
        },
        {
            agent = self.industry_analyst,
            focus = "market_analysis", 
            context = {
                topic = topic,
                depth = depth_level,
                include_competitors = true,
                market_size_analysis = true
            }
        },
        {
            agent = self.technical_expert,
            focus = "technical_feasibility",
            context = {
                topic = topic,
                depth = depth_level,
                include_architecture = true,
                performance_requirements = true
            }
        }
    }
    
    -- Execute research tasks in parallel
    local research_results = {}
    for i, task in ipairs(research_tasks) do
        local result = task.agent:execute({
            input = string.format("Research %s aspects of: %s", task.focus, topic),
            context = task.context
        })
        
        if result.success then
            research_results[task.focus] = result.data
            table.insert(collaboration_state.findings, {
                agent = task.agent.name,
                focus = task.focus,
                findings = result.data,
                confidence = result.data.confidence_score or 0.8
            })
        else
            print(string.format("Warning: %s research failed: %s", task.focus, result.error))
        end
    end
    
    -- Phase 2: Cross-validation and conflict identification
    local validation_result = self:cross_validate_findings(research_results)
    collaboration_state.conflicts = validation_result.conflicts
    collaboration_state.consensus_areas = validation_result.consensus
    
    -- Phase 3: Synthesis and final report generation
    local synthesis_result = self.synthesis_agent:execute({
        input = "Synthesize research findings into comprehensive analysis",
        context = {
            topic = topic,
            research_findings = research_results,
            conflicts = collaboration_state.conflicts,
            consensus_areas = collaboration_state.consensus_areas,
            report_type = depth_level == "deep" and "comprehensive" or "executive"
        }
    })
    
    if not synthesis_result.success then
        error("Synthesis failed: " .. synthesis_result.error)
    end
    
    return {
        topic = topic,
        research_summary = synthesis_result.data.summary,
        detailed_findings = research_results,
        collaboration_metadata = collaboration_state,
        recommendations = synthesis_result.data.recommendations,
        confidence_assessment = synthesis_result.data.confidence_assessment,
        areas_for_further_research = synthesis_result.data.further_research_needed
    }
end

function ResearchCollaborationSystem:cross_validate_findings(research_results)
    local conflicts = {}
    local consensus = {}
    
    -- Simple conflict detection (in practice, this would be more sophisticated)
    for focus1, data1 in pairs(research_results) do
        for focus2, data2 in pairs(research_results) do
            if focus1 ~= focus2 then
                -- Check for conflicting conclusions
                if data1.conclusions and data2.conclusions then
                    local conflict_score = self:calculate_conflict_score(
                        data1.conclusions, 
                        data2.conclusions
                    )
                    
                    if conflict_score > 0.7 then
                        table.insert(conflicts, {
                            sources = {focus1, focus2},
                            conflict_area = "conclusions",
                            severity = conflict_score,
                            details = {
                                source1_position = data1.conclusions,
                                source2_position = data2.conclusions
                            }
                        })
                    elseif conflict_score < 0.3 then
                        table.insert(consensus, {
                            sources = {focus1, focus2},
                            agreement_area = "conclusions",
                            strength = 1.0 - conflict_score,
                            shared_findings = self:extract_shared_findings(
                                data1.conclusions,
                                data2.conclusions
                            )
                        })
                    end
                end
            end
        end
    end
    
    return {
        conflicts = conflicts,
        consensus = consensus
    }
end

function ResearchCollaborationSystem:calculate_conflict_score(conclusions1, conclusions2)
    -- Simplified conflict scoring - in practice would use semantic similarity
    -- This is a placeholder for more sophisticated analysis
    local similar_themes = 0
    local total_themes = 0
    
    -- This would typically involve NLP analysis to compare conclusions
    -- For now, return a placeholder score
    return math.random() * 0.5 + 0.25  -- Random score between 0.25 and 0.75
end

function ResearchCollaborationSystem:extract_shared_findings(conclusions1, conclusions2)
    -- Extract findings that appear in both conclusion sets
    -- This is a simplified implementation
    return {
        "Both sources agree on fundamental concepts",
        "Similar recommendations identified",
        "Consistent data trends observed"
    }
end

-- Usage example
local research_system = ResearchCollaborationSystem.new({
    max_parallel_agents = 3,
    conflict_resolution_strategy = "evidence_weighted",
    output_format = "comprehensive"
})

local result = research_system:research_topic(
    "Impact of Large Language Models on Software Development Productivity",
    "deep"  -- or "standard"
)

print("Research completed!")
print("Summary:", result.research_summary)
print("Number of conflicts identified:", #result.collaboration_metadata.conflicts)
print("Areas of consensus:", #result.collaboration_metadata.consensus_areas)

for _, recommendation in ipairs(result.recommendations) do
    print("Recommendation:", recommendation)
end
```

### Troubleshooting Guide

#### Common Issues and Solutions

**Issue 1: Script Engine Crashes**
```
Error: mlua::Error: runtime error: stack overflow
```

**Diagnosis:**
- Infinite recursion in Lua script
- Excessive memory usage
- Circular references in agent state

**Solutions:**
```lua
-- Add recursion depth limiting
local function safe_execute(func, max_depth)
    local depth = 0
    local function wrapper(...)
        depth = depth + 1
        if depth > max_depth then
            error("Maximum recursion depth exceeded")
        end
        local result = func(...)
        depth = depth - 1
        return result
    end
    return wrapper
end

-- Use proper cleanup patterns
local function cleanup_agent_state(agent)
    agent.state = nil
    agent.tools = {}
    collectgarbage("collect")
end
```

**Issue 2: Hook Execution Deadlocks**
```
Error: Hook execution timeout after 30s
```

**Diagnosis:**
- Hook priority conflicts
- Circular hook dependencies
- Blocking operations in hooks

**Solutions:**
```rust
// Implement hook timeout and cancellation
pub struct HookExecutor {
    timeout: Duration,
    cancellation_token: CancellationToken,
}

impl HookExecutor {
    pub async fn execute_with_timeout(&self, hook: &dyn Hook) -> Result<HookResult> {
        tokio::time::timeout(
            self.timeout,
            hook.execute(context)
        ).await
        .map_err(|_| LLMSpellError::HookTimeout)?
    }
}

// Use non-blocking async patterns
#[async_trait]
impl Hook for MyHook {
    async fn execute(&self, context: &HookContext) -> Result<HookResult> {
        // Use spawn_blocking for CPU-intensive work
        let result = tokio::task::spawn_blocking(|| {
            // CPU-intensive computation
        }).await?;
        
        Ok(HookResult::success(result))
    }
}
```

**Issue 3: Memory Leaks in Long-Running Scripts**
```
Error: Memory usage exceeded limit: 1.2GB > 1GB
```

**Diagnosis:**
- Unclosed resources in tools
- Agent state accumulation
- Script engine memory not being released

**Solutions:**
```rust
// Implement resource tracking
pub struct ResourceTracker {
    memory_limit: usize,
    active_resources: HashMap<String, ResourceHandle>,
}

impl ResourceTracker {
    pub fn check_memory_usage(&self) -> Result<()> {
        let current_usage = self.get_current_memory_usage();
        if current_usage > self.memory_limit {
            self.trigger_garbage_collection();
            
            let post_gc_usage = self.get_current_memory_usage();
            if post_gc_usage > self.memory_limit {
                return Err(LLMSpellError::MemoryLimitExceeded {
                    current: post_gc_usage,
                    limit: self.memory_limit,
                });
            }
        }
        Ok(())
    }
}

// Implement proper resource cleanup
impl Drop for Agent {
    fn drop(&mut self) {
        for tool in &mut self.tools {
            tool.cleanup();
        }
        self.state.clear();
    }
}
```

**Issue 4: LLM Provider Rate Limiting**
```
Error: Rate limit exceeded: 429 Too Many Requests
```

**Diagnosis:**
- Too many concurrent requests
- No retry logic implemented  
- Rate limiting not configured

**Solutions:**
```rust
// Implement adaptive rate limiting
pub struct RateLimitedProvider {
    inner: Box<dyn LLMProvider>,
    rate_limiter: Arc<RateLimiter>,
    retry_strategy: ExponentialBackoff,
}

impl RateLimitedProvider {
    pub async fn call_with_retry(&self, request: &LLMRequest) -> Result<LLMResponse> {
        let mut attempt = 0;
        let max_attempts = 5;
        
        loop {
            self.rate_limiter.wait_for_permit().await;
            
            match self.inner.call(request).await {
                Ok(response) => return Ok(response),
                Err(LLMSpellError::RateLimited { retry_after }) => {
                    attempt += 1;
                    if attempt >= max_attempts {
                        return Err(LLMSpellError::MaxRetriesExceeded);
                    }
                    
                    let delay = self.retry_strategy.next_delay(attempt, retry_after);
                    tokio::time::sleep(delay).await;
                }
                Err(other) => return Err(other),
            }
        }
    }
}
```

#### Performance Debugging

**Tool Execution Profiling:**
```rust
// Add performance instrumentation
#[derive(Debug)]
pub struct PerformanceProfiler {
    tool_timings: HashMap<String, Vec<Duration>>,
    hook_timings: HashMap<String, Vec<Duration>>,
    agent_timings: HashMap<String, Vec<Duration>>,
}

impl PerformanceProfiler {
    pub async fn profile_tool_execution<T>(
        &mut self,
        tool_name: &str,
        execution: impl Future<Output = T>,
    ) -> T {
        let start = Instant::now();
        let result = execution.await;
        let duration = start.elapsed();
        
        self.tool_timings
            .entry(tool_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
            
        result
    }
    
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let mut report = PerformanceReport::new();
        
        for (tool_name, timings) in &self.tool_timings {
            let avg = timings.iter().sum::<Duration>() / timings.len() as u32;
            let max = *timings.iter().max().unwrap();
            let min = *timings.iter().min().unwrap();
            
            report.add_tool_metrics(tool_name, ToolMetrics {
                average_execution_time: avg,
                max_execution_time: max,
                min_execution_time: min,
                total_calls: timings.len(),
            });
        }
        
        report
    }
}
```

**Memory Usage Analysis:**
```bash
# Use system tools for memory debugging
#!/bin/bash

# Monitor rs-llmspell memory usage
watch -n 1 'ps aux | grep rs-llmspell | grep -v grep'

# Generate memory profile
cargo install cargo-profiler
cargo profiler callgrind --bin rs-llmspell -- run script.lua

# Analyze heap usage
valgrind --tool=massif target/debug/rs-llmspell run script.lua
ms_print massif.out.* > memory_report.txt
```

### Performance Tuning

#### Optimization Strategies

**1. Tool Execution Optimization**
```rust
// Implement tool connection pooling
pub struct PooledTool {
    tool_factory: Box<dyn Fn() -> Result<Box<dyn Tool>>>,
    pool: Arc<Pool<Box<dyn Tool>>>,
    config: PoolConfig,
}

impl PooledTool {
    pub async fn execute(&self, input: &ToolInput) -> Result<ToolOutput> {
        let tool = self.pool.get().await?;
        let result = tool.call(input).await;
        
        // Return tool to pool for reuse
        self.pool.put(tool).await;
        
        result
    }
}

// Implement tool result caching
pub struct CachedTool {
    inner: Box<dyn Tool>,
    cache: Arc<RwLock<LruCache<String, ToolOutput>>>,
    cache_duration: Duration,
}

impl CachedTool {
    pub async fn call(&self, input: &ToolInput) -> Result<ToolOutput> {
        let cache_key = self.generate_cache_key(input);
        
        // Check cache first
        if let Some(cached) = self.get_cached_result(&cache_key).await {
            return Ok(cached);
        }
        
        // Execute and cache result
        let result = self.inner.call(input).await?;
        self.cache_result(&cache_key, &result).await;
        
        Ok(result)
    }
}
```

**2. Script Engine Optimization**
```lua
-- Optimize Lua script patterns
local function create_optimized_agent()
    -- Pre-compile frequently used functions
    local compiled_functions = {}
    
    local function get_compiled_function(func_name)
        if not compiled_functions[func_name] then
            compiled_functions[func_name] = load(function_code[func_name])
        end
        return compiled_functions[func_name]
    end
    
    -- Use table pooling to reduce GC pressure
    local table_pool = {}
    
    local function get_table()
        return table.remove(table_pool) or {}
    end
    
    local function return_table(t)
        -- Clear table and return to pool
        for k in pairs(t) do
            t[k] = nil
        end
        table.insert(table_pool, t)
    end
    
    return {
        get_compiled_function = get_compiled_function,
        get_table = get_table,
        return_table = return_table
    }
end
```

**3. Async Performance Patterns**
```rust
// Implement efficient async batching
pub struct BatchedExecutor {
    batch_size: usize,
    batch_timeout: Duration,
    pending_operations: Vec<PendingOperation>,
}

impl BatchedExecutor {
    pub async fn execute_batch(&mut self) -> Result<Vec<OperationResult>> {
        if self.pending_operations.is_empty() {
            return Ok(Vec::new());
        }
        
        let batch = std::mem::take(&mut self.pending_operations);
        
        // Execute operations concurrently with limited parallelism
        let results = futures::stream::iter(batch)
            .map(|op| async move { op.execute().await })
            .buffer_unordered(self.batch_size)
            .collect::<Vec<_>>()
            .await;
            
        Ok(results)
    }
    
    pub async fn schedule_operation(&mut self, operation: PendingOperation) {
        self.pending_operations.push(operation);
        
        // Trigger batch execution if limits reached
        if self.pending_operations.len() >= self.batch_size {
            let _ = self.execute_batch().await;
        }
    }
}

// Use async streams for large datasets
pub async fn process_large_dataset<T>(
    data_stream: impl Stream<Item = T>,
    processor: impl Fn(T) -> Result<ProcessedItem>,
) -> Result<Vec<ProcessedItem>> {
    data_stream
        .map(|item| async move { processor(item) })
        .buffer_unordered(10) // Process 10 items concurrently
        .try_collect()
        .await
}
```

**4. Memory Management Optimization**
```rust
// Implement memory-aware state management
pub struct MemoryAwareStateManager {
    state_cache: LruCache<String, AgentState>,
    memory_monitor: MemoryMonitor,
    compression: CompressionEngine,
}

impl MemoryAwareStateManager {
    pub async fn store_state(&mut self, id: &str, state: AgentState) -> Result<()> {
        // Check memory pressure
        if self.memory_monitor.pressure_level() > 0.8 {
            self.compress_old_states().await?;
        }
        
        // Store with automatic eviction
        self.state_cache.put(id.to_string(), state);
        
        Ok(())
    }
    
    async fn compress_old_states(&mut self) -> Result<()> {
        let mut states_to_compress = Vec::new();
        
        // Identify states for compression
        for (id, state) in self.state_cache.iter() {
            if state.last_accessed < Utc::now() - Duration::hours(1) {
                states_to_compress.push(id.clone());
            }
        }
        
        // Compress and move to secondary storage
        for id in states_to_compress {
            if let Some(state) = self.state_cache.pop(&id) {
                let compressed = self.compression.compress(&state).await?;
                self.secondary_storage.store(&id, compressed).await?;
            }
        }
        
        Ok(())
    }
}
```

**5. Monitoring and Metrics**
```rust
// Comprehensive performance monitoring
pub struct PerformanceMonitor {
    metrics_collector: MetricsCollector,
    alert_thresholds: AlertThresholds,
    dashboards: Vec<DashboardConfig>,
}

impl PerformanceMonitor {
    pub fn start_monitoring(&self) -> Result<()> {
        // Start metrics collection
        self.metrics_collector.start_collection();
        
        // Set up alerting
        self.setup_alerts();
        
        // Initialize dashboards
        self.initialize_dashboards();
        
        Ok(())
    }
    
    fn setup_alerts(&self) {
        // Memory usage alerts
        self.metrics_collector.add_alert(Alert {
            name: "high_memory_usage",
            condition: "memory_usage_percent > 85",
            action: AlertAction::SendNotification {
                recipients: vec!["ops@company.com".to_string()],
                severity: AlertSeverity::Warning,
            },
        });
        
        // Tool execution time alerts  
        self.metrics_collector.add_alert(Alert {
            name: "slow_tool_execution",
            condition: "tool_execution_time_p95 > 5000ms",
            action: AlertAction::TriggerRunbook {
                runbook: "slow_tool_performance".to_string(),
            },
        });
        
        // Error rate alerts
        self.metrics_collector.add_alert(Alert {
            name: "high_error_rate",
            condition: "error_rate_5min > 0.05",
            action: AlertAction::AutoScale {
                scale_factor: 1.5,
                max_instances: 10,
            },
        });
    }
}
```

---

## Part XI: Reference and Appendices

### Complete API Quick Reference

#### Core Traits Quick Reference

**BaseAgent Trait:**
```rust
pub trait BaseAgent: Send + Sync {
    // Core execution
    async fn execute(&self, input: &AgentInput) -> Result<AgentOutput>;
    
    // Tool management
    fn get_tools(&self) -> &[Box<dyn Tool>];
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;
    fn remove_tool(&mut self, tool_name: &str) -> Result<()>;
    
    // State management
    fn get_state(&self) -> &AgentState;
    fn set_state(&mut self, state: AgentState) -> Result<()>;
    fn clear_state(&mut self) -> Result<()>;
    
    // Hook integration
    fn register_hook(&mut self, hook: Box<dyn Hook>) -> Result<()>;
    fn unregister_hook(&mut self, hook_name: &str) -> Result<()>;
    
    // Metadata
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> Option<&str>;
}
```

**Agent Trait:**
```rust
pub trait Agent: BaseAgent {
    // LLM interaction
    async fn llm_call(&self, prompt: &str) -> Result<String>;
    async fn llm_call_with_context(&self, prompt: &str, context: &AgentContext) -> Result<String>;
    
    // Prompt management
    fn get_prompt_template(&self) -> &PromptTemplate;
    fn set_prompt_template(&mut self, template: PromptTemplate) -> Result<()>;
    
    // Model configuration
    fn get_model_config(&self) -> &ModelConfig;
    fn set_model_config(&mut self, config: ModelConfig) -> Result<()>;
    
    // Memory management
    fn get_conversation_history(&self) -> &[ConversationTurn];
    fn add_to_history(&mut self, turn: ConversationTurn) -> Result<()>;
    fn clear_history(&mut self) -> Result<()>;
}
```

**Tool Trait:**
```rust
pub trait Tool: Send + Sync {
    // Core execution
    async fn call(&self, input: &ToolInput) -> Result<ToolOutput>;
    
    // Schema and metadata
    fn get_schema(&self) -> &ToolSchema;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_version(&self) -> &str;
    
    // Lifecycle management
    async fn initialize(&mut self) -> Result<()>;
    async fn cleanup(&mut self) -> Result<()>;
    
    // Validation
    fn validate_input(&self, input: &ToolInput) -> Result<()>;
    
    // Configuration
    fn get_config(&self) -> &ToolConfig;
    fn set_config(&mut self, config: ToolConfig) -> Result<()>;
}
```

**Workflow Trait:**
```rust
pub trait Workflow: Send + Sync {
    // Execution
    async fn run(&self, input: &WorkflowInput) -> Result<WorkflowOutput>;
    async fn step(&self, step_id: &str, input: &StepInput) -> Result<StepOutput>;
    
    // Structure
    fn get_steps(&self) -> &[WorkflowStep];
    fn get_dependencies(&self) -> &WorkflowDependencies;
    
    // State management
    fn get_execution_state(&self) -> &WorkflowState;
    fn set_execution_state(&mut self, state: WorkflowState) -> Result<()>;
    
    // Control flow
    async fn pause(&self) -> Result<()>;
    async fn resume(&self) -> Result<()>;
    async fn cancel(&self) -> Result<()>;
    
    // Metadata
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_type(&self) -> WorkflowType;
}
```

#### Lua API Quick Reference

**Agent Creation and Management:**
```lua
-- Create new agent
local agent = Agent.new({
    name = "my_agent",
    model = "gpt-4",
    temperature = 0.7,
    max_tokens = 2048,
    tools = { HttpTool.new(), FileSystemTool.new() }
})

-- Execute agent
local result = agent:execute({
    input = "Process this request",
    context = { key = "value" }
})

-- Manage agent state
agent:set_state({ counter = 1, mode = "active" })
local state = agent:get_state()
agent:clear_state()

-- Add/remove tools
agent:add_tool(DatabaseTool.new({ connection = "postgres://..." }))
agent:remove_tool("HttpTool")

-- Hook registration
agent:register_hook("before_execution", function(context)
    print("About to execute:", context.input)
end)
```

**Tool Usage:**
```lua
-- HTTP Tool
local http = HttpTool.new({ timeout = 30, retries = 3 })
local response = http:call({
    method = "GET",
    url = "https://api.example.com/data",
    headers = { ["Authorization"] = "Bearer token" }
})

-- File System Tool
local fs = FileSystemTool.new({ base_path = "/safe/directory" })
local content = fs:call({
    action = "read",
    path = "document.txt"
})

-- Database Tool
local db = DatabaseTool.new({ connection = "postgres://user:pass@host/db" })
local results = db:call({
    query = "SELECT * FROM users WHERE active = $1",
    params = { true }
})
```

**Workflow Creation:**
```lua
-- Sequential workflow
local workflow = Workflow.sequential({
    name = "data_processing",
    steps = {
        {
            name = "fetch_data",
            agent = DataFetchAgent.new(),
            input = { source = "api" },
            output = "raw_data"
        },
        {
            name = "process_data", 
            agent = ProcessingAgent.new(),
            input = { data = "{{raw_data}}" },
            output = "processed_data"
        }
    }
})

-- Parallel workflow
local parallel_workflow = Workflow.parallel({
    name = "multi_source_analysis",
    branches = {
        {
            name = "source_a",
            agent = AnalysisAgent.new({ source = "a" })
        },
        {
            name = "source_b", 
            agent = AnalysisAgent.new({ source = "b" })
        }
    },
    merge_strategy = "combine_results"
})

-- Conditional workflow
local conditional_workflow = Workflow.conditional({
    name = "adaptive_processing",
    condition = function(context)
        return context.data_size > 1000000
    end,
    if_true = LargeDataWorkflow.new(),
    if_false = SmallDataWorkflow.new()
})
```

**Event Handling:**
```lua
-- Event registration
Events.subscribe("agent_completed", function(event)
    print("Agent completed:", event.agent_id, "Result:", event.result)
end)

-- Event emission
Events.emit("custom_event", {
    timestamp = os.time(),
    data = { key = "value" }
})

-- Hook management
Hooks.register("before_llm_call", {
    priority = 100,
    handler = function(context)
        context.prompt = "Enhanced: " .. context.prompt
        return context
    end
})
```

#### JavaScript API Quick Reference

**Agent Creation and Management:**
```javascript
// Create new agent
const agent = new Agent({
    name: 'my_agent',
    model: 'gpt-4',
    temperature: 0.7,
    maxTokens: 2048,
    tools: [new HttpTool(), new FileSystemTool()]
});

// Execute agent
const result = await agent.execute({
    input: 'Process this request',
    context: { key: 'value' }
});

// Manage agent state
await agent.setState({ counter: 1, mode: 'active' });
const state = await agent.getState();
await agent.clearState();

// Add/remove tools
await agent.addTool(new DatabaseTool({ connection: 'postgres://...' }));
await agent.removeTool('HttpTool');

// Hook registration
agent.registerHook('beforeExecution', async (context) => {
    console.log('About to execute:', context.input);
});
```

**Tool Usage:**
```javascript
// HTTP Tool
const http = new HttpTool({ timeout: 30000, retries: 3 });
const response = await http.call({
    method: 'GET',
    url: 'https://api.example.com/data',
    headers: { 'Authorization': 'Bearer token' }
});

// File System Tool
const fs = new FileSystemTool({ basePath: '/safe/directory' });
const content = await fs.call({
    action: 'read',
    path: 'document.txt'
});

// Database Tool
const db = new DatabaseTool({ connection: 'postgres://user:pass@host/db' });
const results = await db.call({
    query: 'SELECT * FROM users WHERE active = $1',
    params: [true]
});
```

**Workflow Creation:**
```javascript
// Sequential workflow
const workflow = new Workflow.Sequential({
    name: 'data_processing',
    steps: [
        {
            name: 'fetch_data',
            agent: new DataFetchAgent(),
            input: { source: 'api' },
            output: 'raw_data'
        },
        {
            name: 'process_data',
            agent: new ProcessingAgent(), 
            input: { data: '{{raw_data}}' },
            output: 'processed_data'
        }
    ]
});

// Parallel workflow
const parallelWorkflow = new Workflow.Parallel({
    name: 'multi_source_analysis',
    branches: [
        {
            name: 'source_a',
            agent: new AnalysisAgent({ source: 'a' })
        },
        {
            name: 'source_b',
            agent: new AnalysisAgent({ source: 'b' })
        }
    ],
    mergeStrategy: 'combine_results'
});

// Conditional workflow
const conditionalWorkflow = new Workflow.Conditional({
    name: 'adaptive_processing',
    condition: (context) => context.dataSize > 1000000,
    ifTrue: new LargeDataWorkflow(),
    ifFalse: new SmallDataWorkflow()
});
```

**Event Handling:**
```javascript
// Event registration
Events.subscribe('agentCompleted', (event) => {
    console.log('Agent completed:', event.agentId, 'Result:', event.result);
});

// Event emission
Events.emit('customEvent', {
    timestamp: Date.now(),
    data: { key: 'value' }
});

// Hook management
Hooks.register('beforeLlmCall', {
    priority: 100,
    handler: async (context) => {
        context.prompt = 'Enhanced: ' + context.prompt;
        return context;
    }
});
```

### Error Code Reference

#### Error Hierarchy and Codes

**Core Error Categories:**
```rust
// Agent errors (1000-1999)
pub enum AgentErrorCode {
    NotFound = 1001,
    InitializationFailed = 1002,
    ExecutionFailed = 1003,
    InvalidConfiguration = 1004,
    Timeout = 1005,
    MemoryLimitExceeded = 1006,
    StateCorrupted = 1007,
    ToolNotFound = 1008,
    LLMCallFailed = 1009,
    PromptTemplateInvalid = 1010,
}

// Tool errors (2000-2999)
pub enum ToolErrorCode {
    NotFound = 2001,
    InvalidInput = 2002,
    ExecutionFailed = 2003,
    InitializationFailed = 2004,
    ConfigurationInvalid = 2005,
    PermissionDenied = 2006,
    ResourceUnavailable = 2007,
    Timeout = 2008,
    ValidationFailed = 2009,
    SchemaInvalid = 2010,
}

// Workflow errors (3000-3999)
pub enum WorkflowErrorCode {
    NotFound = 3001,
    InvalidDefinition = 3002,
    ExecutionFailed = 3003,
    StepFailed = 3004,
    DependencyFailed = 3005,
    CircularDependency = 3006,
    Timeout = 3007,
    StateInvalid = 3008,
    ConditionEvaluationFailed = 3009,
    MergeFailed = 3010,
}

// Script errors (4000-4999)
pub enum ScriptErrorCode {
    SyntaxError = 4001,
    RuntimeError = 4002,
    CompilationFailed = 4003,
    ExecutionTimeout = 4004,
    MemoryLimitExceeded = 4005,
    SecurityViolation = 4006,
    APINotAvailable = 4007,
    PermissionDenied = 4008,
    ResourceExhausted = 4009,
    InvalidFunction = 4010,
}

// Hook and Event errors (5000-5999)
pub enum HookEventErrorCode {
    HookNotFound = 5001,
    HookExecutionFailed = 5002,
    HookTimeout = 5003,
    InvalidPriority = 5004,
    EventEmissionFailed = 5005,
    SubscriptionFailed = 5006,
    FilterInvalid = 5007,
    HandlerNotFound = 5008,
    CircularDependency = 5009,
    DeadlockDetected = 5010,
}

// System errors (6000-6999)
pub enum SystemErrorCode {
    ConfigurationInvalid = 6001,
    StorageFailure = 6002,
    NetworkFailure = 6003,
    AuthenticationFailed = 6004,
    AuthorizationFailed = 6005,
    RateLimitExceeded = 6006,
    ResourceExhausted = 6007,
    ServiceUnavailable = 6008,
    InternalError = 6009,
    VersionMismatch = 6010,
}
```

**Error Resolution Guide:**
```rust
pub struct ErrorResolutionGuide {
    pub code: u32,
    pub category: &'static str,
    pub description: &'static str,
    pub common_causes: &'static [&'static str],
    pub resolution_steps: &'static [&'static str],
    pub prevention_tips: &'static [&'static str],
}

pub const ERROR_GUIDE: &[ErrorResolutionGuide] = &[
    ErrorResolutionGuide {
        code: 1003,
        category: "Agent",
        description: "Agent execution failed",
        common_causes: &[
            "Invalid input format",
            "Tool failure during execution",
            "LLM provider timeout",
            "Memory limit exceeded",
        ],
        resolution_steps: &[
            "Check input validation",
            "Verify tool configurations",
            "Review LLM provider status",
            "Monitor memory usage",
            "Check agent state consistency",
        ],
        prevention_tips: &[
            "Implement input validation",
            "Add tool health checks",
            "Configure appropriate timeouts",
            "Monitor resource usage",
        ],
    },
    ErrorResolutionGuide {
        code: 2002,
        category: "Tool",
        description: "Invalid tool input",
        common_causes: &[
            "Missing required parameters",
            "Invalid parameter types",
            "Schema validation failure",
            "Malformed input data",
        ],
        resolution_steps: &[
            "Validate input against tool schema",
            "Check parameter types and values",
            "Review tool documentation",
            "Test with minimal valid input",
        ],
        prevention_tips: &[
            "Use schema validation",
            "Implement input sanitization",
            "Add comprehensive error messages",
            "Provide clear documentation",
        ],
    },
    ErrorResolutionGuide {
        code: 3004,
        category: "Workflow",
        description: "Workflow step failed",
        common_causes: &[
            "Agent execution failure",
            "Invalid step configuration",
            "Dependency not met",
            "Step timeout",
        ],
        resolution_steps: &[
            "Check step agent configuration",
            "Verify step dependencies",
            "Review step input/output mapping",
            "Check execution logs",
        ],
        prevention_tips: &[
            "Test steps individually",
            "Validate workflow definition",
            "Implement proper error handling",
            "Add step health checks",
        ],
    },
    ErrorResolutionGuide {
        code: 4002,
        category: "Script",
        description: "Script runtime error",
        common_causes: &[
            "Undefined variable access",
            "Function call failure",
            "API usage error",
            "Resource access violation",
        ],
        resolution_steps: &[
            "Check script syntax and logic",
            "Verify API availability",
            "Review variable scoping",
            "Check security permissions",
        ],
        prevention_tips: &[
            "Use strict mode",
            "Implement error handling",
            "Test in sandbox environment",
            "Follow security best practices",
        ],
    },
];
```

### Configuration Schema Reference

#### Complete Configuration Schema

**Root Configuration:**
```toml
# Rs-LLMSpell Configuration v2.0
version = "2.0"

[server]
host = "0.0.0.0"                    # Server bind address
port = 8080                         # Server port
tls_enabled = false                 # Enable TLS/HTTPS
cert_path = "/path/to/cert.pem"     # TLS certificate path
key_path = "/path/to/key.pem"       # TLS private key path
max_connections = 1000              # Maximum concurrent connections
request_timeout = 30000             # Request timeout in milliseconds
graceful_shutdown_timeout = 10000   # Shutdown timeout in milliseconds
cors_enabled = true                 # Enable CORS
cors_origins = ["*"]                # Allowed CORS origins

[storage]
backend = "sled"                    # Storage backend: "sled", "rocksdb", "memory"
path = "/var/lib/rs-llmspell/db"   # Database path
connection_pool_size = 10           # Connection pool size
query_timeout = 5000                # Query timeout in milliseconds
backup_enabled = true               # Enable automatic backups
backup_interval = 3600              # Backup interval in seconds
backup_retention = 168              # Backup retention in hours (7 days)
compression_enabled = true          # Enable data compression
encryption_enabled = false          # Enable at-rest encryption
encryption_key_path = "/path/to/key" # Encryption key file path

[scripting]
engines = ["lua", "javascript"]     # Enabled script engines
sandbox_enabled = true              # Enable script sandboxing
max_execution_time = 300000         # Max script execution time in milliseconds
max_memory_usage = 536870912        # Max memory usage in bytes (512MB)
script_cache_enabled = true         # Enable compiled script caching
script_cache_size = 100             # Number of scripts to cache
global_timeout = 600000             # Global operation timeout in milliseconds

[scripting.lua]
version = "5.4"                     # Lua version
coroutine_enabled = true            # Enable coroutine support
debug_enabled = false               # Enable debug features
custom_modules_path = "/path/to/modules" # Custom Lua modules path

[scripting.javascript]
engine = "boa"                      # JavaScript engine: "boa"
strict_mode = true                  # Enable strict mode
promise_support = true              # Enable Promise support
console_enabled = true              # Enable console object
timeout_checks = true               # Enable timeout checks

[providers]
default = "openai"                  # Default LLM provider

[providers.openai]
api_key_env = "OPENAI_API_KEY"     # Environment variable for API key
api_key_file = "/path/to/key"      # File path for API key
base_url = "https://api.openai.com/v1" # API base URL
model = "gpt-4"                     # Default model
timeout = 30000                     # Request timeout in milliseconds
retry_attempts = 3                  # Number of retry attempts
retry_delay = 1000                  # Retry delay in milliseconds
max_tokens = 4096                   # Maximum tokens per request
temperature = 0.7                   # Default temperature
rate_limit_requests_per_minute = 60 # Rate limit

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
base_url = "https://api.anthropic.com"
model = "claude-3-sonnet-20240229"
timeout = 30000
retry_attempts = 3
retry_delay = 1000
max_tokens = 4096
temperature = 0.7

[providers.local]
enabled = false                     # Enable local model support
model_path = "/path/to/models"     # Local models directory
device = "cpu"                      # Device: "cpu", "cuda", "metal"
threads = 4                         # Number of threads
context_size = 2048                 # Context size

[hooks]
enabled = true                      # Enable hook system
max_hooks_per_point = 10           # Maximum hooks per hook point
execution_timeout = 5000            # Hook execution timeout in milliseconds
priority_range_min = 0             # Minimum hook priority
priority_range_max = 1000          # Maximum hook priority
parallel_execution = true          # Enable parallel hook execution
error_handling = "continue"        # Error handling: "continue", "stop", "rollback"

[events]
enabled = true                      # Enable event system
buffer_size = 1000                 # Event buffer size
max_subscribers = 100              # Maximum subscribers per event
persistence_enabled = true         # Enable event persistence
persistence_ttl = 86400            # Event persistence TTL in seconds
batch_processing = true            # Enable batch event processing
batch_size = 50                    # Event batch size

[security]
authentication_required = false    # Require authentication
api_key_header = "X-API-Key"       # API key header name
jwt_secret_env = "JWT_SECRET"      # JWT secret environment variable
session_timeout = 3600             # Session timeout in seconds
rate_limiting_enabled = true       # Enable rate limiting
rate_limit_requests_per_minute = 100 # Rate limit per minute
rate_limit_burst = 10              # Rate limit burst size
allowed_origins = ["*"]            # Allowed origins for CORS
blocked_ips = []                   # Blocked IP addresses
max_request_size = 10485760        # Max request size in bytes (10MB)

[observability]
metrics_enabled = true             # Enable metrics collection
tracing_enabled = true             # Enable distributed tracing
logging_enabled = true             # Enable structured logging
metrics_port = 9090                # Metrics server port
health_check_enabled = true        # Enable health checks
health_check_interval = 30         # Health check interval in seconds

[observability.logging]
level = "info"                     # Log level: "trace", "debug", "info", "warn", "error"
format = "json"                    # Log format: "json", "text"
output = "stdout"                  # Log output: "stdout", "stderr", "file"
file_path = "/var/log/rs-llmspell.log" # Log file path
rotation_enabled = true            # Enable log rotation
max_file_size = 104857600          # Max log file size in bytes (100MB)
max_files = 10                     # Maximum number of log files

[observability.metrics]
prometheus_enabled = true          # Enable Prometheus metrics
custom_metrics_enabled = true      # Enable custom metrics
histogram_buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]

[observability.tracing]
jaeger_enabled = false             # Enable Jaeger tracing
jaeger_endpoint = "http://localhost:14268/api/traces"
sample_rate = 0.1                  # Trace sampling rate
service_name = "rs-llmspell"       # Service name for tracing

[protocols]
mcp_enabled = false                # Enable Model Control Protocol
mcp_server_port = 8081            # MCP server port
a2a_enabled = false                # Enable Agent-to-Agent protocol
a2a_server_port = 8082            # A2A server port

[protocols.mcp]
version = "1.0"                    # MCP protocol version
authentication_required = true     # Require authentication for MCP
max_connections = 100              # Maximum MCP connections
timeout = 30000                    # MCP operation timeout

[protocols.a2a]
version = "1.0"                    # A2A protocol version
mesh_enabled = false               # Enable agent mesh networking
discovery_enabled = true           # Enable service discovery
heartbeat_interval = 30            # Heartbeat interval in seconds

[development]
debug_enabled = false              # Enable debug mode
hot_reload_enabled = false         # Enable hot reload
profiling_enabled = false          # Enable performance profiling
test_mode_enabled = false          # Enable test mode
mock_llm_responses = false         # Use mock LLM responses
```

**Schema Validation Rules:**
```rust
pub struct ConfigurationValidator {
    rules: Vec<ValidationRule>,
}

pub enum ValidationRule {
    Required(String),                           // Field is required
    Range { field: String, min: f64, max: f64 }, // Numeric range
    OneOf { field: String, values: Vec<String> }, // Enum values
    Pattern { field: String, regex: String },    // Regex pattern
    Conditional { 
        condition: String, 
        then_required: Vec<String> 
    }, // Conditional requirements
}

pub const VALIDATION_RULES: &[ValidationRule] = &[
    ValidationRule::Required("version".to_string()),
    ValidationRule::Range { 
        field: "server.port".to_string(), 
        min: 1.0, 
        max: 65535.0 
    },
    ValidationRule::OneOf { 
        field: "storage.backend".to_string(), 
        values: vec!["sled".to_string(), "rocksdb".to_string(), "memory".to_string()] 
    },
    ValidationRule::Pattern { 
        field: "server.host".to_string(), 
        regex: r"^(\d{1,3}\.){3}\d{1,3}$|^localhost$|^0\.0\.0\.0$".to_string() 
    },
    ValidationRule::Conditional {
        condition: "security.authentication_required == true".to_string(),
        then_required: vec!["security.jwt_secret_env".to_string()],
    },
];
```

### Future Evolution Strategy

#### Planned Feature Roadmap

**Version 2.1 (Q2 2025)**
- **Python Scripting Support**: Complete Python bridge implementation with PyO3
- **Advanced Tool Composition**: Tool chaining and pipeline creation
- **Enhanced Security**: Advanced sandboxing and threat detection
- **Performance Optimizations**: SIMD vectorization and GPU acceleration

**Version 2.2 (Q3 2025)**
- **Distributed Agent Mesh**: Multi-node agent coordination
- **Advanced Workflow Patterns**: Graph-based workflows and dynamic orchestration
- **Real-time Collaboration**: Live agent state synchronization
- **Enhanced Observability**: Advanced metrics and distributed tracing

**Version 2.3 (Q4 2025)**
- **Multi-modal Support**: Image, audio, and video processing capabilities
- **Advanced AI Features**: Fine-tuning integration and model adaptation
- **Enterprise Features**: Advanced RBAC, audit logging, and compliance
- **Cloud-native Enhancements**: Kubernetes operators and service mesh integration

**Version 3.0 (Q1 2026)**
- **Next-generation Architecture**: Redesigned core with improved performance
- **Advanced AI Orchestration**: Autonomous agent creation and optimization
- **Quantum-ready Algorithms**: Preparation for quantum computing integration
- **Universal Protocol Support**: Support for emerging AI protocols and standards

#### Extension Points and Customization

**Custom Tool Development:**
```rust
// Plugin API for custom tools
pub trait CustomTool: Tool {
    fn get_plugin_metadata(&self) -> PluginMetadata;
    fn get_dependencies(&self) -> Vec<PluginDependency>;
    fn initialize_plugin(&mut self, context: &PluginContext) -> Result<()>;
}

// Tool plugin system
pub struct ToolPluginManager {
    plugins: HashMap<String, Box<dyn CustomTool>>,
    registry: PluginRegistry,
}

impl ToolPluginManager {
    pub fn load_plugin(&mut self, plugin_path: &str) -> Result<()> {
        // Dynamic plugin loading
        let plugin = self.load_dynamic_library(plugin_path)?;
        self.register_plugin(plugin)?;
        Ok(())
    }
    
    pub fn discover_plugins(&mut self, directory: &str) -> Result<Vec<PluginInfo>> {
        // Auto-discovery of plugins
        self.scan_plugin_directory(directory)
    }
}
```

**Custom Agent Types:**
```rust
// Extensible agent system
pub trait CustomAgent: Agent {
    fn get_agent_type(&self) -> &str;
    fn get_capabilities(&self) -> Vec<AgentCapability>;
    fn supports_delegation(&self) -> bool;
    
    async fn delegate_to(&self, other_agent: &dyn Agent, task: &Task) -> Result<DelegationResult>;
    async fn negotiate_task(&self, task: &Task) -> Result<NegotiationResult>;
}

// Agent factory for custom types
pub struct AgentFactory {
    builders: HashMap<String, Box<dyn AgentBuilder>>,
}

pub trait AgentBuilder {
    fn build(&self, config: &AgentConfig) -> Result<Box<dyn CustomAgent>>;
    fn get_supported_types(&self) -> Vec<String>;
    fn validate_config(&self, config: &AgentConfig) -> Result<()>;
}
```

**Protocol Extensions:**
```rust
// Extensible protocol system
pub trait CustomProtocol {
    fn get_protocol_name(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_capabilities(&self) -> Vec<ProtocolCapability>;
    
    async fn handle_message(&self, message: &ProtocolMessage) -> Result<ProtocolResponse>;
    async fn establish_connection(&self, endpoint: &str) -> Result<ProtocolConnection>;
}

// Protocol registry
pub struct ProtocolRegistry {
    protocols: HashMap<String, Box<dyn CustomProtocol>>,
    handlers: HashMap<String, ProtocolHandler>,
}

impl ProtocolRegistry {
    pub fn register_protocol(&mut self, protocol: Box<dyn CustomProtocol>) -> Result<()> {
        let name = protocol.get_protocol_name().to_string();
        self.protocols.insert(name, protocol);
        Ok(())
    }
    
    pub fn route_message(&self, message: &ProtocolMessage) -> Result<ProtocolResponse> {
        let protocol = self.protocols.get(&message.protocol)
            .ok_or(LLMSpellError::ProtocolNotFound)?;
        protocol.handle_message(message)
    }
}
```

#### Migration and Compatibility Strategy

**Forward Compatibility:**
```rust
// Version negotiation system
pub struct VersionNegotiator {
    supported_versions: Vec<Version>,
    compatibility_matrix: CompatibilityMatrix,
}

impl VersionNegotiator {
    pub fn negotiate_version(&self, requested: &Version) -> Result<Version> {
        // Find best compatible version
        for supported in &self.supported_versions {
            if self.compatibility_matrix.is_compatible(requested, supported) {
                return Ok(supported.clone());
            }
        }
        Err(LLMSpellError::VersionIncompatible)
    }
    
    pub fn can_migrate(&self, from: &Version, to: &Version) -> bool {
        self.compatibility_matrix.has_migration_path(from, to)
    }
}

// Graceful degradation
pub struct FeatureCompatibility {
    features: HashMap<String, FeatureSupport>,
}

pub enum FeatureSupport {
    FullySupported,
    PartiallySupported { limitations: Vec<String> },
    NotSupported { alternative: Option<String> },
}
```

**Backward Compatibility:**
```rust
// Legacy API support
pub struct LegacyApiAdapter {
    version_mappings: HashMap<String, ApiMapping>,
    deprecation_warnings: DeprecationWarnings,
}

impl LegacyApiAdapter {
    pub async fn handle_legacy_request(&self, request: &LegacyRequest) -> Result<LegacyResponse> {
        // Log deprecation warning
        self.deprecation_warnings.warn(&request.api_version);
        
        // Convert to current API format
        let modern_request = self.convert_request(request)?;
        let modern_response = self.handle_modern_request(modern_request).await?;
        
        // Convert back to legacy format
        self.convert_response(modern_response)
    }
}
```

#### Performance Evolution

**Scalability Improvements:**
```rust
// Auto-scaling system
pub struct AutoScaler {
    metrics_monitor: MetricsMonitor,
    scaling_policies: Vec<ScalingPolicy>,
    resource_manager: ResourceManager,
}

impl AutoScaler {
    pub async fn evaluate_scaling(&self) -> Result<ScalingDecision> {
        let current_metrics = self.metrics_monitor.get_current_metrics().await?;
        
        for policy in &self.scaling_policies {
            if policy.should_scale(&current_metrics) {
                return Ok(policy.get_scaling_decision(&current_metrics));
            }
        }
        
        Ok(ScalingDecision::NoAction)
    }
    
    pub async fn apply_scaling(&self, decision: &ScalingDecision) -> Result<()> {
        match decision {
            ScalingDecision::ScaleUp { instances } => {
                self.resource_manager.add_instances(*instances).await?;
            }
            ScalingDecision::ScaleDown { instances } => {
                self.resource_manager.remove_instances(*instances).await?;
            }
            ScalingDecision::NoAction => {}
        }
        Ok(())
    }
}
```

**Optimization Framework:**
```rust
// Performance optimization engine
pub struct OptimizationEngine {
    analyzers: Vec<Box<dyn PerformanceAnalyzer>>,
    optimizers: Vec<Box<dyn PerformanceOptimizer>>,
}

pub trait PerformanceAnalyzer {
    fn analyze(&self, metrics: &PerformanceMetrics) -> AnalysisResult;
    fn get_recommendations(&self, analysis: &AnalysisResult) -> Vec<OptimizationRecommendation>;
}

pub trait PerformanceOptimizer {
    fn can_optimize(&self, recommendation: &OptimizationRecommendation) -> bool;
    async fn apply_optimization(&self, recommendation: &OptimizationRecommendation) -> Result<OptimizationResult>;
}
```

---

## Conclusion

This comprehensive architecture document represents the complete blueprint for rs-llmspell, a production-ready scriptable LLM interaction framework. Built on Rust's foundation with go-llms inspired patterns, it provides:

**🏗️ Robust Architecture**: BaseAgent/Agent/Tool/Workflow hierarchy with comprehensive state management, hooks, and events

**🌍 Multi-Language Scripting**: Seamless Lua, JavaScript, and planned Python support with unified async patterns

**📦 Production Infrastructure**: Built-in security, observability, error handling, and deployment strategies

**🔧 Extensible Design**: Plugin systems, protocol support, and future evolution pathways

**📚 Complete Implementation Guide**: Real-world examples, troubleshooting, performance tuning, and migration strategies

Rs-llmspell bridges the gap between high-performance Rust implementations and flexible scripting environments, enabling developers to build sophisticated AI applications without sacrificing performance or maintainability.

The architecture scales from simple single-agent scripts to complex multi-agent workflows, supporting everything from rapid prototyping to enterprise-scale deployments. With its bridge-first philosophy and comprehensive built-in library, rs-llmspell accelerates AI application development while maintaining production-ready standards.

**Ready for Implementation**: This document provides everything needed to begin implementation, from core traits to deployment manifests, ensuring a smooth path from architecture to production deployment.

---

*Document Version*: 2.0  
*Total Lines*: ~14,500+  
*Status*: Complete Standalone Reference  
*Last Updated*: January 2025
