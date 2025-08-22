# Code Review Assistant - Level 6: ADVANCED

**Real-World Application**: DevOps automation and CI/CD code quality assurance (2025 trend)  
**Complexity**: ⭐⭐⭐⭐☆  
**Est. Runtime**: 30-60 seconds | **API Cost**: ~$0.02-0.05

## Overview

The Code Review Assistant automates comprehensive code reviews using 7 specialized AI agents, addressing the growing need for automated quality assurance in modern CI/CD pipelines. This application demonstrates how multiple agents can work sequentially to analyze code from different perspectives, providing actionable feedback similar to enterprise tools like GitHub Copilot for PRs and AWS CodeGuru.

## Features Demonstrated

### llmspell Crates Showcased
- `llmspell-agents`: 7 specialized review agents with distinct prompts
- `llmspell-workflows`: Sequential workflow with state management
- `llmspell-tools`: File operations for reading/writing code and reports
- `llmspell-bridge`: Lua script integration with async agent execution
- `llmspell-state-persistence`: Output collection via workflow state

### Progressive Complexity
| Aspect | Implementation | New in This Level |
|--------|---------------|-------------------|
| Agents | 7 specialized reviewers | Multi-agent orchestration |
| Workflow | Sequential with state | State-based output collection |
| Providers | OpenAI + Anthropic | Multi-provider coordination |
| Output | JSON + Markdown + Text | Structured multi-format output |
| Tools | File operations | Integrated tool usage |

### Agent Specializations
- **Security Reviewer** (GPT-4o-mini): Authentication, injection, crypto vulnerabilities
- **Quality Reviewer** (Claude-3-Haiku): Maintainability, readability, error handling
- **Performance Reviewer** (GPT-4o-mini): Algorithms, resource management
- **Best Practices Reviewer** (GPT-4o-mini): SOLID principles, design patterns
- **Dependency Reviewer** (GPT-3.5-Turbo): Architecture, coupling issues
- **Fix Generator** (Claude-3-Haiku): Actionable code fixes
- **Report Writer** (GPT-4o-mini): Comprehensive markdown reports

## Quick Start

### Prerequisites
- llmspell built and available (`cargo build --release`)
- API Keys: `OPENAI_API_KEY` and/or `ANTHROPIC_API_KEY`
- Config: `config.toml` for file system permissions

### 1. Basic Demo Mode
```bash
# Creates sample code with issues and runs review
./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua
```

### 2. With Custom Code Input
```bash
# Review your own code
./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua \
  -- --input my-code.lua --output /tmp/my-review
```

### 3. With Configuration
```bash
# Use configuration file for permissions
./target/debug/llmspell -c examples/script-users/applications/code-review-assistant/config.toml \
  run examples/script-users/applications/code-review-assistant/main.lua
```

## Configuration Structure

The application uses a comprehensive configuration system:

```lua
local config = {
    system_name = "code_review_assistant_v2",
    
    -- Model assignments for each reviewer
    models = {
        security_reviewer = "gpt-4o-mini",
        quality_reviewer = "claude-3-haiku-20240307",
        practices_reviewer = "gpt-4o-mini",
        performance_reviewer = "gpt-4o-mini",
        dependencies_reviewer = "gpt-3.5-turbo",
        fix_generator = "claude-3-haiku-20240307",
        report_writer = "gpt-4o-mini"
    },
    
    -- Provider configuration
    providers = {
        quality_reviewer = "anthropic",
        fix_generator = "anthropic"
    },
    
    -- File paths
    files = {
        code_directory = "/tmp/code-to-review",
        findings_output = "/tmp/review-findings.json",
        report_output = "/tmp/review-report.md",
        fixes_output = "/tmp/suggested-fixes.json",
        summary_output = "/tmp/review-summary.txt"
    },
    
    -- Review settings
    review_settings = {
        max_files_to_review = 10,
        severity_levels = {"critical", "high", "medium", "low", "info"},
        auto_fix_threshold = "medium"
    }
}
```

## Architecture

```
Code Review Workflow (Sequential)
├── Agent Creation Phase
│   ├── Security Reviewer (GPT-4o-mini)
│   ├── Quality Reviewer (Claude-3-Haiku)
│   ├── Performance Reviewer (GPT-4o-mini)
│   ├── Best Practices Reviewer (GPT-4o-mini)
│   ├── Dependency Reviewer (GPT-3.5-Turbo)
│   ├── Fix Generator (Claude-3-Haiku)
│   └── Report Writer (GPT-4o-mini)
│
├── File Processing Phase
│   ├── Load Code Input (code-input.lua)
│   └── Prepare Code Content for Review
│
├── Sequential Review Execution
│   ├── Step 1: Security Review → State[:workflow:ID:agent:security:output]
│   ├── Step 2: Quality Review → State[:workflow:ID:agent:quality:output]
│   ├── Step 3: Performance Review → State[:workflow:ID:agent:performance:output]
│   ├── Step 4: Best Practices Review → State[:workflow:ID:agent:practices:output]
│   └── Step 5: Dependencies Review → State[:workflow:ID:agent:dependencies:output]
│
└── Output Generation Phase
    ├── Collect All State Outputs
    ├── Generate Fixes and Report
    ├── Save review-findings.json
    ├── Save review-report.md
    └── Save review-summary.txt
```

