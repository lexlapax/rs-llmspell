# Code Review Assistant

An automated code review system with security scanning, quality analysis, and improvement suggestions using llmspell's loop workflows and parallel multi-aspect review.

## Overview

The Code Review Assistant demonstrates:
- **4-Phase Architecture**: Analysis → Review → Aggregation → Report
- **Loop Workflow**: Iterates through multiple files for review
- **Parallel Sub-workflows**: 4 simultaneous review aspects per file
- **7 Specialized Agents**: Security, quality, practices, performance reviewers + processors
- **Blueprint v2.0 Compliant**: Production-grade code review patterns

## Prerequisites

### Required
- llmspell built and available (`cargo build --release`)
- At least one of:
  - OpenAI API key: `export OPENAI_API_KEY="sk-..."`
  - Anthropic API key: `export ANTHROPIC_API_KEY="sk-ant-..."`

### Optional
- Both API keys for multi-provider functionality
- GitHub integration for PR comments (future enhancement)

## Quick Start

### 1. Basic Execution (No API Keys)
```bash
# Runs with simulated agents and sample code analysis
./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua
```

### 2. With Configuration File
```bash
# Uses the provided config.toml for provider settings
LLMSPELL_CONFIG=examples/script-users/applications/code-review-assistant/config.toml \
  ./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua
```

### 3. Full Production Mode
```bash
# Set API keys for real code analysis
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Run with full capabilities
./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua
```

## Architecture

### Workflow Structure

```
Main Review Workflow (Sequential)
├── Phase 1: Code Analysis (Parallel)
│   ├── Load Code Files (Tool)
│   ├── Parse Structure (Tool)
│   └── Check Syntax (Tool)
│
├── Phase 2: Review Process (Loop - iterates 3 files)
│   └── For each file:
│       └── File Review (Parallel Sub-workflow)
│           ├── Security Review (Agent)
│           ├── Quality Review (Agent)
│           ├── Practices Review (Agent)
│           └── Performance Review (Agent)
│
├── Phase 3: Issue Aggregation (Sequential)
│   ├── Deduplicate Findings (Tool)
│   ├── Prioritize Issues (Agent)
│   └── Generate Fixes (Agent)
│
└── Phase 4: Report Generation (Sequential)
    ├── Create Report (Agent)
    ├── Format PR Comment (Tool)
    └── Save Reports (Tool)
```

### Agents

| Agent | Model | Purpose | Temperature |
|-------|-------|---------|-------------|
| **Security Reviewer** | GPT-4o-mini | Vulnerability detection (SQL injection, XSS, etc.) | 0.2 |
| **Quality Reviewer** | Claude-3-haiku | Code quality and maintainability analysis | 0.3 |
| **Practices Reviewer** | GPT-4o-mini | Best practices and conventions compliance | 0.3 |
| **Performance Reviewer** | GPT-3.5-turbo | Performance bottleneck identification | 0.4 |
| **Issue Prioritizer** | GPT-4o-mini | Severity ranking and impact assessment | 0.2 |
| **Fix Generator** | Claude-3-haiku | Code fix suggestions for issues | 0.3 |
| **Report Writer** | GPT-4o-mini | Comprehensive review report creation | 0.4 |

### Tools

- **file_operations**: Code file loading and report saving
- **text_manipulator**: Report formatting and text processing
- **json_processor**: Issue deduplication and data merging
- **code_analyzer**: Code structure parsing (simulated)
- **syntax_validator**: Syntax checking (simulated)

## Sample Code Issues

The assistant creates sample files with various issues to demonstrate detection capabilities:

### Security Issues
- SQL injection vulnerabilities
- Command injection risks
- Hardcoded API keys and secrets
- Missing input validation
- Plain text password storage

### Quality Issues
- High cyclomatic complexity
- Missing error handling
- Magic numbers without explanation
- Code duplication
- Ignored errors

### Best Practice Violations
- Use of `eval()` function
- Global variable usage
- Missing resource cleanup
- No request validation
- Improper naming conventions

