# Research-Chat v1.0

**Phase 13 Workflow-Template Delegation Example**

AI research assistant with conversational follow-up, demonstrating workflow-template composition pattern with session-based memory sharing.

## Overview

Research-Chat validates **Phase 13.13 (Workflow-Template Delegation)** by composing two templates into a sequential workflow:

1. **research-assistant** template → gathers information, stores in RAG
2. **interactive-chat** template → retrieves research context, enables Q&A

Both templates share memory via identical `session_id`, proving that workflow→template execution works and memory integration is complete.

## Architecture

### Composition Pattern (Option E)

```
┌─────────────────────────────────────────────┐
│  Sequential Workflow: "research-chat"       │
├─────────────────────────────────────────────┤
│                                             │
│  Step 1: StepType::Template                │
│  ├─ template_id: "research-assistant"      │
│  ├─ params: { topic, session_id, ... }     │
│  └─ Action: Web search + RAG ingest        │
│           (Memory Anchor: session_id)       │
│                                             │
│  Step 2: StepType::Template                │
│  ├─ template_id: "interactive-chat"        │
│  ├─ params: { message, session_id, ... }   │
│  └─ Action: RAG retrieval + LLM response   │
│           (Memory Retrieval: session_id)    │
│                                             │
└─────────────────────────────────────────────┘
                     │
                     ▼
        Template Executor Flow:
        TemplateBridge → WorkflowBridge →
        Workflows → StepExecutionContext →
        StepExecutor → TemplateExecutor
```

### Key Architectural Concepts

1. **Workflow-Template Bridge**
   - Workflows delegate to templates via `StepType::Template`
   - TemplateBridge injected into workflow execution pipeline
   - StepExecutor handles template steps using TemplateExecutor trait

2. **Session-Based Memory Sharing**
   - Research template stores findings with `session_id` anchor
   - Chat template retrieves context using same `session_id`
   - Memory persists across independent template executions
   - Enables multi-turn conversations with research context

3. **Reference Implementation**
   - Shows HOW composition works (extensible by users)
   - Validates Phase 13 completion (memory + templates + workflows)
   - Demonstrates zero-day retention problem solution

## Usage

### Basic Usage

```bash
llmspell app run research-chat --topic "Rust async programming"
```

### Custom Parameters

```bash
llmspell app run research-chat \
  --topic "Rust ownership and borrowing" \
  --max_sources 15 \
  --question "What are the key rules of the borrow checker?"
```

### Debug Mode

```bash
RUST_LOG=debug llmspell app run research-chat --topic "Tokio runtime"
```

### Continue Conversation

After workflow execution completes, you can continue the conversation using the printed session ID:

```bash
llmspell template exec interactive-chat \
  --param session_id=research-chat-20231030-143022 \
  --param message="Can you elaborate on async/await syntax?"
```

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `topic` | string | "Rust async programming" | Research topic to investigate |
| `max_sources` | integer | 10 | Maximum number of sources (1-50) |
| `question` | string | "Summarize the key findings" | Initial question for chat phase |
| `max_turns` | integer | 1 | Maximum chat turns in interactive phase |

## Expected Output

```
=== Research-Chat v1.0 (Phase 13 Composition Demo) ===
Workflow-Template Delegation with Shared Memory

Session ID: research-chat-20231030-143022
Topic: Rust async programming

1. Creating research workflow with template delegation...
  ✅ Added research step (research-assistant template)
  ✅ Added chat step (interactive-chat template)

2. Executing workflow...
  → Research phase: Gathering information on 'Rust async programming'
  → Chat phase: Answering question with research context

3. Results:
=============================================================
  ✅ Workflow Status: SUCCESS

  📊 Execution Summary:
    • Steps Completed: 2 (research + chat)
    • Session ID: research-chat-20231030-143022
    • Memory Sharing: ACTIVE
    • Templates Used: research-assistant, interactive-chat

  📚 Research Phase Output:
    [Research findings stored in memory...]

  💬 Chat Phase Output:
    [AI response with research context...]

  🔄 To Continue This Conversation:
    llmspell template exec interactive-chat \
      --param session_id=research-chat-20231030-143022 \
      --param message="Your next question here"
```

## Prerequisites

- OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable
- llmspell v0.13.0+ (Phase 13 implementation)
- Templates: research-assistant, interactive-chat (built-in since Phase 12)

## Validation Criteria

This application validates the following Phase 13 requirements:

- [x] Workflow→template execution works (StepType::Template)
- [x] TemplateBridge accessible in workflow execution context
- [x] Session-based memory sharing across templates
- [x] Research context persists and retrieves correctly
- [x] Chat references research findings (memory integration confirmed)
- [x] Zero-day retention problem solved (memory + composition)

## Technical Implementation

### Lua Code Highlights

**Workflow Creation**:
```lua
local workflow = Workflow.sequential("research-chat")
```

**Template Step Addition** (Phase 13 API):
```lua
workflow:add_template_step("research", "research-assistant", {
    topic = args.topic,
    session_id = session_id,  -- Memory anchor
    memory_enabled = true,
})
```

**Session ID Generation**:
```lua
local session_id = "research-chat-" .. os.date("%Y%m%d-%H%M%S")
```

### Rust Infrastructure (Phase 13 Implementation)

1. **StepType::Template** (Task 13.13.1)
   - Added variant to `llmspell-workflows/src/traits.rs`
   - Fields: `template_id: String`, `params: serde_json::Value`

2. **StepExecutor Template Handler** (Task 13.13.2)
   - Template execution in `llmspell-workflows/src/step_executor.rs`
   - Retrieves TemplateExecutor from StepExecutionContext

3. **Workflow Builder Helpers** (Task 13.13.3)
   - `.add_template_step()` convenience method
   - Added to SequentialWorkflowBuilder

4. **Bridge Integration** (Task 13.13.4)
   - TemplateBridge → WorkflowBridge → Workflows → StepExecutionContext
   - Template executor passed via builder pattern

5. **Lua Bridge Support** (Task 13.13.4b)
   - `parse_workflow_step()` supports "template" step type
   - `add_template_step()` Lua method added to workflow builders

## References

- **Phase 13 Design**: `/docs/in-progress/phase-13-design-doc.md`
- **Template Architecture**: `/docs/technical/template-system-architecture.md`
- **Workflow Patterns**: `/docs/user-guide/workflows/`
- **Template Documentation**: `/docs/user-guide/templates/`
- **TODO Tracking**: `/TODO.md` (Phase 13.13)

## License

Part of rs-llmspell project - Production-ready AI workflow orchestration platform.

---

**Status**: ✅ Complete (Phase 13.13.5 validation artifact)
**Version**: 1.0.0
**Category**: Example Application (Reference Implementation)
**Complexity**: Medium
