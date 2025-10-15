# Workflow Orchestrator Template

**Version:** 0.1.0
**Category:** Workflow
**Status:** Placeholder Implementation (Phase 12.4.4)

## Overview

The Workflow Orchestrator template enables creation of custom multi-step workflows by composing agents, tools, and templates. It provides a high-level orchestration layer for complex business logic and automation scenarios.

### What It Does

- **Workflow Definition**: Define multi-step workflows declaratively
- **Agent Orchestration**: Coordinate multiple AI agents
- **Tool Integration**: Seamlessly integrate tools into workflows
- **Conditional Logic**: Branch and loop based on results
- **Error Handling**: Robust error recovery and retry logic
- **Workflow Templates**: Reusable workflow patterns

### Use Cases

- Multi-step business processes
- Data pipeline orchestration
- Complex automation scenarios
- Integration workflows
- Custom template creation

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec workflow-orchestrator \
  --param workflow_definition='{"steps":[...]}' \
  --param input_data='{"key":"value"}'
```

### Lua - Basic Usage

```lua
local result = Template.execute("workflow-orchestrator", {
    workflow_definition = {
        steps = {
            {type = "agent", name = "analyzer", input = "data"},
            {type = "tool", name = "web-search", input = "query"},
            {type = "template", name = "code-generator", input = "spec"}
        }
    },
    input_data = {topic = "AI workflows"}
})

print(result.result)
```

---

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `workflow_definition` | Object | Workflow structure with steps and connections |
| `input_data` | Object | Initial input data for workflow |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `max_steps` | Integer | `50` | Maximum workflow steps to prevent infinite loops |
| `timeout_seconds` | Integer | `300` | Workflow execution timeout |
| `error_handling` | Enum | `"stop"` | Error handling: `stop`, `continue`, `retry` |
| `model` | String | `"ollama/llama3.2:3b"` | Default LLM for agents |

**Inspect Full Schema:**
```bash
llmspell template schema workflow-orchestrator
```

---

## Implementation Status

⚠️ **Note**: This template is a **placeholder implementation** as of Phase 12.4.4.

**Implemented:**
- ✅ Template metadata and parameter schema
- ✅ Parameter validation
- ✅ Cost estimation
- ✅ 13 comprehensive unit tests

**Placeholder/Pending:**
- ⏳ Workflow execution engine
- ⏳ Agent orchestration
- ⏳ Conditional branching
- ⏳ Error recovery logic
- ⏳ Workflow state management

**Expected**: Full implementation in Phase 15 (Advanced Workflows)

---

## Workflow Definition Format

```json
{
  "workflow_definition": {
    "name": "Research and Generate",
    "description": "Research topic and generate code",
    "steps": [
      {
        "id": "step1",
        "type": "template",
        "template": "research-assistant",
        "input": {
          "topic": "{{input_data.topic}}",
          "max_sources": 10
        },
        "output": "research_result"
      },
      {
        "id": "step2",
        "type": "template",
        "template": "code-generator",
        "input": {
          "description": "{{step1.research_result.result}}",
          "language": "rust"
        },
        "output": "code_result"
      },
      {
        "id": "step3",
        "type": "agent",
        "agent": "code-reviewer",
        "input": {
          "code": "{{step2.code_result.result}}"
        },
        "output": "review_result"
      }
    ],
    "output": {
      "code": "{{step2.code_result}}",
      "review": "{{step3.review_result}}"
    }
  },
  "input_data": {
    "topic": "Binary search tree implementation"
  }
}
```

---

## Examples

### CLI Examples

#### Simple Sequential Workflow
```bash
llmspell template exec workflow-orchestrator \
  --param workflow_definition='{"steps":[{"type":"agent","name":"analyzer"}]}' \
  --param input_data='{"text":"Analyze this data"}' \
  --output-dir ./workflow_results
```

#### Research + Code Generation Workflow
```bash
llmspell template exec workflow-orchestrator \
  --param workflow_definition='{"steps":[
    {"type":"template","template":"research-assistant","input":{"topic":"{{input.topic}}"}},
    {"type":"template","template":"code-generator","input":{"description":"{{step1.result}}"}}
  ]}' \
  --param input_data='{"topic":"REST API design"}' \
  --param max_steps=10
