# Communication Manager v3.0

A comprehensive business communication management system demonstrating conditional workflows with the new table-based condition API, state persistence, and multi-agent orchestration.

## Overview

The Communication Manager showcases:
- **NEW Conditional Routing**: Table-based conditions using `{ type = "always" }` and `{ type = "never" }`
- **Then/Else Branches**: Proper conditional workflow with escalation (then) and standard (else) paths
- **5 Business Agents**: Communication classifier, sentiment analyzer, response generator, schedule coordinator, tracking agent
- **State Persistence**: Session management and communication thread tracking
- **Business Layer Architecture**: Enterprise-grade communication automation

## Prerequisites

### Required
- llmspell built and available (`cargo build --release`)
- At least one of:
  - OpenAI API key: `export OPENAI_API_KEY="sk-..."`
  - Anthropic API key: `export ANTHROPIC_API_KEY="sk-ant-..."`

### Optional
- Both API keys for multi-provider functionality
- Custom webhook endpoints for notifications

## Quick Start

### 1. Basic Execution (No API Keys)
```bash
# Runs with simulated agents and basic responses
./target/debug/llmspell run examples/script-users/applications/customer-support-bot/main.lua
```

### 2. With Configuration File
```bash
# Uses the provided config.toml for provider settings
LLMSPELL_CONFIG=examples/script-users/applications/customer-support-bot/config.toml \
  ./target/debug/llmspell run examples/script-users/applications/customer-support-bot/main.lua
```

### 3. Full Production Mode
```bash
# Set API keys for real ticket processing
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Run with full capabilities
./target/debug/llmspell run examples/script-users/applications/customer-support-bot/main.lua
```

## Architecture

### Workflow Structure (NEW Conditional API)

```
Communication Management (Conditional Workflow)
├── Initial Analysis Steps
│   ├── Classify Communication (Agent)
│   └── Analyze Sentiment (Agent)
│
├── Condition: { type = "always" }  // Demo: takes then_branch
│   // ✅ NOW WORKING: { type = "shared_data_equals", key = "sentiment", value = "NEGATIVE" }
│
├── THEN Branch (Escalation Path)
│   ├── Escalate Response (Agent) - Urgent empathetic response
│   └── Notify Management (Tool: webhook) - Alert executives
│
└── ELSE Branch (Standard Path)
    ├── Standard Response (Agent) - Professional response
    ├── Coordinate Schedule (Agent) - Meeting coordination
    └── Track Communication (Agent) - Thread tracking
```

### Agents (5 Business Agents)

| Agent | Model | Purpose | Temperature |
|-------|-------|---------|-------------|
| **Communication Classifier** | GPT-4o-mini | Classifies by type, priority (1-10), urgency | 0.3 |
| **Sentiment Analyzer** | Claude-3-haiku | Returns POSITIVE, NEUTRAL, or NEGATIVE | 0.3 |
| **Response Generator** | GPT-4o-mini | Professional responses (escalation or standard) | 0.4 |
| **Schedule Coordinator** | Claude-3-haiku | Meeting times and follow-up scheduling | 0.3 |
| **Tracking Agent** | GPT-4o-mini | Communication thread and relationship tracking | 0.3 |

### NEW Conditional Routing API

```lua
-- Table-based conditions (not Lua functions!)
:condition({ 
    type = "always"     -- Always executes then_branch
    -- OR
    type = "never"      -- Always executes else_branch
    -- ✅ NOW WORKING: type = "shared_data_equals", key = "sentiment", value = "NEGATIVE"
    -- ✅ NOW WORKING: type = "shared_data_exists", key = "user_id"
})

-- Separate then/else step methods
:add_then_step({ ... })  -- Escalation path
:add_else_step({ ... })  -- Standard path
```

## Configuration

### config.toml Structure

```toml
default_engine = "lua"

[providers.providers.openai]
provider_type = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o-mini"

[providers.providers.anthropic]
provider_type = "anthropic"
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-haiku-20240307"
```

### Customization Options

Edit `main.lua` to customize:

```lua
local config = {
    models = {
        classifier = "openai/gpt-4o-mini",       -- Change models
        sentiment = "anthropic/claude-3-haiku",  -- For different tasks
    },
    thresholds = {
        urgent_priority = 8,      -- Adjust urgency threshold
        negative_sentiment = -0.5 -- Adjust escalation trigger
    }
}
```

## Sample Tickets

The system processes three sample tickets demonstrating different scenarios:

1. **Login Issues** - Frustrated customer, potential urgent
2. **General Question** - Polite inquiry, standard handling
3. **Production Down** - Critical emergency, immediate escalation

## Output Files

| File | Description |
|------|-------------|
| `/tmp/support-tickets.txt` | Input tickets for processing |
| `/tmp/support-responses.txt` | Generated customer responses |
| `/tmp/support-logs.txt` | Execution summary and metrics |

## Performance Metrics

Typical execution times:

- **Ticket Classification**: ~30ms
- **Urgent Handler** (Parallel): ~50ms
- **Standard Handler** (Sequential): ~75ms
- **Total Processing**: ~150-200ms per ticket batch

## Parallel vs Sequential Processing

### Urgent Handler (Parallel)
- Simultaneous response generation and supervisor notification
- Faster total processing time
- Used for critical tickets requiring immediate action

### Standard Handler (Sequential)
- Step-by-step processing: sentiment → response → notification
- More thorough analysis
- Used for normal priority tickets

## Cost Considerations

**Warning**: Real API usage incurs costs:

- **OpenAI GPT-4o-mini**: ~$0.15 per 1M input tokens
- **Anthropic Claude-3-haiku**: ~$0.25 per 1M input tokens
- **Typical run cost**: $0.0005 - $0.002 per ticket

## Troubleshooting

### "Agent needs API key" Messages
- System continues with basic responses
- Set environment variables for full functionality

### Webhook Errors
- webhook-caller tool requires specific format
- Check endpoint URLs in config

### Routing Issues
- Verify conditional workflow support in llmspell build
- Check classification agent output format

## Blueprint Compliance

✅ Main Router with conditional logic
✅ Parallel workflow for urgent tickets
✅ Sequential workflow for standard tickets
✅ 3 specialized agents
✅ Multiple tool integrations
✅ Production-grade error handling

## Example Use Cases

1. **Help Desk Automation**: Automatic ticket triage and response
2. **Emergency Escalation**: Immediate handling of critical issues
3. **Sentiment-Based Routing**: Emotional tone detection for better service
4. **Multi-Channel Support**: Email, chat, and ticket processing
5. **SLA Management**: Priority-based response times

## Extending the System

1. **Add More Agents**: Knowledge base search, solution finder
2. **Enhanced Routing**: Multi-level conditional branches
3. **State Persistence**: Track ticket history across sessions
4. **Analytics Integration**: Performance metrics and reporting
5. **Custom Templates**: Industry-specific response templates

## Related Examples

### Cookbook Patterns
For focused communication patterns and building blocks:
- **[webhook-integration.lua](../../cookbook/webhook-integration.lua)** - Webhook calling patterns for notifications
- **[workflow-patterns.lua](../../cookbook/)** - Conditional workflow building blocks

### Other Applications
- **Content Generation Platform**: Similar conditional routing patterns
- **Data Pipeline**: Parallel processing techniques