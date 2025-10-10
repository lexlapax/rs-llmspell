# llmspell-workflows

Workflow orchestration patterns for Rs-LLMSpell.

## Features
- Built-in workflow types (Sequential, Parallel, Conditional, Loop, MapReduce, Pipeline)
- Deterministic execution with state management
- Error handling and retry policies

## Usage
```rust
use llmspell_workflows::{SequentialWorkflow, WorkflowBuilder};

let workflow = SequentialWorkflow::builder()
    .add_step(research_agent)
    .add_step(analysis_agent)
    .with_error_policy(RetryPolicy::exponential(3))
    .build()?;
```

## Agent Output Collection

All workflow types automatically collect agent outputs into the workflow result metadata:

```lua
local result = workflow:execute(input)

-- Access collected agent outputs
if result.metadata and result.metadata.extra then
    local outputs = result.metadata.extra.agent_outputs or {}

    for agent_id, output in pairs(outputs) do
        print(agent_id .. ": " .. tostring(output))
    end
end
```

**Workflow Types Supporting Agent Outputs**:
- ✅ Sequential workflows
- ✅ Parallel workflows
- ✅ Loop workflows
- ✅ Conditional workflows

**Output Structure**:
- Key: Agent ID (with timestamp suffix, e.g., `"requirements_analyst_1234567890"`)
- Value: JSON output from agent execution

**When Outputs Are Collected**:
- Only workflows with agent steps populate `agent_outputs`
- Workflows with only tool/workflow steps do not add this key
- Failed agents may still have outputs if partial execution occurred

## Dependencies
- `llmspell-core` - Core Workflow trait
- `llmspell-agents` - Agent implementations
- `llmspell-hooks` - Event emission during execution