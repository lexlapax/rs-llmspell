# Professional Process Orchestrator

## Overview
Professional-grade business process orchestration application demonstrating advanced workflow patterns, conditional routing, and multi-agent coordination. This represents Layer 5 (Professional Mastery) in the Universal → Professional progression.

## Features
- **8-Agent Architecture**: Specialized agents for different process types
- **NEW Conditional Workflows**: Table-based conditions with then/else branches
- **Two Conditional Workflows**: Master orchestration and incident routing
- **Multi-Process Support**: Approval, migration, QA, and incident workflows
- **Professional Complexity**: Enterprise-grade orchestration patterns
- **State Management**: Persistent workflow state and audit trails

## Agent Architecture
1. **Process Intake Agent** - Initial categorization and intake
2. **Rules Classifier Agent** - Business rules and routing logic
3. **Approval Coordinator Agent** - Authorization workflows
4. **Migration Manager Agent** - Data migration orchestration
5. **QA Coordinator Agent** - Quality assurance workflows
6. **Incident Manager Agent** - Incident response coordination
7. **Notification Orchestrator Agent** - Cross-process communications
8. **Master Orchestrator Agent** - High-level coordination

## Process Types
- **Approval Workflows**: Purchase requests, authorization, escalation
- **Data Migration**: System migrations, validation, coordination
- **Quality Assurance**: Testing workflows, quality gates, compliance
- **Incident Response**: Severity assessment, escalation, resolution

## Usage

### Basic (no API keys)
```bash
./target/debug/llmspell run examples/script-users/applications/process-orchestrator/main.lua
```

### With configuration
```bash
LLMSPELL_CONFIG=examples/script-users/applications/process-orchestrator/config.toml ./target/debug/llmspell run examples/script-users/applications/process-orchestrator/main.lua
```

### Full features (with API keys)
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
./target/debug/llmspell run examples/script-users/applications/process-orchestrator/main.lua
```

## Expected Output
- Professional process orchestration execution
- 4 different business scenario demonstrations
- Specialized agent coordination
- Workflow routing and execution summaries
- Professional complexity validation

## Progression Context
**Layer 5: Professional Mastery**
- **From**: Business (5 agents, state persistence) 
- **To**: Professional (8 agents, conditional routing)
- **Next**: Expert/Enterprise (12+ agents, complex patterns)

## Technical Architecture

### NEW Conditional Workflow Implementation
The process orchestrator now uses the new table-based conditional API:

```lua
-- Master Orchestration Workflow
:conditional()
:condition({ 
    type = "never"  -- Demo: always takes else_branch (standard path)
    -- ✅ NOW WORKING: type = "shared_data_equals", key = "process_type", value = "INCIDENT"
})

-- THEN branch: Incident handling
:add_then_step({ ... })  

-- ELSE branch: Standard processing  
:add_else_step({ ... })

-- Incident Routing Workflow
:condition({ 
    type = "always"  -- Demo: always takes then_branch (critical path)
    -- ✅ NOW WORKING: type = "shared_data_equals", key = "severity", value = "CRITICAL"
})
```

### Workflow Components
- **Agents**: 8 specialized professional agents
- **Workflows**: 2 conditional workflows (master orchestration + incident routing)
- **Patterns**: Conditional with proper then/else branches
- **Conditions**: Using "always" and "never" for demonstration
- **Tools**: http_request, webhook_caller, file_operations
- **State**: Professional state management with persistence

## Performance Targets
- **Execution Time**: ~3-5 seconds (professional complexity)
- **Process Coverage**: 4 different business process types
- **Agent Utilization**: 8 agents with specialized roles
- **Success Rate**: 100% orchestration completion