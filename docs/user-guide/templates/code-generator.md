# Code Generator Template

**Version:** 0.1.0
**Category:** CodeGen
**Status:** Production Structure (Phase 12.4.3)

## Overview

The Code Generator template automates the creation of source code from natural language descriptions. It follows a 3-phase workflow: specification generation, implementation, and test creation.

### What It Does

- **Specification Generation**: Convert requirements to detailed specs
- **Code Implementation**: Generate working code in multiple languages
- **Test Generation**: Create comprehensive unit and integration tests
- **Documentation**: Auto-generate code comments and README files

### Use Cases

- Rapid prototyping
- Boilerplate code generation
- Test suite scaffolding
- API client generation
- Data structure implementation

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec code-generator \
  --param description="Create a binary search tree in Rust" \
  --param language="rust"
```

### Lua - Basic Usage

```lua
local result = Template.execute("code-generator", {
    description = "REST API client with error handling",
    language = "rust"
})

print(result.result)
```

---

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `description` | String | Natural language description of code to generate |
| `language` | Enum | Programming language: `rust`, `python`, `javascript`, `typescript`, `go` |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `include_tests` | Boolean | `true` | Generate unit tests |
| `include_docs` | Boolean | `true` | Generate documentation |
| `style_guide` | String | Language default | Coding style guide to follow |
| `model` | String | `"ollama/llama3.2:3b"` | LLM for code generation |

**Inspect Full Schema:**
```bash
llmspell template schema code-generator
```

---

## Implementation Status

✅ **Production Structure**: This template has complete structure with validation (Phase 12.4.3)

**Implemented:**
- ✅ Template metadata and parameter schema
- ✅ Parameter validation with constraints
- ✅ 3-phase workflow structure
- ✅ Cost estimation
- ✅ Output formatting
- ✅ 14 comprehensive unit tests

**Placeholder Content:**
- ⏳ LLM-based code generation (uses placeholder text)
- ⏳ Real test generation
- ⏳ Multi-language support

**Expected**: Full LLM integration in Phase 14

---

## Output Format

### Generated Artifacts

```
code_generator_output/
├── specification.md         # Detailed implementation spec
├── implementation.{ext}     # Generated source code
├── tests.{ext}              # Unit and integration tests
└── README.md                # Documentation
```

### CLI Output
```bash
llmspell template exec code-generator \
  --param description="Binary tree with traversal" \
  --param language="rust" \
  --output-dir ./generated_code

# Creates files in ./generated_code/
```

### Lua Access
```lua
local result = Template.execute("code-generator", {
    description = "HTTP client with retry logic",
    language = "rust",
    include_tests = true
})

-- Access artifacts
for _, artifact in ipairs(result.artifacts) do
    if artifact.filename:match("%.rs$") then
        print("Generated Rust code:")
        print(artifact.content)
    end
end
```

---

## Examples

### CLI Examples

#### Basic Code Generation
```bash
llmspell template exec code-generator \
  --param description="LRU cache with generic types" \
  --param language="rust" \
  --param include_tests=true
```

#### Generate Without Tests
```bash
llmspell template exec code-generator \
  --param description="JSON parser with error handling" \
  --param language="python" \
  --param include_tests=false
```

#### Multiple Languages
```bash
# Generate in Rust
llmspell template exec code-generator \
  --param description="Rate limiter" \
  --param language="rust" \
  --output-dir ./rust_impl

# Generate same in Python
llmspell template exec code-generator \
  --param description="Rate limiter" \
  --param language="python" \
  --output-dir ./python_impl
```

### Lua Examples

```lua
-- Generate with full documentation
local result = Template.execute("code-generator", {
    description = "Concurrent task queue with priorities",
    language = "rust",
    include_tests = true,
    include_docs = true
})

-- Save artifacts
for _, artifact in ipairs(result.artifacts) do
    local file = io.open(artifact.filename, "w")
    file:write(artifact.content)
    file:close()
    print("Saved: " .. artifact.filename)
end

-- Check metrics
print("Generation took: " .. result.metrics.duration_ms .. "ms")
print("Tokens used: " .. (result.metrics.tokens_used or "N/A"))
```

---

## Cost Estimation

```bash
llmspell template info code-generator --show-schema
```

### Estimated Costs

| Configuration | Tokens | Duration | Cost (USD) |
|--------------|--------|----------|------------|
| Basic (no tests) | ~1,500 | ~5s | $0.00015 |
| With tests | ~3,000 | ~10s | $0.00030 |
| Full (tests + docs) | ~4,500 | ~15s | $0.00045 |

**Note**: Actual costs vary by model, description complexity, and code size.

---

## Troubleshooting

### Error: "Unsupported language: java"

**Cause**: Language not in supported list

**Solution**: Use supported language:
```bash
--param language="rust"     # Supported
--param language="python"   # Supported
--param language="javascript" # Supported
--param language="typescript" # Supported
--param language="go"       # Supported
```

### Generated Code Quality

**Current Status**: Phase 12.4.3 uses placeholder code generation.

**For Production Use**:
1. Review and test all generated code
2. Apply security auditing
3. Run linters and formatters
4. Add manual tests where needed

**Future**: Phase 14 will integrate production LLM generation

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Research Assistant Template](./research-assistant.md) (production example)
- [Code Quality Guidelines](../../development/code-quality.md)

---

## Roadmap

### Phase 12.4.3 (Current)
- ✅ Complete structure and validation
- ✅ 3-phase workflow design
- ⏳ Placeholder generation

### Phase 14 (Planned)
- LLM-powered code generation
- Multi-language support
- Style guide enforcement
- Security best practices
- Code review integration

### Phase 15 (Future)
- Interactive refinement
- Code optimization suggestions
- Performance analysis
- Collaborative code generation

---

**Last Updated**: Phase 12.4.3 (Production Structure)
**Next Review**: Phase 14 (LLM Integration)
