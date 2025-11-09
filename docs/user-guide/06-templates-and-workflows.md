# Templates & Workflows

**Pre-built AI workflow templates for rapid experimentation**

🔗 **Navigation**: [← User Guide](README.md) | [Lua Scripting](04-lua-scripting.md) | [CLI Reference](05-cli-reference.md)

---

## Overview

llmspell provides 10 production-ready AI workflow templates for common tasks. Templates are pre-configured multi-agent workflows that you can execute via CLI or Lua.

**What You'll Learn:**
- Available templates and when to use them
- CLI and Lua execution
- Template customization basics

**For Template Development:**
See [Developer Guide - Template Creation](../developer-guide/template-creation.md)

---

## Available Templates (10 Total)

### Research & Analysis (3 templates)

**1. research-assistant**
- Multi-step research with source gathering
- Automatic citation and fact-checking
- Best for: Deep research tasks

**2. data-analysis**
- Statistical analysis with visualizations
- Pattern detection and insights
- Best for: Analyzing datasets

**3. knowledge-management**
- Information extraction and organization
- Graph-based knowledge structuring
- Best for: Building knowledge bases

### Development (2 templates)

**4. code-generator**
- Function/class generation from description
- Multiple language support
- Best for: Rapid prototyping

**5. code-review**
- Automated code review with best practices
- Security and performance analysis
- Best for: Code quality checks

### Content (3 templates)

**6. content-generation**
- Blog posts, articles, documentation
- SEO optimization and formatting
- Best for: Content creation

**7. interactive-chat**
- Conversational AI with memory
- Context-aware responses
- Best for: Chatbots and assistants

**8. document-processor**
- Document parsing and transformation
- Multi-format support (PDF, Markdown, HTML)
- Best for: Document workflows

### Advanced (2 templates)

**9. file-classification**
- Automated file categorization
- Custom taxonomy support
- Best for: Organizing large filesets

**10. workflow-orchestrator**
- Complex multi-agent orchestration
- Conditional branching and loops
- Best for: Advanced automation

---

## Quick Start: CLI Usage

### List Templates

```bash
# Show all available templates
llmspell template list

# Search for templates
llmspell template search "code"

# Get detailed info
llmspell template info code-generator
```

### Execute Template

```bash
# Basic execution
llmspell template exec code-generator \
    --param description="A function to validate email addresses" \
    --param language="rust"

# With all parameters
llmspell template exec research-assistant \
    --param topic="Rust async programming" \
    --param depth="comprehensive" \
    --param sources="5" \
    --param model="openai/gpt-4o" \
    --output json > research.json
```

### View Parameter Schema

```bash
# See required and optional parameters
llmspell template schema code-generator

# Output shows:
# Required:
#   - description (string): What to generate
#   - language (string): Programming language
# Optional:
#   - model (string): LLM to use (default: openai/gpt-4o-mini)
#   - style (string): Code style (default: idiomatic)
```

---

## Lua API Usage

### Basic Execution

```lua
-- List all templates
local templates = Template.list()
for _, t in ipairs(templates) do
    print(t.name, "-", t.description)
end

-- Execute template
local result = Template.execute("code-generator", {
    description = "A function to validate email addresses",
    language = "rust",
    model = "openai/gpt-4o-mini"
})

print(result.content)
```

### With Cost Estimation

```lua
-- Estimate cost before execution
local cost = Template.estimate_cost("research-assistant", {
    topic = "Climate change impacts",
    depth = "comprehensive"
})

print(string.format("Estimated cost: $%.4f", cost))

-- Execute if cost is acceptable
if cost < 1.0 then
    local result = Template.execute("research-assistant", {
        topic = "Climate change impacts",
        depth = "comprehensive"
    })
end
```

### Search Templates

```lua
-- Search by keyword
local results = Template.search("code", {
    category = "codegen"
})

for _, template in ipairs(results) do
    print(template.name)
end

-- Get template info
local info = Template.info("code-generator")
print("Category:", info.category)
print("Required params:", table.concat(info.required_params, ", "))
```

---

## Template Examples

### Example 1: Code Generation

