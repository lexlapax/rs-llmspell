# Code Generator Template

**Version:** 0.1.0
**Category:** CodeGen
**Status:** ✅ Production Ready (Phase 12.8.3)

## Overview

The Code Generator template automates the creation of source code from natural language descriptions using a 3-agent sequential chain:

1. **Specification Agent** - Convert description to detailed technical specification
2. **Implementation Agent** - Generate working code from specification
3. **Test Agent** - Create comprehensive unit tests for the code
4. **Static Analysis** - In-memory code quality checks

### What It Does

The Code Generator template orchestrates multiple AI agents to:

- **Specify**: Generate detailed technical specification from natural language (Spec Agent)
- **Implement**: Write experimental workflows for rapid exploration code following the spec (Implementation Agent)
- **Test**: Create comprehensive unit tests with edge cases (Test Agent)
- **Analyze**: Perform static code quality checks (pattern detection)

### Use Cases

- Rapid prototyping from requirements
- Boilerplate code generation
- Test suite scaffolding
- Algorithm implementation from descriptions
- Data structure creation
- API client generation

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec code-generator \
  --param description="Create a binary search tree in Rust" \
  --param language="rust"
```

### CLI - With Memory and Provider

Enable memory-enhanced execution with custom provider:

```bash
llmspell template exec code-generator \
  --param description="Create a binary search tree in Rust" \
  --param language="rust" \
  --param session-id="user-session-123" \
  --param memory-enabled=true \
  --param context-budget=3000 \
  --param provider-name="ollama"
```

### Lua - Basic Usage

```lua
local result = Template.execute("code-generator", {
    description = "REST API client with error handling",
    language = "rust"
})

print(result.result.value)
```

### Lua - With Memory and Provider

Enable memory-enhanced execution:

```lua
local result = Template.execute("code-generator", {
    description = "REST API client with error handling",
    language = "rust",
    session_id = "user-session-123",
    memory_enabled = true,
    context_budget = 3000,
    provider_name = "ollama"
})
```

---

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `description` | String | Natural language description of code to generate (minimum 10 characters) |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `language` | Enum | `"rust"` | Programming language (see below) |
| `include_tests` | Boolean | `true` | Whether to generate unit tests |
| `model` | String | `"ollama/llama3.2:3b"` | LLM model for generation agents |

### Memory Parameters

All templates support optional memory integration for context-aware execution:

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `session_id` | String | `null` | Any string | Session identifier for conversation memory filtering |
| `memory_enabled` | Boolean | `true` | `true`, `false` | Enable memory-enhanced execution (uses episodic + semantic memory) |
| `context_budget` | Integer | `2000` | `100-8000` | Token budget for context assembly (higher = more context) |

**Memory Integration**: When `session_id` is provided and `memory_enabled` is `true`, the template will:
- Retrieve relevant episodic memory from conversation history
- Query semantic memory for related concepts
- Assemble context within the `context_budget` token limit
- Provide memory-enhanced context to LLM for better results

### Provider Parameters

Templates support dual-path provider resolution:

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `provider_name` | String | `null` | `"ollama"`, `"openai"`, etc. | Provider name (mutually exclusive with `model`) |

**Provider Resolution**:
- Use `provider_name` to select a provider with its default model (e.g., `provider_name: "ollama"`)
- Use `model` for explicit model selection (e.g., `model: "gpt-4"`)
- If both provided, `model` takes precedence
- `provider_name` and `model` are mutually exclusive

### Supported Languages

- **rust** - Rust with idiomatic patterns, `#[test]` framework
- **python** - Python with unittest/pytest
- **javascript** - JavaScript with Jest/Mocha
- **typescript** - TypeScript with Jest
- **go** - Go with testing package
- **java** - Java with JUnit
- **cpp** - C++ with Google Test
- **lua** - Lua with idiomatic patterns, busted/luaunit framework

**Inspect Full Schema:**
```bash
llmspell template schema code-generator
```

---

## Implementation Details

### Phase 1: Specification Agent (96 lines)
- **AgentConfig**: Temperature 0.3 (structured, deterministic specs)
- **Max Tokens**: 2000 (detailed spec output)
- **Resource Limits**: 120s execution time, 512MB memory, 0 tool calls
- **Model Parsing**: Split "provider/model-id", default to "ollama"
- **Agent Creation**: `context.agent_registry().create_agent(config)` → `Arc<dyn Agent>`
- **Prompt Strategy**: "You are an expert software architect..." + requirements list
- **Structured Guidelines**: Numbered sections (Purpose, Data Structures, Algorithms, API, Error Handling, Testing)
- **Output**: Technical specification document with implementation guidance

**Temperature Rationale**: 0.3 is low for deterministic, structured output - specs need consistency

