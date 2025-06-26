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

## Dependencies
- `llmspell-core` - Core Workflow trait
- `llmspell-agents` - Agent implementations
- `llmspell-hooks` - Event emission during execution