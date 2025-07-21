# Lua Examples Testing Plan

**Date**: 2025-07-21  
**Purpose**: Comprehensive testing strategy for Lua examples focusing on agents and workflows

## Overview

We need to verify that all Lua examples in `examples/lua/` work correctly with the current implementation, especially after fixing the Agent.createAsync issues. Since API keys are available in environment variables, we can test real agent execution.

## Testing Categories

### 1. Agent Examples (`examples/lua/agents/`)
- `agent-composition.lua` - Agent composition patterns
- `agent-coordinator.lua` - Multi-agent coordination
- `agent-monitor.lua` - Agent monitoring capabilities
- `agent-orchestrator.lua` - Agent orchestration
- `agent-processor.lua` - Data processing with agents

### 2. Workflow Examples (`examples/lua/workflows/`)
- `workflow-sequential.lua` - Step-by-step execution
- `workflow-conditional.lua` - Conditional branching
- `workflow-loop.lua` - Loop-based workflows
- `workflow-parallel.lua` - Parallel execution
- `workflow-agent-integration.lua` - Agent-workflow integration

### 3. Core Examples
- `test-cli.lua` - CLI functionality testing
- `test-globals.lua` - Global API testing

## Testing Strategy

### Phase 1: Non-LLM Tests (Quick Validation)
First, test examples that don't require actual LLM calls:
- `test-globals.lua` - Tests global APIs
- `test-cli.lua` - Tests CLI integration
- Workflow examples with tool-only steps (no agents)

### Phase 2: Agent Examples (With API Keys)
Test agent examples with real API calls:
- Use timeout protection (30s per example)
- Capture both success and error cases
- Log API usage for monitoring

### Phase 3: Integration Tests
Test complex agent-workflow integrations:
- `workflow-agent-integration.lua`
- Examples that combine multiple agents

## Test Script Design

### 1. Create `run-agent-examples.sh`
```bash
#!/bin/bash
# Test agent-specific examples
# Similar to run-all-tools-examples.sh but for agents
```

### 2. Create `run-workflow-examples.sh`
```bash
#!/bin/bash
# Test workflow examples
# Handle both tool-only and agent-based workflows
```

### 3. Create `run-all-lua-examples.sh`
```bash
#!/bin/bash
# Master script that runs all categories
# Provides comprehensive report
```

## Key Considerations

### 1. API Rate Limiting
- Add delays between agent tests to avoid rate limits
- Use smaller models (gpt-3.5-turbo) for testing
- Limit max_tokens to reduce API usage

### 2. Error Handling
- Examples should handle missing API keys gracefully
- Timeout protection for hanging requests
- Clear error messages for debugging

### 3. Test Isolation
- Each example should be independent
- Clean up any created files/state
- Use test-specific configurations when needed

### 4. Output Validation
- Check for expected output patterns
- Verify no Lua errors occur
- Ensure agents actually execute (not just create)

## Implementation Steps

1. **Create Test Configuration**
   - `llmspell-test.toml` with minimal tokens
   - Test-specific agent prompts

2. **Implement Test Scripts**
   - Modular design for different categories
   - Progress reporting during execution
   - Summary statistics

3. **Add Safety Checks**
   - Verify API keys exist before running
   - Confirm user wants to run tests (API costs)
   - Option to run in "dry-run" mode

4. **Documentation**
   - Expected output for each example
   - Common failure modes and fixes
   - API usage estimates

## Success Criteria

1. **All examples execute without Lua errors**
2. **Agent examples create and execute agents successfully**
3. **Workflow examples complete their defined steps**
4. **Integration examples demonstrate agent-workflow cooperation**
5. **Clear reporting of any failures with actionable fixes**

## Risk Mitigation

1. **API Costs**: Use minimal tokens, add cost warnings
2. **Rate Limits**: Add delays, use retries with backoff
3. **Timeouts**: 30s timeout per example, clear timeout messages
4. **Environment**: Check for required API keys upfront

## Next Steps

1. Review and approve this plan
2. Implement test scripts incrementally
3. Run tests and fix any discovered issues
4. Document results and update examples as needed