### Phase 2: Implementation Agent (96 lines)
- **AgentConfig**: Temperature 0.5 (creative for design choices)
- **Max Tokens**: 3000 (longer code output)
- **Resource Limits**: 180s execution time (3 min), 512MB memory, 0 tool calls
- **Input**: Full specification from Phase 1
- **Prompt Strategy**: "Provide ONLY the code (no explanations)" - prevents verbose output
- **Language-Specific**: "{language}-idiomatic patterns" in prompt
- **Quality Signal**: "experimental workflows for rapid exploration, not just a stub"
- **Output**: Implemented code following specification

**Temperature Rationale**: 0.5 higher than spec (0.3) - implementation needs creativity for data structures/algorithms, but still structured for correctness

### Phase 3: Test Agent (104 lines)
- **AgentConfig**: Temperature 0.4 (balanced - creative for edge cases, structured for syntax)
- **Max Tokens**: 2500 (comprehensive tests, shorter than implementation)
- **Resource Limits**: 150s execution time (2.5 min), 512MB memory, 0 tool calls
- **Input**: Implementation code from Phase 2
- **Test Framework Dispatch**: Language-specific (Rust: `#[test]`, Python: unittest/pytest, etc.)
- **Coverage Requirements**: ">80% code coverage" in prompt
- **Test Scenarios**: Happy path, edge cases (empty, null, boundary), error conditions
- **Output**: Comprehensive test suite

**Temperature Rationale**: 0.4 between spec (0.3) and impl (0.5) - creative for edge cases, deterministic for valid syntax

### Phase 4: Static Code Analysis (116 lines)
- **Pragmatic Approach**: In-memory pattern detection (not external linters)
- **Rationale**: Tool-based linting (clippy, pylint, eslint) requires file system + process execution - templates operate on in-memory strings
- **Metrics Provided**:
  - Line counts: total, non-empty, comments, code
  - Comment density percentage
  - Code density (non-empty/total ratio)
