# Code Review Template

**Category**: Development
**Version**: 0.1.0
**Status**: Production Ready

## Overview

The Code Review Template provides automated multi-aspect code analysis through 7 specialized AI reviewers. Each reviewer focuses on a specific domain (security, quality, performance, best practices, dependencies, architecture, documentation) to provide comprehensive, actionable feedback on your codebase.

**Key Features:**
- **7 Specialized Review Aspects**: Security, quality, performance, practices, dependencies, architecture, and documentation analysis
- **Severity Filtering**: Focus on critical/high/medium/low priority issues
- **Multiple Languages**: Rust, Python, JavaScript, TypeScript, Go, Java support
- **Fix Generation**: Automatic fix suggestions with before/after code snippets
- **Multiple Output Formats**: Markdown reports, JSON for CI/CD, plain text
- **Language-Specific Rules**: Tailored analysis for each programming language

## Quick Start

### Basic Review

Review a single file with all 7 aspects:

```bash
llmspell template exec code-review \
  --param code_path=src/main.rs \
  --param language=rust
```

### Security-Focused Review

Only analyze security vulnerabilities:

```bash
llmspell template exec code-review \
  --param code_path=src/auth.py \
  --param language=python \
  --param aspects='["security"]' \
  --param severity_filter=critical
```

### CLI - With Memory and Provider

Enable memory-enhanced execution with custom provider:

```bash
llmspell template exec code-review \
  --param code_path=src/main.rs \
  --param language=rust \
  --param session-id="user-session-123" \
  --param memory-enabled=true \
  --param context-budget=3000 \
  --param provider-name="ollama"
```

### CI/CD Integration

Generate JSON output for automated pipelines:

```bash
llmspell template exec code-review \
  --param code_path=src/ \
  --param language=javascript \
  --param severity_filter=high \
  --param output_format=json > review-results.json
```

### Lua - With Memory and Provider

Enable memory-enhanced execution:

```lua
local result = Template.execute("code-review", {
    code_path = "src/main.rs",
    language = "rust",
    session_id = "user-session-123",
    memory_enabled = true,
    context_budget = 3000,
    provider_name = "ollama"
})
```

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `code_path` | String | Path to file or directory to review |
| `language` | String | Programming language: `rust`, `python`, `javascript`, `typescript`, `go`, `java` |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `aspects` | Array | `["security", "quality", "performance", "practices", "dependencies", "architecture", "docs"]` | Review aspects to analyze |
| `severity_filter` | String | `"all"` | Filter by severity: `critical`, `high`, `medium`, `low`, `all` |
| `generate_fixes` | Boolean | `false` | Generate automatic fix suggestions |
| `output_format` | String | `"markdown"` | Output format: `markdown`, `json`, `text` |
| `model` | String | `"ollama/llama3.2:3b"` | LLM model for review agents |
| `temperature` | Float | `0.2` | Temperature (0.0-1.0) for LLM consistency |

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

## Review Aspects

### 1. Security Review (`security`)

Analyzes code for security vulnerabilities and sensitive data exposure.

**Checks:**
- Authentication and authorization flaws
- Injection vulnerabilities (SQL, command, XSS)
- Sensitive data exposure (API keys, passwords, PII)
- Cryptographic weaknesses
- Insecure configurations
- Race conditions and concurrency issues

**Example Issue:**
```
Severity: CRITICAL
Description: Potential SQL injection in user_query function
Recommendation: Use parameterized queries instead of string concatenation
```

### 2. Quality Review (`quality`)

Focuses on code maintainability, readability, and best practices.

**Checks:**
- Code complexity and readability
- Error handling patterns
- Magic numbers and hardcoded values
- Code duplication (DRY violations)
- Naming conventions
- Documentation and comments

**Metrics Reported:**
- Complexity: high/medium/low
- Readability score: 1-10
- Maintainability score: 1-10

### 3. Performance Review (`performance`)

Identifies performance bottlenecks and optimization opportunities.

**Checks:**
- Inefficient algorithms (O(n²) or worse)
- Memory leaks and excessive allocations
- Unnecessary loops and iterations
- Database query optimization
- Caching opportunities
- Resource management issues

### 4. Best Practices Review (`practices`)