```

### Lua Examples

#### Multi-Template Workflow
```lua
local workflow = {
    workflow_definition = {
        name = "Research, Generate, Test",
        steps = {
            {
                id = "research",
                type = "template",
                template = "research-assistant",
                input = {topic = "{{input.topic}}"},
                output = "research_data"
            },
            {
                id = "generate",
                type = "template",
                template = "code-generator",
                input = {
                    description = "{{research.research_data.result}}",
                    language = "rust",
                    include_tests = true
                },
                output = "code_data"
            }
        }
    },
    input_data = {
        topic = "Async HTTP client with retry logic"
    }
}

local result = Template.execute("workflow-orchestrator", workflow)

print("Workflow completed:")
print("Research tokens: " .. (result.metrics.tokens_used or "N/A"))
print("Duration: " .. result.metrics.duration_ms .. "ms")
```

#### Conditional Workflow
```lua
local workflow = {
    workflow_definition = {
        steps = {
            {
                id = "analyze",
                type = "agent",
                agent = "analyzer",
                input = {text = "{{input.text}}"},
                output = "analysis"
            },
            {
                id = "decide",
                type = "conditional",
                condition = "{{analysis.sentiment}} == 'positive'",
                then_step = "positive_action",
                else_step = "negative_action"
            },
            {
                id = "positive_action",
                type = "template",
                template = "code-generator",
                input = {description = "Generate success handler"}
            },
            {
                id = "negative_action",
                type = "template",
                template = "code-generator",
                input = {description = "Generate error handler"}
            }
        }
    },
    input_data = {
        text = "This is great!"
    }
}

local result = Template.execute("workflow-orchestrator", workflow)
```

---

## Cost Estimation

Workflow costs are the sum of all step costs:

```bash
llmspell template info workflow-orchestrator --show-schema
```

### Example Workflow Costs

| Workflow | Steps | Estimated Tokens | Duration | Cost (USD) |
|----------|-------|-----------------|----------|------------|
| Simple (2 templates) | 2 | ~5,000 | ~15s | $0.00050 |
| Medium (4 steps) | 4 | ~12,000 | ~40s | $0.00120 |
| Complex (10+ steps) | 10+ | ~30,000+ | ~120s+ | $0.00300+ |

**Note**: Actual costs depend on step types, parameters, and data size.

---

## Workflow Patterns

### Sequential Pattern
Execute steps in order, passing outputs forward:
```
Step 1 → Step 2 → Step 3 → Final Output
```

### Parallel Pattern
Execute independent steps concurrently:
```
           ┌─ Step 2a ─┐
Step 1 ─ ──│─ Step 2b ─│─→ Step 3
           └─ Step 2c ─┘
```

### Conditional Pattern
Branch based on intermediate results:
```
                ┌─ Branch A ─┐
Step 1 → Check ─┤            ├─→ Final
                └─ Branch B ─┘
```

### Loop Pattern
Repeat steps until condition met:
```
Step 1 → Step 2 → Check → [Back to Step 2] → Final
```

---

## Troubleshooting

### Error: "Workflow exceeds max_steps limit"

**Cause**: Workflow has too many steps or infinite loop

**Solution**: Increase `max_steps` or fix loop condition:
```bash
--param max_steps=100
```

### Error: "Workflow timeout exceeded"

**Cause**: Workflow took longer than `timeout_seconds`

**Solution**: Increase timeout or optimize workflow:
```bash
--param timeout_seconds=600  # 10 minutes
```

### Using Placeholder Implementation

**Current Behavior**: The template validates workflow definitions but doesn't execute full orchestration.

**Workaround**: For production workflows:
1. Use llmspell-workflows crate directly
2. Manually orchestrate agents/tools/templates
3. Wait for Phase 15 full implementation

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Workflow System Guide](../../workflows/README.md)
- [Agent Orchestration](../../agents/orchestration.md)
- [Research Assistant Template](./research-assistant.md) (production example)

---

## Roadmap

### Phase 15 (Planned)
- Complete workflow execution engine
- Conditional branching and loops
- Parallel step execution
- Error recovery and retry logic
- Workflow state persistence
- Debugging and monitoring tools

### Phase 16 (Future)
- Visual workflow builder
- Workflow marketplace
- Collaborative workflows
- Real-time monitoring dashboard
- Performance optimization

---

**Last Updated**: Phase 12.4.4 (Placeholder Implementation)
**Next Review**: Phase 15 (Advanced Workflows)
