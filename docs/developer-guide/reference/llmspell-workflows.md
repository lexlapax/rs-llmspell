# llmspell-workflows

**Workflow orchestration and execution**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-workflows) | [Source](../../../../llmspell-workflows)

---

## Overview

`llmspell-workflows` provides powerful workflow orchestration including sequential, parallel, and conditional execution patterns with state management and error handling.

**Key Features:**
- 🔄 Sequential, parallel, conditional flows
- 📊 Step state management
- ⚡ Async execution
- 🔧 Error handling and retries
- 📈 Progress tracking
- 🎯 Dynamic step resolution
- 🏗️ Workflow composition
- 📝 Workflow templates

## Core Components

```rust
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<Step>,
    pub flow_type: FlowType,
    pub state: WorkflowState,
}

pub enum FlowType {
    Sequential,
    Parallel,
    Conditional(Box<dyn Condition>),
    Loop { max_iterations: usize },
}

pub struct Step {
    pub name: String,
    pub step_type: StepType,
    pub retry_policy: Option<RetryPolicy>,
}

pub enum StepType {
    Tool { tool_name: String, parameters: serde_json::Value },
    Agent { agent_id: String, input: String },
    Workflow { workflow_id: ComponentId, input: serde_json::Value },
}
```

## WorkflowBuilder

```rust
let workflow = WorkflowBuilder::new()
    .name("data-pipeline")
    .sequential()
    .add_step("fetch", Step::tool("web-fetch", json!({"url": "..."})))
    .add_step("process", Step::agent(processor_agent))
    .add_step("save", Step::tool("file-write", json!({"path": "output.json"})))
    .with_error_handler(ErrorHandler::Retry { max_attempts: 3 })
    .build()?;

let result = workflow.execute(context).await?;
```

## Parallel Execution

```rust
let workflow = WorkflowBuilder::new()
    .name("parallel-analysis")
    .parallel()
    .add_step("analyze1", Step::agent(analyst1))
    .add_step("analyze2", Step::agent(analyst2))
    .add_step("analyze3", Step::agent(analyst3))
    .build()?;
```

## Conditional Flows

```rust
let workflow = WorkflowBuilder::new()
    .name("conditional-process")
    .conditional(|state| {
        if state.get("score")?.as_f64()? > 0.8 {
            Branch::A
        } else {
            Branch::B
        }
    })
    .on_branch(Branch::A, high_quality_flow)
    .on_branch(Branch::B, standard_flow)
    .build()?;
```

## Agent Output Collection

All workflow types automatically collect agent outputs during execution:

```rust
use llmspell_workflows::SequentialWorkflowBuilder;
use llmspell_core::traits::workflow::Workflow;

// Execute workflow with agents
let result = workflow.execute(input, context).await?;

// Access collected agent outputs from metadata
if let Some(agent_outputs) = result.metadata.extra.get("agent_outputs") {
    if let Some(outputs_map) = agent_outputs.as_object() {
        for (agent_id, output) in outputs_map {
            println!("Agent {}: {:?}", agent_id, output);
        }
    }
}
```

**Key Points:**
- Agent outputs are automatically collected into `result.metadata.extra.agent_outputs`
- Only workflows with agent steps populate this field
- Agent IDs include timestamp suffixes (e.g., `"analyst_1234567890"`)
- Failed agents may still have outputs if partial execution occurred

**Implementation Details:**

```rust
// In execute_impl(), after workflow completes:
let mut agent_outputs = serde_json::Map::new();
if let Some(ref state) = context.state {
    for step in &self.steps {
        if let StepType::Agent { agent_id, .. } = &step.step_type {
            let key = format!("workflow:{}:agent:{}:output", execution_id, agent_id);
            if let Ok(Some(output)) = state.read(&key).await {
                agent_outputs.insert(agent_id.clone(), output);
            }
        }
    }
}
if !agent_outputs.is_empty() {
    metadata.extra.insert("agent_outputs".to_string(),
                         serde_json::Value::Object(agent_outputs));
}
```

## Related Documentation

- [llmspell-agents](llmspell-agents.md) - Agent execution in workflows
- [llmspell-state-persistence](llmspell-state-persistence.md) - Workflow state management