Validates adherence to software engineering principles and design patterns.

**Checks:**
- SOLID principles compliance
- Design pattern correctness
- Anti-patterns detection
- Code organization
- Testing considerations
- Documentation standards

### 5. Dependency Review (`dependencies`)

Analyzes project dependencies and module structure.

**Checks:**
- Outdated dependencies (if visible in imports)
- Unnecessary dependencies
- Circular dependencies
- Tight coupling issues
- Missing abstraction layers

### 6. Architecture Review (`architecture`)

Evaluates high-level design and system structure.

**Checks:**
- Modularity and separation of concerns
- Abstraction levels
- Component coupling and cohesion
- Scalability considerations
- Design pattern application

### 7. Documentation Review (`docs`)

Assesses code documentation completeness and quality.

**Checks:**
- API documentation coverage
- Code comment quality and clarity
- Function/method documentation
- Complex logic explanations
- README and usage examples

## Severity Levels

Issues are categorized by severity to help prioritize fixes:

| Level | Description | Examples |
|-------|-------------|----------|
| **Critical** | Security vulnerabilities, data loss risks | SQL injection, buffer overflow, auth bypass |
| **High** | Major bugs, significant performance issues | Memory leak, O(n³) algorithm, broken error handling |
| **Medium** | Code quality issues, maintainability concerns | Code duplication, missing tests, poor naming |
| **Low** | Minor improvements, style issues | Missing comments, magic numbers, formatting |

### Severity Filtering

The `severity_filter` parameter controls which issues are included:

- `critical`: Only critical issues
- `high`: Critical + High issues
- `medium`: Critical + High + Medium issues
- `low`: All issues (same as `all`)
- `all`: All severity levels (default)

## Output Formats

### Markdown (Default)

Human-readable reports with syntax highlighting:

```markdown
# Code Review Report

**File**: src/api/auth.rs
**Language**: rust

## Summary

- **Total Issues**: 3
  - Critical: 1
  - High: 1
  - Medium: 1
  - Low: 0

## Detailed Findings

### [CRITICAL - security] Line 42

**Description**: Hardcoded API key in source code
**Recommendation**: Move credentials to environment variables or secure vault

---
```

### JSON

Machine-readable format for CI/CD integration:

```json
{
  "total_issues": 3,
  "severity_counts": {
    "critical": 1,
    "high": 1,
    "medium": 1,
    "low": 0
  },
  "issues": [
    {
      "severity": "critical",
      "aspect": "security",
      "line": 42,
      "description": "Hardcoded API key in source code",
      "recommendation": "Move credentials to environment variables"
    }
  ],
  "summaries": [
    {
      "aspect": "security",
      "summary": "Found 1 critical security issue requiring immediate attention"
    }
  ]
}
```

### Plain Text

Concise console-friendly output:

```
=== CODE REVIEW RESULTS ===

Total Issues: 3
  Critical: 1
  High: 1
  Medium: 1
  Low: 0

=== ISSUES BY ASPECT ===

[CRITICAL] security (line 42)
  Description: Hardcoded API key in source code
  Recommendation: Move credentials to environment variables
```

## Fix Generation

Enable automatic fix suggestions with `generate_fixes=true`:

```bash
llmspell template exec code-review \
  --param code_path=src/utils.py \
  --param language=python \
  --param generate_fixes=true
```

**Fix Output Example:**

```markdown
## Suggested Fixes

### Fix 1: Remove magic number in timeout calculation

**Explanation**: Replace hardcoded 3600 with named constant for clarity

**Original**:
```python
timeout = 3600  # seconds
```

**Fixed**:
```python
SECONDS_PER_HOUR = 3600
timeout = SECONDS_PER_HOUR
```
```

## Language-Specific Analysis

Each language receives tailored analysis based on its idioms and best practices:

### Rust
- Ownership and borrow checker issues
- Unsafe code blocks scrutiny
- Cargo.toml dependency analysis
- Error propagation with `?` operator
- Clippy lint suggestions

### Python
- PEP 8 style compliance
- Type hints usage
- Exception handling patterns
- requirements.txt/poetry analysis
- Pythonic idioms (list comprehensions, generators)

