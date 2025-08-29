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
    Agent(Box<dyn Agent>),
    Tool { name: String, params: Value },
    Workflow(Box<Workflow>),
    Custom(Box<dyn CustomStep>),
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

## Related Documentation

- [llmspell-agents](llmspell-agents.md) - Agent execution in workflows
- [llmspell-state-persistence](llmspell-state-persistence.md) - Workflow state management