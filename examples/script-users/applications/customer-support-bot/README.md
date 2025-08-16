# Customer Support System

An intelligent ticket routing and response generation system demonstrating conditional workflows, parallel processing, and multi-agent collaboration in llmspell.

## Overview

The Customer Support System showcases:
- **Conditional Routing**: Intelligent ticket classification and priority-based routing
- **Parallel Processing**: Urgent tickets handled with simultaneous response and escalation
- **Sequential Workflows**: Standard tickets processed through sentiment analysis pipeline
- **3 Specialized Agents**: Classifier, sentiment analyzer, and response generator
- **Blueprint v2.0 Compliant**: Production-ready architecture patterns

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

### Workflow Structure

```
Main Router (Sequential)
├── Ticket Classification (Agent)
│   ├── Priority Assessment (1-10 scale)
│   ├── Category Assignment
│   └── Urgency Detection
│
└── Conditional Routing
    ├── Urgent Handler (Parallel Workflow)
    │   ├── Generate Immediate Response (Agent)
    │   └── Notify Supervisor (Tool: webhook)
    │
    └── Standard Handler (Sequential Workflow)
        ├── Analyze Sentiment (Agent)
        ├── Generate Response (Agent)
        └── Notify Customer (Tool: webhook)
```

### Agents

| Agent | Model | Purpose | Temperature |
|-------|-------|---------|-------------|
| **Ticket Classifier** | GPT-4o-mini | Categorizes and prioritizes tickets | 0.3 |
| **Sentiment Analyzer** | Claude-3-haiku | Detects customer emotion and escalation needs | 0.2 |
| **Response Generator** | GPT-4o-mini | Creates professional customer responses | 0.6 |

### Routing Logic

- **Urgent Route**: Priority ≥ 8 or critical keywords detected
- **Standard Route**: All other tickets
- **Escalation**: Negative sentiment ≤ -0.5 triggers supervisor alert

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

- **Content Generation Platform**: Similar conditional routing patterns
- **Data Pipeline**: Parallel processing techniques
- **Workflow Examples**: Basic patterns in `examples/lua/workflows/`