### JavaScript/TypeScript
- ESLint rule violations
- Promise/async-await patterns
- package.json dependency audit
- TypeScript type safety (for .ts files)
- React/Vue/Node.js specific checks

### Go
- gofmt compliance
- go.mod dependency review
- Goroutine and channel patterns
- Error handling conventions
- Standard library usage

### Java
- Maven/Gradle dependency analysis
- Exception handling patterns
- Stream API usage
- Spring Boot specific checks (if detected)
- Memory management patterns

## Use Cases

### 1. Pre-Commit Reviews

Catch issues before they reach version control:

```bash
# Review staged changes
git diff --staged > /tmp/staged.diff
llmspell template exec code-review \
  --param code_path=/tmp/staged.diff \
  --param language=rust \
  --param severity_filter=high
```

### 2. Pull Request Automation

Automated PR reviews in CI/CD:

```yaml
# GitHub Actions example
- name: Code Review
  run: |
    llmspell template exec code-review \
      --param code_path=. \
      --param language=python \
      --param output_format=json \
      --param severity_filter=critical > review.json

- name: Comment on PR
  run: |
    # Parse review.json and post findings as PR comment
    gh pr comment ${{ github.event.pull_request.number }} \
      --body "$(cat review.json)"
```

### 3. Security Audits

Focus on security vulnerabilities only:

```bash
llmspell template exec code-review \
  --param code_path=src/ \
  --param language=java \
  --param aspects='["security", "dependencies"]' \
  --param severity_filter=critical \
  --param generate_fixes=true
```

### 4. Code Quality Gates

Enforce quality thresholds in CI:

```bash
# Exit with error if critical issues found
llmspell template exec code-review \
  --param code_path=src/ \
  --param language=typescript \
  --param output_format=json | \
jq -e '.severity_counts.critical == 0'
```

### 5. Legacy Code Analysis

Assess technical debt in large codebases:

```bash
llmspell template exec code-review \
  --param code_path=legacy/monolith.py \
  --param language=python \
  --param aspects='["quality", "architecture", "performance"]' \
  --param output_format=markdown > technical-debt-report.md
```

## Lua Script Integration

### Basic Review Script

```lua
local Template = require("llmspell.template")

-- Review a file
local result = Template.execute("code-review", {
    code_path = "src/main.rs",
    language = "rust",
    aspects = {"security", "quality", "performance"},
    severity_filter = "high"
})

print("Total Issues:", result.metadata.total_issues)
print("Critical:", result.metadata.critical_issues)

-- Exit with error if critical issues found
if result.metadata.critical_issues > 0 then
    os.exit(1)
end
```

### Multi-File Review Workflow

```lua
local Template = require("llmspell.template")

local files = {
    { path = "src/auth.rs", language = "rust" },
    { path = "src/api.py", language = "python" },
    { path = "src/utils.js", language = "javascript" }
}

local all_issues = {}

for _, file in ipairs(files) do
    print(string.format("Reviewing %s...", file.path))

    local result = Template.execute("code-review", {
        code_path = file.path,
        language = file.language,
        severity_filter = "high",
        output_format = "json"
    })

    local review = json.decode(result.output)
    for _, issue in ipairs(review.issues) do
        table.insert(all_issues, issue)
    end
end

print(string.format("\nTotal issues across %d files: %d", #files, #all_issues))
```

### Conditional Fix Generation

```lua
local Template = require("llmspell.template")

-- First pass: identify issues
local review = Template.execute("code-review", {
    code_path = "src/app.rs",
    language = "rust",
    output_format = "json"
})

local result = json.decode(review.output)

-- Second pass: generate fixes only if issues found
if result.total_issues > 0 then
    print(string.format("Found %d issues, generating fixes...", result.total_issues))

    local fixes = Template.execute("code-review", {
        code_path = "src/app.rs",
        language = "rust",
        generate_fixes = true,
        output_format = "markdown"
    })

    -- Save fixes to file
    local f = io.open("fixes.md", "w")
    f:write(fixes.output)
    f:close()
end
```

## Performance Considerations

### Execution Time

Review time scales with code size and number of aspects:

