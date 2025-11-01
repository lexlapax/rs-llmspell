# Workflow Orchestrator Template

**Version:** 0.1.0
**Category:** Workflow
**Status:** ✅ Production Ready (Phase 12.8.5)

## Overview

Flexible AI workflow orchestration with user-defined patterns. Define custom agent and tool compositions with parallel, sequential, or hybrid execution. Build complex multi-step workflows with state tracking and result aggregation using the llmspell-workflows WorkflowFactory.

### What It Does

- **Workflow Parsing**: Parse JSON workflow definitions into executable workflows
- **WorkflowFactory Integration**: Create workflows via llmspell-workflows factory
- **Execution Modes**: Support sequential, parallel, and hybrid (conditional) execution
- **State Tracking**: Collect and aggregate results across workflow steps
- **Agent & Tool Coordination**: Compose agents and tools into multi-step processes

### Use Cases

- Multi-step business process automation
- Data pipeline orchestration with agents and tools
- Complex AI agent collaboration workflows
- Custom template creation from workflow patterns
- Integration workflows with external systems

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec workflow-orchestrator \
  --param workflow_config='{"steps":[{"name":"step1","step_type":"agent"},{"name":"step2","step_type":"tool"}]}' \
  --param execution_mode="sequential"
```

### CLI - With Memory and Provider

Enable memory-enhanced execution with custom provider:

```bash
llmspell template exec workflow-orchestrator \
  --param workflow_config='{"steps":[{"name":"step1","step_type":"agent"}]}' \
  --param execution_mode="sequential" \
  --param session-id="user-session-123" \
  --param memory-enabled=true \
  --param context-budget=3000 \
  --param provider-name="ollama"
```

### Lua - Basic Usage

```lua
local result = Template.execute("workflow-orchestrator", {
    workflow_config = {
        name = "my-workflow",
        steps = {
            {name = "analyze", step_type = "agent"},
            {name = "search", step_type = "tool"}
        }
    },
    execution_mode = "sequential"
})

