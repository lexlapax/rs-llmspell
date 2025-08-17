# Workflow Automation Hub

## Overview

The Workflow Automation Hub is a Blueprint v2.0 compliant application that demonstrates llmspell's advanced workflow orchestration capabilities. It showcases conditional routing, nested workflows, parallel execution, and intelligent error recovery through a sophisticated multi-agent system.

## Architecture

This application implements the complete Blueprint v2.0 architecture:

- **Main Controller**: Conditional workflow that routes requests based on classification
- **Sequential Engine**: Executes workflow steps in order with dependency analysis
- **Dynamic Engine**: Handles complex workflows with nested execution
- **Monitoring Workflow**: Parallel monitoring of system resources
- **Error Handler**: Conditional error recovery with intelligent resolution

## Prerequisites

### Required API Keys
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### System Requirements
- Memory: 512MB minimum
- CPU: 1 core minimum
- Storage: 5GB for workflow artifacts
- llmspell version: 0.8.0+

## Running the Application

### Basic Execution (No API Keys)
```bash
./target/debug/llmspell run examples/script-users/applications/workflow-hub/main.lua
```
This will demonstrate the workflow structure but agents will be inactive.

### With Configuration
```bash
LLMSPELL_CONFIG=examples/script-users/applications/workflow-hub/config.toml \
./target/debug/llmspell run examples/script-users/applications/workflow-hub/main.lua
```

### Full Features (With API Keys)
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
LLMSPELL_CONFIG=examples/script-users/applications/workflow-hub/config.toml \
./target/debug/llmspell run examples/script-users/applications/workflow-hub/main.lua
```

## Components

### LLM Agents (4 Required per Blueprint)

1. **Workflow Optimizer** (GPT-4o-mini)
   - Optimizes execution strategies
   - Routes workflows based on classification
   - Temperature: 0.3 for consistency

2. **Error Resolver** (Claude-3-haiku)
   - Resolves workflow errors
   - Suggests recovery strategies
   - Temperature: 0.2 for reliability

3. **Workflow Generator** (GPT-4o-mini)
   - Creates workflows from requirements
   - Returns structured YAML format
   - Temperature: 0.6 for creativity

4. **Dependency Analyzer** (GPT-3.5-turbo)
   - Analyzes workflow dependencies
   - Determines execution order
   - Temperature: 0.2 for accuracy

### Workflows

#### Main Controller (Conditional)
- Classifies incoming workflow requests
- Routes to monitoring or dynamic execution
- Demonstrates conditional branching

#### Sequential Execution Engine
- Parses workflow definitions
- Analyzes dependencies
- Executes steps in order
- Logs execution results

#### Dynamic Execution Engine (Nested)
- Generates workflows from requirements
- Executes nested sequential workflow
- Executes nested monitoring workflow
- Demonstrates workflow composition

#### Monitoring Workflow (Parallel)
- Monitors system resources
- Checks service health
- Lists running processes
- Executes in parallel for efficiency

#### Error Handler (Conditional)
- Checks for execution errors
- Conditionally resolves errors or logs success
- Demonstrates error recovery patterns

### Tools Used

All tools are real Phase 7 implementations:

- **file_operations**: Read/write workflow definitions and logs
- **json_processor**: Process and query JSON data
- **text_manipulator**: Parse workflow definitions
- **system_monitor**: Get system metrics
- **service_checker**: Check service health
- **process_executor**: List running processes

## Output Files

The application generates several artifacts:

- `/tmp/workflow-definitions.yaml`: Sample workflow definitions
- `/tmp/workflow-logs.json`: Execution logs (if agents active)
- `/tmp/monitoring-report.txt`: Comprehensive execution summary
- `/tmp/error-report.txt`: Error details (if any)

## Sample Output

```
=== Workflow Automation Hub v1.0 ===
Blueprint-compliant workflow orchestration system

1. Creating 4 LLM Agents per blueprint...
  ✅ Workflow Optimizer Agent created
  ✅ Error Resolver Agent created
  ✅ Workflow Generator Agent created
  ✅ Dependency Analyzer Agent created

2. Preparing sample workflow definitions...
  ✅ Created workflow definitions file

3. Creating workflow components...
  ✅ Monitoring Workflow (Parallel) created
  ✅ Sequential Execution Engine created
  ✅ Dynamic Execution Engine (nested workflows) created
  ✅ Error Handler (Conditional) created
  ✅ Main Controller (Conditional) created

4. Executing workflow automation hub...
=============================================================

5. Workflow Hub Results:
=============================================================
  ✅ Hub Status: COMPLETED
  ⏱️  Total Execution Time: 250ms
  🏗️  Architecture: Blueprint v2.0 Compliant

  📊 Components Executed:
    • Main Controller: Conditional routing
    • Dynamic Engine: Nested workflow execution
    • Sequential Engine: Step-by-step processing
    • Monitoring: Parallel resource checks
    • Error Handler: Conditional error resolution
```

## Configuration

The `config.toml` file provides extensive configuration options:

### Provider Configuration
- Primary: OpenAI for optimization and generation
- Secondary: Anthropic for error resolution
- Configurable timeouts and models

### Workflow Settings
- Conditional routing configuration
- Sequential execution parameters
- Parallel execution limits
- Error handling strategies

### Monitoring
- Resource monitoring intervals
- Metrics tracking
- Performance thresholds

### Security
- Input validation
- Output sanitization
- Rate limiting
- Path restrictions

## Cost Considerations

This application uses real LLM APIs which incur costs:

- **Workflow Optimizer**: ~$0.002 per classification
- **Error Resolver**: ~$0.001 per resolution
- **Workflow Generator**: ~$0.003 per generation
- **Dependency Analyzer**: ~$0.001 per analysis

Estimated cost per full execution: $0.01-$0.05

## Troubleshooting

### No API Keys
If running without API keys, agents will show as "Inactive (no API key)" but the workflow structure will still execute using fallback tools.

### API Errors
Check the `/tmp/workflow-hub.log` file for detailed error messages. Common issues:
- Invalid API keys
- Rate limiting
- Network connectivity

### Performance Issues
- Reduce parallel execution limits in config
- Enable caching for repeated operations
- Use simpler models for non-critical tasks

## Blueprint Compliance

This application achieves 100% Blueprint v2.0 compliance:

✅ Main Controller with conditional routing
✅ 4 specialized LLM agents
✅ Nested workflow support
✅ Parallel execution capabilities
✅ Conditional error handling
✅ Real Phase 7 tools only
✅ State persistence ready
✅ Event system compatible
✅ Production-grade architecture

## Future Enhancements

1. **Visual Workflow Builder**: Web UI for creating workflows
2. **Workflow Templates**: Pre-built workflows for common tasks
3. **Advanced Monitoring**: Real-time execution visualization
4. **Cost Optimization**: Automatic model selection based on task
5. **Workflow Marketplace**: Share and discover workflows

## License

This example is part of the llmspell project and follows the same license terms.