## Learning Path

### Prerequisites
- **Complete Apps 01-05**: Basic agents, workflows, and tools
- **Understand**: Sequential workflows, state management, multi-agent coordination

### You'll Learn
- Multi-agent orchestration with specialized roles
- State-based output collection from workflows
- Multi-provider coordination (OpenAI + Anthropic)
- Structured output generation in multiple formats
- Error handling in sequential workflows

### Next Step
- **App 07 (document-intelligence)**: Adds Composite Agents and all 4 workflow types
- **Enhancement Path**: Convert to parallel workflow for 5x speed improvement

## Sample Issues Detected

The demo mode creates files with various issues to demonstrate capabilities:

### JavaScript (auth.js)
- **Security**: Hardcoded secret keys, weak MD5 hashing, no rate limiting
- **Quality**: No input validation, console logging of sensitive data
- **Practices**: Direct object mutation, missing error handling

### Python (data_processor.py)
- **Security**: No path validation, unsafe file operations
- **Quality**: Ambiguous state, magic numbers, inconsistent return types
- **Performance**: Inefficient nested loops, multiple data passes
- **Practices**: God methods, modifying input parameters

### Go (api_handler.go)
- **Security**: SQL injection, no authentication, race conditions
- **Quality**: Ignored errors, no request validation
- **Performance**: String concatenation in loops, resource leaks
- **Practices**: Global variables, missing mutex for concurrent access

## Sample Output

### Generated Files
| File | Description | Size |
|------|-------------|------|
| `review-findings.json` | All issues found by reviewers | ~5-10KB |
| `review-report.md` | Markdown summary report | ~3-5KB |
| `review-summary.txt` | Quick text summary | ~500B |

### Example Finding (review-findings.json)
```json
{
  "auth.js": {
    "security": [
      {
        "severity": "critical",
        "issue": "Hardcoded secret key",
        "line": 5,
        "fix": "Use environment variables for secrets"
      }
    ],
    "quality": [
      {
        "severity": "high",
        "issue": "No input validation",
        "line": 12,
        "fix": "Add validation for user inputs"
      }
    ]
  }
}
```

## Execution Flow

1. **Initialization**: Creates specialized AI agents with specific prompts
2. **File Preparation**: Creates demo files or reads from specified directory
3. **Workflow Creation**: Builds sequential workflow with review steps
4. **Review Execution**: Processes each file through all reviewers
5. **Result Collection**: Gathers outputs from workflow execution
6. **Report Generation**: Creates summary and saves output files

## Cost Considerations

**Warning**: Real API usage incurs costs:

- **Per-file review cost**: ~$0.005-$0.01
- **Report generation**: ~$0.002
- **Typical run (3 files)**: ~$0.02-$0.03

To minimize costs:
- Limit `max_files_to_review` in configuration
- Use GPT-3.5-Turbo for non-critical reviews
- Test with demo mode before production use

## Current Status

### Working Features
- ✅ All 7 reviewers generate meaningful feedback
- ✅ Code content properly passed to agents
- ✅ State-based output collection
- ✅ Multi-format output generation
- ✅ Error handling and recovery

### Enhancement Opportunities
- Convert to parallel workflow (5x speed improvement)
- Add Hook system for critical issues
- Integrate more tools (web_search for CVEs, webhooks for CI/CD)
- Add session persistence for review history

## Troubleshooting

### "Tool execution failed" Errors
- Ensure output directories are writable
- Check file permissions for code directory

### Empty Review Results
- Verify API keys are set correctly
- Check network connectivity to AI providers

### Workflow Execution Issues
- Review log output for specific step failures
- Ensure all required agents are created successfully

## Related Applications

### Progressive Learning Path
1. **Apps 01-02**: Foundation - Basic agents and tools
2. **Apps 03-05**: Business Ready - Sessions, hooks, events
3. **App 06** (This): Advanced - Multi-agent orchestration
4. **Apps 07-08**: Expert techniques - Composite agents, meta-workflows
5. **Apps 09-10**: Production patterns - Dynamic workflows, 20+ agents

### Similar Applications
- **App 10 (webapp-creator)**: 20-agent orchestration for full-stack generation
- **App 05 (content-creator)**: Parallel workflow for content generation
- **App 07 (document-intelligence)**: Composite agents for document analysis

## Version History

- **v3.0.0**: Current - Standardized header, proper code passing, state collection
- **v2.0.0**: Sequential workflow implementation
- **v1.0.0**: Initial parallel workflow attempt