```bash
llmspell template exec code-generator \
    --param description="A REST API client for GitHub" \
    --param language="rust" \
    --param style="async" \
    --output text
```

**Output**: Complete Rust code with error handling, async/await, and documentation.

### Example 2: Research

```bash
llmspell template exec research-assistant \
    --param topic="Quantum computing applications" \
    --param depth="comprehensive" \
    --param sources="10" \
    --output markdown > research_report.md
```

**Output**: Structured research report with sources, citations, and key findings.

### Example 3: Document Processing

```bash
llmspell template exec document-processor \
    --param input="contract.pdf" \
    --param operation="extract_key_terms" \
    --param format="json"
```

**Output**: JSON with extracted terms, definitions, and document structure.

### Example 4: Code Review

```lua
local review = Template.execute("code-review", {
    code = [[
        fn process_data(data: Vec<String>) -> Result<(), Error> {
            for item in data {
                println!("{}", item);
            }
            Ok(())
        }
    ]],
    language = "rust",
    focus = "performance"
})

print("Review:", review.content)
print("Issues found:", #review.issues)
```

---

## Customization Basics

### Override Default Model

```bash
# Use different model
llmspell template exec code-generator \
    --param description="..." \
    --param language="rust" \
    --param model="anthropic/claude-3-5-sonnet"
```

### Adjust Output Format

```bash
# JSON output
llmspell template exec research-assistant \
    --param topic="..." \
    --output json

# Markdown output
llmspell template exec content-generation \
    --param topic="..." \
    --output markdown

# Plain text (default)
llmspell template exec code-generator \
    --param description="..." \
    --output text
```

---

## Template Categories

| Category | Templates | Use Case |
|----------|-----------|----------|
| **Research** | research-assistant, knowledge-management | Information gathering |
| **Chat** | interactive-chat | Conversational interfaces |
| **Analysis** | data-analysis | Data insights |
| **CodeGen** | code-generator, code-review | Software development |
| **Document** | document-processor, content-generation | Document workflows |
| **Workflow** | workflow-orchestrator, file-classification | Automation |

---

## Performance

| Metric | Target | Actual |
|--------|--------|--------|
| Template list | <10ms | 0.5ms (20x faster) |
| Template execute overhead | <100ms | 2ms (50x faster) |
| Parameter validation | <5ms | 0.1ms (50x faster) |

---

## Advanced Topics

### Custom Templates

Create your own templates - see [Template Creation Guide](../developer-guide/template-creation.md) for:
- Template trait implementation
- Parameter schema design
- Multi-agent orchestration patterns
- Registry integration

### Template Composition

```lua
-- Chain templates
local code = Template.execute("code-generator", { ... })
local review = Template.execute("code-review", {
    code = code.content,
    language = "rust"
})
```

### Integration with Workflows

```lua
-- Use templates in workflows
local workflow = Workflow.new({
    type = "sequential",
    steps = {
        {
            name = "generate",
            template = "code-generator",
            params = { description = "...", language = "rust" }
        },
        {
            name = "review",
            template = "code-review",
            params = { code = "{{generate.content}}", language = "rust" }
        }
    }
})

local result = workflow:execute({})
```

---

## Troubleshooting

**Template not found:**
```bash
# Verify template exists
llmspell template list | grep my-template

# Check spelling
llmspell template search "my"
```

**Missing required parameter:**
```bash
# Check schema
llmspell template schema my-template

# Execute with all required params
llmspell template exec my-template \
    --param param1="value1" \
    --param param2="value2"
```

**Cost concerns:**
```lua
-- Estimate before executing
local cost = Template.estimate_cost("template-name", params)
if cost > max_cost then
    print("Too expensive!")
else
    Template.execute("template-name", params)
end
```

---

## Next Steps

- **Create Custom Templates**: [Template Creation Guide](../developer-guide/template-creation.md)
- **Learn Workflows**: See Workflow sections in [Core Concepts](02-core-concepts.md)
- **Explore Examples**: `examples/script-users/templates/` for working examples
- **CLI Reference**: [CLI Guide](05-cli-reference.md) for all template commands

---

**Version**: 0.13.0 | **Phase**: 13b.18.3 | **Last Updated**: 2025-11-08
