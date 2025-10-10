-- ============================================================
-- LLMSPELL APPLICATION SHOWCASE
-- ============================================================
-- Application ID: 06 - Code Review Assistant v3.0.0
-- Complexity Level: 3 [ADVANCED]
-- Real-World Use Case: DevOps automation and code quality assurance (2025 CI/CD trend)
-- 
-- Purpose: Automated multi-aspect code review with 7 specialized AI reviewers
-- Architecture: Sequential workflow with state-based output collection
-- Crates Showcased: llmspell-agents, llmspell-workflows, llmspell-tools, llmspell-bridge
-- Key Features:
--   • 7 specialized review agents (security, quality, performance, etc.)
--   • Sequential workflow for comprehensive analysis
--   • Multi-provider support (OpenAI + Anthropic)
--   • Structured JSON output with actionable fixes
--   • State-based result aggregation
--
-- Prerequisites:
--   • API Keys: OPENAI_API_KEY and/or ANTHROPIC_API_KEY
--   • Config: config.toml for file system permissions
--   • Resources: Code files to review or uses demo mode
--
-- HOW TO RUN:
-- 1. Basic Demo (creates sample code with issues):
--    ./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua
--
-- 2. With Custom Code Input:
--    ./target/debug/llmspell run examples/script-users/applications/code-review-assistant/main.lua \
--    -- --input my-code.lua --output /tmp/my-review
--
-- 3. With Configuration:
--    ./target/debug/llmspell -c examples/script-users/applications/code-review-assistant/config.toml \
--    run examples/script-users/applications/code-review-assistant/main.lua
--
-- Expected Output:
--   • review-findings.json - All issues found by reviewers
--   • review-report.md - Markdown summary report
--   • review-summary.txt - Quick text summary
--   • Runtime: ~30-60 seconds | API Cost: ~$0.02-0.05
--
-- Progressive Learning:
--   • Previous: App 05 (content-creator) introduced parallel workflows
--   • This App: Adds sequential multi-agent review pattern
--   • Next: App 07 (document-intelligence) will add Composite Agents
-- ============================================================

print("=== Code Review Assistant v3.0 ===")
print("Application 06: ADVANCED - Multi-aspect AI code review system")
print("Showcasing: Sequential workflows with 7 specialized agents\n")

-- ============================================================
-- Configuration
-- ============================================================

local json = JSON  -- Global JSON provided by llmspell

-- Load code to review (like webapp-creator loads user requirements)
local code_input_file = ARGS and ARGS.input or "code-input.lua"
local code_input_path = "examples/script-users/applications/code-review-assistant/" .. code_input_file

print("📂 Loading code input from: " .. code_input_file)

-- Load the code samples to review
local code_samples = dofile(code_input_path)
if not code_samples then
    error("Failed to load code input file: " .. code_input_path)
end