| Code Size | Aspects | Estimated Time* |
|-----------|---------|-----------------|
| <100 lines | 7 (all) | ~30 seconds |
| 100-500 lines | 7 (all) | ~2 minutes |
| 500-1000 lines | 7 (all) | ~4 minutes |
| <100 lines | 1-2 | ~10 seconds |

*Times vary based on LLM model speed and hardware

### Optimization Tips

1. **Filter Aspects**: Use only required aspects to reduce execution time
   ```bash
   --param aspects='["security"]'  # 7x faster than all aspects
   ```

2. **Severity Pre-filtering**: Review high-priority files first
   ```bash
   --param severity_filter=critical  # Focus on critical issues
   ```

3. **Parallel Reviews**: Review multiple files in parallel
   ```bash
   # Review files concurrently
   for file in src/*.rs; do
       llmspell template exec code-review --param code_path="$file" --param language=rust &
   done
   wait
   ```

4. **Local vs Cloud Models**: Use local models (Ollama) for faster iteration
   ```bash
   --param model=ollama/llama3.2:3b  # Faster than cloud models
   ```

## Troubleshooting

### Common Issues

**Issue**: "Failed to read code file"
```
Error: Failed to read code file '/path/to/file': No such file or directory
```
**Solution**: Verify file path is absolute or relative to working directory

---

**Issue**: "Invalid language parameter"
```
ValidationError: language must be one of: rust, python, javascript, typescript, go, java
```
**Solution**: Check language parameter matches supported values exactly

---

**Issue**: "Timeout during review"
```
Error: Agent execution failed: Timeout after 180 seconds
```
**Solution**:
- Reduce code size by reviewing smaller files
- Decrease number of aspects
- Increase timeout (requires code modification)

---

**Issue**: "Low quality review output"
```
Warning: Review agent returned incomplete analysis
```
**Solution**:
- Try different LLM model with `--param model=...`
- Increase temperature slightly: `--param temperature=0.3`
- Ensure code file is valid and parseable

### Getting Help

- **Documentation**: See `/docs/user-guide/templates/` for detailed guides
- **Issues**: Report bugs at https://github.com/lexlapax/rs-llmspell/issues
- **Examples**: Check `examples/templates/code-review/` for sample scripts

## Best Practices

1. **Start Small**: Review individual files before entire directories
2. **Severity First**: Filter by `critical` to address urgent issues
3. **Aspect Selection**: Choose 2-3 relevant aspects for faster reviews
4. **Iterate**: Run multiple passes with different aspects
5. **Automate**: Integrate into CI/CD for consistent quality gates
6. **Model Selection**: Use faster models for development, thorough models for production
7. **Fix Generation**: Enable only after identifying key issues to save time

## Advanced Configuration

### Custom Model Per Aspect

While not directly supported via parameters, you can implement this in Lua:

```lua
local aspects_config = {
    security = { model = "ollama/mistral:7b", temperature = 0.1 },
    quality = { model = "ollama/llama3.2:3b", temperature = 0.3 },
    performance = { model = "ollama/codellama:13b", temperature = 0.2 }
}

for aspect, config in pairs(aspects_config) do
    Template.execute("code-review", {
        code_path = "src/app.rs",
        language = "rust",
        aspects = {aspect},
        model = config.model,
        temperature = config.temperature
    })
end
```

### Environment-Specific Rules

```lua
local env = os.getenv("ENV") or "development"

local severity_map = {
    production = "critical",
    staging = "high",
    development = "medium"
}

Template.execute("code-review", {
    code_path = "src/",
    language = "python",
    severity_filter = severity_map[env] or "all"
})
```

## Future Enhancements

Planned features for upcoming versions:

- [ ] Directory scanning (recursive reviews)
- [ ] Multi-file analysis (cross-file dependency checking)
- [ ] Custom rule definitions (user-provided checklists)
- [ ] Interactive fix application (apply fixes automatically)
- [ ] IDE integration (VS Code, IntelliJ plugins)
- [ ] Historical trend analysis (track code quality over time)
- [ ] Team collaboration (shared review configurations)

## See Also

- [Code Generator Template](code-generator.md) - Generate code from specifications
- [Research Assistant Template](research-assistant.md) - Research best practices
- [Workflow Orchestrator Template](workflow-orchestrator.md) - Custom review pipelines