### Performance Problems
- Synchronous blocking I/O
- O(n²) algorithms
- Repeated DOM access in loops
- Resource leaks
- Missing concurrent access control

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
        security_reviewer = "openai/gpt-4",  -- Use GPT-4 for better security analysis
        quality_reviewer = "anthropic/claude-3-opus",  -- Premium quality review
    },
    review_settings = {
        max_files_to_review = 10,  -- Review more files
        auto_fix_threshold = "high"  -- Only fix critical issues
    }
}
```

## Output Files

| File | Description |
|------|-------------|
| `/tmp/code-to-review/*.js/py/go` | Sample code files with issues |
| `/tmp/review-findings.json` | Raw findings from all reviewers |
| `/tmp/review-report.md` | Comprehensive markdown report |
| `/tmp/pr-comment.md` | Formatted GitHub PR comment |
| `/tmp/review-summary.txt` | Execution summary and metrics |

## Performance Metrics

Typical execution times:

- **Code Analysis** (Parallel): ~100ms
- **Review Process** (Loop × 3 files): ~150ms
  - Each file review (Parallel): ~50ms
- **Issue Aggregation** (Sequential): ~75ms
- **Report Generation** (Sequential): ~75ms
- **Total Review Time**: ~400ms

## Loop Workflow Implementation

The review process demonstrates loop workflows:

```lua
local review_process = Workflow.builder()
    :name("review_process")
    :loop_workflow()
    :max_iterations(3)  -- Review 3 files
    :add_step({
        name = "review_file",
        type = "workflow",
        workflow = file_review_workflow  -- Nested parallel workflow
    })
    :build()
```

Each iteration runs a parallel sub-workflow with 4 simultaneous reviewers.

## Parallel Review Benefits

### Per-File Analysis
- 4 aspects reviewed simultaneously
- Independent agent processing
- Faster than sequential review
- Comprehensive coverage

### Overall Performance
- Loop enables scalable file processing
- Parallel analysis reduces total time
- Nested workflows provide modularity

## Cost Considerations

**Warning**: Real API usage incurs costs:

- **Security/Quality Review**: ~$0.002 per file
- **Fix Generation**: ~$0.001 per issue
- **Report Generation**: ~$0.002 per report
- **Typical run cost**: $0.01 - $0.02 per repository

To minimize costs:
1. Limit `max_files_to_review`
2. Use cheaper models for non-critical reviews
3. Cache results for unchanged files

## Issue Prioritization

Issues are categorized by severity:

1. **CRITICAL**: Security vulnerabilities requiring immediate fix
2. **HIGH**: Significant bugs or security risks
3. **MEDIUM**: Quality issues affecting maintainability
4. **LOW**: Best practice violations
5. **INFO**: Style suggestions and improvements

## Troubleshooting

### "Agent needs API key" Messages
- Review continues with basic analysis
- Set environment variables for AI-powered features

### Loop Workflow Issues
- Check `max_iterations` configuration
- Verify file count doesn't exceed limits

### Parallel Execution Problems
- Ensure sufficient system resources
- Check for timeout issues with large files

## Blueprint Compliance

✅ 4-Phase Sequential Architecture
✅ Loop workflow for file iteration
✅ Parallel sub-workflows for multi-aspect review
✅ 7 specialized agents as required
✅ Issue aggregation and prioritization
✅ Comprehensive report generation
✅ Production error handling

## Example Use Cases

1. **Pull Request Reviews**: Automated code review for PRs
2. **Security Audits**: Vulnerability scanning across codebases
3. **Code Quality Gates**: Enforce quality standards in CI/CD
4. **Legacy Code Analysis**: Assess technical debt in old code
5. **Compliance Checking**: Verify adherence to coding standards

## Extending the System

1. **Add More Reviewers**: Documentation, testing, accessibility
2. **Language Support**: Add language-specific analyzers
3. **IDE Integration**: Real-time review in development
4. **Custom Rules**: Company-specific coding standards
5. **Fix Application**: Automatic fix application with approval

## Integration Points

### GitHub Integration
```bash
# Future: Direct PR comment posting
gh pr comment 123 --body-file /tmp/pr-comment.md
```

### CI/CD Pipeline
```yaml
# Example GitHub Actions integration
- name: Code Review
  run: |
    llmspell run code-review-assistant/main.lua
    cat /tmp/review-summary.txt
```

### Custom Analyzers
The system uses simulated analyzers but can integrate real tools:
- ESLint for JavaScript
- Pylint for Python
- golangci-lint for Go
- SonarQube for comprehensive analysis

## Review Report Structure

Generated reports include:

1. **Executive Summary**: High-level findings and metrics
2. **Critical Issues**: Security vulnerabilities and bugs
3. **Code Quality**: Maintainability and complexity analysis
4. **Performance Analysis**: Bottlenecks and optimizations
5. **Best Practices**: Convention compliance
6. **Suggested Fixes**: Actionable code improvements
7. **Metrics**: Coverage, complexity, and quality scores

## Related Examples

- **Customer Support System**: Conditional routing patterns
- **Data Pipeline**: Loop workflow techniques
- **Content Generation Platform**: Multi-agent collaboration
- **Workflow Examples**: Basic patterns in `examples/lua/workflows/`

## Support

For issues or questions:
- Review the main llmspell documentation
- Check blueprint.md for architectural patterns
- See examples/script-users/getting-started/ for basics