print("  ✓ Loaded " .. #code_samples .. " code files to review\n")

-- Configuration
local config = {
    system_name = "code_review_assistant_v3",
    models = {
        security = "gpt-4o-mini",
        quality = "claude-3-haiku-20240307", 
        performance = "gpt-4o-mini",
        practices = "gpt-4o-mini",
        dependencies = "gpt-3.5-turbo",
        fix_generator = "claude-3-haiku-20240307",
        report_writer = "gpt-4o-mini"
    },
    providers = {
        quality = "anthropic",
        fix_generator = "anthropic"
    },
    output_dir = ARGS and ARGS.output or "/tmp/code-review-output"
}

-- ============================================================
-- Create Specialized Review Agents
-- ============================================================

print("🤖 Creating specialized review agents...\n")

local timestamp = os.time()
local agents = {}

-- 1. Security Reviewer
agents.security = Agent.builder()
    :name("security_reviewer_" .. timestamp)
    :type("llm")
    :model(config.models.security)
    :temperature(0.2)
    :system_prompt([[You are a security expert specializing in code vulnerability analysis.
Review the provided code for security issues including:
- Authentication and authorization flaws
- Injection vulnerabilities (SQL, command, XSS)
- Sensitive data exposure
- Cryptographic weaknesses
- Insecure configurations
- Race conditions and concurrency issues

Output a JSON object with this exact format:
{
    "issues": [
        {
            "severity": "critical|high|medium|low",
            "type": "security",
            "line": <line_number_if_identifiable>,
            "description": "detailed description of the issue",
            "recommendation": "specific fix recommendation"
        }
    ],
    "summary": "brief overall security assessment"
}]])
    :build()
print("  1. Security Reviewer: ✓")

-- 2. Code Quality Reviewer
agents.quality = Agent.builder()
    :name("quality_reviewer_" .. timestamp)
    :type("llm")
    :provider(config.providers.quality)
    :model(config.models.quality)
    :temperature(0.3)
    :system_prompt([[You are a code quality expert focusing on maintainability and readability.
Review the provided code for quality issues including:
- Code complexity and readability
- Error handling
- Magic numbers and hardcoded values
- Code duplication
- Naming conventions
- Documentation and comments

Output a JSON object with this exact format:
{
    "issues": [
        {
            "severity": "high|medium|low",
            "type": "quality",
            "description": "detailed description",
            "suggestion": "improvement suggestion"
        }
    ],
    "metrics": {
        "complexity": "high|medium|low",
        "readability": 7,
        "maintainability": 6
    }
}]])
    :build()
print("  2. Code Quality Reviewer: ✓")

-- 3. Performance Reviewer
agents.performance = Agent.builder()
    :name("performance_reviewer_" .. timestamp)
    :type("llm")
    :model(config.models.performance)
    :temperature(0.2)
    :system_prompt([[You are a performance optimization expert.
Review the provided code for performance issues including:
- Inefficient algorithms (O(n²) or worse)
- Memory leaks and excessive allocations
- Unnecessary loops and iterations
- Database query optimization
- Caching opportunities
- Resource management

Output a JSON object with this exact format:
{
    "issues": [
        {
            "severity": "high|medium|low",
            "type": "performance",
            "description": "detailed description",
            "impact": "performance impact",
            "optimization": "suggested optimization"
        }
    ],
    "recommendations": ["general performance improvement 1", "improvement 2"]
}]])
    :build()
print("  3. Performance Reviewer: ✓")

-- 4. Best Practices Reviewer
agents.practices = Agent.builder()
    :name("practices_reviewer_" .. timestamp)
    :type("llm")
    :model(config.models.practices)
    :temperature(0.3)
    :system_prompt([[You are a software engineering best practices expert.
Review the provided code for violations of best practices including:
- SOLID principles violations
- Design pattern misuse
- Anti-patterns
- Code organization issues
- Testing considerations
- Documentation standards

Output a JSON object with this exact format:
{
    "violations": [
        {
            "principle": "principle or pattern violated",
            "description": "detailed description",
            "recommendation": "how to improve"
        }
    ],
    "suggestions": ["best practice improvement 1", "improvement 2"]
}]])
    :build()
print("  4. Best Practices Reviewer: ✓")

-- 5. Dependency Reviewer  
agents.dependencies = Agent.builder()
    :name("dependency_reviewer_" .. timestamp)
    :type("llm")
    :model(config.models.dependencies)
    :temperature(0.2)
    :system_prompt([[You are a dependency and architecture expert.
Review the provided code for dependency issues including:
- Outdated or vulnerable dependencies
- Unnecessary dependencies
- Circular dependencies
- Tight coupling
- Missing abstractions

Output a JSON object with this exact format:
{
    "issues": [
        {
            "type": "dependency",
            "description": "detailed description",
            "recommendation": "suggested improvement"
        }
    ],
    "architecture_notes": "overall architecture assessment"
}]])
    :build()
print("  5. Dependency Reviewer: ✓")

-- 6. Fix Generator
agents.fix_generator = Agent.builder()
    :name("fix_generator_" .. timestamp)
    :type("llm")
    :provider(config.providers.fix_generator)
    :model(config.models.fix_generator)
    :temperature(0.4)
    :system_prompt([[You are a code fixing expert.
Given code and identified issues, generate specific fixes.

Output a JSON object with this exact format:
{
    "fixes": [
        {
            "issue": "issue being fixed",
            "original_code": "problematic snippet",
            "fixed_code": "corrected snippet",
            "explanation": "what was changed and why"
        }
    ]
}]])
    :build()
print("  6. Fix Generator: ✓")

-- 7. Report Writer
agents.report_writer = Agent.builder()
    :name("report_writer_" .. timestamp)
    :type("llm")
    :model(config.models.report_writer)
    :temperature(0.3)
    :system_prompt([[You are a technical report writer.
Create a comprehensive code review report from the provided review results.

The report should be in Markdown format and include:
1. Executive Summary
2. Critical Issues (security and high-severity)
3. Code Quality Assessment
4. Performance Considerations
5. Recommendations (prioritized)
6. Detailed Findings by Category

Make it actionable and professional.]])
    :build()
print("  7. Report Writer: ✓")

print("\n✅ All agents created successfully\n")

-- ============================================================
-- Process Each Code File
-- ============================================================

print("🔍 Starting code review process...\n")

local all_reviews = {}
local all_issues = {}

-- Process each code sample
for file_idx, code_sample in ipairs(code_samples) do
    print(string.format("📄 Reviewing file %d/%d: %s", 
        file_idx, #code_samples, code_sample.filename))
    
    -- Create a workflow for this specific file review
    -- (Following webapp-creator pattern of creating workflows dynamically)
    local file_workflow = Workflow.builder()
        :name("review_" .. code_sample.filename:gsub("%.", "_"))
        :description("Review workflow for " .. code_sample.filename)
        :timeout_ms(300000)
        :sequential()  -- Sequential to ensure each reviewer completes
    
    -- Add review steps - CRITICAL: Pass the actual code content!
    local review_prompts = {
        {
            step = "security_review",
            agent = "security_reviewer_" .. timestamp,
            prompt = string.format(
                "Review the following %s code for security issues:\n\nFile: %s\n\n%s",
                code_sample.language, code_sample.filename, code_sample.code
            )
        },
        {
            step = "quality_review", 
            agent = "quality_reviewer_" .. timestamp,
            prompt = string.format(
                "Review the following %s code for quality issues:\n\nFile: %s\n\n%s",
                code_sample.language, code_sample.filename, code_sample.code
            )
        },
        {
            step = "performance_review",
            agent = "performance_reviewer_" .. timestamp,
            prompt = string.format(
                "Review the following %s code for performance issues:\n\nFile: %s\n\n%s",
                code_sample.language, code_sample.filename, code_sample.code
            )
        },
        {
            step = "practices_review",
            agent = "practices_reviewer_" .. timestamp,
            prompt = string.format(
                "Review the following %s code for best practices violations:\n\nFile: %s\n\n%s",
                code_sample.language, code_sample.filename, code_sample.code
            )
        },
        {
            step = "dependencies_review",
            agent = "dependency_reviewer_" .. timestamp,
            prompt = string.format(
                "Review the following %s code for dependency issues:\n\nFile: %s\n\n%s",
                code_sample.language, code_sample.filename, code_sample.code
            )
        }
    }
    
    -- Add each review step to the workflow
    for _, review in ipairs(review_prompts) do
        file_workflow:add_step({
            name = review.step,
            type = "agent",
            agent = review.agent,
            input = review.prompt  -- The actual code is in the prompt!
        })
    end
    
    -- Build and execute the workflow
    file_workflow = file_workflow:build()
    
    -- Execute with minimal input (the code is already in each step)
    local workflow_input = {
        text = "Begin review",
        context = {
            filename = code_sample.filename,
            language = code_sample.language
        }
    }
    
    print("  ⏳ Executing review workflow...")
    local result = file_workflow:execute(workflow_input)
    
    -- Don't check result.success - just collect outputs from State like webapp-creator
    print("  ✓ Review workflow executed")
    
    -- Collect the review results from State
    local file_review = {
        filename = code_sample.filename,
        language = code_sample.language,
        reviews = {}
    }
    
    -- Get agent outputs from workflow metadata (automatically collected)
    local outputs = result.metadata and result.metadata.extra
        and result.metadata.extra.agent_outputs or {}

    file_review.reviews = {
        security = outputs["security_reviewer_" .. timestamp] or "",
        quality = outputs["quality_reviewer_" .. timestamp] or "",
        performance = outputs["performance_reviewer_" .. timestamp] or "",
        practices = outputs["practices_reviewer_" .. timestamp] or "",
        dependencies = outputs["dependency_reviewer_" .. timestamp] or ""
    }
    
    table.insert(all_reviews, file_review)
    
    -- Aggregate issues for fix generation
    table.insert(all_issues, {
        file = code_sample.filename,
        code = code_sample.code,
        review_outputs = file_review.reviews
    })
    
    print("")
end

-- ============================================================
-- Generate Fixes for Critical Issues
-- ============================================================

print("🔧 Generating fixes for identified issues...\n")

-- Create a workflow to generate fixes
local fix_workflow = Workflow.builder()
    :name("fix_generation")
    :description("Generate fixes for critical issues")
    :timeout_ms(120000)
    :sequential()

-- Add fix generation step with aggregated issues
local issues_summary = "Based on the code reviews, generate fixes for the following critical issues:\n\n"
for _, issue_data in ipairs(all_issues) do
    issues_summary = issues_summary .. "File: " .. issue_data.file .. "\n"
    issues_summary = issues_summary .. "Review Output: " .. tostring(issue_data.review_output) .. "\n\n"
end

fix_workflow:add_step({
    name = "generate_fixes",
    type = "agent",
    agent = "fix_generator_" .. timestamp,
    input = issues_summary
})

fix_workflow = fix_workflow:build()
local fix_result = fix_workflow:execute({text = "Generate fixes"})

-- Get fix output from workflow metadata
local fix_outputs = fix_result.metadata and fix_result.metadata.extra
    and fix_result.metadata.extra.agent_outputs or {}
local generated_fixes = fix_outputs["fix_generator_" .. timestamp] or ""

if generated_fixes ~= "" then
    print("  ✓ Fixes generated")
else
    print("  ⚠ No fixes generated")
end

-- ============================================================
-- Generate Final Report
-- ============================================================

print("📝 Generating comprehensive report...\n")

-- Prepare report input
local report_input = string.format([[
Generate a comprehensive code review report based on the following reviews:

Total Files Reviewed: %d

Review Results:
%s

Generated Fixes:
%s

Please create a professional markdown report with actionable recommendations.
]], #code_samples, json.stringify(all_reviews), generated_fixes)

-- Create report generation workflow
local report_workflow = Workflow.builder()
    :name("report_generation")
    :description("Generate final review report")
    :timeout_ms(60000)
    :sequential()

report_workflow:add_step({
    name = "write_report",
    type = "agent",
    agent = "report_writer_" .. timestamp,
    input = report_input
})

report_workflow = report_workflow:build()
local report_result = report_workflow:execute({text = "Generate report"})

-- Get report output from workflow metadata
local report_outputs = report_result.metadata and report_result.metadata.extra
    and report_result.metadata.extra.agent_outputs or {}
local report_output = report_outputs["report_writer_" .. timestamp]

local final_report = "# Code Review Report\n\n"
if report_output and report_output ~= "" then
    print("  ✓ Report generated")
    final_report = report_output
else
    print("  ⚠ Using basic report format")
    -- Fallback to basic report
    final_report = final_report .. "## Summary\n\n"
    final_report = final_report .. string.format("- Files Reviewed: %d\n", #code_samples)
    final_report = final_report .. string.format("- Review Date: %s\n\n", os.date("%Y-%m-%d %H:%M:%S"))
    final_report = final_report .. "## Files Reviewed\n\n"
    for _, sample in ipairs(code_samples) do
        final_report = final_report .. string.format("- %s (%s)\n", sample.filename, sample.language)
    end
end

-- ============================================================
-- Save Outputs
-- ============================================================

print("\n💾 Saving review outputs...\n")

-- Save detailed findings
local findings_path = config.output_dir .. "/review-findings.json"
Tool.execute("file-operations", {
    operation = "write",
    path = findings_path,
    input = json.stringify({
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        files_reviewed = #code_samples,
        reviews = all_reviews,
        fixes = generated_fixes
    })
})
print("  ✓ Findings saved to: " .. findings_path)

-- Save markdown report
local report_path = config.output_dir .. "/review-report.md"
Tool.execute("file-operations", {
    operation = "write",
    path = report_path,
    input = final_report
})
print("  ✓ Report saved to: " .. report_path)

-- Save summary
local summary_path = config.output_dir .. "/review-summary.txt"
local summary = string.format([[
Code Review Complete!
====================
System: %s
Files Reviewed: %d
Output Directory: %s

Generated Files:
- review-findings.json: Detailed review findings
- review-report.md: Comprehensive markdown report
- review-summary.txt: This summary

Review Date: %s
]], config.system_name, #code_samples, config.output_dir, os.date("%Y-%m-%d %H:%M:%S"))

Tool.execute("file-operations", {
    operation = "write",
    path = summary_path,
    input = summary
})
print("  ✓ Summary saved to: " .. summary_path)

print("\n" .. summary)
print("✅ Code review completed successfully!")