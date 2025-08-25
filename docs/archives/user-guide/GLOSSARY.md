# LLMSpell Terminology Glossary

**Version**: 1.0  
**Status**: Reference Document  
**Last Updated**: August 2025

> **📚 Terminology Reference**: Standardized terms and definitions used throughout LLMSpell documentation.

---

## Core Terms

### Agent
An AI entity that processes prompts and generates responses. Agents can be:
- **LLM Agents**: Backed by language models (OpenAI, Anthropic, etc.)
- **Tool Agents**: Execute tools based on prompts
- **Workflow Agents**: Orchestrate workflows
- **Hybrid Agents**: Combine multiple capabilities

**Preferred over**: Assistant, Bot, AI

### Tool
A pre-built function that performs a specific operation. Tools are organized into categories and can be invoked directly or through agents.

**Preferred over**: Function, Utility, Helper

### Workflow
An orchestrated sequence of steps that can include agents, tools, and control flow logic.

**Preferred over**: Pipeline, Process, Flow

### Session
A stateful interaction context that persists across multiple operations. Sessions track conversation history, state, and artifacts.

**Preferred over**: Context, Conversation, Thread

### State
Persistent data that survives across script executions. Can be memory-only or persisted to storage.

**Preferred over**: Data, Storage, Memory

### Hook
An extension point that allows custom code to run at specific lifecycle events.

**Preferred over**: Callback, Interceptor, Middleware

### Event
A system occurrence that can be tracked, correlated, and exported.

**Preferred over**: Log, Telemetry, Metric

### Provider
An LLM service provider (OpenAI, Anthropic, Azure, etc.) that supplies models.

**Preferred over**: Service, Backend, Vendor

### Model
A specific LLM variant from a provider (e.g., "gpt-4", "claude-3").

**Preferred over**: Engine, Version, Variant

### Bridge
The integration layer between Rust and scripting languages.

**Preferred over**: Binding, Interface, Wrapper

---

## Architecture Terms

### Registry
A central collection that manages instances of a particular type (ToolRegistry, AgentRegistry).

**Preferred over**: Manager, Repository, Store

### Builder
A pattern for constructing complex objects with a fluent API.

**Preferred over**: Factory, Constructor, Creator

### Executor
A component that runs operations (workflows, steps, hooks).

**Preferred over**: Runner, Processor, Handler

### Manager
A service component that handles lifecycle and operations (*Manager suffix for services).

**Preferred over**: Service, Controller, Coordinator

---

## Configuration Terms

### Config/Configuration
Settings that control behavior, typically in TOML format.

**Preferred over**: Settings, Options, Parameters

### Template
A reusable configuration pattern for agents or workflows.

**Preferred over**: Preset, Blueprint, Pattern

### Profile
A named configuration set for different environments or use cases.

**Preferred over**: Environment, Mode, Variant

---

## Operation Terms

### Execute
To run an agent, tool, or workflow.

**Preferred over**: Run, Invoke, Call

### Invoke
Specifically for tool operations.

**Preferred over**: Call, Run, Execute

### Build
To construct an object using a builder pattern.

**Preferred over**: Create, Make, Construct

### Load
To retrieve from persistent storage.

**Preferred over**: Fetch, Get, Retrieve

### Save
To persist to storage.

**Preferred over**: Store, Persist, Write

---

## User Interface Terms

### Script
A Lua file containing LLMSpell code.

**Preferred over**: Program, Code, File

### Prompt
User input to an agent.

**Preferred over**: Input, Query, Question

### Response
Agent output.

**Preferred over**: Output, Reply, Answer

### Result
Operation outcome, typically includes success/failure status.

**Preferred over**: Output, Return, Response

---

## State Management Terms

### Persistence
Long-term storage of state.

**Preferred over**: Storage, Database, Saving

### Artifact
A saved piece of content (code, document, data).

**Preferred over**: File, Document, Object

### Migration
Moving state between schema versions.

**Preferred over**: Upgrade, Transform, Convert

---

## Error Handling Terms

### Error
A recoverable failure condition.

**Preferred over**: Exception, Fault, Failure

### Panic
An unrecoverable failure (Rust-specific).

**Preferred over**: Crash, Fatal, Abort

### Retry
Attempting an operation again after failure.

**Preferred over**: Repeat, Reattempt, Try again

---

## Performance Terms

### Latency
Time delay in operations.

**Preferred over**: Delay, Lag, Response time

### Throughput
Operations per unit time.

**Preferred over**: Rate, Speed, Bandwidth

### Overhead
Additional resource usage from infrastructure.

**Preferred over**: Cost, Tax, Burden

---

## Documentation Terms

### Guide
A tutorial or how-to document.

**Preferred over**: Tutorial, Manual, Handbook

### Reference
API or technical documentation.

**Preferred over**: API Docs, Specification, Manual

### Example
Code demonstrating usage.

**Preferred over**: Sample, Demo, Snippet

---

## Usage Guidelines

### Consistency Rules
1. Always use the preferred term in new documentation
2. Update old documentation when making other changes
3. Use terms consistently within a document
4. Define terms on first use in user-facing docs

### Capitalization
- **Proper nouns**: LLMSpell, OpenAI, Anthropic
- **Features**: Agent, Tool, Workflow (when referring to the module)
- **Generic uses**: agent, tool, workflow (when referring to instances)

### Abbreviations
- LLM: Large Language Model
- API: Application Programming Interface
- CLI: Command Line Interface
- TOML: Tom's Obvious, Minimal Language

---

## See Also

- [Documentation Template](TEMPLATE.md) - Standard documentation format
- [Style Guide](../developer-guide/style-guide.md) - Code style guidelines
- [Contributing](../../CONTRIBUTING.md) - Contribution guidelines