- **Pattern Detection**:
  - Error handling: Searches for "Error", "Result", "Exception", "try"
  - Documentation: Language-specific checks (/// for Rust, """ for Python, /** for JS)
- **User Guidance**: Report includes notes about static vs tool-based analysis
- **Recommendations**: Suggests language-specific linters for comprehensive checks

**No External Tools**: Works out-of-box without clippy, pylint, eslint installed

---

## Execution Phases

### Phase 1: Specification Generation

**Duration**: ~5-8s
**Infrastructure**: Requires AgentRegistry, LLM

Creates a specification agent that:
1. Analyzes the natural language description
2. Designs data structures and algorithms
3. Defines API signatures and contracts
4. Specifies error handling strategy
5. Outlines testing approach

**Output**: Technical specification document (typically 50-100 lines)

### Phase 2: Code Implementation

**Duration**: ~8-12s
**Infrastructure**: Requires AgentRegistry, LLM

Creates an implementation agent that:
1. Reads the specification from Phase 1
2. Generates experimental workflows for rapid exploration code
3. Follows language-specific idioms
4. Includes error handling
5. Applies best practices

**Output**: Implemented code (typically 80-150 lines)

### Phase 3: Test Generation

**Duration**: ~6-10s
**Infrastructure**: Requires AgentRegistry, LLM

Creates a test agent that:
1. Analyzes the implementation code
2. Generates comprehensive unit tests
3. Covers happy path, edge cases, errors
4. Uses language-specific test frameworks
5. Targets >80% code coverage

**Output**: Test suite (typically 80-120 lines)

### Phase 4: Static Analysis

**Duration**: ~0.1s
**Infrastructure**: None (in-memory)

Performs static code analysis:
1. Counts lines (total, non-empty, comments, code)
2. Detects error handling patterns
3. Checks for documentation
4. Calculates metrics
5. Generates quality report

**Output**: Code quality analysis report

---

## Output Format

### Text Output

```
# Code Generation Report

**Description**: A simple calculator that can add and subtract two numbers
**Language**: rust
**Tests Included**: Yes

---

## Specification

**Purpose**: Create a simple calculator with addition and subtraction
**Data Structures**: Struct Calculator with methods add() and subtract()
**Algorithms**: Basic arithmetic operations with type safety
...

---

## Implementation

```rust
pub struct Calculator;

impl Calculator {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn subtract(a: i32, b: i32) -> i32 {
        a - b
    }
}
```

---

## Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Calculator::add(2, 3), 5);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(Calculator::subtract(5, 3), 2);
    }
}
```

---

## Code Quality Analysis

**Metrics**:
- Total lines: 90
- Code lines: 75
- Comment lines: 10
- Comment density: 13.3%

**Quality Checks**:
- ✓ Error handling detected
- ✓ Documentation present

---

Generated by LLMSpell Code Generator Template
```

### JSON Output

```json
{
  "status": "ok",
  "result": {
    "type": "text",
    "value": "# Code Generation Report\n\n..."
  },
  "artifacts": [
    {
      "filename": "specification.md",
      "content": "...",
      "mime_type": "text/markdown"
    },
    {
      "filename": "implementation.rs",
      "content": "...",
      "mime_type": "text/x-rust"
    },
    {
      "filename": "tests.rs",
      "content": "...",
      "mime_type": "text/x-rust"
    }
  ],
  "metrics": {
    "duration_ms": 21600,
    "agents_invoked": 3,
    "custom_metrics": {
      "language": "rust",
      "spec_lines": 79,
      "code_lines": 90,
      "test_lines": 94
    }
  }
}
```

---

## Examples

### Example 1: Basic Calculator (Rust)

```bash
llmspell template exec code-generator \
  --param description="A simple calculator that can add and subtract two numbers" \
  --param language="rust" \
  --param model="ollama/llama3.2:3b" \
  --output text
```

**Result**: 21.6 seconds, 79-line spec, 90-line code, 94-line tests

### Example 2: Binary Search Tree (Python)

```bash
llmspell template exec code-generator \
  --param description="Binary search tree with insert, search, and delete operations" \
  --param language="python" \
  --param include_tests=true
```

**Result**: Technical spec + Python implementation with unittest tests

### Example 3: HTTP Client (TypeScript)

```bash
llmspell template exec code-generator \
  --param description="HTTP client with retry logic and timeout handling" \
  --param language="typescript" \
  --param model="ollama/llama3.2:3b" \
  --output json
```

**Result**: JSON output with specification, TypeScript code, Jest tests

### Example 4: Generate Without Tests

```bash
llmspell template exec code-generator \
  --param description="LRU cache with generic types" \
  --param language="rust" \
  --param include_tests=false
```

**Result**: Specification + implementation only (no tests)

### Example 5: Lua Batch Generation

```lua
local descriptions = {
    "Rate limiter with token bucket algorithm",
    "JSON parser with error recovery",
    "Merkle tree implementation"
}

for _, desc in ipairs(descriptions) do
    print("\nGenerating: " .. desc)

    local result = Template.execute("code-generator", {
        description = desc,
        language = "rust",
        include_tests = true
    })

    if result.status == "ok" then
        -- Save artifacts
        for _, artifact in ipairs(result.artifacts) do
            local file = io.open(artifact.filename, "w")
            file:write(artifact.content)
            file:close()
        end
        print("  ✓ Generated: " .. result.metrics.custom_metrics.code_lines .. " lines")
    else
        print("  ✗ Failed")
    end
end
```

---

## Performance

**Test Configuration**:
- Description: "A simple calculator that can add and subtract two numbers"
- Language: rust
- Model: ollama/llama3.2:3b
- Include tests: true

**Results**:
- **Duration**: 21.6 seconds total
- **Agents**: 3 (spec-agent, impl-agent, test-agent)
- **Output**:
  - Specification: 79 lines
  - Implementation: 90 lines
  - Tests: 94 lines
- **Quality**: Zero clippy warnings, all tests pass

**Phase Breakdown**:
- Phase 1 (Spec): ~5-8s
- Phase 2 (Impl): ~8-12s
- Phase 3 (Test): ~6-10s
- Phase 4 (Analysis): ~0.1s

**Scaling**:
- Simple tasks (calculator, utilities): 15-25 seconds
- Medium tasks (data structures, algorithms): 25-40 seconds
- Complex tasks (multi-module systems): 40-60 seconds

---

## Troubleshooting

### Error: "Parameter 'description' too short"

**Cause**: Description is less than 10 characters.

**Solution**: Provide detailed description:
```bash
# Too short
--param description="calc"

# Good
--param description="A calculator with addition and subtraction"
```

### Error: "Unsupported language: java8"

**Cause**: Invalid language parameter value.

**Solution**: Use exact supported language names:
```bash
--param language="rust"       # ✓ Valid
--param language="python"     # ✓ Valid
--param language="javascript" # ✓ Valid
--param language="typescript" # ✓ Valid
--param language="go"         # ✓ Valid
--param language="java"       # ✓ Valid
--param language="cpp"        # ✓ Valid
--param language="lua"        # ✓ Valid
```

### Generated Code Quality

**Issue**: Code doesn't compile or has syntax errors.

**Solutions**:
1. Review specification - ensure it's clear and complete
2. Check language-specific output - verify idiomatic patterns used
3. Run language-specific linters:
   - Rust: `rustfmt` + `cargo clippy`
   - Python: `black` + `pylint`
   - JavaScript/TypeScript: `eslint` + `prettier`
   - Go: `gofmt` + `golint`

**Note**: Static analysis is basic pattern detection - always run language-specific tools for production code.

### Tests Failing

**Issue**: Generated tests don't pass.

**Cause**: Implementation may not match test expectations.

**Solutions**:
1. Review specification and implementation alignment
2. Check test assumptions against actual behavior
3. Manually adjust tests or implementation as needed
4. Re-run generation with clearer description

---

## Architecture Insights

### Why 3-Agent Chain?

**Rationale**: Sequential agent chain provides:
- **Separation of Concerns**: Spec, implementation, and testing are distinct responsibilities
- **Quality Gates**: Each phase validates previous phase output
- **Iterative Refinement**: Spec guides implementation, implementation guides tests
- **Traceable**: Clear lineage from requirements → spec → code → tests

### Temperature Tuning Philosophy

- **Spec Agent (0.3)**: Low temperature for structured, deterministic specifications - specs need consistency across runs
- **Impl Agent (0.5)**: Medium temperature for creative design choices (data structures, algorithms) while maintaining correctness
- **Test Agent (0.4)**: Balanced temperature - creative for edge case discovery, deterministic for valid test syntax

### Why In-Memory Static Analysis?

**Rationale**:
- Templates operate on in-memory code strings, not files
- External linters (clippy, pylint, eslint) require file system + process execution
- Static analysis provides immediate value without external dependencies
- Users can run language-specific linters separately for comprehensive checks

### Sequential Data Flow

Data flows linearly through phases:
1. Description → Spec Agent → SpecificationResult (text)
2. SpecificationResult → Impl Agent → ImplementationResult (code)
3. ImplementationResult → Test Agent → TestResult (tests)
4. ImplementationResult → Static Analysis → QualityReport (metrics)

Each agent receives output from previous agent as input.

---

## Cost Estimation

**Estimated Costs (per execution)**

| Configuration | Tokens | Duration | Agents |
|--------------|--------|----------|--------|
| No tests | ~3,500 | ~15s | 2 (spec + impl) |
| With tests | ~6,000 | ~22s | 3 (spec + impl + test) |

**Token Breakdown**:
- Specification: ~1,500 tokens
- Implementation: ~2,000 tokens
- Tests: ~2,500 tokens

**Note**: Actual costs depend on model, description complexity, and code size.

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Research Assistant Template](./research-assistant.md) (4-phase RAG pipeline)
- [Data Analysis Template](./data-analysis.md) (3-agent chain pattern)
- [Interactive Chat Template](./interactive-chat.md) (session management)

---

## Implementation Status

### Phase 12.8.3 - ✅ COMPLETE (All 4 Phases)

**Implemented** (412 lines):
- ✅ Phase 1: Specification agent (96 lines)
- ✅ Phase 2: Implementation agent (96 lines)
- ✅ Phase 3: Test agent (104 lines)
- ✅ Phase 4: Static code analysis (116 lines)

**Quality Metrics**:
- ✅ Compilation: Clean (0 errors, 0 warnings)
- ✅ Clippy: Clean (0 warnings)
- ✅ End-to-end testing: 21.6s for calculator example
- ✅ Multi-language support: 8 languages (Rust, Python, JavaScript, TypeScript, Go, Java, C++, Lua)

**Key Achievements**:
1. First 3-agent sequential chain template (spec → impl → test)
2. Established temperature tuning pattern for different agent roles
3. In-memory static analysis without external dependencies
4. Language-specific test framework dispatch

**Timeline**:
- Estimated: 8-10 hours
- Actual: ~2.5 hours (efficient with established patterns from 12.8.1)

**Test Results**:
```
Description: "A simple calculator that can add and subtract two numbers"
Language: rust
Duration: 21.6 seconds
Output:
  - Specification: 79 lines
  - Implementation: 90 lines (Calculator struct with add/subtract)
  - Tests: 94 lines (comprehensive test coverage)
Quality: Zero clippy warnings, all tests pass
```

---

## Changelog

### v0.1.0 (Phase 12.8.3) - Production Ready

**Implemented** (412 lines total):
- ✅ 3-agent chain: Specification → Implementation → Test
- ✅ Static code analysis with in-memory pattern detection
- ✅ 8 language support (Rust, Python, JavaScript, TypeScript, Go, Java, C++, Lua)
- ✅ Language-specific test framework dispatch
- ✅ Temperature-tuned agents (0.3 spec, 0.5 impl, 0.4 test)
- ✅ End-to-end testing (21.6s for calculator)
- ✅ Zero clippy warnings

**Key Features**:
- Sequential agent chain with data flow
- Production-ready code generation
- Comprehensive test coverage (>80% target)
- In-memory quality analysis
- Type-safe parameter validation
- Rich error handling

---

**Last Updated**: Phase 12.8.3 (Production Implementation)
**Status**: ✅ Ready for Production Use
