# LLMSpell Examples Reference & Standards

✅ **CURRENT**: Phase 8 - Comprehensive guide to 60+ production examples
**Version**: 0.8.0 | **Examples**: 60+ | **Categories**: 9

**Quick Navigation**: [Standards](#standards) | [Catalog](#example-catalog) | [Learning Paths](#learning-paths) | [Patterns](#pattern-library)

---

## Overview

This document serves as both:
1. **Standards Guide**: How to write examples
2. **Reference Catalog**: Index of all 60+ examples
3. **Learning Paths**: Recommended progression
4. **Pattern Library**: Common patterns from cookbook

The examples in `examples/*` are **production-quality documentation** that demonstrate real usage patterns.

## Standards

### 📋 Metadata Header

Every example MUST start with a metadata header:

### Lua Examples
```lua
-- Example: [Name of Example]
-- Purpose: [What this example demonstrates]
-- Audience: [Script Users|Rust Developers|System Integrators]
-- Prerequisites: [What users need before running this]
-- Expected Output: [What should happen when run successfully]
-- Version: [LLMSpell version this works with]
-- Tags: [agent, tool, workflow, state, error-handling, etc.]
```

### Rust Examples
```rust
//! Example: [Name of Example]
//! Purpose: [What this example demonstrates]
//! Audience: [Rust Developers|Library Users]
//! Prerequisites: [Required dependencies or setup]
//! Expected Output: [What should happen when run successfully]
//! Version: [LLMSpell version this works with]
//! Tags: [agent, tool, workflow, state, error-handling, etc.]
```

### 🏗️ Structure Requirements

### File Organization
1. **Naming Convention**:
   - Use kebab-case for files: `error-handling.lua`, `custom-tool.rs`
   - Prefix with numbers for sequential learning: `00-hello-world.lua`
   - Be descriptive but concise

2. **Directory Placement**:
   - Place in appropriate audience directory
   - Use correct scope subdirectory
   - Group related examples together

### Code Structure
1. **Imports/Requires First**:
   ```lua
   -- All requires at the top
   local json = require("json")
   ```

2. **Configuration Section**:
   ```lua
   -- Configuration (use environment variables)
   local API_KEY = os.getenv("OPENAI_API_KEY")
   local CONFIG = {
       timeout = 30,
       retries = 3
   }
   ```

3. **Main Logic**:
   - Clear section comments
   - Logical flow from simple to complex
   - Group related operations

4. **Error Handling**:
   - Always include error handling
   - Show both success and failure paths
   - Provide helpful error messages

### ✅ Quality Requirements

### Documentation
1. **Inline Comments**:
   - Explain WHY, not just WHAT
   - Comment complex logic
   - Note important assumptions

2. **Expected Output**:
   - Document what successful execution looks like
   - Include sample output in comments
   - Note any side effects

### Error Handling
1. **Required Patterns**:
   ```lua
   -- Good: Proper error handling
   local success, result = pcall(function()
       return Tool.execute("FileReader", {path = "data.txt"})
   end)
   
   if not success then
       print("Error reading file: " .. tostring(result))
       -- Graceful degradation or recovery
   end
   ```

2. **Never Do**:
   ```lua
   -- Bad: No error handling
   local data = Tool.execute("FileReader", {path = "data.txt"})
   ```

### Security
1. **No Hardcoded Secrets**:
   ```lua
   -- Good: Use environment variables
   local api_key = os.getenv("OPENAI_API_KEY")
   
   -- Bad: Hardcoded secrets
   local api_key = "sk-abc123..."  -- NEVER DO THIS
   ```

2. **Input Validation**:
   - Validate user inputs
   - Sanitize file paths
   - Check boundaries

### 🧪 Testing Requirements

### Self-Contained
- Examples must be runnable without external dependencies
- Include test data or generate it
- Clean up after execution

### CI Integration
- Must pass in CI environment
- Handle missing API keys gracefully
- Timeout appropriately

### Verification
```lua
-- Include verification at the end
assert(result ~= nil, "Result should not be nil")
assert(type(result) == "table", "Result should be a table")
print("✅ Example completed successfully!")
```

### 📝 Documentation Requirements

### README Files
Each directory must have a README.md that includes:
1. Purpose of the examples
2. Prerequisites
3. How to run
4. Common issues
5. Learning path

### Cross-References
- Link to related examples
- Reference documentation
- Point to next steps

### 🚀 Performance Considerations

### Resource Usage
1. **Clean Up Resources**:
   ```lua
   -- Always clean up
   if session then
       session:close()
   end
   ```

2. **Reasonable Limits**:
   - Don't create 1000s of objects
   - Use appropriate timeouts
   - Limit retry attempts

### Efficiency
- Show efficient patterns
- Avoid unnecessary operations
- Demonstrate caching where appropriate

### 🔄 Maintenance

### Version Compatibility
- Tag examples with version
- Update when APIs change
- Note deprecations

### Regular Testing
- Test with each release
- Update for breaking changes
- Keep output current

### 📏 Style Guide

### Lua Style
```lua
-- Use local variables
local my_var = "value"

-- Use snake_case for variables
local user_name = "Alice"

-- Use PascalCase for globals (Tool, Agent, etc.)
local result = Tool.execute("Calculator", {})

-- Consistent indentation (2 spaces)
if condition then
  do_something()
end
```

### Rust Style
```rust
// Follow Rust conventions
use llmspell::prelude::*;

// Use snake_case for functions
fn process_data(input: &str) -> Result<String> {
    // Implementation
}

// Use CamelCase for types
struct CustomAgent {
    // Fields
}

// Proper error handling
let result = operation().context("Failed to perform operation")?;
```

## ✨ Example Template

### Lua Template
```lua
-- Example: Template Example
-- Purpose: Demonstrates example structure
-- Audience: Script Users
-- Prerequisites: None
-- Expected Output: Prints "Hello, World!"
-- Version: 0.7.0
-- Tags: basic, template

-- Configuration
local CONFIG = {
    message = "Hello, World!"
}

-- Main function
local function main()
    -- Example logic here
    print(CONFIG.message)
    
    -- Verification
    assert(CONFIG.message ~= nil, "Message should not be nil")
    print("✅ Example completed successfully!")
end

-- Error handling wrapper
local success, err = pcall(main)
if not success then
    print("❌ Error: " .. tostring(err))
    os.exit(1)
end
```

### Rust Template
```rust
//! Example: Template Example
//! Purpose: Demonstrates example structure
//! Audience: Rust Developers
//! Prerequisites: llmspell = "0.7"
//! Expected Output: Prints "Hello, World!"
//! Version: 0.7.0
//! Tags: basic, template

use anyhow::Result;
use llmspell::prelude::*;

fn main() -> Result<()> {
    // Configuration
    let config = Config {
        message: "Hello, World!".to_string(),
    };
    
    // Example logic
    println!("{}", config.message);
    
    // Verification
    assert!(!config.message.is_empty());
    println!("✅ Example completed successfully!");
    
    Ok(())
}

struct Config {
    message: String,
}
```

### 🎯 Goals

Every example should:
1. **Teach** - Demonstrate a concept clearly
2. **Work** - Run successfully out of the box
3. **Inspire** - Show what's possible
4. **Guide** - Lead to next steps
5. **Document** - Be self-explanatory

---

## Example Catalog

### Script Users (50+ Lua Examples)

#### Getting Started (6 Progressive Examples)
| File | Purpose | Concepts | Time |
|------|---------|----------|------|
| `00-hello-world.lua` | Simplest possible script | Basic structure | 2 min |
| `01-first-tool.lua` | Using the file tool | Tool execution, parameters | 5 min |
| `02-first-agent.lua` | Creating an LLM agent | Agent creation, providers | 10 min |
| `03-first-workflow.lua` | Sequential workflow | Workflow patterns | 15 min |
| `04-handle-errors.lua` | Error handling patterns | Try-catch, recovery | 10 min |
| `05-first-rag.lua` | RAG basics (Phase 8) | Vectors, search, embeddings | 20 min |

#### Features (5 Core Demonstrations)
| File | Purpose | Key APIs |
|------|---------|----------|
| `tool-basics.lua` | All 37 tools overview | `Tool.list()`, `Tool.execute()` |
| `agent-basics.lua` | Agent patterns | `Agent.builder()`, providers |
| `workflow-basics.lua` | 4 workflow types | Sequential, Parallel, Conditional, Loop |
| `state-persistence.lua` | State management | `State.set()`, `State.get()` |
| `provider-info.lua` | Provider configuration | Multiple LLM providers |

#### Cookbook (11 Production Patterns)
| File | Purpose | Production Use |
|------|---------|----------------|
| `error-handling.lua` | Comprehensive error recovery | Retry, fallback, logging |
| `rate-limiting.lua` | API quota management | Token buckets, backoff |
| `caching.lua` | Performance optimization | Memory, Redis caching |
| `multi-agent-coordination.lua` | Agent collaboration | Pipeline, fork-join |
| `webhook-integration.lua` | External systems | HTTP callbacks |
| `performance-monitoring.lua` | Observability | Metrics, tracing |
| `security-patterns.lua` | Input validation | Sanitization, validation |
| `state-management.lua` | Persistence patterns | Save/load, migrations |
| **RAG Patterns (Phase 8):** | | |
| `rag-multi-tenant.lua` | Tenant isolation | StateScope, quotas |
| `rag-session.lua` | Conversational memory | Session vectors |
| `rag-cost-optimization.lua` | 70% cost reduction | Caching, batching |

#### Advanced Patterns (4 Examples)
| File | Purpose | Complexity |
|------|---------|------------|
| `multi-agent-orchestration.lua` | Complex coordination | High |
| `complex-workflows.lua` | Nested workflows | High |
| `tool-integration-patterns.lua` | Tool chaining | Medium |
| `monitoring-security.lua` | Production monitoring | High |

#### Applications (9 Complete Apps)
| Directory | Description | Features |
|-----------|-------------|----------|
| `webapp-creator/` | Multi-agent web app generator | Agents, workflows, tools |
| `code-review-assistant/` | Automated code review | Git integration, analysis |
| `content-creator/` | Multi-format content | Templates, generation |
| `communication-manager/` | Email orchestration | SMTP, templates |
| `file-organizer/` | Intelligent file organization | Patterns, rules |
| `process-orchestrator/` | Workflow automation | State machines |
| `research-collector/` | v2.0 with RAG | Web scraping, RAG |
| **RAG Applications (Phase 8):** | | |
| `knowledge-base/` | Personal knowledge management | RAG, search, Q&A |
| `personal-assistant/` | AI productivity companion | RAG, tools, agents |

### Rust Developers (6 Examples)
| Directory | Purpose | Key Concepts |
|-----------|---------|--------------||
| `custom-tool-example/` | Tool creation | Tool trait, BaseAgent |
| `custom-agent-example/` | Agent implementation | Providers, execution |
| `async-patterns-example/` | Concurrent execution | Tokio, futures |
| `extension-pattern-example/` | Plugin architecture | Traits, dynamic loading |
| `builder-pattern-example/` | Configuration | Builder pattern |
| `integration-test-example/` | Testing strategies | Test helpers |

### Tests & Benchmarks
| File | Purpose | Coverage |
|------|---------|----------|
| `test-rag-basic.lua` | Basic RAG operations | Ingest, search |
| `test-rag-e2e.lua` | End-to-end testing | Full pipeline |
| `test-rag-errors.lua` | Error handling | Edge cases |
| `rag-benchmark.lua` | Performance measurement | Latency, throughput |

### Configuration Templates (10 Custom Configs)

**Note**: Most users should use builtin profiles (`-p profile-name`) instead. Custom configs are for advanced scenarios.

| File | Purpose | When to Use |
|------|---------|-------------|
| `basic.toml` | Basic features | Learning state |
| `example-providers.toml` | OpenAI + Anthropic | Custom provider settings |
| `state-enabled.toml` | Persistence | Custom state backends |
| `rag-basic.toml` | RAG starter | Simple RAG learning |
| `rag-multi-tenant.toml` | Multi-tenant RAG | SaaS platforms |
| `applications.toml` | Full features | Complete apps |

**Builtin Profiles** (use `-p profile-name`): minimal, development, providers, state, sessions, ollama, candle, rag-dev, rag-prod, rag-perf

---

## Learning Paths

### 🎓 Beginner Path (2-3 hours)
```
1. 00-hello-world.lua         (5 min)
   ↓
2. 01-first-tool.lua          (10 min)
   ↓
3. 02-first-agent.lua         (15 min)
   ↓
4. tool-basics.lua            (20 min)
   ↓
5. 04-handle-errors.lua       (15 min)
   ↓
6. error-handling.lua         (30 min)
```

### 🚀 RAG Developer Path (4-5 hours)
```
1. 05-first-rag.lua           (30 min)
   ↓
2. rag-multi-tenant.lua       (45 min)
   ↓
3. rag-session.lua            (45 min)
   ↓
4. rag-cost-optimization.lua  (30 min)
   ↓
5. knowledge-base/            (60 min)
   ↓
6. personal-assistant/        (60 min)
```

### 🏗️ Production Path (6-8 hours)
```
1. multi-agent-coordination.lua  (45 min)
   ↓
2. performance-monitoring.lua    (30 min)
   ↓
3. security-patterns.lua         (30 min)
   ↓
4. webapp-creator/               (90 min)
   ↓
5. production-guide.md           (60 min)
```

### 🦀 Rust Extension Path (4-6 hours)
```
1. custom-tool-example/       (60 min)
   ↓
2. custom-agent-example/      (60 min)
   ↓
3. async-patterns-example/    (45 min)
   ↓
4. builder-pattern-example/   (30 min)
   ↓
5. integration-test-example/  (45 min)
```

---

## Pattern Library

### Error Handling Pattern
```lua
-- From cookbook/error-handling.lua
local function safe_operation(fn)
    local success, result = pcall(fn)
    if not success then
        Logger.error("Operation failed", {error = result})
        -- Retry logic
        for i = 1, 3 do
            success, result = pcall(fn)
            if success then break end
            os.execute("sleep " .. (i * 2))  -- Exponential backoff
        end
    end
    return success, result
end
```

### Multi-Agent Pipeline Pattern
```lua
-- From cookbook/multi-agent-coordination.lua
local pipeline = Workflow.sequential({
    name = "research_pipeline",
    steps = {
        {type = "agent", id = "researcher"},
        {type = "agent", id = "analyst"},
        {type = "agent", id = "writer"}
    }
})
```

### RAG with Caching Pattern
```lua
-- From cookbook/rag-cost-optimization.lua
local function cached_search(query)
    local cache_key = "search:" .. query
    local cached = State.get(cache_key)
    
    if cached then
        return cached
    end
    
    local results = RAG.search(query, {limit = 10})
    State.set(cache_key, results, {ttl = 3600})  -- 1 hour TTL
    return results
end
```

### State Management Pattern
```lua
-- From cookbook/state-management.lua
local function with_state_transaction(fn)
    State.begin_transaction()
    local success, result = pcall(fn)
    
    if success then
        State.commit()
    else
        State.rollback()
        error(result)
    end
    
    return result
end
```

---

## Running Examples

### Individual Examples
```bash
# Basic example
./target/debug/llmspell run examples/script-users/getting-started/00-hello-world.lua

# With configuration
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/getting-started/05-first-rag.lua

# Application with arguments
cd examples/script-users/applications/webapp-creator
../../../../target/debug/llmspell -c config.toml run main.lua \
  -- --input user-input-ecommerce.lua --output ./generated
```

### Batch Execution
```bash
# Run all getting-started examples
for file in examples/script-users/getting-started/*.lua; do
    echo "Running $file"
    ./target/debug/llmspell run "$file"
done

# Run all cookbook examples with appropriate configs
for file in examples/script-users/cookbook/*.lua; do
    config="examples/script-users/configs/example-providers.toml"
    if [[ $file == *"rag"* ]]; then
        config="examples/script-users/configs/rag-basic.toml"
    fi
    ./target/debug/llmspell -c "$config" run "$file"
done
```

---

## Common Issues & Solutions

| Issue | Solution | Example |
|-------|----------|---------||
| "API key not found" | Set environment variables | `export OPENAI_API_KEY="..."` |
| "Tool not found" | Check tool name spelling | Use `Tool.list()` to see available |
| "Agent timeout" | Increase timeout or check network | `agent.timeout = 60` |
| "RAG not available" | Use RAG-enabled config | `-c configs/rag-basic.toml` |
| "Permission denied" | Check file paths and sandbox | Use `/tmp` or workspace paths |

---

## Summary

**60+ Production Examples** covering:
- ✅ All 37 tools
- ✅ Agent patterns and multi-agent coordination
- ✅ 4 workflow types
- ✅ State persistence
- ✅ RAG with multi-tenancy (Phase 8)
- ✅ Production patterns (error handling, caching, monitoring)
- ✅ 9 complete applications
- ✅ Rust extension patterns

**Quality**: All examples follow standards and include proper headers, error handling, and documentation.

**Learning**: Progressive paths from beginner to production deployment.

---

*This reference combines example standards with a comprehensive catalog of all 60+ examples, learning paths, and common patterns from the production cookbook.*