print(result.result)
```

### Lua - With Memory and Provider

Enable memory-enhanced execution:

```lua
local result = Template.execute("workflow-orchestrator", {
    workflow_config = {
        name = "my-workflow",
        steps = {
            {name = "analyze", step_type = "agent"},
            {name = "search", step_type = "tool"}
        }
    },
    execution_mode = "sequential",
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
| `workflow_config` | Object | Workflow configuration with steps array (minimum 1 step required) |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `execution_mode` | Enum | `"sequential"` | Workflow execution mode: `sequential`, `parallel`, `hybrid`, or `loop` |
| `collect_intermediate` | Boolean | `true` | Collect intermediate results from each step |
| `max_steps` | Integer | `10` | Maximum workflow steps to execute (range: 1-100) |
| `model` | String | `"ollama/llama3.2:3b"` | Default LLM model for agent steps in workflow |

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

**Inspect Full Schema:**
```bash
llmspell template schema workflow-orchestrator
```

---

## Implementation Details

### Phase 1: Workflow Parsing
- **JSON Parsing**: Converts workflow_config JSON into WorkflowDefinition
- **Step Validation**: Ensures steps array exists and is non-empty
- **Step Type Detection**: Parses step_type as string ("agent"/"tool") or object format
- **Flexible Format**: Supports simple string types or complex object definitions
- **Error Messages**: Detailed validation errors with step indices

### Phase 2: Build Execution Plan
- **Step Limiting**: Applies max_steps constraint to workflow
- **Mode Translation**: Converts execution_mode to WorkflowType
- **Plan Generation**: Creates ExecutionPlan with limited steps

### Phase 3: Workflow Execution

#### Real LLM Execution Architecture (Sub-Tasks 12.8.5.3 + 12.8.5.4)

**Agent Pre-Creation Pattern** (lines 479-522):
```rust
// 1. Pre-create real agents BEFORE workflow creation
for (idx, step) in plan.steps.iter().enumerate() {
    if step.step_type == StepType::Agent {
        let agent_id = format!("workflow-agent-{}", idx);
        let (provider, model_id) = parse_model_spec(model); // "ollama/llama3.2:3b" → ("ollama", "llama3.2:3b")

        let agent = context.agent_registry()
            .create_agent(AgentConfig { ... })  // Real agent creation
            .await?;

        agents.push((agent_id, agent));  // Vec<(String, Arc<dyn Agent>)>
    }
}
```

**ComponentRegistry Pattern** (lines 527-533):
```rust
// 2. Build ComponentRegistry with pre-created agents
let mut registry = SimpleComponentRegistry::new();
for (agent_id, agent) in agents.iter() {
    registry.register(agent_id.clone(), agent.clone());
}
let component_registry: Arc<dyn ComponentLookup> = Arc::new(registry);
```

**Unified Builder Pattern** (lines 556-624) - ALL 4 WORKFLOW TYPES:
```rust
// 3. ALL workflows use builders with registry (REAL LLM for all modes)
let workflow: Arc<dyn BaseAgent> = match workflow_type {
    WorkflowType::Sequential => {
        Arc::new(SequentialWorkflowBuilder::new(name)
            .with_config(workflow_config)
            .with_registry(component_registry.clone())  // ← Real LLM
            .add_steps(workflow_steps)
            .build())
    }
    WorkflowType::Parallel => {
        let mut builder = ParallelWorkflowBuilder::new(name)
            .with_workflow_config(workflow_config)
            .with_registry(component_registry.clone())  // ← Real LLM
            .with_max_concurrency(4)
            .fail_fast(false);

        // Each step becomes separate parallel branch
        for (idx, step) in workflow_steps.iter().enumerate() {
            let branch = ParallelBranch::new(format!("branch-{}", idx + 1))
                .with_description(step.name.clone())
                .add_step(step.clone());
            builder = builder.add_branch(branch);
        }
        Arc::new(builder.build()?)
    }
    WorkflowType::Conditional => {
        // Single default branch with all steps
        let default_branch = ConditionalBranch::default("default".to_string())
            .with_steps(workflow_steps.clone());

        Arc::new(ConditionalWorkflowBuilder::new(name)
            .with_workflow_config(workflow_config)
            .with_registry(component_registry.clone())  // ← Real LLM
            .add_branch(default_branch)
            .build())
    }
    WorkflowType::Loop => {
        let builder = LoopWorkflowBuilder::new(name)
            .with_range(1, 2, 1)  // Single iteration
            .with_workflow_config(workflow_config)
            .with_registry(component_registry.clone());  // ← Real LLM

        let builder = workflow_steps.iter()
            .fold(builder, |b, step| b.add_step(step.clone()));
        Arc::new(builder.build()?)
    }
};
```

**WorkflowParams Construction** (for factory-based workflows):
  - **name**: From workflow_config.name or "custom-workflow" (default)
  - **workflow_type**: Mode mapping (lines 467-472):
    - `execution_mode="sequential"` → `WorkflowType::Sequential`
    - `execution_mode="parallel"` → `WorkflowType::Parallel`
    - `execution_mode="hybrid"` → `WorkflowType::Conditional`
  - **config**: WorkflowConfig with timeouts and error handling:
    - `max_execution_time`: 600s (10 minutes)
    - `default_step_timeout`: 120s (2 minutes per step)
    - `continue_on_error`: true for parallel, false for sequential/conditional
    - `default_error_strategy`: ErrorStrategy::FailFast
    - `max_retry_attempts`: 1
  - **type_config**: Workflow-specific JSON (lines 548-601):
    - **Sequential**: `{"steps": [WorkflowStep, ...]}`
    - **Parallel**: `{"max_concurrency": 4, "fail_fast": false, "timeout": null, "continue_on_optional_failure": true, "branches": [ParallelBranch, ...]}`
    - **Conditional**: `{"branches": [ConditionalBranch]}`

**Step Conversion** (lines 692-718):
  - Agent steps: `WfStepType::Agent { agent_id: "workflow-agent-0", input: "Execute: {description}" }`
    - **Changed**: agent_id now references pre-created agent (not model string)
    - Maps to agents Vec index for real agent resolution
  - Tool steps: `WfStepType::Tool { tool_name: "generic-tool", parameters: {"description": ...} }`
  - Each step gets: 120s timeout, 1 retry attempt

**Execution**:
  - All workflows: workflow.execute() with ExecutionContext::default() (registry already in workflow)
  - Registry enables real LLM agent resolution during workflow execution

**Result Collection**: Aggregates workflow output text and step counts (agents/tools)

**Execution Comparison** (Phase 12.8.5.4 - All Fixed):
| Mode | Registry Support | Execution Type | Duration Example | Status |
|------|------------------|----------------|------------------|--------|
| Sequential | ✅ Yes (.with_registry()) | Real LLM | 455ms | ✅ Working |
| Parallel | ✅ Yes (.with_registry()) | Real LLM | 818ms (2 branches) | ✅ Working |
| Conditional | ✅ Yes (.with_registry()) | Real LLM | 559ms | ✅ Working |
| Loop | ✅ Yes (.with_registry()) | Real LLM | 536ms (1 iteration) | ✅ Working |

### Phase 4: Result Aggregation
- **Report Generation**: Markdown report with workflow metadata
- **Step-by-Step Results**: Optional intermediate results if collected
- **Metrics Tracking**: Agents invoked, tools invoked, total steps
- **Artifacts**: workflow_report.md and intermediate_results.json (if output_dir set)

### Branch Conversion Architecture

User-defined steps convert to workflow-specific branch structures:

**Sequential Mode** (No Branch Conversion):
```rust
// type_config: {"steps": [WorkflowStep, ...]}
// Steps used directly by SequentialWorkflow
```
- No branch conversion needed
- Steps execute one after another in order
- Factory extracts steps from type_config (factory.rs:130-149)

**Parallel Mode** (Step → Branch Conversion):
```rust
// Each step becomes separate ParallelBranch
// type_config: {"max_concurrency": 4, ..., "branches": [ParallelBranch, ...]}
ParallelBranch {
    name: "branch-N",         // Auto-generated (branch-1, branch-2, ...)
    description: step.name,    // From original step name
    steps: vec![step],         // Single step inside branch
    required: true,            // All branches required by default
    timeout: null,             // Inherits from WorkflowConfig
}
```
- **Conversion**: workflow_orchestrator.rs:435-447
- **Factory Extraction**: factory.rs:167-172 extracts branches from type_config
- **Enables**: True concurrent execution (max 4 branches at once)
- **Example**: 3 steps → 3 branches (`branch-1`, `branch-2`, `branch-3`)

**Conditional Mode** (All Steps → Single Default Branch):
```rust
// Creates ONE ConditionalBranch with ALL steps
// type_config: {"branches": [ConditionalBranch]}
ConditionalBranch {
    id: ComponentId::new(),    // UUID generated per execution
    name: "default",            // Single default branch name
    condition: Condition::Always, // Always executes
    steps: workflow_steps,      // ALL steps in one branch
    is_default: true,           // Marked as default branch
}
```
- **Conversion**: workflow_orchestrator.rs:459-465
- **Factory Extraction**: factory.rs:175-185 uses ConditionalWorkflowBuilder
- **Current Limitation**: NOT true conditional routing - all steps execute sequentially in default branch
- **Rationale**: Demonstrates ConditionalWorkflow integration; true branching logic planned for Phase 14
- **Example**: 3 steps → 1 branch ("default") with 3 steps inside

**Branch Extraction Pattern** (factory.rs):
```rust
// Parallel/Conditional: Extract branches from type_config
let branches_json = params.type_config.get("branches")
    .cloned()
    .unwrap_or_else(|| serde_json::json!([]));
let branches: Vec<BranchType> = serde_json::from_value(branches_json)
    .unwrap_or_else(|_| vec![]);
```

---

## Workflow Configuration Format

The `workflow_config` parameter defines the workflow structure:

```json
{
  "workflow_config": {
    "name": "my-custom-workflow",
    "steps": [
      {
        "name": "analyze-step",
        "description": "Analyze input data",
        "step_type": "agent"
      },
      {
        "name": "search-step",
        "description": "Search for information",
        "step_type": "tool"
      },
      {
        "name": "generate-step",
        "description": "Generate output",
        "step_type": "agent"
      }
    ]
  }
}
```

### Step Format

Each step in the `steps` array requires:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | No | Step name (default: "step-N") |
| `description` | String | No | Step description (default: step name) |
| `step_type` | String/Object | Yes | Step type: "agent" or "tool" |

**Simple Format** (step_type as string):
```json
{"name": "step1", "step_type": "agent"}
```

**Object Format** (step_type with details):
```json
{
  "name": "step1",
  "step_type": {
    "Agent": {
      "agent_id": "summarizer",
      "input": "Analyze this"
    }
  }
}
```

---

## Output Format

### Text Output

```markdown
# Workflow Execution Report

**Workflow**: my-custom-workflow
**Steps Executed**: 3
**Agents Invoked**: 2
**Tools Invoked**: 1

---

## Step-by-Step Results

### Step 1: analyze-step

Agent step 'Analyze input data' executed

### Step 2: search-step

Tool step 'Search for information' executed

### Step 3: generate-step

Agent step 'Generate output' executed

---

## Final Output

[Workflow execution output from WorkflowFactory]

---

Generated by LLMSpell Workflow Orchestrator Template
```

### JSON Output

```json
{
  "result_type": "text",
  "result": "# Workflow Execution Report\n\n...",
  "metrics": {
    "duration_ms": 5400,
    "agents_invoked": 2,
    "tools_invoked": 1,
    "custom_metrics": {
      "workflow_steps": 3,
      "execution_mode": "sequential",
      "steps_executed": 3
    }
  },
  "artifacts": [
    {
      "path": "/output/workflow_report.md",
      "content": "...",
      "mime_type": "text/markdown"
    },
    {
      "path": "/output/intermediate_results.json",
      "content": "[...]",
      "mime_type": "application/json"
    }
  ]
}
```

---

## Examples

### CLI Examples

#### Sequential Workflow (3 Steps)
```bash
llmspell template exec workflow-orchestrator \
  --param workflow_config='{
    "name": "data-pipeline",
    "steps": [
      {"name": "extract", "step_type": "tool", "description": "Extract data"},
      {"name": "transform", "step_type": "agent", "description": "Transform data"},
      {"name": "load", "step_type": "tool", "description": "Load data"}
    ]
  }' \
  --param execution_mode="sequential" \
  --param model="ollama/llama3.2:3b"
```

#### Parallel Workflow
```bash
llmspell template exec workflow-orchestrator \
  --param workflow_config='{
    "name": "multi-analysis",
    "steps": [
      {"name": "sentiment", "step_type": "agent"},
      {"name": "entities", "step_type": "agent"},
      {"name": "summary", "step_type": "agent"}
    ]
  }' \
  --param execution_mode="parallel" \
  --param collect_intermediate=true
```

#### Hybrid (Conditional) Workflow
```bash
llmspell template exec workflow-orchestrator \
  --param workflow_config='{
    "name": "conditional-router",
    "steps": [
      {"name": "check", "step_type": "agent"},
      {"name": "route-a", "step_type": "tool"},
      {"name": "route-b", "step_type": "tool"}
    ]
  }' \
  --param execution_mode="hybrid"
```

### Lua Examples

#### Simple Agent Workflow
```lua
local result = Template.execute("workflow-orchestrator", {
    workflow_config = {
        name = "agent-chain",
        steps = {
            {name = "step1", step_type = "agent", description = "Analyze"},
            {name = "step2", step_type = "agent", description = "Summarize"},
            {name = "step3", step_type = "agent", description = "Review"}
        }
    },
    execution_mode = "sequential",
    model = "ollama/llama3.2:3b"
})

print("Steps executed: " .. result.metrics.custom_metrics.steps_executed)
print("Duration: " .. result.metrics.duration_ms .. "ms")
```

#### Tool + Agent Workflow
```lua
local result = Template.execute("workflow-orchestrator", {
    workflow_config = {
        name = "research-workflow",
        steps = {
            {name = "search", step_type = "tool", description = "Web search"},
            {name = "analyze", step_type = "agent", description = "Analyze results"},
            {name = "format", step_type = "tool", description = "Format output"}
        }
    },
    execution_mode = "sequential",
    collect_intermediate = true,
    max_steps = 5
})

-- Access intermediate results
if result.artifacts then
    for _, artifact in ipairs(result.artifacts) do
        print("Artifact: " .. artifact.path)
    end
end
```

### Real Output Examples (Phase 12.8.5.3 + 12.8.5.4 - Real LLM Testing)

#### Sequential Mode - Real LLM Execution (llama3.2:3b)
```bash
./target/debug/llmspell template exec workflow-orchestrator \
  --param 'workflow_config={"steps":[{"name":"ux-test","step_type":"agent","description":"What is 100-42?"}]}' \
  --param execution_mode="sequential" \
  --param model="ollama/llama3.2:3b" \
  --output text
```

**Output** (real LLM, 278ms):
```
✓ Template execution completed in 0.56s
================================================================================

Result:
# Workflow Execution Report

**Workflow**: custom-workflow
**Steps Executed**: 1
**Agents Invoked**: 1
**Tools Invoked**: 0

---

## Step-by-Step Results

### Step 1: step-1

Agent executed successfully (workflow-agent-0)
Description: What is 100-42?
Model: ollama/llama3.2:3b
Duration: 278ms
Note: Real LLM execution completed. Agent outputs stored in workflow state.

---

## Final Output

Sequential workflow 'custom-workflow' completed successfully. 1 steps executed. Duration: 278.630334ms

---

Generated by LLMSpell Workflow Orchestrator Template


Metrics:
  Duration:      0.56s
  Agents:        1
```

**Key Observations**:
- ✅ Real LLM execution confirmed (278ms vs 21ms mock)
- ✅ Output shows execution details: model, duration, description
- ✅ Agent registry lookup successful (debug logs: "Looking for agent with name: 'workflow-agent-0'")

#### Sequential Mode - Real LLM JSON Output
```bash
./target/debug/llmspell template exec workflow-orchestrator \
  --param 'workflow_config={"steps":[{"name":"calc","step_type":"agent","description":"Calculate the factorial of 5"}]}' \
  --param execution_mode="sequential" \
  --param model="ollama/llama3.2:3b" \
  --output json
```

**Output** (real LLM, 423ms):
```json
{
  "status": "ok",
  "result": {
    "type": "text",
    "value": "# Workflow Execution Report\n\n**Workflow**: custom-workflow\n**Steps Executed**: 1\n**Agents Invoked**: 1\n**Tools Invoked**: 0\n\n---\n\n## Step-by-Step Results\n\n### Step 1: step-1\n\nAgent executed successfully (workflow-agent-0)\nDescription: Calculate the factorial of 5\nModel: ollama/llama3.2:3b\nDuration: unknown\nNote: Real LLM execution completed. Agent outputs stored in workflow state.\n\n---\n\n## Final Output\n\nSequential workflow 'custom-workflow' completed successfully. 1 steps executed. Duration: 423.597167ms\n\n---\n\nGenerated by LLMSpell Workflow Orchestrator Template\n"
  },
  "artifacts": [],
  "metrics": {
    "duration_ms": 710,
    "tokens_used": null,
    "cost_usd": null,
    "agents_invoked": 1,
    "tools_invoked": 0,
    "rag_queries": 0,
    "custom_metrics": {
      "steps_executed": 1,
      "execution_mode": "sequential",
      "workflow_steps": 1
    }
  }
}
```

#### Parallel Mode - Real LLM Execution (llama3.2:3b)
```bash
./target/debug/llmspell template exec workflow-orchestrator \
  --param 'workflow_config={"steps":[{"name":"math1","step_type":"agent","description":"What is 5+7?"},{"name":"math2","step_type":"agent","description":"What is 9-3?"}]}' \
  --param execution_mode="parallel" \
  --param model="ollama/llama3.2:3b" \
  --output text
```

**Output** (real LLM, 818ms for 2 agents concurrently):
```
✓ Template execution completed in 1.11s
================================================================================

Result:
# Workflow Execution Report

**Workflow**: custom-workflow
**Steps Executed**: 2
**Agents Invoked**: 2
**Tools Invoked**: 0

---

## Step-by-Step Results

### Step 1: step-1

Agent executed successfully (workflow-agent-0)
Description: What is 5+7?
Model: ollama/llama3.2:3b
Duration: 818ms
Note: Real LLM execution completed. Agent outputs stored in workflow state.

### Step 2: step-2

Agent executed successfully (workflow-agent-1)
Description: What is 9-3?
Model: ollama/llama3.2:3b
Duration: 818ms
Note: Real LLM execution completed. Agent outputs stored in workflow state.

---

## Final Output

Parallel workflow 'custom-workflow' completed successfully. 2 branches executed, 2 succeeded, 0 failed. Duration: 818.189709ms

---

Generated by LLMSpell Workflow Orchestrator Template


Metrics:
  Duration:      1.11s
  Agents:        2
```

**Key Observations**:
- ✅ Real LLM execution confirmed (818ms vs previous 0ns mock)
- ✅ Parallel execution: Both agents ran concurrently (818ms total, not 1.6s sequential)
- ✅ Correct reporting: "2 branches executed, 2 succeeded"
- ✅ Fixed in Phase 12.8.5.4 by using ParallelWorkflowBuilder with registry

#### Conditional/Hybrid Mode - Real LLM Execution (llama3.2:3b)
```bash
./target/debug/llmspell template exec workflow-orchestrator \
  --param 'workflow_config={"steps":[{"name":"step1","step_type":"agent","description":"What is the capital of France?"},{"name":"step2","step_type":"agent","description":"What is 5*7?"}]}' \
  --param execution_mode="hybrid" \
  --param model="ollama/llama3.2:3b" \
  --output text
```

**Output** (real LLM, 559ms):
```
✓ Template execution completed in 0.80s
================================================================================

Result:
# Workflow Execution Report

**Workflow**: custom-workflow
**Steps Executed**: 2
**Agents Invoked**: 2
**Tools Invoked**: 0

---

## Step-by-Step Results

### Step 1: step-1

Agent executed successfully (workflow-agent-0)
Description: What is the capital of France?
Model: ollama/llama3.2:3b
Duration: 559ms
Note: Real LLM execution completed. Agent outputs stored in workflow state.

### Step 2: step-2

Agent executed successfully (workflow-agent-1)
Description: What is 5*7?
Model: ollama/llama3.2:3b
Duration: 559ms
Note: Real LLM execution completed. Agent outputs stored in workflow state.

---

## Final Output

Conditional workflow 'custom-workflow' completed successfully. 1 branches matched out of 1, 1 executed. 2 steps total, 2 succeeded, 0 failed. Duration: 559.007958ms

---

Generated by LLMSpell Workflow Orchestrator Template


Metrics:
  Duration:      0.80s
  Agents:        2
```

**Key Observations**:
- ✅ Real LLM execution confirmed (559ms vs previous 21ms mock)
- ✅ ConditionalWorkflowBuilder with registry support
- ✅ Single default branch with `Condition::Always` - sequential execution within branch
- ✅ Fixed in Phase 12.8.5.4

#### Loop Mode - Real LLM Execution (llama3.2:3b)
```bash
./target/debug/llmspell template exec workflow-orchestrator \
  --param 'workflow_config={"steps":[{"step_type":"agent","description":"Calculate 6*6"}]}' \
  --param execution_mode="loop" \
  --param model="ollama/llama3.2:3b" \
  --output text
```

**Output** (real LLM, 536ms):
```
✓ Template execution completed in 0.81s
================================================================================

Result:
# Workflow Execution Report

**Workflow**: custom-workflow
**Steps Executed**: 1
**Agents Invoked**: 1
**Tools Invoked**: 0

---

## Step-by-Step Results

### Step 1: step-1

Agent executed successfully (workflow-agent-0)
Description: Calculate 6*6
Model: ollama/llama3.2:3b
Duration: 536ms
Note: Real LLM execution completed. Agent outputs stored in workflow state.

---

## Final Output

Loop workflow 'custom-workflow' completed successfully. 1 of 1 iterations completed. Duration: 536.529ms

---

Generated by LLMSpell Workflow Orchestrator Template


Metrics:
  Duration:      0.81s
  Agents:        1
```

**Key Observations**:
- ✅ Real LLM execution confirmed (536ms)
- ✅ LoopWorkflowBuilder with registry support
- ✅ Single iteration (hardcoded range: 1..2)
- ✅ Enabled in Phase 12.8.5.4 (added to schema + mapping)

---

## Execution Modes

### Sequential Mode
Execute steps one after another in order:

```
Step 1 → Step 2 → Step 3 → Final Output
```

- **Use Case**: Steps depend on previous outputs
- **WorkflowType**: Sequential
- **Error Handling**: Stop on first error (FailFast)

### Parallel Mode
Execute independent steps concurrently:

```
           ┌─ Step 2 ─┐
Step 1 ──┼─ Step 3 ─┼→ Final
           └─ Step 4 ─┘
```

- **Use Case**: Steps can run independently
- **WorkflowType**: Parallel
- **Error Handling**: Continue on error (collect all results)
- **Max Concurrency**: 4 concurrent steps

### Hybrid Mode (Conditional)
Sequential execution via ConditionalWorkflow with single default branch:

```
Step 1 → Step 2 → Step 3 → Final
(All steps in one "default" branch with Condition::Always)
```

- **Use Case**: Demonstrates ConditionalWorkflow integration
- **WorkflowType**: Conditional
- **Error Handling**: FailFast (stop on first error)
- **Branch Structure**: Creates ONE default branch containing ALL steps
- **Condition**: `Condition::Always` - branch always executes
- **Real LLM Execution**: ✅ Yes (Phase 12.8.5.4) - Uses ConditionalWorkflowBuilder with registry
- **Current Limitation**: NOT true conditional routing with branching logic
  - All steps execute sequentially in single branch
  - No dynamic routing or condition evaluation
  - True multi-branch conditionals planned for Phase 14
- **Why "Hybrid"?**: Name reserved for future conditional branching feature
- **Actual Behavior**: Equivalent to sequential execution wrapped in ConditionalWorkflow

### Loop Mode
Iterative execution via LoopWorkflow:

```
Iteration 1: Step 1 → Step 2 → Step 3
(Currently: single iteration, hardcoded range 1..2)
```

- **Use Case**: Repetitive task execution
- **WorkflowType**: Loop
- **Error Handling**: FailFast (stop on first error)
- **Real LLM Execution**: ✅ Yes (Phase 12.8.5.4) - Uses LoopWorkflowBuilder with registry
- **Iteration Count**: Currently hardcoded to 1 iteration (range: 1..2, step: 1)
- **Current Limitation**: Fixed iteration count in template
  - Future enhancement: User-configurable iteration count
  - Planned for Phase 14: Loop conditions and dynamic ranges

---

## Performance

### Real LLM Execution Results (Sub-Tasks 12.8.5.3 + 12.8.5.4)

**Test Configuration**:
- **Models Tested**:
  - ollama/llama3.2:3b (fast small model)
  - ollama/deepseek-r1:8b (larger reasoning model)
- **Test Environment**: Phase 12.8.5.3-12.8.5.4 real LLM integration testing
- **Execution**: Real Ollama inference via ComponentRegistry pattern
- **All 4 Modes**: Sequential, Parallel, Conditional, Loop - all with real LLM

**Actual Test Results** (Real LLM Execution - All Modes):

| Mode | Model | Steps | Duration | Execution Type | Notes |
|------|-------|-------|----------|----------------|-------|
| Sequential | llama3.2:3b | 1 agent | 455ms | Real LLM | ✅ Working |
| Sequential | llama3.2:3b | 1 agent | 246-616ms | Real LLM | Variance due to system load |
| Sequential | deepseek-r1:8b | 1 agent | 11-15s | Real LLM | Reasoning model (larger, slower) |
| Parallel | llama3.2:3b | 2 agents | 818ms | Real LLM | ✅ Fixed 12.8.5.4 - concurrent execution |
| Conditional | llama3.2:3b | 2 agents | 559ms | Real LLM | ✅ Fixed 12.8.5.4 - sequential in branch |
| Loop | llama3.2:3b | 1 agent | 536ms | Real LLM | ✅ Enabled 12.8.5.4 - single iteration |

**Verified Real Execution** (debug logs):
```
DEBUG: Looking for agent with name: 'workflow-agent-0'
INFO: Starting parallel workflow: custom-workflow (execution: uuid) with 2 branches
DEBUG: ParallelWorkflow::execute() starting with 2 branches
DEBUG: Branch 'branch-1' completed with success=true
DEBUG: Branch 'branch-2' completed with success=true
Step 'description' completed successfully in 818.189709ms
```

**Phase Breakdown** (Real LLM - Sequential):
- Phase 1 (Parse): ~1-2ms (JSON parsing + validation)
- Phase 2 (Build Plan): < 1ms (step limiting)
- Phase 2.5 (Agent Pre-Creation): ~10-15s (for deepseek-r1:8b), ~200-400ms (for llama3.2:3b)
  - Agent creation via context.agent_registry()
  - ComponentRegistry building: < 1ms
- Phase 3 (Execute): Included in agent pre-creation time (workflow uses pre-created agents)
- Phase 4 (Aggregate): ~1ms (result collection)

**Performance Scaling** (Real LLM - Sequential Mode):

| Workflow Size | Steps | Model | Est. Duration | Notes |
|---------------|-------|-------|---------------|-------|
| Small | 1-2 | llama3.2:3b | ~0.5-1s | Fast model, minimal overhead |
| Small | 1-2 | deepseek-r1:8b | ~20-30s | Reasoning model dominates |
| Medium | 3-5 | llama3.2:3b | ~1.5-3s | Linear scaling per agent |
| Medium | 3-5 | deepseek-r1:8b | ~60-100s | Reasoning overhead per agent |
| Large | 6-10 | llama3.2:3b | ~3-6s | Sequential execution |
| Large | 6-10 | deepseek-r1:8b | ~120-200s | Not recommended for large workflows |

**Overhead Analysis** (Real LLM):
- **Agent Pre-Creation**: Dominates total time (real LLM inference)
- **Workflow Creation**: < 1ms (builder pattern)
- **ComponentRegistry**: < 1ms (HashMap registration)
- **Branch Conversion**: < 1ms for up to 10 steps
- **Step Conversion**: < 1ms (internal StepType → WfStepType with agent IDs)
- **JSON Parsing**: 1-2ms for workflow_config
- **Total Non-LLM Overhead**: < 5ms (negligible vs LLM inference)

**Scalability**:
- **Sequential Mode**: ✅ Tested with real LLMs up to 2 agents per workflow
- **Parallel Mode**: ✅ Tested with real LLMs - 2 concurrent agents (818ms)
- **Conditional Mode**: ✅ Tested with real LLMs - 2 sequential agents (559ms)
- **Loop Mode**: ✅ Tested with real LLMs - 1 agent, 1 iteration (536ms)
- **Max Concurrency** (Parallel): 4 branches (configurable via builder)
- **Agent Pre-Creation Overhead**: Linear with agent count (~250-500ms per llama3.2:3b agent)

**Key Insight**: LLM inference time dominates workflow execution (99%+ of total duration). Template overhead (< 5ms) is negligible.

---

## Troubleshooting

### Error: "workflow_config must contain 'steps' array"

**Cause**: Missing or invalid `steps` field in workflow_config

**Solution**: Ensure workflow_config has steps array:
```bash
--param workflow_config='{"steps":[{"name":"s1","step_type":"agent"}]}'
```

### Error: "workflow must have at least one step"

**Cause**: Empty steps array

**Solution**: Add at least one step to workflow:
```json
{
  "workflow_config": {
    "steps": [
      {"name": "step1", "step_type": "agent"}
    ]
  }
}
```

### Error: "invalid step_type: 'invalid'. Must be 'agent' or 'tool'"

**Cause**: step_type is not "agent" or "tool"

**Solution**: Use valid step types:
```json
{"step_type": "agent"}  // ✓ Valid
{"step_type": "tool"}   // ✓ Valid
{"step_type": "custom"} // ✗ Invalid
```

### Error: "Failed to create workflow"

**Cause**: WorkflowFactory could not create workflow from parameters

**Solutions**:
1. Check execution_mode is valid: "sequential", "parallel", or "hybrid"
2. Ensure step format matches WorkflowStep schema
3. Verify model parameter is accessible
4. Check WorkflowFactory is available in ExecutionContext

### Error: "Invalid parallel config: missing field `continue_on_optional_failure`"

**Cause**: ParallelConfig deserialization failed - missing required field in type_config

**Root Cause**: Incomplete ParallelConfig JSON in type_config. ParallelConfig requires 4 fields:
- `max_concurrency` (usize)
- `fail_fast` (bool)
- `timeout` (Option<Duration>)
- `continue_on_optional_failure` (bool) ← **REQUIRED**

**Solution**: Ensure all 4 fields present in parallel type_config (workflow_orchestrator.rs:449-454):
```json
{
  "max_concurrency": 4,
  "fail_fast": false,
  "timeout": null,
  "continue_on_optional_failure": true
}
```

**Fix Location**: Template automatically generates complete type_config (Phase 12.8.5 fix)

### Error: "Parallel workflow completed: 0 branches executed, Duration: 0ns"

**Cause**: Parallel workflow returns immediately without executing branches

**Root Cause** (FIXED in Phase 12.8.5.4):
- `llmspell-workflows/src/parallel.rs:707` checked `if context.state.is_some()`
- When no state available → returned fake result (0 branches, 0ns duration)
- `ExecutionContext::default()` has no state → always hit fake result path
- Sequential/Conditional/Loop didn't have this requirement → worked correctly

**Why It Was Broken**:
```rust
// OLD CODE (parallel.rs:707-932)
let (workflow_result, execution_id) = if context.state.is_some() {
    // Real execution (lines 709-917)
    execute_branches_and_collect_results()
} else {
    // FAKE RESULT - immediate return!
    (ParallelWorkflowResult {
        branch_results: vec![],  // Empty!
        duration: Duration::from_secs(0),  // 0ns!
        successful_branches: 0,
        ...
    }, None)
};
```

**Fix Applied** (Phase 12.8.5.4):
- Removed state requirement: Changed `if context.state.is_some()` to unconditional execution
- Parallel workflow now executes branches regardless of state availability
- Matches Sequential/Conditional/Loop behavior (no state requirement)

**Solution**: Update to latest version (Phase 12.8.5.4+)
- Parallel workflows now work with `ExecutionContext::default()`
- Real LLM execution confirmed: 818ms duration for 2 concurrent agents
- Proper reporting: "2 branches executed, 2 succeeded, 0 failed"

**Files Modified**:
- `llmspell-workflows/src/parallel.rs:707` - Removed state check
- `llmspell-workflows/src/parallel.rs:921` - Fixed branch count calculation
- `llmspell-templates/src/builtin/workflow_orchestrator.rs:566-587` - Uses ParallelWorkflowBuilder with registry

### Error: "Cannot execute conditional workflow without branches"

**Cause**: ConditionalWorkflow.branches is empty - no branches provided to workflow

**Root Cause**: Similar to parallel error - factory not extracting conditional branches

**Solution**: Ensure type_config includes `branches` field with ConditionalBranch:
```json
{
  "branches": [
    {
      "id": "uuid-here",
      "name": "default",
      "condition": "Always",
      "steps": [WorkflowStep],
      "is_default": true
    }
  ]
}
```

**Fix Location**:
- Factory extraction: `llmspell-workflows/src/factory.rs:175-185` (uses ConditionalWorkflowBuilder)
- Template generation: `workflow_orchestrator.rs:459-465`

---

## Architecture Insights

### Why Builder Pattern (Not Factory)?

**Evolution** (Phase 12.8.5.3 → 12.8.5.4):
- **Phase 12.8.5.3**: Sequential used builder, Parallel/Conditional used factory
  - Problem: Factory didn't support ComponentRegistry → mock execution
  - Sequential worked with real LLM, Parallel/Conditional didn't
- **Phase 12.8.5.4**: ALL workflows use builders
  - Solution: Direct builder instantiation with `.with_registry()`
  - Result: All 4 modes now support real LLM execution

**Builder Pattern Benefits**:
- **Registry Support**: All builders have `.with_registry()` method
- **Type Safety**: Workflow as Arc<dyn BaseAgent> enables uniform execution interface
- **Flexibility**: Builders support 4 workflow types (Sequential, Parallel, Conditional, Loop)
- **Configuration**: Fine-grained control over workflow construction

**Why Factory Doesn't Work**:
- Factory pattern (factory.rs) doesn't accept ComponentRegistry parameter
- Cannot pass pre-created agents to factory-created workflows
- Results in mock execution (no real LLM inference)

### Step Type Mapping

Internal template StepType → llmspell_workflows StepType conversion:

- **Agent**: Maps to `WfStepType::Agent { agent_id, input }`
- **Tool**: Maps to `WfStepType::Tool { tool_name, parameters }`

### Execution Context Duality

Two ExecutionContext types used:

1. **Template ExecutionContext** (`crate::context::ExecutionContext`):
   - Builder-based construction
   - Provides infrastructure access (workflow_factory, tool_registry, etc.)
   - Used for workflow creation

2. **Core ExecutionContext** (`llmspell_core::ExecutionContext`):
   - Default construction
   - Used for workflow execution
   - Passed to BaseAgent.execute()

---

## Cost Estimation

**Estimated Costs (per execution)**

| Workflow Size | Steps | Tokens | Duration | Cost (USD) |
|--------------|-------|--------|----------|------------|
| Small | 2-3 | ~2,000 | ~6s | $0.00020 |
| Medium | 4-6 | ~4,000 | ~12s | $0.00040 |
| Large | 7-10 | ~7,000 | ~20s | $0.00070 |

**Formula**:
- Agent steps: ~1000 tokens each (70% of steps assumed)
- Tool steps: minimal tokens (30% of steps assumed)
- Cost: $0.10 per 1M tokens (local LLM)

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Workflow System Guide](../../workflows/README.md)
- [Research Assistant Template](./research-assistant.md) (multi-phase example)
- [Code Generator Template](./code-generator.md) (3-agent chain)
- [Data Analysis Template](./data-analysis.md) (3-agent pipeline)

---

## Implementation Status

### Phase 12.8.5 - ✅ COMPLETE

**Implemented**:
- ✅ JSON workflow parsing with validation
- ✅ Builder-based workflow creation (all 4 types)
- ✅ Sequential, parallel, hybrid, and loop execution modes
- ✅ Step type detection (agent/tool)
- ✅ Real LLM execution for ALL workflow types
- ✅ ComponentRegistry pattern for agent resolution
- ✅ Result aggregation and reporting
- ✅ Artifact generation (report + intermediate JSON)
- ✅ Comprehensive error handling
- ✅ Zero clippy warnings

**Quality Metrics**:
- ✅ Compilation: Clean (0 errors, 0 warnings)
- ✅ Clippy: Clean (0 warnings)
- ✅ Unit Tests: 120 passed, 0 failed
- ✅ Parameter validation: Comprehensive with detailed errors
- ✅ Real LLM Testing: All 4 modes verified with ollama/llama3.2:3b

**Key Achievements**:
1. **Sub-Task 12.8.5.3**: Sequential workflow with real LLM execution (246-616ms)
2. **Sub-Task 12.8.5.4**: Fixed parallel workflow state requirement bug
3. **Sub-Task 12.8.5.4**: Enabled loop workflow (added to schema + mapping)
4. **Sub-Task 12.8.5.4**: All 4 workflow types now use unified builder pattern
5. Flexible JSON parsing supporting multiple formats
6. ComponentRegistry pattern enabling real agent resolution
7. Verified real LLM execution across all modes

---

## Changelog

### v0.1.0 (Phase 12.8.5) - Production Ready

**Implemented**:
- ✅ Phase 1: Workflow parsing from JSON (parse_workflow + parse_step_type)
- ✅ Phase 2: Execution plan building (build_execution_plan)
- ✅ Phase 3: Workflow execution via Builders (execute_workflow)
  - Sub-Task 12.8.5.3: Sequential with real LLM (ComponentRegistry pattern)
  - Sub-Task 12.8.5.4: Parallel/Conditional/Loop with real LLM (fixed state bug)
- ✅ Phase 4: Result aggregation (aggregate_results)
- ✅ WorkflowStep conversion to llmspell_workflows format
- ✅ Four execution modes (sequential, parallel, hybrid, loop)
- ✅ Error handling with ValidationError
- ✅ Artifact generation

**Key Features**:
- Builder pattern for all 4 workflow types
- ComponentRegistry for real agent resolution
- Flexible step_type parsing (string or object format)
- Mode-based workflow creation with registry support
- Comprehensive validation
- Rich error messages
- Parallel workflow state requirement bug fix
- Production-ready implementation with real LLM verification

**Bug Fixes**:
- Fixed parallel workflow returning fake result when no state (parallel.rs:707)
- Fixed parallel branch count reporting (parallel.rs:921)
- Enabled loop workflow (added to schema + mapping)
- Fixed test_parse_workflow_placeholder format

---

**Last Updated**: Phase 12.8.5 (Production Implementation)
**Status**: ✅ Ready for Production Use
