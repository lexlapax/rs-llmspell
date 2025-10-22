# LLMSpell Template Examples

Comprehensive examples demonstrating the Template global API for discovering, introspecting, and executing built-in templates.

## Overview

The Template system provides a high-level abstraction for common AI workflows that combine agents, tools, RAG, and workflows into reusable patterns. Templates are parameterized, validated, and provide cost estimation.

## Built-in Templates (Phase 12)

LLMSpell ships with 6 production-ready templates:

1. **Research Assistant** (`research-assistant`) - Multi-source research with web search, RAG ingestion, AI synthesis, and citation validation
2. **Interactive Chat** (`interactive-chat`) - Session-based conversation with optional tool integration
3. **Data Analysis** (`data-analysis`) - Statistical analysis and visualization
4. **Code Generator** (`code-generator`) - Specification → Implementation → Testing workflow
5. **Document Processor** (`document-processor`) - PDF/OCR extraction and transformation
6. **Workflow Orchestrator** (`workflow-orchestrator`) - Custom agent/tool composition patterns

## Template Global API

### Discovery & Introspection

```lua
-- List all templates
local templates = Template.list()

-- List templates by category
local research_templates = Template.list("research")
local chat_templates = Template.list("chat")

-- Search templates by keyword
local results = Template.search("research")

-- Get template info (with or without schema)
local info = Template.info("research-assistant", false)  -- Basic info
local info_full = Template.info("research-assistant", true)  -- Include schema

-- Get parameter schema only
local schema = Template.schema("research-assistant")
```

### Execution

```lua
-- Execute template with parameters
local params = {
    topic = "Rust async programming",
    max_sources = 5,
    model = "ollama/llama3.2:3b",
    output_format = "markdown"
}

local output = Template.execute("research-assistant", params)

-- Inspect result
print(output.result)  -- Main result (text, structured, file, or multiple)
print(output.result_type)  -- "text", "structured", "file", or "multiple"

-- Check metrics
print(output.metrics.duration_ms)
print(output.metrics.agents_invoked)
print(output.metrics.tools_invoked)
print(output.metrics.rag_queries)

-- Access artifacts
for _, artifact in ipairs(output.artifacts) do
    print(artifact.path, artifact.mime_type)
end
```

### Cost Estimation

```lua
-- Estimate execution cost before running
local estimate = Template.estimate_cost("research-assistant", params)

if estimate then
    print(string.format("Estimated tokens: %d", estimate.estimated_tokens or 0))
    print(string.format("Estimated cost: $%.4f", estimate.estimated_cost_usd or 0))
    print(string.format("Estimated duration: %dms", estimate.estimated_duration_ms or 0))
    print(string.format("Confidence: %.2f", estimate.confidence))
end
```

## Running Examples

### Discovery Example

```bash
# List all templates and inspect schemas
llmspell lua examples/templates/discovery.lua
```

### Template Execution Examples

```bash
# Research Assistant (fully functional)
llmspell lua examples/templates/research/lua-basic.lua

# Interactive Chat (programmatic mode)
llmspell lua examples/templates/chat/lua-basic.lua

# Other templates (placeholder implementations)
llmspell lua examples/templates/analysis/lua-basic.lua
llmspell lua examples/templates/codegen/lua-basic.lua
llmspell lua examples/templates/documents/lua-basic.lua
llmspell lua examples/templates/orchestration/lua-basic.lua
```

## Template Categories

Templates are organized into categories for easy discovery:

- `TemplateCategory::Research` - Academic and professional research
- `TemplateCategory::Chat` - Conversational AI and dialogue
- `TemplateCategory::Analysis` - Data analysis and visualization
- `TemplateCategory::CodeGen` - Code generation and synthesis
- `TemplateCategory::Document` - Document processing and transformation
- `TemplateCategory::Workflow` - Custom workflow orchestration
- `TemplateCategory::Custom(name)` - User-defined categories

## Parameter Validation

Templates use ConfigSchema for parameter validation with rich constraints:

```lua
-- Example schema inspection
local schema = Template.schema("research-assistant")

for _, param in ipairs(schema.parameters) do
    print(param.name, param.param_type, param.required and "REQUIRED" or "optional")

    if param.constraints then
        -- Check numeric constraints
        if param.constraints.min then print("  min:", param.constraints.min) end
        if param.constraints.max then print("  max:", param.constraints.max) end

        -- Check string constraints
        if param.constraints.min_length then print("  min_length:", param.constraints.min_length) end
        if param.constraints.max_length then print("  max_length:", param.constraints.max_length) end
        if param.constraints.pattern then print("  pattern:", param.constraints.pattern) end

        -- Check enum constraints
        if param.constraints.allowed_values then
            print("  allowed_values:", table.concat(param.constraints.allowed_values, ", "))
        end
    end
end
```

## Error Handling

Always use pcall for template execution to handle errors gracefully:

```lua
local success, output = pcall(function()
    return Template.execute("research-assistant", params)
end)

if not success then
    print(string.format("ERROR: Template execution failed: %s", output))
    os.exit(1)
end

-- Process output
print(output.result)
```

## Template Output Structure

TemplateOutput contains:

```lua
{
    result_type = "text" | "structured" | "file" | "multiple",
    result = <content based on result_type>,
    metadata = {
        template_id = "research-assistant",
        template_version = "0.1.0",
        parameters = { ... }  -- Original parameters
    },
    artifacts = [
        { path = "...", content = "...", mime_type = "..." },
        ...
    ],
    metrics = {
        duration_ms = 1234,
        agents_invoked = 2,
        tools_invoked = 5,
        rag_queries = 1,
        custom = { ... }  -- Template-specific metrics
    }
}
```

## Implementation Status (Phase 12.5)

- ✅ **Template System**: Core traits, registry, parameter validation
- ✅ **Research Assistant**: Fully functional 4-phase workflow (Phase 12.3)
- ✅ **Interactive Chat**: Fully functional session-based chat (Phase 12.4.1)
- ⚠️  **Data Analysis**: Placeholder implementation (Phase 12.4.2)
- ⚠️  **Code Generator**: Placeholder implementation (Phase 12.4.3)
- ⚠️  **Document Processor**: Placeholder implementation (Phase 12.4.4)
- ⚠️  **Workflow Orchestrator**: Placeholder implementation (Phase 12.4.4)
- ✅ **Template Global API**: Lua bindings complete

## Next Steps

- **Phase 12.4.2-12.4.4**: Complete remaining template implementations
- **Phase 12.6**: Comprehensive testing and quality assurance
- **Phase 13**: Adaptive Memory System (A-TKG integration)
- **Phase 14+**: Additional templates and advanced features

## Documentation

- **Template Trait**: `llmspell-templates/src/core.rs`
- **Built-in Templates**: `llmspell-templates/src/builtin/`
- **Lua API Implementation**: `llmspell-bridge/src/lua/globals/template.rs`
- **Parameter Validation**: `llmspell-templates/src/validation.rs`

## Contributing

When adding new templates:

1. Implement the `Template` trait from `llmspell-templates`
2. Define `TemplateMetadata` (id, name, description, category, version, requires, tags)
3. Implement `config_schema()` with full parameter validation
4. Implement `execute()` with proper error handling
5. Implement `estimate_cost()` for cost transparency
6. Register in `builtin::register_builtin_templates()`
7. Add example to `examples/templates/<category>/lua-basic.lua`
8. Update this README with new template info

## License

LLMSpell Template System - Part of the LLMSpell Project
